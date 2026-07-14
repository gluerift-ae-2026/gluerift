use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Type {
    Unit,
    Bool,
    BoundedInt { min: i64, max: i64 },
    BitVec { width: u8 },
    Sum { variants: Vec<Variant> },
    Product { fields: Vec<Field> },
    ObjectResult { ok: Box<Type>, err: Box<Type> },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    pub name: String,
    pub payload: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Value {
    Unit,
    Bool {
        value: bool,
    },
    BoundedInt {
        value: i64,
    },
    BitVec {
        value: u64,
    },
    Sum {
        variant: String,
        payload: Box<Value>,
    },
    Product {
        fields: BTreeMap<String, Value>,
    },
    ObjectResult {
        branch: ResultBranch,
        value: Box<Value>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultBranch {
    Ok,
    Err,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumerationLimits {
    pub max_values_per_type: usize,
    pub max_product_arity: usize,
    pub max_sum_variants: usize,
    pub max_bit_width: u8,
    pub max_recursion_depth: usize,
}

impl Default for EnumerationLimits {
    fn default() -> Self {
        Self {
            max_values_per_type: 65_536,
            max_product_arity: 32,
            max_sum_variants: 256,
            max_bit_width: 16,
            max_recursion_depth: 64,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("invalid bounded integer interval {min}..={max}")]
    InvalidBounds { min: i64, max: i64 },
    #[error("bit-vector width {width} exceeds supported range 1..={maximum}")]
    InvalidBitWidth { width: u8, maximum: u8 },
    #[error("duplicate {kind} name `{name}`")]
    DuplicateName { kind: &'static str, name: String },
    #[error("empty sum type")]
    EmptySum,
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("value does not inhabit the expected type at {path}: {reason}")]
    ValueMismatch { path: String, reason: String },
}

impl Type {
    pub fn normalized(&self) -> Type {
        match self {
            Type::Unit => Type::Unit,
            Type::Bool => Type::Bool,
            Type::BoundedInt { min, max } => Type::BoundedInt {
                min: *min,
                max: *max,
            },
            Type::BitVec { width } => Type::BitVec { width: *width },
            Type::Sum { variants } => {
                let mut variants: Vec<_> = variants
                    .iter()
                    .map(|variant| Variant {
                        name: variant.name.clone(),
                        payload: variant.payload.normalized(),
                    })
                    .collect();
                variants.sort_by(|a, b| a.name.cmp(&b.name));
                Type::Sum { variants }
            }
            Type::Product { fields } => {
                let mut fields: Vec<_> = fields
                    .iter()
                    .map(|field| Field {
                        name: field.name.clone(),
                        ty: field.ty.normalized(),
                    })
                    .collect();
                fields.sort_by(|a, b| a.name.cmp(&b.name));
                Type::Product { fields }
            }
            Type::ObjectResult { ok, err } => Type::ObjectResult {
                ok: Box::new(ok.normalized()),
                err: Box::new(err.normalized()),
            },
        }
    }

    pub fn validate(&self, limits: &EnumerationLimits) -> Result<(), TypeError> {
        self.validate_at(limits, 0)
    }

    fn validate_at(&self, limits: &EnumerationLimits, depth: usize) -> Result<(), TypeError> {
        if depth > limits.max_recursion_depth {
            return Err(TypeError::ResourceLimit("type recursion depth".into()));
        }
        match self {
            Type::Unit | Type::Bool => Ok(()),
            Type::BoundedInt { min, max } => {
                if min > max {
                    Err(TypeError::InvalidBounds {
                        min: *min,
                        max: *max,
                    })
                } else {
                    Ok(())
                }
            }
            Type::BitVec { width } => {
                if *width == 0 || *width >= 64 {
                    Err(TypeError::InvalidBitWidth {
                        width: *width,
                        maximum: 63,
                    })
                } else if *width > limits.max_bit_width {
                    Err(TypeError::ResourceLimit("bit-vector width".into()))
                } else {
                    Ok(())
                }
            }
            Type::Sum { variants } => {
                if variants.is_empty() {
                    return Err(TypeError::EmptySum);
                }
                if variants.len() > limits.max_sum_variants {
                    return Err(TypeError::ResourceLimit("sum variant count".into()));
                }
                let mut names = BTreeSet::new();
                for variant in variants {
                    if !names.insert(&variant.name) {
                        return Err(TypeError::DuplicateName {
                            kind: "variant",
                            name: variant.name.clone(),
                        });
                    }
                    variant.payload.validate_at(limits, depth + 1)?;
                }
                Ok(())
            }
            Type::Product { fields } => {
                if fields.len() > limits.max_product_arity {
                    return Err(TypeError::ResourceLimit("product field count".into()));
                }
                let mut names = BTreeSet::new();
                for field in fields {
                    if !names.insert(&field.name) {
                        return Err(TypeError::DuplicateName {
                            kind: "field",
                            name: field.name.clone(),
                        });
                    }
                    field.ty.validate_at(limits, depth + 1)?;
                }
                Ok(())
            }
            Type::ObjectResult { ok, err } => {
                ok.validate_at(limits, depth + 1)?;
                err.validate_at(limits, depth + 1)
            }
        }
    }

    pub fn enumerate(&self, limits: &EnumerationLimits) -> Result<Vec<Value>, TypeError> {
        self.validate(limits)?;
        let values = self.enumerate_at(limits)?;
        if values.len() > limits.max_values_per_type {
            return Err(TypeError::ResourceLimit(format!(
                "type has {} values, limit is {}",
                values.len(),
                limits.max_values_per_type
            )));
        }
        Ok(values)
    }

    fn enumerate_at(&self, limits: &EnumerationLimits) -> Result<Vec<Value>, TypeError> {
        let mut out = match self {
            Type::Unit => vec![Value::Unit],
            Type::Bool => vec![Value::Bool { value: false }, Value::Bool { value: true }],
            Type::BoundedInt { min, max } => {
                let count: usize = (*max as i128 - *min as i128 + 1)
                    .try_into()
                    .map_err(|_| TypeError::ResourceLimit("bounded integer cardinality".into()))?;
                if count > limits.max_values_per_type {
                    return Err(TypeError::ResourceLimit(
                        "bounded integer cardinality".into(),
                    ));
                }
                (*min..=*max)
                    .map(|value| Value::BoundedInt { value })
                    .collect()
            }
            Type::BitVec { width } => {
                let count = 1usize
                    .checked_shl(u32::from(*width))
                    .ok_or_else(|| TypeError::ResourceLimit("bit-vector cardinality".into()))?;
                if count > limits.max_values_per_type {
                    return Err(TypeError::ResourceLimit("bit-vector cardinality".into()));
                }
                (0..count as u64)
                    .map(|value| Value::BitVec { value })
                    .collect()
            }
            Type::Sum { variants } => {
                let mut values = Vec::new();
                for variant in variants {
                    for payload in variant.payload.enumerate_at(limits)? {
                        values.push(Value::Sum {
                            variant: variant.name.clone(),
                            payload: Box::new(payload),
                        });
                        ensure_limit(values.len(), limits)?;
                    }
                }
                values
            }
            Type::Product { fields } => {
                let mut products: Vec<BTreeMap<String, Value>> = vec![BTreeMap::new()];
                for field in fields {
                    let field_values = field.ty.enumerate_at(limits)?;
                    let mut next = Vec::new();
                    for product in &products {
                        for value in &field_values {
                            let mut entry = product.clone();
                            entry.insert(field.name.clone(), value.clone());
                            next.push(entry);
                            ensure_limit(next.len(), limits)?;
                        }
                    }
                    products = next;
                }
                products
                    .into_iter()
                    .map(|fields| Value::Product { fields })
                    .collect()
            }
            Type::ObjectResult { ok, err } => {
                let mut values = Vec::new();
                for value in ok.enumerate_at(limits)? {
                    values.push(Value::ObjectResult {
                        branch: ResultBranch::Ok,
                        value: Box::new(value),
                    });
                    ensure_limit(values.len(), limits)?;
                }
                for value in err.enumerate_at(limits)? {
                    values.push(Value::ObjectResult {
                        branch: ResultBranch::Err,
                        value: Box::new(value),
                    });
                    ensure_limit(values.len(), limits)?;
                }
                values
            }
        };
        out.sort();
        out.dedup();
        ensure_limit(out.len(), limits)?;
        Ok(out)
    }

    pub fn contains(&self, value: &Value) -> bool {
        match (self, value) {
            (Type::Unit, Value::Unit) => true,
            (Type::Bool, Value::Bool { .. }) => true,
            (Type::BoundedInt { min, max }, Value::BoundedInt { value }) => {
                min <= value && value <= max
            }
            (Type::BitVec { width }, Value::BitVec { value }) if *width < 64 => {
                *value < (1u64 << *width)
            }
            (Type::Sum { variants }, Value::Sum { variant, payload }) => variants
                .iter()
                .find(|candidate| candidate.name == *variant)
                .is_some_and(|candidate| candidate.payload.contains(payload)),
            (Type::Product { fields }, Value::Product { fields: values }) => {
                fields.len() == values.len()
                    && fields.iter().all(|field| {
                        values
                            .get(&field.name)
                            .is_some_and(|value| field.ty.contains(value))
                    })
            }
            (
                Type::ObjectResult { ok, .. },
                Value::ObjectResult {
                    branch: ResultBranch::Ok,
                    value,
                },
            ) => ok.contains(value),
            (
                Type::ObjectResult { err, .. },
                Value::ObjectResult {
                    branch: ResultBranch::Err,
                    value,
                },
            ) => err.contains(value),
            _ => false,
        }
    }

    pub fn assert_contains(&self, value: &Value, path: &str) -> Result<(), TypeError> {
        if self.contains(value) {
            Ok(())
        } else {
            Err(TypeError::ValueMismatch {
                path: path.into(),
                reason: format!("{value:?} is not a member of {self:?}"),
            })
        }
    }

    /// Conservative set of endpoint-local constructor/field paths.  This is
    /// the v0.3.1a P2 interpretation: exclusions require explicit policy-owned
    /// irrelevance rather than a semantic dead-code analysis.
    pub fn reachable_paths(&self) -> BTreeSet<Vec<String>> {
        let mut out = BTreeSet::new();
        self.collect_paths(Vec::new(), &mut out);
        out
    }

    fn collect_paths(&self, prefix: Vec<String>, out: &mut BTreeSet<Vec<String>>) {
        match self {
            Type::Unit => {}
            Type::Bool | Type::BoundedInt { .. } | Type::BitVec { .. } => {
                out.insert(prefix);
            }
            Type::Sum { variants } => {
                out.insert(prefix.clone());
                for variant in variants {
                    if variant.payload != Type::Unit {
                        let mut p = prefix.clone();
                        p.push(variant.name.clone());
                        p.push("$payload".into());
                        variant.payload.collect_paths(p, out);
                    }
                }
            }
            Type::Product { fields } => {
                for field in fields {
                    let mut p = prefix.clone();
                    p.push(field.name.clone());
                    field.ty.collect_paths(p, out);
                }
            }
            Type::ObjectResult { ok, err } => {
                out.insert(prefix.clone());
                for (name, ty) in [("Ok", ok.as_ref()), ("Err", err.as_ref())] {
                    if ty != &Type::Unit {
                        let mut p = prefix.clone();
                        p.push(name.into());
                        p.push("$value".into());
                        ty.collect_paths(p, out);
                    }
                }
            }
        }
    }
}

fn ensure_limit(size: usize, limits: &EnumerationLimits) -> Result<(), TypeError> {
    if size > limits.max_values_per_type {
        Err(TypeError::ResourceLimit(
            "finite enumeration cardinality".into(),
        ))
    } else {
        Ok(())
    }
}

impl Value {
    pub fn constructor_name(&self) -> Option<&str> {
        match self {
            Value::Sum { variant, .. } => Some(variant),
            Value::ObjectResult {
                branch: ResultBranch::Ok,
                ..
            } => Some("Ok"),
            Value::ObjectResult {
                branch: ResultBranch::Err,
                ..
            } => Some("Err"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_enumeration_is_canonical() {
        let ty = Type::Product {
            fields: vec![
                Field {
                    name: "x".into(),
                    ty: Type::Bool,
                },
                Field {
                    name: "y".into(),
                    ty: Type::Bool,
                },
            ],
        };
        let values = ty.enumerate(&EnumerationLimits::default()).unwrap();
        assert_eq!(values.len(), 4);
        assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn rejects_duplicate_fields() {
        let ty = Type::Product {
            fields: vec![
                Field {
                    name: "x".into(),
                    ty: Type::Unit,
                },
                Field {
                    name: "x".into(),
                    ty: Type::Unit,
                },
            ],
        };
        assert!(matches!(
            ty.validate(&EnumerationLimits::default()),
            Err(TypeError::DuplicateName { .. })
        ));
    }

    #[test]
    fn rejects_unrepresentable_bounded_integer_cardinality() {
        let ty = Type::BoundedInt {
            min: i64::MIN,
            max: i64::MAX,
        };
        assert!(matches!(
            ty.enumerate(&EnumerationLimits::default()),
            Err(TypeError::ResourceLimit(_))
        ));
    }
}
