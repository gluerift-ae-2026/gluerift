# GlueRift Lean Core

This directory mechanizes contract v0.3.1a §§3–8 and §14 for the stated total
finite/shared-domain core. It does not verify the Rust checker, native backends,
Protobuf, partial conversions, or Extended profiles.

Pinned build and hygiene audit:

```sh
./build.sh
./audit.sh
```

The local bootstrap is locked to official Lean 4.19.0 for Darwin/aarch64. On
another platform, provide the exact `lean-toolchain` release or add an official
archive and verified SHA-256 to `toolchain-lock.json`; do not silently substitute
a different compiler.

The executable finite models are emitted while building:

- V01: six round trips pass, carrier equality is empty, native equality aligns;
- T01: two lawful-safe transformations compose to lawful-harmful while all six
  requested laws continue to pass; and
- T02: a selected-comparator-sound twist is law-breaking/inapplicable because
  target-carrier round trip fails.
