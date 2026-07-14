#![recursion_limit = "512"]

pub mod canonical;
pub mod conformance;
pub mod evidence;
pub mod model;
pub mod process;
pub mod reference;
pub mod wire;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/gluerift.native.v1.rs"));
}
