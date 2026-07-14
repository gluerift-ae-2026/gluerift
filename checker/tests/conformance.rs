use std::collections::BTreeSet;

use gluerift::adapter_ir::{Adapter, AdapterContext, SumVariantMap};
use gluerift::canonical::canonical_sha256;
use gluerift::comparator::evaluate_pair;
use gluerift::comparison::{
    CertificationProfile, EvidenceMetadata, PropertyRequest, RequiredLaws, RunConfiguration,
    ValidationRequest, check,
};
use gluerift::composition::{CompositionRequest, TotalJudgment, compose_and_check};
use gluerift::domain::{ComparatorSpec, DomainSpec, PairDomainSpec, ValidationScope, ValuePair};
use gluerift::observer_ir::{Observation, Observer, PolicyMapEntry};
use gluerift::relation_ir::{
    EndpointPolicy, MatchCoverageMode, ObservationPair, PolicyDimension, Relation,
};
use gluerift::report::{BindingStatus, LawId, Status, TransformationClassification};
use gluerift::transformation::{
    ActionDomainRule, DeclaredCandidateDescriptor, GenerationMode, GeneratorDescriptor,
    InverseRequirementDescriptor, NormalizationDescriptor, TransformationCandidate,
    TransformationFamilyDescriptor, TwistDescriptor, classify_transformation,
};
use gluerift::type_ir::{EnumerationLimits, Type, Value, Variant};

fn sum_type(names: &[&str]) -> Type {
    Type::Sum {
        variants: names
            .iter()
            .map(|name| Variant {
                name: (*name).into(),
                payload: Type::Unit,
            })
            .collect(),
    }
}
fn sum(name: &str) -> Value {
    Value::Sum {
        variant: name.into(),
        payload: Box::new(Value::Unit),
    }
}
fn atom(value: &str) -> Observation {
    Observation::Atom {
        value: value.into(),
    }
}
fn sum_map(entries: &[(&str, &str)]) -> Adapter {
    Adapter::SumMap {
        variants: entries
            .iter()
            .map(|(source, target)| {
                (
                    (*source).into(),
                    SumVariantMap {
                        target: (*target).into(),
                        adapter: Adapter::Identity,
                    },
                )
            })
            .collect(),
    }
}
fn enum_permutation(entries: &[(&str, &str)]) -> Adapter {
    Adapter::EnumPermutation {
        mapping: entries
            .iter()
            .map(|(source, target)| ((*source).into(), (*target).into()))
            .collect(),
    }
}
fn finite_policy(entries: &[(&str, &str)]) -> Observer {
    let mut table: Vec<_> = entries
        .iter()
        .map(|(value, role)| PolicyMapEntry {
            value: sum(value),
            atom: (*role).into(),
        })
        .collect();
    table.sort_by(|a, b| a.value.cmp(&b.value));
    Observer::FinitePolicyMap {
        path: vec![],
        table,
    }
}
fn dimension(
    id: &str,
    source: &[(&str, &str)],
    target: &[(&str, &str)],
    safe: Relation,
    matched: Option<Relation>,
) -> PolicyDimension {
    let source_codomain: Vec<_> = source
        .iter()
        .map(|(_, value)| atom(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let target_codomain: Vec<_> = target
        .iter()
        .map(|(_, value)| atom(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    PolicyDimension {
        id: id.into(),
        source_codomain,
        target_codomain,
        source_observer: finite_policy(source),
        target_observer: finite_policy(target),
        safe_relation: safe,
        match_relation: matched,
    }
}
#[allow(clippy::too_many_arguments)]
fn scope(
    comparator: ComparatorSpec,
    source_domain: DomainSpec,
    target_domain: DomainSpec,
    source_cmp: DomainSpec,
    target_cmp: DomainSpec,
    universe: PairDomainSpec,
    source_carrier: DomainSpec,
    target_carrier: DomainSpec,
    source_full: DomainSpec,
    target_full: DomainSpec,
) -> ValidationScope {
    ValidationScope {
        schema: "gluerift.validation-scope/v0.3.1a".into(),
        source_domain,
        target_domain,
        source_comparison_domain: source_cmp,
        target_comparison_domain: target_cmp,
        comparison_universe: universe,
        source_carrier_domain: source_carrier,
        target_carrier_domain: target_carrier,
        source_full_transport_domain: source_full,
        target_full_transport_domain: target_full,
        comparator,
    }
}
fn config() -> RunConfiguration {
    RunConfiguration::default()
}
fn request(
    scope: &ValidationScope,
    policy: &EndpointPolicy,
    config: &RunConfiguration,
    profile: CertificationProfile,
    properties: Vec<PropertyRequest>,
    family_hash: String,
) -> ValidationRequest {
    ValidationRequest {
        schema: "gluerift.validation-request/v0.3.1a".into(),
        request_id: "conformance".into(),
        profile,
        validation_scope_sha256: canonical_sha256(scope).unwrap(),
        endpoint_policy_sha256: canonical_sha256(policy).unwrap(),
        run_configuration_sha256: canonical_sha256(config).unwrap(),
        required_laws: RequiredLaws::all(),
        required_properties: properties,
        required_bridges: vec![],
        required_transformation_family_sha256: family_hash,
    }
}
fn all() -> DomainSpec {
    DomainSpec::All
}
fn finite(mut values: Vec<Value>) -> DomainSpec {
    values.sort();
    DomainSpec::FiniteSet { values }
}
fn pairs(mut values: Vec<(&str, &str)>) -> PairDomainSpec {
    values.sort();
    PairDomainSpec::FinitePairSet {
        pairs: values
            .into_iter()
            .map(|(source, target)| ValuePair {
                source: sum(source),
                target: sum(target),
            })
            .collect(),
    }
}
fn family() -> TransformationFamilyDescriptor {
    TransformationFamilyDescriptor {
        schema: "gluerift.transformation-family/v0.3.1a".into(),
        semantic_contract_version: "0.3.1a".into(),
        family_id: "core-structural".into(),
        action_domain: ActionDomainRule {
            kind: "all".into(),
            ownership: "derived-from-carrier-type".into(),
            ordering: "canonical-type-value-order".into(),
        },
        enumerated_generators: vec![
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
        ],
        admitted_declared_candidates: vec![
            DeclaredCandidateDescriptor {
                kind: "bounded-complement".into(),
                generation_rule_id: "core.scalar.declared-bounded-complement".into(),
            },
            DeclaredCandidateDescriptor {
                kind: "modular-affine".into(),
                generation_rule_id: "core.scalar.declared-modular-affine".into(),
            },
        ],
        normalization: NormalizationDescriptor {
            path_order: "lexicographic".into(),
            composition_order: "right-to-left-application".into(),
            identity_elimination: true,
            flatten_compose: true,
            duplicate_elimination: "canonical-adapter-sha256".into(),
            generator_ordinal_order: "normalized-ir-lexicographic".into(),
        },
        inverse_requirement: InverseRequirementDescriptor {
            domain: "complete-all-carrier-values".into(),
            left_identity: "exhaustive".into(),
            right_identity: "exhaustive".into(),
            totality: "required".into(),
        },
        twist: TwistDescriptor {
            side: "target".into(),
            construction: "carrier-conjugation".into(),
            source_encode: "base-source-encode".into(),
            source_decode: "base-source-decode".into(),
            target_encode: "compose-base-target-encode-then-transformation".into(),
            target_decode: "compose-inverse-then-base-target-decode".into(),
        },
        completeness_statement: vec![
            "exact_within_core_structural_family".into(),
            "unknown_outside_declared_family".into(),
        ],
        scalar_discovery_completeness_claimed: false,
        general_automorphism_completeness_claimed: false,
    }
}

#[test]
fn v01_separates_carrier_and_native_comparators_with_all_roundtrips() {
    let context = AdapterContext {
        schema: "gluerift.adapter-context/v0.3.1a".into(),
        source_type: sum_type(&["s0", "s1"]),
        target_type: sum_type(&["t0", "t1"]),
        carrier_type: sum_type(&["L0", "L1", "R0", "R1"]),
        source_encode: sum_map(&[("s0", "L0"), ("s1", "L1")]),
        source_decode: sum_map(&[("L0", "s0"), ("L1", "s1"), ("R0", "s0"), ("R1", "s1")]),
        target_encode: sum_map(&[("t0", "R0"), ("t1", "R1")]),
        target_decode: sum_map(&[("L0", "t0"), ("L1", "t1"), ("R0", "t0"), ("R1", "t1")]),
    };
    let safe_relation = Relation::FiniteTable {
        left_codomain: vec![atom("s0"), atom("s1")],
        right_codomain: vec![atom("t0"), atom("t1")],
        allowed_pairs: vec![
            ObservationPair {
                left: atom("s0"),
                right: atom("t1"),
            },
            ObservationPair {
                left: atom("s1"),
                right: atom("t0"),
            },
        ],
    };
    let policy = EndpointPolicy {
        schema: "gluerift.policy/v0.3.1a".into(),
        match_coverage: MatchCoverageMode::None,
        dimensions: vec![dimension(
            "policy",
            &[("s0", "s0"), ("s1", "s1")],
            &[("t0", "t0"), ("t1", "t1")],
            safe_relation,
            None,
        )],
        safe_dimensions: vec!["policy".into()],
        match_dimensions: vec![],
        explicitly_irrelevant_paths: vec![],
    };
    let base_scope = |comparator| {
        scope(
            comparator,
            all(),
            all(),
            all(),
            all(),
            PairDomainSpec::Product {
                source: all(),
                target: all(),
            },
            finite(vec![sum("L0"), sum("L1")]),
            finite(vec![sum("R0"), sum("R1")]),
            all(),
            all(),
        )
    };
    let cfg = config();
    let carrier_scope = base_scope(ComparatorSpec::CarrierExact);
    let mut carrier_request = request(
        &carrier_scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySound,
        vec![PropertyRequest::PolicySoundness],
        "0".repeat(64),
    );
    carrier_request.required_bridges = vec![gluerift::report::BridgeKind::CarrierTarget];
    let carrier = check(
        &context,
        &carrier_scope,
        &policy,
        &carrier_request,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
    )
    .unwrap();
    assert!(
        carrier
            .roundtrip_report
            .laws
            .iter()
            .all(|law| law.status == Status::ProvedExhaustive)
    );
    assert!(carrier.induced_relation.is_empty());
    assert_eq!(
        carrier.check_report.properties.policy_soundness.status,
        Status::ProvedExhaustive
    );
    assert_eq!(
        carrier.check_report.bridges.carrier_target_bridge.status,
        Status::Disproved
    );

    let target_scope = base_scope(ComparatorSpec::TargetNativeExact);
    let target_request = request(
        &target_scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySound,
        vec![PropertyRequest::PolicySoundness],
        "0".repeat(64),
    );
    let target = check(
        &context,
        &target_scope,
        &policy,
        &target_request,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
    )
    .unwrap();
    assert_eq!(
        target.induced_relation,
        [
            ValuePair {
                source: sum("s0"),
                target: sum("t0")
            },
            ValuePair {
                source: sum("s1"),
                target: sum("t1")
            }
        ]
        .into_iter()
        .collect()
    );
    assert_eq!(
        target.check_report.properties.policy_soundness.status,
        Status::Disproved
    );
    assert_eq!(
        target.check_report.bridges.carrier_target_bridge.status,
        Status::Disproved
    );

    let source_scope = base_scope(ComparatorSpec::SourceNativeExact);
    let source_request = request(
        &source_scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySound,
        vec![PropertyRequest::PolicySoundness],
        "0".repeat(64),
    );
    let source = check(
        &context,
        &source_scope,
        &policy,
        &source_request,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
    )
    .unwrap();
    assert_eq!(source.induced_relation, target.induced_relation);
}

#[test]
fn source_native_comparator_truth_table_has_the_contract_direction() {
    let context = AdapterContext {
        schema: "gluerift.adapter-context/v0.3.1a".into(),
        source_type: sum_type(&["s0", "s1"]),
        target_type: sum_type(&["t0", "t1"]),
        carrier_type: sum_type(&["L", "R"]),
        source_encode: sum_map(&[("s0", "L"), ("s1", "R")]),
        source_decode: sum_map(&[("L", "s1"), ("R", "s0")]),
        target_encode: sum_map(&[("t0", "L"), ("t1", "R")]),
        target_decode: sum_map(&[("L", "t0"), ("R", "t1")]),
    };
    let expected: BTreeSet<_> = [("s1", "t0"), ("s0", "t1")]
        .into_iter()
        .map(|(source, target)| ValuePair {
            source: sum(source),
            target: sum(target),
        })
        .collect();
    let actual: BTreeSet<_> = ["s0", "s1"]
        .into_iter()
        .flat_map(|source| {
            ["t0", "t1"].into_iter().map(move |target| ValuePair {
                source: sum(source),
                target: sum(target),
            })
        })
        .filter(|pair| evaluate_pair(&context, ComparatorSpec::SourceNativeExact, pair).equal)
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn a01_generated_decision_swap_is_lawful_harmful_from_aligned_base() {
    let source_ty = sum_type(&["ALLOW", "DENY"]);
    let target_ty = sum_type(&["Blocked", "Permitted"]);
    let carrier_ty = sum_type(&["ALLOW", "DENY"]);
    let base = AdapterContext {
        schema: "gluerift.adapter-context/v0.3.1a".into(),
        source_type: source_ty,
        target_type: target_ty,
        carrier_type: carrier_ty,
        source_encode: Adapter::Identity,
        source_decode: Adapter::Identity,
        target_encode: sum_map(&[("Blocked", "DENY"), ("Permitted", "ALLOW")]),
        target_decode: sum_map(&[("ALLOW", "Permitted"), ("DENY", "Blocked")]),
    };
    let policy = EndpointPolicy {
        schema: "gluerift.policy/v0.3.1a".into(),
        match_coverage: MatchCoverageMode::Nonempty,
        dimensions: vec![dimension(
            "decision-role",
            &[("ALLOW", "allow"), ("DENY", "deny")],
            &[("Blocked", "deny"), ("Permitted", "allow")],
            Relation::Exact,
            Some(Relation::Exact),
        )],
        safe_dimensions: vec!["decision-role".into()],
        match_dimensions: vec!["decision-role".into()],
        explicitly_irrelevant_paths: vec![],
    };
    let scope = scope(
        ComparatorSpec::TargetNativeExact,
        all(),
        all(),
        all(),
        all(),
        PairDomainSpec::Product {
            source: all(),
            target: all(),
        },
        all(),
        all(),
        all(),
        all(),
    );
    let family = family();
    let cfg = config();
    let req = request(
        &scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySoundAdequate,
        vec![
            PropertyRequest::PolicySoundness,
            PropertyRequest::ComparisonAdequacy,
            PropertyRequest::ComparisonPrecision,
            PropertyRequest::FaithfulComparison,
        ],
        canonical_sha256(&family).unwrap(),
    );
    let base_run = check(
        &base,
        &scope,
        &policy,
        &req,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
    )
    .unwrap();
    assert!(base_run.check_report.certification.granted);
    let swap = enum_permutation(&[("ALLOW", "DENY"), ("DENY", "ALLOW")]);
    let candidate = TransformationCandidate {
        generation_mode: GenerationMode::Enumerated,
        generation_rule_id: "core.enum.payload-compatible".into(),
        generation_parent_path: vec![],
        generation_ordinal: 1,
        transformation_ir: swap.clone(),
        inverse_ir: swap,
    };
    let transformed = classify_transformation(
        &base,
        &candidate,
        &family,
        &scope,
        &policy,
        &req,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
        Some(&base_run),
    )
    .unwrap();
    assert_eq!(
        transformed.report.classification,
        TransformationClassification::LawfulHarmful
    );
    assert_eq!(
        transformed.report.base_alignment_status,
        BindingStatus::ProvedExhaustive
    );
    assert_eq!(
        transformed.report.candidate_binding_status,
        BindingStatus::ProvedExhaustive
    );
    assert!(
        transformed
            .transformed_run
            .roundtrip_report
            .laws
            .iter()
            .all(|law| law.status == Status::ProvedExhaustive)
    );
    assert_eq!(
        transformed
            .transformed_run
            .check_report
            .properties
            .policy_soundness
            .status,
        Status::Disproved
    );
    assert_eq!(
        transformed
            .transformed_run
            .check_report
            .properties
            .comparison_adequacy
            .status,
        Status::Disproved
    );
}

#[test]
fn t01_nonclosure_is_policy_only_and_every_candidate_is_lawful() {
    let ty = sum_type(&["a", "b", "c"]);
    let base = AdapterContext {
        schema: "gluerift.adapter-context/v0.3.1a".into(),
        source_type: ty.clone(),
        target_type: ty.clone(),
        carrier_type: ty,
        source_encode: Adapter::Identity,
        source_decode: Adapter::Identity,
        target_encode: Adapter::Identity,
        target_decode: Adapter::Identity,
    };
    let deny = atom("deny");
    let allow = atom("allow");
    let elements = vec![allow.clone(), deny.clone()];
    let mut edges = vec![
        ObservationPair {
            left: deny.clone(),
            right: deny.clone(),
        },
        ObservationPair {
            left: deny,
            right: allow.clone(),
        },
        ObservationPair {
            left: allow.clone(),
            right: allow,
        },
    ];
    edges.sort();
    let tna = Relation::TargetNoAmplification {
        elements,
        preorder_edges: edges,
    };
    let policy = EndpointPolicy {
        schema: "gluerift.policy/v0.3.1a".into(),
        match_coverage: MatchCoverageMode::None,
        dimensions: vec![dimension(
            "policy",
            &[("a", "deny"), ("b", "allow"), ("c", "allow")],
            &[("a", "deny"), ("b", "deny"), ("c", "allow")],
            tna,
            None,
        )],
        safe_dimensions: vec!["policy".into()],
        match_dimensions: vec![],
        explicitly_irrelevant_paths: vec![],
    };
    let scope = scope(
        ComparatorSpec::TargetNativeExact,
        all(),
        all(),
        all(),
        all(),
        PairDomainSpec::Product {
            source: all(),
            target: all(),
        },
        all(),
        all(),
        all(),
        all(),
    );
    let family = family();
    let cfg = config();
    let req = request(
        &scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySound,
        vec![PropertyRequest::PolicySoundness],
        canonical_sha256(&family).unwrap(),
    );
    let s1 = enum_permutation(&[("a", "b"), ("b", "a"), ("c", "c")]);
    let s2 = enum_permutation(&[("a", "a"), ("b", "c"), ("c", "b")]);
    let composite = Adapter::Compose {
        first: Box::new(s2.clone()),
        second: Box::new(s1.clone()),
    }
    .normalize();
    let candidates = [
        (
            s1.clone(),
            s1,
            "core.enum.payload-compatible",
            TransformationClassification::LawfulSafe,
        ),
        (
            s2.clone(),
            s2,
            "core.enum.payload-compatible",
            TransformationClassification::LawfulSafe,
        ),
        (
            composite.clone(),
            Adapter::Compose {
                first: Box::new(enum_permutation(&[("a", "b"), ("b", "a"), ("c", "c")])),
                second: Box::new(enum_permutation(&[("a", "a"), ("b", "c"), ("c", "b")])),
            }
            .normalize(),
            "core.nested.canonical-path-product",
            TransformationClassification::LawfulHarmful,
        ),
    ];
    for (ordinal, (ir, inverse, rule, expected)) in candidates.into_iter().enumerate() {
        let candidate = TransformationCandidate {
            generation_mode: GenerationMode::Enumerated,
            generation_rule_id: rule.into(),
            generation_parent_path: vec![],
            generation_ordinal: ordinal,
            transformation_ir: ir,
            inverse_ir: inverse,
        };
        let classified = classify_transformation(
            &base,
            &candidate,
            &family,
            &scope,
            &policy,
            &req,
            &cfg,
            &EvidenceMetadata::deterministic_default(),
            None,
        )
        .unwrap();
        assert_eq!(classified.report.classification, expected);
        assert_eq!(
            classified.report.lawfulness_status,
            Status::ProvedExhaustive
        );
        assert!(
            classified
                .report
                .roundtrip_statuses
                .values()
                .all(|status| *status == Status::ProvedExhaustive)
        );
    }
}

#[test]
fn t02_sound_but_target_carrier_failure_is_inapplicable() {
    let base = AdapterContext {
        schema: "gluerift.adapter-context/v0.3.1a".into(),
        source_type: sum_type(&["x", "y"]),
        target_type: sum_type(&["a", "b", "c"]),
        carrier_type: sum_type(&["0", "1", "2"]),
        source_encode: sum_map(&[("x", "0"), ("y", "1")]),
        source_decode: sum_map(&[("0", "x"), ("1", "y"), ("2", "x")]),
        target_encode: enum_permutation(&[("a", "0"), ("b", "1"), ("c", "2")]),
        target_decode: sum_map(&[("0", "a"), ("1", "b"), ("2", "a")]),
    };
    let safe = Relation::FiniteTable {
        left_codomain: vec![atom("x"), atom("y")],
        right_codomain: vec![atom("a"), atom("b"), atom("c")],
        allowed_pairs: vec![
            ObservationPair {
                left: atom("x"),
                right: atom("a"),
            },
            ObservationPair {
                left: atom("y"),
                right: atom("b"),
            },
        ],
    };
    let policy = EndpointPolicy {
        schema: "gluerift.policy/v0.3.1a".into(),
        match_coverage: MatchCoverageMode::None,
        dimensions: vec![dimension(
            "safe",
            &[("x", "x"), ("y", "y")],
            &[("a", "a"), ("b", "b"), ("c", "c")],
            safe,
            None,
        )],
        safe_dimensions: vec!["safe".into()],
        match_dimensions: vec![],
        explicitly_irrelevant_paths: vec![],
    };
    let scope = scope(
        ComparatorSpec::TargetNativeExact,
        all(),
        finite(vec![sum("a"), sum("b")]),
        all(),
        finite(vec![sum("a"), sum("b")]),
        pairs(vec![("x", "a"), ("x", "b"), ("y", "b")]),
        finite(vec![sum("0"), sum("1")]),
        finite(vec![sum("0"), sum("1")]),
        all(),
        finite(vec![sum("a"), sum("b")]),
    );
    let family = family();
    let cfg = config();
    let req = request(
        &scope,
        &policy,
        &cfg,
        CertificationProfile::PolicySound,
        vec![PropertyRequest::PolicySoundness],
        canonical_sha256(&family).unwrap(),
    );
    let swap = enum_permutation(&[("0", "2"), ("1", "1"), ("2", "0")]);
    let candidate = TransformationCandidate {
        generation_mode: GenerationMode::Enumerated,
        generation_rule_id: "core.enum.payload-compatible".into(),
        generation_parent_path: vec![],
        generation_ordinal: 0,
        transformation_ir: swap.clone(),
        inverse_ir: swap,
    };
    let classified = classify_transformation(
        &base,
        &candidate,
        &family,
        &scope,
        &policy,
        &req,
        &cfg,
        &EvidenceMetadata::deterministic_default(),
        None,
    )
    .unwrap();
    assert_eq!(
        classified
            .transformed_run
            .check_report
            .properties
            .policy_soundness
            .status,
        Status::ProvedExhaustive
    );
    assert_eq!(
        classified.report.classification,
        TransformationClassification::LawBreakingOrInapplicable
    );
    assert_eq!(
        classified.report.roundtrip_statuses[&LawId::TargetCarrier],
        Status::Disproved
    );
    assert_eq!(
        classified.report.inapplicability_reasons,
        vec!["required-law-disproved"]
    );
}

#[test]
fn total_composition_checks_intermediate_observer_bridge() {
    let ty = Type::Bool;
    let observer = Observer::FinitePolicyMap {
        path: vec![],
        table: vec![
            PolicyMapEntry {
                value: Value::Bool { value: false },
                atom: "false".into(),
            },
            PolicyMapEntry {
                value: Value::Bool { value: true },
                atom: "true".into(),
            },
        ],
    };
    let judgment = TotalJudgment {
        input_type: ty.clone(),
        output_type: ty.clone(),
        input_domain: all(),
        output_domain: all(),
        input_observer: observer.clone(),
        output_observer: observer,
        adapter: Adapter::Identity,
        relation: Relation::Exact,
    };
    let request = CompositionRequest {
        first: judgment.clone(),
        second: judgment,
        composed_relation: Relation::Exact,
    };
    let envelope = gluerift::report::CommonEnvelope {
        schema: "gluerift.derivation-report/v0.3.1a".into(),
        semantic_contract_version: "0.3.1a".into(),
        tool_build_sha256: "0".repeat(64),
        run_configuration_sha256: "0".repeat(64),
        evidence_id: "C01".into(),
        candidate_sha256: "0".repeat(64),
        types_sha256: "0".repeat(64),
        validation_scope_sha256: "not-applicable".into(),
        endpoint_policy_sha256: "0".repeat(64),
        validation_request_sha256: "0".repeat(64),
        comparator_spec_sha256: "not-applicable".into(),
        dependency_evidence_ids: vec![],
        status: Status::NotRequested,
    };
    let report = compose_and_check(&request, &EnumerationLimits::default(), envelope).unwrap();
    assert!(
        report
            .relation_bridge
            .starts_with("proved-exhaustive:exact-intermediate-observer:")
    );
}

#[test]
fn authored_enumerated_fixture_candidates_resolve_to_generated_family_members() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("checker has workspace parent");
    let family: TransformationFamilyDescriptor = serde_json::from_slice(
        &std::fs::read(root.join("spec/transformation-families/core-structural-v0.3.1a.json"))
            .expect("family bytes"),
    )
    .expect("family JSON");
    let config: RunConfiguration = serde_json::from_slice(
        &std::fs::read(root.join("spec/run-config/core-v0.3.1a.json"))
            .expect("configuration bytes"),
    )
    .expect("configuration JSON");
    for fixture in [
        "fixtures/attacks/A01",
        "fixtures/attacks/A02",
        "fixtures/attacks/A05",
        "fixtures/transformations/T01/sigma1",
        "fixtures/transformations/T01/sigma2",
        "fixtures/transformations/T01/sigma1-compose-sigma2",
        "fixtures/transformations/T02/sigma",
    ] {
        let context_path = if fixture.starts_with("fixtures/transformations/T01/") {
            "fixtures/transformations/T01/context.json".to_owned()
        } else if fixture == "fixtures/transformations/T02/sigma" {
            "fixtures/transformations/T02/context.json".to_owned()
        } else {
            format!("{fixture}/context.json")
        };
        let context: AdapterContext =
            serde_json::from_slice(&std::fs::read(root.join(context_path)).expect("context bytes"))
                .expect("context JSON");
        let candidate: TransformationCandidate = serde_json::from_slice(
            &std::fs::read(root.join(fixture).join("transformation.json"))
                .expect("candidate bytes"),
        )
        .expect("candidate JSON");
        family
            .resolve_candidate(&candidate, &context.carrier_type, &config)
            .unwrap_or_else(|error| panic!("{fixture}: {error}"));
    }
}

#[test]
fn invalid_tna_request_is_never_certificate_eligible() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("checker has workspace parent");
    let directory = root.join("fixtures/benign/H04/tna");
    let context: AdapterContext = serde_json::from_slice(
        &std::fs::read(directory.join("context.json")).expect("context bytes"),
    )
    .expect("context JSON");
    let scope: ValidationScope =
        serde_json::from_slice(&std::fs::read(directory.join("scope.json")).expect("scope bytes"))
            .expect("scope JSON");
    let policy: EndpointPolicy = serde_json::from_slice(
        &std::fs::read(directory.join("policy.json")).expect("policy bytes"),
    )
    .expect("policy JSON");
    let mut request: ValidationRequest = serde_json::from_slice(
        &std::fs::read(directory.join("request.json")).expect("request bytes"),
    )
    .expect("request JSON");
    let tna = request
        .required_properties
        .iter_mut()
        .find_map(|property| match property {
            PropertyRequest::TargetNonAmplification { dimension_ids } => Some(dimension_ids),
            _ => None,
        })
        .expect("H04.tna requests TNA");
    *tna = vec!["missing-policy-dimension".into()];

    let checked = check(
        &context,
        &scope,
        &policy,
        &request,
        &RunConfiguration::default(),
        &EvidenceMetadata::deterministic_default(),
    )
    .expect("invalid TNA is a typed report result");
    assert_eq!(
        checked
            .check_report
            .properties
            .target_non_amplification
            .aggregate_status,
        Status::Invalid
    );
    assert!(!checked.check_report.certification.eligible);
    assert!(!checked.check_report.certification.granted);
}

#[test]
fn transformation_classification_checks_soundness_even_when_ps_is_not_requested() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("checker has workspace parent");
    let directory = root.join("fixtures/transformations/T01/sigma1-compose-sigma2");
    let read = |name: &str| std::fs::read(directory.join(name)).expect("fixture bytes");
    let base: AdapterContext = serde_json::from_slice(
        &std::fs::read(root.join("fixtures/transformations/T01/context.json"))
            .expect("base context bytes"),
    )
    .expect("base context JSON");
    let scope: ValidationScope = serde_json::from_slice(&read("scope.json")).expect("scope JSON");
    let policy: EndpointPolicy = serde_json::from_slice(&read("policy.json")).expect("policy JSON");
    let mut request: ValidationRequest =
        serde_json::from_slice(&read("request.json")).expect("request JSON");
    request.profile = CertificationProfile::Diagnostic;
    request.required_properties.clear();
    let candidate: TransformationCandidate =
        serde_json::from_slice(&read("transformation.json")).expect("transformation JSON");
    let family: TransformationFamilyDescriptor = serde_json::from_slice(
        &std::fs::read(root.join("spec/transformation-families/core-structural-v0.3.1a.json"))
            .expect("family bytes"),
    )
    .expect("family JSON");

    let classified = classify_transformation(
        &base,
        &candidate,
        &family,
        &scope,
        &policy,
        &request,
        &RunConfiguration::default(),
        &EvidenceMetadata::deterministic_default(),
        None,
    )
    .expect("diagnostic transformation classification");
    assert_eq!(
        classified.report.classification,
        TransformationClassification::LawfulHarmful
    );
    assert_ne!(classified.report.harmful_witness_sha256, "not-applicable");
    assert!(
        classified
            .transformed_run
            .witnesses
            .contains_key(&classified.report.harmful_witness_sha256)
    );
}

#[test]
fn static_adapter_typing_rejects_extensional_identity_and_bad_permutations() {
    assert!(
        Adapter::Identity
            .type_check(&sum_type(&["a"]), &sum_type(&["b"]))
            .is_err()
    );
    assert!(
        enum_permutation(&[("a", "x"), ("b", "x")])
            .type_check(&sum_type(&["a", "b"]), &sum_type(&["x", "y"]))
            .is_err()
    );
    assert!(
        Adapter::BoundedComplement { min: 0, max: 9 }
            .type_check(
                &Type::BoundedInt { min: 0, max: 10 },
                &Type::BoundedInt { min: 0, max: 10 }
            )
            .is_err()
    );
    assert!(
        Adapter::ModularAffine {
            width: 4,
            scale: 3,
            offset: 1
        }
        .type_check(&Type::BitVec { width: 3 }, &Type::BitVec { width: 3 })
        .is_err()
    );
}
