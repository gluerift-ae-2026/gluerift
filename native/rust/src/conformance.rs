use crate::canonical;
use crate::model::{BoundsLogicalCarrier, DecisionLogicalCarrier};
use crate::pb::{E01Carrier, E02Carrier};
use crate::process::{self, ProcessResult};
use crate::reference::{AdapterTruthTableRow, NativeReferenceBundle};
use crate::wire::{decode_frame, encode_frame};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NativeExecutables {
    pub go_source: PathBuf,
    pub rust_target: PathBuf,
    pub repo: PathBuf,
    pub use_sandbox_exec: bool,
    pub outer_isolation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ViolationWitness {
    pub expected_role: String,
    pub fixture_id: String,
    pub nested_adapter_path: String,
    pub source_role: String,
    pub source_value: Value,
    pub target_role: String,
    pub target_value: Value,
    pub transported_role: String,
    pub transported_value: Value,
    pub violated_or_missing_dimensions: Vec<String>,
    pub witness_kind: String,
}

#[derive(Clone, Debug)]
pub struct SemanticRun {
    pub adapter_value_mismatches: Vec<String>,
    pub checked_adapter_value_count: usize,
    pub checked_comparator_pair_count: usize,
    pub comparator_truth_table_mismatches: Vec<String>,
    pub fixture_id: String,
    pub main_processes: Vec<ProcessResult>,
    pub ordinary_comparator_result: String,
    pub policy_soundness: String,
    pub reference_bundle_evidence_id: String,
    pub reference_bundle_sha256: String,
    pub roundtrip_truth_table_mismatches: Vec<String>,
    pub roundtrips: BTreeMap<String, String>,
    pub source_program_output: Value,
    pub target_program_output: Value,
    pub transcript: Vec<String>,
    pub transported_source: Value,
    pub witness: ViolationWitness,
}

#[derive(Clone, Debug)]
struct NativeCall {
    process: ProcessResult,
    payload: Option<Value>,
}

fn invoke(
    executables: &NativeExecutables,
    role: &str,
    fixture: &str,
    operation: &str,
    value: Option<&Value>,
    input: &[u8],
) -> Result<NativeCall> {
    let executable = match role {
        "go-source" => &executables.go_source,
        "rust-target" => &executables.rust_target,
        _ => bail!("unknown native role {role}"),
    };
    let logical_executable = match role {
        "go-source" => "native/bin/gluerift-native-source",
        "rust-target" => "native/bin/gluerift-native-target",
        _ => unreachable!(),
    };
    let mut arguments = vec![
        "--fixture".to_owned(),
        fixture.to_owned(),
        "--operation".to_owned(),
        operation.to_owned(),
    ];
    if let Some(value) = value {
        arguments.push("--value".to_owned());
        arguments.push(String::from_utf8(canonical::to_vec(value)?)?);
    }
    let native_cwd = executables.repo.join("native");
    let process = process::run(
        executable,
        logical_executable,
        role,
        operation,
        &arguments,
        input,
        &executables.repo,
        &native_cwd,
        "native",
        executables.use_sandbox_exec,
        &executables.outer_isolation,
    )?;
    let json_bytes = if (matches!(operation, "encode" | "program-output") && role == "go-source")
        || (operation == "encode" && role == "rust-target")
    {
        &process.stderr
    } else {
        &process.stdout
    };
    let payload = if json_bytes.is_empty() {
        None
    } else {
        let envelope: Value = serde_json::from_slice(json_bytes)
            .with_context(|| format!("parse protocol envelope for {role}/{fixture}/{operation}"))?;
        if envelope.get("schema") != Some(&json!("gluerift.native-protocol/v1"))
            || envelope.get("fixture_id") != Some(&json!(fixture))
            || envelope.get("operation_id") != Some(&json!(operation))
        {
            bail!("protocol envelope binding mismatch for {role}/{fixture}/{operation}")
        }
        Some(
            envelope
                .get("payload")
                .cloned()
                .context("protocol payload")?,
        )
    };
    Ok(NativeCall { process, payload })
}

fn encode(
    executables: &NativeExecutables,
    role: &str,
    fixture: &str,
    value: &Value,
) -> Result<Vec<u8>> {
    Ok(
        invoke(executables, role, fixture, "encode", Some(value), &[])?
            .process
            .stdout,
    )
}

fn decode<T: DeserializeOwned>(
    executables: &NativeExecutables,
    role: &str,
    fixture: &str,
    carrier: &[u8],
) -> Result<T> {
    let call = invoke(executables, role, fixture, "decode", None, carrier)?;
    serde_json::from_value(call.payload.context("missing decode payload")?["native"].clone())
        .context("decode native payload")
}

fn compare(
    executables: &NativeExecutables,
    fixture: &str,
    carrier: &[u8],
    target: &Value,
) -> Result<bool> {
    let call = invoke(
        executables,
        "rust-target",
        fixture,
        "compare",
        Some(target),
        carrier,
    )?;
    Ok(call.payload.context("missing compare payload")?["ordinary_comparator"] == "EQUAL")
}

pub fn run(
    fixture: &str,
    executables: &NativeExecutables,
    bundle: &NativeReferenceBundle,
    reference_bundle_sha256: &str,
) -> Result<SemanticRun> {
    if bundle.fixture_id != fixture {
        bail!("reference bundle fixture mismatch")
    }

    let mut adapter_value_mismatches = Vec::new();
    for (map, rows) in [
        ("source_encode", bundle.source_encode_truth_table.as_slice()),
        ("source_decode", bundle.source_decode_truth_table.as_slice()),
        ("target_encode", bundle.target_encode_truth_table.as_slice()),
        ("target_decode", bundle.target_decode_truth_table.as_slice()),
    ] {
        validate_map_rows(
            executables,
            fixture,
            map,
            rows,
            &mut adapter_value_mismatches,
        )?;
    }

    let mut comparator_truth_table_mismatches = Vec::new();
    for (index, row) in bundle.target_native_relation_truth_table.iter().enumerate() {
        let source_native = ir_to_native(fixture, "source", &row.source)?;
        let target_native = ir_to_native(fixture, "target", &row.target)?;
        let wire = encode(executables, "go-source", fixture, &source_native)?;
        let transported_native: Value = decode(executables, "rust-target", fixture, &wire)?;
        let transported = native_to_ir(fixture, "target", &transported_native)?;
        let equal = compare(executables, fixture, &wire, &target_native)?;
        if transported != row.transported_source_as_target_native || equal != row.equal {
            comparator_truth_table_mismatches.push(format!("relation row {index}"));
        }
    }

    let mut roundtrip_truth_table_mismatches = Vec::new();
    let mut roundtrips = BTreeMap::new();
    for (law_id, table) in &bundle.six_roundtrip_truth_tables {
        let before = roundtrip_truth_table_mismatches.len();
        if table.law_id != *law_id {
            roundtrip_truth_table_mismatches.push(format!("{law_id}: table law-id mismatch"));
        }
        for (row_index, row) in table.rows.iter().enumerate() {
            let mut current = row.input.clone();
            for (stage_index, stage) in row.stages.iter().enumerate() {
                current = observe_map(executables, fixture, &stage.stage, &current)?;
                if current != stage.output {
                    roundtrip_truth_table_mismatches.push(format!(
                        "{law_id} row {row_index} stage {stage_index} ({})",
                        stage.stage
                    ));
                }
            }
            if (current == row.input) != row.final_matches_input {
                roundtrip_truth_table_mismatches
                    .push(format!("{law_id} row {row_index} final equality"));
            }
        }
        let status = if roundtrip_truth_table_mismatches.len() == before {
            "proved-exhaustive"
        } else {
            "disproved"
        };
        roundtrips.insert(external_law_id(law_id)?.to_owned(), status.to_owned());
    }

    let source_call = invoke(
        executables,
        "go-source",
        fixture,
        "program-output",
        None,
        &[],
    )?;
    let source_payload = source_call
        .payload
        .clone()
        .context("source program payload")?;
    let source_output = native_to_ir(fixture, "source", &source_payload["native"])?;
    let compare_call = invoke(
        executables,
        "rust-target",
        fixture,
        "transport-compare",
        None,
        &source_call.process.stdout,
    )?;
    let compare_payload = compare_call
        .payload
        .clone()
        .context("transport compare payload")?;
    let transported = native_to_ir(
        fixture,
        "target",
        &compare_payload["transported_source_as_target_native"],
    )?;
    let target_output = native_to_ir(fixture, "target", &compare_payload["target_program_output"])?;
    let ordinary = compare_payload["ordinary_comparator"]
        .as_str()
        .context("ordinary comparator result")?
        .to_owned();
    let canonical_witness = &bundle.canonical_unsafe_witness;
    if source_output != canonical_witness.source_value
        || transported != canonical_witness.transported_source_as_target_native
        || target_output != canonical_witness.target_value
        || ordinary != "EQUAL"
    {
        bail!("{fixture} operational transcript differs from checker bundle witness")
    }

    let witness = ViolationWitness {
        expected_role: "policy-safe-value".into(),
        fixture_id: fixture.into(),
        nested_adapter_path: canonical_witness.semantic_path.join("."),
        source_role: "checker-source".into(),
        source_value: canonical_witness.source_value.clone(),
        target_role: "checker-target".into(),
        target_value: canonical_witness.target_value.clone(),
        transported_role: "source-as-target-native".into(),
        transported_value: canonical_witness
            .transported_source_as_target_native
            .clone(),
        violated_or_missing_dimensions: canonical_witness.violated_or_missing_dimensions.clone(),
        witness_kind: canonical_witness.witness_kind.clone(),
    };
    let transcript = vec![
        format!("checker native-reference bundle: sha256:{reference_bundle_sha256}"),
        format!(
            "source program output (checker IR): {}",
            String::from_utf8(canonical::to_vec(&source_output)?)?
        ),
        format!(
            "target program output (checker IR): {}",
            String::from_utf8(canonical::to_vec(&target_output)?)?
        ),
        format!(
            "transported source as target native (checker IR): {}",
            String::from_utf8(canonical::to_vec(&transported)?)?
        ),
        "ordinary target-native comparator: EQUAL".into(),
        "four adapter truth tables: proved-exhaustive".into(),
        "target-native relation truth table: proved-exhaustive".into(),
        "six checker round-trip truth tables: proved-exhaustive".into(),
        "comparator_definedness: proved-exhaustive".into(),
        "policy_soundness: disproved".into(),
    ];

    Ok(SemanticRun {
        checked_adapter_value_count: bundle.source_encode_truth_table.len()
            + bundle.source_decode_truth_table.len()
            + bundle.target_encode_truth_table.len()
            + bundle.target_decode_truth_table.len(),
        checked_comparator_pair_count: bundle.target_native_relation_truth_table.len(),
        adapter_value_mismatches,
        comparator_truth_table_mismatches,
        fixture_id: fixture.into(),
        main_processes: vec![source_call.process, compare_call.process],
        ordinary_comparator_result: ordinary,
        policy_soundness: "disproved".into(),
        reference_bundle_evidence_id: bundle.evidence_id.clone(),
        reference_bundle_sha256: reference_bundle_sha256.into(),
        roundtrip_truth_table_mismatches,
        roundtrips,
        source_program_output: source_output,
        target_program_output: target_output,
        transcript,
        transported_source: transported,
        witness,
    })
}

fn validate_map_rows(
    executables: &NativeExecutables,
    fixture: &str,
    map: &str,
    rows: &[AdapterTruthTableRow],
    mismatches: &mut Vec<String>,
) -> Result<()> {
    for (index, row) in rows.iter().enumerate() {
        let observed = observe_map(executables, fixture, map, &row.input)?;
        if observed != row.output {
            mismatches.push(format!("{map} row {index}"));
        }
    }
    Ok(())
}

fn observe_map(
    executables: &NativeExecutables,
    fixture: &str,
    map: &str,
    input: &Value,
) -> Result<Value> {
    match map {
        "source_encode" | "target_encode" => {
            let endpoint = if map == "source_encode" {
                "source"
            } else {
                "target"
            };
            let role = if endpoint == "source" {
                "go-source"
            } else {
                "rust-target"
            };
            let native = ir_to_native(fixture, endpoint, input)?;
            let wire = encode(executables, role, fixture, &native)?;
            wire_to_carrier_ir(fixture, &wire)
        }
        "source_decode" | "target_decode" => {
            let endpoint = if map == "source_decode" {
                "source"
            } else {
                "target"
            };
            let role = if endpoint == "source" {
                "go-source"
            } else {
                "rust-target"
            };
            let wire = carrier_ir_to_wire(fixture, input)?;
            let native: Value = decode(executables, role, fixture, &wire)?;
            native_to_ir(fixture, endpoint, &native)
        }
        _ => bail!("unknown checker map stage {map}"),
    }
}

fn external_law_id(checker_id: &str) -> Result<&'static str> {
    match checker_id {
        "source-native" => Ok("source-native-roundtrip"),
        "target-native" => Ok("target-native-roundtrip"),
        "source-carrier" => Ok("source-carrier-roundtrip"),
        "target-carrier" => Ok("target-carrier-roundtrip"),
        "source-full-transport" => Ok("source-full-transport-roundtrip"),
        "target-full-transport" => Ok("target-full-transport-roundtrip"),
        _ => bail!("unknown checker law id {checker_id}"),
    }
}

fn ir_to_native(fixture: &str, endpoint: &str, value: &Value) -> Result<Value> {
    match fixture {
        "E01" => Ok(Value::String(ir_sum_variant(value)?.to_owned())),
        "E02" => Ok(json!({
            "policy": {"bounds": {
                "maximum": ir_nested_int(value, "maximum")?,
                "minimum": ir_nested_int(value, "minimum")?
            }}
        })),
        _ => bail!("unknown native fixture {fixture}/{endpoint}"),
    }
}

fn native_to_ir(fixture: &str, _endpoint: &str, value: &Value) -> Result<Value> {
    match fixture {
        "E01" => Ok(ir_sum(
            value
                .as_str()
                .context("E01 native value must be a string")?,
        )),
        "E02" => {
            let bounds = value
                .get("policy")
                .and_then(|value| value.get("bounds"))
                .context("E02 native value lacks policy.bounds")?;
            Ok(ir_nested(
                bounds
                    .get("minimum")
                    .and_then(Value::as_i64)
                    .context("E02 native minimum")?,
                bounds
                    .get("maximum")
                    .and_then(Value::as_i64)
                    .context("E02 native maximum")?,
                false,
            ))
        }
        _ => bail!("unknown native fixture {fixture}"),
    }
}

fn wire_to_carrier_ir(fixture: &str, wire: &[u8]) -> Result<Value> {
    match fixture {
        "E01" => {
            let carrier = DecisionLogicalCarrier::from_proto(decode_frame::<E01Carrier>(wire)?)?;
            Ok(ir_sum(match carrier {
                DecisionLogicalCarrier::Deny => "DECISION_CARRIER_DENY",
                DecisionLogicalCarrier::Allow => "DECISION_CARRIER_ALLOW",
            }))
        }
        "E02" => {
            let carrier = BoundsLogicalCarrier::from_proto(decode_frame::<E02Carrier>(wire)?)?;
            Ok(ir_nested(
                i64::from(carrier.minimum_slot),
                i64::from(carrier.maximum_slot),
                true,
            ))
        }
        _ => bail!("unknown native fixture {fixture}"),
    }
}

fn carrier_ir_to_wire(fixture: &str, value: &Value) -> Result<Vec<u8>> {
    match fixture {
        "E01" => {
            let carrier = match ir_sum_variant(value)? {
                "DECISION_CARRIER_DENY" => DecisionLogicalCarrier::Deny,
                "DECISION_CARRIER_ALLOW" => DecisionLogicalCarrier::Allow,
                variant => bail!("unknown E01 carrier variant {variant}"),
            };
            encode_frame(&carrier.to_proto())
        }
        "E02" => {
            let carrier = BoundsLogicalCarrier {
                minimum_slot: i32::try_from(ir_nested_int(value, "minimum_slot")?)?,
                maximum_slot: i32::try_from(ir_nested_int(value, "maximum_slot")?)?,
            };
            encode_frame(&carrier.to_proto())
        }
        _ => bail!("unknown native fixture {fixture}"),
    }
}

fn ir_sum(variant: &str) -> Value {
    json!({"kind": "sum", "payload": {"kind": "unit"}, "variant": variant})
}

fn ir_sum_variant(value: &Value) -> Result<&str> {
    if value.get("kind") != Some(&json!("sum")) {
        bail!("checker value is not a sum")
    }
    value
        .get("variant")
        .and_then(Value::as_str)
        .context("checker sum lacks variant")
}

fn ir_nested(minimum: i64, maximum: i64, carrier: bool) -> Value {
    let minimum_name = if carrier { "minimum_slot" } else { "minimum" };
    let maximum_name = if carrier { "maximum_slot" } else { "maximum" };
    json!({
        "kind": "product",
        "fields": {"output": {
            "kind": "product",
            "fields": {"policy": {
                "kind": "product",
                "fields": {"bounds": {
                    "kind": "product",
                    "fields": {
                        maximum_name: {"kind": "bounded-int", "value": maximum},
                        minimum_name: {"kind": "bounded-int", "value": minimum}
                    }
                }}
            }}
        }}
    })
}

fn ir_nested_int(value: &Value, name: &str) -> Result<i64> {
    value
        .get("fields")
        .and_then(|value| value.get("output"))
        .and_then(|value| value.get("fields"))
        .and_then(|value| value.get("policy"))
        .and_then(|value| value.get("fields"))
        .and_then(|value| value.get("bounds"))
        .and_then(|value| value.get("fields"))
        .and_then(|value| value.get(name))
        .and_then(|value| value.get("value"))
        .and_then(Value::as_i64)
        .with_context(|| format!("checker E02 value lacks output.policy.bounds.{name}"))
}

pub fn ensure_success(run: &SemanticRun) -> Result<()> {
    if !run.adapter_value_mismatches.is_empty() {
        bail!(
            "native adapter mismatches: {:?}",
            run.adapter_value_mismatches
        )
    }
    if !run.comparator_truth_table_mismatches.is_empty() {
        bail!(
            "native comparator mismatches: {:?}",
            run.comparator_truth_table_mismatches
        )
    }
    if !run.roundtrip_truth_table_mismatches.is_empty() {
        bail!(
            "native round-trip table mismatches: {:?}",
            run.roundtrip_truth_table_mismatches
        )
    }
    if run
        .roundtrips
        .values()
        .any(|status| status != "proved-exhaustive")
    {
        bail!(
            "one or more native round trips failed: {:?}",
            run.roundtrips
        )
    }
    if run.ordinary_comparator_result != "EQUAL" || run.policy_soundness != "disproved" {
        bail!("native operational attack did not close")
    }
    Ok(())
}
