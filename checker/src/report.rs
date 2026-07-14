use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::domain::ComparatorSpec;
use crate::relation_ir::MatchCoverageMode;
use crate::type_ir::Value;

pub const NOT_APPLICABLE: &str = "not-applicable";
pub const NOT_REQUIRED_DIRECT_NATIVE: &str = "not-required-direct-native";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AbsentEvidence {
    NotApplicable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EvidenceValue<T> {
    Present(T),
    Absent(AbsentEvidence),
}

impl<T> EvidenceValue<T> {
    pub fn from_option(value: Option<T>) -> Self {
        value
            .map(Self::Present)
            .unwrap_or(Self::Absent(AbsentEvidence::NotApplicable))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    ProvedExhaustive,
    Disproved,
    Unknown,
    NotRequested,
    Invalid,
    ToolError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BindingStatus {
    ProvedExhaustive,
    NotRequired,
    ToolError,
}

impl Status {
    pub fn is_proved(self) -> bool {
        self == Self::ProvedExhaustive
    }
    pub fn is_terminal_error(self) -> bool {
        matches!(self, Self::Invalid | Self::ToolError)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommonEnvelope {
    pub schema: String,
    pub semantic_contract_version: String,
    pub tool_build_sha256: String,
    pub run_configuration_sha256: String,
    pub evidence_id: String,
    pub candidate_sha256: String,
    pub types_sha256: String,
    pub validation_scope_sha256: String,
    pub endpoint_policy_sha256: String,
    pub validation_request_sha256: String,
    pub comparator_spec_sha256: String,
    pub dependency_evidence_ids: Vec<String>,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyResult {
    pub status: Status,
    pub checked_pair_count: usize,
    pub witness_sha256: String,
}

impl PropertyResult {
    pub fn not_requested() -> Self {
        Self {
            status: Status::NotRequested,
            checked_pair_count: 0,
            witness_sha256: NOT_APPLICABLE.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparatorDefinednessReport {
    pub status: Status,
    pub checked_input_count: usize,
    pub witness_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonReport {
    pub comparator_kind: ComparatorSpec,
    pub comparator_spec_sha256: String,
    pub universe_pair_count: usize,
    pub induced_equality_pair_count: usize,
    pub comparator_definedness: ComparatorDefinednessReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeReference {
    pub status: Status,
    pub report_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgesReport {
    pub carrier_target_bridge: BridgeReference,
    pub carrier_source_bridge: BridgeReference,
    pub selected_carrier_bridge_status: Status,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyContractStatus {
    PolicyUnconstrained,
    UniversalDeclared,
    Constrained,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchCoverageReport {
    pub mode: MatchCoverageMode,
    pub status: Status,
    pub source_comparison_domain_sha256: String,
    pub target_comparison_domain_sha256: String,
    pub source_comparison_domain_count: usize,
    pub target_comparison_domain_count: usize,
    pub matched_source_count: usize,
    pub matched_target_count: usize,
    pub empty_match_witness_sha256: String,
    pub unmatched_source_witness_sha256: String,
    pub unmatched_target_witness_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageReport {
    pub status: Status,
    pub relevant_path_count: usize,
    pub observed_path_count: usize,
    pub explicitly_irrelevant_path_count: usize,
    pub uncovered_paths: Vec<EndpointPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointPath {
    pub endpoint: String,
    pub path: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyReport {
    pub safe_dimension_count: usize,
    pub safe_pair_count: usize,
    pub unsafe_pair_count: usize,
    pub safe_is_universal: bool,
    pub policy_contract_status: PolicyContractStatus,
    pub policy_vacuity_warning: bool,
    pub match_dimension_count: usize,
    pub match_pair_count: usize,
    pub match_subset_safe_status: Status,
    pub match_coverage: MatchCoverageReport,
    pub match_shape_compatibility: Status,
    pub safe_anchor_coverage: CoverageReport,
    pub match_anchor_coverage: CoverageReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TnaDimensionResult {
    pub dimension_id: String,
    pub preorder_sha256: String,
    pub status: Status,
    pub checked_pair_count: usize,
    pub witness_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetNonAmplificationReport {
    pub aggregate_status: Status,
    pub checked_dimension_count: usize,
    pub checked_pair_count: usize,
    pub dimensions: Vec<TnaDimensionResult>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertiesReport {
    pub policy_soundness: PropertyResult,
    pub comparison_adequacy: PropertyResult,
    pub comparison_precision: PropertyResult,
    pub faithful_comparison: PropertyResult,
    pub target_non_amplification: TargetNonAmplificationReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CertificationReport {
    pub requested_profile: String,
    pub profile_property_consistency_status: Status,
    pub minimum_required_property_kinds: Vec<String>,
    pub explicit_required_property_kinds: Vec<String>,
    pub extra_required_property_kinds: Vec<String>,
    pub explicit_required_law_ids: Vec<LawId>,
    pub safe_match_equality_status: Status,
    pub safe_match_equality_witness_sha256: String,
    pub eligible: bool,
    pub granted: bool,
    pub blocking_reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub comparison: ComparisonReport,
    pub bridges: BridgesReport,
    pub policy: PolicyReport,
    pub properties: PropertiesReport,
    pub certification: CertificationReport,
    pub roundtrip_report_sha256: String,
    pub carrier_summary_sha256: String,
    pub witness_sha256s: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LawId {
    SourceNative,
    TargetNative,
    SourceCarrier,
    TargetCarrier,
    SourceFullTransport,
    TargetFullTransport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundTripLawReport {
    pub law_id: LawId,
    pub domain_sha256: String,
    pub declared_input_count: usize,
    pub checked_input_count: usize,
    pub status: Status,
    pub transport_coverage_status: EvidenceValue<Status>,
    pub final_equality_status: Status,
    pub execution_trace_table_sha256: String,
    pub first_failing_input: EvidenceValue<Value>,
    pub first_failure_trace: Vec<StageTrace>,
    pub witness_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTrace {
    pub stage: String,
    pub result: Result<Value, crate::adapter_ir::ConversionError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionTraceRow {
    pub input: Value,
    pub stages: Vec<StageTrace>,
    pub final_matches_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionTraceTableReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub law_id: LawId,
    pub rows: Vec<ExecutionTraceRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundTripReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub laws: Vec<RoundTripLawReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BridgeKind {
    CarrierTarget,
    CarrierSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub bridge_kind: BridgeKind,
    pub universe_pair_count: usize,
    pub checked_pair_count: usize,
    pub counterexample_pair: EvidenceValue<crate::domain::ValuePair>,
    pub carrier_comparator_evidence: crate::witness::ComparatorEvidence,
    pub native_comparator_evidence: crate::witness::ComparatorEvidence,
    pub sufficient_rule_coverage: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarrierSummary {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub source_successful_image: Vec<Value>,
    pub target_successful_image: Vec<Value>,
    pub shared_carrier_classes: Vec<Value>,
    pub class_endpoint_pairs: Vec<CarrierClassPair>,
    pub class_observation_conflicts: Vec<String>,
    pub evidence_basis: String,
    pub applicability_to_selected_comparator: String,
    pub bridge_report_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarrierClassPair {
    pub carrier: Value,
    pub source: Value,
    pub target: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivationReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub judgment_kind: String,
    pub relation_kind: String,
    pub adapter_path: String,
    pub observer_paths: Vec<Vec<String>>,
    pub input_domain_sha256: String,
    pub output_domain_sha256: String,
    pub children: Vec<String>,
    pub relation_bridge: String,
    pub exhaustive_crosscheck_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransformationClassification {
    LawfulSafe,
    LawfulHarmful,
    LawBreakingOrInapplicable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformationReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub transformation_family_sha256: String,
    pub generation_mode: String,
    pub generation_rule_id: String,
    pub generation_parent_path: Vec<String>,
    pub generation_ordinal: usize,
    pub transformation_ir: crate::adapter_ir::Adapter,
    pub transformation_sha256: String,
    pub inverse_ir: crate::adapter_ir::Adapter,
    pub inverse_sha256: String,
    pub inverse_check_status: Status,
    pub action_domain: Vec<Value>,
    pub action_domain_sha256: String,
    pub twist_side: String,
    pub twist_construction: String,
    pub comparator_spec_sha256: String,
    pub candidate_context_sha256: String,
    pub base_check_report_sha256: String,
    pub base_alignment_status: BindingStatus,
    pub base_source_encode_sha256: String,
    pub base_source_decode_sha256: String,
    pub base_target_encode_sha256: String,
    pub base_target_decode_sha256: String,
    pub transformed_context_sha256: String,
    pub transformed_check_report_sha256: String,
    pub candidate_binding_status: BindingStatus,
    pub transformed_source_encode_sha256: String,
    pub transformed_source_decode_sha256: String,
    pub transformed_target_encode_sha256: String,
    pub transformed_target_decode_sha256: String,
    pub four_map_construction_status: Status,
    pub four_map_semantics_check_sha256: String,
    pub well_typed_status: Status,
    pub comparator_definedness_status: Status,
    pub requested_law_ids: Vec<LawId>,
    pub roundtrip_statuses: BTreeMap<LawId, Status>,
    pub lawfulness_status: Status,
    pub classification: TransformationClassification,
    pub inapplicability_reasons: Vec<String>,
    pub selected_property_statuses: BTreeMap<String, Status>,
    pub harmful_witness_sha256: String,
    pub transformed_bridge_report_sha256: String,
    pub family_completeness_statement: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineReport {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub baseline_id: String,
    pub paired_check_report_sha256: String,
    pub law_statuses: BTreeMap<LawId, Status>,
    pub profile_property_consistency: Status,
    pub match_subset_safe: Status,
    pub safe_anchor_coverage: Status,
    pub match_anchor_coverage: Status,
    pub match_shape_compatibility: Status,
    pub comparator_definedness: Status,
    pub match_coverage_status: Status,
    pub match_coverage: MatchCoverageReport,
    pub policy_contract_status: PolicyContractStatus,
    pub policy_witnesses: Vec<String>,
    pub property_statuses: BTreeMap<String, Status>,
    pub property_witnesses: BTreeMap<String, String>,
    pub target_non_amplification: TargetNonAmplificationReport,
    pub derivation_report_sha256: String,
    pub derivation_parity_status: Status,
    pub validity_parity_status: Status,
    pub coverage_parity_status: Status,
    pub policy_parity_status: Status,
    pub property_parity_status: Status,
    pub witness_parity_status: Status,
}
