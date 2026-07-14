use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adapter_ir::{Adapter, validate_total};
use crate::canonical::{CanonicalError, canonical_sha256};
use crate::domain::DomainSpec;
use crate::observer_ir::{Observer, ObserverError};
use crate::relation_ir::Relation;
use crate::report::{CommonEnvelope, DerivationReport, Status};
use crate::type_ir::{EnumerationLimits, Type, TypeError, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TotalJudgment {
    pub input_type: Type,
    pub output_type: Type,
    pub input_domain: DomainSpec,
    pub output_domain: DomainSpec,
    pub input_observer: Observer,
    pub output_observer: Observer,
    pub adapter: Adapter,
    pub relation: Relation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionRequest {
    pub first: TotalJudgment,
    pub second: TotalJudgment,
    pub composed_relation: Relation,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error(transparent)]
    Observer(#[from] ObserverError),
    #[error("adapter is not total or typed: {0}")]
    Adapter(String),
    #[error("unsupported Core composition relation")]
    UnsupportedRelation,
    #[error("first output type differs from second input type")]
    IntermediateTypeMismatch,
    #[error("intermediate result is outside second checked domain: {0:?}")]
    IntermediateDomain(Value),
    #[error("judgment relation fails for input {0:?}")]
    RelationFailure(Value),
    #[error("composed relation does not contain the relational composition")]
    RelationBridgeFailure,
}

pub fn check_judgment(
    judgment: &TotalJudgment,
    limits: &EnumerationLimits,
) -> Result<Vec<(Value, Value)>, CompositionError> {
    judgment
        .relation
        .validate()
        .map_err(|_| CompositionError::UnsupportedRelation)?;
    validate_total(
        "composition_adapter",
        &judgment.adapter,
        &judgment.input_type,
        &judgment.output_type,
        limits,
    )
    .map_err(|error| CompositionError::Adapter(error.to_string()))?;
    let input = judgment
        .input_domain
        .resolve(&judgment.input_type, limits, "composition.input")
        .map_err(|error| CompositionError::Adapter(error.to_string()))?;
    let output = judgment
        .output_domain
        .resolve(&judgment.output_type, limits, "composition.output")
        .map_err(|error| CompositionError::Adapter(error.to_string()))?;
    judgment.input_observer.validate_on_domain(&input)?;
    judgment.output_observer.validate_on_domain(&output)?;
    let output_set: std::collections::BTreeSet<_> = output.into_iter().collect();
    let mut pairs = Vec::new();
    for value in input {
        let mapped = judgment
            .adapter
            .eval(&value)
            .map_err(|error| CompositionError::Adapter(error.to_string()))?;
        if !output_set.contains(&mapped) {
            return Err(CompositionError::IntermediateDomain(mapped));
        }
        let left = judgment.input_observer.evaluate(&value)?;
        let right = judgment.output_observer.evaluate(&mapped)?;
        if !judgment.relation.allows(&left, &right) {
            return Err(CompositionError::RelationFailure(value));
        }
        pairs.push((value, mapped));
    }
    Ok(pairs)
}

pub fn compose_and_check(
    request: &CompositionRequest,
    limits: &EnumerationLimits,
    mut envelope: CommonEnvelope,
) -> Result<DerivationReport, CompositionError> {
    if request.first.output_type != request.second.input_type {
        return Err(CompositionError::IntermediateTypeMismatch);
    }
    let relation_kind = match (
        &request.first.relation,
        &request.second.relation,
        &request.composed_relation,
    ) {
        (Relation::Exact, Relation::Exact, Relation::Exact) => "exact",
        (
            first @ Relation::TargetNoAmplification { .. },
            second @ Relation::TargetNoAmplification { .. },
            composed @ Relation::TargetNoAmplification { .. },
        ) if first == second && second == composed => "target-no-amplification",
        _ => return Err(CompositionError::UnsupportedRelation),
    };
    let first_pairs = check_judgment(&request.first, limits)?;
    let second_pairs = check_judgment(&request.second, limits)?;
    let second_inputs: std::collections::BTreeSet<_> = second_pairs
        .iter()
        .map(|(input, _)| input.clone())
        .collect();
    for (_, intermediate) in &first_pairs {
        if !second_inputs.contains(intermediate) {
            return Err(CompositionError::IntermediateDomain(intermediate.clone()));
        }
    }
    let intermediate_values = request
        .first
        .output_domain
        .resolve(
            &request.first.output_type,
            limits,
            "composition.intermediate",
        )
        .map_err(|error| CompositionError::Adapter(error.to_string()))?;
    let mut bridge_rows = Vec::new();
    for value in &intermediate_values {
        let first_observation = request.first.output_observer.evaluate(value)?;
        let second_observation = request.second.input_observer.evaluate(value)?;
        if first_observation != second_observation {
            return Err(CompositionError::RelationBridgeFailure);
        }
        bridge_rows.push((value.clone(), first_observation, second_observation));
    }
    let bridge_hash = canonical_sha256(&bridge_rows)?;
    let composed = Adapter::Compose {
        first: Box::new(request.first.adapter.clone()),
        second: Box::new(request.second.adapter.clone()),
    };
    let composed_judgment = TotalJudgment {
        input_type: request.first.input_type.clone(),
        output_type: request.second.output_type.clone(),
        input_domain: request.first.input_domain.clone(),
        output_domain: request.second.output_domain.clone(),
        input_observer: request.first.input_observer.clone(),
        output_observer: request.second.output_observer.clone(),
        adapter: composed,
        relation: request.composed_relation.clone(),
    };
    let crosscheck = check_judgment(&composed_judgment, limits)?;
    envelope.status = Status::ProvedExhaustive;
    Ok(DerivationReport {
        envelope,
        judgment_kind: "total-success".into(),
        relation_kind: relation_kind.into(),
        adapter_path: "$".into(),
        observer_paths: request
            .first
            .input_observer
            .observed_paths()
            .into_iter()
            .chain(request.second.output_observer.observed_paths())
            .collect(),
        input_domain_sha256: canonical_sha256(&request.first.input_domain)?,
        output_domain_sha256: canonical_sha256(&request.second.output_domain)?,
        children: vec![
            canonical_sha256(&first_pairs)?,
            canonical_sha256(&second_pairs)?,
        ],
        relation_bridge: format!(
            "proved-exhaustive:{relation_kind}-intermediate-observer:{bridge_hash}"
        ),
        exhaustive_crosscheck_sha256: canonical_sha256(&crosscheck)?,
    })
}
