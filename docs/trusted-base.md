# Trusted base

The endpoint policy owner supplies the comparator, finite scope, Safe and Match
relations, observers, and requested laws/properties. GlueRift validates their
schema, typing, coverage, inclusion, vacuity, and profile consistency, but does
not infer their truth.

The Rust reference checker is trusted to execute the finite semantics; fixture
oracles, BL4 parity, canonical witness replay, and native backend conformance
exercise that implementation. Lean's pinned kernel and libraries check the
stated total finite mathematical core, not the Rust checker or native binaries.

Pinned Rust, Go, Protobuf, compiler/linker, Darwin host/toolchain descriptor,
and dependency identities form the native build and execution trust base. The
descriptor hashes the tested host and tool executables; it is not a complete OS
image. Native adapter implementations and the ordinary comparator are
exhaustively compared with the checker-emitted content-addressed reference
bundle. This is backend conformance evidence, not formal verification of the
binaries.
