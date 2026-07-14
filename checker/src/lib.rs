//! GlueRift Minimal Core reference semantics.
//!
//! The crate deliberately has one semantic evaluator.  The GlueRift and BL4
//! presentations call the same functions in [`comparison`], so a baseline
//! cannot diverge from the checker through duplicated semantics.

pub mod adapter_ir;
pub mod bridge;
pub mod canonical;
pub mod carrier;
pub mod comparator;
pub mod comparison;
pub mod composition;
pub mod domain;
pub mod native_reference;
pub mod observer_ir;
pub mod relation_ir;
pub mod report;
pub mod roundtrip;
pub mod transformation;
pub mod type_ir;
pub mod witness;

pub use adapter_ir::{Adapter, AdapterContext, ConversionError};
pub use canonical::{canonical_bytes, canonical_sha256};
pub use comparison::{CheckError, CheckedRun, check};
pub use domain::{ComparatorSpec, DomainSpec, PairDomainSpec, ValidationScope};
pub use observer_ir::{Observation, Observer};
pub use relation_ir::{EndpointPolicy, Relation};
pub use report::{CheckReport, Status};
pub use type_ir::{Type, Value};

pub const CONTRACT_VERSION: &str = "0.3.1a";
