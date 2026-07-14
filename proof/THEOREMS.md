# Mechanized theorem index

All theorem groups below concern only total maps in the finite/shared-domain
mathematical core. Domain predicates remain explicit whenever a statement is
scope-sensitive. No entry claims verification of the Rust interpreter, native
processes, Protobuf, meta-level partial conversion, or an Extended profile.

| Contract group | Lean module | Mechanized result |
|---|---|---|
| L1 | `Gluerift.L1Injectivity` | Native RT implies encoder injectivity only for two values proved inside the admitted domain. |
| L2 | `Gluerift.L2Twist` | Target-side `σ`/`σ⁻¹` conjugation preserves target native and total carrier RT. |
| L3 | `Gluerift.L3FullTransport` | The same clean total twist preserves STS and TST transport. |
| L4 | `Gluerift.L4DirectLaundering` | The constructed pair lies directly in carrier-, target-native-, and source-native equality; an unsafe in-scope pair disproves target-native soundness without a bridge. |
| L5 | `Gluerift.L5VacuityDivergence` | Empty induced equality makes soundness vacuous, nonempty Match defeats adequacy, disjoint images empty only carrier equality, and executable V01 separates carrier/native relations while all six laws pass. |
| L6 | `Gluerift.L6ComparatorBridge` | Both bridge directions retain their exact native-domain or carrier-domain coverage premises; scoped equivalence is derived only from those premises. |
| L7 | `Gluerift.L7NativeShape` | Target-native is functional and source-native inverse-functional; the opposite Match shape needs adequacy and full-transport coverage of Match's corresponding projection. |
| L8 | `Gluerift.L8ResidualTransformations` | Exact stabilizer subgroup closure, two- and three-way lawful partitions, executable lawful-policy-only T01 non-closure, and executable sound-but-law-breaking T02 inapplicability. |
| L9 | `Gluerift.L9TotalComposition` | Total-success judgments compose under relation composition; exact equality and checker-validated preorder/TNA are the two specialized rules. |

`Gluerift.AxiomAudit` inventories every load-bearing theorem. `./audit.sh`
rejects proof escapes and currently reports that every audited theorem depends
on no axioms.
