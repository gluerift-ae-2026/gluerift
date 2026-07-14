use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Serialize the integer-only control language used by the native layer in
/// RFC-8785 key order. The native evidence vocabulary intentionally excludes
/// floating-point values, avoiding cross-runtime number-format ambiguity.
pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value)?;
    let mut out = Vec::new();
    write_value(&value, &mut out)?;
    Ok(out)
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn sha256<T: Serialize>(value: &T) -> Result<String> {
    Ok(sha256_bytes(&to_vec(value)?))
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

pub fn write_file<T: Serialize>(path: &Path, value: &T) -> Result<String> {
    let bytes = to_vec(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, &bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn write_value(value: &Value, out: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(number) => {
            if number.is_f64() {
                bail!("floating-point numbers are forbidden in native canonical evidence")
            }
            out.extend_from_slice(number.to_string().as_bytes());
        }
        Value::String(text) => out.extend_from_slice(serde_json::to_string(text)?.as_bytes()),
        Value::Array(values) => {
            out.push(b'[');
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                write_value(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(values) => {
            out.push(b'{');
            let sorted: BTreeMap<&String, &Value> = values.iter().collect();
            for (index, (key, item)) in sorted.into_iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                out.extend_from_slice(serde_json::to_string(key)?.as_bytes());
                out.push(b':');
                write_value(item, out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_object_keys_recursively() {
        let value = json!({"z": 1, "a": {"q": false, "b": "x"}});
        assert_eq!(
            String::from_utf8(to_vec(&value).unwrap()).unwrap(),
            r#"{"a":{"b":"x","q":false},"z":1}"#
        );
    }
}
