# GlueRift Minimal Core

GlueRift is the executable artifact for *Round Trips Can Lie: Policy-Laundering
Attacks on Cross-Language Validation Adapters*. The frozen semantic contract is
v0.3.1a.

Starting from the source-only package, provision the checksum/version-pinned
tools and dependency caches, then reproduce the checked release:

```sh
./artifact/bootstrap-tools
./artifact/reproduce
```

`bootstrap-tools` is the only provisioning step and may use the network. The
reproduction command consumes those pinned local tools, enforces network-off
execution for semantic/native work, and writes all build/output state outside
the source tree. It selects its mode from the package contents: when checked
owner files are absent (the source-only package), it generates a complete
checked release at the reported external staging root; when all owner files are
present (the checked release), it validates them first and byte-compares the
regenerated graph. A partial owner set is rejected instead of guessed. The
explicit `--stage-only` flag forces generation mode. The tested profile is
Darwin/arm64; the host lock is a pinned Darwin host/toolchain descriptor, not a
content-addressed full OS image.

It validates immutable inputs, builds and audits the Lean core, builds and tests
the Rust reference checker, evaluates the categorical fixtures and BL2/BL4,
replays E01 and E02 as isolated Go/Rust/Protobuf processes, validates the native
backends against checker-emitted finite reference bundles, reconstructs the evidence
DAG, and generates both the canonical TSV and the TeX fragments directly input
by the manuscript. Checked-release mode byte-checks all of those table files.

The artifact is a bounded, finite, total-success verification package. Its exact
trust boundary and literal limitations are recorded under `docs/`.
