use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::adapter_ir::AdapterContext;
use crate::canonical::{CanonicalError, canonical_sha256};
use crate::type_ir::{EnumerationLimits, Type, TypeError, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparatorSpec {
    CarrierExact,
    TargetNativeExact,
    SourceNativeExact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum DomainSpec {
    All,
    FiniteSet { values: Vec<Value> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PairDomainSpec {
    Product {
        source: DomainSpec,
        target: DomainSpec,
    },
    FinitePairSet {
        pairs: Vec<ValuePair>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValuePair {
    pub source: Value,
    pub target: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationScope {
    pub schema: String,
    /// This is exactly D_S = D_S^rt in Minimal Core; there is no second native
    /// round-trip domain field.
    pub source_domain: DomainSpec,
    /// This is exactly D_T = D_T^rt in Minimal Core.
    pub target_domain: DomainSpec,
    pub source_comparison_domain: DomainSpec,
    pub target_comparison_domain: DomainSpec,
    pub comparison_universe: PairDomainSpec,
    pub source_carrier_domain: DomainSpec,
    pub target_carrier_domain: DomainSpec,
    pub source_full_transport_domain: DomainSpec,
    pub target_full_transport_domain: DomainSpec,
    pub comparator: ComparatorSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedScope {
    pub source_domain: Vec<Value>,
    pub target_domain: Vec<Value>,
    pub source_comparison_domain: Vec<Value>,
    pub target_comparison_domain: Vec<Value>,
    pub comparison_universe: Vec<ValuePair>,
    pub source_carrier_domain: Vec<Value>,
    pub target_carrier_domain: Vec<Value>,
    pub source_full_transport_domain: Vec<Value>,
    pub target_full_transport_domain: Vec<Value>,
    pub comparator: ComparatorSpec,
    pub source_comparison_domain_sha256: String,
    pub target_comparison_domain_sha256: String,
}

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("scope schema must be gluerift.validation-scope/v0.3.1a")]
    WrongSchema,
    #[error("{domain} contains a value outside its type: {value:?}")]
    OutOfType { domain: &'static str, value: Value },
    #[error("{domain} must be duplicate-free and canonically sorted")]
    NonCanonical { domain: &'static str },
    #[error("comparison universe must be nonempty")]
    EmptyUniverse,
    #[error("comparison-universe pair lies outside declared comparison domains: {pair:?}")]
    PairOutsideComparisonDomain { pair: ValuePair },
    #[error(
        "full-transport domain does not cover comparison projection on endpoint {endpoint}: {value:?}"
    )]
    FullTransportCoverage {
        endpoint: &'static str,
        value: Value,
    },
}

impl DomainSpec {
    pub fn resolve(
        &self,
        ty: &Type,
        limits: &EnumerationLimits,
        name: &'static str,
    ) -> Result<Vec<Value>, DomainError> {
        match self {
            DomainSpec::All => Ok(ty.enumerate(limits)?),
            DomainSpec::FiniteSet { values } => {
                if !is_strict_sorted(values) {
                    return Err(DomainError::NonCanonical { domain: name });
                }
                for value in values {
                    if !ty.contains(value) {
                        return Err(DomainError::OutOfType {
                            domain: name,
                            value: value.clone(),
                        });
                    }
                }
                Ok(values.clone())
            }
        }
    }
}

impl ValidationScope {
    pub fn resolve(
        &self,
        context: &AdapterContext,
        limits: &EnumerationLimits,
    ) -> Result<ResolvedScope, DomainError> {
        if self.schema != "gluerift.validation-scope/v0.3.1a" {
            return Err(DomainError::WrongSchema);
        }
        let source_domain =
            self.source_domain
                .resolve(&context.source_type, limits, "source_domain")?;
        let target_domain =
            self.target_domain
                .resolve(&context.target_type, limits, "target_domain")?;
        let source_comparison_domain = self.source_comparison_domain.resolve(
            &context.source_type,
            limits,
            "source_comparison_domain",
        )?;
        let target_comparison_domain = self.target_comparison_domain.resolve(
            &context.target_type,
            limits,
            "target_comparison_domain",
        )?;
        let source_carrier_domain = self.source_carrier_domain.resolve(
            &context.carrier_type,
            limits,
            "source_carrier_domain",
        )?;
        let target_carrier_domain = self.target_carrier_domain.resolve(
            &context.carrier_type,
            limits,
            "target_carrier_domain",
        )?;
        let source_full_transport_domain = self.source_full_transport_domain.resolve(
            &context.source_type,
            limits,
            "source_full_transport_domain",
        )?;
        let target_full_transport_domain = self.target_full_transport_domain.resolve(
            &context.target_type,
            limits,
            "target_full_transport_domain",
        )?;

        let mut comparison_universe = match &self.comparison_universe {
            PairDomainSpec::Product { source, target } => {
                let sources =
                    source.resolve(&context.source_type, limits, "comparison_universe.source")?;
                let targets =
                    target.resolve(&context.target_type, limits, "comparison_universe.target")?;
                let mut pairs = Vec::with_capacity(sources.len().saturating_mul(targets.len()));
                for source in &sources {
                    for target in &targets {
                        pairs.push(ValuePair {
                            source: source.clone(),
                            target: target.clone(),
                        });
                    }
                }
                pairs
            }
            PairDomainSpec::FinitePairSet { pairs } => {
                if !is_strict_sorted(pairs) {
                    return Err(DomainError::NonCanonical {
                        domain: "comparison_universe",
                    });
                }
                pairs.clone()
            }
        };
        comparison_universe.sort();
        comparison_universe.dedup();
        if comparison_universe.is_empty() {
            return Err(DomainError::EmptyUniverse);
        }

        let source_cmp: BTreeSet<_> = source_comparison_domain.iter().cloned().collect();
        let target_cmp: BTreeSet<_> = target_comparison_domain.iter().cloned().collect();
        for pair in &comparison_universe {
            if !source_cmp.contains(&pair.source) || !target_cmp.contains(&pair.target) {
                return Err(DomainError::PairOutsideComparisonDomain { pair: pair.clone() });
            }
        }
        let source_transport: BTreeSet<_> = source_full_transport_domain.iter().cloned().collect();
        let target_transport: BTreeSet<_> = target_full_transport_domain.iter().cloned().collect();
        for pair in &comparison_universe {
            if !source_transport.contains(&pair.source) {
                return Err(DomainError::FullTransportCoverage {
                    endpoint: "source",
                    value: pair.source.clone(),
                });
            }
            if !target_transport.contains(&pair.target) {
                return Err(DomainError::FullTransportCoverage {
                    endpoint: "target",
                    value: pair.target.clone(),
                });
            }
        }

        Ok(ResolvedScope {
            source_comparison_domain_sha256: canonical_sha256(&source_comparison_domain)?,
            target_comparison_domain_sha256: canonical_sha256(&target_comparison_domain)?,
            source_domain,
            target_domain,
            source_comparison_domain,
            target_comparison_domain,
            comparison_universe,
            source_carrier_domain,
            target_carrier_domain,
            source_full_transport_domain,
            target_full_transport_domain,
            comparator: self.comparator,
        })
    }
}

fn is_strict_sorted<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_sets_must_be_canonical() {
        let spec = DomainSpec::FiniteSet {
            values: vec![Value::Bool { value: true }, Value::Bool { value: false }],
        };
        assert!(matches!(
            spec.resolve(&Type::Bool, &EnumerationLimits::default(), "x"),
            Err(DomainError::NonCanonical { .. })
        ));
    }
}
