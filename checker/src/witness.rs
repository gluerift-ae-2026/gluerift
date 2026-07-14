use serde::{Deserialize, Serialize};

use crate::adapter_ir::ConversionError;
use crate::canonical::{CanonicalError, canonical_sha256};
use crate::domain::ComparatorSpec;
use crate::report::{CommonEnvelope, EvidenceValue, StageTrace, Status};
use crate::type_ir::Value;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ComparatorEvidence {
    CarrierExact {
        source_encoding: Result<Value, ConversionError>,
        target_encoding: Result<Value, ConversionError>,
        common_carrier: EvidenceValue<Value>,
    },
    TargetNativeExact {
        source_encoding: Result<Value, ConversionError>,
        target_decode_result: Result<Value, ConversionError>,
        compared_target_value: Value,
    },
    SourceNativeExact {
        target_encoding: Result<Value, ConversionError>,
        source_decode_result: Result<Value, ConversionError>,
        compared_source_value: Value,
    },
    NotApplicable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WitnessKind {
    UnsafeFalseAgreement,
    MissingRequiredMatch,
    ExtraSafeEquality,
    ComparatorUndefined,
    BridgeDivergence,
    RoundtripFailure,
    MatchCoverageEmpty,
    MatchCoverageSourceGap,
    MatchCoverageTargetGap,
    SafeMatchDivergence,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    #[serde(flatten)]
    pub envelope: CommonEnvelope,
    pub witness_kind: WitnessKind,
    pub source_value: EvidenceValue<Value>,
    pub target_value: EvidenceValue<Value>,
    pub comparator_kind: EvidenceValue<ComparatorSpec>,
    pub comparator_spec_sha256: String,
    pub comparator_evidence: ComparatorEvidence,
    pub violated_or_missing_dimensions: Vec<String>,
    pub adapter_path: Vec<String>,
    pub replay_command: Vec<String>,
    pub coverage_mode: String,
    pub source_comparison_domain_sha256: String,
    pub target_comparison_domain_sha256: String,
    pub match_pair_count: EvidenceValue<usize>,
    pub safe_membership: EvidenceValue<bool>,
    pub match_membership: EvidenceValue<bool>,
    pub roundtrip_trace: Vec<StageTrace>,
}

impl Witness {
    pub fn with_envelope(envelope: &CommonEnvelope, witness_kind: WitnessKind) -> Self {
        let mut envelope = envelope.clone();
        envelope.schema = "gluerift.witness/v0.3.1a".into();
        envelope.status = Status::Disproved;
        Self {
            envelope,
            witness_kind,
            source_value: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            target_value: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            comparator_kind: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            comparator_spec_sha256: "not-applicable".into(),
            comparator_evidence: ComparatorEvidence::NotApplicable,
            violated_or_missing_dimensions: Vec::new(),
            adapter_path: Vec::new(),
            replay_command: Vec::new(),
            coverage_mode: "not-applicable".into(),
            source_comparison_domain_sha256: "not-applicable".into(),
            target_comparison_domain_sha256: "not-applicable".into(),
            match_pair_count: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            safe_membership: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            match_membership: EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable),
            roundtrip_trace: Vec::new(),
        }
    }

    pub fn sha256(&self) -> Result<String, CanonicalError> {
        canonical_sha256(self)
    }
}
