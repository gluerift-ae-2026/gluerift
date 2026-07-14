use crate::pb::{BoundsCarrier, DecisionCarrier, E01Carrier, E02Carrier, PolicyCarrier};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub const ALL_LAW_IDS: [&str; 6] = [
    "source-native-roundtrip",
    "target-native-roundtrip",
    "source-carrier-roundtrip",
    "target-carrier-roundtrip",
    "source-full-transport-roundtrip",
    "target-full-transport-roundtrip",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum TargetDecision {
    Blocked,
    Permitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecisionLogicalCarrier {
    Deny,
    Allow,
}

impl DecisionLogicalCarrier {
    pub fn to_proto(self) -> E01Carrier {
        let decision = match self {
            Self::Deny => DecisionCarrier::Deny as i32,
            Self::Allow => DecisionCarrier::Allow as i32,
        };
        E01Carrier { decision }
    }

    pub fn from_proto(value: E01Carrier) -> Result<Self> {
        match DecisionCarrier::try_from(value.decision).ok() {
            Some(DecisionCarrier::Deny) => Ok(Self::Deny),
            Some(DecisionCarrier::Allow) => Ok(Self::Allow),
            _ => bail!("malformed E01 carrier: unspecified or unknown decision"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Bounds {
    pub minimum: i32,
    pub maximum: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Policy {
    pub bounds: Bounds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct NestedOutput {
    pub policy: Policy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct BoundsLogicalCarrier {
    pub minimum_slot: i32,
    pub maximum_slot: i32,
}

impl BoundsLogicalCarrier {
    pub fn to_proto(self) -> E02Carrier {
        E02Carrier {
            policy: Some(PolicyCarrier {
                bounds: Some(BoundsCarrier {
                    minimum_slot: self.minimum_slot,
                    maximum_slot: self.maximum_slot,
                }),
            }),
        }
    }

    pub fn from_proto(value: E02Carrier) -> Result<Self> {
        let policy = value
            .policy
            .ok_or_else(|| anyhow::anyhow!("missing E02 policy"))?;
        let bounds = policy
            .bounds
            .ok_or_else(|| anyhow::anyhow!("missing E02 bounds"))?;
        validate_bound(bounds.minimum_slot)?;
        validate_bound(bounds.maximum_slot)?;
        Ok(Self {
            minimum_slot: bounds.minimum_slot,
            maximum_slot: bounds.maximum_slot,
        })
    }
}

pub fn nested(minimum: i32, maximum: i32) -> NestedOutput {
    NestedOutput {
        policy: Policy {
            bounds: Bounds { minimum, maximum },
        },
    }
}

pub fn validate_nested(value: NestedOutput) -> Result<()> {
    validate_bound(value.policy.bounds.minimum)?;
    validate_bound(value.policy.bounds.maximum)
}

fn validate_bound(value: i32) -> Result<()> {
    if (0..=2).contains(&value) {
        Ok(())
    } else {
        bail!("E02 bounded integer {value} is outside 0..=2")
    }
}
