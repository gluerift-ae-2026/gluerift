use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::adapter_ir::{AdapterContext, ConversionError};
use crate::domain::{ComparatorSpec, ResolvedScope, ValuePair};
use crate::report::Status;
use crate::type_ir::Value;
use crate::witness::ComparatorEvidence;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairEvaluation {
    pub pair: ValuePair,
    pub equal: bool,
    pub evidence: ComparatorEvidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InducedRelation {
    pub pairs: BTreeSet<ValuePair>,
    pub evaluations: BTreeMap<ValuePair, PairEvaluation>,
}

pub type DefinednessFailure = (Value, Option<Value>, ConversionError);
pub type DefinednessEvaluation = (Status, usize, Option<DefinednessFailure>);

pub fn evaluate_pair(
    context: &AdapterContext,
    comparator: ComparatorSpec,
    pair: &ValuePair,
) -> PairEvaluation {
    match comparator {
        ComparatorSpec::CarrierExact => {
            let source_encoding = context.source_encode.eval(&pair.source);
            let target_encoding = context.target_encode.eval(&pair.target);
            let common_carrier = match (&source_encoding, &target_encoding) {
                (Ok(source), Ok(target)) if source == target => Some(source.clone()),
                _ => None,
            };
            PairEvaluation {
                pair: pair.clone(),
                equal: common_carrier.is_some(),
                evidence: ComparatorEvidence::CarrierExact {
                    source_encoding,
                    target_encoding,
                    common_carrier: crate::report::EvidenceValue::from_option(common_carrier),
                },
            }
        }
        ComparatorSpec::TargetNativeExact => {
            let source_encoding = context.source_encode.eval(&pair.source);
            let target_decode_result = source_encoding
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|carrier| context.target_decode.eval(carrier));
            PairEvaluation {
                pair: pair.clone(),
                equal: target_decode_result
                    .as_ref()
                    .is_ok_and(|target| target == &pair.target),
                evidence: ComparatorEvidence::TargetNativeExact {
                    source_encoding,
                    target_decode_result,
                    compared_target_value: pair.target.clone(),
                },
            }
        }
        ComparatorSpec::SourceNativeExact => {
            let target_encoding = context.target_encode.eval(&pair.target);
            let source_decode_result = target_encoding
                .as_ref()
                .map_err(Clone::clone)
                .and_then(|carrier| context.source_decode.eval(carrier));
            PairEvaluation {
                pair: pair.clone(),
                equal: source_decode_result
                    .as_ref()
                    .is_ok_and(|source| source == &pair.source),
                evidence: ComparatorEvidence::SourceNativeExact {
                    target_encoding,
                    source_decode_result,
                    compared_source_value: pair.source.clone(),
                },
            }
        }
    }
}

pub fn induced_relation(context: &AdapterContext, scope: &ResolvedScope) -> InducedRelation {
    let mut pairs = BTreeSet::new();
    let mut evaluations = BTreeMap::new();
    for pair in &scope.comparison_universe {
        let evaluation = evaluate_pair(context, scope.comparator, pair);
        if evaluation.equal {
            pairs.insert(pair.clone());
        }
        evaluations.insert(pair.clone(), evaluation);
    }
    InducedRelation { pairs, evaluations }
}

pub fn comparator_definedness(
    context: &AdapterContext,
    scope: &ResolvedScope,
) -> DefinednessEvaluation {
    let sources: BTreeSet<_> = scope
        .comparison_universe
        .iter()
        .map(|pair| pair.source.clone())
        .collect();
    let targets: BTreeSet<_> = scope
        .comparison_universe
        .iter()
        .map(|pair| pair.target.clone())
        .collect();
    let mut checked = 0;
    match scope.comparator {
        ComparatorSpec::CarrierExact => {
            for source in &sources {
                checked += 1;
                if let Err(error) = context.source_encode.eval(source) {
                    return (
                        Status::Disproved,
                        checked,
                        Some((source.clone(), None, error)),
                    );
                }
            }
            for target in &targets {
                checked += 1;
                if let Err(error) = context.target_encode.eval(target) {
                    return (
                        Status::Disproved,
                        checked,
                        Some((target.clone(), None, error)),
                    );
                }
            }
        }
        ComparatorSpec::TargetNativeExact => {
            for source in &sources {
                checked += 1;
                let carrier = match context.source_encode.eval(source) {
                    Ok(value) => value,
                    Err(error) => {
                        return (
                            Status::Disproved,
                            checked,
                            Some((source.clone(), None, error)),
                        );
                    }
                };
                if let Err(error) = context.target_decode.eval(&carrier) {
                    return (
                        Status::Disproved,
                        checked,
                        Some((source.clone(), Some(carrier), error)),
                    );
                }
            }
        }
        ComparatorSpec::SourceNativeExact => {
            for target in &targets {
                checked += 1;
                let carrier = match context.target_encode.eval(target) {
                    Ok(value) => value,
                    Err(error) => {
                        return (
                            Status::Disproved,
                            checked,
                            Some((target.clone(), None, error)),
                        );
                    }
                };
                if let Err(error) = context.source_decode.eval(&carrier) {
                    return (
                        Status::Disproved,
                        checked,
                        Some((target.clone(), Some(carrier), error)),
                    );
                }
            }
        }
    }
    (Status::ProvedExhaustive, checked, None)
}
