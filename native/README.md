# GlueRift native replay

This directory realizes the v0.3.1a E01 and E02 operational witnesses. A Go
source producer and a Rust target/comparator are separate executables. They
exchange one bounded, length-delimited message generated from the shared
`proto/gluerift_native.proto`; the harness clears the ambient environment,
enforces a network-denying execution context, and treats the checker-emitted
content-addressed native-reference bundle as its only semantic authority. It
exhaustively compares all four native adapter maps, the complete target-native
relation, all six staged round-trip tables, and the operational witness/path
with that bundle. There is no second hand-written Rust reference model.

The semantic fixtures are fixed as follows:

- E01: source `DENY` encodes as carrier `DENY`; the conjugated target decoder
  maps that carrier to `Permitted`, matching the Rust program output.
- E02: `output.policy.bounds.{minimum,maximum}` ranges over `0..=2`. The target
  map exchanges the same-typed carrier slots. Source `{0,2}` therefore
  transports to Rust `{2,0}`, and the witness identifies
  `output.policy.bounds.minimum` as receiving the source `maximum` role.

From the source-only package, use the supported top-level provisioning flow:

```sh
./artifact/bootstrap-tools
./artifact/reproduce
```

For native-layer development only, the lower-level provisioning/check commands
are:

```sh
native/scripts/bootstrap-tools
native/scripts/generate-proto --check
```

The top-level checker supplies the E01-to-A01 and E02-to-A02 bundle and context
hash bindings. Run
the native layer into an external staging directory with:

```sh
native/scripts/reproduce \
  --bindings /path/to/reference-bindings.json \
  --out-dir /path/to/staging/native \
  --build-dir /path/to/external-build/native \
  --logical-out-prefix artifact/staging/native
```

Both directories must be absent or empty, external to the source tree, and
non-nested. If `--build-dir` is omitted, a sibling `${out_dir}.build` directory
is used. The canonical output contains only the two native manifests, small
build and dependency manifests, backend-conformance reports, replay reports,
human transcripts, and a canonical index. Compilers, caches, target trees, and
executables remain in the external build directory. No absolute checkout or
temporary path is serialized into evidence.
