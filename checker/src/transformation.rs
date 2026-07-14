use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CONTRACT_VERSION;
use crate::adapter_ir::{Adapter, AdapterContext, BranchMapping, ProductFieldMap, SumVariantMap};
use crate::canonical::{CanonicalError, canonical_bytes, canonical_sha256};
use crate::comparator::evaluate_pair;
use crate::comparison::{
    CheckedRun, EvidenceMetadata, RunConfiguration, ValidationRequest, check, install_witness,
    policy_pair_witness, witness_evidence_id,
};
use crate::domain::ValidationScope;
use crate::relation_ir::EndpointPolicy;
use crate::report::{
    BindingStatus, CommonEnvelope, NOT_APPLICABLE, NOT_REQUIRED_DIRECT_NATIVE, Status,
    TransformationClassification, TransformationReport,
};
use crate::type_ir::{EnumerationLimits, ResultBranch, Type, Value, Variant};
use crate::witness::WitnessKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformationFamilyDescriptor {
    pub schema: String,
    pub semantic_contract_version: String,
    pub family_id: String,
    pub action_domain: ActionDomainRule,
    pub enumerated_generators: Vec<GeneratorDescriptor>,
    pub admitted_declared_candidates: Vec<DeclaredCandidateDescriptor>,
    pub normalization: NormalizationDescriptor,
    pub inverse_requirement: InverseRequirementDescriptor,
    pub twist: TwistDescriptor,
    pub completeness_statement: Vec<String>,
    pub scalar_discovery_completeness_claimed: bool,
    pub general_automorphism_completeness_claimed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDomainRule {
    pub kind: String,
    pub ownership: String,
    pub ordering: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorDescriptor {
    pub kind: String,
    pub generation_rule_id: String,
    pub compatibility: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredCandidateDescriptor {
    pub kind: String,
    pub generation_rule_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizationDescriptor {
    pub path_order: String,
    pub composition_order: String,
    pub identity_elimination: bool,
    pub flatten_compose: bool,
    pub duplicate_elimination: String,
    pub generator_ordinal_order: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InverseRequirementDescriptor {
    pub domain: String,
    pub left_identity: String,
    pub right_identity: String,
    pub totality: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TwistDescriptor {
    pub side: String,
    pub construction: String,
    pub source_encode: String,
    pub source_decode: String,
    pub target_encode: String,
    pub target_decode: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformationCandidate {
    pub generation_mode: GenerationMode,
    pub generation_rule_id: String,
    pub generation_parent_path: Vec<String>,
    pub generation_ordinal: usize,
    pub transformation_ir: Adapter,
    pub inverse_ir: Adapter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GenerationMode {
    Enumerated,
    DeclaredCandidate,
}

#[derive(Clone, Debug)]
pub struct ClassifiedTransformation {
    pub report: TransformationReport,
    pub transformed_context: AdapterContext,
    pub transformed_run: CheckedRun,
}

#[derive(Debug, Error)]
pub enum TransformationError {
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("transformation-family schema must be gluerift.transformation-family/v0.3.1a")]
    WrongSchema,
    #[error("transformation family hash differs from request binding")]
    FamilyHashMismatch,
    #[error("transformation enumeration exceeded configured limit {0}")]
    Limit(usize),
    #[error("candidate transformation cannot produce a typed transformed context: {0}")]
    Inapplicable(String),
    #[error("transformed semantic check failed structurally: {0}")]
    Check(String),
}

impl TransformationFamilyDescriptor {
    pub fn validate(&self) -> Result<(), TransformationError> {
        if self.schema != "gluerift.transformation-family/v0.3.1a" {
            return Err(TransformationError::WrongSchema);
        }
        if self.semantic_contract_version != CONTRACT_VERSION
            || self.family_id != "core-structural"
            || self.action_domain.kind != "all"
            || self.action_domain.ownership != "derived-from-carrier-type"
            || self.action_domain.ordering != "canonical-type-value-order"
            || self.normalization.path_order != "lexicographic"
            || self.normalization.composition_order != "right-to-left-application"
            || !self.normalization.identity_elimination
            || !self.normalization.flatten_compose
            || self.normalization.duplicate_elimination != "canonical-adapter-sha256"
            || self.normalization.generator_ordinal_order != "normalized-ir-lexicographic"
            || self.inverse_requirement.domain != "complete-all-carrier-values"
            || self.inverse_requirement.left_identity != "exhaustive"
            || self.inverse_requirement.right_identity != "exhaustive"
            || self.inverse_requirement.totality != "required"
            || self.twist.side != "target"
            || self.twist.construction != "carrier-conjugation"
            || self.scalar_discovery_completeness_claimed
            || self.general_automorphism_completeness_claimed
        {
            return Err(TransformationError::Inapplicable(
                "family descriptor violates the frozen Core semantics".into(),
            ));
        }
        let expected_generators = vec![
            GeneratorDescriptor {
                kind: "enum-permutation".into(),
                generation_rule_id: "core.enum.payload-compatible".into(),
                compatibility: "identical-payload-type".into(),
            },
            GeneratorDescriptor {
                kind: "field-permutation".into(),
                generation_rule_id: "core.product.same-typed-fields".into(),
                compatibility: "identical-field-type".into(),
            },
            GeneratorDescriptor {
                kind: "nested-structural-composition".into(),
                generation_rule_id: "core.nested.canonical-path-product".into(),
                compatibility: "recursive-compatible-subterm".into(),
            },
            GeneratorDescriptor {
                kind: "result-branch-permutation".into(),
                generation_rule_id: "core.result.compatible-branches".into(),
                compatibility: "identical-branch-payload-type".into(),
            },
        ];
        let expected_declared = vec![
            DeclaredCandidateDescriptor {
                kind: "bounded-complement".into(),
                generation_rule_id: "core.scalar.declared-bounded-complement".into(),
            },
            DeclaredCandidateDescriptor {
                kind: "modular-affine".into(),
                generation_rule_id: "core.scalar.declared-modular-affine".into(),
            },
        ];
        if self.enumerated_generators != expected_generators
            || self.admitted_declared_candidates != expected_declared
            || self.completeness_statement
                != [
                    "exact_within_core_structural_family",
                    "unknown_outside_declared_family",
                ]
        {
            return Err(TransformationError::Inapplicable(
                "family generators, declared scalar registry, or completeness wording differ from the frozen Core descriptor".into(),
            ));
        }
        let generator_ids: BTreeSet<_> = self
            .enumerated_generators
            .iter()
            .map(|entry| &entry.generation_rule_id)
            .collect();
        let declared_ids: BTreeSet<_> = self
            .admitted_declared_candidates
            .iter()
            .map(|entry| &entry.generation_rule_id)
            .collect();
        if generator_ids.len() != self.enumerated_generators.len()
            || declared_ids.len() != self.admitted_declared_candidates.len()
        {
            return Err(TransformationError::Inapplicable(
                "family generator rule IDs must be unique".into(),
            ));
        }
        Ok(())
    }

    pub fn enumerate(
        &self,
        carrier: &Type,
        limits: &RunConfiguration,
    ) -> Result<Vec<TransformationCandidate>, TransformationError> {
        self.validate()?;
        let carrier = carrier.normalized();
        let mut candidates = if self.enumerated_generators.is_empty() {
            Vec::new()
        } else {
            structural_candidates(
                &carrier,
                &limits.enumeration_limits(),
                limits.max_transformations,
            )?
        };
        deduplicate_candidates(&mut candidates)?;
        if candidates.len() > limits.max_transformations {
            return Err(TransformationError::Limit(limits.max_transformations));
        }
        for (ordinal, candidate) in candidates.iter_mut().enumerate() {
            candidate.generation_ordinal = ordinal;
        }
        Ok(candidates)
    }

    /// Checks that an authored candidate is actually a member of the frozen
    /// family.  Classification alone is not admission: enumerated candidates
    /// must bind the generator's normalized IR, inverse, rule, parent path and
    /// canonical ordinal; scalar candidates must bind an explicitly admitted
    /// declared kind/rule.
    pub fn resolve_candidate(
        &self,
        candidate: &TransformationCandidate,
        carrier: &Type,
        limits: &RunConfiguration,
    ) -> Result<TransformationCandidate, TransformationError> {
        self.validate()?;
        match candidate.generation_mode {
            GenerationMode::Enumerated => {
                let normalized = candidate.transformation_ir.normalize();
                let inverse = candidate.inverse_ir.normalize();
                let found = self
                    .enumerate(carrier, limits)?
                    .into_iter()
                    .find(|generated| {
                        generated.transformation_ir == normalized
                            && generated.inverse_ir == inverse
                            && generated.generation_rule_id == candidate.generation_rule_id
                            && generated.generation_parent_path == candidate.generation_parent_path
                    });
                found.ok_or_else(|| TransformationError::Inapplicable("enumerated candidate is not the canonical family member named by its normalized IR, inverse, rule and parent path".into()))
            }
            GenerationMode::DeclaredCandidate => {
                let kind = match candidate.transformation_ir.normalize() {
                    Adapter::BoundedComplement { .. } => "bounded-complement",
                    Adapter::ModularAffine { .. } => "modular-affine",
                    _ => return Err(TransformationError::Inapplicable("declared candidate uses an adapter kind outside the declared scalar registry".into())),
                };
                if !self.admitted_declared_candidates.iter().any(|entry| {
                    entry.kind == kind && entry.generation_rule_id == candidate.generation_rule_id
                }) {
                    return Err(TransformationError::Inapplicable(format!(
                        "declared {kind} candidate is not admitted by generation rule {}",
                        candidate.generation_rule_id
                    )));
                }
                Ok(TransformationCandidate {
                    generation_mode: candidate.generation_mode,
                    generation_rule_id: candidate.generation_rule_id.clone(),
                    generation_parent_path: candidate.generation_parent_path.clone(),
                    generation_ordinal: candidate.generation_ordinal,
                    transformation_ir: candidate.transformation_ir.normalize(),
                    inverse_ir: candidate.inverse_ir.normalize(),
                })
            }
        }
    }
}

pub fn construct_twist(
    base: &AdapterContext,
    candidate: &TransformationCandidate,
) -> AdapterContext {
    let base = base.normalized();
    AdapterContext {
        schema: base.schema.clone(),
        source_type: base.source_type.clone(),
        target_type: base.target_type.clone(),
        carrier_type: base.carrier_type.clone(),
        source_encode: base.source_encode.clone(),
        source_decode: base.source_decode.clone(),
        target_encode: Adapter::Compose {
            first: Box::new(base.target_encode.clone()),
            second: Box::new(candidate.transformation_ir.clone()),
        }
        .normalize(),
        target_decode: Adapter::Compose {
            first: Box::new(candidate.inverse_ir.clone()),
            second: Box::new(base.target_decode.clone()),
        }
        .normalize(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn classify_transformation(
    base: &AdapterContext,
    candidate: &TransformationCandidate,
    family: &TransformationFamilyDescriptor,
    scope: &ValidationScope,
    policy: &EndpointPolicy,
    request: &ValidationRequest,
    run_configuration: &RunConfiguration,
    metadata: &EvidenceMetadata,
    base_run: Option<&CheckedRun>,
) -> Result<ClassifiedTransformation, TransformationError> {
    family.validate()?;
    let family_hash = canonical_sha256(family)?;
    if request.required_transformation_family_sha256 != family_hash {
        return Err(TransformationError::FamilyHashMismatch);
    }
    let resolved_candidate =
        family.resolve_candidate(candidate, &base.carrier_type, run_configuration)?;
    let candidate = &resolved_candidate;
    let action_domain = base
        .carrier_type
        .enumerate(&run_configuration.enumeration_limits())
        .map_err(|error| TransformationError::Inapplicable(error.to_string()))?;
    let inverse_ok = inverse_check(candidate, &base.carrier_type, &action_domain);
    let transformed = construct_twist(base, candidate);
    let mut transformed_run = check(
        &transformed,
        scope,
        policy,
        request,
        run_configuration,
        metadata,
    )
    .map_err(|error| TransformationError::Check(error.to_string()))?;
    let transformed_hash = canonical_sha256(&transformed)?;
    let transformed_check_hash = canonical_sha256(&transformed_run.check_report)?;
    let candidate_binding =
        if transformed_run.check_report.envelope.candidate_sha256 == transformed_hash {
            BindingStatus::ProvedExhaustive
        } else {
            BindingStatus::ToolError
        };
    let well_typed = transformed
        .validate(&run_configuration.enumeration_limits())
        .is_ok();
    let required_laws = request.required_laws.ids();
    let roundtrip_statuses: BTreeMap<_, _> = transformed_run
        .roundtrip_report
        .laws
        .iter()
        .map(|law| (law.law_id, law.status))
        .collect();
    let all_laws = required_laws
        .iter()
        .all(|law| roundtrip_statuses[law] == Status::ProvedExhaustive);
    let defined = transformed_run
        .check_report
        .comparison
        .comparator_definedness
        .status
        == Status::ProvedExhaustive;
    let construction_ok = verify_construction(
        base,
        &transformed,
        candidate,
        &run_configuration.enumeration_limits(),
    );
    if candidate.generation_mode == GenerationMode::Enumerated && (!inverse_ok || !construction_ok)
    {
        return Err(TransformationError::Check("generated structural family member failed its inverse or four-map construction conformance check".into()));
    }
    let lawful = well_typed && inverse_ok && construction_ok && defined && all_laws;
    // Transformation classification owns a direct Sound check independently
    // of the request's display properties (§16.2 step 7).  If PS was also
    // requested, the common semantic kernel must agree exactly.
    let resolved_scope = scope
        .resolve(base, &run_configuration.enumeration_limits())
        .map_err(|error| TransformationError::Check(error.to_string()))?;
    let source_observer_domain = base
        .source_type
        .enumerate(&run_configuration.enumeration_limits())
        .map_err(|error| TransformationError::Check(error.to_string()))?;
    let target_observer_domain = base
        .target_type
        .enumerate(&run_configuration.enumeration_limits())
        .map_err(|error| TransformationError::Check(error.to_string()))?;
    let constructed_policy = policy
        .construct(
            &resolved_scope,
            &base.source_type,
            &base.target_type,
            &source_observer_domain,
            &target_observer_domain,
        )
        .map_err(|error| TransformationError::Check(error.to_string()))?;
    if constructed_policy.safe != transformed_run.safe {
        return Err(TransformationError::Check(
            "independent transformation soundness policy differs from transformed check".into(),
        ));
    }
    let sound_failure = transformed_run
        .induced_relation
        .difference(&transformed_run.safe)
        .next()
        .cloned();
    let sound_status = if sound_failure.is_some() {
        Status::Disproved
    } else {
        Status::ProvedExhaustive
    };
    let reported_soundness = transformed_run
        .check_report
        .properties
        .policy_soundness
        .status;
    if reported_soundness != Status::NotRequested && reported_soundness != sound_status {
        return Err(TransformationError::Check(format!(
            "requested soundness status {reported_soundness:?} disagrees with direct transformation status {sound_status:?}"
        )));
    }
    let classification = if lawful {
        if sound_status == Status::ProvedExhaustive {
            TransformationClassification::LawfulSafe
        } else {
            TransformationClassification::LawfulHarmful
        }
    } else {
        TransformationClassification::LawBreakingOrInapplicable
    };
    let mut reasons = Vec::new();
    if !well_typed {
        reasons.push("ill-typed".into());
    }
    if !inverse_ok {
        reasons.push("inverse-invalid".into());
    }
    if !construction_ok {
        reasons.push("conjugation-construction-invalid".into());
    }
    if !defined {
        reasons.push("comparator-undefined".into());
    }
    if !all_laws {
        reasons.push("required-law-disproved".into());
    }
    let base_hash = canonical_sha256(&base.normalized())?;
    let (base_check_hash, base_check_evidence_id, base_alignment) = if let Some(run) = base_run {
        let all_base_laws = run
            .roundtrip_report
            .laws
            .iter()
            .all(|law| law.status == Status::ProvedExhaustive);
        let base_envelope = &run.check_report.envelope;
        let transformed_envelope = &transformed_run.check_report.envelope;
        let aligned = base_envelope.candidate_sha256 == base_hash
            && base_envelope.comparator_spec_sha256 == transformed_envelope.comparator_spec_sha256
            && base_envelope.validation_scope_sha256
                == transformed_envelope.validation_scope_sha256
            && base_envelope.endpoint_policy_sha256 == transformed_envelope.endpoint_policy_sha256
            && base_envelope.validation_request_sha256
                == transformed_envelope.validation_request_sha256
            && base_envelope.run_configuration_sha256
                == transformed_envelope.run_configuration_sha256
            && run.check_report.comparison.comparator_definedness.status
                == Status::ProvedExhaustive
            && all_base_laws
            && [
                run.check_report.properties.policy_soundness.status,
                run.check_report.properties.comparison_adequacy.status,
                run.check_report.properties.comparison_precision.status,
                run.check_report.properties.faithful_comparison.status,
            ]
            .into_iter()
            .all(Status::is_proved);
        (
            canonical_sha256(&run.check_report)?,
            Some(run.check_report.envelope.evidence_id.clone()),
            if aligned {
                BindingStatus::ProvedExhaustive
            } else {
                BindingStatus::ToolError
            },
        )
    } else {
        (NOT_APPLICABLE.into(), None, BindingStatus::NotRequired)
    };
    let selected_properties = BTreeMap::from([
        (
            "policy-soundness".into(),
            transformed_run
                .check_report
                .properties
                .policy_soundness
                .status,
        ),
        (
            "comparison-adequacy".into(),
            transformed_run
                .check_report
                .properties
                .comparison_adequacy
                .status,
        ),
        (
            "comparison-precision".into(),
            transformed_run
                .check_report
                .properties
                .comparison_precision
                .status,
        ),
        (
            "faithful-comparison".into(),
            transformed_run
                .check_report
                .properties
                .faithful_comparison
                .status,
        ),
        (
            "target-non-amplification".into(),
            transformed_run
                .check_report
                .properties
                .target_non_amplification
                .aggregate_status,
        ),
    ]);
    let harmful_witness = if classification == TransformationClassification::LawfulHarmful {
        let existing = &transformed_run
            .check_report
            .properties
            .policy_soundness
            .witness_sha256;
        if existing != NOT_APPLICABLE {
            existing.clone()
        } else {
            let pair = sound_failure.clone().ok_or_else(|| {
                TransformationError::Check(
                    "lawful-harmful classification has no unsafe induced pair".into(),
                )
            })?;
            let evidence = evaluate_pair(&transformed, scope.comparator, &pair).evidence;
            let mut witness_envelope = transformed_run.check_report.envelope.clone();
            witness_envelope.schema = "gluerift.witness/v0.3.1a".into();
            witness_envelope.evidence_id = format!(
                "{}:transformation-witness",
                transformed_run.check_report.envelope.evidence_id
            );
            witness_envelope.status = Status::Disproved;
            install_witness(
                policy_pair_witness(
                    WitnessKind::UnsafeFalseAgreement,
                    &pair,
                    scope.comparator,
                    &transformed_run.check_report.envelope.comparator_spec_sha256,
                    &resolved_scope,
                    policy,
                    &constructed_policy,
                    Some(evidence),
                    &witness_envelope,
                ),
                &mut transformed_run.witnesses,
            )?
        }
    } else {
        NOT_APPLICABLE.into()
    };
    let transformed_bridge_report_sha256 = match scope.comparator {
        crate::domain::ComparatorSpec::CarrierExact => NOT_APPLICABLE.into(),
        crate::domain::ComparatorSpec::TargetNativeExact
        | crate::domain::ComparatorSpec::SourceNativeExact => {
            // Classification above consumes the selected native relation
            // directly.  The independently evaluated bridge remains a child
            // diagnostic of the transformed check but is not claimed as a
            // transfer premise (§§4.3, 17.7).
            NOT_REQUIRED_DIRECT_NATIVE.into()
        }
    };
    let comparator_hash = transformed_run
        .check_report
        .envelope
        .comparator_spec_sha256
        .clone();
    let map_semantics = map_semantics_table(
        base,
        &transformed,
        candidate,
        &run_configuration.enumeration_limits(),
    )
    .map_err(|error| TransformationError::Inapplicable(error.to_string()))?;
    let mut dependency_evidence_ids =
        vec![transformed_run.check_report.envelope.evidence_id.clone()];
    if let Some(id) = base_check_evidence_id {
        dependency_evidence_ids.insert(0, id);
    }
    if harmful_witness != NOT_APPLICABLE {
        dependency_evidence_ids.push(
            witness_evidence_id(&transformed_run.witnesses, &harmful_witness)
                .ok_or_else(|| {
                    TransformationError::Check(format!(
                        "unresolved harmful witness {harmful_witness}"
                    ))
                })?
                .into(),
        );
    }
    dependency_evidence_ids.sort();
    dependency_evidence_ids.dedup();
    let envelope = CommonEnvelope {
        schema: "gluerift.transformation-report/v0.3.1a".into(),
        semantic_contract_version: CONTRACT_VERSION.into(),
        tool_build_sha256: metadata.tool_build_sha256.clone(),
        run_configuration_sha256: canonical_sha256(run_configuration)?,
        evidence_id: format!(
            "{}:transformation:{}",
            request.request_id,
            canonical_sha256(&candidate.transformation_ir)?
        ),
        candidate_sha256: base_hash.clone(),
        types_sha256: canonical_sha256(&(
            &base.source_type,
            &base.target_type,
            &base.carrier_type,
        ))?,
        validation_scope_sha256: canonical_sha256(scope)?,
        endpoint_policy_sha256: canonical_sha256(policy)?,
        validation_request_sha256: canonical_sha256(request)?,
        comparator_spec_sha256: comparator_hash.clone(),
        dependency_evidence_ids,
        status: if classification == TransformationClassification::LawfulHarmful {
            Status::Disproved
        } else if lawful {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
    };
    let report = TransformationReport {
        envelope,
        transformation_family_sha256: family_hash,
        generation_mode: match candidate.generation_mode {
            GenerationMode::Enumerated => "enumerated",
            GenerationMode::DeclaredCandidate => "declared-candidate",
        }
        .into(),
        generation_rule_id: candidate.generation_rule_id.clone(),
        generation_parent_path: candidate.generation_parent_path.clone(),
        generation_ordinal: candidate.generation_ordinal,
        transformation_ir: candidate.transformation_ir.normalize(),
        transformation_sha256: canonical_sha256(&candidate.transformation_ir.normalize())?,
        inverse_ir: candidate.inverse_ir.normalize(),
        inverse_sha256: canonical_sha256(&candidate.inverse_ir.normalize())?,
        inverse_check_status: if inverse_ok {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
        action_domain: action_domain.clone(),
        action_domain_sha256: canonical_sha256(&action_domain)?,
        twist_side: "target".into(),
        twist_construction: "carrier-conjugation".into(),
        comparator_spec_sha256: comparator_hash,
        candidate_context_sha256: base_hash,
        base_check_report_sha256: base_check_hash,
        base_alignment_status: base_alignment,
        base_source_encode_sha256: canonical_sha256(&base.source_encode.normalize())?,
        base_source_decode_sha256: canonical_sha256(&base.source_decode.normalize())?,
        base_target_encode_sha256: canonical_sha256(&base.target_encode.normalize())?,
        base_target_decode_sha256: canonical_sha256(&base.target_decode.normalize())?,
        transformed_context_sha256: transformed_hash,
        transformed_check_report_sha256: transformed_check_hash,
        candidate_binding_status: candidate_binding,
        transformed_source_encode_sha256: canonical_sha256(&transformed.source_encode.normalize())?,
        transformed_source_decode_sha256: canonical_sha256(&transformed.source_decode.normalize())?,
        transformed_target_encode_sha256: canonical_sha256(&transformed.target_encode.normalize())?,
        transformed_target_decode_sha256: canonical_sha256(&transformed.target_decode.normalize())?,
        four_map_construction_status: if construction_ok {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
        four_map_semantics_check_sha256: canonical_sha256(&map_semantics)?,
        well_typed_status: if well_typed {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
        comparator_definedness_status: transformed_run
            .check_report
            .comparison
            .comparator_definedness
            .status,
        requested_law_ids: required_laws,
        roundtrip_statuses,
        lawfulness_status: if lawful {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        },
        classification,
        inapplicability_reasons: reasons,
        selected_property_statuses: selected_properties,
        harmful_witness_sha256: harmful_witness,
        transformed_bridge_report_sha256,
        family_completeness_statement: family.completeness_statement.join("; "),
    };
    Ok(ClassifiedTransformation {
        report,
        transformed_context: transformed,
        transformed_run,
    })
}

fn inverse_check(candidate: &TransformationCandidate, carrier: &Type, domain: &[Value]) -> bool {
    domain.iter().all(|value| {
        let forward = candidate.transformation_ir.eval(value);
        let reverse = candidate.inverse_ir.eval(value);
        forward.as_ref().is_ok_and(|mapped| {
            carrier.contains(mapped) && candidate.inverse_ir.eval(mapped).as_ref() == Ok(value)
        }) && reverse.as_ref().is_ok_and(|mapped| {
            carrier.contains(mapped)
                && candidate.transformation_ir.eval(mapped).as_ref() == Ok(value)
        })
    })
}

fn verify_construction(
    base: &AdapterContext,
    transformed: &AdapterContext,
    candidate: &TransformationCandidate,
    limits: &EnumerationLimits,
) -> bool {
    let base = base.normalized();
    if transformed.source_encode != base.source_encode
        || transformed.source_decode != base.source_decode
    {
        return false;
    }
    let expected = construct_twist(&base, candidate);
    if &expected != transformed {
        return false;
    }
    let Ok(source_domain) = base.source_type.enumerate(limits) else {
        return false;
    };
    let Ok(target_domain) = base.target_type.enumerate(limits) else {
        return false;
    };
    let Ok(carrier_domain) = base.carrier_type.enumerate(limits) else {
        return false;
    };
    // Exhaustive equality on each map's own complete input domain.
    source_domain
        .iter()
        .all(|source| transformed.source_encode.eval(source) == expected.source_encode.eval(source))
        && target_domain.iter().all(|target| {
            transformed.target_encode.eval(target) == expected.target_encode.eval(target)
        })
        && carrier_domain.iter().all(|carrier| {
            transformed.source_decode.eval(carrier) == expected.source_decode.eval(carrier)
                && transformed.target_decode.eval(carrier) == expected.target_decode.eval(carrier)
        })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MapSemanticsRow {
    map_id: String,
    input: Value,
    base_result: Result<Value, crate::adapter_ir::ConversionError>,
    transformed_result: Result<Value, crate::adapter_ir::ConversionError>,
    expected_result: Result<Value, crate::adapter_ir::ConversionError>,
}

fn map_semantics_table(
    base: &AdapterContext,
    transformed: &AdapterContext,
    candidate: &TransformationCandidate,
    limits: &EnumerationLimits,
) -> Result<Vec<MapSemanticsRow>, crate::type_ir::TypeError> {
    let expected = construct_twist(base, candidate);
    let source = base.source_type.enumerate(limits)?;
    let target = base.target_type.enumerate(limits)?;
    let carrier = base.carrier_type.enumerate(limits)?;
    let mut rows = Vec::new();
    for value in &source {
        rows.push(MapSemanticsRow {
            map_id: "source-encode".into(),
            input: value.clone(),
            base_result: base.source_encode.eval(value),
            transformed_result: transformed.source_encode.eval(value),
            expected_result: expected.source_encode.eval(value),
        });
    }
    for value in &target {
        rows.push(MapSemanticsRow {
            map_id: "target-encode".into(),
            input: value.clone(),
            base_result: base.target_encode.eval(value),
            transformed_result: transformed.target_encode.eval(value),
            expected_result: expected.target_encode.eval(value),
        });
    }
    for value in carrier {
        rows.push(MapSemanticsRow {
            map_id: "source-decode".into(),
            input: value.clone(),
            base_result: base.source_decode.eval(&value),
            transformed_result: transformed.source_decode.eval(&value),
            expected_result: expected.source_decode.eval(&value),
        });
        rows.push(MapSemanticsRow {
            map_id: "target-decode".into(),
            input: value.clone(),
            base_result: base.target_decode.eval(&value),
            transformed_result: transformed.target_decode.eval(&value),
            expected_result: expected.target_decode.eval(&value),
        });
    }
    Ok(rows)
}

fn structural_candidates(
    ty: &Type,
    limits: &EnumerationLimits,
    max: usize,
) -> Result<Vec<TransformationCandidate>, TransformationError> {
    let pairs = structural_pairs(ty, Vec::new(), limits, max)?;
    let mut out: Vec<_> = pairs
        .into_iter()
        .enumerate()
        .map(
            |(ordinal, (ir, inverse, rule, path))| TransformationCandidate {
                generation_mode: GenerationMode::Enumerated,
                generation_rule_id: rule,
                generation_parent_path: path,
                generation_ordinal: ordinal,
                transformation_ir: ir.normalize(),
                inverse_ir: inverse.normalize(),
            },
        )
        .collect();
    out.retain(|candidate| candidate.transformation_ir != Adapter::Identity);
    deduplicate_candidates(&mut out)?;
    Ok(out)
}

type StructuralPair = (Adapter, Adapter, String, Vec<String>);

fn structural_pairs(
    ty: &Type,
    path: Vec<String>,
    limits: &EnumerationLimits,
    max: usize,
) -> Result<Vec<StructuralPair>, TransformationError> {
    let _ = limits;
    let mut out = vec![(
        Adapter::Identity,
        Adapter::Identity,
        "identity".into(),
        path.clone(),
    )];
    match ty {
        Type::Sum { variants } => {
            let mut groups: BTreeMap<Type, Vec<usize>> = BTreeMap::new();
            for (index, variant) in variants.iter().enumerate() {
                groups
                    .entry(variant.payload.clone())
                    .or_default()
                    .push(index);
            }
            let mut group_permutations: Vec<Vec<Vec<usize>>> = Vec::new();
            for indices in groups.values() {
                group_permutations.push(permutations(indices, max)?);
            }
            for selection in cartesian(&group_permutations, max)? {
                let mut permutation: Vec<usize> = (0..variants.len()).collect();
                for (indices, permuted) in groups.values().zip(selection) {
                    for (source, target) in indices.iter().zip(permuted) {
                        permutation[*source] = target;
                    }
                }
                if permutation.iter().enumerate().all(|(i, p)| i == *p) {
                    continue;
                }
                let inverse = invert_permutation(&permutation);
                let all_unit = variants.iter().all(|variant| variant.payload == Type::Unit);
                let (forward_ir, inverse_ir) = if all_unit {
                    let forward = variants
                        .iter()
                        .enumerate()
                        .map(|(i, variant)| {
                            (variant.name.clone(), variants[permutation[i]].name.clone())
                        })
                        .collect();
                    let backward = variants
                        .iter()
                        .enumerate()
                        .map(|(i, variant)| {
                            (variant.name.clone(), variants[inverse[i]].name.clone())
                        })
                        .collect();
                    (
                        Adapter::EnumPermutation { mapping: forward },
                        Adapter::EnumPermutation { mapping: backward },
                    )
                } else {
                    (
                        sum_permutation(variants, &permutation),
                        sum_permutation(variants, &inverse),
                    )
                };
                out.push((
                    forward_ir,
                    inverse_ir,
                    "core.enum.payload-compatible".into(),
                    path.clone(),
                ));
            }
            for variant in variants {
                let mut child_path = path.clone();
                child_path.push(variant.name.clone());
                child_path.push("$payload".into());
                for (child, child_inverse, _, origin_path) in
                    structural_pairs(&variant.payload, child_path.clone(), limits, max)?
                        .into_iter()
                        .filter(|(child, _, _, _)| *child != Adapter::Identity)
                {
                    let mut forward = BTreeMap::new();
                    let mut inverse = BTreeMap::new();
                    for item in variants {
                        forward.insert(
                            item.name.clone(),
                            SumVariantMap {
                                target: item.name.clone(),
                                adapter: if item.name == variant.name {
                                    child.clone()
                                } else {
                                    Adapter::Identity
                                },
                            },
                        );
                        inverse.insert(
                            item.name.clone(),
                            SumVariantMap {
                                target: item.name.clone(),
                                adapter: if item.name == variant.name {
                                    child_inverse.clone()
                                } else {
                                    Adapter::Identity
                                },
                            },
                        );
                    }
                    out.push((
                        Adapter::SumMap { variants: forward },
                        Adapter::SumMap { variants: inverse },
                        "core.nested.canonical-path-product".into(),
                        origin_path,
                    ));
                }
            }
        }
        Type::Product { fields } => {
            let mut groups: BTreeMap<Type, Vec<usize>> = BTreeMap::new();
            for (index, field) in fields.iter().enumerate() {
                groups.entry(field.ty.clone()).or_default().push(index);
            }
            let mut group_permutations = Vec::new();
            for indices in groups.values() {
                group_permutations.push(permutations(indices, max)?);
            }
            for selection in cartesian(&group_permutations, max)? {
                let mut permutation: Vec<usize> = (0..fields.len()).collect();
                for (indices, permuted) in groups.values().zip(selection) {
                    for (source, target) in indices.iter().zip(permuted) {
                        permutation[*source] = target;
                    }
                }
                if permutation.iter().enumerate().all(|(i, p)| i == *p) {
                    continue;
                }
                let inverse = invert_permutation(&permutation);
                let forward = fields
                    .iter()
                    .enumerate()
                    .map(|(source, field)| {
                        (fields[permutation[source]].name.clone(), field.name.clone())
                    })
                    .collect();
                let backward = fields
                    .iter()
                    .enumerate()
                    .map(|(source, field)| {
                        (fields[inverse[source]].name.clone(), field.name.clone())
                    })
                    .collect();
                out.push((
                    Adapter::FieldPermutation { mapping: forward },
                    Adapter::FieldPermutation { mapping: backward },
                    "core.product.same-typed-fields".into(),
                    path.clone(),
                ));
            }
            for field in fields {
                let mut child_path = path.clone();
                child_path.push(field.name.clone());
                for (child, child_inverse, _, origin_path) in
                    structural_pairs(&field.ty, child_path.clone(), limits, max)?
                        .into_iter()
                        .filter(|(child, _, _, _)| *child != Adapter::Identity)
                {
                    let forward = fields
                        .iter()
                        .map(|item| {
                            (
                                item.name.clone(),
                                ProductFieldMap {
                                    source: item.name.clone(),
                                    adapter: if item.name == field.name {
                                        child.clone()
                                    } else {
                                        Adapter::Identity
                                    },
                                },
                            )
                        })
                        .collect();
                    let inverse = fields
                        .iter()
                        .map(|item| {
                            (
                                item.name.clone(),
                                ProductFieldMap {
                                    source: item.name.clone(),
                                    adapter: if item.name == field.name {
                                        child_inverse.clone()
                                    } else {
                                        Adapter::Identity
                                    },
                                },
                            )
                        })
                        .collect();
                    out.push((
                        Adapter::ProductMap { fields: forward },
                        Adapter::ProductMap { fields: inverse },
                        "core.nested.canonical-path-product".into(),
                        origin_path,
                    ));
                }
            }
        }
        Type::ObjectResult { ok, err } => {
            if ok == err {
                let swap = Adapter::ResultMap {
                    branch_mapping: BranchMapping::Swap,
                    ok: Box::new(Adapter::Identity),
                    err: Box::new(Adapter::Identity),
                };
                out.push((
                    swap.clone(),
                    swap,
                    "core.result.compatible-branches".into(),
                    path.clone(),
                ));
            }
            for (branch, ty) in [
                (ResultBranch::Ok, ok.as_ref()),
                (ResultBranch::Err, err.as_ref()),
            ] {
                let name = if branch == ResultBranch::Ok {
                    "Ok"
                } else {
                    "Err"
                };
                let mut child_path = path.clone();
                child_path.push(name.into());
                child_path.push("$value".into());
                for (child, child_inverse, _, origin_path) in
                    structural_pairs(ty, child_path.clone(), limits, max)?
                        .into_iter()
                        .filter(|(child, _, _, _)| *child != Adapter::Identity)
                {
                    let forward = Adapter::ResultMap {
                        branch_mapping: BranchMapping::Preserve,
                        ok: Box::new(if branch == ResultBranch::Ok {
                            child.clone()
                        } else {
                            Adapter::Identity
                        }),
                        err: Box::new(if branch == ResultBranch::Err {
                            child.clone()
                        } else {
                            Adapter::Identity
                        }),
                    };
                    let inverse = Adapter::ResultMap {
                        branch_mapping: BranchMapping::Preserve,
                        ok: Box::new(if branch == ResultBranch::Ok {
                            child_inverse.clone()
                        } else {
                            Adapter::Identity
                        }),
                        err: Box::new(if branch == ResultBranch::Err {
                            child_inverse.clone()
                        } else {
                            Adapter::Identity
                        }),
                    };
                    out.push((
                        forward,
                        inverse,
                        "core.nested.canonical-path-product".into(),
                        origin_path,
                    ));
                }
            }
        }
        _ => {}
    }
    if out.len() > max {
        return Err(TransformationError::Limit(max));
    }
    // Compose one structural choice per distinct origin in canonical path
    // order, including ancestor/descendant choices.  This is the finite nested
    // product promised by the family descriptor (for example, a product-field
    // swap together with independent swaps inside its sum-valued fields).
    // Reusing a choice or taking arbitrary same-origin word powers is not a
    // structural choice.  Same-origin enum pairs are admitted separately to
    // retain the contract-fixed normalized T01 composition witness.
    let primitives: Vec<_> = out
        .iter()
        .filter(|(adapter, _, _, _)| *adapter != Adapter::Identity)
        .cloned()
        .collect();
    let mut keyed_primitives = Vec::with_capacity(primitives.len());
    for primitive in primitives {
        keyed_primitives.push((
            primitive.3.clone(),
            canonical_bytes(&primitive.0)?,
            canonical_sha256(&primitive.0)?,
            primitive,
        ));
    }
    keyed_primitives.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let mut seen_primitive_hashes = BTreeSet::new();
    keyed_primitives.retain(|primitive| seen_primitive_hashes.insert(primitive.2.clone()));
    let primitives: Vec<_> = keyed_primitives
        .into_iter()
        .map(|(_, _, _, primitive)| primitive)
        .collect();

    append_distinct_origin_combinations(&primitives, 0, &mut Vec::new(), &mut out, max, &path)?;

    for left_index in 0..primitives.len() {
        for right_index in (left_index + 1)..primitives.len() {
            let left = &primitives[left_index];
            let right = &primitives[right_index];
            let same_origin_enum_pair = left.3 == right.3
                && matches!(left.0, Adapter::EnumPermutation { .. })
                && matches!(right.0, Adapter::EnumPermutation { .. });
            if !same_origin_enum_pair {
                continue;
            }
            let forward = Adapter::Compose {
                first: Box::new(left.0.clone()),
                second: Box::new(right.0.clone()),
            }
            .normalize();
            let inverse = Adapter::Compose {
                first: Box::new(right.1.clone()),
                second: Box::new(left.1.clone()),
            }
            .normalize();
            out.push((
                forward,
                inverse,
                "core.nested.canonical-path-product".into(),
                path.clone(),
            ));
            if out.len() > max {
                return Err(TransformationError::Limit(max));
            }
        }
    }
    deduplicate_structural(out)
}

fn append_distinct_origin_combinations(
    primitives: &[StructuralPair],
    start: usize,
    selected: &mut Vec<usize>,
    output: &mut Vec<StructuralPair>,
    max: usize,
    parent_path: &[String],
) -> Result<(), TransformationError> {
    for index in start..primitives.len() {
        if selected
            .iter()
            .any(|selected_index| primitives[*selected_index].3 == primitives[index].3)
        {
            continue;
        }
        selected.push(index);
        if selected.len() >= 2 {
            let forward = selected
                .iter()
                .fold(Adapter::Identity, |accumulator, item| {
                    Adapter::Compose {
                        first: Box::new(accumulator),
                        second: Box::new(primitives[*item].0.clone()),
                    }
                    .normalize()
                });
            let inverse = selected
                .iter()
                .rev()
                .fold(Adapter::Identity, |accumulator, item| {
                    Adapter::Compose {
                        first: Box::new(accumulator),
                        second: Box::new(primitives[*item].1.clone()),
                    }
                    .normalize()
                });
            output.push((
                forward,
                inverse,
                "core.nested.canonical-path-product".into(),
                parent_path.to_vec(),
            ));
            if output.len() > max {
                return Err(TransformationError::Limit(max));
            }
        }
        append_distinct_origin_combinations(
            primitives,
            index + 1,
            selected,
            output,
            max,
            parent_path,
        )?;
        selected.pop();
    }
    Ok(())
}

fn deduplicate_structural(
    values: Vec<StructuralPair>,
) -> Result<Vec<StructuralPair>, TransformationError> {
    let mut keyed = Vec::with_capacity(values.len());
    for value in values {
        keyed.push((
            canonical_bytes(&value.0)?,
            canonical_sha256(&value.0)?,
            value,
        ));
    }
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    let mut seen_hashes = BTreeSet::new();
    keyed.retain(|value| seen_hashes.insert(value.1.clone()));
    Ok(keyed.into_iter().map(|(_, _, value)| value).collect())
}

fn sum_permutation(variants: &[Variant], permutation: &[usize]) -> Adapter {
    Adapter::SumMap {
        variants: variants
            .iter()
            .enumerate()
            .map(|(source, variant)| {
                (
                    variant.name.clone(),
                    SumVariantMap {
                        target: variants[permutation[source]].name.clone(),
                        adapter: Adapter::Identity,
                    },
                )
            })
            .collect(),
    }
}

fn invert_permutation(permutation: &[usize]) -> Vec<usize> {
    let mut inverse = vec![0; permutation.len()];
    for (source, target) in permutation.iter().enumerate() {
        inverse[*target] = source;
    }
    inverse
}

fn permutations(values: &[usize], max: usize) -> Result<Vec<Vec<usize>>, TransformationError> {
    let mut cardinality = 1usize;
    for factor in 2..=values.len() {
        cardinality = cardinality
            .checked_mul(factor)
            .ok_or(TransformationError::Limit(max))?;
        if cardinality > max {
            return Err(TransformationError::Limit(max));
        }
    }
    fn recurse(prefix: &mut Vec<usize>, remaining: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if remaining.is_empty() {
            out.push(prefix.clone());
            return;
        }
        for index in 0..remaining.len() {
            let value = remaining.remove(index);
            prefix.push(value);
            recurse(prefix, remaining, out);
            prefix.pop();
            remaining.insert(index, value);
        }
    }
    let mut out = Vec::with_capacity(cardinality);
    recurse(&mut Vec::new(), &mut values.to_vec(), &mut out);
    Ok(out)
}

fn cartesian(
    groups: &[Vec<Vec<usize>>],
    max: usize,
) -> Result<Vec<Vec<Vec<usize>>>, TransformationError> {
    let cardinality = groups.iter().try_fold(1usize, |accumulator, group| {
        accumulator.checked_mul(group.len())
    });
    if cardinality.is_none_or(|count| count > max) {
        return Err(TransformationError::Limit(max));
    }
    let mut out = Vec::with_capacity(cardinality.unwrap_or_default());
    out.push(Vec::new());
    for group in groups {
        let next_capacity = out.len().saturating_mul(group.len());
        if next_capacity > max {
            return Err(TransformationError::Limit(max));
        }
        let mut next = Vec::with_capacity(next_capacity);
        for prefix in &out {
            for item in group {
                let mut value = prefix.clone();
                value.push(item.clone());
                next.push(value);
            }
        }
        out = next;
    }
    Ok(out)
}

fn deduplicate_candidates(
    candidates: &mut Vec<TransformationCandidate>,
) -> Result<(), CanonicalError> {
    let mut keyed = Vec::with_capacity(candidates.len());
    for candidate in candidates.drain(..) {
        keyed.push((
            canonical_bytes(&candidate.transformation_ir)?,
            canonical_sha256(&candidate.transformation_ir)?,
            candidate,
        ));
    }
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    let mut seen_hashes = BTreeSet::new();
    keyed.retain(|value| seen_hashes.insert(value.1.clone()));
    candidates.extend(keyed.into_iter().map(|(_, _, candidate)| candidate));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_ir::Field;

    #[test]
    fn enum_swap_is_generated_with_inverse() {
        let ty = Type::Sum {
            variants: vec![
                Variant {
                    name: "A".into(),
                    payload: Type::Unit,
                },
                Variant {
                    name: "B".into(),
                    payload: Type::Unit,
                },
            ],
        };
        let pairs = structural_candidates(&ty, &EnumerationLimits::default(), 100).unwrap();
        assert!(pairs.iter().any(|candidate| matches!(
            candidate.transformation_ir,
            Adapter::EnumPermutation { .. }
        ) && candidate.transformation_ir
            == candidate.inverse_ir));
    }

    #[test]
    fn structural_permutation_limit_is_checked_before_factorial_materialization() {
        let ty = Type::Sum {
            variants: (0..10)
                .map(|index| Variant {
                    name: format!("V{index:02}"),
                    payload: Type::Unit,
                })
                .collect(),
        };
        assert!(matches!(
            structural_candidates(&ty, &EnumerationLimits::default(), 100),
            Err(TransformationError::Limit(100))
        ));
    }

    #[test]
    fn nested_product_sum_family_has_fixed_complete_normalized_count() {
        let choice = Type::Sum {
            variants: vec![
                Variant {
                    name: "Off".into(),
                    payload: Type::Unit,
                },
                Variant {
                    name: "On".into(),
                    payload: Type::Unit,
                },
            ],
        };
        let ty = Type::Product {
            fields: vec![
                crate::type_ir::Field {
                    name: "left".into(),
                    ty: choice.clone(),
                },
                crate::type_ir::Field {
                    name: "right".into(),
                    ty: choice,
                },
            ],
        };
        let candidates = structural_candidates(&ty, &EnumerationLimits::default(), 100)
            .expect("bounded nested structural family");
        // Three primitive choices (field, left payload, right payload) and
        // every nonempty canonical subset: 2^3 - 1.
        assert_eq!(candidates.len(), 7);
        let hashes: BTreeSet<_> = candidates
            .iter()
            .map(|candidate| canonical_sha256(&candidate.transformation_ir).unwrap())
            .collect();
        assert_eq!(hashes.len(), candidates.len());
        assert!(candidates.windows(2).all(|pair| {
            canonical_bytes(&pair[0].transformation_ir).unwrap()
                < canonical_bytes(&pair[1].transformation_ir).unwrap()
        }));
    }

    #[test]
    fn same_typed_fields_generate_swap() {
        let ty = Type::Product {
            fields: vec![
                Field {
                    name: "minimum".into(),
                    ty: Type::Bool,
                },
                Field {
                    name: "maximum".into(),
                    ty: Type::Bool,
                },
            ],
        };
        let pairs = structural_candidates(&ty, &EnumerationLimits::default(), 100).unwrap();
        assert!(pairs.iter().any(|candidate| matches!(
            candidate.transformation_ir,
            Adapter::FieldPermutation { .. }
        ) && candidate.transformation_ir
            != Adapter::Identity));
    }
}
