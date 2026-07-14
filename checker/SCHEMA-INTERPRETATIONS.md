# Frozen Core schema interpretations

These implementation notes record the three approved P2 interpretations of
the v0.3.1a contract. They do not amend the contract.

- `source_domain` is exactly `D_S = D_S^rt`, and `target_domain` is exactly
  `D_T = D_T^rt`; the checker intentionally has no second native-round-trip
  domain field.
- The structural analyzer enumerates enum, same-typed field, compatible
  object-`Result`, and nested structural transformations. `BoundedComplement`
  and `ModularAffine` enter only through the separately validated declared
  candidate registry; no scalar-discovery completeness is claimed.
- Anchor relevance is computed conservatively for the total four-map Core:
  reachable endpoint structure read by those maps is combined with active
  observer paths. A leaf read covers its traversed ancestors, a parent-value
  read covers its descendants, and exclusion is possible only through a
  policy-owned `explicitly_irrelevant_paths` declaration.
