use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::type_ir::{ResultBranch, Value};

pub type ValuePath = Vec<String>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnchorReads {
    /// Paths whose constructor/leaf identity is inspected exactly.
    pub exact: BTreeSet<ValuePath>,
    /// Paths whose complete finite value is inspected; this covers all
    /// semantic descendants of that path.
    pub whole_value: BTreeSet<ValuePath>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Observation {
    Atom {
        value: String,
    },
    Value {
        value: Value,
    },
    Tuple {
        items: Vec<Observation>,
    },
    Roles {
        roles: BTreeMap<String, Observation>,
    },
    Case {
        constructor: String,
        value: Box<Observation>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Observer {
    ConstructorRole {
        path: ValuePath,
        table: BTreeMap<String, String>,
    },
    FieldRole {
        roles: Vec<RoleObserver>,
    },
    FinitePolicyMap {
        path: ValuePath,
        table: Vec<PolicyMapEntry>,
    },
    Tuple {
        items: Vec<Observer>,
    },
    Case {
        scrutinee_path: ValuePath,
        branches: BTreeMap<String, Box<Observer>>,
    },
    ExternalObserverRef {
        id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleObserver {
    pub role: String,
    pub path: ValuePath,
    pub inner: Box<Observer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyMapEntry {
    pub value: Value,
    pub atom: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "error_kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ObserverError {
    #[error("unsupported observer `{id}`")]
    UnsupportedObserver { id: String },
    #[error("invalid path {path:?}: {reason}")]
    InvalidPath { path: ValuePath, reason: String },
    #[error("constructor `{constructor}` is absent from observer table")]
    MissingConstructor { constructor: String },
    #[error("value is absent from finite policy map: {value:?}")]
    MissingPolicyValue { value: Value },
    #[error("duplicate role `{role}`")]
    DuplicateRole { role: String },
    #[error("field roles must be duplicate-free and ordered by role name")]
    NonCanonicalRoles,
    #[error("finite policy table must be duplicate-free and canonically ordered")]
    NonCanonicalPolicyTable,
    #[error("case branch `{constructor}` is absent")]
    MissingCaseBranch { constructor: String },
}

impl Observer {
    pub fn anchor_reads(&self) -> AnchorReads {
        let mut reads = AnchorReads::default();
        self.collect_anchor_reads(Vec::new(), &mut reads);
        reads
    }

    fn collect_anchor_reads(&self, prefix: ValuePath, reads: &mut AnchorReads) {
        match self {
            Observer::ConstructorRole { path, .. } => {
                reads.exact.insert(join_path(&prefix, path));
            }
            Observer::FinitePolicyMap { path, .. } => {
                reads.whole_value.insert(join_path(&prefix, path));
            }
            Observer::FieldRole { roles } => {
                for role in roles {
                    role.inner
                        .collect_anchor_reads(join_path(&prefix, &role.path), reads);
                }
            }
            Observer::Tuple { items } => {
                for item in items {
                    item.collect_anchor_reads(prefix.clone(), reads);
                }
            }
            Observer::Case {
                scrutinee_path,
                branches,
            } => {
                let base = join_path(&prefix, scrutinee_path);
                reads.exact.insert(base.clone());
                for (constructor, branch) in branches {
                    let mut branch_path = base.clone();
                    branch_path.push(constructor.clone());
                    branch_path.push("$payload".into());
                    branch.collect_anchor_reads(branch_path, reads);
                }
            }
            Observer::ExternalObserverRef { .. } => {}
        }
    }

    /// Syntax-directed coverage validation over the complete declared
    /// endpoint domain.  Evaluation alone detects missing entries, but exact
    /// reachable tables also reject authored dead/extra constructors or
    /// values, as required by the closed Observer IR.
    pub fn validate_on_domain(&self, roots: &[Value]) -> Result<(), ObserverError> {
        match self {
            Observer::ConstructorRole { path, table } => {
                let mut reachable = BTreeSet::new();
                for root in roots {
                    let selected = resolve_path(root, path)?;
                    let constructor =
                        selected
                            .constructor_name()
                            .ok_or_else(|| ObserverError::InvalidPath {
                                path: path.clone(),
                                reason: "ConstructorRole selected a non-constructor value".into(),
                            })?;
                    reachable.insert(constructor.to_owned());
                }
                let authored: BTreeSet<_> = table.keys().cloned().collect();
                if authored != reachable {
                    return Err(ObserverError::InvalidPath {
                        path: path.clone(),
                        reason: format!(
                            "ConstructorRole table keys {authored:?} differ from reachable constructors {reachable:?}"
                        ),
                    });
                }
                Ok(())
            }
            Observer::FinitePolicyMap { path, table } => {
                if !table.windows(2).all(|pair| pair[0].value < pair[1].value) {
                    return Err(ObserverError::NonCanonicalPolicyTable);
                }
                let reachable: BTreeSet<_> = roots
                    .iter()
                    .map(|root| resolve_path(root, path).cloned())
                    .collect::<Result<_, _>>()?;
                let authored: BTreeSet<_> = table.iter().map(|entry| entry.value.clone()).collect();
                if authored != reachable {
                    return Err(ObserverError::InvalidPath {
                        path: path.clone(),
                        reason: "FinitePolicyMap table is not exact over reachable values".into(),
                    });
                }
                Ok(())
            }
            Observer::FieldRole { roles } => {
                if !roles.windows(2).all(|pair| pair[0].role < pair[1].role) {
                    return Err(ObserverError::NonCanonicalRoles);
                }
                for role in roles {
                    let selected: Vec<_> = roots
                        .iter()
                        .map(|root| resolve_path(root, &role.path).cloned())
                        .collect::<Result<_, _>>()?;
                    role.inner.validate_on_domain(&selected)?;
                }
                Ok(())
            }
            Observer::Tuple { items } => {
                for item in items {
                    item.validate_on_domain(roots)?;
                }
                Ok(())
            }
            Observer::Case {
                scrutinee_path,
                branches,
            } => {
                let mut payloads: BTreeMap<String, Vec<Value>> = BTreeMap::new();
                for root in roots {
                    let selected = resolve_path(root, scrutinee_path)?;
                    let constructor =
                        selected
                            .constructor_name()
                            .ok_or_else(|| ObserverError::InvalidPath {
                                path: scrutinee_path.clone(),
                                reason: "Case selected a non-constructor value".into(),
                            })?;
                    let payload = match selected {
                        Value::Sum { payload, .. } | Value::ObjectResult { value: payload, .. } => {
                            payload.as_ref()
                        }
                        _ => {
                            return Err(ObserverError::InvalidPath {
                                path: scrutinee_path.clone(),
                                reason: "Case selected a value without a payload".into(),
                            });
                        }
                    };
                    payloads
                        .entry(constructor.to_owned())
                        .or_default()
                        .push(payload.clone());
                }
                let reachable: BTreeSet<_> = payloads.keys().cloned().collect();
                let authored: BTreeSet<_> = branches.keys().cloned().collect();
                if authored != reachable {
                    return Err(ObserverError::InvalidPath {
                        path: scrutinee_path.clone(),
                        reason: format!(
                            "Case branches {authored:?} differ from reachable constructors {reachable:?}"
                        ),
                    });
                }
                for (constructor, values) in payloads {
                    branches[&constructor].validate_on_domain(&values)?;
                }
                Ok(())
            }
            Observer::ExternalObserverRef { id } => {
                Err(ObserverError::UnsupportedObserver { id: id.clone() })
            }
        }
    }

    pub fn evaluate(&self, root: &Value) -> Result<Observation, ObserverError> {
        match self {
            Observer::ConstructorRole { path, table } => {
                let value = resolve_path(root, path)?;
                let constructor =
                    value
                        .constructor_name()
                        .ok_or_else(|| ObserverError::InvalidPath {
                            path: path.clone(),
                            reason: "ConstructorRole selected a non-constructor value".into(),
                        })?;
                let role =
                    table
                        .get(constructor)
                        .ok_or_else(|| ObserverError::MissingConstructor {
                            constructor: constructor.into(),
                        })?;
                Ok(Observation::Atom {
                    value: role.clone(),
                })
            }
            Observer::FieldRole { roles } => {
                if !roles.windows(2).all(|pair| pair[0].role < pair[1].role) {
                    return Err(ObserverError::NonCanonicalRoles);
                }
                let mut output = BTreeMap::new();
                for role in roles {
                    if output.contains_key(&role.role) {
                        return Err(ObserverError::DuplicateRole {
                            role: role.role.clone(),
                        });
                    }
                    let selected = resolve_path(root, &role.path)?;
                    output.insert(role.role.clone(), role.inner.evaluate(selected)?);
                }
                Ok(Observation::Roles { roles: output })
            }
            Observer::FinitePolicyMap { path, table } => {
                if !table.windows(2).all(|pair| pair[0].value < pair[1].value) {
                    return Err(ObserverError::NonCanonicalPolicyTable);
                }
                let selected = resolve_path(root, path)?;
                let atom = table
                    .iter()
                    .find(|entry| entry.value == *selected)
                    .ok_or_else(|| ObserverError::MissingPolicyValue {
                        value: selected.clone(),
                    })?;
                Ok(Observation::Atom {
                    value: atom.atom.clone(),
                })
            }
            Observer::Tuple { items } => Ok(Observation::Tuple {
                items: items
                    .iter()
                    .map(|item| item.evaluate(root))
                    .collect::<Result<_, _>>()?,
            }),
            Observer::Case {
                scrutinee_path,
                branches,
            } => {
                let selected = resolve_path(root, scrutinee_path)?;
                let constructor =
                    selected
                        .constructor_name()
                        .ok_or_else(|| ObserverError::InvalidPath {
                            path: scrutinee_path.clone(),
                            reason: "Case selected a non-constructor value".into(),
                        })?;
                let branch =
                    branches
                        .get(constructor)
                        .ok_or_else(|| ObserverError::MissingCaseBranch {
                            constructor: constructor.into(),
                        })?;
                let payload = match selected {
                    Value::Sum { payload, .. } | Value::ObjectResult { value: payload, .. } => {
                        payload.as_ref()
                    }
                    _ => {
                        return Err(ObserverError::InvalidPath {
                            path: scrutinee_path.clone(),
                            reason: "Case selected a value without a payload".into(),
                        });
                    }
                };
                Ok(Observation::Case {
                    constructor: constructor.into(),
                    value: Box::new(branch.evaluate(payload)?),
                })
            }
            Observer::ExternalObserverRef { id } => {
                Err(ObserverError::UnsupportedObserver { id: id.clone() })
            }
        }
    }

    pub fn contains_external(&self) -> bool {
        match self {
            Observer::ExternalObserverRef { .. } => true,
            Observer::FieldRole { roles } => {
                roles.iter().any(|role| role.inner.contains_external())
            }
            Observer::Tuple { items } => items.iter().any(Observer::contains_external),
            Observer::Case { branches, .. } => {
                branches.values().any(|branch| branch.contains_external())
            }
            Observer::ConstructorRole { .. } | Observer::FinitePolicyMap { .. } => false,
        }
    }

    pub fn observed_paths(&self) -> BTreeSet<ValuePath> {
        let mut output = BTreeSet::new();
        self.collect_paths(Vec::new(), &mut output);
        output
    }

    fn collect_paths(&self, prefix: ValuePath, output: &mut BTreeSet<ValuePath>) {
        match self {
            Observer::ConstructorRole { path, .. } | Observer::FinitePolicyMap { path, .. } => {
                output.insert(join_path(&prefix, path));
            }
            Observer::FieldRole { roles } => {
                for role in roles {
                    let base = join_path(&prefix, &role.path);
                    output.insert(base.clone());
                    role.inner.collect_paths(base, output);
                }
            }
            Observer::Tuple { items } => {
                for item in items {
                    item.collect_paths(prefix.clone(), output);
                }
            }
            Observer::Case {
                scrutinee_path,
                branches,
            } => {
                let base = join_path(&prefix, scrutinee_path);
                output.insert(base.clone());
                for (constructor, branch) in branches {
                    let mut branch_path = base.clone();
                    branch_path.push(constructor.clone());
                    branch_path.push("$payload".into());
                    branch.collect_paths(branch_path, output);
                }
            }
            Observer::ExternalObserverRef { .. } => {}
        }
    }
}

pub fn resolve_path<'a>(root: &'a Value, path: &[String]) -> Result<&'a Value, ObserverError> {
    let mut current = root;
    let mut consumed = Vec::new();
    for segment in path {
        consumed.push(segment.clone());
        current = match (current, segment.as_str()) {
            (Value::Product { fields }, name) => {
                fields.get(name).ok_or_else(|| ObserverError::InvalidPath {
                    path: consumed.clone(),
                    reason: format!("product has no field `{name}`"),
                })?
            }
            (Value::Sum { payload, .. }, "$payload") => payload,
            (Value::Sum { variant, .. }, name) if variant == name => current,
            (Value::ObjectResult { value, .. }, "$value") => value,
            (
                Value::ObjectResult {
                    branch: ResultBranch::Ok,
                    ..
                },
                "Ok",
            ) => current,
            (
                Value::ObjectResult {
                    branch: ResultBranch::Err,
                    ..
                },
                "Err",
            ) => current,
            _ => {
                return Err(ObserverError::InvalidPath {
                    path: consumed,
                    reason: format!("segment `{segment}` is incompatible with {current:?}"),
                });
            }
        };
    }
    Ok(current)
}

fn join_path(prefix: &[String], suffix: &[String]) -> Vec<String> {
    prefix.iter().chain(suffix).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_field_policy_observation() {
        let value = Value::Product {
            fields: BTreeMap::from([(
                "outer".into(),
                Value::Product {
                    fields: BTreeMap::from([("risk".into(), Value::BoundedInt { value: 2 })]),
                },
            )]),
        };
        let observer = Observer::FinitePolicyMap {
            path: vec!["outer".into(), "risk".into()],
            table: vec![PolicyMapEntry {
                value: Value::BoundedInt { value: 2 },
                atom: "high".into(),
            }],
        };
        assert_eq!(
            observer.evaluate(&value).unwrap(),
            Observation::Atom {
                value: "high".into()
            }
        );
    }

    #[test]
    fn closed_policy_table_rejects_unreachable_extra_value() {
        let observer = Observer::FinitePolicyMap {
            path: vec![],
            table: vec![
                PolicyMapEntry {
                    value: Value::BoundedInt { value: 0 },
                    atom: "low".into(),
                },
                PolicyMapEntry {
                    value: Value::BoundedInt { value: 1 },
                    atom: "high".into(),
                },
            ],
        };
        assert!(matches!(
            observer.validate_on_domain(&[Value::BoundedInt { value: 0 }]),
            Err(ObserverError::InvalidPath { .. })
        ));
    }
}
