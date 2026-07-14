use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode};

use clap::{Args, Parser, Subcommand};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use gluerift::adapter_ir::{AdapterContext, AdapterTypeError};
use gluerift::canonical::{canonical_bytes, canonical_sha256};
use gluerift::comparison::{
    CertificationProfile, CheckError, EvidenceMetadata, PropertyRequest, RunConfiguration,
    ValidationRequest, check,
};
use gluerift::composition::{CompositionRequest, compose_and_check};
use gluerift::domain::{DomainError, ValidationScope};
use gluerift::native_reference::build_native_reference_bundle;
use gluerift::relation_ir::{EndpointPolicy, MatchCoverageMode};
use gluerift::report::{
    AbsentEvidence, BaselineReport, BindingStatus, BridgeKind, CommonEnvelope, EvidenceValue,
    LawId, MatchCoverageReport, PolicyContractStatus, Status, TargetNonAmplificationReport,
    TransformationClassification, TransformationReport,
};
use gluerift::transformation::{
    TransformationCandidate, TransformationFamilyDescriptor, classify_transformation,
};
use gluerift::type_ir::TypeError;
use gluerift::{CONTRACT_VERSION, CheckReport};

#[derive(Parser)]
#[command(
    name = "gluerift",
    version = "0.3.1a",
    about = "GlueRift finite reference checker"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Check(CheckArgs),
    Roundtrip(CheckArgs),
    DeriveCarrier(CheckArgs),
    Transformations(TransformationArgs),
    RunFixtures {
        #[arg(long)]
        registry: PathBuf,
        #[arg(long)]
        out_dir: PathBuf,
    },
    RunBaselines {
        #[arg(long)]
        registry: PathBuf,
        #[arg(long)]
        baselines: String,
        #[arg(long)]
        out_dir: PathBuf,
    },
    ReplayNative {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Reproduce {
        #[arg(long)]
        profile: String,
        #[arg(long)]
        out_dir: PathBuf,
    },
}

#[derive(Args, Clone)]
struct CheckArgs {
    #[arg(long)]
    context: PathBuf,
    #[arg(long)]
    scope: PathBuf,
    #[arg(long)]
    policy: PathBuf,
    #[arg(long)]
    request: PathBuf,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Args)]
struct TransformationArgs {
    #[command(flatten)]
    check: CheckArgs,
    #[arg(long)]
    family: PathBuf,
}

#[derive(Debug)]
struct CliError {
    code: u8,
    message: String,
}

impl CliError {
    fn semantic(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }
    fn unknown(message: impl Into<String>) -> Self {
        Self {
            code: 3,
            message: message.into(),
        }
    }
    fn tool(message: impl Into<String>) -> Self {
        Self {
            code: 4,
            message: message.into(),
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("gluerift: {}", error.message);
            ExitCode::from(error.code)
        }
    }
}

fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Command::Check(args) => {
            let loaded = load_inputs(&args)?;
            let checked = evaluate_loaded(&loaded)?;
            write_checked_run(&args.out, &checked, OutputSelection::Check)?;
            ensure_direct_status(checked.check_report.envelope.status)
        }
        Command::Roundtrip(args) => {
            let loaded = load_inputs(&args)?;
            let checked = evaluate_loaded(&loaded)?;
            write_checked_run(&args.out, &checked, OutputSelection::Roundtrip)?;
            let required = &checked.check_report.certification.explicit_required_law_ids;
            let status = aggregate_cli_status(
                checked
                    .roundtrip_report
                    .laws
                    .iter()
                    .filter(|law| required.contains(&law.law_id))
                    .map(|law| law.status),
            );
            ensure_direct_status(status)
        }
        Command::DeriveCarrier(args) => {
            let loaded = load_inputs(&args)?;
            let checked = evaluate_loaded(&loaded)?;
            write_checked_run(&args.out, &checked, OutputSelection::Carrier)?;
            ensure_direct_status(checked.check_report.bridges.selected_carrier_bridge_status)
        }
        Command::Transformations(args) => run_transformations(&args),
        Command::RunFixtures { registry, out_dir } => run_fixtures(&registry, &out_dir),
        Command::RunBaselines {
            registry,
            baselines,
            out_dir,
        } => run_baselines(&registry, &baselines, &out_dir),
        Command::ReplayNative { manifest, out } => replay_native(&manifest, &out),
        Command::Reproduce { profile, out_dir } => reproduce(&profile, &out_dir),
    }
}

struct LoadedInputs {
    context: AdapterContext,
    scope: ValidationScope,
    policy: EndpointPolicy,
    request: ValidationRequest,
    configuration: RunConfiguration,
}

fn load_inputs(args: &CheckArgs) -> Result<LoadedInputs, CliError> {
    let workspace = workspace_root(&args.context)
        .or_else(|_| workspace_root(&args.request))
        .or_else(|_| workspace_root(Path::new(".")))?;
    let configuration_path = workspace.join("spec/run-config/core-v0.3.1a.json");
    Ok(LoadedInputs {
        context: read_json(&args.context)?,
        scope: read_json(&args.scope)?,
        policy: read_json(&args.policy)?,
        request: read_json(&args.request)?,
        configuration: read_json(&configuration_path)?,
    })
}

fn evaluate_loaded(inputs: &LoadedInputs) -> Result<gluerift::CheckedRun, CliError> {
    let metadata = EvidenceMetadata::for_current_executable().map_err(io_error)?;
    check(
        &inputs.context,
        &inputs.scope,
        &inputs.policy,
        &inputs.request,
        &inputs.configuration,
        &metadata,
    )
    .map_err(map_check_error)
}

#[derive(Clone, Copy)]
enum OutputSelection {
    Check,
    Roundtrip,
    Carrier,
}

fn write_checked_run(
    path: &Path,
    run: &gluerift::CheckedRun,
    selection: OutputSelection,
) -> Result<(), CliError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let evidence = parent.join("evidence");
    fs::create_dir_all(&evidence).map_err(io_error)?;
    for (hash, witness) in &run.witnesses {
        write_jcs(
            &evidence.join("witnesses").join(format!("{hash}.json")),
            witness,
        )?;
    }
    for (law, table) in &run.execution_trace_tables {
        let hash = canonical_sha256(table).map_err(canonical_error)?;
        let law_name = enum_string(*law)?;
        write_jcs(
            &evidence
                .join("execution-traces")
                .join(format!("{law_name}-{hash}.json")),
            table,
        )?;
    }
    write_jcs(
        &evidence.join(format!(
            "roundtrip-{}.json",
            canonical_sha256(&run.roundtrip_report).map_err(canonical_error)?
        )),
        &run.roundtrip_report,
    )?;
    for (kind, report) in &run.bridge_reports {
        let name = enum_string(*kind)?;
        write_jcs(
            &evidence.join(format!(
                "bridge-{name}-{}.json",
                canonical_sha256(report).map_err(canonical_error)?
            )),
            report,
        )?;
    }
    write_jcs(
        &evidence.join(format!(
            "carrier-summary-{}.json",
            canonical_sha256(&run.carrier_summary).map_err(canonical_error)?
        )),
        &run.carrier_summary,
    )?;
    match selection {
        OutputSelection::Check => write_jcs(path, &run.check_report),
        OutputSelection::Roundtrip => write_jcs(path, &run.roundtrip_report),
        OutputSelection::Carrier => write_jcs(path, &run.carrier_summary),
    }
}

fn run_transformations(args: &TransformationArgs) -> Result<(), CliError> {
    let loaded = load_inputs(&args.check)?;
    let family: TransformationFamilyDescriptor = read_json(&args.family)?;
    let family_hash = canonical_sha256(&family).map_err(canonical_error)?;
    if family_hash != loaded.request.required_transformation_family_sha256 {
        return Err(CliError::tool(
            "transformation family hash does not match request",
        ));
    }
    let base_run = evaluate_loaded(&loaded)?;
    let metadata = EvidenceMetadata::for_current_executable().map_err(io_error)?;
    let candidates = family
        .enumerate(&loaded.context.carrier_type, &loaded.configuration)
        .map_err(|error| CliError::tool(error.to_string()))?;
    let mut reports = Vec::new();
    let out_dir = args
        .check
        .out
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("transformations");
    for candidate in candidates {
        let classified = classify_transformation(
            &loaded.context,
            &candidate,
            &family,
            &loaded.scope,
            &loaded.policy,
            &loaded.request,
            &loaded.configuration,
            &metadata,
            Some(&base_run),
        )
        .map_err(|error| CliError::tool(error.to_string()))?;
        let hash = canonical_sha256(&classified.report).map_err(canonical_error)?;
        write_jcs(&out_dir.join(format!("{hash}.json")), &classified.report)?;
        write_jcs(
            &out_dir.join("contexts").join(format!(
                "{}.json",
                classified.report.transformed_context_sha256
            )),
            &classified.transformed_context,
        )?;
        reports.push(classified.report);
    }
    reports.sort_by(|a, b| a.transformation_sha256.cmp(&b.transformation_sha256));
    write_jcs(
        &args.check.out,
        &TransformationBatch {
            schema: "gluerift.transformation-results/v0.3.1a".into(),
            transformation_family_sha256: family_hash,
            reports,
        },
    )
}

#[derive(Serialize)]
struct TransformationBatch {
    schema: String,
    transformation_family_sha256: String,
    reports: Vec<TransformationReport>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureRegistry {
    schema: String,
    semantic_contract_version: String,
    runs: Vec<FixtureRow>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureRow {
    run_id: String,
    fixture_kind: String,
    context_logical_path: String,
    transformation_base_context_logical_path: String,
    scope_logical_path: String,
    policy_logical_path: String,
    request_logical_path: String,
    request_id: String,
    validation_request_sha256: String,
    profile: CertificationProfile,
    required_law_ids: Vec<LawId>,
    required_properties: Vec<PropertyRequest>,
    required_properties_sha256: String,
    required_bridge_ids: Vec<BridgeKind>,
    required_transformation_family_sha256: String,
    comparator_spec_sha256: String,
    run_configuration_sha256: String,
    expected_profile_property_consistency: Status,
    match_coverage_mode: MatchCoverageMode,
    expected_match_coverage_status: Status,
    expected_safe_match_equality_status: Status,
    expected_certificate_eligibility: bool,
    expected_certificate_granted: bool,
    expected_comparator_definedness_status: Status,
    expected_law_statuses: BTreeMap<LawId, Status>,
    expected_property_statuses: BTreeMap<String, Status>,
    expected_bridge_statuses: BTreeMap<BridgeKind, Status>,
    expected_policy_contract_status: PolicyContractStatus,
    transformation_report_required: bool,
    transformation_sha256: String,
    expected_transformation_classification: String,
    expected_candidate_binding_status: ExpectedBindingStatus,
    expected_base_alignment_status: ExpectedBindingStatus,
    required_witness_kinds: Vec<String>,
    bl2_paired: bool,
    bl4_paired: bool,
    native_replay_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ExpectedBindingStatus {
    ProvedExhaustive,
    NotRequired,
    ToolError,
    NotApplicable,
}

impl ExpectedBindingStatus {
    fn required_value(self) -> Result<BindingStatus, CliError> {
        match self {
            Self::ProvedExhaustive => Ok(BindingStatus::ProvedExhaustive),
            Self::NotRequired => Ok(BindingStatus::NotRequired),
            Self::ToolError => Ok(BindingStatus::ToolError),
            Self::NotApplicable => Err(CliError::tool(
                "transformation-required row uses not-applicable binding expectation",
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureResults {
    schema: String,
    runs: Vec<FixtureResultRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureResultRow {
    run_id: String,
    fixture_kind: String,
    check_report_sha256: String,
    check_report_evidence_id: String,
    check_report_logical_path: String,
    validation_request_sha256: String,
    candidate_sha256: String,
    comparator_kind: String,
    profile: ProfileAggregate,
    match_coverage: gluerift::report::MatchCoverageReport,
    six_roundtrip_statuses: BTreeMap<LawId, Status>,
    comparator_definedness: Status,
    bridge_statuses: BTreeMap<String, Status>,
    policy_contract_status: PolicyContractStatus,
    policy_vacuity_warning: bool,
    policy_witnesses: Vec<String>,
    property_statuses: PropertyStatusesAggregate,
    property_witnesses: Vec<PropertyWitnessAggregate>,
    certification: CertificationAggregate,
    transformation_results: Vec<TransformationAggregate>,
    native_replay_id: String,
    derivation_status: Status,
    derivation_report_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PropertyStatusesAggregate {
    policy_soundness: Status,
    comparison_adequacy: Status,
    comparison_precision: Status,
    faithful_comparison: Status,
    target_non_amplification_aggregate: Status,
    target_non_amplification_by_dimension: BTreeMap<String, Status>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PropertyWitnessAggregate {
    property_id: String,
    witness_kind: String,
    witness_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProfileAggregate {
    requested_profile: String,
    profile_property_consistency_status: Status,
    safe_match_equality_status: Status,
    safe_match_equality_witness_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CertificationAggregate {
    eligible: bool,
    granted: bool,
    blocking_reasons: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TransformationAggregate {
    transformation_report_sha256: String,
    base_context_sha256: String,
    base_check_report_sha256: String,
    base_alignment_status: BindingStatus,
    transformed_context_sha256: String,
    transformed_check_report_sha256: String,
    candidate_binding_status: BindingStatus,
    transformation_sha256: String,
    inverse_sha256: String,
    action_domain_sha256: String,
    lawfulness_status: Status,
    classification: TransformationClassification,
    harmful_witness_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
struct BaselineResultsIndex {
    schema: String,
    runs: Vec<BaselineResultsRow>,
}
#[derive(Clone, Debug, Serialize)]
struct BaselineResultsRow {
    run_id: String,
    #[serde(rename = "BL2_result")]
    bl2_result: EvidenceValue<Bl2IndexResult>,
    #[serde(rename = "BL4_result")]
    bl4_result: EvidenceValue<Bl4IndexResult>,
}
#[derive(Clone, Debug, Serialize)]
struct Bl2IndexResult {
    report_logical_path: String,
    report_sha256: String,
    evidence_id: String,
    law_statuses: BTreeMap<LawId, Status>,
}
#[derive(Clone, Debug, Serialize)]
struct Bl4IndexResult {
    report_logical_path: String,
    report_sha256: String,
    evidence_id: String,
    common_validity_statuses: BTreeMap<String, Status>,
    match_coverage: MatchCoverageReport,
    policy_contract_status: PolicyContractStatus,
    policy_witnesses: Vec<String>,
    property_statuses: BTreeMap<String, Status>,
    target_non_amplification: TargetNonAmplificationReport,
    property_witnesses: BTreeMap<String, String>,
    validity_parity_status: Status,
    coverage_parity_status: Status,
    policy_parity_status: Status,
    property_parity_status: Status,
    witness_parity_status: Status,
}

#[derive(Clone, Debug, Serialize)]
struct RoundTripBaselineReport {
    #[serde(flatten)]
    envelope: CommonEnvelope,
    baseline_id: String,
    paired_check_report_sha256: String,
    law_statuses: BTreeMap<LawId, Status>,
}

fn run_fixtures(registry_path: &Path, out_dir: &Path) -> Result<(), CliError> {
    let workspace = workspace_root(registry_path)?;
    let source_manifest_path = workspace.join("artifact/source-inputs.manifest.json");
    let source_manifest: serde_json::Value = read_json(&source_manifest_path)?;
    let source_inputs_manifest_sha256 =
        canonical_sha256(&source_manifest).map_err(canonical_error)?;
    let source_tree_sha256 = source_manifest
        .get("source_tree_sha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CliError::invalid("source-input manifest lacks source_tree_sha256"))?;
    let registry: FixtureRegistry = read_json(registry_path)?;
    if registry.schema != "gluerift.fixture-registry/v0.3.1a"
        || registry.semantic_contract_version != CONTRACT_VERSION
    {
        return Err(CliError::invalid("wrong fixture registry schema/version"));
    }
    let family_path = workspace.join("spec/transformation-families/core-structural-v0.3.1a.json");
    let family: TransformationFamilyDescriptor = read_json(&family_path)?;
    let configuration: RunConfiguration =
        read_json(&workspace.join("spec/run-config/core-v0.3.1a.json"))?;
    let metadata = EvidenceMetadata::for_current_executable().map_err(io_error)?;
    let mut results = Vec::new();
    for row in &registry.runs {
        let scope: ValidationScope = read_json(&workspace.join(&row.scope_logical_path))?;
        let policy: EndpointPolicy = read_json(&workspace.join(&row.policy_logical_path))?;
        let request: ValidationRequest = read_json(&workspace.join(&row.request_logical_path))?;
        validate_registry_declarations(&workspace, row, &scope, &policy, &request, &configuration)?;
        let run_dir = out_dir.join("runs").join(sanitize(&row.run_id));
        let (checked, transformation_report, transformed_context) = evaluate_fixture_row(
            &workspace,
            row,
            &family,
            &scope,
            &policy,
            &request,
            &configuration,
            &metadata,
        )?;
        if let Some(report) = &transformation_report {
            write_jcs(&run_dir.join("transformation.json"), report)?;
        }
        if let Some(context) = &transformed_context {
            write_jcs(&run_dir.join("constructed-context.json"), context)?;
        }
        validate_oracles(row, &checked)?;
        write_checked_run(
            &run_dir.join("check.json"),
            &checked,
            OutputSelection::Check,
        )?;
        if row.native_replay_id != "not-applicable" {
            let transformation = transformation_report.as_ref().ok_or_else(|| {
                CliError::tool(format!(
                    "{} native replay lacks transformation evidence",
                    row.run_id
                ))
            })?;
            let context = transformed_context.as_ref().ok_or_else(|| {
                CliError::tool(format!(
                    "{} native replay lacks constructed context",
                    row.run_id
                ))
            })?;
            let bundle = build_native_reference_bundle(
                &row.native_replay_id,
                &row.run_id,
                context,
                &scope,
                &configuration,
                &checked,
                transformation,
                &source_inputs_manifest_sha256,
                source_tree_sha256,
            )
            .map_err(|error| {
                CliError::tool(format!(
                    "{} native reference bundle failed: {error}",
                    row.run_id
                ))
            })?;
            write_jcs(&run_dir.join("native-reference-bundle.json"), &bundle)?;
        }
        let derivation =
            run_composition_if_present(&workspace, row, &checked, &configuration, &run_dir, true)?;
        let aggregate = aggregate_fixture(
            &workspace,
            &run_dir,
            row,
            &checked,
            transformation_report.as_ref(),
            derivation.as_ref(),
        )?;
        results.push(aggregate);
    }
    results.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    write_jcs(
        &out_dir.join("fixture-results.json"),
        &FixtureResults {
            schema: "gluerift.fixture-results/v0.3.1a".into(),
            runs: results,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn evaluate_fixture_row(
    workspace: &Path,
    row: &FixtureRow,
    family: &TransformationFamilyDescriptor,
    scope: &ValidationScope,
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    configuration: &RunConfiguration,
    metadata: &EvidenceMetadata,
) -> Result<
    (
        gluerift::CheckedRun,
        Option<TransformationReport>,
        Option<AdapterContext>,
    ),
    CliError,
> {
    if !row.transformation_report_required {
        let context_path = workspace.join(&row.context_logical_path);
        let context: AdapterContext = read_json(&context_path)?;
        let run = check(&context, scope, policy, request, configuration, metadata)
            .map_err(map_check_error)?;
        return Ok((run, None, None));
    }
    let base: AdapterContext =
        read_json(&workspace.join(&row.transformation_base_context_logical_path))?;
    let base_run =
        check(&base, scope, policy, request, configuration, metadata).map_err(map_check_error)?;
    let request_parent = workspace
        .join(&row.request_logical_path)
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| CliError::tool(format!("{} request path has no parent", row.run_id)))?;
    let candidate: TransformationCandidate =
        read_json(&request_parent.join("transformation.json"))?;
    if canonical_sha256(&candidate.transformation_ir).map_err(canonical_error)?
        != row.transformation_sha256
    {
        return Err(CliError::tool(format!(
            "{} transformation hash mismatch",
            row.run_id
        )));
    }
    let classified = classify_transformation(
        &base,
        &candidate,
        family,
        scope,
        policy,
        request,
        configuration,
        metadata,
        if row.expected_base_alignment_status == ExpectedBindingStatus::NotRequired {
            None
        } else {
            Some(&base_run)
        },
    )
    .map_err(|error| {
        CliError::tool(format!(
            "{} transformation classification failed: {error}",
            row.run_id
        ))
    })?;
    let expected = parse_classification(&row.expected_transformation_classification)?;
    if classified.report.classification != expected
        || classified.report.candidate_binding_status
            != row.expected_candidate_binding_status.required_value()?
        || classified.report.base_alignment_status
            != row.expected_base_alignment_status.required_value()?
    {
        return Err(CliError::tool(format!(
            "{} transformation oracle mismatch",
            row.run_id
        )));
    }
    Ok((
        classified.transformed_run,
        Some(classified.report),
        Some(classified.transformed_context),
    ))
}

fn validate_registry_declarations(
    workspace: &Path,
    row: &FixtureRow,
    scope: &ValidationScope,
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    config: &RunConfiguration,
) -> Result<(), CliError> {
    let request_hash = canonical_sha256(request).map_err(canonical_error)?;
    let property_hash = canonical_sha256(&request.required_properties).map_err(canonical_error)?;
    if request.request_id != row.request_id
        || request_hash != row.validation_request_sha256
        || request.profile != row.profile
        || request.required_laws.ids() != row.required_law_ids
        || request.required_properties != row.required_properties
        || property_hash != row.required_properties_sha256
        || request.required_bridges != row.required_bridge_ids
        || request.required_transformation_family_sha256
            != row.required_transformation_family_sha256
        || canonical_sha256(&scope.comparator).map_err(canonical_error)?
            != row.comparator_spec_sha256
        || canonical_sha256(config).map_err(canonical_error)? != row.run_configuration_sha256
        || policy.match_coverage != row.match_coverage_mode
    {
        return Err(CliError::tool(format!(
            "{} declaration binding mismatch",
            row.run_id
        )));
    }
    if !row
        .required_witness_kinds
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        return Err(CliError::tool(format!(
            "{} witness-kind declaration is not canonical",
            row.run_id
        )));
    }
    if row.expected_law_statuses.len() != 6 {
        return Err(CliError::tool(format!(
            "{} must declare all six law result slots",
            row.run_id
        )));
    }
    let expected_native = match row.run_id.as_str() {
        "A01" => "E01",
        "A02" => "E02",
        _ => "not-applicable",
    };
    if row.native_replay_id != expected_native {
        return Err(CliError::tool(format!(
            "{} native replay binding must be {expected_native}",
            row.run_id
        )));
    }
    if row.transformation_report_required {
        if row.transformation_base_context_logical_path == "not-applicable"
            || !workspace
                .join(&row.transformation_base_context_logical_path)
                .is_file()
            || !row
                .context_logical_path
                .starts_with("artifact/staging/generated-contexts/")
            || row.transformation_sha256 == "not-applicable"
            || row.expected_candidate_binding_status == ExpectedBindingStatus::NotApplicable
            || row.expected_base_alignment_status == ExpectedBindingStatus::NotApplicable
        {
            return Err(CliError::tool(format!(
                "{} has incomplete transformation provenance declarations",
                row.run_id
            )));
        }
    } else if !workspace.join(&row.context_logical_path).is_file()
        || row.transformation_base_context_logical_path != "not-applicable"
        || row.transformation_sha256 != "not-applicable"
        || row.expected_candidate_binding_status != ExpectedBindingStatus::NotApplicable
        || row.expected_base_alignment_status != ExpectedBindingStatus::NotApplicable
        || row.expected_transformation_classification != "not-applicable"
    {
        return Err(CliError::tool(format!(
            "{} ordinary row contains transformation provenance or lacks its context",
            row.run_id
        )));
    }
    if matches!(row.run_id.as_str(), "A01" | "A02" | "A03" | "A05")
        && request.required_laws.ids().len() != 6
    {
        return Err(CliError::tool(format!(
            "{} attack request must explicitly require all six laws",
            row.run_id
        )));
    }
    Ok(())
}

fn validate_oracles(row: &FixtureRow, checked: &gluerift::CheckedRun) -> Result<(), CliError> {
    let report = &checked.check_report;
    if report.certification.profile_property_consistency_status
        != row.expected_profile_property_consistency
        || report.policy.match_coverage.status != row.expected_match_coverage_status
        || report.certification.safe_match_equality_status
            != row.expected_safe_match_equality_status
        || report.certification.eligible != row.expected_certificate_eligibility
        || report.certification.granted != row.expected_certificate_granted
        || report.comparison.comparator_definedness.status
            != row.expected_comparator_definedness_status
        || report.policy.policy_contract_status != row.expected_policy_contract_status
    {
        return Err(CliError::tool(format!(
            "{} top-level oracle mismatch",
            row.run_id
        )));
    }
    let laws: BTreeMap<_, _> = checked
        .roundtrip_report
        .laws
        .iter()
        .map(|law| (law.law_id, law.status))
        .collect();
    if laws != row.expected_law_statuses {
        return Err(CliError::tool(format!(
            "{} round-trip oracle mismatch",
            row.run_id
        )));
    }
    let properties = property_status_map(report);
    if properties != row.expected_property_statuses {
        return Err(CliError::tool(format!(
            "{} property oracle mismatch",
            row.run_id
        )));
    }
    let bridge_map = BTreeMap::from([
        (
            BridgeKind::CarrierTarget,
            report.bridges.carrier_target_bridge.status,
        ),
        (
            BridgeKind::CarrierSource,
            report.bridges.carrier_source_bridge.status,
        ),
    ]);
    if bridge_map != row.expected_bridge_statuses {
        return Err(CliError::tool(format!(
            "{} bridge oracle mismatch",
            row.run_id
        )));
    }
    let actual_witness_kinds: std::collections::BTreeSet<_> = checked
        .witnesses
        .values()
        .map(|witness| enum_string(witness.witness_kind))
        .collect::<Result<_, _>>()?;
    if !row
        .required_witness_kinds
        .iter()
        .all(|kind| actual_witness_kinds.contains(kind))
    {
        return Err(CliError::tool(format!(
            "{} required witness missing",
            row.run_id
        )));
    }
    Ok(())
}

fn aggregate_fixture(
    _workspace: &Path,
    _run_dir: &Path,
    row: &FixtureRow,
    run: &gluerift::CheckedRun,
    transformation: Option<&TransformationReport>,
    derivation: Option<&gluerift::report::DerivationReport>,
) -> Result<FixtureResultRow, CliError> {
    let report = &run.check_report;
    let properties = PropertyStatusesAggregate {
        policy_soundness: report.properties.policy_soundness.status,
        comparison_adequacy: report.properties.comparison_adequacy.status,
        comparison_precision: report.properties.comparison_precision.status,
        faithful_comparison: report.properties.faithful_comparison.status,
        target_non_amplification_aggregate: report
            .properties
            .target_non_amplification
            .aggregate_status,
        target_non_amplification_by_dimension: report
            .properties
            .target_non_amplification
            .dimensions
            .iter()
            .map(|dimension| (dimension.dimension_id.clone(), dimension.status))
            .collect(),
    };
    let mut property_witnesses = Vec::new();
    for (property_id, result) in [
        ("policy-soundness", &report.properties.policy_soundness),
        (
            "comparison-adequacy",
            &report.properties.comparison_adequacy,
        ),
        (
            "comparison-precision",
            &report.properties.comparison_precision,
        ),
        (
            "faithful-comparison",
            &report.properties.faithful_comparison,
        ),
    ] {
        if result.witness_sha256 == "not-applicable" {
            continue;
        }
        let witness = run.witnesses.get(&result.witness_sha256).ok_or_else(|| {
            CliError::tool(format!(
                "unresolved property witness {}",
                result.witness_sha256
            ))
        })?;
        property_witnesses.push(PropertyWitnessAggregate {
            property_id: property_id.into(),
            witness_kind: enum_string(witness.witness_kind)?,
            witness_sha256: result.witness_sha256.clone(),
        });
    }
    for dimension in &report.properties.target_non_amplification.dimensions {
        if dimension.witness_sha256 == "not-applicable" {
            continue;
        }
        let witness = run
            .witnesses
            .get(&dimension.witness_sha256)
            .ok_or_else(|| {
                CliError::tool(format!(
                    "unresolved TNA witness {}",
                    dimension.witness_sha256
                ))
            })?;
        property_witnesses.push(PropertyWitnessAggregate {
            property_id: format!("target-non-amplification/{}", dimension.dimension_id),
            witness_kind: enum_string(witness.witness_kind)?,
            witness_sha256: dimension.witness_sha256.clone(),
        });
    }
    property_witnesses.sort_by(|a, b| {
        (&a.property_id, &a.witness_kind, &a.witness_sha256).cmp(&(
            &b.property_id,
            &b.witness_kind,
            &b.witness_sha256,
        ))
    });
    let policy_witnesses = [
        &report.policy.match_coverage.empty_match_witness_sha256,
        &report.policy.match_coverage.unmatched_source_witness_sha256,
        &report.policy.match_coverage.unmatched_target_witness_sha256,
        &report.certification.safe_match_equality_witness_sha256,
    ]
    .into_iter()
    .filter(|hash| hash.as_str() != "not-applicable")
    .cloned()
    .collect();
    let transformations = if let Some(item) = transformation {
        vec![TransformationAggregate {
            transformation_report_sha256: canonical_sha256(item).map_err(canonical_error)?,
            base_context_sha256: item.candidate_context_sha256.clone(),
            base_check_report_sha256: item.base_check_report_sha256.clone(),
            base_alignment_status: item.base_alignment_status,
            transformed_context_sha256: item.transformed_context_sha256.clone(),
            transformed_check_report_sha256: item.transformed_check_report_sha256.clone(),
            candidate_binding_status: item.candidate_binding_status,
            transformation_sha256: item.transformation_sha256.clone(),
            inverse_sha256: item.inverse_sha256.clone(),
            action_domain_sha256: item.action_domain_sha256.clone(),
            lawfulness_status: item.lawfulness_status,
            classification: item.classification,
            harmful_witness_sha256: item.harmful_witness_sha256.clone(),
        }]
    } else {
        Vec::new()
    };
    Ok(FixtureResultRow {
        run_id: row.run_id.clone(),
        fixture_kind: row.fixture_kind.clone(),
        check_report_sha256: canonical_sha256(report).map_err(canonical_error)?,
        check_report_evidence_id: report.envelope.evidence_id.clone(),
        check_report_logical_path: semantic_logical_path(&row.run_id, "check.json"),
        validation_request_sha256: report.envelope.validation_request_sha256.clone(),
        candidate_sha256: report.envelope.candidate_sha256.clone(),
        comparator_kind: enum_string(report.comparison.comparator_kind)?,
        profile: ProfileAggregate {
            requested_profile: report.certification.requested_profile.clone(),
            profile_property_consistency_status: report
                .certification
                .profile_property_consistency_status,
            safe_match_equality_status: report.certification.safe_match_equality_status,
            safe_match_equality_witness_sha256: report
                .certification
                .safe_match_equality_witness_sha256
                .clone(),
        },
        match_coverage: report.policy.match_coverage.clone(),
        six_roundtrip_statuses: run
            .roundtrip_report
            .laws
            .iter()
            .map(|law| (law.law_id, law.status))
            .collect(),
        comparator_definedness: report.comparison.comparator_definedness.status,
        bridge_statuses: BTreeMap::from([
            (
                "carrier_target".into(),
                report.bridges.carrier_target_bridge.status,
            ),
            (
                "carrier_source".into(),
                report.bridges.carrier_source_bridge.status,
            ),
            (
                "selected_carrier_bridge".into(),
                report.bridges.selected_carrier_bridge_status,
            ),
        ]),
        policy_contract_status: report.policy.policy_contract_status.clone(),
        policy_vacuity_warning: report.policy.policy_vacuity_warning,
        policy_witnesses,
        property_statuses: properties,
        property_witnesses,
        certification: CertificationAggregate {
            eligible: report.certification.eligible,
            granted: report.certification.granted,
            blocking_reasons: report.certification.blocking_reasons.clone(),
        },
        transformation_results: transformations,
        native_replay_id: row.native_replay_id.clone(),
        derivation_status: derivation.map_or(Status::NotRequested, |report| report.envelope.status),
        derivation_report_sha256: derivation
            .map(canonical_sha256)
            .transpose()
            .map_err(canonical_error)?
            .unwrap_or_else(|| "not-applicable".into()),
    })
}

fn run_composition_if_present(
    workspace: &Path,
    row: &FixtureRow,
    run: &gluerift::CheckedRun,
    config: &RunConfiguration,
    run_dir: &Path,
    emit: bool,
) -> Result<Option<gluerift::report::DerivationReport>, CliError> {
    let path = workspace
        .join(&row.request_logical_path)
        .parent()
        .map(|parent| parent.join("composition.json"))
        .ok_or_else(|| CliError::tool(format!("{} request path has no parent", row.run_id)))?;
    if !path.is_file() {
        return Ok(None);
    }
    let request: CompositionRequest = read_json(&path)?;
    let check = &run.check_report;
    let envelope = CommonEnvelope {
        schema: "gluerift.derivation-report/v0.3.1a".into(),
        semantic_contract_version: CONTRACT_VERSION.into(),
        tool_build_sha256: check.envelope.tool_build_sha256.clone(),
        run_configuration_sha256: check.envelope.run_configuration_sha256.clone(),
        evidence_id: format!("{}:derivation", row.run_id),
        candidate_sha256: check.envelope.candidate_sha256.clone(),
        types_sha256: check.envelope.types_sha256.clone(),
        validation_scope_sha256: check.envelope.validation_scope_sha256.clone(),
        endpoint_policy_sha256: check.envelope.endpoint_policy_sha256.clone(),
        validation_request_sha256: check.envelope.validation_request_sha256.clone(),
        comparator_spec_sha256: check.envelope.comparator_spec_sha256.clone(),
        dependency_evidence_ids: vec![check.envelope.evidence_id.clone()],
        status: Status::NotRequested,
    };
    let report = compose_and_check(&request, &config.enumeration_limits(), envelope)
        .map_err(|error| CliError::tool(format!("{} composition failure: {error}", row.run_id)))?;
    if emit {
        write_jcs(&run_dir.join("derivation.json"), &report)?;
    }
    Ok(Some(report))
}

fn property_status_map(report: &CheckReport) -> BTreeMap<String, Status> {
    BTreeMap::from([
        (
            "policy-soundness".into(),
            report.properties.policy_soundness.status,
        ),
        (
            "comparison-adequacy".into(),
            report.properties.comparison_adequacy.status,
        ),
        (
            "comparison-precision".into(),
            report.properties.comparison_precision.status,
        ),
        (
            "faithful-comparison".into(),
            report.properties.faithful_comparison.status,
        ),
        (
            "target-non-amplification".into(),
            report.properties.target_non_amplification.aggregate_status,
        ),
    ])
}

fn run_baselines(registry: &Path, baselines: &str, out_dir: &Path) -> Result<(), CliError> {
    let selected: std::collections::BTreeSet<_> = baselines.split(',').map(str::trim).collect();
    if !selected.iter().all(|id| matches!(*id, "BL2" | "BL4")) {
        return Err(CliError::invalid("Core supports only BL2 and BL4"));
    }
    let workspace = workspace_root(registry)?;
    let registry_data: FixtureRegistry = read_json(registry)?;
    let configuration: RunConfiguration =
        read_json(&workspace.join("spec/run-config/core-v0.3.1a.json"))?;
    let family: TransformationFamilyDescriptor =
        read_json(&workspace.join("spec/transformation-families/core-structural-v0.3.1a.json"))?;
    let metadata = EvidenceMetadata::for_current_executable().map_err(io_error)?;
    let mut index = Vec::new();
    for row in &registry_data.runs {
        if !row.bl2_paired && !row.bl4_paired {
            continue;
        }
        let run_dir = out_dir.join("baselines").join(sanitize(&row.run_id));
        let scope: ValidationScope = read_json(&workspace.join(&row.scope_logical_path))?;
        let policy: EndpointPolicy = read_json(&workspace.join(&row.policy_logical_path))?;
        let request: ValidationRequest = read_json(&workspace.join(&row.request_logical_path))?;
        validate_registry_declarations(&workspace, row, &scope, &policy, &request, &configuration)?;
        let (checked, _, _) = evaluate_fixture_row(
            &workspace,
            row,
            &family,
            &scope,
            &policy,
            &request,
            &configuration,
            &metadata,
        )?;
        let derivation =
            run_composition_if_present(&workspace, row, &checked, &configuration, &run_dir, false)?;
        let mut index_row = BaselineResultsRow {
            run_id: row.run_id.clone(),
            bl2_result: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            bl4_result: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
        };
        if row.bl2_paired && selected.contains("BL2") {
            let baseline = roundtrip_baseline_report(&checked, row, &metadata, &configuration)?;
            let path = run_dir.join("BL2.json");
            write_jcs(&path, &baseline)?;
            index_row.bl2_result = EvidenceValue::Present(Bl2IndexResult {
                report_logical_path: baseline_logical_path(&row.run_id, "BL2.json"),
                report_sha256: canonical_sha256(&baseline).map_err(canonical_error)?,
                evidence_id: baseline.envelope.evidence_id.clone(),
                law_statuses: baseline.law_statuses.clone(),
            });
        }
        if row.bl4_paired && selected.contains("BL4") {
            let baseline = baseline_report(
                &checked,
                row,
                &metadata,
                &configuration,
                "BL4",
                derivation.as_ref(),
                true,
            )?;
            let path = run_dir.join("BL4.json");
            write_jcs(&path, &baseline)?;
            index_row.bl4_result = EvidenceValue::Present(Bl4IndexResult {
                report_logical_path: baseline_logical_path(&row.run_id, "BL4.json"),
                report_sha256: canonical_sha256(&baseline).map_err(canonical_error)?,
                evidence_id: baseline.envelope.evidence_id.clone(),
                common_validity_statuses: BTreeMap::from([
                    (
                        "profile_property_consistency".into(),
                        baseline.profile_property_consistency,
                    ),
                    ("match_subset_safe".into(), baseline.match_subset_safe),
                    ("safe_anchor_coverage".into(), baseline.safe_anchor_coverage),
                    (
                        "match_anchor_coverage".into(),
                        baseline.match_anchor_coverage,
                    ),
                    (
                        "match_shape_compatibility".into(),
                        baseline.match_shape_compatibility,
                    ),
                    (
                        "comparator_definedness".into(),
                        baseline.comparator_definedness,
                    ),
                ]),
                match_coverage: baseline.match_coverage.clone(),
                policy_contract_status: baseline.policy_contract_status.clone(),
                policy_witnesses: baseline.policy_witnesses.clone(),
                property_statuses: baseline.property_statuses.clone(),
                target_non_amplification: baseline.target_non_amplification.clone(),
                property_witnesses: baseline.property_witnesses.clone(),
                validity_parity_status: baseline.validity_parity_status,
                coverage_parity_status: baseline.coverage_parity_status,
                policy_parity_status: baseline.policy_parity_status,
                property_parity_status: baseline.property_parity_status,
                witness_parity_status: baseline.witness_parity_status,
            });
        }
        index.push(index_row);
    }
    index.sort_by(|a, b| a.run_id.cmp(&b.run_id));
    write_jcs(
        &out_dir.join("baseline-results.json"),
        &BaselineResultsIndex {
            schema: "gluerift.baseline-results/v0.3.1a".into(),
            runs: index,
        },
    )
}

fn roundtrip_baseline_report(
    run: &gluerift::CheckedRun,
    row: &FixtureRow,
    metadata: &EvidenceMetadata,
    config: &RunConfiguration,
) -> Result<RoundTripBaselineReport, CliError> {
    let report = &run.check_report;
    let law_statuses: BTreeMap<_, _> = run
        .roundtrip_report
        .laws
        .iter()
        .map(|law| (law.law_id, law.status))
        .collect();
    let status = if law_statuses
        .values()
        .all(|status| *status == Status::ProvedExhaustive)
    {
        Status::ProvedExhaustive
    } else {
        Status::Disproved
    };
    Ok(RoundTripBaselineReport {
        envelope: CommonEnvelope {
            schema: "gluerift.baseline-report/v0.3.1a".into(),
            semantic_contract_version: CONTRACT_VERSION.into(),
            tool_build_sha256: metadata.tool_build_sha256.clone(),
            run_configuration_sha256: canonical_sha256(config).map_err(canonical_error)?,
            evidence_id: format!("{}:baseline:BL2", row.run_id),
            candidate_sha256: report.envelope.candidate_sha256.clone(),
            types_sha256: report.envelope.types_sha256.clone(),
            validation_scope_sha256: report.envelope.validation_scope_sha256.clone(),
            endpoint_policy_sha256: report.envelope.endpoint_policy_sha256.clone(),
            validation_request_sha256: report.envelope.validation_request_sha256.clone(),
            comparator_spec_sha256: report.envelope.comparator_spec_sha256.clone(),
            dependency_evidence_ids: vec![run.roundtrip_report.envelope.evidence_id.clone()],
            status,
        },
        baseline_id: "BL2".into(),
        paired_check_report_sha256: canonical_sha256(report).map_err(canonical_error)?,
        law_statuses,
    })
}

fn baseline_report(
    run: &gluerift::CheckedRun,
    row: &FixtureRow,
    metadata: &EvidenceMetadata,
    config: &RunConfiguration,
    baseline_id: &str,
    derivation: Option<&gluerift::report::DerivationReport>,
    direct_relation: bool,
) -> Result<BaselineReport, CliError> {
    let report = &run.check_report;
    let (properties, witness_parity) = if direct_relation {
        direct_relation_projection(run)?
    } else {
        (
            BTreeMap::from([
                ("policy-soundness".into(), Status::NotRequested),
                ("comparison-adequacy".into(), Status::NotRequested),
                ("comparison-precision".into(), Status::NotRequested),
                ("faithful-comparison".into(), Status::NotRequested),
                ("target-non-amplification".into(), Status::NotRequested),
            ]),
            Status::NotRequested,
        )
    };
    let witnesses = BTreeMap::from([
        (
            "policy-soundness".into(),
            report.properties.policy_soundness.witness_sha256.clone(),
        ),
        (
            "comparison-adequacy".into(),
            report.properties.comparison_adequacy.witness_sha256.clone(),
        ),
        (
            "comparison-precision".into(),
            report
                .properties
                .comparison_precision
                .witness_sha256
                .clone(),
        ),
        (
            "faithful-comparison".into(),
            report.properties.faithful_comparison.witness_sha256.clone(),
        ),
    ]);
    let law_statuses: BTreeMap<_, _> = run
        .roundtrip_report
        .laws
        .iter()
        .map(|law| (law.law_id, law.status))
        .collect();
    let property_parity = if !direct_relation || properties == property_status_map(report) {
        Status::ProvedExhaustive
    } else {
        Status::ToolError
    };
    if direct_relation
        && (property_parity != Status::ProvedExhaustive
            || witness_parity != Status::ProvedExhaustive)
    {
        return Err(CliError::tool(format!(
            "{} BL4 Direct-Relation parity failure",
            row.run_id
        )));
    }
    let derivation_hash = derivation
        .map(canonical_sha256)
        .transpose()
        .map_err(canonical_error)?
        .unwrap_or_else(|| "not-applicable".into());
    let mut dependency_evidence_ids = vec![report.envelope.evidence_id.clone()];
    if let Some(derivation) = derivation {
        dependency_evidence_ids.push(derivation.envelope.evidence_id.clone());
    }
    Ok(BaselineReport {
        envelope: CommonEnvelope {
            schema: "gluerift.baseline-report/v0.3.1a".into(),
            semantic_contract_version: CONTRACT_VERSION.into(),
            tool_build_sha256: metadata.tool_build_sha256.clone(),
            run_configuration_sha256: canonical_sha256(config).map_err(canonical_error)?,
            evidence_id: format!("{}:baseline:{baseline_id}", row.run_id),
            candidate_sha256: report.envelope.candidate_sha256.clone(),
            types_sha256: report.envelope.types_sha256.clone(),
            validation_scope_sha256: report.envelope.validation_scope_sha256.clone(),
            endpoint_policy_sha256: report.envelope.endpoint_policy_sha256.clone(),
            validation_request_sha256: report.envelope.validation_request_sha256.clone(),
            comparator_spec_sha256: report.envelope.comparator_spec_sha256.clone(),
            dependency_evidence_ids,
            status: if direct_relation {
                report.envelope.status
            } else if law_statuses
                .values()
                .all(|status| *status == Status::ProvedExhaustive)
            {
                Status::ProvedExhaustive
            } else {
                Status::Disproved
            },
        },
        baseline_id: baseline_id.into(),
        paired_check_report_sha256: canonical_sha256(report).map_err(canonical_error)?,
        law_statuses,
        profile_property_consistency: if direct_relation {
            report.certification.profile_property_consistency_status
        } else {
            Status::NotRequested
        },
        match_subset_safe: if direct_relation {
            report.policy.match_subset_safe_status
        } else {
            Status::NotRequested
        },
        safe_anchor_coverage: report.policy.safe_anchor_coverage.status,
        match_anchor_coverage: report.policy.match_anchor_coverage.status,
        match_shape_compatibility: report.policy.match_shape_compatibility,
        comparator_definedness: report.comparison.comparator_definedness.status,
        match_coverage_status: report.policy.match_coverage.status,
        match_coverage: report.policy.match_coverage.clone(),
        policy_contract_status: report.policy.policy_contract_status.clone(),
        policy_witnesses: vec![
            report
                .policy
                .match_coverage
                .empty_match_witness_sha256
                .clone(),
            report
                .policy
                .match_coverage
                .unmatched_source_witness_sha256
                .clone(),
            report
                .policy
                .match_coverage
                .unmatched_target_witness_sha256
                .clone(),
        ]
        .into_iter()
        .filter(|hash| hash != "not-applicable")
        .collect(),
        property_statuses: properties,
        property_witnesses: witnesses,
        target_non_amplification: report.properties.target_non_amplification.clone(),
        derivation_report_sha256: derivation_hash,
        derivation_parity_status: if derivation.is_some() && direct_relation {
            Status::ProvedExhaustive
        } else {
            Status::NotRequested
        },
        validity_parity_status: if direct_relation {
            Status::ProvedExhaustive
        } else {
            Status::NotRequested
        },
        coverage_parity_status: if direct_relation {
            Status::ProvedExhaustive
        } else {
            Status::NotRequested
        },
        policy_parity_status: if direct_relation {
            Status::ProvedExhaustive
        } else {
            Status::NotRequested
        },
        property_parity_status: property_parity,
        witness_parity_status: witness_parity,
    })
}

fn direct_relation_projection(
    run: &gluerift::CheckedRun,
) -> Result<(BTreeMap<String, Status>, Status), CliError> {
    let report = &run.check_report;
    let direct = |current: &gluerift::report::PropertyResult, holds: bool| {
        if current.status == Status::NotRequested {
            Status::NotRequested
        } else if matches!(
            current.status,
            Status::Invalid | Status::Unknown | Status::ToolError
        ) {
            current.status
        } else if holds {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        }
    };
    let statuses = BTreeMap::from([
        (
            "policy-soundness".into(),
            direct(
                &report.properties.policy_soundness,
                run.induced_relation.is_subset(&run.safe),
            ),
        ),
        (
            "comparison-adequacy".into(),
            direct(
                &report.properties.comparison_adequacy,
                run.matched.is_subset(&run.induced_relation),
            ),
        ),
        (
            "comparison-precision".into(),
            direct(
                &report.properties.comparison_precision,
                run.induced_relation.is_subset(&run.matched),
            ),
        ),
        (
            "faithful-comparison".into(),
            direct(
                &report.properties.faithful_comparison,
                run.induced_relation == run.matched,
            ),
        ),
        (
            "target-non-amplification".into(),
            report.properties.target_non_amplification.aggregate_status,
        ),
    ]);
    let expected_pairs = [
        (
            &report.properties.policy_soundness,
            run.induced_relation.difference(&run.safe).next(),
        ),
        (
            &report.properties.comparison_adequacy,
            run.matched.difference(&run.induced_relation).next(),
        ),
        (
            &report.properties.comparison_precision,
            run.induced_relation.difference(&run.matched).next(),
        ),
        (
            &report.properties.faithful_comparison,
            run.induced_relation
                .symmetric_difference(&run.matched)
                .next(),
        ),
    ];
    for (property, expected) in expected_pairs {
        if property.status != Status::Disproved {
            continue;
        }
        let witness = run
            .witnesses
            .get(&property.witness_sha256)
            .ok_or_else(|| CliError::tool("BL4 parity witness hash is unresolved"))?;
        let (source, target) = match (&witness.source_value, &witness.target_value) {
            (EvidenceValue::Present(source), EvidenceValue::Present(target)) => (source, target),
            _ => return Err(CliError::tool("BL4 pair witness is typed not-applicable")),
        };
        let pair = gluerift::domain::ValuePair {
            source: source.clone(),
            target: target.clone(),
        };
        if expected != Some(&pair) {
            return Err(CliError::tool("BL4 first witness pair mismatch"));
        }
    }
    Ok((statuses, Status::ProvedExhaustive))
}

fn replay_native(manifest: &Path, out: &Path) -> Result<(), CliError> {
    let workspace = workspace_root(manifest)?;
    let driver = workspace.join("native/scripts/reproduce");
    if !driver.is_file() {
        return Err(CliError::tool("native/scripts/reproduce driver is absent"));
    }
    let status = ProcessCommand::new(&driver)
        .arg("--bindings")
        .arg(manifest)
        .arg("--out-dir")
        .arg(out)
        .arg("--logical-out-prefix")
        .arg("artifact/staging/native")
        .current_dir(&workspace)
        .env_clear()
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .env("TZ", "UTC")
        .status()
        .map_err(io_error)?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::tool(format!(
            "native replay exited with {status}"
        )))
    }
}

fn reproduce(profile: &str, out_dir: &Path) -> Result<(), CliError> {
    if profile != "core" {
        return Err(CliError::invalid("only the core profile is supported"));
    }
    let workspace = workspace_root(Path::new("."))?;
    let script = workspace.join("artifact/reproduce");
    let status = ProcessCommand::new(script)
        .arg("--out-dir")
        .arg(out_dir)
        .current_dir(&workspace)
        .env("GLUERIFT_FROM_CLI", "1")
        .status()
        .map_err(io_error)?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::tool(format!("reproduction exited with {status}")))
    }
}

fn parse_classification(value: &str) -> Result<TransformationClassification, CliError> {
    match value {
        "lawful-safe" => Ok(TransformationClassification::LawfulSafe),
        "lawful-harmful" => Ok(TransformationClassification::LawfulHarmful),
        "law-breaking-or-inapplicable" => {
            Ok(TransformationClassification::LawBreakingOrInapplicable)
        }
        other => Err(CliError::invalid(format!(
            "invalid transformation classification `{other}`"
        ))),
    }
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, CliError> {
    let bytes = fs::read(path).map_err(io_error)?;
    serde_json::from_slice(&bytes)
        .map_err(|error| CliError::invalid(format!("{}: {error}", path.display())))
}

fn write_jcs<T: Serialize>(path: &Path, value: &T) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }
    let bytes = canonical_bytes(value).map_err(canonical_error)?;
    fs::write(path, bytes).map_err(io_error)
}

fn enum_string<T: Serialize>(value: T) -> Result<String, CliError> {
    let json = serde_json::to_value(value).map_err(json_error)?;
    json.as_str()
        .map(str::to_owned)
        .ok_or_else(|| CliError::tool("enum did not serialize as a string"))
}

fn semantic_logical_path(run_id: &str, leaf: &str) -> String {
    format!(
        "artifact/evidence/semantic/runs/{}/{leaf}",
        sanitize(run_id)
    )
}

fn baseline_logical_path(run_id: &str, leaf: &str) -> String {
    format!("artifact/evidence/baselines/{}/{leaf}", sanitize(run_id))
}

fn workspace_root(path: &Path) -> Result<PathBuf, CliError> {
    let start = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().map_err(io_error)?.join(path)
    };
    let mut cursor = if start.is_dir() {
        start
    } else {
        start.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    loop {
        if cursor
            .join("ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md")
            .is_file()
        {
            return Ok(cursor);
        }
        if !cursor.pop() {
            break;
        }
    }
    Err(CliError::tool("could not locate workspace root"))
}

fn sanitize(run_id: &str) -> String {
    run_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
fn io_error(error: std::io::Error) -> CliError {
    CliError::tool(error.to_string())
}
fn json_error(error: serde_json::Error) -> CliError {
    CliError::invalid(error.to_string())
}
fn canonical_error(error: impl std::fmt::Display) -> CliError {
    CliError::tool(error.to_string())
}

fn map_check_error(error: CheckError) -> CliError {
    let message = error.to_string();
    let resource_failure = matches!(
        &error,
        CheckError::UniverseLimit { .. }
            | CheckError::Adapter(AdapterTypeError::Type(TypeError::ResourceLimit(_)))
            | CheckError::Domain(DomainError::Type(TypeError::ResourceLimit(_)))
    );
    if resource_failure {
        return CliError::tool(message);
    }
    if matches!(
        error,
        CheckError::Adapter(_)
            | CheckError::Domain(_)
            | CheckError::Policy(_)
            | CheckError::WrongRequestSchema
            | CheckError::WrongRunConfigurationSchema
            | CheckError::NonCanonicalRunConfiguration
            | CheckError::NonCanonicalBridges
    ) {
        CliError::invalid(message)
    } else {
        CliError::tool(message)
    }
}

fn aggregate_cli_status(statuses: impl IntoIterator<Item = Status>) -> Status {
    let statuses: Vec<_> = statuses.into_iter().collect();
    if statuses.contains(&Status::ToolError) {
        Status::ToolError
    } else if statuses.contains(&Status::Unknown) {
        Status::Unknown
    } else if statuses.contains(&Status::Invalid) {
        Status::Invalid
    } else if statuses.contains(&Status::Disproved) {
        Status::Disproved
    } else {
        Status::ProvedExhaustive
    }
}

fn ensure_direct_status(status: Status) -> Result<(), CliError> {
    match status {
        Status::ProvedExhaustive | Status::NotRequested => Ok(()),
        Status::Disproved => Err(CliError::semantic(
            "one or more requested semantic obligations were disproved",
        )),
        Status::Invalid => Err(CliError::invalid(
            "validation request or one of its prerequisites is invalid",
        )),
        Status::Unknown => Err(CliError::unknown(
            "one or more requested obligations have unknown status",
        )),
        Status::ToolError => Err(CliError::tool(
            "one or more requested obligations failed due to a tool error",
        )),
    }
}
