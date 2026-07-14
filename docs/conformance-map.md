# GlueRift Minimal Core conformance map

The sole normative source is
`ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md`, whose approved
SHA-256 is
`1b0ebee64fcb482f87e1d37bece9a5ae2fc44bac7121607f31a531ea9dcf9fc7`.
Earlier contract versions are excluded from implementation decisions and source
input ownership.

| Contract gate | Canonical implementation owner | Executable conformance evidence |
|---|---|---|
| L1–L9 total finite core | `proof/GlueRift/*.lean` | pinned `lake build`, proof-hygiene and axiom audit |
| Core Type, Adapter, Observer, Relation IR | `checker/src/` | Rust unit tests plus schema-bound fixture runs |
| Three comparators and definedness | Rust semantic kernel | comparator microtests and V01 dual run |
| Six round trips | Rust semantic kernel | every attack/base/native reference run and BL2 |
| Safe, Match, coverage, four comparison properties, TNA | Rust semantic kernel | categorical fixture matrix and witness replay |
| Lawful transformation partition | Rust semantic kernel | A01/A02/A03/A05, T01, and T02 reports |
| Structural-family enumeration | Rust semantic kernel and hashed family descriptor | A01/A02/A05 generation plus nested-family microtest |
| Total exact/preorder composition | Rust semantic kernel and Lean L9 | C01 exact and TNA derivations with exhaustive recheck |
| Direct-Relation baseline | shared semantic kernel, `baselines/BL4/` presentation | registry-selected parity checks and shared first witnesses |
| Exhaustive round-trip baseline | shared semantic kernel, `baselines/BL2/` presentation | A01/A02/A03/A05 law-layer acceptance |
| Native operational witnesses | `native/E01/`, `native/E02/`, shared Protobuf | process replay, six laws, ordinary target-native equality, backend conformance |
| Canonical reports and schemas | `spec/schema/`, generated staging evidence | schema validation, RFC 8785 bytes, SHA-256 edge validation |
| Evidence DAG and claim bounds | `artifact/reproduction-manifest.json`, `artifact/claims.json` | topological hash audit and forbidden-overstatement rejection |
| Unique result owner and paper table | `artifact/results/results.json`, `artifact/tables/` | regeneration and byte comparison |
| One-command release | `artifact/reproduce` | clean external staging build and complete gate audit |

## Fixed categorical oracles

- A01, A02, A03, and A05: aligned bases pass definedness, all six laws,
  soundness, adequacy, precision, and faithfulness; mechanically transformed
  candidates remain lawful and disprove all four comparison properties.
- H01 and H02: all four comparison properties pass.
- H04 TNA: all four properties and the one requested TNA dimension pass.
- H04 exact: adequacy passes; soundness, precision, and faithfulness fail.
- V01: all six laws pass; carrier relation is empty while target- and
  source-native relations are diagonal; target-native soundness fails.
- V02: policy/request invalid because requested nonempty Match coverage cannot be
  established.
- V06: requested semantics are `unknown` with `unsupported-observer`.
- V10: soundness and adequacy pass; precision and faithfulness fail on the first
  canonical extra-safe-equality witness.
- T01: the two fixed generators are `lawful-safe`; their fixed right-to-left
  composite is `lawful-harmful`, with every requested law still passing.
- T02: soundness passes as a diagnostic, a requested carrier round trip fails,
  and classification is `law-breaking-or-inapplicable`.
- The empty safety-dimension conformance run is `policy-unconstrained`, warns,
  and cannot receive a security certificate.
- C01 contains only total-success exact and verified-preorder derivations.
- E01 and E02 reproduce ordinary target-native false agreement in separate
  processes and bind exactly to A01 and A02 respectively.

## Integration invariants

All canonical workspace paths are repository-relative POSIX paths. Canonical
reports contain no timestamps, host names, temporary paths, or unordered maps.
Every property and witness binds the selected comparator hash. Every generated
attack binds its aligned base, normalized transformation and inverse, complete
carrier action domain, mechanically conjugated four maps, requested-law results,
transformed candidate, classification, and canonical witnesses. Unsupported
semantics remain `unknown`; they are never approximated.
