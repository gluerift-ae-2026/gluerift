use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::{ResolvedScope, ValuePair};
use crate::observer_ir::{Observation, Observer, ObserverError, ValuePath};
use crate::type_ir::{Type, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Relation {
    Exact,
    TargetNoAmplification {
        elements: Vec<Observation>,
        preorder_edges: Vec<ObservationPair>,
    },
    FiniteTable {
        left_codomain: Vec<Observation>,
        right_codomain: Vec<Observation>,
        allowed_pairs: Vec<ObservationPair>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationPair {
    pub left: Observation,
    pub right: Observation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointPolicy {
    pub schema: String,
    pub match_coverage: MatchCoverageMode,
    pub dimensions: Vec<PolicyDimension>,
    pub safe_dimensions: Vec<String>,
    pub match_dimensions: Vec<String>,
    #[serde(default)]
    pub explicitly_irrelevant_paths: Vec<IrrelevantPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDimension {
    pub id: String,
    pub source_codomain: Vec<Observation>,
    pub target_codomain: Vec<Observation>,
    pub source_observer: Observer,
    pub target_observer: Observer,
    pub safe_relation: Relation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_relation: Option<Relation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchCoverageMode {
    None,
    Nonempty,
    SourceTotal,
    TargetTotal,
    BidirectionalTotal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Endpoint {
    Source,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IrrelevanceAppliesTo {
    Safety,
    Matching,
    Both,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IrrelevantPath {
    pub endpoint: Endpoint,
    pub path: ValuePath,
    pub applies_to: IrrelevanceAppliesTo,
    pub justification: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructedPolicy {
    pub safe: BTreeSet<ValuePair>,
    pub matched: BTreeSet<ValuePair>,
    pub observations: BTreeMap<(String, ValuePair), (Observation, Observation)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum PolicyError {
    #[error("policy schema must be gluerift.policy/v0.3.1a")]
    WrongSchema,
    #[error("duplicate or noncanonical dimension id `{0}`")]
    NonCanonicalDimension(String),
    #[error("dimension reference list `{list}` must be duplicate-free and sorted")]
    NonCanonicalReferences { list: &'static str },
    #[error("unknown policy dimension `{0}`")]
    UnknownDimension(String),
    #[error("match dimension `{0}` has no match_relation")]
    MissingMatchRelation(String),
    #[error("observer evaluation in dimension `{dimension}` failed: {source}")]
    Observer {
        dimension: String,
        #[source]
        source: ObserverError,
    },
    #[error(
        "observation in dimension `{dimension}` lies outside declared {endpoint} codomain: {observation:?}"
    )]
    ObservationOutsideCodomain {
        dimension: String,
        endpoint: &'static str,
        observation: Observation,
    },
    #[error("relation in dimension `{dimension}` is invalid: {reason}")]
    InvalidRelation { dimension: String, reason: String },
    #[error("Match is not a subset of Safe; first pair is {0:?}")]
    MatchNotSafe(ValuePair),
    #[error("explicit irrelevance justification must be nonempty")]
    EmptyIrrelevanceJustification,
    #[error("explicit irrelevance path is not reachable on {endpoint:?}: {path:?}")]
    InvalidIrrelevancePath { endpoint: Endpoint, path: ValuePath },
}

impl Relation {
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Relation::Exact => Ok(()),
            Relation::TargetNoAmplification {
                elements,
                preorder_edges,
            } => {
                ensure_sorted_unique(elements, "preorder elements")?;
                ensure_sorted_unique(preorder_edges, "preorder edges")?;
                let element_set: BTreeSet<_> = elements.iter().cloned().collect();
                if preorder_edges.iter().any(|edge| {
                    !element_set.contains(&edge.left) || !element_set.contains(&edge.right)
                }) {
                    return Err("preorder edge endpoint is outside elements".into());
                }
                let edge_set: BTreeSet<_> = preorder_edges.iter().cloned().collect();
                for element in elements {
                    if !edge_set.contains(&ObservationPair {
                        left: element.clone(),
                        right: element.clone(),
                    }) {
                        return Err("preorder is not reflexive".into());
                    }
                }
                for xy in preorder_edges {
                    for yz in preorder_edges.iter().filter(|edge| edge.left == xy.right) {
                        if !edge_set.contains(&ObservationPair {
                            left: xy.left.clone(),
                            right: yz.right.clone(),
                        }) {
                            return Err("preorder is not transitive".into());
                        }
                    }
                }
                Ok(())
            }
            Relation::FiniteTable {
                left_codomain,
                right_codomain,
                allowed_pairs,
            } => {
                ensure_sorted_unique(left_codomain, "left codomain")?;
                ensure_sorted_unique(right_codomain, "right codomain")?;
                ensure_sorted_unique(allowed_pairs, "allowed pairs")?;
                let left: BTreeSet<_> = left_codomain.iter().cloned().collect();
                let right: BTreeSet<_> = right_codomain.iter().cloned().collect();
                if allowed_pairs
                    .iter()
                    .any(|pair| !left.contains(&pair.left) || !right.contains(&pair.right))
                {
                    return Err("finite-table pair lies outside its codomain".into());
                }
                Ok(())
            }
        }
    }

    pub fn allows(&self, source: &Observation, target: &Observation) -> bool {
        match self {
            Relation::Exact => source == target,
            Relation::TargetNoAmplification { preorder_edges, .. } => {
                // Target policy must be <= source policy.
                preorder_edges
                    .binary_search(&ObservationPair {
                        left: target.clone(),
                        right: source.clone(),
                    })
                    .is_ok()
            }
            Relation::FiniteTable { allowed_pairs, .. } => allowed_pairs
                .binary_search(&ObservationPair {
                    left: source.clone(),
                    right: target.clone(),
                })
                .is_ok(),
        }
    }
}

impl EndpointPolicy {
    pub fn construct(
        &self,
        scope: &ResolvedScope,
        source_type: &Type,
        target_type: &Type,
        source_observer_domain: &[Value],
        target_observer_domain: &[Value],
    ) -> Result<ConstructedPolicy, PolicyError> {
        if self.schema != "gluerift.policy/v0.3.1a" {
            return Err(PolicyError::WrongSchema);
        }
        if !self
            .dimensions
            .windows(2)
            .all(|pair| pair[0].id < pair[1].id)
        {
            return Err(PolicyError::NonCanonicalDimension(
                self.dimensions
                    .windows(2)
                    .find_map(|p| (p[0].id >= p[1].id).then(|| p[1].id.clone()))
                    .unwrap_or_default(),
            ));
        }
        if !strict_sorted(&self.safe_dimensions) {
            return Err(PolicyError::NonCanonicalReferences {
                list: "safe_dimensions",
            });
        }
        if !strict_sorted(&self.match_dimensions) {
            return Err(PolicyError::NonCanonicalReferences {
                list: "match_dimensions",
            });
        }
        if self
            .explicitly_irrelevant_paths
            .iter()
            .any(|item| item.justification.trim().is_empty())
        {
            return Err(PolicyError::EmptyIrrelevanceJustification);
        }
        if !strict_sorted(&self.explicitly_irrelevant_paths) {
            return Err(PolicyError::InvalidRelation {
                dimension: "policy".into(),
                reason: "explicit irrelevance paths must be duplicate-free and canonically sorted"
                    .into(),
            });
        }
        let source_paths = source_type.reachable_paths();
        let target_paths = target_type.reachable_paths();
        for item in &self.explicitly_irrelevant_paths {
            let reachable = match item.endpoint {
                Endpoint::Source => &source_paths,
                Endpoint::Target => &target_paths,
            };
            if !item.path.is_empty()
                && !reachable.iter().any(|path| {
                    item.path.len() <= path.len()
                        && item
                            .path
                            .iter()
                            .zip(path)
                            .all(|(left, right)| left == right)
                })
            {
                return Err(PolicyError::InvalidIrrelevancePath {
                    endpoint: item.endpoint,
                    path: item.path.clone(),
                });
            }
        }
        let dimensions: BTreeMap<_, _> = self
            .dimensions
            .iter()
            .map(|dimension| (dimension.id.as_str(), dimension))
            .collect();
        for id in self
            .safe_dimensions
            .iter()
            .chain(self.match_dimensions.iter())
        {
            if !dimensions.contains_key(id.as_str()) {
                return Err(PolicyError::UnknownDimension(id.clone()));
            }
        }
        let active_dimensions: BTreeSet<_> = self
            .safe_dimensions
            .iter()
            .chain(self.match_dimensions.iter())
            .map(String::as_str)
            .collect();
        for dimension in &self.dimensions {
            if !strict_sorted(&dimension.source_codomain)
                || !strict_sorted(&dimension.target_codomain)
            {
                return Err(PolicyError::InvalidRelation {
                    dimension: dimension.id.clone(),
                    reason: "observer codomains must be duplicate-free and sorted".into(),
                });
            }
            dimension
                .safe_relation
                .validate()
                .map_err(|reason| PolicyError::InvalidRelation {
                    dimension: dimension.id.clone(),
                    reason,
                })?;
            if let Some(relation) = &dimension.match_relation {
                relation
                    .validate()
                    .map_err(|reason| PolicyError::InvalidRelation {
                        dimension: dimension.id.clone(),
                        reason,
                    })?;
            }
            validate_relation_codomains(dimension, &dimension.safe_relation)?;
            if let Some(relation) = &dimension.match_relation {
                validate_relation_codomains(dimension, relation)?;
            }
            if !active_dimensions.contains(dimension.id.as_str()) {
                continue;
            }
            dimension
                .source_observer
                .validate_on_domain(source_observer_domain)
                .map_err(|source| PolicyError::Observer {
                    dimension: dimension.id.clone(),
                    source,
                })?;
            dimension
                .target_observer
                .validate_on_domain(target_observer_domain)
                .map_err(|source| PolicyError::Observer {
                    dimension: dimension.id.clone(),
                    source,
                })?;
            for value in &scope.source_comparison_domain {
                let observation = dimension
                    .source_observer
                    .evaluate(value)
                    .map_err(|source| PolicyError::Observer {
                        dimension: dimension.id.clone(),
                        source,
                    })?;
                if dimension
                    .source_codomain
                    .binary_search(&observation)
                    .is_err()
                {
                    return Err(PolicyError::ObservationOutsideCodomain {
                        dimension: dimension.id.clone(),
                        endpoint: "source",
                        observation,
                    });
                }
            }
            for value in &scope.target_comparison_domain {
                let observation = dimension
                    .target_observer
                    .evaluate(value)
                    .map_err(|source| PolicyError::Observer {
                        dimension: dimension.id.clone(),
                        source,
                    })?;
                if dimension
                    .target_codomain
                    .binary_search(&observation)
                    .is_err()
                {
                    return Err(PolicyError::ObservationOutsideCodomain {
                        dimension: dimension.id.clone(),
                        endpoint: "target",
                        observation,
                    });
                }
            }
        }
        for id in &self.match_dimensions {
            if dimensions[id.as_str()].match_relation.is_none() {
                return Err(PolicyError::MissingMatchRelation(id.clone()));
            }
        }

        let mut safe = BTreeSet::new();
        let mut matched = BTreeSet::new();
        let mut observations = BTreeMap::new();
        for pair in &scope.comparison_universe {
            let mut safe_pair = true;
            for id in &self.safe_dimensions {
                let dimension = dimensions[id.as_str()];
                let (source, target) = observe_dimension(dimension, pair)?;
                safe_pair &= dimension.safe_relation.allows(&source, &target);
                observations.insert((id.clone(), pair.clone()), (source, target));
            }
            if safe_pair {
                safe.insert(pair.clone());
            }

            let mut match_pair = !self.match_dimensions.is_empty();
            for id in &self.match_dimensions {
                let dimension = dimensions[id.as_str()];
                let (source, target) = observe_dimension(dimension, pair)?;
                let relation = dimension
                    .match_relation
                    .as_ref()
                    .ok_or_else(|| PolicyError::MissingMatchRelation(id.clone()))?;
                match_pair &= relation.allows(&source, &target);
                observations.insert((id.clone(), pair.clone()), (source, target));
            }
            if match_pair {
                matched.insert(pair.clone());
            }
        }
        if let Some(pair) = matched.difference(&safe).next() {
            return Err(PolicyError::MatchNotSafe(pair.clone()));
        }
        Ok(ConstructedPolicy {
            safe,
            matched,
            observations,
        })
    }

    pub fn dimension(&self, id: &str) -> Option<&PolicyDimension> {
        self.dimensions.iter().find(|dimension| dimension.id == id)
    }
}

fn validate_relation_codomains(
    dimension: &PolicyDimension,
    relation: &Relation,
) -> Result<(), PolicyError> {
    let valid = match relation {
        Relation::Exact => dimension.source_codomain == dimension.target_codomain,
        Relation::TargetNoAmplification { elements, .. } => {
            &dimension.source_codomain == elements && &dimension.target_codomain == elements
        }
        Relation::FiniteTable {
            left_codomain,
            right_codomain,
            ..
        } => {
            &dimension.source_codomain == left_codomain
                && &dimension.target_codomain == right_codomain
        }
    };
    if valid {
        Ok(())
    } else {
        Err(PolicyError::InvalidRelation {
            dimension: dimension.id.clone(),
            reason: "relation codomain does not exactly match dimension codomains".into(),
        })
    }
}

fn observe_dimension(
    dimension: &PolicyDimension,
    pair: &ValuePair,
) -> Result<(Observation, Observation), PolicyError> {
    let source = dimension
        .source_observer
        .evaluate(&pair.source)
        .map_err(|source| PolicyError::Observer {
            dimension: dimension.id.clone(),
            source,
        })?;
    let target = dimension
        .target_observer
        .evaluate(&pair.target)
        .map_err(|source| PolicyError::Observer {
            dimension: dimension.id.clone(),
            source,
        })?;
    if dimension.source_codomain.binary_search(&source).is_err() {
        return Err(PolicyError::ObservationOutsideCodomain {
            dimension: dimension.id.clone(),
            endpoint: "source",
            observation: source,
        });
    }
    if dimension.target_codomain.binary_search(&target).is_err() {
        return Err(PolicyError::ObservationOutsideCodomain {
            dimension: dimension.id.clone(),
            endpoint: "target",
            observation: target,
        });
    }
    Ok((source, target))
}

fn ensure_sorted_unique<T: Ord>(values: &[T], label: &str) -> Result<(), String> {
    if strict_sorted(values) || values.len() <= 1 {
        Ok(())
    } else {
        Err(format!("{label} must be duplicate-free and sorted"))
    }
}

fn strict_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(value: &str) -> Observation {
        Observation::Atom {
            value: value.into(),
        }
    }

    #[test]
    fn tna_uses_target_below_source_direction() {
        let deny = atom("deny");
        let allow = atom("allow");
        let relation = Relation::TargetNoAmplification {
            elements: vec![allow.clone(), deny.clone()]
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            preorder_edges: vec![
                ObservationPair {
                    left: deny.clone(),
                    right: deny.clone(),
                },
                ObservationPair {
                    left: deny.clone(),
                    right: allow.clone(),
                },
                ObservationPair {
                    left: allow.clone(),
                    right: allow.clone(),
                },
            ]
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        };
        relation.validate().unwrap();
        assert!(relation.allows(&allow, &deny));
        assert!(!relation.allows(&deny, &allow));
    }
}
