# Contract provenance amendment — 2026-07-14

This administrative record distinguishes two byte sequences that were both
circulated under the `v0.3.1a` label. It does not change the semantic contract,
fixture registry, expected categorical matrix, or admissible evidence.

## Hash transition

- Previously circulated `v0.3.1a` SHA-256:
  `f10f8f9068202be2fc5c4e37c71ca15092e14fcffe3bef7c1141b128d4e86ffe`
- Current normative `v0.3.1a` SHA-256:
  `1b0ebee64fcb482f87e1d37bece9a5ae2fc44bac7121607f31a531ea9dcf9fc7`
- Normative path:
  `ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md`

## Bounded changes

The current bytes add or clarify only the audit corrections requested during
artifact review:

1. checker-emitted native reference bundles are the sole semantic authority
   for finite native conformance;
2. bundle identities are content-addressed through native and release evidence;
3. the pinned Darwin guarantee is named a host/toolchain descriptor rather
   than a complete OS image;
4. observer coverage is described as conservative reachable-path coverage; and
5. source/checked-release reproduction bindings are made explicit.

The fixture registry hash, run-configuration hash, transformation-family hash,
and categorical oracle were not changed by this contract-byte correction. No
result category was revised after implementation.

## Review status

The final external artifact re-evaluation dated 2026-07-14 explicitly approves
the current `1b0e…9fc7` contract bytes and reports no remaining P0 defect. It
classifies the hash transition as a provenance issue, not a research-result or
expected-matrix change. This file is the requested permanent provenance record.
