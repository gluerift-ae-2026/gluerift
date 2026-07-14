# GlueRift Minimal Core implementation decisions

This document records the bounded implementation interpretations approved with
the frozen v0.3.1a contract. It does not amend the research contract.

## Domain aliases

The schema names are aliases for the mathematical native round-trip domains:

```text
source_domain := D_S := D_S^rt
target_domain := D_T := D_T^rt
```

Minimal Core has no second source- or target-native round-trip domain field.

## Transformation admission

The `core-structural` family distinguishes two admission modes:

```text
enumerated_generators:
  EnumPermutation
  FieldPermutation
  compatible Result branch permutation
  nested structural composition

admitted_declared_candidates:
  BoundedComplement
  ModularAffine
```

An enumerated generator participates in the finite structural-family
completeness claim. A declared scalar candidate is exhaustively checked after
admission, but does not support a scalar-discovery completeness claim.

## Conservative anchor coverage

Relevant paths are the conservative union of every reachable constructor and
field path read by the normalized ASTs of the four adapter maps and all active
endpoint observers. A path can be removed only by a policy-owned explicit
irrelevance declaration. The implementation does not perform semantic dead-code
analysis to shrink this set.

## Result ownership

One Rust semantic evaluator owns all finite verdicts. GlueRift and BL4 invoke
that same evaluator; their difference is presentation and additional diagnostic
provenance, never a second implementation of the set-theoretic checks.

