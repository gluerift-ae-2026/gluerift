use crate::canonical;
use crate::evidence::{ReferenceBinding, ReferenceBindings};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterTruthTableRow {
    pub input: Value,
    pub output: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetNativeRelationRow {
    pub source: Value,
    pub target: Value,
    pub transported_source_as_target_native: Value,
    pub equal: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTruthStage {
    pub stage: String,
    pub output: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoundTripTruthRow {
    pub input: Value,
    pub stages: Vec<NativeTruthStage>,
    pub final_matches_input: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRoundTripTruthTable {
    pub law_id: String,
    pub rows: Vec<NativeRoundTripTruthRow>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeUnsafeWitness {
    pub witness_kind: String,
    pub source_value: Value,
    pub target_value: Value,
    pub transported_source_as_target_native: Value,
    pub semantic_path: Vec<String>,
    pub violated_or_missing_dimensions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReferenceBundle {
    pub schema: String,
    pub semantic_contract_version: String,
    pub tool_build_sha256: String,
    pub run_configuration_sha256: String,
    pub evidence_id: String,
    pub dependency_evidence_ids: Vec<String>,
    pub status: String,
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
    pub comparison_universe: Vec<Value>,
    pub source_encode_truth_table: Vec<AdapterTruthTableRow>,
    pub source_decode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_encode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_decode_truth_table: Vec<AdapterTruthTableRow>,
    pub target_native_relation_truth_table: Vec<TargetNativeRelationRow>,
    pub six_roundtrip_truth_tables: BTreeMap<String, NativeRoundTripTruthTable>,
    pub comparator_definedness: Value,
    pub canonical_unsafe_witness: NativeUnsafeWitness,
    pub canonical_unsafe_witness_sha256: String,
}

impl NativeReferenceBundle {
    pub fn read(
        repo: &Path,
        bindings: &ReferenceBindings,
        binding: &ReferenceBinding,
    ) -> Result<Self> {
        let path = repo.join(&binding.reference_bundle_logical_path);
        let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        let raw_hash = canonical::sha256_bytes(&bytes);
        if raw_hash != binding.reference_bundle_sha256 {
            bail!("{} reference bundle file hash mismatch", binding.fixture_id)
        }
        let bundle: Self =
            serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))?;
        if canonical::sha256(&bundle_as_value(&bytes)?)? != binding.reference_bundle_sha256 {
            bail!(
                "{} reference bundle is not canonical/hash stable",
                binding.fixture_id
            )
        }
        bundle.validate(bindings, binding)?;
        Ok(bundle)
    }

    fn validate(&self, bindings: &ReferenceBindings, binding: &ReferenceBinding) -> Result<()> {
        if self.schema != "gluerift.native-reference-bundle/v0.3.1a"
            || self.semantic_contract_version != "0.3.1a"
            || self.status != "proved-exhaustive"
            || self.fixture_id != binding.fixture_id
            || self.reference_run_id != binding.reference_run_id
            || self.source_inputs_manifest_sha256 != bindings.source_inputs_manifest_sha256
            || self.source_tree_sha256 != bindings.source_tree_sha256
            || self.candidate_context_sha256 != binding.reference_candidate_sha256
            || self.transformation_report_sha256 != binding.transformation_report_sha256
            || self.reference_check_report_sha256 != binding.reference_check_report_sha256
            || self.reference_check_evidence_id != binding.reference_check_evidence_id
            || self.run_configuration_sha256 != binding.run_configuration_sha256
            || self.types_sha256 != binding.types_sha256
            || self.validation_scope_sha256 != binding.validation_scope_sha256
            || self.endpoint_policy_sha256 != binding.endpoint_policy_sha256
            || self.validation_request_sha256 != binding.validation_request_sha256
            || self.comparator_spec_sha256 != binding.comparator_spec_sha256
        {
            bail!(
                "{} reference bundle provenance binding mismatch",
                binding.fixture_id
            )
        }
        if self.six_roundtrip_truth_tables.len() != 6
            || self.comparator_definedness.get("status")
                != Some(&Value::String("proved-exhaustive".into()))
            || self.canonical_unsafe_witness.witness_kind != "unsafe-false-agreement"
            || canonical::sha256(&self.canonical_unsafe_witness)?
                != self.canonical_unsafe_witness_sha256
        {
            bail!(
                "{} reference bundle semantic invariant failed",
                binding.fixture_id
            )
        }
        Ok(())
    }
}

fn bundle_as_value(bytes: &[u8]) -> Result<Value> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Regression guard for the dual-model failure mode: observations are judged
/// exclusively against the immutable checker bundle, never a local model.
pub fn observation_mismatches_bundle(observed: &Value, bundled: &Value) -> bool {
    observed != bundled
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn joint_native_and_legacy_model_mutation_is_rejected_by_bundle() {
        let immutable_checker_bundle = json!({"variant": "DENY"});
        let mutated_native = json!({"variant": "ALLOW"});
        let mutated_legacy_local_model = mutated_native.clone();
        assert_eq!(mutated_native, mutated_legacy_local_model);
        assert!(observation_mismatches_bundle(
            &mutated_native,
            &immutable_checker_bundle
        ));
    }
}
