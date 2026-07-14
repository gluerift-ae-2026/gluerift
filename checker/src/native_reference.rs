use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::CONTRACT_VERSION;
use crate::adapter_ir::{Adapter, AdapterContext};
use crate::canonical::canonical_sha256;
use crate::comparison::{CheckedRun, RunConfiguration};
use crate::domain::{ComparatorSpec, ValidationScope, ValuePair};
use crate::report::{ComparatorDefinednessReport, LawId, Status, TransformationReport};
use crate::type_ir::{ResultBranch, Value};
use crate::witness::WitnessKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterTruthTableRow {
    pub input: Value,
    pub output: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetNativeRelationRow {
    pub source: Value,
    pub target: Value,
    pub transported_source_as_target_native: Value,
    pub equal: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTruthStage {
    pub stage: String,
    pub output: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoundTripTruthRow {
    pub input: Value,
    pub stages: Vec<NativeTruthStage>,
    pub final_matches_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoundTripTruthTable {
    pub law_id: LawId,
    pub rows: Vec<NativeRoundTripTruthRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeUnsafeWitness {
    pub witness_kind: WitnessKind,
    pub source_value: Value,
    pub target_value: Value,
    pub transported_source_as_target_native: Value,
    pub semantic_path: Vec<String>,
    pub violated_or_missing_dimensions: Vec<String>,
}

/// A checker-emitted, content-addressed semantic oracle for native replay.
///
/// The native harness is intentionally not allowed to reconstruct these values
/// with a second hand-written model.  It must execute the real Go/Rust programs
/// and compare every observed row with this bundle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReferenceBundle {
    pub schema: String,
    pub semantic_contract_version: String,
    pub tool_build_sha256: String,
    pub run_configuration_sha256: String,
    pub evidence_id: String,
    pub dependency_evidence_ids: Vec<String>,
    pub status: Status,
    pub fixture_id: String,
    pub reference_run_id: String,
    pub source_inputs_manifest_sha256: String,
    pub source_tree_sha256: String,
    pub candidate_context_sha256: String,
    pub transformation_report_sha256: String,
    pub reference_check_report_sha256: String,
    pub reference_check_evidence_id: String,
    pub types_sha256: String,
    pub validation_scope_sha256: String,
    pub endpoint_policy_sha256: String,
    pub validation_request_sha256: String,
    pub comparator_spec_sha256: String,
    pub canonical_source_domain: Vec<Value>,
    pub canonical_target_domain: Vec<Value>,
    pub canonical_source_carrier_domain: Vec<Value>,
    pub canonical_target_carrier_domain: Vec<Value>,
    pub comparison_universe: Vec<ValuePair>,
    pub source_encode_truth_table: Vec<AdapterTruthTableRow>,
    pub source_decode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_encode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_decode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_native_relation_truth_table: Vec<TargetNativeRelationRow>,
    pub six_roundtrip_truth_tables: BTreeMap<LawId, NativeRoundTripTruthTable>,
    pub comparator_definedness: ComparatorDefinednessReport,
    pub canonical_unsafe_witness: NativeUnsafeWitness,
    pub canonical_unsafe_witness_sha256: String,
}

#[allow(clippy::too_many_arguments)]
pub fn build_native_reference_bundle(
    fixture_id: &str,
    reference_run_id: &str,
    context: &AdapterContext,
    scope: &ValidationScope,
    configuration: &RunConfiguration,
    checked: &CheckedRun,
    transformation: &TransformationReport,
    source_inputs_manifest_sha256: &str,
    source_tree_sha256: &str,
) -> Result<NativeReferenceBundle, String> {
    if scope.comparator != ComparatorSpec::TargetNativeExact {
        return Err("native reference bundle requires target-native-exact comparator".into());
    }
    if checked.check_report.properties.policy_soundness.status != Status::Disproved {
        return Err("native attack replay requires a disproved policy-soundness result".into());
    }

    let context = context.normalized();
    let resolved = scope
        .resolve(&context, &configuration.enumeration_limits())
        .map_err(|error| error.to_string())?;

    let source_encode_truth_table = truth_table(&context.source_encode, &resolved.source_domain)?;
    let source_decode_truth_table =
        truth_table(&context.source_decode, &resolved.source_carrier_domain)?;
    let target_encode_truth_table = truth_table(&context.target_encode, &resolved.target_domain)?;
    let target_decode_truth_table =
        truth_table(&context.target_decode, &resolved.target_carrier_domain)?;

    let mut target_native_relation_truth_table = Vec::new();
    for pair in &resolved.comparison_universe {
        let carrier = context
            .source_encode
            .eval(&pair.source)
            .map_err(|error| error.to_string())?;
        let transported = context
            .target_decode
            .eval(&carrier)
            .map_err(|error| error.to_string())?;
        target_native_relation_truth_table.push(TargetNativeRelationRow {
            source: pair.source.clone(),
            target: pair.target.clone(),
            equal: transported == pair.target,
            transported_source_as_target_native: transported,
        });
    }

    let mut six_roundtrip_truth_tables = BTreeMap::new();
    for (law_id, report) in &checked.execution_trace_tables {
        let mut rows = Vec::new();
        for row in &report.rows {
            let mut stages = Vec::new();
            for stage in &row.stages {
                let output = stage.result.as_ref().map_err(|error| {
                    format!(
                        "{law_id:?} truth-table stage {} failed: {error}",
                        stage.stage
                    )
                })?;
                stages.push(NativeTruthStage {
                    stage: stage.stage.clone(),
                    output: output.clone(),
                });
            }
            rows.push(NativeRoundTripTruthRow {
                input: row.input.clone(),
                stages,
                final_matches_input: row.final_matches_input,
            });
        }
        six_roundtrip_truth_tables.insert(
            *law_id,
            NativeRoundTripTruthTable {
                law_id: *law_id,
                rows,
            },
        );
    }
    if six_roundtrip_truth_tables.len() != 6 {
        return Err("native reference bundle must contain all six round-trip tables".into());
    }

    let unsafe_pairs: BTreeSet<_> = checked
        .induced_relation
        .difference(&checked.safe)
        .cloned()
        .collect();
    let pair = unsafe_pairs
        .iter()
        .max_by(|left, right| canonical_witness_order(left, right))
        .ok_or_else(|| "native reference run has no unsafe false-agreement pair".to_string())?;
    let carrier = context
        .source_encode
        .eval(&pair.source)
        .map_err(|error| error.to_string())?;
    let transported = context
        .target_decode
        .eval(&carrier)
        .map_err(|error| error.to_string())?;
    let witness_basis = checked
        .witnesses
        .values()
        .find(|witness| witness.witness_kind == WitnessKind::UnsafeFalseAgreement)
        .ok_or_else(|| "check report lacks unsafe-false-agreement witness evidence".to_string())?;
    let canonical_unsafe_witness = NativeUnsafeWitness {
        witness_kind: WitnessKind::UnsafeFalseAgreement,
        source_value: pair.source.clone(),
        target_value: pair.target.clone(),
        transported_source_as_target_native: transported,
        semantic_path: witness_basis.adapter_path.clone(),
        violated_or_missing_dimensions: witness_basis.violated_or_missing_dimensions.clone(),
    };
    let canonical_unsafe_witness_sha256 =
        canonical_sha256(&canonical_unsafe_witness).map_err(|error| error.to_string())?;

    let reference_check_report_sha256 =
        canonical_sha256(&checked.check_report).map_err(|error| error.to_string())?;
    let transformation_report_sha256 =
        canonical_sha256(transformation).map_err(|error| error.to_string())?;
    let mut dependency_evidence_ids = vec![
        checked.check_report.envelope.evidence_id.clone(),
        transformation.envelope.evidence_id.clone(),
        witness_basis.envelope.evidence_id.clone(),
    ];
    dependency_evidence_ids.sort();
    dependency_evidence_ids.dedup();

    Ok(NativeReferenceBundle {
        schema: "gluerift.native-reference-bundle/v0.3.1a".into(),
        semantic_contract_version: CONTRACT_VERSION.into(),
        tool_build_sha256: checked.check_report.envelope.tool_build_sha256.clone(),
        run_configuration_sha256: checked
            .check_report
            .envelope
            .run_configuration_sha256
            .clone(),
        evidence_id: format!(
            "{}:native-reference-bundle",
            checked.check_report.envelope.evidence_id
        ),
        dependency_evidence_ids,
        status: Status::ProvedExhaustive,
        fixture_id: fixture_id.into(),
        reference_run_id: reference_run_id.into(),
        source_inputs_manifest_sha256: source_inputs_manifest_sha256.into(),
        source_tree_sha256: source_tree_sha256.into(),
        candidate_context_sha256: checked.check_report.envelope.candidate_sha256.clone(),
        transformation_report_sha256,
        reference_check_report_sha256,
        reference_check_evidence_id: checked.check_report.envelope.evidence_id.clone(),
        types_sha256: checked.check_report.envelope.types_sha256.clone(),
        validation_scope_sha256: checked
            .check_report
            .envelope
            .validation_scope_sha256
            .clone(),
        endpoint_policy_sha256: checked.check_report.envelope.endpoint_policy_sha256.clone(),
        validation_request_sha256: checked
            .check_report
            .envelope
            .validation_request_sha256
            .clone(),
        comparator_spec_sha256: checked.check_report.envelope.comparator_spec_sha256.clone(),
        canonical_source_domain: resolved.source_domain,
        canonical_target_domain: resolved.target_domain,
        canonical_source_carrier_domain: resolved.source_carrier_domain,
        canonical_target_carrier_domain: resolved.target_carrier_domain,
        comparison_universe: resolved.comparison_universe,
        source_encode_truth_table,
        source_decode_truth_table,
        target_encode_truth_table,
        target_decode_truth_table,
        target_native_relation_truth_table,
        six_roundtrip_truth_tables,
        comparator_definedness: checked
            .check_report
            .comparison
            .comparator_definedness
            .clone(),
        canonical_unsafe_witness,
        canonical_unsafe_witness_sha256,
    })
}

fn truth_table(adapter: &Adapter, domain: &[Value]) -> Result<Vec<AdapterTruthTableRow>, String> {
    domain
        .iter()
        .map(|input| {
            Ok(AdapterTruthTableRow {
                input: input.clone(),
                output: adapter.eval(input).map_err(|error| error.to_string())?,
            })
        })
        .collect()
}

fn canonical_witness_order(left: &ValuePair, right: &ValuePair) -> Ordering {
    semantic_distance(&left.source, &left.target)
        .cmp(&semantic_distance(&right.source, &right.target))
        .then_with(|| left.cmp(right))
}

fn semantic_distance(left: &Value, right: &Value) -> u128 {
    match (left, right) {
        (Value::Unit, Value::Unit) => 0,
        (Value::Bool { value: left }, Value::Bool { value: right }) => u128::from(left != right),
        (Value::BoundedInt { value: left }, Value::BoundedInt { value: right }) => {
            (*left as i128 - *right as i128).unsigned_abs()
        }
        (Value::BitVec { value: left }, Value::BitVec { value: right }) => {
            left.abs_diff(*right) as u128
        }
        (
            Value::Sum {
                variant: left_variant,
                payload: left_payload,
            },
            Value::Sum {
                variant: right_variant,
                payload: right_payload,
            },
        ) => {
            u128::from(left_variant != right_variant)
                + semantic_distance(left_payload, right_payload)
        }
        (Value::Product { fields: left }, Value::Product { fields: right }) => left
            .keys()
            .chain(right.keys())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|key| match (left.get(key), right.get(key)) {
                (Some(left), Some(right)) => semantic_distance(left, right),
                _ => 1,
            })
            .sum(),
        (
            Value::ObjectResult {
                branch: left_branch,
                value: left_value,
            },
            Value::ObjectResult {
                branch: right_branch,
                value: right_value,
            },
        ) => {
            u128::from(matches!(
                (left_branch, right_branch),
                (ResultBranch::Ok, ResultBranch::Err) | (ResultBranch::Err, ResultBranch::Ok)
            )) + semantic_distance(left_value, right_value)
        }
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_witness_prefers_maximal_structural_separation() {
        let pair = |source, target| ValuePair {
            source: Value::BoundedInt { value: source },
            target: Value::BoundedInt { value: target },
        };
        assert_eq!(
            canonical_witness_order(&pair(0, 2), &pair(0, 1)),
            Ordering::Greater
        );
    }
}
