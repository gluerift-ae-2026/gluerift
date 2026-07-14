use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::type_ir::{EnumerationLimits, ResultBranch, Type, TypeError, Value};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Adapter {
    Identity,
    Compose {
        first: Box<Adapter>,
        second: Box<Adapter>,
    },
    EnumPermutation {
        mapping: BTreeMap<String, String>,
    },
    FieldPermutation {
        /// Output-field name to input-field name.
        mapping: BTreeMap<String, String>,
    },
    SumMap {
        variants: BTreeMap<String, SumVariantMap>,
    },
    ProductMap {
        fields: BTreeMap<String, ProductFieldMap>,
    },
    ResultMap {
        branch_mapping: BranchMapping,
        ok: Box<Adapter>,
        err: Box<Adapter>,
    },
    BoundedComplement {
        min: i64,
        max: i64,
    },
    ModularAffine {
        width: u8,
        scale: u64,
        offset: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SumVariantMap {
    pub target: String,
    pub adapter: Adapter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductFieldMap {
    pub source: String,
    pub adapter: Adapter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BranchMapping {
    Preserve,
    Swap,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterContext {
    pub schema: String,
    pub source_type: Type,
    pub target_type: Type,
    pub carrier_type: Type,
    pub source_encode: Adapter,
    pub source_decode: Adapter,
    pub target_encode: Adapter,
    pub target_decode: Adapter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationStep {
    pub adapter_path: String,
    pub input: Value,
    pub result: Result<Value, ConversionError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("adapter evaluation failed at {adapter_path}: {reason}")]
#[serde(deny_unknown_fields)]
pub struct ConversionError {
    pub adapter_path: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum AdapterTypeError {
    #[error(transparent)]
    Type(#[from] TypeError),
    #[error("context schema must be gluerift.adapter-context/v0.3.1a")]
    WrongSchema,
    #[error("adapter `{map}` is not total and typed on input {input:?}: {reason}")]
    Evaluation {
        map: &'static str,
        input: Value,
        reason: String,
    },
    #[error("adapter `{map}` produced an out-of-type value on input {input:?}: {output:?}")]
    OutputType {
        map: &'static str,
        input: Value,
        output: Value,
    },
    #[error("adapter `{map}` fails syntax-directed type checking: {reason}")]
    StaticType { map: &'static str, reason: String },
}

impl Adapter {
    pub fn type_check(&self, input: &Type, output: &Type) -> Result<(), String> {
        match self {
            Adapter::Compose { first, second } => {
                let intermediate = first.infer_output_type(input)?;
                first.type_check(input, &intermediate)?;
                second.type_check(&intermediate, output)
            }
            Adapter::Identity => exact_types(input, output, "Identity"),
            Adapter::EnumPermutation { .. }
            | Adapter::FieldPermutation { .. }
            | Adapter::ResultMap { .. }
            | Adapter::BoundedComplement { .. }
            | Adapter::ModularAffine { .. } => {
                let inferred = self.infer_output_type(input)?;
                exact_types(&inferred, output, "adapter")
            }
            Adapter::SumMap { variants: mappings } => {
                let Type::Sum {
                    variants: source_variants,
                } = input
                else {
                    return Err("SumMap input must be Sum".into());
                };
                let Type::Sum {
                    variants: target_variants,
                } = output
                else {
                    return Err("SumMap output must be Sum".into());
                };
                let source_names: BTreeSet<_> = source_variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect();
                let mapping_names: BTreeSet<_> = mappings.keys().map(String::as_str).collect();
                if source_names != mapping_names {
                    return Err("SumMap must cover every source constructor exactly once".into());
                }
                let target_map: BTreeMap<_, _> = target_variants
                    .iter()
                    .map(|variant| (variant.name.as_str(), &variant.payload))
                    .collect();
                for source in source_variants {
                    let rule = &mappings[&source.name];
                    let target_payload = target_map.get(rule.target.as_str()).ok_or_else(|| {
                        format!(
                            "SumMap target `{}` is absent from declared output",
                            rule.target
                        )
                    })?;
                    rule.adapter.type_check(&source.payload, target_payload)?;
                }
                Ok(())
            }
            Adapter::ProductMap { fields: mappings } => {
                let Type::Product {
                    fields: source_fields,
                } = input
                else {
                    return Err("ProductMap input must be Product".into());
                };
                let Type::Product {
                    fields: target_fields,
                } = output
                else {
                    return Err("ProductMap output must be Product".into());
                };
                let source_map: BTreeMap<_, _> = source_fields
                    .iter()
                    .map(|field| (field.name.as_str(), &field.ty))
                    .collect();
                let target_names: BTreeSet<_> = target_fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect();
                let mapping_names: BTreeSet<_> = mappings.keys().map(String::as_str).collect();
                if target_names != mapping_names {
                    return Err("ProductMap must supply every output field exactly once".into());
                }
                for target in target_fields {
                    let rule = &mappings[&target.name];
                    let source = source_map
                        .get(rule.source.as_str())
                        .ok_or_else(|| format!("ProductMap source `{}` is absent", rule.source))?;
                    rule.adapter.type_check(source, &target.ty)?;
                }
                Ok(())
            }
        }
    }

    pub fn infer_output_type(&self, input: &Type) -> Result<Type, String> {
        match self {
            Adapter::Identity => Ok(input.normalized()),
            Adapter::Compose { first, second } => {
                let intermediate = first.infer_output_type(input)?;
                second.infer_output_type(&intermediate)
            }
            Adapter::EnumPermutation { mapping } => {
                let Type::Sum { variants } = input else {
                    return Err("EnumPermutation input must be Sum".into());
                };
                if variants.iter().any(|variant| variant.payload != Type::Unit) {
                    return Err("EnumPermutation requires payload-free variants".into());
                }
                let input_names: BTreeSet<_> = variants
                    .iter()
                    .map(|variant| variant.name.clone())
                    .collect();
                let keys: BTreeSet<_> = mapping.keys().cloned().collect();
                let targets: BTreeSet<_> = mapping.values().cloned().collect();
                if keys != input_names || targets.len() != mapping.len() {
                    return Err("EnumPermutation must be a total bijection".into());
                }
                Ok(Type::Sum {
                    variants: targets
                        .into_iter()
                        .map(|name| crate::type_ir::Variant {
                            name,
                            payload: Type::Unit,
                        })
                        .collect(),
                })
            }
            Adapter::FieldPermutation { mapping } => {
                let Type::Product { fields } = input else {
                    return Err("FieldPermutation input must be Product".into());
                };
                let input_map: BTreeMap<_, _> = fields
                    .iter()
                    .map(|field| (field.name.clone(), field.ty.clone()))
                    .collect();
                let sources: BTreeSet<_> = mapping.values().cloned().collect();
                if mapping.len() != fields.len()
                    || sources.len() != fields.len()
                    || !sources.iter().all(|name| input_map.contains_key(name))
                {
                    return Err(
                        "FieldPermutation must consume every input field exactly once".into(),
                    );
                }
                Ok(Type::Product {
                    fields: mapping
                        .iter()
                        .map(|(target, source)| crate::type_ir::Field {
                            name: target.clone(),
                            ty: input_map[source].clone(),
                        })
                        .collect(),
                })
            }
            Adapter::SumMap { variants: mappings } => {
                let Type::Sum { variants } = input else {
                    return Err("SumMap input must be Sum".into());
                };
                let input_names: BTreeSet<_> = variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect();
                let mapping_names: BTreeSet<_> = mappings.keys().map(String::as_str).collect();
                if input_names != mapping_names {
                    return Err("SumMap must cover every source constructor exactly once".into());
                }
                let mut targets: BTreeMap<String, Type> = BTreeMap::new();
                for variant in variants {
                    let rule = &mappings[&variant.name];
                    let payload = rule.adapter.infer_output_type(&variant.payload)?;
                    if let Some(existing) = targets.get(&rule.target) {
                        if existing.normalized() != payload.normalized() {
                            return Err(format!(
                                "SumMap target `{}` receives incompatible payloads",
                                rule.target
                            ));
                        }
                    } else {
                        targets.insert(rule.target.clone(), payload);
                    }
                }
                Ok(Type::Sum {
                    variants: targets
                        .into_iter()
                        .map(|(name, payload)| crate::type_ir::Variant { name, payload })
                        .collect(),
                })
            }
            Adapter::ProductMap { fields: mappings } => {
                let Type::Product { fields } = input else {
                    return Err("ProductMap input must be Product".into());
                };
                let input_map: BTreeMap<_, _> = fields
                    .iter()
                    .map(|field| (field.name.clone(), field.ty.clone()))
                    .collect();
                let mut output = Vec::new();
                for (target, rule) in mappings {
                    let source = input_map.get(&rule.source).ok_or_else(|| {
                        format!("ProductMap source field `{}` does not exist", rule.source)
                    })?;
                    output.push(crate::type_ir::Field {
                        name: target.clone(),
                        ty: rule.adapter.infer_output_type(source)?,
                    });
                }
                Ok(Type::Product { fields: output })
            }
            Adapter::ResultMap {
                branch_mapping,
                ok,
                err,
            } => {
                let Type::ObjectResult {
                    ok: source_ok,
                    err: source_err,
                } = input
                else {
                    return Err("ResultMap input must be ObjectResult".into());
                };
                let mapped_ok = ok.infer_output_type(source_ok)?;
                let mapped_err = err.infer_output_type(source_err)?;
                Ok(match branch_mapping {
                    BranchMapping::Preserve => Type::ObjectResult {
                        ok: Box::new(mapped_ok),
                        err: Box::new(mapped_err),
                    },
                    BranchMapping::Swap => Type::ObjectResult {
                        ok: Box::new(mapped_err),
                        err: Box::new(mapped_ok),
                    },
                })
            }
            Adapter::BoundedComplement { min, max } => match input {
                Type::BoundedInt {
                    min: input_min,
                    max: input_max,
                } if input_min == min && input_max == max && min <= max => Ok(input.clone()),
                _ => {
                    Err("BoundedComplement bounds must exactly equal its input bounded type".into())
                }
            },
            Adapter::ModularAffine { width, .. } => match input {
                Type::BitVec { width: input_width } if input_width == width => Ok(input.clone()),
                _ => Err("ModularAffine width must exactly equal its input BitVec width".into()),
            },
        }
    }

    pub fn eval(&self, input: &Value) -> Result<Value, ConversionError> {
        self.eval_at(input, "$")
    }

    fn eval_at(&self, input: &Value, path: &str) -> Result<Value, ConversionError> {
        match self {
            Adapter::Identity => Ok(input.clone()),
            Adapter::Compose { first, second } => {
                let middle = first.eval_at(input, &format!("{path}.first"))?;
                second.eval_at(&middle, &format!("{path}.second"))
            }
            Adapter::EnumPermutation { mapping } => match input {
                Value::Sum { variant, payload } if matches!(payload.as_ref(), Value::Unit) => {
                    let target = mapping.get(variant).ok_or_else(|| {
                        fail(path, format!("enum mapping is not total at `{variant}`"))
                    })?;
                    Ok(Value::Sum {
                        variant: target.clone(),
                        payload: Box::new(Value::Unit),
                    })
                }
                _ => Err(fail(
                    path,
                    "EnumPermutation requires a payload-free Sum value",
                )),
            },
            Adapter::FieldPermutation { mapping } => match input {
                Value::Product { fields } => {
                    if mapping.len() != fields.len() {
                        return Err(fail(
                            path,
                            "field permutation cardinality differs from input product",
                        ));
                    }
                    let mut used = BTreeSet::new();
                    let mut output = BTreeMap::new();
                    for (target, source) in mapping {
                        if !used.insert(source) {
                            return Err(fail(path, format!("input field `{source}` is reused")));
                        }
                        let value = fields
                            .get(source)
                            .ok_or_else(|| fail(path, format!("missing input field `{source}`")))?;
                        output.insert(target.clone(), value.clone());
                    }
                    if used.len() != fields.len() {
                        return Err(fail(
                            path,
                            "field permutation does not consume every input field",
                        ));
                    }
                    Ok(Value::Product { fields: output })
                }
                _ => Err(fail(path, "FieldPermutation requires a Product value")),
            },
            Adapter::SumMap { variants } => match input {
                Value::Sum { variant, payload } => {
                    let rule = variants.get(variant).ok_or_else(|| {
                        fail(path, format!("sum mapping is not total at `{variant}`"))
                    })?;
                    let mapped = rule
                        .adapter
                        .eval_at(payload, &format!("{path}.variants[{variant}].adapter"))?;
                    Ok(Value::Sum {
                        variant: rule.target.clone(),
                        payload: Box::new(mapped),
                    })
                }
                _ => Err(fail(path, "SumMap requires a Sum value")),
            },
            Adapter::ProductMap { fields: mappings } => match input {
                Value::Product { fields } => {
                    let mut output = BTreeMap::new();
                    for (target, rule) in mappings {
                        let value = fields.get(&rule.source).ok_or_else(|| {
                            fail(path, format!("missing input field `{}`", rule.source))
                        })?;
                        let mapped = rule
                            .adapter
                            .eval_at(value, &format!("{path}.fields[{target}].adapter"))?;
                        output.insert(target.clone(), mapped);
                    }
                    Ok(Value::Product { fields: output })
                }
                _ => Err(fail(path, "ProductMap requires a Product value")),
            },
            Adapter::ResultMap {
                branch_mapping,
                ok,
                err,
            } => match input {
                Value::ObjectResult {
                    branch: ResultBranch::Ok,
                    value,
                } => {
                    let mapped = ok.eval_at(value, &format!("{path}.ok"))?;
                    let branch = match branch_mapping {
                        BranchMapping::Preserve => ResultBranch::Ok,
                        BranchMapping::Swap => ResultBranch::Err,
                    };
                    Ok(Value::ObjectResult {
                        branch,
                        value: Box::new(mapped),
                    })
                }
                Value::ObjectResult {
                    branch: ResultBranch::Err,
                    value,
                } => {
                    let mapped = err.eval_at(value, &format!("{path}.err"))?;
                    let branch = match branch_mapping {
                        BranchMapping::Preserve => ResultBranch::Err,
                        BranchMapping::Swap => ResultBranch::Ok,
                    };
                    Ok(Value::ObjectResult {
                        branch,
                        value: Box::new(mapped),
                    })
                }
                _ => Err(fail(path, "ResultMap requires an ObjectResult value")),
            },
            Adapter::BoundedComplement { min, max } => match input {
                Value::BoundedInt { value } if min <= value && value <= max && min <= max => {
                    let result = (*min as i128 + *max as i128 - *value as i128)
                        .try_into()
                        .map_err(|_| fail(path, "bounded complement overflow"))?;
                    Ok(Value::BoundedInt { value: result })
                }
                _ => Err(fail(
                    path,
                    "BoundedComplement input is outside its exact bounded domain",
                )),
            },
            Adapter::ModularAffine {
                width,
                scale,
                offset,
            } => match input {
                Value::BitVec { value }
                    if *width > 0 && *width < 64 && *value < (1u64 << *width) =>
                {
                    let modulus = 1i128 << *width;
                    let result = ((*scale as i128 * *value as i128) + *offset as i128)
                        .rem_euclid(modulus) as u64;
                    Ok(Value::BitVec { value: result })
                }
                _ => Err(fail(path, "ModularAffine input does not match its width")),
            },
        }
    }

    pub fn normalize(&self) -> Adapter {
        match self {
            Adapter::Compose { first, second } => {
                let mut stages = Vec::new();
                collect_composition_stages(&first.normalize(), &mut stages);
                collect_composition_stages(&second.normalize(), &mut stages);
                let mut stages = stages
                    .into_iter()
                    .filter(|stage| !matches!(stage, Adapter::Identity));
                let Some(first) = stages.next() else {
                    return Adapter::Identity;
                };
                stages.fold(first, |accumulator, stage| Adapter::Compose {
                    first: Box::new(accumulator),
                    second: Box::new(stage),
                })
            }
            Adapter::SumMap { variants } => Adapter::SumMap {
                variants: variants
                    .iter()
                    .map(|(name, rule)| {
                        (
                            name.clone(),
                            SumVariantMap {
                                target: rule.target.clone(),
                                adapter: rule.adapter.normalize(),
                            },
                        )
                    })
                    .collect(),
            },
            Adapter::ProductMap { fields } => Adapter::ProductMap {
                fields: fields
                    .iter()
                    .map(|(name, rule)| {
                        (
                            name.clone(),
                            ProductFieldMap {
                                source: rule.source.clone(),
                                adapter: rule.adapter.normalize(),
                            },
                        )
                    })
                    .collect(),
            },
            Adapter::ResultMap {
                branch_mapping,
                ok,
                err,
            } => Adapter::ResultMap {
                branch_mapping: *branch_mapping,
                ok: Box::new(ok.normalize()),
                err: Box::new(err.normalize()),
            },
            Adapter::ModularAffine {
                width,
                scale,
                offset,
            } if *width > 0 && *width < 64 => {
                let modulus = 1u64 << *width;
                Adapter::ModularAffine {
                    width: *width,
                    scale: *scale % modulus,
                    offset: (*offset as i128).rem_euclid(modulus as i128) as i64,
                }
            }
            other => other.clone(),
        }
    }

    pub fn modular_inverse(width: u8, scale: u64, offset: i64) -> Result<Adapter, ConversionError> {
        if width == 0 || width >= 64 {
            return Err(fail("$", "invalid modular width"));
        }
        let modulus = 1i128 << width;
        let inverse_scale = modular_inverse_i128(scale as i128, modulus)
            .ok_or_else(|| fail("$", "modular scale has no inverse"))?;
        let inverse_offset = (-inverse_scale * offset as i128).rem_euclid(modulus) as i64;
        Ok(Adapter::ModularAffine {
            width,
            scale: inverse_scale as u64,
            offset: inverse_offset,
        })
    }
}

fn collect_composition_stages(adapter: &Adapter, output: &mut Vec<Adapter>) {
    match adapter {
        Adapter::Compose { first, second } => {
            collect_composition_stages(first, output);
            collect_composition_stages(second, output);
        }
        other => output.push(other.clone()),
    }
}

fn exact_types(left: &Type, right: &Type, node: &str) -> Result<(), String> {
    if left.normalized() == right.normalized() {
        Ok(())
    } else {
        Err(format!(
            "{node} requires exact types, got {left:?} -> {right:?}"
        ))
    }
}

impl AdapterContext {
    pub fn validate(&self, limits: &EnumerationLimits) -> Result<(), AdapterTypeError> {
        if self.schema != "gluerift.adapter-context/v0.3.1a" {
            return Err(AdapterTypeError::WrongSchema);
        }
        self.source_type.validate(limits)?;
        self.target_type.validate(limits)?;
        self.carrier_type.validate(limits)?;
        validate_total(
            "source_encode",
            &self.source_encode,
            &self.source_type,
            &self.carrier_type,
            limits,
        )?;
        validate_total(
            "source_decode",
            &self.source_decode,
            &self.carrier_type,
            &self.source_type,
            limits,
        )?;
        validate_total(
            "target_encode",
            &self.target_encode,
            &self.target_type,
            &self.carrier_type,
            limits,
        )?;
        validate_total(
            "target_decode",
            &self.target_decode,
            &self.carrier_type,
            &self.target_type,
            limits,
        )?;
        Ok(())
    }

    pub fn normalized(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            source_type: self.source_type.normalized(),
            target_type: self.target_type.normalized(),
            carrier_type: self.carrier_type.normalized(),
            source_encode: self.source_encode.normalize(),
            source_decode: self.source_decode.normalize(),
            target_encode: self.target_encode.normalize(),
            target_decode: self.target_decode.normalize(),
        }
    }
}

pub fn validate_total(
    map: &'static str,
    adapter: &Adapter,
    input: &Type,
    output: &Type,
    limits: &EnumerationLimits,
) -> Result<(), AdapterTypeError> {
    adapter
        .type_check(input, output)
        .map_err(|reason| AdapterTypeError::StaticType { map, reason })?;
    for value in input.enumerate(limits)? {
        let result = adapter
            .eval(&value)
            .map_err(|error| AdapterTypeError::Evaluation {
                map,
                input: value.clone(),
                reason: error.to_string(),
            })?;
        if !output.contains(&result) {
            return Err(AdapterTypeError::OutputType {
                map,
                input: value,
                output: result,
            });
        }
    }
    Ok(())
}

fn modular_inverse_i128(a: i128, modulus: i128) -> Option<i128> {
    let (mut old_r, mut r) = (a.rem_euclid(modulus), modulus);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_s, s) = (s, old_s - quotient * s);
    }
    (old_r == 1).then(|| old_s.rem_euclid(modulus))
}

fn fail(path: &str, reason: impl Into<String>) -> ConversionError {
    ConversionError {
        adapter_path: path.into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_four_affine_has_working_inverse() {
        let adapter = Adapter::ModularAffine {
            width: 4,
            scale: 5,
            offset: 3,
        };
        let inverse = Adapter::modular_inverse(4, 5, 3).unwrap();
        for x in 0..16 {
            let value = Value::BitVec { value: x };
            assert_eq!(inverse.eval(&adapter.eval(&value).unwrap()).unwrap(), value);
            assert_eq!(adapter.eval(&inverse.eval(&value).unwrap()).unwrap(), value);
        }
    }

    #[test]
    fn even_scale_is_not_invertible() {
        assert!(Adapter::modular_inverse(4, 2, 1).is_err());
    }

    #[test]
    fn modular_affine_normalization_reduces_congruent_coefficients() {
        assert_eq!(
            Adapter::ModularAffine {
                width: 4,
                scale: 21,
                offset: -13,
            }
            .normalize(),
            Adapter::ModularAffine {
                width: 4,
                scale: 5,
                offset: 3,
            }
        );
    }

    #[test]
    fn product_map_preserves_diagnostic_child_path() {
        let mut fields = BTreeMap::new();
        fields.insert(
            "out".into(),
            ProductFieldMap {
                source: "missing".into(),
                adapter: Adapter::Identity,
            },
        );
        let error = Adapter::ProductMap { fields }
            .eval(&Value::Product {
                fields: BTreeMap::new(),
            })
            .unwrap_err();
        assert!(error.reason.contains("missing"));
    }
}
