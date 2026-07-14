# Core baseline configurations

`BL2/config.json` selects exactly A01, A02, A03, and A05 and accepts a row only
when every explicitly requested round-trip law is `proved-exhaustive`. It does
not receive `Safe` or `Match` as an acceptance relation.

`BL4/config.json` selects every contract-required Direct-Relation row. BL4 and
GlueRift share the same semantic kernel, normalized inputs, validity checks,
pair order, and top-level witness selection. The runner therefore treats any
listed parity-field difference as a tooling error rather than a research
result.
