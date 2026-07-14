use anyhow::{Result, bail};
use gluerift_native::canonical;
use gluerift_native::model::{
    BoundsLogicalCarrier, DecisionLogicalCarrier, NestedOutput, TargetDecision, nested,
    validate_nested,
};
use gluerift_native::pb::{E01Carrier, E02Carrier};
use gluerift_native::wire::{decode_frame, encode_frame, read_stdin};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write;

const SCHEMA: &str = "gluerift.native-protocol/v1";

#[derive(Debug)]
struct Args {
    fixture: String,
    operation: String,
    value: Option<String>,
}

#[derive(Serialize)]
struct Envelope<T> {
    schema: &'static str,
    fixture_id: String,
    operation_id: String,
    payload: T,
}

fn parse_args() -> Result<Args> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        if !matches!(flag.as_str(), "--fixture" | "--operation" | "--value") {
            bail!("unknown argument {flag}")
        }
        let value = args
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing value for {flag}"))?;
        if values.insert(flag.clone(), value).is_some() {
            bail!("duplicate argument {flag}")
        }
    }
    Ok(Args {
        fixture: values
            .remove("--fixture")
            .ok_or_else(|| anyhow::anyhow!("missing --fixture"))?,
        operation: values
            .remove("--operation")
            .ok_or_else(|| anyhow::anyhow!("missing --operation"))?,
        value: values.remove("--value"),
    })
}

fn write_json<T: Serialize>(fixture: &str, operation: &str, payload: T) -> Result<()> {
    let envelope = Envelope {
        schema: SCHEMA,
        fixture_id: fixture.to_owned(),
        operation_id: operation.to_owned(),
        payload,
    };
    std::io::stdout().write_all(&canonical::to_vec(&envelope)?)?;
    Ok(())
}

fn write_binary<T: Serialize>(
    fixture: &str,
    operation: &str,
    payload: T,
    bytes: &[u8],
) -> Result<()> {
    let envelope = Envelope {
        schema: SCHEMA,
        fixture_id: fixture.to_owned(),
        operation_id: operation.to_owned(),
        payload,
    };
    std::io::stderr().write_all(&canonical::to_vec(&envelope)?)?;
    std::io::stdout().write_all(bytes)?;
    Ok(())
}

fn required_value(args: &Args) -> Result<&str> {
    args.value
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--value is required for encode"))
}

// These are the manually written Rust backend maps. They are intentionally
// separate from the reference maps used by the harness so backend-conformance
// testing can detect a one-sided or incorrectly oriented implementation.
fn native_decision_encode(value: TargetDecision) -> DecisionLogicalCarrier {
    match value {
        TargetDecision::Blocked => DecisionLogicalCarrier::Allow,
        TargetDecision::Permitted => DecisionLogicalCarrier::Deny,
    }
}

fn native_decision_decode(value: DecisionLogicalCarrier) -> TargetDecision {
    match value {
        DecisionLogicalCarrier::Deny => TargetDecision::Permitted,
        DecisionLogicalCarrier::Allow => TargetDecision::Blocked,
    }
}

fn native_bounds_encode(value: NestedOutput) -> BoundsLogicalCarrier {
    BoundsLogicalCarrier {
        minimum_slot: value.policy.bounds.maximum,
        maximum_slot: value.policy.bounds.minimum,
    }
}

fn native_bounds_decode(value: BoundsLogicalCarrier) -> NestedOutput {
    nested(value.maximum_slot, value.minimum_slot)
}

fn run(args: Args) -> Result<()> {
    match (args.fixture.as_str(), args.operation.as_str()) {
        ("E01", "encode") => {
            let native: TargetDecision = serde_json::from_str(required_value(&args)?)?;
            let carrier = native_decision_encode(native);
            let bytes = encode_frame(&carrier.to_proto())?;
            write_binary(
                "E01",
                "encode",
                json!({"native": native, "carrier": carrier}),
                &bytes,
            )
        }
        ("E01", "decode") => {
            let proto: E01Carrier = decode_frame(&read_stdin()?)?;
            let carrier = DecisionLogicalCarrier::from_proto(proto)?;
            let native = native_decision_decode(carrier);
            write_json(
                "E01",
                "decode",
                json!({"carrier": carrier, "native": native}),
            )
        }
        ("E01", "program-output") => write_json(
            "E01",
            "program-output",
            json!({"native": TargetDecision::Permitted}),
        ),
        ("E01", "transport-compare") => {
            let proto: E01Carrier = decode_frame(&read_stdin()?)?;
            let carrier = DecisionLogicalCarrier::from_proto(proto)?;
            let transported = native_decision_decode(carrier);
            let target_program_output = TargetDecision::Permitted;
            let ordinary_comparator = if transported == target_program_output {
                "EQUAL"
            } else {
                "NOT_EQUAL"
            };
            write_json(
                "E01",
                "transport-compare",
                json!({
                    "carrier": carrier,
                    "transported_source_as_target_native": transported,
                    "target_program_output": target_program_output,
                    "ordinary_comparator": ordinary_comparator
                }),
            )
        }
        ("E01", "compare") => {
            let expected: TargetDecision = serde_json::from_str(required_value(&args)?)?;
            let proto: E01Carrier = decode_frame(&read_stdin()?)?;
            let carrier = DecisionLogicalCarrier::from_proto(proto)?;
            let transported = native_decision_decode(carrier);
            let ordinary_comparator = if transported == expected {
                "EQUAL"
            } else {
                "NOT_EQUAL"
            };
            write_json(
                "E01",
                "compare",
                json!({"transported": transported, "candidate": expected, "ordinary_comparator": ordinary_comparator}),
            )
        }
        ("E02", "encode") => {
            let native: NestedOutput = serde_json::from_str(required_value(&args)?)?;
            validate_nested(native)?;
            let carrier = native_bounds_encode(native);
            let bytes = encode_frame(&carrier.to_proto())?;
            write_binary(
                "E02",
                "encode",
                json!({"native": native, "carrier": carrier}),
                &bytes,
            )
        }
        ("E02", "decode") => {
            let proto: E02Carrier = decode_frame(&read_stdin()?)?;
            let carrier = BoundsLogicalCarrier::from_proto(proto)?;
            let native = native_bounds_decode(carrier);
            write_json(
                "E02",
                "decode",
                json!({"carrier": carrier, "native": native}),
            )
        }
        ("E02", "program-output") => {
            write_json("E02", "program-output", json!({"native": nested(2, 0)}))
        }
        ("E02", "transport-compare") => {
            let proto: E02Carrier = decode_frame(&read_stdin()?)?;
            let carrier = BoundsLogicalCarrier::from_proto(proto)?;
            let transported = native_bounds_decode(carrier);
            let target_program_output = nested(2, 0);
            let ordinary_comparator = if transported == target_program_output {
                "EQUAL"
            } else {
                "NOT_EQUAL"
            };
            write_json(
                "E02",
                "transport-compare",
                json!({
                    "carrier": carrier,
                    "transported_source_as_target_native": transported,
                    "target_program_output": target_program_output,
                    "ordinary_comparator": ordinary_comparator,
                    "nested_adapter_path": "output.policy.bounds.minimum"
                }),
            )
        }
        ("E02", "compare") => {
            let expected: NestedOutput = serde_json::from_str(required_value(&args)?)?;
            validate_nested(expected)?;
            let proto: E02Carrier = decode_frame(&read_stdin()?)?;
            let carrier = BoundsLogicalCarrier::from_proto(proto)?;
            let transported = native_bounds_decode(carrier);
            let ordinary_comparator = if transported == expected {
                "EQUAL"
            } else {
                "NOT_EQUAL"
            };
            write_json(
                "E02",
                "compare",
                json!({"transported": transported, "candidate": expected, "ordinary_comparator": ordinary_comparator}),
            )
        }
        (_, "encode") if args.value.is_none() => bail!("--value is required for encode"),
        _ => bail!(
            "unsupported fixture/operation: {}/{}",
            args.fixture,
            args.operation
        ),
    }
}

fn main() {
    let result = parse_args().and_then(run);
    if let Err(error) = result {
        let payload = json!({
            "schema": SCHEMA,
            "status": "malformed-message",
            "error": format!("{error:#}")
        });
        let bytes =
            canonical::to_vec(&payload).unwrap_or_else(|_| b"{\"status\":\"tool-error\"}".to_vec());
        let _ = std::io::stderr().write_all(&bytes);
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use serde_json::Value;

    #[test]
    fn parse_nested_payload_shape() {
        let value: NestedOutput = serde_json::from_value(json!({
            "policy": {"bounds": {"minimum": 0, "maximum": 2}}
        }))
        .unwrap();
        assert_eq!(value, nested(0, 2));
    }

    #[test]
    fn protocol_is_integer_only() {
        let value: Value = serde_json::from_slice(
            &canonical::to_vec(&json!({"fixture_id": "E01", "count": 2})).unwrap(),
        )
        .context("parse canonical protocol")
        .unwrap();
        assert_eq!(value["count"], 2);
    }
}
