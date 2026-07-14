use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CONTRACT_VERSION;
use crate::adapter_ir::{AdapterContext, AdapterTypeError};
use crate::bridge::{evaluate_bridge, not_requested_bridge};
use crate::canonical::{CanonicalError, canonical_sha256, sha256_bytes};
use crate::carrier::derive_carrier_summary;
use crate::comparator::{InducedRelation, comparator_definedness, induced_relation};
use crate::domain::{
    ComparatorSpec, DomainError, PairDomainSpec, ResolvedScope, ValidationScope, ValuePair,
};
use crate::observer_ir::ObserverError;
use crate::relation_ir::{
    ConstructedPolicy, Endpoint, EndpointPolicy, IrrelevanceAppliesTo, MatchCoverageMode,
    PolicyError, Relation,
};
use crate::report::*;
use crate::roundtrip::{RoundTripEvaluation, evaluate_roundtrips, materialize_roundtrip_report};
use crate::type_ir::{EnumerationLimits, Value};
use crate::witness::{ComparatorEvidence, Witness, WitnessKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfiguration {
    pub schema: String,
    pub max_values_per_type: usize,
    pub max_product_arity: usize,
    pub max_sum_variants: usize,
    pub max_bit_width: u8,
    pub max_recursion_depth: usize,
    pub max_universe_pairs: usize,
    pub max_transformations: usize,
}

impl Default for RunConfiguration {
    fn default() -> Self {
        Self {
            schema: "gluerift.run-configuration/v0.3.1a".into(),
            max_values_per_type: 4_096,
            max_product_arity: 32,
            max_sum_variants: 32,
            max_bit_width: 8,
            max_recursion_depth: 12,
            max_universe_pairs: 65_536,
            max_transformations: 4_096,
        }
    }
}

impl RunConfiguration {
    pub fn enumeration_limits(&self) -> EnumerationLimits {
        EnumerationLimits {
            max_values_per_type: self.max_values_per_type,
            max_product_arity: self.max_product_arity,
            max_sum_variants: self.max_sum_variants,
            max_bit_width: self.max_bit_width,
            max_recursion_depth: self.max_recursion_depth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRequest {
    pub schema: String,
    pub request_id: String,
    pub profile: CertificationProfile,
    pub validation_scope_sha256: String,
    pub endpoint_policy_sha256: String,
    pub run_configuration_sha256: String,
    pub required_laws: RequiredLaws,
    pub required_properties: Vec<PropertyRequest>,
    pub required_bridges: Vec<BridgeKind>,
    pub required_transformation_family_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CertificationProfile {
    Diagnostic,
    PolicySound,
    PolicySoundAdequate,
    FaithfulExact,
}

impl CertificationProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Diagnostic => "diagnostic",
            Self::PolicySound => "policy-sound",
            Self::PolicySoundAdequate => "policy-sound-adequate",
            Self::FaithfulExact => "faithful-exact",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredLaws {
    pub source_native_roundtrip: bool,
    pub target_native_roundtrip: bool,
    pub source_carrier_roundtrip: bool,
    pub target_carrier_roundtrip: bool,
    pub source_full_transport: bool,
    pub target_full_transport: bool,
}

impl RequiredLaws {
    pub fn all() -> Self {
        Self {
            source_native_roundtrip: true,
            target_native_roundtrip: true,
            source_carrier_roundtrip: true,
            target_carrier_roundtrip: true,
            source_full_transport: true,
            target_full_transport: true,
        }
    }

    pub fn ids(&self) -> Vec<LawId> {
        [
            (self.source_native_roundtrip, LawId::SourceNative),
            (self.target_native_roundtrip, LawId::TargetNative),
            (self.source_carrier_roundtrip, LawId::SourceCarrier),
            (self.target_carrier_roundtrip, LawId::TargetCarrier),
            (self.source_full_transport, LawId::SourceFullTransport),
            (self.target_full_transport, LawId::TargetFullTransport),
        ]
        .into_iter()
        .filter_map(|(required, id)| required.then_some(id))
        .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PropertyRequest {
    PolicySoundness,
    ComparisonAdequacy,
    ComparisonPrecision,
    FaithfulComparison,
    TargetNonAmplification { dimension_ids: Vec<String> },
}

impl PropertyRequest {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::PolicySoundness => "policy-soundness",
            Self::ComparisonAdequacy => "comparison-adequacy",
            Self::ComparisonPrecision => "comparison-precision",
            Self::FaithfulComparison => "faithful-comparison",
            Self::TargetNonAmplification { .. } => "target-non-amplification",
        }
    }
}

#[derive(Clone, Debug)]
pub struct EvidenceMetadata {
    pub tool_build_sha256: String,
}

impl EvidenceMetadata {
    pub fn deterministic_default() -> Self {
        Self {
            tool_build_sha256: sha256_bytes(format!("gluerift-{CONTRACT_VERSION}").as_bytes()),
        }
    }

    pub fn for_current_executable() -> std::io::Result<Self> {
        let bytes = fs::read(std::env::current_exe()?)?;
        Ok(Self {
            tool_build_sha256: sha256_bytes(&bytes),
        })
    }
}

#[derive(Clone, Debug)]
pub struct CheckedRun {
    pub check_report: CheckReport,
    pub roundtrip_report: RoundTripReport,
    pub execution_trace_tables: BTreeMap<LawId, ExecutionTraceTableReport>,
    pub bridge_reports: BTreeMap<BridgeKind, BridgeReport>,
    pub carrier_summary: CarrierSummary,
    pub witnesses: BTreeMap<String, Witness>,
    pub induced_relation: BTreeSet<ValuePair>,
    pub safe: BTreeSet<ValuePair>,
    pub matched: BTreeSet<ValuePair>,
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error(transparent)]
    Adapter(#[from] AdapterTypeError),
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Policy(#[from] PolicyError),
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("request schema must be gluerift.validation-request/v0.3.1a")]
    WrongRequestSchema,
    #[error("run configuration schema must be gluerift.run-configuration/v0.3.1a")]
    WrongRunConfigurationSchema,
    #[error("run configuration values differ from the frozen Core limits")]
    NonCanonicalRunConfiguration,
    #[error("request hash binding mismatch for {field}: expected {expected}, actual {actual}")]
    HashMismatch {
        field: &'static str,
        expected: String,
        actual: String,
    },
    #[error("comparison universe has {actual} pairs, limit is {limit}")]
    UniverseLimit { actual: usize, limit: usize },
    #[error("required bridge list must be duplicate-free and sorted")]
    NonCanonicalBridges,
    #[error("internal semantic invariant failed: {0}")]
    InternalInvariant(String),
}

pub fn check(
    context: &AdapterContext,
    scope: &ValidationScope,
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    run_configuration: &RunConfiguration,
    metadata: &EvidenceMetadata,
) -> Result<CheckedRun, CheckError> {
    if request.schema != "gluerift.validation-request/v0.3.1a" {
        return Err(CheckError::WrongRequestSchema);
    }
    if run_configuration.schema != "gluerift.run-configuration/v0.3.1a" {
        return Err(CheckError::WrongRunConfigurationSchema);
    }
    if run_configuration != &RunConfiguration::default() {
        return Err(CheckError::NonCanonicalRunConfiguration);
    }
    if !request
        .required_bridges
        .windows(2)
        .all(|pair| pair[0] < pair[1])
    {
        return Err(CheckError::NonCanonicalBridges);
    }
    if !request
        .required_properties
        .windows(2)
        .all(|pair| property_rank(pair[0].kind()) < property_rank(pair[1].kind()))
    {
        return Err(CheckError::Policy(PolicyError::InvalidRelation {
            dimension: "request".into(),
            reason:
                "required_properties must be duplicate-free and in canonical property-kind order"
                    .into(),
        }));
    }
    for property in &request.required_properties {
        if let PropertyRequest::TargetNonAmplification { dimension_ids } = property
            && (dimension_ids.is_empty() || !dimension_ids.windows(2).all(|pair| pair[0] < pair[1]))
        {
            return Err(CheckError::Policy(PolicyError::InvalidRelation {
                dimension: "request".into(),
                reason: "TNA dimension IDs must be nonempty, duplicate-free, and sorted".into(),
            }));
        }
    }
    let limits = run_configuration.enumeration_limits();
    context.validate(&limits)?;
    // Refuse an oversized product universe before materializing its Cartesian
    // product.  Resource exhaustion is a tool error, never a sampled or
    // malformed semantic verdict (§15.3).
    let declared_universe_size = match &scope.comparison_universe {
        PairDomainSpec::Product { source, target } => {
            let source_count = source
                .resolve(&context.source_type, &limits, "comparison_universe.source")?
                .len();
            let target_count = target
                .resolve(&context.target_type, &limits, "comparison_universe.target")?
                .len();
            source_count.saturating_mul(target_count)
        }
        PairDomainSpec::FinitePairSet { pairs } => pairs.len(),
    };
    if declared_universe_size > run_configuration.max_universe_pairs {
        return Err(CheckError::UniverseLimit {
            actual: declared_universe_size,
            limit: run_configuration.max_universe_pairs,
        });
    }
    let resolved = scope.resolve(context, &limits)?;
    if resolved.comparison_universe.len() > run_configuration.max_universe_pairs {
        return Err(CheckError::UniverseLimit {
            actual: resolved.comparison_universe.len(),
            limit: run_configuration.max_universe_pairs,
        });
    }

    let context = context.normalized();
    let candidate_sha256 = canonical_sha256(&context)?;
    let types_sha256 = canonical_sha256(&(
        &context.source_type,
        &context.target_type,
        &context.carrier_type,
    ))?;
    let validation_scope_sha256 = canonical_sha256(scope)?;
    let endpoint_policy_sha256 = canonical_sha256(policy)?;
    let validation_request_sha256 = canonical_sha256(request)?;
    let run_configuration_sha256 = canonical_sha256(run_configuration)?;
    let comparator_spec_sha256 = canonical_sha256(&scope.comparator)?;
    check_hash(
        "validation_scope_sha256",
        &request.validation_scope_sha256,
        &validation_scope_sha256,
    )?;
    check_hash(
        "endpoint_policy_sha256",
        &request.endpoint_policy_sha256,
        &endpoint_policy_sha256,
    )?;
    check_hash(
        "run_configuration_sha256",
        &request.run_configuration_sha256,
        &run_configuration_sha256,
    )?;

    let base_envelope = |schema: &str, evidence_suffix: &str| CommonEnvelope {
        schema: schema.into(),
        semantic_contract_version: CONTRACT_VERSION.into(),
        tool_build_sha256: metadata.tool_build_sha256.clone(),
        run_configuration_sha256: run_configuration_sha256.clone(),
        evidence_id: format!(
            "{}:{}:{}",
            request.request_id,
            &candidate_sha256[..16],
            evidence_suffix
        ),
        candidate_sha256: candidate_sha256.clone(),
        types_sha256: types_sha256.clone(),
        validation_scope_sha256: validation_scope_sha256.clone(),
        endpoint_policy_sha256: endpoint_policy_sha256.clone(),
        validation_request_sha256: validation_request_sha256.clone(),
        comparator_spec_sha256: comparator_spec_sha256.clone(),
        dependency_evidence_ids: Vec::new(),
        status: Status::ProvedExhaustive,
    };
    let witness_envelope = base_envelope("gluerift.witness/v0.3.1a", "witness");

    let mut witnesses = BTreeMap::new();
    let mut roundtrips = evaluate_roundtrips(&context, &resolved)?;
    install_roundtrip_witnesses(
        &mut roundtrips,
        &resolved,
        &comparator_spec_sha256,
        &request.request_id,
        &witness_envelope,
        &mut witnesses,
    )?;
    let mut execution_trace_tables = BTreeMap::new();
    for (law_id, rows) in &roundtrips.tables {
        let mut envelope = base_envelope(
            "gluerift.execution-trace-table/v0.3.1a",
            &format!("execution-trace-{}", law_id_name(*law_id)),
        );
        envelope.status = roundtrips.laws[law_id].status;
        let table = ExecutionTraceTableReport {
            envelope,
            law_id: *law_id,
            rows: rows.clone(),
        };
        let hash = canonical_sha256(&table)?;
        roundtrips
            .laws
            .get_mut(law_id)
            .ok_or_else(|| CheckError::InternalInvariant(format!("missing {law_id:?} report")))?
            .execution_trace_table_sha256 = hash;
        execution_trace_tables.insert(*law_id, table);
    }
    let mut roundtrip_envelope = base_envelope("gluerift.roundtrip-report/v0.3.1a", "roundtrips");
    roundtrip_envelope.status = aggregate_status(roundtrips.laws.values().map(|law| law.status));
    roundtrip_envelope.dependency_evidence_ids = execution_trace_tables
        .values()
        .map(|table| table.envelope.evidence_id.clone())
        .collect();
    for report in roundtrips.laws.values() {
        if report.witness_sha256 != NOT_APPLICABLE {
            roundtrip_envelope.dependency_evidence_ids.push(
                witness_evidence_id(&witnesses, &report.witness_sha256)
                    .ok_or_else(|| {
                        CheckError::InternalInvariant(format!(
                            "unresolved round-trip witness {}",
                            report.witness_sha256
                        ))
                    })?
                    .into(),
            );
        }
    }
    roundtrip_envelope.dependency_evidence_ids.sort();
    roundtrip_envelope.dependency_evidence_ids.dedup();
    let roundtrip_report = materialize_roundtrip_report(roundtrip_envelope, &roundtrips);
    let roundtrip_report_sha256 = canonical_sha256(&roundtrip_report)?;

    let (definedness_status, definedness_checked_count, definedness_failure) =
        comparator_definedness(&context, &resolved);
    let mut definedness_witness_sha256 = NOT_APPLICABLE.to_string();
    if let Some((input, transported, error)) = definedness_failure {
        let (source, target) = match resolved.comparator {
            ComparatorSpec::TargetNativeExact | ComparatorSpec::CarrierExact => (Some(input), None),
            ComparatorSpec::SourceNativeExact => (None, Some(input)),
        };
        let witness = Witness {
            envelope: witness_envelope.clone(),
            witness_kind: WitnessKind::ComparatorUndefined,
            source_value: EvidenceValue::from_option(source),
            target_value: EvidenceValue::from_option(target),
            comparator_kind: EvidenceValue::Present(resolved.comparator),
            comparator_spec_sha256: comparator_spec_sha256.clone(),
            comparator_evidence: ComparatorEvidence::NotApplicable,
            violated_or_missing_dimensions: Vec::new(),
            adapter_path: normalized_path(&error.adapter_path),
            replay_command: vec![
                "gluerift".into(),
                "check".into(),
                "--request".into(),
                request.request_id.clone(),
            ],
            coverage_mode: "not-applicable".into(),
            source_comparison_domain_sha256: resolved.source_comparison_domain_sha256.clone(),
            target_comparison_domain_sha256: resolved.target_comparison_domain_sha256.clone(),
            match_pair_count: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            safe_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            match_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            roundtrip_trace: transported
                .map(|value| {
                    vec![StageTrace {
                        stage: "transport-intermediate".into(),
                        result: Ok(value),
                    }]
                })
                .unwrap_or_default(),
        };
        let hash = install_witness(witness, &mut witnesses)?;
        definedness_witness_sha256 = hash;
    }
    let definedness = ComparatorDefinednessReport {
        status: definedness_status,
        checked_input_count: definedness_checked_count,
        witness_sha256: definedness_witness_sha256.clone(),
    };
    let induced = induced_relation(&context, &resolved);

    let source_observer_domain = context
        .source_type
        .enumerate(&limits)
        .map_err(AdapterTypeError::from)?;
    let target_observer_domain = context
        .target_type
        .enumerate(&limits)
        .map_err(AdapterTypeError::from)?;
    let policy_construction = match policy.construct(
        &resolved,
        &context.source_type,
        &context.target_type,
        &source_observer_domain,
        &target_observer_domain,
    ) {
        Ok(constructed) => PolicyConstructionState::Supported(constructed),
        Err(PolicyError::Observer {
            source: ObserverError::UnsupportedObserver { .. },
            ..
        }) => PolicyConstructionState::Unsupported,
        Err(error) => return Err(CheckError::Policy(error)),
    };
    let empty_policy = ConstructedPolicy {
        safe: BTreeSet::new(),
        matched: BTreeSet::new(),
        observations: BTreeMap::new(),
    };
    let constructed = match &policy_construction {
        PolicyConstructionState::Supported(value) => value,
        PolicyConstructionState::Unsupported => &empty_policy,
    };

    let mut coverage = match_coverage(
        policy,
        &resolved,
        &constructed.matched,
        &comparator_spec_sha256,
        &request.request_id,
        &witness_envelope,
        &mut witnesses,
    )?;
    let (safe_anchor_coverage, match_anchor_coverage) = anchor_coverage(policy, &context, request);
    let shape_status =
        match_shape_compatibility(&resolved, &constructed.matched, request, &roundtrips);
    let profile = profile_consistency(policy, request, constructed, coverage.status);
    let unsupported = matches!(policy_construction, PolicyConstructionState::Unsupported);
    if unsupported {
        coverage.status = if policy.match_coverage == MatchCoverageMode::None {
            Status::NotRequested
        } else {
            Status::Unknown
        };
    }

    let policy_contract_status = if policy.safe_dimensions.is_empty() {
        PolicyContractStatus::PolicyUnconstrained
    } else if constructed.safe.len() == resolved.comparison_universe.len() {
        PolicyContractStatus::UniversalDeclared
    } else {
        PolicyContractStatus::Constrained
    };
    let safe_is_universal = policy.safe_dimensions.is_empty()
        || constructed.safe.len() == resolved.comparison_universe.len();

    // Match coverage is a request-level validity obligation (§7.6–§7.7), not
    // merely a prerequisite of Match-dependent properties.  If the policy
    // owner requested nonempty/total Match coverage and it fails, no candidate
    // property is issued under the weakened request (V02).
    let requested_match_coverage_valid = match policy.match_coverage {
        MatchCoverageMode::None => coverage.status == Status::NotRequested,
        _ => coverage.status == Status::ProvedExhaustive,
    };
    let safe_property_prerequisites_valid = safe_anchor_coverage.status == Status::ProvedExhaustive
        && requested_match_coverage_valid
        && profile.status == Status::ProvedExhaustive
        && !unsupported;
    let match_property_prerequisites_valid = safe_property_prerequisites_valid
        && coverage.status == Status::ProvedExhaustive
        && match_anchor_coverage.status == Status::ProvedExhaustive
        && shape_status == Status::ProvedExhaustive
        && profile.status == Status::ProvedExhaustive;
    // Comparator totality is an independent certification prerequisite.  A
    // failure does not manufacture a property counterexample or mask the
    // directly enumerated partial relation (§§3.4, 15.4).
    let properties = evaluate_properties(
        &context,
        &resolved,
        policy,
        request,
        constructed,
        &induced,
        safe_property_prerequisites_valid,
        match_property_prerequisites_valid,
        unsupported,
        &comparator_spec_sha256,
        &witness_envelope,
        &mut witnesses,
    )?;

    let mut bridge_reports = BTreeMap::new();
    for kind in [BridgeKind::CarrierTarget, BridgeKind::CarrierSource] {
        let mandatory = matches!(
            (resolved.comparator, kind),
            (ComparatorSpec::TargetNativeExact, BridgeKind::CarrierTarget)
                | (ComparatorSpec::SourceNativeExact, BridgeKind::CarrierSource)
        );
        let requested = request.required_bridges.contains(&kind) || mandatory;
        let envelope = base_envelope(
            "gluerift.bridge-report/v0.3.1a",
            match kind {
                BridgeKind::CarrierTarget => "bridge-carrier-target",
                BridgeKind::CarrierSource => "bridge-carrier-source",
            },
        );
        let mut report = if requested {
            evaluate_bridge(envelope, &context, &resolved, kind)
        } else {
            not_requested_bridge(envelope, kind)
        };
        if report.envelope.status == Status::Disproved {
            let pair = match &report.counterexample_pair {
                EvidenceValue::Present(pair) => pair.clone(),
                EvidenceValue::Absent(_) => {
                    return Err(CheckError::InternalInvariant(
                        "disproved bridge has no counterexample pair".into(),
                    ));
                }
            };
            let witness = Witness {
                envelope: witness_envelope.clone(),
                witness_kind: WitnessKind::BridgeDivergence,
                source_value: EvidenceValue::Present(pair.source.clone()),
                target_value: EvidenceValue::Present(pair.target.clone()),
                comparator_kind: EvidenceValue::Present(resolved.comparator),
                comparator_spec_sha256: comparator_spec_sha256.clone(),
                comparator_evidence: report.native_comparator_evidence.clone(),
                violated_or_missing_dimensions: Vec::new(),
                adapter_path: Vec::new(),
                replay_command: vec![
                    "gluerift".into(),
                    "check".into(),
                    "--request".into(),
                    request.request_id.clone(),
                ],
                coverage_mode: "not-applicable".into(),
                source_comparison_domain_sha256: resolved.source_comparison_domain_sha256.clone(),
                target_comparison_domain_sha256: resolved.target_comparison_domain_sha256.clone(),
                match_pair_count: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
                safe_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
                match_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
                roundtrip_trace: Vec::new(),
            };
            let hash = install_witness(witness, &mut witnesses)?;
            report.envelope.dependency_evidence_ids.push(
                witness_evidence_id(&witnesses, &hash)
                    .ok_or_else(|| {
                        CheckError::InternalInvariant(format!("unresolved bridge witness {hash}"))
                    })?
                    .into(),
            );
        }
        bridge_reports.insert(kind, report);
    }
    let carrier_target = bridge_reports
        .get(&BridgeKind::CarrierTarget)
        .ok_or_else(|| {
            CheckError::InternalInvariant("carrier-target bridge report missing".into())
        })?;
    let carrier_source = bridge_reports
        .get(&BridgeKind::CarrierSource)
        .ok_or_else(|| {
            CheckError::InternalInvariant("carrier-source bridge report missing".into())
        })?;
    let carrier_target_hash = canonical_sha256(carrier_target)?;
    let carrier_source_hash = canonical_sha256(carrier_source)?;
    let selected_bridge = match resolved.comparator {
        ComparatorSpec::TargetNativeExact => (
            carrier_target.envelope.status,
            carrier_target_hash.clone(),
            Some(carrier_target.envelope.evidence_id.clone()),
        ),
        ComparatorSpec::SourceNativeExact => (
            carrier_source.envelope.status,
            carrier_source_hash.clone(),
            Some(carrier_source.envelope.evidence_id.clone()),
        ),
        ComparatorSpec::CarrierExact => (Status::NotRequested, NOT_APPLICABLE.into(), None),
    };

    let mut carrier_envelope = base_envelope("gluerift.carrier-summary/v0.3.1a", "carrier-summary");
    if let Some(evidence_id) = &selected_bridge.2 {
        carrier_envelope
            .dependency_evidence_ids
            .push(evidence_id.clone());
    }
    let mut carrier_summary = derive_carrier_summary(
        carrier_envelope,
        &context,
        &resolved,
        selected_bridge.0,
        selected_bridge.1.clone(),
    );
    carrier_summary.class_observation_conflicts =
        carrier_observation_conflicts(&carrier_summary, policy, constructed)?;
    let carrier_summary_sha256 = canonical_sha256(&carrier_summary)?;

    let minimum = profile.minimum.clone();
    let explicit = request
        .required_properties
        .iter()
        .map(|property| property.kind().to_string())
        .collect::<Vec<_>>();
    let extras = explicit
        .iter()
        .filter(|kind| !minimum.contains(kind))
        .cloned()
        .collect::<Vec<_>>();
    let safe_match = if request.profile == CertificationProfile::FaithfulExact {
        if constructed.safe == constructed.matched {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        }
    } else {
        Status::NotRequested
    };
    let safe_match_witness = if safe_match == Status::Disproved {
        let pair = constructed
            .safe
            .symmetric_difference(&constructed.matched)
            .next()
            .cloned()
            .ok_or_else(|| {
                CheckError::InternalInvariant(
                    "Safe/Match disproof lacks symmetric-difference pair".into(),
                )
            })?;
        let witness = policy_pair_witness(
            WitnessKind::SafeMatchDivergence,
            &pair,
            resolved.comparator,
            &comparator_spec_sha256,
            &resolved,
            policy,
            constructed,
            None,
            &witness_envelope,
        );
        install_witness(witness, &mut witnesses)?
    } else {
        NOT_APPLICABLE.into()
    };

    let required_law_ids = request.required_laws.ids();
    let all_required_laws_proved = required_law_ids
        .iter()
        .all(|id| roundtrips.laws[id].status == Status::ProvedExhaustive);
    let policy_constrained = policy_contract_status != PolicyContractStatus::PolicyUnconstrained;
    let requested_statuses = requested_property_statuses(request, &properties);
    // Eligibility requires a complete, supported evaluation for every
    // explicitly requested property.  A semantic disproof is still an
    // eligible (but non-granting) result; invalid/unknown/tool-error is not.
    let all_requested_properties_evaluated = requested_statuses
        .iter()
        .all(|status| matches!(status, Status::ProvedExhaustive | Status::Disproved));
    let eligible = request.profile != CertificationProfile::Diagnostic
        && profile.status == Status::ProvedExhaustive
        && !unsupported
        && policy_constrained
        && definedness.status == Status::ProvedExhaustive
        && all_required_laws_proved
        && safe_anchor_coverage.status == Status::ProvedExhaustive
        && match_anchor_coverage.status != Status::Disproved
        && shape_status == Status::ProvedExhaustive
        && all_requested_properties_evaluated;
    let granted = eligible
        && requested_statuses
            .iter()
            .all(|status| *status == Status::ProvedExhaustive);
    let mut blocking = Vec::new();
    if request.profile == CertificationProfile::Diagnostic {
        blocking.push("diagnostic-profile".into());
    }
    if profile.status != Status::ProvedExhaustive {
        blocking.push("profile-property-inconsistent".into());
    }
    if unsupported {
        blocking.push("unsupported-observer".into());
    }
    if !policy_constrained {
        blocking.push("policy-unconstrained".into());
    }
    if definedness.status != Status::ProvedExhaustive {
        blocking.push("comparator-undefined".into());
    }
    if !all_required_laws_proved {
        blocking.push("required-law-disproved".into());
    }
    if safe_anchor_coverage.status != Status::ProvedExhaustive
        || match_anchor_coverage.status == Status::Disproved
    {
        blocking.push("anchor-coverage-failed".into());
    }
    if shape_status != Status::ProvedExhaustive {
        blocking.push("match-shape-incompatible".into());
    }
    for (property, status) in request.required_properties.iter().zip(requested_statuses) {
        if status != Status::ProvedExhaustive {
            blocking.push(format!(
                "required-property-{}-{}",
                property.kind(),
                status_name(status)
            ));
        }
    }
    blocking.sort();
    blocking.dedup();

    let certification = CertificationReport {
        requested_profile: request.profile.as_str().into(),
        profile_property_consistency_status: profile.status,
        minimum_required_property_kinds: minimum,
        explicit_required_property_kinds: explicit,
        extra_required_property_kinds: extras,
        explicit_required_law_ids: required_law_ids,
        safe_match_equality_status: safe_match,
        safe_match_equality_witness_sha256: safe_match_witness,
        eligible,
        granted,
        blocking_reasons: blocking,
    };
    let comparison = ComparisonReport {
        comparator_kind: resolved.comparator,
        comparator_spec_sha256: comparator_spec_sha256.clone(),
        universe_pair_count: resolved.comparison_universe.len(),
        induced_equality_pair_count: induced.pairs.len(),
        comparator_definedness: definedness,
    };
    let bridges = BridgesReport {
        carrier_target_bridge: BridgeReference {
            status: carrier_target.envelope.status,
            report_sha256: carrier_target_hash,
        },
        carrier_source_bridge: BridgeReference {
            status: carrier_source.envelope.status,
            report_sha256: carrier_source_hash,
        },
        selected_carrier_bridge_status: selected_bridge.0,
    };
    let policy_report = PolicyReport {
        safe_dimension_count: policy.safe_dimensions.len(),
        safe_pair_count: if policy.safe_dimensions.is_empty() {
            resolved.comparison_universe.len()
        } else {
            constructed.safe.len()
        },
        unsafe_pair_count: if policy.safe_dimensions.is_empty() {
            0
        } else {
            resolved.comparison_universe.len() - constructed.safe.len()
        },
        safe_is_universal,
        policy_contract_status,
        policy_vacuity_warning: safe_is_universal,
        match_dimension_count: policy.match_dimensions.len(),
        match_pair_count: constructed.matched.len(),
        match_subset_safe_status: if unsupported {
            Status::Unknown
        } else {
            Status::ProvedExhaustive
        },
        match_coverage: coverage,
        match_shape_compatibility: shape_status,
        safe_anchor_coverage,
        match_anchor_coverage,
    };
    let required_law_statuses = request
        .required_laws
        .ids()
        .into_iter()
        .map(|id| roundtrips.laws[&id].status);
    let required_bridge_statuses = request
        .required_bridges
        .iter()
        .map(|kind| bridge_reports[kind].envelope.status);
    let overall = aggregate_status(
        [profile.status, definedness_status]
            .into_iter()
            .chain(requested_property_statuses(request, &properties))
            .chain(required_law_statuses)
            .chain(required_bridge_statuses),
    );
    let mut check_envelope = base_envelope("gluerift.check-report/v0.3.1a", "check");
    check_envelope.status = overall;
    check_envelope.dependency_evidence_ids = vec![
        roundtrip_report.envelope.evidence_id.clone(),
        carrier_summary.envelope.evidence_id.clone(),
    ];
    check_envelope.dependency_evidence_ids.extend(
        bridge_reports
            .values()
            .map(|report| report.envelope.evidence_id.clone()),
    );
    check_envelope.dependency_evidence_ids.extend(
        witnesses
            .values()
            .map(|witness| witness.envelope.evidence_id.clone()),
    );
    check_envelope.dependency_evidence_ids.sort();
    check_envelope.dependency_evidence_ids.dedup();
    let mut witness_sha256s: Vec<_> = witnesses.keys().cloned().collect();
    witness_sha256s.sort();
    let check_report = CheckReport {
        envelope: check_envelope,
        comparison,
        bridges,
        policy: policy_report,
        properties,
        certification,
        roundtrip_report_sha256,
        carrier_summary_sha256,
        witness_sha256s,
    };
    Ok(CheckedRun {
        check_report,
        roundtrip_report,
        execution_trace_tables,
        bridge_reports,
        carrier_summary,
        witnesses,
        induced_relation: induced.pairs,
        safe: if policy.safe_dimensions.is_empty() {
            resolved.comparison_universe.iter().cloned().collect()
        } else {
            constructed.safe.clone()
        },
        matched: constructed.matched.clone(),
    })
}

enum PolicyConstructionState {
    Supported(ConstructedPolicy),
    Unsupported,
}

struct ProfileCheck {
    status: Status,
    minimum: Vec<String>,
}

fn profile_consistency(
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    constructed: &ConstructedPolicy,
    coverage_status: Status,
) -> ProfileCheck {
    let minimum: Vec<String> = match request.profile {
        CertificationProfile::Diagnostic => vec![],
        CertificationProfile::PolicySound => vec!["policy-soundness".into()],
        CertificationProfile::PolicySoundAdequate => {
            vec!["policy-soundness".into(), "comparison-adequacy".into()]
        }
        CertificationProfile::FaithfulExact => vec!["faithful-comparison".into()],
    };
    let kinds: Vec<_> = request
        .required_properties
        .iter()
        .map(PropertyRequest::kind)
        .collect();
    let unique: BTreeSet<_> = kinds.iter().copied().collect();
    let duplicates = unique.len() != kinds.len();
    let minimum_present = minimum.iter().all(|kind| unique.contains(kind.as_str()));
    let match_dependent = kinds.iter().any(|kind| {
        matches!(
            *kind,
            "comparison-adequacy" | "comparison-precision" | "faithful-comparison"
        )
    });
    let match_valid = !match_dependent
        || (!policy.match_dimensions.is_empty()
            && policy.match_coverage != MatchCoverageMode::None
            && coverage_status == Status::ProvedExhaustive);
    let faithful_valid = request.profile != CertificationProfile::FaithfulExact
        || (!policy.match_dimensions.is_empty()
            && constructed.safe == constructed.matched
            && coverage_status == Status::ProvedExhaustive);
    ProfileCheck {
        status: if !duplicates && minimum_present && match_valid && faithful_valid {
            Status::ProvedExhaustive
        } else {
            Status::Invalid
        },
        minimum,
    }
}

fn match_coverage(
    policy: &EndpointPolicy,
    scope: &ResolvedScope,
    matched: &BTreeSet<ValuePair>,
    comparator_hash: &str,
    request_id: &str,
    witness_envelope: &CommonEnvelope,
    witnesses: &mut BTreeMap<String, Witness>,
) -> Result<MatchCoverageReport, CanonicalError> {
    let matched_sources: BTreeSet<_> = matched.iter().map(|pair| pair.source.clone()).collect();
    let matched_targets: BTreeSet<_> = matched.iter().map(|pair| pair.target.clone()).collect();
    let source_gap = scope
        .source_comparison_domain
        .iter()
        .find(|value| !matched_sources.contains(*value))
        .cloned();
    let target_gap = scope
        .target_comparison_domain
        .iter()
        .find(|value| !matched_targets.contains(*value))
        .cloned();
    let status = match policy.match_coverage {
        MatchCoverageMode::None => Status::NotRequested,
        MatchCoverageMode::Nonempty => {
            if matched.is_empty() {
                Status::Disproved
            } else {
                Status::ProvedExhaustive
            }
        }
        MatchCoverageMode::SourceTotal => {
            if source_gap.is_some() {
                Status::Disproved
            } else {
                Status::ProvedExhaustive
            }
        }
        MatchCoverageMode::TargetTotal => {
            if target_gap.is_some() {
                Status::Disproved
            } else {
                Status::ProvedExhaustive
            }
        }
        MatchCoverageMode::BidirectionalTotal => {
            if source_gap.is_some() || target_gap.is_some() {
                Status::Disproved
            } else {
                Status::ProvedExhaustive
            }
        }
    };
    let base = |kind: WitnessKind, source: Option<Value>, target: Option<Value>| Witness {
        envelope: witness_envelope.clone(),
        witness_kind: kind,
        source_value: EvidenceValue::from_option(source),
        target_value: EvidenceValue::from_option(target),
        comparator_kind: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
        comparator_spec_sha256: comparator_hash.into(),
        comparator_evidence: ComparatorEvidence::NotApplicable,
        violated_or_missing_dimensions: policy.match_dimensions.clone(),
        adapter_path: Vec::new(),
        replay_command: vec![
            "gluerift".into(),
            "check".into(),
            "--request".into(),
            request_id.into(),
        ],
        coverage_mode: match_coverage_mode_name(policy.match_coverage).into(),
        source_comparison_domain_sha256: scope.source_comparison_domain_sha256.clone(),
        target_comparison_domain_sha256: scope.target_comparison_domain_sha256.clone(),
        match_pair_count: EvidenceValue::Present(matched.len()),
        safe_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
        match_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
        roundtrip_trace: Vec::new(),
    };
    let empty_hash = if policy.match_coverage == MatchCoverageMode::Nonempty && matched.is_empty() {
        install_witness(base(WitnessKind::MatchCoverageEmpty, None, None), witnesses)?
    } else {
        NOT_APPLICABLE.into()
    };
    let source_hash = if matches!(
        policy.match_coverage,
        MatchCoverageMode::SourceTotal | MatchCoverageMode::BidirectionalTotal
    ) {
        source_gap
            .clone()
            .map(|value| {
                install_witness(
                    base(WitnessKind::MatchCoverageSourceGap, Some(value), None),
                    witnesses,
                )
            })
            .transpose()?
            .unwrap_or_else(|| NOT_APPLICABLE.into())
    } else {
        NOT_APPLICABLE.into()
    };
    let target_hash = if matches!(
        policy.match_coverage,
        MatchCoverageMode::TargetTotal | MatchCoverageMode::BidirectionalTotal
    ) {
        target_gap
            .clone()
            .map(|value| {
                install_witness(
                    base(WitnessKind::MatchCoverageTargetGap, None, Some(value)),
                    witnesses,
                )
            })
            .transpose()?
            .unwrap_or_else(|| NOT_APPLICABLE.into())
    } else {
        NOT_APPLICABLE.into()
    };
    Ok(MatchCoverageReport {
        mode: policy.match_coverage,
        status,
        source_comparison_domain_sha256: scope.source_comparison_domain_sha256.clone(),
        target_comparison_domain_sha256: scope.target_comparison_domain_sha256.clone(),
        source_comparison_domain_count: scope.source_comparison_domain.len(),
        target_comparison_domain_count: scope.target_comparison_domain.len(),
        matched_source_count: matched_sources.len(),
        matched_target_count: matched_targets.len(),
        empty_match_witness_sha256: empty_hash,
        unmatched_source_witness_sha256: source_hash,
        unmatched_target_witness_sha256: target_hash,
    })
}

fn anchor_coverage(
    policy: &EndpointPolicy,
    context: &AdapterContext,
    request: &ValidationRequest,
) -> (CoverageReport, CoverageReport) {
    let safe = coverage_for(policy, context, &policy.safe_dimensions, true);
    let match_requested = request.required_properties.iter().any(|property| {
        matches!(
            property,
            PropertyRequest::ComparisonAdequacy
                | PropertyRequest::ComparisonPrecision
                | PropertyRequest::FaithfulComparison
        )
    });
    let matching = if match_requested {
        coverage_for(policy, context, &policy.match_dimensions, false)
    } else {
        CoverageReport {
            status: Status::NotRequested,
            relevant_path_count: 0,
            observed_path_count: 0,
            explicitly_irrelevant_path_count: 0,
            uncovered_paths: Vec::new(),
        }
    };
    (safe, matching)
}

fn coverage_for(
    policy: &EndpointPolicy,
    context: &AdapterContext,
    ids: &[String],
    safety: bool,
) -> CoverageReport {
    if ids.is_empty() {
        return CoverageReport {
            status: Status::ProvedExhaustive,
            relevant_path_count: 0,
            observed_path_count: 0,
            explicitly_irrelevant_path_count: 0,
            uncovered_paths: Vec::new(),
        };
    }
    let mut source_exact = BTreeSet::new();
    let mut source_whole = BTreeSet::new();
    let mut target_exact = BTreeSet::new();
    let mut target_whole = BTreeSet::new();
    for id in ids {
        if let Some(dimension) = policy.dimension(id) {
            let source = dimension.source_observer.anchor_reads();
            source_exact.extend(source.exact);
            source_whole.extend(source.whole_value);
            let target = dimension.target_observer.anchor_reads();
            target_exact.extend(target.exact);
            target_whole.extend(target.whole_value);
        }
    }
    let mut uncovered = Vec::new();
    let mut irrelevant_count = 0;
    for (endpoint, paths, exact, whole) in [
        (
            Endpoint::Source,
            context.source_type.reachable_paths(),
            &source_exact,
            &source_whole,
        ),
        (
            Endpoint::Target,
            context.target_type.reachable_paths(),
            &target_exact,
            &target_whole,
        ),
    ] {
        for path in paths {
            let covered = exact.contains(&path)
                || whole
                    .iter()
                    .any(|whole_path| path_starts_with(&path, whole_path));
            let irrelevant = policy.explicitly_irrelevant_paths.iter().any(|item| {
                item.endpoint == endpoint
                    && matches!(
                        (safety, item.applies_to),
                        (
                            true,
                            IrrelevanceAppliesTo::Safety | IrrelevanceAppliesTo::Both
                        ) | (
                            false,
                            IrrelevanceAppliesTo::Matching | IrrelevanceAppliesTo::Both
                        )
                    )
                    && path_starts_with(&path, &item.path)
            });
            if irrelevant {
                irrelevant_count += 1;
            }
            if !covered && !irrelevant {
                uncovered.push(EndpointPath {
                    endpoint: match endpoint {
                        Endpoint::Source => "source",
                        Endpoint::Target => "target",
                    }
                    .into(),
                    path,
                });
            }
        }
    }
    uncovered.sort();
    CoverageReport {
        status: if uncovered.is_empty() {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
        relevant_path_count: context.source_type.reachable_paths().len()
            + context.target_type.reachable_paths().len(),
        observed_path_count: source_exact.union(&source_whole).count()
            + target_exact.union(&target_whole).count(),
        explicitly_irrelevant_path_count: irrelevant_count,
        uncovered_paths: uncovered,
    }
}

fn path_starts_with(path: &[String], prefix: &[String]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(a, b)| a == b)
}

fn carrier_observation_conflicts(
    summary: &CarrierSummary,
    policy: &EndpointPolicy,
    constructed: &ConstructedPolicy,
) -> Result<Vec<String>, CanonicalError> {
    let mut conflicts = Vec::new();
    for class in &summary.class_endpoint_pairs {
        let pair = ValuePair {
            source: class.source.clone(),
            target: class.target.clone(),
        };
        let pair_hash = canonical_sha256(&pair)?;
        let violated_safe: Vec<_> = policy
            .safe_dimensions
            .iter()
            .filter(|id| {
                constructed
                    .observations
                    .get(&((*id).clone(), pair.clone()))
                    .is_some_and(|(source, target)| {
                        policy.dimension(id).is_some_and(|dimension| {
                            !dimension.safe_relation.allows(source, target)
                        })
                    })
            })
            .cloned()
            .collect();
        if !violated_safe.is_empty() {
            conflicts.push(format!(
                "unsafe-carrier-class:{pair_hash}:{}",
                violated_safe.join(",")
            ));
        }
        if !policy.match_dimensions.is_empty() && !constructed.matched.contains(&pair) {
            conflicts.push(format!(
                "nonmatching-carrier-class:{pair_hash}:{}",
                policy.match_dimensions.join(",")
            ));
        }
    }
    conflicts.sort();
    conflicts.dedup();
    Ok(conflicts)
}

fn match_shape_compatibility(
    scope: &ResolvedScope,
    matched: &BTreeSet<ValuePair>,
    request: &ValidationRequest,
    roundtrips: &RoundTripEvaluation,
) -> Status {
    let functional = is_functional(matched);
    let inverse_functional = is_inverse_functional(matched);
    let required = request.required_laws.ids();
    let passed =
        |law| required.contains(&law) && roundtrips.laws[&law].status == Status::ProvedExhaustive;
    let source_projection: BTreeSet<_> = matched.iter().map(|pair| pair.source.clone()).collect();
    let target_projection: BTreeSet<_> = matched.iter().map(|pair| pair.target.clone()).collect();
    let source_native_covered = source_projection
        .iter()
        .all(|value| scope.source_domain.binary_search(value).is_ok());
    let target_native_covered = target_projection
        .iter()
        .all(|value| scope.target_domain.binary_search(value).is_ok());
    let source_full_covered = source_projection.iter().all(|value| {
        scope
            .source_full_transport_domain
            .binary_search(value)
            .is_ok()
    });
    let target_full_covered = target_projection.iter().all(|value| {
        scope
            .target_full_transport_domain
            .binary_search(value)
            .is_ok()
    });
    let compatible = match scope.comparator {
        ComparatorSpec::CarrierExact => {
            (!(passed(LawId::TargetNative) && target_native_covered) || functional)
                && (!(passed(LawId::SourceNative) && source_native_covered) || inverse_functional)
        }
        ComparatorSpec::TargetNativeExact => {
            functional
                && (!(passed(LawId::SourceFullTransport) && source_full_covered)
                    || inverse_functional)
        }
        ComparatorSpec::SourceNativeExact => {
            inverse_functional
                && (!(passed(LawId::TargetFullTransport) && target_full_covered) || functional)
        }
    };
    if compatible {
        Status::ProvedExhaustive
    } else {
        Status::Invalid
    }
}

fn is_functional(pairs: &BTreeSet<ValuePair>) -> bool {
    let mut map = BTreeMap::new();
    pairs.iter().all(|pair| {
        map.insert(pair.source.clone(), pair.target.clone())
            .is_none_or(|existing| existing == pair.target)
    })
}
fn is_inverse_functional(pairs: &BTreeSet<ValuePair>) -> bool {
    let mut map = BTreeMap::new();
    pairs.iter().all(|pair| {
        map.insert(pair.target.clone(), pair.source.clone())
            .is_none_or(|existing| existing == pair.source)
    })
}

#[allow(clippy::too_many_arguments)]
fn evaluate_properties(
    context: &AdapterContext,
    scope: &ResolvedScope,
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    constructed: &ConstructedPolicy,
    induced: &InducedRelation,
    safe_prerequisites_valid: bool,
    match_prerequisites_valid: bool,
    unsupported: bool,
    comparator_hash: &str,
    witness_envelope: &CommonEnvelope,
    witnesses: &mut BTreeMap<String, Witness>,
) -> Result<PropertiesReport, CanonicalError> {
    let fallback = if unsupported {
        Status::Unknown
    } else {
        Status::Invalid
    };
    let requested = |kind: &str| {
        request
            .required_properties
            .iter()
            .any(|property| property.kind() == kind)
    };
    let mut property = |kind: &str,
                        prerequisites_valid: bool,
                        failure: Option<(ValuePair, WitnessKind)>,
                        checked: usize|
     -> Result<PropertyResult, CanonicalError> {
        if !requested(kind) {
            return Ok(PropertyResult::not_requested());
        }
        if !prerequisites_valid {
            return Ok(PropertyResult {
                status: fallback,
                checked_pair_count: 0,
                witness_sha256: NOT_APPLICABLE.into(),
            });
        }
        if let Some((pair, witness_kind)) = failure {
            let evidence = induced
                .evaluations
                .get(&pair)
                .map(|evaluation| evaluation.evidence.clone())
                .unwrap_or(ComparatorEvidence::NotApplicable);
            let witness = policy_pair_witness(
                witness_kind,
                &pair,
                scope.comparator,
                comparator_hash,
                scope,
                policy,
                constructed,
                Some(evidence),
                witness_envelope,
            );
            let hash = install_witness(witness, witnesses)?;
            Ok(PropertyResult {
                status: Status::Disproved,
                checked_pair_count: checked,
                witness_sha256: hash,
            })
        } else {
            Ok(PropertyResult {
                status: Status::ProvedExhaustive,
                checked_pair_count: checked,
                witness_sha256: NOT_APPLICABLE.into(),
            })
        }
    };
    let sound_failure = induced
        .pairs
        .difference(&constructed.safe)
        .next()
        .cloned()
        .map(|pair| (pair, WitnessKind::UnsafeFalseAgreement));
    let adequate_failure = constructed
        .matched
        .difference(&induced.pairs)
        .next()
        .cloned()
        .map(|pair| (pair, WitnessKind::MissingRequiredMatch));
    let precise_failure = induced
        .pairs
        .difference(&constructed.matched)
        .next()
        .cloned()
        .map(|pair| (pair, WitnessKind::ExtraSafeEquality));
    let faithful_failure = constructed
        .matched
        .symmetric_difference(&induced.pairs)
        .next()
        .cloned()
        .map(|pair| {
            let kind = if constructed.matched.contains(&pair) {
                WitnessKind::MissingRequiredMatch
            } else {
                WitnessKind::ExtraSafeEquality
            };
            (pair, kind)
        });
    let policy_soundness = property(
        "policy-soundness",
        safe_prerequisites_valid,
        sound_failure,
        induced.pairs.len(),
    )?;
    let comparison_adequacy = property(
        "comparison-adequacy",
        match_prerequisites_valid,
        adequate_failure,
        constructed.matched.len(),
    )?;
    let comparison_precision = property(
        "comparison-precision",
        match_prerequisites_valid,
        precise_failure,
        induced.pairs.len(),
    )?;
    let faithful_comparison = property(
        "faithful-comparison",
        match_prerequisites_valid,
        faithful_failure,
        scope.comparison_universe.len(),
    )?;

    let target_non_amplification =
        if let Some(PropertyRequest::TargetNonAmplification { dimension_ids }) = request
            .required_properties
            .iter()
            .find(|property| matches!(property, PropertyRequest::TargetNonAmplification { .. }))
        {
            if !safe_prerequisites_valid {
                TargetNonAmplificationReport {
                    aggregate_status: fallback,
                    checked_dimension_count: 0,
                    checked_pair_count: 0,
                    dimensions: Vec::new(),
                }
            } else {
                let mut dimensions = Vec::new();
                let mut aggregate = Status::ProvedExhaustive;
                let mut pair_count = 0;
                for id in dimension_ids {
                    let Some(dimension) = policy.dimension(id) else {
                        aggregate = Status::Invalid;
                        continue;
                    };
                    if !policy.safe_dimensions.contains(id)
                        || !matches!(
                            dimension.safe_relation,
                            Relation::TargetNoAmplification { .. }
                        )
                    {
                        aggregate = Status::Invalid;
                        continue;
                    }
                    let mut failure = None;
                    let mut dimension_checked = 0;
                    for pair in &induced.pairs {
                        dimension_checked += 1;
                        let observations =
                            constructed.observations.get(&(id.clone(), pair.clone()));
                        if failure.is_none()
                            && observations.is_none_or(|(source, target)| {
                                !dimension.safe_relation.allows(source, target)
                            })
                        {
                            failure = Some(pair.clone());
                        }
                    }
                    pair_count += dimension_checked;
                    let witness_hash = if let Some(ref pair) = failure {
                        aggregate = Status::Disproved;
                        let evidence = induced.evaluations[pair].evidence.clone();
                        install_witness(
                            policy_pair_witness(
                                WitnessKind::UnsafeFalseAgreement,
                                pair,
                                scope.comparator,
                                comparator_hash,
                                scope,
                                policy,
                                constructed,
                                Some(evidence),
                                witness_envelope,
                            ),
                            witnesses,
                        )?
                    } else {
                        NOT_APPLICABLE.into()
                    };
                    dimensions.push(TnaDimensionResult {
                        dimension_id: id.clone(),
                        preorder_sha256: canonical_sha256(&dimension.safe_relation)?,
                        status: if failure.is_some() {
                            Status::Disproved
                        } else {
                            Status::ProvedExhaustive
                        },
                        checked_pair_count: dimension_checked,
                        witness_sha256: witness_hash,
                    });
                }
                TargetNonAmplificationReport {
                    aggregate_status: aggregate,
                    checked_dimension_count: dimensions.len(),
                    checked_pair_count: pair_count,
                    dimensions,
                }
            }
        } else {
            TargetNonAmplificationReport {
                aggregate_status: Status::NotRequested,
                checked_dimension_count: 0,
                checked_pair_count: 0,
                dimensions: Vec::new(),
            }
        };
    // `context` is intentionally consumed by the same evaluator path even
    // though all comparison evidence was already cached above.
    let _ = context;
    Ok(PropertiesReport {
        policy_soundness,
        comparison_adequacy,
        comparison_precision,
        faithful_comparison,
        target_non_amplification,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn policy_pair_witness(
    kind: WitnessKind,
    pair: &ValuePair,
    comparator: ComparatorSpec,
    comparator_hash: &str,
    scope: &ResolvedScope,
    policy: &EndpointPolicy,
    constructed: &ConstructedPolicy,
    evidence: Option<ComparatorEvidence>,
    witness_envelope: &CommonEnvelope,
) -> Witness {
    let violated: Vec<String> = match kind {
        WitnessKind::UnsafeFalseAgreement => policy
            .safe_dimensions
            .iter()
            .filter(|id| {
                constructed
                    .observations
                    .get(&((*id).clone(), pair.clone()))
                    .is_some_and(|(source, target)| {
                        policy.dimension(id).is_some_and(|dimension| {
                            !dimension.safe_relation.allows(source, target)
                        })
                    })
            })
            .cloned()
            .collect(),
        WitnessKind::MissingRequiredMatch | WitnessKind::ExtraSafeEquality => {
            policy.match_dimensions.clone()
        }
        _ => Vec::new(),
    };
    let adapter_path = violated
        .iter()
        .filter_map(|id| policy.dimension(id))
        .flat_map(|dimension| {
            dimension
                .source_observer
                .observed_paths()
                .into_iter()
                .chain(dimension.target_observer.observed_paths())
        })
        .next()
        .unwrap_or_default();
    Witness {
        envelope: witness_envelope.clone(),
        witness_kind: kind,
        source_value: EvidenceValue::Present(pair.source.clone()),
        target_value: EvidenceValue::Present(pair.target.clone()),
        comparator_kind: EvidenceValue::Present(comparator),
        comparator_spec_sha256: comparator_hash.into(),
        comparator_evidence: evidence.unwrap_or(ComparatorEvidence::NotApplicable),
        violated_or_missing_dimensions: violated,
        adapter_path,
        replay_command: vec![
            "gluerift".into(),
            "check".into(),
            "--context".into(),
            "<context>".into(),
            "--scope".into(),
            "<scope>".into(),
            "--policy".into(),
            "<policy>".into(),
            "--request".into(),
            "<request>".into(),
        ],
        coverage_mode: "not-applicable".into(),
        source_comparison_domain_sha256: scope.source_comparison_domain_sha256.clone(),
        target_comparison_domain_sha256: scope.target_comparison_domain_sha256.clone(),
        match_pair_count: EvidenceValue::Present(constructed.matched.len()),
        safe_membership: EvidenceValue::Present(constructed.safe.contains(pair)),
        match_membership: EvidenceValue::Present(constructed.matched.contains(pair)),
        roundtrip_trace: Vec::new(),
    }
}

fn install_roundtrip_witnesses(
    evaluation: &mut RoundTripEvaluation,
    scope: &ResolvedScope,
    comparator_hash: &str,
    request_id: &str,
    witness_envelope: &CommonEnvelope,
    witnesses: &mut BTreeMap<String, Witness>,
) -> Result<(), CanonicalError> {
    for report in evaluation
        .laws
        .values_mut()
        .filter(|report| report.status == Status::Disproved)
    {
        let input = match &report.first_failing_input {
            EvidenceValue::Present(value) => Some(value.clone()),
            EvidenceValue::Absent(_) => None,
        };
        let (source, target) = match report.law_id {
            LawId::SourceNative | LawId::SourceFullTransport => (input, None),
            LawId::TargetNative | LawId::TargetFullTransport => (None, input),
            LawId::SourceCarrier | LawId::TargetCarrier => (None, None),
        };
        let witness = Witness {
            envelope: witness_envelope.clone(),
            witness_kind: WitnessKind::RoundtripFailure,
            source_value: EvidenceValue::from_option(source),
            target_value: EvidenceValue::from_option(target),
            comparator_kind: EvidenceValue::Present(scope.comparator),
            comparator_spec_sha256: comparator_hash.into(),
            comparator_evidence: ComparatorEvidence::NotApplicable,
            violated_or_missing_dimensions: Vec::new(),
            adapter_path: report
                .first_failure_trace
                .last()
                .map(|trace| vec![trace.stage.clone()])
                .unwrap_or_default(),
            replay_command: vec![
                "gluerift".into(),
                "roundtrip".into(),
                "--request".into(),
                request_id.into(),
            ],
            coverage_mode: "not-applicable".into(),
            source_comparison_domain_sha256: scope.source_comparison_domain_sha256.clone(),
            target_comparison_domain_sha256: scope.target_comparison_domain_sha256.clone(),
            match_pair_count: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            safe_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            match_membership: EvidenceValue::Absent(AbsentEvidence::NotApplicable),
            roundtrip_trace: report.first_failure_trace.clone(),
        };
        report.witness_sha256 = install_witness(witness, witnesses)?;
    }
    Ok(())
}

pub(crate) fn install_witness(
    mut witness: Witness,
    store: &mut BTreeMap<String, Witness>,
) -> Result<String, CanonicalError> {
    let base_id = witness.envelope.evidence_id.clone();
    witness.envelope.evidence_id = "gluerift-witness-seed".into();
    let seed = canonical_sha256(&witness)?;
    witness.envelope.evidence_id = format!(
        "{base_id}:{}:{}",
        witness_kind_name(witness.witness_kind),
        &seed[..16]
    );
    let hash = canonical_sha256(&witness)?;
    store.insert(hash.clone(), witness);
    Ok(hash)
}

pub(crate) fn witness_evidence_id<'a>(
    store: &'a BTreeMap<String, Witness>,
    hash: &str,
) -> Option<&'a str> {
    store
        .get(hash)
        .map(|witness| witness.envelope.evidence_id.as_str())
}

fn witness_kind_name(kind: WitnessKind) -> &'static str {
    match kind {
        WitnessKind::UnsafeFalseAgreement => "unsafe-false-agreement",
        WitnessKind::MissingRequiredMatch => "missing-required-match",
        WitnessKind::ExtraSafeEquality => "extra-safe-equality",
        WitnessKind::ComparatorUndefined => "comparator-undefined",
        WitnessKind::BridgeDivergence => "bridge-divergence",
        WitnessKind::RoundtripFailure => "roundtrip-failure",
        WitnessKind::MatchCoverageEmpty => "match-coverage-empty",
        WitnessKind::MatchCoverageSourceGap => "match-coverage-source-gap",
        WitnessKind::MatchCoverageTargetGap => "match-coverage-target-gap",
        WitnessKind::SafeMatchDivergence => "safe-match-divergence",
    }
}

fn normalized_path(path: &str) -> Vec<String> {
    path.split(['.', '/'])
        .filter(|segment| !segment.is_empty() && *segment != "$")
        .map(str::to_owned)
        .collect()
}

fn law_id_name(law: LawId) -> &'static str {
    match law {
        LawId::SourceNative => "source-native",
        LawId::TargetNative => "target-native",
        LawId::SourceCarrier => "source-carrier",
        LawId::TargetCarrier => "target-carrier",
        LawId::SourceFullTransport => "source-full-transport",
        LawId::TargetFullTransport => "target-full-transport",
    }
}

fn requested_property_statuses(
    request: &ValidationRequest,
    properties: &PropertiesReport,
) -> Vec<Status> {
    request
        .required_properties
        .iter()
        .map(|property| match property {
            PropertyRequest::PolicySoundness => properties.policy_soundness.status,
            PropertyRequest::ComparisonAdequacy => properties.comparison_adequacy.status,
            PropertyRequest::ComparisonPrecision => properties.comparison_precision.status,
            PropertyRequest::FaithfulComparison => properties.faithful_comparison.status,
            PropertyRequest::TargetNonAmplification { .. } => {
                properties.target_non_amplification.aggregate_status
            }
        })
        .collect()
}

fn check_hash(field: &'static str, expected: &str, actual: &str) -> Result<(), CheckError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CheckError::HashMismatch {
            field,
            expected: expected.into(),
            actual: actual.into(),
        })
    }
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::ProvedExhaustive => "proved-exhaustive",
        Status::Disproved => "disproved",
        Status::Unknown => "unknown",
        Status::NotRequested => "not-requested",
        Status::Invalid => "invalid",
        Status::ToolError => "tool-error",
    }
}

fn match_coverage_mode_name(mode: MatchCoverageMode) -> &'static str {
    match mode {
        MatchCoverageMode::None => "none",
        MatchCoverageMode::Nonempty => "nonempty",
        MatchCoverageMode::SourceTotal => "source-total",
        MatchCoverageMode::TargetTotal => "target-total",
        MatchCoverageMode::BidirectionalTotal => "bidirectional-total",
    }
}

fn aggregate_status(statuses: impl IntoIterator<Item = Status>) -> Status {
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

fn property_rank(kind: &str) -> usize {
    match kind {
        "policy-soundness" => 0,
        "comparison-adequacy" => 1,
        "comparison-precision" => 2,
        "faithful-comparison" => 3,
        "target-non-amplification" => 4,
        _ => usize::MAX,
    }
}
