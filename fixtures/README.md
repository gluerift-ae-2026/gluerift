# GlueRift Core fixtures

This directory is the authored finite input suite for semantic contract
v0.3.1a. `registry.json` is the canonical oracle owner. Evaluator inputs never
contain `expected_*` fields; the fixture runner reads those fields only after
semantic evaluation.

The four attack families and the T01/T02 transformation cases contain an
authored aligned base context plus a normalized transformation declaration.
Their transformed contexts are not checked in. They are reconstructed by
carrier conjugation under `artifact/staging/generated-contexts/` and bound by
the transformation report.

The two P2 interpretations visible in these inputs are:

- `source_domain` is the single Minimal-Core source native-roundtrip domain,
  and `target_domain` is the corresponding target domain.
- A01, A02, and A05 use generated structural transformations; A03 is an
  admitted declared bounded-complement candidate.

A02 is aligned with native E02 on the exact endpoint path
`output.policy.bounds.minimum`. Its carrier transformation acts at parent path
`output.policy.bounds` and swaps `minimum_slot` with `maximum_slot` over the
complete 0..2 product domain.

Regenerate and validate the authored inputs locally with:

```sh
./fixtures/generate.py
./fixtures/validate.py
```

`validate.py` checks exact canonical JSON bytes, every fixture and baseline
instance against its v0.3.1a schema, cross-file hashes, all §10.1 registry
fields, paired baseline sets, transformation staging discipline, the
categorical declaration matrix, and separation of expected oracles from
semantic inputs.
