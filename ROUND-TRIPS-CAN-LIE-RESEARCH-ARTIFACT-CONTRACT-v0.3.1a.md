# Round Trips Can Lie

## Policy-Laundering Attacks on Cross-Language Validation Adapters

**Research and artifact implementation contract — version 0.3.1a**

**Working checker name:** **GlueRift**  
**Status:** approval candidate; implementation MUST NOT begin until external review  
**Intended venue:** SCORED ’26 research-paper track  
**Supersedes:** version 0.3.1 as the implementation source of truth  
**Normative language:** **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, and **MAY** are used in the RFC 2119 sense.

---

## 0. Purpose and decision requested

This document retains the externally accepted comparator-indexed Minimal Core of v0.3.1 and fixes one bounded transformation-classification defect, two bounded specification/evidence omissions, and one small profile/request consistency rule. It is a pre-implementation research contract, not evidence that the checker, proofs, fixtures, or native replay already exist.

The requested decision is:

> Does v0.3.1a now define the actual validator comparator, retain non-vacuous two-sided comparison obligations, and give a small enough but logically complete proof artifact to justify implementation?

The scope boundary is based on logical claim dependency, not an estimate of available time. Extended features may be implemented later, but no Extended feature is allowed to repair a failed Core claim.

### 0.1 Binding corrections through v0.3.1a

Version 0.3.1a retains corrections 1–14 from v0.3.1 and adds corrections 15–18:

1. `ComparatorSpec` is a first-class, policy-owned input.
2. Every induced relation, theorem, property, witness, baseline, and native run is indexed by the selected comparator.
3. `TargetNativeExact` is the primary comparator for the attack and native evidence because it matches the motivating validation path.
4. `CarrierExact` remains a required analysis and regression mode; it is not silently substituted for the native comparator.
5. Carrier/native bridge equivalence is an independent check result. Six round-trip laws do not establish it.
6. Carrier summaries or stabilizers may support a native-comparator claim only through a proved bridge. Native-comparator properties themselves are checked directly and need no bridge.
7. Core conversion is total on every requested comparison and round-trip scope. An unexpected meta-level `ConversionError` fails the relevant prerequisite.
8. The `OutcomeContract`, allowed-failure calculus, and effectful composition theorem move to an Extended profile. If later implemented, failure permission is indexed by the original input.
9. Empty safety dimensions produce `policy-unconstrained`, a warning, and no built-in security certification.
10. Core types, adapters, observers, relations, fixtures, baselines, native replays, and proof obligations are reduced to the items enumerated in this contract.
11. The prior working name **REFLENS** is retired because a 2026 system already uses **RefLens**. The replacement working name is **GlueRift**; the paper title remains unchanged.
12. Native manifests bind the comparator, build, toolchain, environment, input, dependencies, and repository-relative logical paths.
13. The paper’s generality wording is narrowed to a recent Go-to-Rust architecture rather than “validators commonly.”
14. Prior work on semantic soundness for language interoperability is added, and the novelty boundary is narrowed accordingly.
15. Generated transformations are partitioned into `lawful-safe`, `lawful-harmful`, and `law-breaking-or-inapplicable`; only well-typed, inverse-valid, mechanically conjugated, selected-comparator-defined transformations satisfying every law explicitly required by the request are called lawful.
16. Match coverage modes receive quantified semantics over the complete source and target comparison domains, not merely over the projection of a sparse universe or the values already appearing in Match.
17. Every generated twist binds its transformation, inverse, complete action domain, exact four-map carrier-conjugation construction, lawfulness evidence, aligned base check, exact transformed-candidate check binding, and transformed-context bridge evidence where applicable.
18. Validation profiles and `required_properties` have a checked compatibility table, and every attack registry row explicitly requires all six round-trip laws.

### 0.2 External approval rule

A **Go** recommendation is warranted only if the reviewer accepts all of the following:

1. The comparator-indexed definitions match the operational comparator used by the motivating architecture.
2. The target-native policy-laundering theorem does not rely on an unproved carrier/native bridge.
3. Soundness, adequacy, precision, and faithfulness are stated over the same selected comparator and independently owned universe.
4. Empty `Match` and empty safety dimensions cannot be presented as complete security evidence.
5. Match-shape requirements are conditional on the selected comparator and requested round-trip laws.
6. Core total-success composition is correctly scoped to exact equality and a verified finite preorder.
7. The Direct-Relation baseline receives the same comparator, semantic inputs, domains, validity checks, and totality obligations.
8. The Minimal Core fixtures and two native replays are sufficient to close the permitted paper claims.
9. The Core/Extended split does not conceal a premise needed by a Core theorem or fixture.
10. Transformation analysis calls a transformation safe or harmful only after well-typedness, comparator definedness, and every request-required law have passed.
11. Match coverage and generated-transformation provenance are specified sufficiently to admit one deterministic implementation.

A **No-Go** recommendation is warranted if any of the following remains:

- the checker proves `CarrierExact` while the native replay uses `TargetNativeExact`;
- a bridge is inferred from the six round-trip laws alone;
- target-native equality is evaluated through `e_T(t)` rather than through `d_T(e_S(s))`;
- an effectful theorem quotients per-input failure permissions by observer value;
- a candidate adapter can choose its comparator, universe, safety relation, or match relation;
- a `policy-unconstrained` run can receive a built-in security certificate;
- BL4 uses a weaker comparator or fewer common obligations than GlueRift;
- a transformation that breaks an explicitly requested round-trip law, or is comparator-undefined, is classified as safe or harmful rather than inapplicable;
- Match totality is evaluated only over pairs already present in `Match` or over a narrowed projection of \(U\);
- a generated twist cannot be reconstructed and checked from its transformation/inverse IR and four-map provenance;
- E01 fails to produce actual target-native false agreement while the required round trips pass.

### 0.3 Source-of-truth rule

Versions 0.3 and 0.3.1 are retained only as design and review history. Version 0.3.1a is the sole implementation source of truth after approval. In particular, no removed requirement from either prior version—including `OutcomeContract`, lists, units, checked casts, SMT, code generation, three native fixtures, four Core baselines, or ten regression fixtures—remains implicitly binding.

If prose conflicts with a numbered definition, theorem precondition, fixture row, schema requirement, or final acceptance gate, the more specific normative item controls and the conflict MUST be corrected before implementation.

---

# Part I — Research contract

## 1. Paper identity and thesis

### 1.1 Title and system name

Preferred paper title:

> **Round Trips Can Lie: Policy-Laundering Attacks on Cross-Language Validation Adapters**

Optional artifact-forward subtitle:

> **GlueRift: Comparator-Indexed Checking of Cross-Language Adapter Comparisons**

The paper MUST NOT present GlueRift as a new general lens calculus.

### 1.2 One-sentence thesis

> A recent Go-to-Rust validation architecture first checks adapter round trips and then reuses those adapters for differential I/O comparison; a round-trip-preserving adapter twist can therefore create target-native false agreement, while a restricted checker can separately test policy soundness and comparison adequacy against endpoint-owned semantics.

### 1.3 Draft abstract

A recent Go-to-Rust validation architecture first checks adapter round trips and then reuses those adapters for differential I/O comparison. We show that exhaustive or mechanically established round-trip laws need not determine the semantic alignment of the comparison. Twisting one endpoint’s encoder by a carrier automorphism and its decoder by the inverse preserves native, carrier-domain, and full cross-language round trips, yet can cause a source `DENY` to transport to the same target-native value as a target `ALLOW`. We call this a **policy-laundering attack**.

Round-trip ambiguity, automorphism invariance, explicit consistency relations, and relation-relative glue soundness are prior ideas. Our contribution is an operational attack on this translation-validation seam and a finite diagnostic realization. GlueRift makes the validator comparator explicit and separates **policy soundness**, which forbids unsafe induced equalities, from **comparison adequacy**, which requires declared corresponding pairs to compare equal. **Precision** and **faithfulness** distinguish extra safe equalities from exact realization of the required-match relation. This rejects both false agreement and vacuous always-different adapters.

The Minimal Core contains a finite typed adapter language, a closed observer language, complete checking over declared finite scopes, total-success composition for exact and finite-preorder observations, restricted harmful-twist generation, concrete witnesses, Lean proofs, and two Go/Rust/Protobuf native replays. A Direct-Relation baseline receives the same comparator and semantic information and is expected to reach the same top-level verdicts. GlueRift claims only any demonstrated advantage in structured diagnostics, local derivations, harmful-twist generation, and evidence binding—not greater logical expressiveness than a complete direct relation checker.

### 1.4 Paper type and load-bearing contributions

This is a **security attack-and-defense paper with a mechanized finite core and executable proof witnesses**.

The load-bearing contributions are:

1. **Operational target-native attack.** All six declared round trips pass while the actual target-native comparator reports an unsafe source/target agreement.
2. **Comparator-indexed formalization.** Carrier, target-native, and source-native equality are distinguished; no bridge is assumed.
3. **Two-sided criterion.** Soundness and adequacy are separated, with precision and faithfulness reported independently.
4. **Restricted executable checker.** A closed finite fragment emits comparator-specific counterexamples and explicitly reports unsupported semantics.
5. **Native replay.** Separate Go and Rust processes reproduce two false agreements through a shared Protobuf carrier under a hermetic manifest.
6. **Fair strongest baseline.** BL4 directly checks the same selected relation with the same policy and scope.

Natural vulnerabilities, prevalence, corpus coverage, performance leadership, whole-program equivalence, arbitrary native-code verification, and synthesis of endpoint meaning are not required or claimed.

---

## 2. Motivation, threat model, and novelty boundary

### 2.1 Protected validation seam

The studied architecture contains:

- a source program or API in language \(S\);
- a target translation in language \(T\);
- endpoint-native semantic types \(S\) and \(T\);
- a language-neutral carrier \(C\);
- source and target encoders and decoders;
- round-trip tests for those adapters; and
- a downstream comparator that transports a source output to a target-native value and compares it with the target program output.

For an output value, the architecture-faithful abstraction is:

\[
F_A(s)=e_S(s)\bind d_T
\]

and the ordinary program comparison is:

\[
F_A(f(v))=g(A_I(v)),
\]

where the input adapter \(A_I\) is honest in the output-laundering fixture.

The paper may say that a recent 2026 preprint describes a Go-to-Rust architecture with this pattern. It MUST NOT call the preprint a peer-reviewed publication, and MUST NOT say that validators “commonly” or “increasingly” use the pattern unless a broader systematic survey is added. It MUST NOT claim that a named implementation is vulnerable unless that implementation is actually executed; a faithful reconstruction must be labeled as such.

### 2.2 Adversary and trusted inputs

The adversary controls or corrupts the four adapter expressions:

\[
A=(e_S,d_S,e_T,d_T).
\]

The adversary may choose a law-preserving twist that passes all declared round-trip tests. The adversary does **not** control:

- the selected `ComparatorSpec`;
- endpoint domains and the comparison universe;
- the safety and required-match relations;
- observer definitions;
- requested laws and certification profile;
- the declared transformation family;
- native build/replay manifests; or
- the reference checker.

Those items belong to the trusted policy owner or fixed artifact release.

The policy owner is trusted to state the intended semantics. GlueRift checks consistency between that policy and the adapter-induced relation; it does not prove that the policy is socially, legally, or organizationally correct.

### 2.3 Non-novel ingredients

The paper MUST NOT claim as independent novelty:

- round-trip or lens laws;
- finite-type automorphisms;
- twisting one map by an automorphism and the inverse map by its inverse;
- explicit consistency relations in symmetric or relational lenses;
- predicate-bearing or partial lenses;
- cycle-consistent solution ambiguity under automorphisms;
- semantic relations for cross-language interoperability;
- verification of glue conversions relative to an independently supplied semantic relation;
- finite exhaustive checking, Protobuf, an adapter DSL, Lean, or native replay by themselves.

### 2.4 Defensible novelty

The defensible novelty is the following combination:

> A round-trip admission criterion in a recent cross-language translation-validation workflow permits an operational target-native false agreement under a law-preserving adapter twist; GlueRift supplies a finite red-team and diagnostic realization that distinguishes unsafe equality from vacuous non-comparison and binds the witness to native execution.

The required positioning sentence is:

> Prior interoperability frameworks verify conversion soundness relative to a semantic relation. We instead expose how a recent translation-validation workflow can admit semantically misaligned glue solely because it checks round trips, and provide a finite red-team and diagnostic realization for that validation seam.

The delta is therefore:

- the operational round-trip-admission attack;
- source/target program false agreement;
- comparator divergence diagnostics;
- harmful-twist generation in a declared structural family;
- soundness/adequacy and policy-vacuity diagnostics; and
- native replay and evidence binding.

It is not the invention of relation-based conversion soundness.

### 2.5 Prior-art and mechanism kill conditions

Before submission, the literature search MUST attempt to find work that jointly supplies the same attack, two-sided finite criterion, harmful-twist generation, native false-agreement replay, and diagnostic/evidence workflow.

BL4 parity is not a research kill condition. A complete direct relation checker should reach the same top-level verdict. If GlueRift shows no material improvement in derivation reuse, transformation generation, counterexample quality, or native evidence binding, the paper MUST delete any “new calculus” or mechanism-superiority claim. The Minimal Core makes no annotation-size or authoring-effort claim against BL4. The paper may still proceed as:

> A round-trip-preserving false-agreement attack on cross-language translation validation, with a two-sided finite checker and native replay.

### 2.6 Pre-submission disclosure record

Before submission, the authors MUST apply the [SCORED ’26 CFP responsible-disclosure rule](https://scored.dev/call_for_papers/) to the final paper claims. If the paper describes a new attack or design weakness affecting an identifiable implementation, vendor, or maintainer, the authors MUST make the required pre-deadline disclosure and record at least the date, recipient, disclosed claim scope, affected artifact or architecture, response status, and any mitigation. If the paper claims only an architecture-faithful reconstruction and no vulnerability in a public implementation, the authors MUST record that classification and its rationale rather than silently treating disclosure as inapplicable.

The paper’s disclosure statement MUST distinguish an architecture-level counterexample from a demonstrated vulnerability in a named implementation. This contract does not authorize the checker or an implementation agent to contact any external party; disclosure is a separate author-approved action.

---

## 3. Formal model

### 3.1 Finite semantic domains

Core semantic types are finite. Let:

\[
\mathcal S=(S,D_S),\qquad
\mathcal T=(T,D_T),\qquad
\mathcal C=(C,K)
\]

where the admitted domains are finite subsets of the representable types.

The adapter context is:

\[
e_S:S\to Result(C,E),\qquad d_S:C\to Result(S,E),
\]

\[
e_T:T\to Result(C,E),\qquad d_T:C\to Result(T,E).
\]

For mathematical readability, \(e_S(s)\downarrow c\) abbreviates \(e_S(s)=Ok(c)\), and \(\bind\) is ordinary `Result` bind.

Although the interpreter uses `Result` to make unexpected conversion failure explicit, Minimal Core adapters are required to succeed on every requested comparison and law scope. Meta-level `ConversionError` is not an allowed semantic result in Core. Object-language `Result<Ok,Err>` is an ordinary finite value type and is not conflated with meta-level conversion failure.

### 3.2 Policy-owned validation scope

The trusted `ValidationScope` declares:

\[
U\subseteq D_S^{cmp}\times D_T^{cmp}
\]

as the nonempty comparison universe, together with native, carrier-domain, and full-transport domains.

The candidate adapter cannot narrow any of these domains. Core domains use only canonical finite sets:

```text
DomainSpec ::=
    All
  | FiniteSet([TypedValue...])

PairDomainSpec ::=
    Product { source: DomainSpec, target: DomainSpec }
  | FinitePairSet([(SourceValue, TargetValue)...])
```

All lists are duplicate-free, type-checked, and canonically sorted. Rich predicate domains are Extended.

### 3.3 First-class comparator

The policy owner MUST select exactly one:

```text
ComparatorSpec ::=
    CarrierExact
  | TargetNativeExact
  | SourceNativeExact
```

There is no implicit default in the schema. Paper attack and native runs explicitly select `TargetNativeExact`.

Define the two partial native transports:

\[
F_A(s)=e_S(s)\bind d_T,
\qquad
G_A(t)=e_T(t)\bind d_S.
\]

The raw induced relations are:

\[
E_A^C(s,t)
\iff
\exists c.\ e_S(s)=Ok(c)\land e_T(t)=Ok(c),
\]

\[
E_A^T(s,t)
\iff
F_A(s)=Ok(t),
\]

\[
E_A^S(s,t)
\iff
G_A(t)=Ok(s).
\]

For selected comparator \(\chi\in\{C,T,S\}\), define the scoped induced relation:

\[
I_A^\chi=E_A^\chi\cap U.
\]

Every later comparison property binds \(\chi\). The document uses no unqualified \(E_A\) or “adapter equality.”

Important operational consequences are:

- `TargetNativeExact` does not inspect \(e_T(t)\) when deciding \(E_A^T(s,t)\);
- `SourceNativeExact` does not inspect \(e_S(s)\) when deciding \(E_A^S(s,t)\); and
- raw Protobuf byte equality is not `CarrierExact` unless canonical wire equality is separately established.

### 3.4 Comparator definedness

Core certification requires positive comparator coverage:

\[
Defined_C(A,U)
\]

holds iff both encoders return `Ok` for every value in the corresponding projections of \(U\);

\[
Defined_T(A,U)
\]

holds iff \(F_A(s)=Ok(t')\) for every \(s\in\pi_S(U)\); and

\[
Defined_S(A,U)
\]

holds iff \(G_A(t)=Ok(s')\) for every \(t\in\pi_T(U)\).

The transported result need not form a pair in \(U\); it is still reported, while \(I_A^\chi\) contains only in-universe pairs. A failed definedness obligation disproves the Core total-success prerequisite. A diagnostic run may enumerate the partial relation but receives no built-in certificate.

### 3.5 Native round trips

The source and target native round trips are:

\[
\forall s\in D_S^{rt}.\ e_S(s)\bind d_S=Ok(s),
\]

\[
\forall t\in D_T^{rt}.\ e_T(t)\bind d_T=Ok(t).
\]

**Lemma 1 — Encoder injectivity.** If a native round trip holds on a domain, the corresponding encoder is injective on that domain.

### 3.6 Carrier-domain round trips

For independently declared \(K_S,K_T\subseteq K\):

\[
\forall c\in K_S.\ d_S(c)\bind e_S=Ok(c),
\]

\[
\forall c\in K_T.\ d_T(c)\bind e_T=Ok(c).
\]

These are claims only over \(K_S\) and \(K_T\). They MUST NOT be silently extended to a source-encoded carrier outside \(K_T\) or a target-encoded carrier outside \(K_S\).

### 3.7 Full cross-language transport round trips

Define:

\[
RT_{STS}(s)=e_S(s)\bind d_T\bind e_T\bind d_S,
\]

\[
RT_{TST}(t)=e_T(t)\bind d_S\bind e_S\bind d_T.
\]

For policy-owned domains \(D_{STS}\) and \(D_{TST}\):

\[
\forall s\in D_{STS}.\ RT_{STS}(s)=Ok(s),
\]

\[
\forall t\in D_{TST}.\ RT_{TST}(t)=Ok(t).
\]

These are positive total-success obligations. They are not implications guarded by optional definedness. A Core comparison profile requires:

\[
\pi_S(U)\subseteq D_{STS},
\qquad
\pi_T(U)\subseteq D_{TST}.
\]

The checker reports every intermediate `Result` and the final equality separately. All six round trips may hold while the carrier and native comparators differ.

### 3.8 Trusted safety and required-match relations

The policy specification supplies:

\[
Safe\subseteq U
\]

where \(Safe(s,t)\) means that reporting \((s,t)\) equal is permitted by the declared security policy, and:

\[
Match\subseteq U
\]

where \(Match(s,t)\) means that the validator is required to recognize \((s,t)\) as corresponding.

Specification validity requires:

\[
Match\subseteq Safe.
\]

Neither relation may inspect the candidate carrier, any adapter result, \(E_A^\chi\), or the selected comparison verdict. The comparator may be policy-owned, but endpoint semantics cannot be defined circularly from its output.

`Match` is not automatically policy-label equality. Extra identity, role, constructor, or value observations may be needed when several values share one policy level.

### 3.9 Comparator-indexed comparison properties

For the selected comparator \(\chi\):

**Policy soundness**

\[
Sound_\chi(A)
\iff
I_A^\chi\subseteq Safe.
\]

**Comparison adequacy**

\[
Adequate_\chi(A)
\iff
Match\subseteq I_A^\chi.
\]

**Comparison precision**

\[
Precise_\chi(A)
\iff
I_A^\chi\subseteq Match.
\]

**Faithful comparison**

\[
Faithful_\chi(A)
\iff
I_A^\chi=Match.
\]

Thus:

\[
Faithful_\chi(A)
\iff
Adequate_\chi(A)\land Precise_\chi(A).
\]

Soundness and precision are intentionally distinct: an adapter may induce an equality that is safe but was not declared a required match.

For a verified finite preorder \((L,\preceq)\), target non-amplification is the special safety instance:

\[
Safe_{TNA}(s,t)
\iff
p_T(t)\preceq p_S(s).
\]

It is checked over the selected \(I_A^\chi\), not over carrier equality by default.

A target-non-amplification request names a nonempty, duplicate-free list of active policy dimensions whose `safe_relation` is `TargetNoAmplification`. The per-dimension obligation is evaluated with that dimension’s source observer, target observer, and finite preorder. The aggregate status is `proved-exhaustive` exactly when every named dimension is `proved-exhaustive`, `disproved` when any named dimension is `disproved`, and otherwise follows the ordinary invalid/unknown/tool-error precedence. An empty list, a duplicate, an inactive dimension, or a dimension with another relation kind makes the request invalid. Thus a single undifferentiated TNA result never silently combines incompatible policy dimensions.

### 3.10 Match coverage and selected-relation vacuity

The policy declares one of:

```text
none | nonempty | source-total | target-total | bidirectional-total
```

Their meanings are:

\[
Nonempty(Match)
\iff
Match\neq\varnothing,
\]

\[
SourceTotal(Match)
\iff
\forall s\in D_S^{cmp}.\
\exists t\in D_T^{cmp}.\
(s,t)\in Match,
\]

\[
TargetTotal(Match)
\iff
\forall t\in D_T^{cmp}.\
\exists s\in D_S^{cmp}.\
(s,t)\in Match,
\]

\[
BidirectionalTotal(Match)
\iff
SourceTotal(Match)\land TargetTotal(Match).
\]

`none` imposes no Match coverage obligation and does **not** assert that \(Match\) is empty. It is permitted only when adequacy, precision, and faithfulness are not requested. Its status is `not-requested`. `nonempty`, `source-total`, `target-total`, and `bidirectional-total` require the corresponding formula above and are checked exhaustively. A satisfied formula is `proved-exhaustive`; a counterexample is `disproved`. Invalid domains or relations are `invalid`, and a resource or tool failure is `tool-error`. Core has no incomplete coverage procedure and therefore emits no `unknown` for a valid finite coverage request.

Any requested adequacy, precision, or faithfulness requires nonempty match dimensions and a coverage mode other than `none`. If a non-`none` coverage obligation is `disproved`, the endpoint policy and validation request are ineligible for candidate-property verdicts and certification: candidate comparison properties are reported `invalid`, not evaluated under a silently weakened policy. Canonical unmatched endpoint witnesses identify the failed quantifier.

Coverage is a policy well-formedness check performed before candidate evaluation. It ranges over the complete independently declared comparison domains, not \(\pi_S(U)\), \(\pi_T(U)\), or the values already appearing in Match. Because \(Match\subseteq U\), a policy owner selecting a total mode must declare \(U\) broadly enough to realize that totality; a sparse universe cannot silently weaken the claim.

If \(I_A^\chi=\varnothing\), soundness is vacuously true. If \(Match\neq\varnothing\), adequacy is false. Therefore an always-different adapter cannot receive a two-sided certificate.

Disjoint encoder images imply \(E_A^C=\varnothing\), but do **not** imply \(E_A^T=\varnothing\) or \(E_A^S=\varnothing\).

### 3.11 Safety-policy vacuity

The report MUST contain:

```text
safe_dimension_count
safe_pair_count
unsafe_pair_count
safe_is_universal
policy_contract_status
policy_vacuity_warning
```

where:

\[
safe\_pair\_count=|Safe|,
\qquad
unsafe\_pair\_count=|U\setminus Safe|,
\]

and, because Core requires \(U\neq\varnothing\):

\[
safe\_is\_universal\iff unsafe\_pair\_count=0.
\]

The deterministic classification is:

| Condition | `policy_contract_status` | Built-in security certification |
|---|---|---|
| `safe_dimensions=[]` | `policy-unconstrained` | forbidden |
| nonempty dimensions and \(Safe=U\) | `universal-declared` | property may be reported, but a vacuity warning is mandatory |
| nonempty dimensions and \(Safe\neq U\) | `constrained` | eligible if all other requirements pass |

An empty safety-dimension list constructs \(Safe=U\) for diagnostic evaluation, but the run MUST say `policy-unconstrained`, MUST set `policy_vacuity_warning=true`, and MUST NOT be cited as evidence that unsafe false agreement was excluded.

A nonempty policy may intentionally declare every pair safe. That is not a malformed specification, because policy truth is trusted. It still sets `safe_is_universal=true` and a warning. A paper claim that the checker excluded an unsafe pair additionally requires `unsafe_pair_count>0`.

BL4 follows exactly the same classification.

### 3.12 Comparator-specific match-shape compatibility

“Functional” means that each source has at most one related target. “Inverse-functional” means that each target has at most one related source.

Under adequacy, `Match` inherits only shape constraints established for the selected relation and requested law bundle:

1. **CarrierExact.** `Match` is functional if target native round trip establishes injectivity of \(e_T\) on \(\pi_T(Match)\). It is inverse-functional if source native round trip establishes injectivity of \(e_S\) on \(\pi_S(Match)\).
2. **TargetNativeExact.** \(E_A^T\), and therefore `Match`, is functional because \(E_A^T\) is the graph of deterministic partial \(F_A\). `Match` is inverse-functional when source full transport:

   \[
   F_A(s)\bind G_A=Ok(s)
   \]

   holds for every \(s\in\pi_S(Match)\). This conclusion is about `Match`, or equivalently about \(E_A^T\) restricted to that covered source set; it does not make the unrestricted \(E_A^T\) inverse-functional.
3. **SourceNativeExact.** \(E_A^S\), and therefore `Match`, is inverse-functional because \(E_A^S\) is the converse graph of deterministic partial \(G_A\). `Match` is functional when target full transport:

   \[
   G_A(t)\bind F_A=Ok(t)
   \]

   holds for every \(t\in\pi_T(Match)\). This conclusion is about `Match`, or equivalently about \(E_A^S\) restricted to that covered target set.

The checker derives necessary checks from the tuple:

```text
(comparator_kind, requested_laws, law_coverage, requested_match_coverage)
```

and reports `match_shape_compatibility`. Passing this check is necessary under the stated profile; it is not a general synthesis or existence theorem.

### 3.13 Certification profiles

Minimal Core defines:

```text
diagnostic
policy-sound
policy-sound-adequate
faithful-exact
```

The profiles and their minimum property bundles are normative:

| `profile` | Minimum `required_properties` | Additional requirements | Certificate eligibility |
|---|---|---|---|
| `diagnostic` | none | no positive certificate | never |
| `policy-sound` | `PolicySoundness` | none beyond the common checks below | if the common eligibility prerequisites hold |
| `policy-sound-adequate` | `PolicySoundness`, `ComparisonAdequacy` | nonempty match dimensions and non-`none` proved Match coverage | if the common eligibility prerequisites hold |
| `faithful-exact` | `FaithfulComparison` | \(Safe=Match\), nonempty match dimensions, and non-`none` proved Match coverage | if the common eligibility prerequisites hold |

`required_properties` MUST explicitly contain a profile’s minimum bundle. The checker never inserts missing properties, silently upgrades a profile, or treats a profile name as an implicit law request. Extra supported properties are allowed, remain explicit, and must pass for the run certificate. Duplicate property kinds are invalid as specified in §7.6.

All positive built-in certifications require:

- a valid, policy-owned `ValidationScope`;
- nonempty \(U\);
- selected-comparator definedness;
- every round-trip law explicitly selected in `required_laws`;
- complete finite enumeration;
- supported observers and relations;
- every requested active safety/matching anchor coverage obligation;
- a non-`policy-unconstrained` policy;
- a result other than `unknown`; and
- a canonical report bound to the candidate, policy, comparator, scope, request, and tool build.

`PolicySoundness`, `ComparisonAdequacy`, `ComparisonPrecision`, and `FaithfulComparison` retain the definitions in §3.9; the profile table only constrains which obligations must be requested and certified. A `faithful-exact` request may explicitly add soundness, adequacy, or precision for diagnostics, but faithfulness together with \(Safe=Match\) already entails their set-theoretic conclusions. No profile claims program equivalence or correctness of the endpoint policy.

`certification.eligible` and `certification.granted` are distinct. `eligible=true` exactly when the profile is not `diagnostic`, the request and policy pass all profile/coverage/typing checks, the policy is not `policy-unconstrained`, comparator definedness and every explicitly required law are proved, complete supported evaluation is available for every explicitly requested property, and all canonical evidence bindings exist. A requested property may be `disproved` without making the run ineligible. `granted=true` exactly when `eligible=true` and every explicitly requested property is `proved-exhaustive`; otherwise it is false. Thus attack runs can be eligible but correctly denied a certificate. `diagnostic` always sets both fields false.

`safe_match_equality_status` is checked only for `faithful-exact`: equality is `proved-exhaustive` or `disproved` over \(U\), with a disproof making profile consistency `invalid`. Its canonical `safe-match-divergence` witness is the first pair in the symmetric difference \(Safe\triangle Match\), together with membership bits for both relations. Other profiles report the status and witness as `not-requested`. The report’s `blocking_reasons` is a duplicate-free, canonically ordered list covering every failed eligibility prerequisite and every disproved required property.

---

## 4. Comparator bridges

### 4.1 Bridge definitions

For the finite universe \(U\):

\[
Bridge_{C,T}(A,U)
\iff
\forall(s,t)\in U.\ E_A^C(s,t)\leftrightarrow E_A^T(s,t),
\]

\[
Bridge_{C,S}(A,U)
\iff
\forall(s,t)\in U.\ E_A^C(s,t)\leftrightarrow E_A^S(s,t).
\]

Each requested bridge is reported as:

```text
proved-exhaustive | disproved | unknown | not-requested
```

A disproved bridge includes the first canonical pair and comparator traces for both sides.

Every top-level registry-owned Core check using `TargetNativeExact` MUST evaluate `carrier_target_bridge`; every such check using `SourceNativeExact` MUST evaluate `carrier_source_bridge`. The opposite native bridge may be `not-requested`. This top-level diagnostic rule does not turn each internally generated transformation candidate into a separate mandatory bridge run: a transformed-context bridge is required only when the request explicitly selects it or carrier-derived evidence is claimed to apply to that transformed native comparator. A `CarrierExact` run evaluates either bridge only when the request or fixture registry selects it; V01 explicitly selects `carrier_target_bridge`. Because the supported Core is finite, a selected bridge must be `proved-exhaustive` or `disproved`; `unknown` is reserved for an unsupported Extended request, while incomplete enumeration is a tool error.

### 4.2 Sufficient pointwise bridge rules

For \((s,t)\in U\):

- \(E_A^C(s,t)\Rightarrow E_A^T(s,t)\) follows from target native round trip at \(t\).
- \(E_A^T(s,t)\Rightarrow E_A^C(s,t)\) follows from target carrier round trip at the \(c\) such that \(e_S(s)=Ok(c)\), provided that exact carrier is inside the proved carrier domain.
- \(E_A^C(s,t)\Rightarrow E_A^S(s,t)\) follows from source native round trip at \(s\).
- \(E_A^S(s,t)\Rightarrow E_A^C(s,t)\) follows from source carrier round trip at the \(c\) such that \(e_T(t)=Ok(c)\), provided that exact carrier is inside the proved carrier domain.

These are sufficient proof rules with explicit coverage, not model substitutions. The six round-trip laws alone do not prove either bridge.

### 4.3 Effect of bridge status

In `TargetNativeExact`, GlueRift directly checks \(I_A^T\). A disproved or unknown `carrier_target_bridge` does not weaken, invalidate, or make unknown a direct target-native soundness or adequacy verdict.

A carrier-class summary or carrier stabilizer may support a selected native-comparator conclusion only when the relevant bridge is `proved-exhaustive`. Otherwise it is labeled `explanatory-only`.

Every bridge report is bound to the exact adapter-context hash. Evidence for \(A\) MUST NOT be reused for a transformed \(A^\sigma\); transfer of a carrier summary or stabilizer result about \(A^\sigma\) requires a separately checked \(Bridge_{C,\chi}(A^\sigma,U)\) whose report binds `transformed_context_sha256`.

In `CarrierExact`, carrier analysis directly applies. `SourceNativeExact` is handled symmetrically.

### 4.4 Mandatory comparator-divergence model

V01 uses:

\[
S=\{s_0,s_1\},\quad T=\{t_0,t_1\},
\]

\[
C=\{L_0,L_1,R_0,R_1\}.
\]

Define:

\[
e_S(s_i)=L_i,\qquad e_T(t_i)=R_i,
\]

\[
d_S(L_i)=d_S(R_i)=s_i,\qquad
d_T(L_i)=d_T(R_i)=t_i,
\]

with:

\[
K_S=\{L_0,L_1\},\qquad K_T=\{R_0,R_1\}.
\]

Freeze all remaining scopes as:

\[
D_S=D_S^{rt}=D_{STS}=S,
\qquad
D_T=D_T^{rt}=D_{TST}=T,
\]

\[
D_S^{cmp}=S,\qquad D_T^{cmp}=T,\qquad U=S\times T.
\]

The policy uses nonempty safety dimensions whose finite table is:

\[
Safe=\{(s_0,t_1),(s_1,t_0)\},
\]

and Match properties are not requested.

All six round trips pass. Yet:

\[
E_A^C=\varnothing
\]

while:

\[
E_A^T=E_A^S=\{(s_0,t_0),(s_1,t_1)\}.
\]

This fixture proves construct divergence without making \(Safe\) universal. It also demonstrates why a bridge cannot be inferred from full transport alone.

---

## 5. Round-trip-preserving policy laundering

### 5.1 Clean shared-domain core

For the mechanized attack theorem, assume total bijections:

\[
e_S:S\cong C,\qquad e_T:T\cong C,
\]

with inverses \(d_S\) and \(d_T\), and a carrier automorphism:

\[
\sigma:C\cong C.
\]

Define the twisted target adapter:

\[
e_T^\sigma=\sigma\circ e_T,
\qquad
d_T^\sigma=d_T\circ\sigma^{-1}.
\]

The source maps are unchanged. Let \(A^\sigma\) denote the resulting four-map context.

### 5.2 Round-trip preservation

**Theorem 2 — Target native and carrier preservation.** The twist preserves target native round trip and total target carrier round trip.

**Theorem 3 — Full transport preservation.** Under the clean total shared-domain assumptions, the twist preserves both \(RT_{STS}\) and \(RT_{TST}\).

The theorem is algebraic and is not claimed as novel. Its purpose is to connect a known ambiguity to the operational comparison seam.

### 5.3 Direct comparator-indexed witness

For any \(c\in C\), let:

\[
s=d_S(\sigma(c)),
\qquad
t=d_T(c).
\]

Then the pair lies simultaneously in all three induced relations for the twisted context:

\[
E_{A^\sigma}^C(s,t),
\qquad
E_{A^\sigma}^T(s,t),
\qquad
E_{A^\sigma}^S(s,t).
\]

In particular:

\[
d_T^\sigma(e_S(s))
=d_T(\sigma^{-1}(\sigma(c)))
=d_T(c)
=t.
\]

Thus the load-bearing target-native attack is direct; it does not depend on a bridge inferred from operational laws.

### 5.4 Policy-laundering definition

For selected comparator \(\chi\), a transformation is a **policy-laundering twist** when:

\[
\exists(s,t)\in U.\
E_{A^\sigma}^\chi(s,t)\land\neg Safe(s,t).
\]

The twist is **lawful** relative to a validation request \(\mathcal R\) exactly when:

\[
\sigma\in Lawful_{\chi,\mathcal F}(A,\mathcal R)
\]

under the complete definition in §6.3. A **lawful policy-laundering twist** is therefore a member of:

\[
HarmfulTrans_{\chi,\mathcal F}(A,\mathcal R).
\]

A raw unsafe equality from an ill-typed, comparator-undefined, inverse-invalid, incorrectly constructed, or request-law-breaking candidate is not a lawful policy-laundering result.

**Corollary 4 — Lawful laundering.** Under the clean assumptions, fix one comparator \(\chi\) and one valid Core request \(\mathcal R_\chi\) selecting it, with a valid policy and scope \(U\). If the pair constructed in §5.3 satisfies:

\[
(s,t)\in U\land(s,t)\notin Safe,
\]

then, provided the request’s hashed family descriptor admits the normalized \(\sigma\), together with its reported normalized inverse, as a member of \(\Sigma_{\mathcal F}(A)\), derives the complete carrier action domain \(D_\sigma=C\), the transformed context is mechanically constructed by the stated conjugation, and every law required by \(\mathcal R_\chi\) is among the six laws proved under the clean assumptions,

\[
\sigma\in HarmfulTrans_{\chi,\mathcal F}(A,\mathcal R_\chi).
\]

Because §5.3 supplies the same unsafe pair in all three induced relations, this corollary may be instantiated separately for `CarrierExact`, `TargetNativeExact`, and `SourceNativeExact` using three comparator-specific valid requests. It does not treat one request as selecting three comparators.

The paper’s operational evidence uses `TargetNativeExact`.

### 5.5 Scoped identifiability statement

An anchor-free validator property that is invariant under every declared law-preserving twist cannot distinguish an aligned adapter from all harmful twists in that family. This is a scoped indistinguishability lemma, not a claim that endpoint semantics are impossible to specify or that every lens law is inadequate for every purpose.

---

## 6. Residual transformations and carrier diagnostics

### 6.1 Declared Core transformation family

Core automatically enumerates only:

- payload-compatible finite enum or sum permutations;
- same-typed product-field permutations;
- object-language `Result` branch permutations when branch types are compatible; and
- nested combinations of those structural transformations.

`BoundedComplement` and `ModularAffine` are checked when declared in a candidate, but automatic scalar-template discovery is Extended.

The tool MUST report:

```text
exact_within_core_structural_family
unknown_outside_declared_family
```

It MUST NOT claim to enumerate every semantic automorphism.

For a base context \(A\), let:

\[
\Sigma_{\mathcal F}(A)
\]

be the finite, normalized candidate set produced or admitted by the hashed transformation-family descriptor. \(\Sigma_{\mathcal F}(A)\) is not called a group unless identity, closure, associativity, and inverses have been established for the relevant subfamily. This notation owns the operational three-way classification in §6.3.

### 6.2 Exact stabilizer

For an exact carrier observer \(\lambda:C\to L\) and a separately proved finite structural group:

\[
G_{\mathcal F}(A)\subseteq\Sigma_{\mathcal F}(A),
\]

\[
Stab(\lambda)=
\{\sigma\in G_{\mathcal F}(A)\mid\lambda\circ\sigma=\lambda\}.
\]

**Theorem 5 — Exact stabilizer.** \(Stab(\lambda)\) is a subgroup of \(G_{\mathcal F}(A)\).

This theorem applies to exact equality observations. It is not generalized to asymmetric safety.

### 6.3 Comparator-indexed lawful transformation partition

Let \(\mathcal R\) be the hashed `ValidationRequest`, and let:

\[
Laws(\mathcal R)
\]

be exactly the subset of the six law flags set to `true` in that request. Let \(D_\sigma\) be the complete finite semantic domain of \(\sigma\)’s carrier input type, canonically derived by the hashed family descriptor. The candidate and request cannot author, narrow, or replace it. In Minimal Core every admitted transformation is a total endomorphism on that complete domain; a path-local structural generator is lifted to a total carrier transformation before admission into \(\Sigma_{\mathcal F}(A)\).

`InverseOK` means that the transformation and reported inverse are well typed, total on this complete \(D_\sigma\), preserve \(D_\sigma\), and satisfy both identities exhaustively:

\[
\forall c\in D_\sigma.\
\sigma^{-1}(\sigma(c))=c
\land
\sigma(\sigma^{-1}(c))=c.
\]

`ConstructedByConjugation` means that the transformed context was produced mechanically on the target side:

\[
e_S^\sigma=e_S,\qquad d_S^\sigma=d_S,
\]

\[
e_T^\sigma=\sigma\circ e_T,\qquad
d_T^\sigma=d_T\circ\sigma^{-1}.
\]

Define:

\[
\begin{aligned}
Lawful_{\chi,\mathcal F}(A,\mathcal R)
=
\{\sigma\in \Sigma_{\mathcal F}(A)\mid{}&
WellTyped(A^\sigma)\\
&{}\land InverseOK(\sigma,\sigma^{-1},D_\sigma)\\
&{}\land ConstructedByConjugation(A,A^\sigma,\sigma)\\
&{}\land Defined_\chi(A^\sigma,U)\\
&{}\land \bigwedge_{\ell\in Laws(\mathcal R)}
Holds_\ell(A^\sigma)\}.
\end{aligned}
\]

Every \(Holds_\ell\) is evaluated over the complete policy-owned domain of that requested law.

The three classification sets are:

\[
SafeTrans_{\chi,\mathcal F}(A,\mathcal R)
=
\{\sigma\in Lawful_{\chi,\mathcal F}(A,\mathcal R)
\mid Sound_\chi(A^\sigma)\},
\]

\[
HarmfulTrans_{\chi,\mathcal F}(A,\mathcal R)
=
\{\sigma\in Lawful_{\chi,\mathcal F}(A,\mathcal R)
\mid \neg Sound_\chi(A^\sigma)\},
\]

\[
InapplicableTrans_{\chi,\mathcal F}(A,\mathcal R)
=
\Sigma_{\mathcal F}(A)\setminus
Lawful_{\chi,\mathcal F}(A,\mathcal R).
\]

For a valid, completely evaluated Core transformation request:

\[
Lawful_{\chi,\mathcal F}
=
SafeTrans_{\chi,\mathcal F}
\mathbin{\dot\cup}
HarmfulTrans_{\chi,\mathcal F},
\]

\[
\Sigma_{\mathcal F}(A)
=
SafeTrans_{\chi,\mathcal F}
\mathbin{\dot\cup}
HarmfulTrans_{\chi,\mathcal F}
\mathbin{\dot\cup}
InapplicableTrans_{\chi,\mathcal F}.
\]

Thus the three sets are a disjoint and exhaustive partition of \(\Sigma_{\mathcal F}(A)\). Their serialized classifications are:

```text
lawful-safe
lawful-harmful
law-breaking-or-inapplicable
```

Classification follows this table:

| Lawfulness | Soundness | Classification |
|---|---|---|
| proved | proved | `lawful-safe` |
| proved | disproved | `lawful-harmful` |
| disproved | not evaluated as a class condition | `law-breaking-or-inapplicable` |
| unknown or tool error | not coercible | no classification; propagate status |

Stable inapplicability reasons are:

```text
ill-typed
inverse-invalid
conjugation-construction-invalid
comparator-undefined
required-law-disproved
```

Multiple reasons are retained in this canonical order. A law-breaking transformation is never called a harmful policy-laundering twist, even if its selected induced relation is unsafe.

`lawful-safe` means only that the transformation is lawful for the request and policy-sound for the selected comparator. It does not imply adequacy, precision, faithfulness, target non-amplification on an unselected dimension, or certification; those statuses remain separate.
A `policy-unconstrained` run may serialize this classification only as a vacuity-marked diagnostic and may not cite `lawful-safe` as security evidence.

The checker recomputes the selected \(E_{A^\sigma}^\chi\) after twisting both the target encoder and decoder. It MUST NOT approximate a target-native verdict by inspecting only the target encoder, and `ConstructedByConjugation` is a provenance/lawfulness check rather than a carrier/native bridge.

Under an asymmetric relation, `SafeTrans` is merely a finite set. The required non-closure counterexample uses one clean total/bijective adapter context in which every carrier permutation is well typed, comparator-defined, and passes every requested law. It establishes:

```text
sigma1                  lawful-safe
sigma2                  lawful-safe
normalize(sigma1∘sigma2) lawful-harmful
```

Concretely, let \(S=T=C=\{a,b,c\}\), let all four base maps be identity, use every complete domain and \(U=S\times T\), select `TargetNativeExact`, and explicitly request all six laws. Give the source policy levels on \((a,b,c)\) as `deny, allow, allow` and the target levels as `deny, deny, allow`, with `deny` below `allow`. Let \(Safe(s,t)\) mean target non-amplification. For \(\sigma_1=(a\ b)\) and \(\sigma_2=(b\ c)\), the target-native graph pairs each target \(t\) with source \(\sigma(t)\). Each transformation is sound. Under right-to-left composition, \((\sigma_1\circ\sigma_2)(c)=a\), so the target-`allow` position \(c\) is paired with the source-`deny` position \(a\). Every carrier permutation remains well typed, inverse-valid, comparator-defined, conjugated, and passing on all six complete domains; the composite becomes harmful only because of asymmetric policy soundness. A composite that merely breaks typing, definedness, inverse validity, conjugation, or a requested law is not a non-closure witness.

No subgroup, orbit-stabilizer, or coset claim is permitted for `SafeTrans`, `HarmfulTrans`, or `InapplicableTrans`.

### 6.4 Carrier summaries

Core may derive:

- successful source and target carrier images;
- shared carrier classes for `CarrierExact`;
- the endpoint pairs induced by each shared class;
- exact observer-label conflicts; and
- target-non-amplification violations on those carrier-induced pairs.

These summaries are derived diagnostics, never trusted carrier semantics. Each summary reports:

```text
evidence_basis =
    carrier-exact
  | selected-via-proved-bridge
  | explanatory-only
applicability_to_selected_comparator
bridge_report_sha256
```

When the selected comparator is native and the relevant bridge is not proved, carrier summaries remain explanatory and cannot discharge top-level soundness, adequacy, precision, faithfulness, or transformation safety.

The Minimal Core makes no deep synthesis claim for lattice intervals, standardized carrier observers, or global carrier semantics.

---

## 7. Minimal closed specification language

### 7.1 Core Type IR

Core exposes exactly:

```text
Type ::=
    Unit
  | Bool
  | BoundedInt { min, max }
  | BitVec { width }
  | Sum {
      variants: [
        { name, payload: Type },
        ...
      ]
    }
  | Product {
      fields: [
        { name, type: Type },
        ...
      ]
    }
  | ObjectResult { ok: Type, err: Type }
```

`ObjectResult` models an endpoint value such as Rust `Result` or a corresponding Go tagged union. Its `Err` constructor is an ordinary semantic branch. It is never interpreted as the checker’s meta-level `ConversionError`.

All types are finite. Bounds and bit widths are explicit and validated against implementation resource limits.

`Option`, lists, recursive types, unbounded integers, floating point, strings beyond finite enumerations, opaque callbacks, and library-native values not represented by this IR are Extended.

### 7.2 Core Adapter IR

Each candidate supplies all four typed expressions \(e_S,d_S,e_T,d_T\) using:

```text
Adapter ::=
    Identity
  | Compose { first: Adapter, second: Adapter }
  | EnumPermutation { mapping }
  | FieldPermutation { mapping }
  | SumMap { variants }
  | ProductMap { fields }
  | ResultMap {
      branch_mapping: preserve | swap,
      ok: Adapter,
      err: Adapter
    }
  | BoundedComplement { min, max }
  | ModularAffine { width, scale, offset }
```

The semantics are:

- `Identity` returns the input.
- `Compose` uses total-success `Result` bind and preserves the full child path in diagnostics.
- `EnumPermutation` is a total bijection over a closed payload-free enum.
- `FieldPermutation` is a total bijection over fields with exactly compatible types.
- `SumMap` is exhaustive over source constructors and maps payloads with typed child adapters.
- `ProductMap` supplies one typed child map for every output field.
- `ResultMap` maps both object-language branches; `swap` is legal only when the mapped payload types agree.
- `BoundedComplement` maps \(x\) to \(min+max-x\) over the exact declared bounded domain.
- `ModularAffine` maps a width-\(w\) bit vector by:

  \[
  x\mapsto ax+b\pmod{2^w}.
  \]

  A requested invertibility or round-trip proof requires \(\gcd(a,2^w)=1\).

A well-typed Core primitive is total on its declared input type. Any emitted meta-level `Err` is an interpreter or candidate-conformance failure and disproves every requested property whose scope reaches that execution.

`Restrict`, `CheckedCast`, general rational or unit affine conversion, bit permutation, list mapping, code emission, and arbitrary callbacks are not Core nodes.

Core `TransformationIR` is not a separate or underspecified callback language. It is a normalized `Adapter` term from carrier type \(C\) to the same \(C\), evaluated by the same reference Adapter evaluator and total-success `Result` semantics. `inverse_ir` is another normalized \(C\to C\) Adapter term. The hashed transformation-family descriptor restricts which Adapter nodes, generator paths, composition order, and normal forms may appear; it does not define a second evaluator.

For every admitted transformation, `action_domain` is the canonical exhaustive resolution of `DomainSpec::All` for \(C\), including its ordered values and hash. Neither a candidate nor a request may supply a finite subset. The two TransformationIR terms must be total mutually inverse endomorphisms on that complete domain before the twist is lawful.

### 7.3 Core Observer IR

Endpoint semantics are expressed only by:

```text
Observer ::=
    ConstructorRole {
      path,
      table: constructor -> canonical_role
    }
  | FieldRole {
      roles: [
        { role, path, inner: Observer },
        ...
      ]
    }
  | FinitePolicyMap {
      path,
      table: reachable_value -> policy_atom
    }
  | Tuple([Observer...])
  | Case {
      scrutinee_path,
      branches: constructor -> Observer
    }
```

Rules:

- every path is endpoint-local and statically typed;
- every table is total over the values reachable in its declared domain;
- `FieldRole` returns a canonically role-keyed tuple, not a physical-field-order tuple;
- `Case` lists each reachable constructor exactly once;
- observers cannot inspect carrier values, adapter results, comparator traces, or candidate paths; and
- observer evaluation is total on its declared endpoint domain.

The schema recognizes:

```text
ExternalObserverRef { id }
```

only as an unsupported marker. A property depending on it reports:

```text
status = unknown
reason = unsupported-observer
```

and receives no certificate. It is used only by V06 in Core.

Generic executable callbacks are forbidden. Rich arithmetic, units, global comparisons, joins, failure classes, generic field identity, and list observations are Extended.

### 7.4 Core Relation IR

Core relation expressions are:

```text
Relation ::=
    Exact
  | TargetNoAmplification {
      elements,
      preorder_edges
    }
  | FiniteTable {
      left_codomain,
      right_codomain,
      allowed_pairs
    }
```

`Exact` compares identical canonical observer codomains.

Before using `TargetNoAmplification` composition, the checker exhaustively validates that the finite relation is reflexive and transitive. Antisymmetry or lattice operations are not required for the Core theorem.

`FiniteTable` is checked directly over its finite declared codomains. It receives no general composition, subgroup, lattice, or synthesis theorem.

Multiple policy dimensions combine by conjunction after type checking. A tuple observer provides the common canonical representation needed for multi-component exact comparison.

### 7.5 Endpoint policy

The Core policy schema is:

```text
EndpointPolicy {
  schema = "gluerift.policy/v0.3.1a"

  match_coverage:
      none | nonempty | source-total | target-total | bidirectional-total

  dimensions: [
    {
      id
      source_codomain
      target_codomain
      source_observer: Observer
      target_observer: Observer
      safe_relation: Relation
      match_relation: Relation | omitted
    }
  ]

  safe_dimensions: [dimension_id...]
  match_dimensions: [dimension_id...]

  explicitly_irrelevant_paths: [
    {
      endpoint: source | target
      path
      applies_to: safety | matching | both
      justification
    }
  ]
}
```

Construction is deterministic:

- each ID in `safe_dimensions` contributes its `safe_relation`;
- each ID in `match_dimensions` contributes a present `match_relation`;
- `safe_dimensions=[]` constructs \(Safe=U\) but triggers §3.11;
- `match_dimensions=[]` constructs \(Match=\varnothing\), not a universal empty conjunction;
- any request for adequacy, precision, or faithfulness requires nonempty match dimensions and declared coverage; and
- the checker exhaustively proves \(Match\subseteq Safe\) before evaluating a candidate.

Each observer output is checked against its endpoint-specific codomain. `Exact` and `TargetNoAmplification` require compatible common canonical codomains as specified by their relation; `FiniteTable` is typed by the declared source and target codomains and may relate different finite sets.

Coverage is checked separately. These obligations are conservative
reachable-path coverage over the normalized IR/observer syntax, not all
syntactic paths in arbitrary source programs:

- the quantified meaning and status of `match_coverage` are exactly those in §3.10; neither an implementation nor a fixture may redefine totality using a projection of \(U\), the current Match pairs, or another inferred domain;

- `safe_anchor_coverage` counts only dimensions named in `safe_dimensions`; every reachable constructor or field that can affect an adapter output or selected-comparator result must be observed by one of those active dimensions or marked irrelevant for `safety`/`both`;
- `match_anchor_coverage` counts only dimensions named in `match_dimensions`; when adequacy, precision, or faithfulness is requested, every such path must be observed by an active match dimension or marked irrelevant for `matching`/`both`; and
- inactive dimensions never satisfy either coverage obligation.

Observer-internal paths are checked against the active dimension that owns them. Irrelevance is a trusted policy assertion; GlueRift does not infer semantic noninterference for arbitrary native code. Empty safety dimensions still trigger `policy-unconstrained` even if every path is marked safety-irrelevant.

### 7.6 ValidationScope and request schemas

The policy-owned scope is:

```text
ValidationScope {
  schema = "gluerift.validation-scope/v0.3.1a"
  source_domain: DomainSpec
  target_domain: DomainSpec
  source_comparison_domain: DomainSpec
  target_comparison_domain: DomainSpec
  comparison_universe: PairDomainSpec
  source_carrier_domain: DomainSpec
  target_carrier_domain: DomainSpec
  source_full_transport_domain: DomainSpec
  target_full_transport_domain: DomainSpec
  comparator: ComparatorSpec
}
```

The normalized comparator is included in:

```text
validation_scope_sha256
comparator_spec_sha256
```

The validation request is:

```text
ValidationRequest {
  schema = "gluerift.validation-request/v0.3.1a"
  request_id
  profile
  validation_scope_sha256
  endpoint_policy_sha256
  run_configuration_sha256
  required_laws: {
    source_native_roundtrip
    target_native_roundtrip
    source_carrier_roundtrip
    target_carrier_roundtrip
    source_full_transport
    target_full_transport
  }
  required_properties: [PropertyRequest...]
  required_bridges
  required_transformation_family_sha256
}
```

where:

```text
PropertyRequest ::=
    PolicySoundness
  | ComparisonAdequacy
  | ComparisonPrecision
  | FaithfulComparison
  | TargetNonAmplification {
      dimension_ids: [active_safe_dimension_id...]
    }
```

`required_properties` is duplicate-free by property kind. At most one `TargetNonAmplification` request is permitted, and it contains the single combined dimension list whose per-dimension and aggregate results appear in §17.1. Multiple TNA requests with overlapping or different lists are invalid rather than merged implicitly.

The checker validates `profile_property_consistency` against the normative table in §3.13. Missing minimum properties, a forbidden `none` Match-coverage mode, a failed required coverage formula, or a `faithful-exact` policy with \(Safe\neq Match\) makes the request `invalid`. Extra supported properties are evaluated as written and become certificate obligations; they do not change the profile name. Profiles add no implicit round-trip laws.
For a valid finite request the consistency status is `proved-exhaustive`; any listed mismatch is `invalid`. It is never a semantic `disproved` verdict about an adapter.

Every A01, A02, A03, and A05 request and registry row MUST explicitly set all six members of `required_laws` to required. Each E01/E02 native manifest binds that same six-law validation request through its corresponding A01/A02 registry row. The fixture registry and native reference binding are checked against the normalized request; a mismatch is `tool-error`, not a semantic verdict.

`required_bridges` names bridge checks that MUST be executed and reported; it does not mean their status must be `proved-exhaustive` for a direct native-comparator certificate. A claim that transfers carrier evidence to a native comparator separately requires the relevant proved bridge through §§4.3 and 20.3.

The candidate has no field that can override this request or scope.

### 7.7 Type and specification checking

Before semantic checking, GlueRift validates:

- all four adapter direction types;
- totality and bijectivity conditions of permutations;
- field, constructor, payload, bound, and width compatibility;
- exact modular arithmetic;
- finite domain membership and nonempty \(U\);
- comparator type correctness;
- observer totality and relation codomains;
- dimension references;
- \(Match\subseteq Safe\);
- requested Match coverage;
- profile/property consistency and certificate eligibility;
- comparator-relative match-shape compatibility;
- separate active safety/matching anchor coverage or scoped explicit irrelevance; and
- all cross-file hashes.

A malformed specification is `invalid`, not a counterexample to a semantic property.

---

## 8. Total-success composition

### 8.1 Closed value judgment

The sole Core composition judgment is:

\[
\Gamma\vdash
f:(X,D_X,o_X)\xrightarrow{R}(Y,D_Y,o_Y).
\]

It means:

1. \(f\) is well typed;
2. for every \(x\in D_X\), evaluation returns some \(Ok(y)\) with \(y\in D_Y\); and
3. \(R(o_X(x),o_Y(y))\) holds.

No allowed meta-level error case exists in this judgment.

### 8.2 Composition theorem

**Theorem 6 — Total-success composition.** Assume:

\[
\Gamma\vdash
f:(X,D_X,o_X)\xrightarrow{R_1}(Y,D_Y,o_Y),
\]

\[
\Gamma\vdash
g:(Y,D_Y,o_Y)\xrightarrow{R_2}(Z,D_Z,o_Z),
\]

every successful \(f\) result is inside the checked input domain of \(g\), and:

\[
R_1;R_2\subseteq R.
\]

Then:

\[
\Gamma\vdash
f\mathbin{>=>}g:
(X,D_X,o_X)\xrightarrow{R}(Z,D_Z,o_Z).
\]

Core synthesizes this rule only for:

- exact equality, by transitivity of equality; and
- target non-amplification, by transitivity of a checker-validated finite preorder.

No general rule is claimed for `FiniteTable`.

### 8.3 Structural derivations and global obligations

A local structural derivation is emitted only when:

1. the adapter uses a supported structural node;
2. the observer syntax decomposes along the same structure;
3. every child judgment holds;
4. intermediate domain containment is checked; and
5. no global dependency is discarded.

Otherwise a property is either discharged by complete direct finite enumeration or reported `unknown`. The Minimal Core does not claim that arbitrary product or sum policies decompose fieldwise.

V06 ensures an unsupported global observer cannot receive `closed-derivation` or a certificate.

### 8.4 Extended effectful rule boundary

The Core theorem does not classify expected conversion failures. If a future Extended `partial-errors` profile is implemented, its primary permission relation MUST be:

\[
AllowedFail_f\subseteq X\times F_f
\]

so that \(AllowedFail_f(x,q)\) means that failure observation \(q\) is permitted for that exact input \(x\).

An observer-level quotient may be emitted only for diagnostics:

\[
R_e^f(o,q)
\iff
\exists x.\ o_X(x)=o\land AllowedFail_f(x,q).
\]

It may support proof only if the checker establishes contract separation:

\[
o_X(x_1)=o_X(x_2)
\Rightarrow
AllowedFail_f(x_1)=AllowedFail_f(x_2).
\]

For future Kleisli composition:

\[
AllowedFail_{f>=>g}(x,First(q))
\iff
AllowedFail_f(x,q),
\]

\[
AllowedFail_{f>=>g}(x,Second(q))
\iff
\exists y.\ f(x)=Ok(y)\land AllowedFail_g(y,q).
\]

These formulas are recorded to prevent regression to v0.3’s observer-quotiented theorem. They are not Core implementation or paper obligations.

---

## 9. Research questions and permitted claims

### RQ1 — Actual-comparator false agreement

Can an adapter pass all six declared round trips while `TargetNativeExact` reports a pair that violates the endpoint-owned safety relation?

Required evidence: Lean Theorems 2–4, A01/A02/A03/A05, and E01/E02.

### RQ2 — Two-sided and policy vacuity

Do soundness and adequacy separately expose unsafe equality, always-different comparison, extra safe equality, empty Match, and an unconstrained safety policy?

Required evidence: V01, V02, V10, the policy-vacuity conformance test, and comparator-indexed reports.

### RQ3 — Restricted checking and diagnostics

For the closed Minimal IR, can exhaustive checking decide the requested properties and emit comparator-specific paths and concrete witnesses, while unsupported observations produce `unknown`?

Required evidence: the full finite fixture matrix, V06, T01/T02 three-way classification, independent transformation-provenance reconstruction, witness replay, and report-schema validation.

### RQ4 — Total-success composition

Can exact and non-amplifying observer obligations be reused through the closed total-success judgment without assuming a global policy decomposes?

Required evidence: Theorem 6 and C01. If the implementation provides no useful derivation beyond direct enumeration, the paper MUST demote composition from a main mechanism claim.

### RQ5 — Strong direct-relation comparison

Given identical comparator, scope, policy, totality, and validity checks, does BL4 reach the same top-level verdict, and what structured diagnostics remain unique to GlueRift?

Required evidence: paired BL4 reports for every required attack and benign run.

### RQ6 — Operational fidelity

Do hermetically bound Go/Rust/Protobuf processes implement the declared `TargetNativeExact` truth table and reproduce the unsafe ordinary equality?

Required evidence: backend conformance plus E01/E02 native transcripts.

### 9.1 Intended claims

If all acceptance gates pass, the paper may claim:

1. round-trip laws do not imply policy soundness of the selected target-native comparison;
2. a well-typed, comparator-defined target twist that preserves every explicitly requested round-trip law can produce target-native policy laundering;
3. soundness and adequacy rule out different vacuities;
4. the finite Core exactly decides supported requests over their declared scopes;
5. exact and verified-preorder obligations compose in the closed total-success judgment;
6. normalized structural twists are generated completely within the declared Core family and partitioned into `lawful-safe`, `lawful-harmful`, and `law-breaking-or-inapplicable`, with inverse and four-map provenance;
7. native E01/E02 reproduce the attack and match the reference comparator; and
8. GlueRift provides the diagnostic differences actually observed against BL4.

### 9.2 Forbidden claims

The paper MUST NOT claim:

- first discovery of round-trip ambiguity or automorphism twisting;
- a new general lens or interoperability theory;
- automatic discovery or correctness of endpoint semantics;
- verification of arbitrary Go/Rust code;
- completeness outside the finite declared IR and transformation family;
- that a passing GlueRift result proves program equivalence;
- that carrier equality models a native comparator without a proved bridge;
- effectful composition in Core;
- formal verification of generated native backends;
- logical superiority over a complete direct relation checker;
- natural vulnerability prevalence;
- that “validators commonly” have the studied architecture; or
- that a `policy-unconstrained` result is security evidence.

---

## 10. Core evaluation contract

### 10.1 Status vocabulary

Semantic obligations use:

```text
proved-exhaustive
disproved
unknown
not-requested
invalid
```

Build, schema, process, hash, or resource failures are `tool-error`, not semantic `unknown`.

`PASS` and `FAIL` may appear in human tables only as renderings of typed statuses. Canonical JSON uses the full vocabulary.

The canonical fixture registry row is:

```text
run_id
fixture_kind
context_logical_path
transformation_base_context_logical_path | not-applicable
scope_logical_path
policy_logical_path
request_logical_path
request_id
validation_request_sha256
profile
required_law_ids
required_properties
required_properties_sha256
required_bridge_ids
required_transformation_family_sha256
comparator_spec_sha256
run_configuration_sha256
expected_profile_property_consistency
match_coverage_mode
expected_match_coverage_status
expected_safe_match_equality_status
expected_certificate_eligibility
expected_certificate_granted
expected_comparator_definedness_status
expected_law_statuses
expected_property_statuses
expected_bridge_statuses
expected_policy_contract_status
transformation_report_required
transformation_sha256 | not-applicable
expected_transformation_classification
expected_candidate_binding_status
expected_base_alignment_status
required_witness_kinds
bl2_paired
bl4_paired
native_replay_id | not-applicable
```

Every run variant, including H04 and V01 comparator variants, has its own row. The registry is canonical, immutable for the paper release, and bound by the reproduction manifest. `required_properties` stores the complete normalized payload, including every TNA `dimension_ids` list, while `required_properties_sha256` binds those bytes.

Before semantic evaluation, the fixture harness compares only declaration fields—request ID/hash, profile, law IDs, full property payload/hash, bridge IDs, transformation-family hash, transformation hash where applicable, comparator hash, Match-coverage mode, `native_replay_id`, and run-configuration hash—to the normalized inputs. The semantic evaluator never receives or consults an `expected_*` field. After evaluation, the fixture runner compares actual typed statuses and witnesses with the registry’s expected-oracle fields. A declaration-binding mismatch or a post-evaluation oracle mismatch is a reproduction `tool-error`, not a semantic verdict and never an input to that verdict.

The following assignment table is normative. `6` means all six law IDs are explicit; `PS`, `CA`, `CP`, `FC`, and `TNA` abbreviate the five §7.6 property requests. The registry owns the full payload and the exact expected statuses.

| Run(s) | Profile | Laws | Explicit properties | Match coverage | Eligible / granted | Transformation expectation |
|---|---|---:|---|---|---|---|
| `A01/A02/A03/A05.base` | `policy-sound-adequate` | 6 | PS, CA, CP, FC | `nonempty` | true / true | aligned base; no transformed classification |
| `A01/A02/A03/A05` | `policy-sound-adequate` | 6 | PS, CA, CP, FC | `nonempty` | true / false | `lawful-harmful`, candidate binding proved |
| `H01/H02` | `faithful-exact` | 6 | PS, CA, CP, FC | `bidirectional-total` | true / true | not applicable |
| `H04.tna` | `policy-sound-adequate` | 6 | PS, CA, CP, FC, TNA | `bidirectional-total` | true / true | not applicable |
| `H04.exact` | `faithful-exact` | 6 | PS, CA, CP, FC | `nonempty` | true / false | not applicable |
| `V01.carrier` | `policy-sound` | 6 | PS | `none` | true / true | not applicable |
| `V01.target` | `policy-sound` | 6 | PS | `none` | true / false | not applicable |
| `V02` | `policy-sound-adequate` | 6 | PS, CA | `nonempty` | false / false | not applicable |
| `V06` | `policy-sound` | 6 | PS | `none` | false / false | not applicable |
| `V10` | `policy-sound-adequate` | 6 | PS, CA, CP, FC | `nonempty` | true / false | not applicable |
| `T01.σ1/T01.σ2` | `policy-sound` | 6 | PS | `none` | true / true | `lawful-safe` |
| `T01.σ1∘σ2` | `policy-sound` | 6 | PS | `none` | true / false | `lawful-harmful` |
| `T02.σ` | `policy-sound` | 6 | PS | `none` | false / false | `law-breaking-or-inapplicable` |
| `policy-vacuity-conformance` | `policy-sound` | 6 | PS | `none` | false / false | not applicable |
| `C01.exact/C01.tna` | `diagnostic` | registry-declared | none | `none` | false / false | not applicable |

For `faithful-exact` rows, `expected_safe_match_equality_status=proved-exhaustive`. V02 deliberately violates its declared coverage and therefore has `expected_profile_property_consistency=invalid`. Every other row has the consistency and coverage statuses implied by the table. A future change to one of these assignments requires a reviewed contract revision, not only a registry edit.

The registry contains exactly three asserted T01 candidate rows: normalized \(\sigma_1\), normalized \(\sigma_2\), and normalized right-to-left \(\sigma_1\circ\sigma_2\), each identified by `transformation_sha256`. The family analyzer may enumerate additional permutations, but they are separate diagnostic entries and cannot replace one of these three conformance oracles.
`transformation_report_required=true` exactly for the four transformed A rows and the four asserted T01/T02 rows. It is false for aligned-base and ordinary comparison rows. E01/E02 bind the already required A01/A02 transformation reports through their native reference fields rather than duplicating a transformation report under the E row.
The A01 row fixes `native_replay_id=E01`, the A02 row fixes `native_replay_id=E02`, and A03/A05 use `not-applicable`. There are no separate `E01.reference` or `E02.reference` semantic registry rows; native replay reports set `reference_run_id=A01` or `A02` respectively.

### 10.2 Required lawful attacks

| ID | Structure | Required distinguishing observation |
|---|---|---|
| A01 | enum/sum permutation | `DENY` versus `ALLOW` |
| A02 | same-typed product-field permutation | `minimum` versus `maximum`, with nested field path |
| A03 | bounded complement | order or risk policy reversal under \(x\mapsto min+max-x\) |
| A05 | object-`Result` branch map | denied/failure role versus success role |

Each attack has an independently checked aligned base context \(A_0\) and the generated twisted candidate \(A_0^\sigma\). Both checks use the identical types, comparator, \(U\), `Safe`, `Match`, scope, policy, request, and run configuration. The base check MUST prove comparator definedness, all six laws, soundness, adequacy, precision, and faithfulness; equivalently for the selected relation, \(I_{A_0}^T=Match\subseteq Safe\). Its certificate is granted under the table in §10.1. This base obligation is not needed for the abstract harmful-membership definition, but it is mandatory evidence for the operational claim that the generated twist creates false agreement from aligned glue.

All four use `TargetNativeExact` and MUST:

- explicitly require all six round-trip laws in both the normalized request and fixture registry;
- pass source native, target native, source carrier, target carrier, source full transport, and target full transport obligations;
- pass comparator definedness;
- make the ordinary selected comparator equal on at least one unsafe pair;
- disprove \(Sound_T\);
- use an honest `Match` for which at least one required pair is absent, thereby also disproving adequacy;
- produce an unsafe false-agreement witness and a missing-match witness;
- be accepted by BL2’s round-trip criterion; and
- be rejected by BL4 on the same failed properties;
- carry a §17.7 transformation report classified `lawful-harmful`; and
- bind the exact transformation, inverse, action domain, conjugated four-map construction, and six passing law statuses;
- bind `base_check_report_sha256` and prove that its `candidate_sha256` equals the transformation report’s base `candidate_context_sha256`;
- prove that the attack check report’s `candidate_sha256` equals the transformation report’s `transformed_context_sha256`; and
- report `base_alignment_status=proved-exhaustive` and `candidate_binding_status=proved-exhaustive` before supporting any generated-laundering claim.

Fixture domains MUST contain values that exercise the twist rather than only fixed points.
A01, A02, and A05 use `generation_mode=enumerated`; A03 uses `generation_mode=declared-candidate` unless later promoted by a reviewed scalar-discovery profile.

### 10.3 Required benign cases

| ID | Case | Required result |
|---|---|---|
| H01 | constructor names differ, roles align | sound, adequate, precise, faithful |
| H02 | physical field order differs, role mapping is correct | sound, adequate, precise, faithful |
| H04 | source policy is more permissive than target policy | TNA run passes; exact run distinguishes stricter semantics |

H04 contains two independently declared endpoint-policy runs over the same lawful adapter:

1. `H04.tna` independently declares three endpoint correspondences as `Match` and `Safe_TNA`, including one strict conservative source-`ALLOW`/target-`DENY` pair; soundness, adequacy, precision, faithfulness, and target non-amplification pass.
2. `H04.exact` uses a nonempty exact `Match=Safe` for the truly equal policy-level pair and leaves the additional conservative induced pair outside that relation; adequacy passes, while soundness, precision, and faithfulness fail with the extra pair.

This construction preserves \(Match\subseteq Safe\) in both runs and demonstrates that non-amplification is not exact equality.

The canonical finite instance has:

\[
S=\{s_D,s_{A1},s_{A2}\},
\qquad
T=\{t_{D1},t_{D2},t_A\}.
\]

The source policy observations are `DENY, ALLOW, ALLOW`; the target observations are `DENY, DENY, ALLOW`. The lawful transport graph is:

\[
I_A^T=
\{(s_D,t_{D1}),(s_{A1},t_{D2}),(s_{A2},t_A)\},
\]

and the independently fixed universe additionally contains an unsafe non-induced pair:

\[
U=I_A^T\cup\{(s_D,t_A)\}.
\]

The TNA policy independently declares its first three pairs as `Safe` and `Match`; the fourth is unsafe. The exact policy independently declares:

\[
Safe=Match=\{(s_D,t_{D1}),(s_{A2},t_A)\}.
\]

Thus neither safety policy is universal. The declarations happen to align with \(I_A^T\) in the TNA run; they are not computed from it. This makes the expected statuses realizable without violating Match inclusion, non-circularity, or comparator functionality.

### 10.4 Required regressions

#### V01 — comparator divergence

V01 instantiates §4.4 with a non-universal safety relation containing only the two off-diagonal pairs and no requested Match property.

Two runs use identical adapter, domains, \(U\), and `Safe`:

| Run | Comparator | Induced relation | Soundness | Bridge |
|---|---|---|---|---|
| `V01.carrier` | `CarrierExact` | empty | proved, vacuous selected relation | `carrier_target_bridge=disproved` |
| `V01.target` | `TargetNativeExact` | diagonal | disproved | `carrier_target_bridge=disproved` |

The target run includes a native-comparator unsafe witness. Neither run may transfer the carrier verdict across the disproved bridge.

#### V02 — empty Match under requested coverage

`match_dimensions=[]` with nonempty or total Match coverage is a policy-specification error. No candidate property or certificate is issued.

#### V06 — unsupported global observer

An `ExternalObserverRef` controls a requested property. The result is `unknown`, the reason is `unsupported-observer`, and certification is ineligible.

#### V10 — extra safe equality

The selected induced relation contains every required match plus an extra pair that is safe but not required:

\[
Sound_T=proved,\qquad
Adequate_T=proved,
\]

\[
Precise_T=disproved,\qquad
Faithful_T=disproved.
\]

The witness is `extra-safe-equality`.

#### Policy-vacuity conformance test

This schema-level test is required even though it need not appear as a main paper fixture:

```text
safe_dimensions = []
safe_is_universal = true
policy_contract_status = policy-unconstrained
policy_vacuity_warning = true
certification.eligible = false
```

#### Core semantic microtests

The checker unit/conformance suite, although not a paper fixture family, MUST include:

- one nontrivial truth table for each of `CarrierExact`, `TargetNativeExact`, and `SourceNativeExact`;
- a width-four `ModularAffine` bijection with odd scale and its computed inverse;
- rejection of an even-scale `ModularAffine` when invertibility is requested; and
- a nested product/sum structural type whose expected permutation count, normalization, and duplicate elimination are fixed.

These tests prevent an unexercised Core constructor or comparator direction from surviving solely because it is listed in the grammar.

#### T01 — lawful asymmetric non-closure

T01 is the total three-element model fixed in §6.3. It MUST classify \(\sigma_1=(a\ b)\) and \(\sigma_2=(b\ c)\) as `lawful-safe` and the right-to-left composite \(\sigma_1\circ\sigma_2\) as `lawful-harmful`. Every candidate passes all six explicitly requested round trips and selected-comparator definedness. The composite is harmful solely because it induces the declared unsafe pair, not because it breaks a law.

#### T02 — sound but law-breaking transformation

T02 prevents soundness from standing in for lawfulness. Its exact finite model is:

\[
S=\{x,y\},\qquad T=\{a,b,c\},\qquad C=\{0,1,2\}.
\]

The four base maps are:

\[
e_S(x)=0,\quad e_S(y)=1,\qquad
d_S(0)=x,\quad d_S(1)=y,\quad d_S(2)=x,
\]
\[
e_T(a)=0,\quad e_T(b)=1,\quad e_T(c)=2,
\qquad
d_T(0)=a,\quad d_T(1)=b,\quad d_T(2)=a.
\]

The requested domains are:

\[
D_S^{rt}=\{x,y\},\quad D_T^{rt}=\{a,b\},\quad
K_S=K_T=\{0,1\},
\]
\[
D_{S\to T\to S}=\{x,y\},\qquad
D_{T\to S\to T}=\{a,b\}.
\]

With `TargetNativeExact`, let \(U=\{(x,a),(y,b),(x,b)\}\) and \(Safe=\{(x,a),(y,b)\}\). The comparison domains are exactly the projections \(\{x,y\}\) and \(\{a,b\}\), so the full-transport coverage rule in §3.7 is satisfied. The base context passes all six laws. For \(\sigma=(0\ 2)\), the conjugated candidate still induces only the two safe diagonal pairs in \(U\), but its target carrier round trip fails at carrier input \(0\):

\[
e_T^\sigma(d_T^\sigma(0))=2\neq0.
\]

The transformation report fixes `action_domain=All(C)`, `inverse_ir=\sigma`, and the complete-domain inverse check. The request uses `profile=policy-sound`, explicitly requests `PolicySoundness` and all six laws, and the policy sets `match_coverage=none`; it requests no Match-dependent property. The transformed candidate MUST therefore be `law-breaking-or-inapplicable` with reason `required-law-disproved`, while its non-classifying soundness diagnostic remains `proved-exhaustive`. It is never `lawful-safe`.

### 10.5 Required composition case

C01 contains only total adapters and supported observers. It has:

- one exact closed derivation; and
- one finite-preorder non-amplification closed derivation.

Both are also exhaustively rechecked. No allowed-error, partial-conversion, unit, or global-error run belongs to C01.

### 10.6 Required native fixtures

Both native manifests bind the explicit all-six-law requests of their corresponding A01/A02 registry rows; the phrase “six declared round trips” is not inferred from a profile name.
The exact reference bindings are `E01 → A01` and `E02 → A02`. Each native manifest’s `context_sha256` MUST equal both the corresponding transformation report’s `transformed_context_sha256` and the corresponding top-level reference check’s `candidate_sha256`; the comparator, scope, policy, request, and run-configuration hashes MUST also match. A mismatch is `tool-error` and the native replay cannot support the operational claim.

#### E01 — Protobuf decision swap

Separate Go and Rust processes use a shared `.proto`. The target adapter is a law-preserving decision twist. The required transcript includes:

```text
source program output: DENY
target program output: Permitted
transported source as Rust native: Permitted
ordinary target-native comparator: EQUAL

six declared round trips: proved-exhaustive
comparator_definedness: proved-exhaustive
policy_soundness: disproved
comparator_kind: target-native-exact
```

#### E02 — repeated-type field-role swap

A nontrivial record has at least two same-typed fields with different semantic roles. The target adapter swaps those roles while its inverse cancels the swap. Separate processes MUST show:

- all six round trips pass;
- a source record transports to the target program’s native output;
- ordinary target-native equality is reported;
- the policy relation rejects the pair; and
- the witness identifies the nested adapter path and violated field roles.

A library-backed wrapper may replace this record only through a later approved contract revision. It is not silently required in v0.3.1a.

E01 and E02 are operational proof-witness suites, not a prevalence experiment.

### 10.7 Expected categorical matrix

`P`, `D`, and `N` below abbreviate `proved-exhaustive`, `disproved`, and `not-requested`.

| Run class | \(\chi\) | six RT | defined | sound | adequate | precise | faithful |
|---|---:|---:|---:|---:|---:|---:|---:|
| A01/A02/A03/A05.base | T | P | P | P | P | P | P |
| A01/A02/A03/A05 | T | P | P | D | D | D | D |
| H01/H02 | T | P | P | P | P | P | P |
| H04.tna | T | P | P | P | P | P | P |
| H04.exact | T | P | P | D | P | D | D |
| V01.carrier | C | P | P | P | N | N | N |
| V01.target | T | P | P | D | N | N | N |
| V10 | T | P | P | P | P | D | D |

V02 is `invalid`; V06 is `unknown`. A fixture specification that cannot realize this matrix MUST be corrected before implementation continues.

The only Core comparison-property TNA request is `H04.tna`. Its normalized payload contains exactly `dimension_ids=["policy-level"]`; that dimension checks three induced pairs and is `proved-exhaustive`, and the aggregate checks one dimension/three pairs and is `proved-exhaustive`. Every other row in this comparison matrix reports TNA `not-requested`. `C01.tna` is a total-success composition derivation, not a `TargetNonAmplification` property request.

Transformation regressions have this separate classification matrix:

| Candidate | requested laws | defined | sound | classification |
|---|---:|---:|---:|---|
| T01.σ1 | P | P | P | `lawful-safe` |
| T01.σ2 | P | P | P | `lawful-safe` |
| T01.σ1∘σ2 | P | P | D | `lawful-harmful` |
| T02.σ | D | P | P | `law-breaking-or-inapplicable` |

---

## 11. Core baselines

### 11.1 BL2 — exhaustive finite round trips

BL2 checks the same candidate, types, domains, totality, and all six requested round-trip laws. It does not receive endpoint `Safe` or `Match` as an acceptance relation.

Expected result: BL2 accepts A01/A02/A03/A05 and E01/E02 at the adapter-law layer.

The Core fixture registry sets `bl2_paired=true` for exactly A01, A02, A03, and A05. E01/E02 expose the same six law statuses through their exact A01/A02 reference bindings and native replay reports; they are not separate BL2 authoring rows. Every other registry row sets `bl2_paired=false`.

### 11.2 BL4 — Direct-Relation

BL4 receives exactly:

\[
(A,\chi,U,Safe,Match)
\]

together with the same:

- endpoint observers and relations;
- comparator definedness rules;
- round-trip request;
- policy/specification validity checks;
- active safety/matching anchor coverage, Match coverage, and match-shape checks;
- policy-vacuity classification;
- finite enumeration engine;
- candidate and scope hashes; and
- native comparator binding when applicable.

BL4 directly enumerates \(I_A^\chi\) and checks the same four set inclusions/equality. It MUST use `TargetNativeExact` for every primary attack and native comparison. It runs V01 separately in carrier and target modes.

BL4 uses the same canonical pair order and comparator-evidence union, so a shared top-level disproof MUST identify the same first witness pair at the same comparison granularity. GlueRift may add an adapter derivation path, carrier class, or generated-twist provenance, but it cannot count a deliberately degraded BL4 counterexample as an advantage.

For C01, BL4 receives the same normalized component and composite relations and may apply the same valid relational-composition rule \(R_1;R_2\subseteq R\), in addition to exhaustive checking of the composite. GlueRift MUST NOT obtain a composition advantage by disabling an equally available rule in BL4. Any claimed difference must come from demonstrated endpoint-local structure, derivation reuse, or diagnostics rather than a weaker baseline implementation.

**Proposition 7 — Top-level parity.** Given identical normalized inputs and the same complete evaluator, GlueRift and BL4 return identical common validity, Match-coverage, policy-vacuity, comparator-definedness, target-non-amplification, soundness, adequacy, precision, and faithfulness statuses, together with the same first top-level policy/property witness where applicable.

BL4 is expected to reject the attacks. GlueRift is not logically stronger.

The Core fixture registry sets `bl4_paired=true` for exactly:

```text
A01
A02
A03
A05
H01
H02
H04.tna
H04.exact
V01.carrier
V01.target
V02
V06
V10
C01.exact
C01.tna
policy-vacuity-conformance
```

BL4 runs every listed ID and matches profile/property consistency, Match coverage and its policy witnesses, semantic status, invalid/unknown classification, policy-vacuity classification, TNA aggregate/per-dimension results, and the shared first top-level witness pair where applicable. Native E01/E02 are operational replays of reference specifications and are not additional BL4 authoring comparisons.

### 11.3 Permitted mechanism comparison

The comparison may measure or inspect only:

- carrier-summary diagnostics;
- comparator-divergence explanation;
- nested adapter paths;
- automatically generated harmful twists;
- reusable total-success derivation trees; and
- native manifest/evidence binding.

If the differences are negligible, the paper removes the mechanism-superiority claim and retains the operational attack plus two-sided checker.

BL0 and BL3 may appear as explanatory prose or Extended runs. They are not Core release gates.

---

# Part II — Artifact implementation contract

## 12. Core conformance boundary

### 12.1 Required Core components

An implementation is Core-conformant only if it contains:

| Layer | Required content |
|---|---|
| Comparator | all three `ComparatorSpec` cases; `TargetNativeExact` primary |
| Types | Unit, Bool, BoundedInt, BitVec, Sum, Product, object-`Result` |
| Adapters | §7.2 only |
| Domains | explicit finite `DomainSpec` and `PairDomainSpec` |
| Observers | §7.3 only, plus recognized unsupported marker |
| Relations | Exact, TNA over finite preorder, direct FiniteTable |
| Checking | six RT laws, definedness, four comparison properties, TNA, bridges |
| Composition | total-success exact/preorder only |
| Transformations | finite structural family in §6.1, lawful three-way partition, T01 and T02 |
| Lean | the nine theorem/counterexample groups in §14 |
| Attacks | A01, A02, A03, A05 |
| Benign | H01, H02, H04 |
| Regressions | V01, V02, V06, V10 and policy-vacuity conformance |
| Baselines | BL2 and BL4 |
| Native | E01 and E02 |
| Reproduction | one command, one canonical result owner, generated categorical table |

No omitted Extended feature is a hidden Core dependency.

### 12.2 Core release invariants

Core MUST satisfy:

1. one reference semantic evaluator owns all finite verdicts;
2. BL4 shares that evaluator and every common validity check;
3. selected-comparator properties are computed directly;
4. bridge failure never changes a direct native-comparator verdict;
5. carrier evidence is guarded by comparator applicability;
6. no arbitrary observer callback executes;
7. no expected meta-level conversion failure is accepted;
8. every disproof has a replayable canonical witness;
9. every property and witness binds `comparator_spec_sha256`;
10. every transformation claim binds its normalized transformation, inverse, action domain, mechanically conjugated four maps, requested laws, and classification; and
11. canonical evidence contains no ambient timestamp, host-specific absolute path, random order, or unbound native executable.

### 12.3 Non-Core items

The following are not Core conformance requirements:

- effectful or partial-error composition;
- `OutcomeContract` and failure vocabularies;
- Option or list IR;
- rich predicates and global arithmetic observers;
- checked casts or general affine/unit conversion;
- the reserved optional fixture identifiers listed in §21, including A04, H03, and E03;
- BL0 or BL3 as executable release gates;
- automatic scalar-template discovery;
- bit/list/graph automorphism enumeration;
- SMT;
- mutation studies, broad subjects, or performance experiments;
- broad custom-relation theorems;
- code generation; or
- verification of arbitrary native adapters.

---

## 13. Repository and canonical artifacts

### 13.1 Required layout

The implementation SHOULD use:

```text
proof/
  lakefile.lean
  lean-toolchain
  GlueRift/
    RoundTrip.lean
    Comparator.lean
    Twist.lean
    Vacuity.lean
    Bridge.lean
    Symmetry.lean
    Composition.lean

checker/
  Cargo.toml
  Cargo.lock
  src/
    type_ir.rs
    adapter_ir.rs
    domain.rs
    comparator.rs
    observer_ir.rs
    relation_ir.rs
    roundtrip.rs
    comparison.rs
    bridge.rs
    composition.rs
    carrier.rs
    transformation.rs
    witness.rs
    report.rs
    main.rs

spec/
  schema/
    gluerift.type-ir.v0.3.1a.schema.json
    gluerift.adapter-context.v0.3.1a.schema.json
    gluerift.validation-scope.v0.3.1a.schema.json
    gluerift.policy.v0.3.1a.schema.json
    gluerift.validation-request.v0.3.1a.schema.json
    gluerift.check-report.v0.3.1a.schema.json
    gluerift.roundtrip-report.v0.3.1a.schema.json
    gluerift.execution-trace-table.v0.3.1a.schema.json
    gluerift.witness.v0.3.1a.schema.json
    gluerift.bridge-report.v0.3.1a.schema.json
    gluerift.derivation-report.v0.3.1a.schema.json
    gluerift.carrier-summary.v0.3.1a.schema.json
    gluerift.transformation-report.v0.3.1a.schema.json
    gluerift.baseline-report.v0.3.1a.schema.json
    gluerift.native-manifest.v0.3.1a.schema.json
    gluerift.build-manifest.v0.3.1a.schema.json
    gluerift.native-replay-report.v0.3.1a.schema.json
    gluerift.backend-conformance.v0.3.1a.schema.json
    gluerift.fixture-results.v0.3.1a.schema.json
    gluerift.fixture-registry.v0.3.1a.schema.json
    gluerift.claim-manifest.v0.3.1a.schema.json
    gluerift.dynamic-dependency-manifest.v0.3.1a.schema.json
    gluerift.source-inputs-manifest.v0.3.1a.schema.json
    gluerift.run-configuration.v0.3.1a.schema.json
    gluerift.transformation-family.v0.3.1a.schema.json
    gluerift.reproduction-manifest.v0.3.1a.schema.json
    gluerift.results.v0.3.1a.schema.json
  run-config/
    core-v0.3.1a.json
  transformation-families/
    core-structural-v0.3.1a.json

fixtures/
  attacks/{A01,A02,A03,A05}/
  benign/{H01,H02,H04}/
  regressions/{V01,V02,V06,V10}/
  transformations/{T01,T02}/
  composition/C01/
  registry.json

baselines/
  BL2/
  BL4/

native/
  proto/
  E01/
  E02/
  manifests/

artifact/
  reproduce
  claims.json
  source-inputs.manifest.json
  reproduction-manifest.json
  results/
    results.json
  tables/

docs/
  threat-model.md
  trusted-base.md
  limitations.md
```

Equivalent organization is permitted only if every canonical owner and hash edge remains unambiguous.

### 13.2 Canonical serialization

Canonical control data and reports use:

- UTF-8;
- RFC 8785 JSON Canonicalization Scheme;
- exact integer or string encodings with no floating-point ambiguity;
- lexicographic ordering for semantically unordered finite collections;
- SHA-256 over canonical bytes;
- repository-relative POSIX logical paths for every workspace source, fixture, output, cwd, and artifact reference; and
- no creation time, host name, temporary directory, or ambient path.

The only absolute-path fields permitted in canonical evidence are schema fields explicitly named `*_absolute_path` or `image_internal_path` in the build/dependency manifests. On the pinned Darwin host profile they denote canonical tool or dynamic-library identities and are meaningful only together with `host_toolchain_descriptor_sha256`; they are not a claim that a complete OS image or root filesystem is captured.

Runtime telemetry such as wall-clock duration may be emitted separately. It is not part of `results.json` or a paper table.

### 13.3 Common report envelope

Every canonical report contains:

```text
schema
semantic_contract_version = "0.3.1a"
tool_build_sha256
run_configuration_sha256
evidence_id
candidate_sha256
types_sha256
validation_scope_sha256
endpoint_policy_sha256
validation_request_sha256
comparator_spec_sha256
dependency_evidence_ids
status
```

An absent input is represented by a typed `not-applicable` field defined by that report schema, not by silently omitting a hash edge.

### 13.4 Immutable inputs, limits, and family descriptors

`artifact/source-inputs.manifest.json` contains a canonical, path-sorted list:

```text
entries: [
  {
    logical_path
    sha256
    executable_bit
    role
  }
]
```

It includes only **primary immutable inputs**: source code, schema definitions, unbound fixture/type/context/policy/scope/request specifications, proofs, lockfiles, Protobuf sources, run configuration, transformation-family descriptor, and human-written templates. It expressly excludes:

- the containing source-input manifest and root reproduction manifest;
- resolved native replay manifests;
- build manifests and dynamic-dependency manifests;
- the claim manifest when it contains evidence IDs;
- generated reports, checked-in result owners, and generated paper tables; and
- any other control artifact that directly or transitively contains `source_tree_sha256`, `source_inputs_manifest_sha256`, a build-manifest hash, or an evidence ID derived from them.

Those exclusions make the graph acyclic. `source_tree_sha256` is the SHA-256 of the canonical primary `entries` array, and the whole source-input manifest has its own `source_inputs_manifest_sha256` bound by the root reproduction manifest. Build caches and outputs are not entries: the source tree is mounted read-only and every Lean, Cargo, Go, Protobuf, native, and table-generation output/cache directory is redirected to the external staging root.

`spec/run-config/core-v0.3.1a.json` fixes all finite-enumeration, recursion, width, memory, output, and semantic resource limits. Its hash is `run_configuration_sha256` in every semantic report and validation request. Core does not permit the earlier alternative of leaving limits implicit in the tool build.

`spec/transformation-families/core-structural-v0.3.1a.json` canonically defines:

- permitted structural generators;
- normalized transformation and inverse IR;
- derivation of the complete `All(C)` carrier action domain and its canonical value order;
- generation-rule IDs, parent paths, ordinals, and `generation_mode`;
- type-compatibility equivalence;
- recursive nesting;
- composition and normalization order;
- duplicate elimination; and
- the completeness wording for the family.

Its canonical hash is the sole meaning of `core-structural`; requests and reports bind that hash rather than an unversioned string. A family member is represented by its embedded normalized IR, inverse, action domain, and provenance—not only by the family hash or a transformed-context hash.

`artifact/reproduction-manifest.json` is the canonical, fixed-path root of the acyclic evidence graph. In dependency order it binds:

1. the primary source-input, run-configuration, transformation-family, fixture-registry, and pinned-image hashes;
2. the role-indexed build/dynamic-dependency manifest-set hashes derived from those primary inputs and image;
3. the resolved native-manifest set;
4. the generated semantic/native evidence-report set;
5. the claim-manifest hash, whose `required_evidence_ids` may name only reports from layer 4 and never the aggregate result owner;
6. the checked-in result-owner hash; and
7. the generated-table hashes.

The root does not attempt to hash itself. No lower layer may contain a hash from a later layer. The reproduction script validates this topological rule and the complete graph before executing any build.

### 13.5 Unique result owner

The checked-in Core evidence owner is:

```text
artifact/results/results.json
```

Every paper table cell is generated from this owner or a content-addressed child report. Hand-entered empirical verdicts are forbidden.

---

## 14. Lean mechanized core

### 14.1 Scope

Lean formalizes the total finite/shared-domain mathematical core. It does not formalize the Rust implementation, native backends, Protobuf runtime, partial conversions, or Extended profiles.

### 14.2 Required theorem and counterexample groups

The Lean build MUST include:

1. **L1 — Injectivity:** native round trip implies encoder injectivity on the admitted domain.
2. **L2 — One-side twist:** a target automorphism/inverse twist preserves target native and total carrier round trips.
3. **L3 — Full transport:** under explicit clean total/shared assumptions, the twist preserves both cross-language full round trips.
4. **L4 — Direct native laundering:** the §5.3 witness lies in `TargetNativeExact` and is laundering when unsafe.
5. **L5 — Two vacuities and divergence:** selected-relation emptiness makes soundness vacuous; nonempty Match defeats adequacy; disjoint images affect only `CarrierExact`; the V01 finite model distinguishes carrier and target-native relations.
6. **L6 — Comparator bridge:** the pointwise implications and scoped bridge equivalence hold under exactly the side conditions in §4.2.
7. **L7 — Native relation shape:** `TargetNativeExact` is functional; under adequacy, source full transport makes `Match` inverse-functional on its covered source projection. Include the source-native dual and do not claim unrestricted native-relation injectivity.
8. **L8 — Lawful residual transformations:** the exact observer stabilizer is a subgroup; the request-scoped lawful/safe/harmful/inapplicable sets form the stated disjoint partition; T01 gives two `lawful-safe` transformations whose composite is `lawful-harmful` while all three preserve every requested law; and T02 shows that a sound but request-law-breaking transformation is inapplicable rather than safe.
9. **L9 — Total composition:** exact and verified-preorder value judgments compose under the stated domain and relation premises.

The theorem names, assumptions, and generated documentation MUST make the scope restrictions visible.

### 14.3 Proof hygiene

Core proof files MUST:

- build with the pinned `lean-toolchain` and lock data;
- contain no `sorry`, `admit`, untracked axiom, or unsafe escape;
- identify any standard classical principles reported by `#print axioms`;
- include executable finite counterexamples for V01, T01 lawful asymmetric non-closure, and T02 law-breaking inapplicability; and
- be built by `lake build` inside reproduction.

The paper may say “mechanized in Lean for the stated total finite core.” It MUST NOT say that the native checker or backend is formally verified.

---

## 15. Reference checker semantics

### 15.1 Exhaustive evaluation

The Rust reference checker:

1. parses and canonicalizes the candidate, policy, scope, and request;
2. type-checks all IR;
3. enumerates finite domains in canonical order;
4. evaluates comparator definedness;
5. evaluates requested round trips stage by stage;
6. directly enumerates the selected \(I_A^\chi\);
7. constructs `Safe` and `Match`;
8. checks policy construction, \(Match\subseteq Safe\), safety vacuity, quantified Match coverage, separate anchor coverage, comparator-relative Match shape, and profile/property consistency;
9. if those candidate-property prerequisites are valid, checks soundness, adequacy, precision, faithfulness, and TNA; otherwise emits the required invalid/unknown policy result without evaluating a weakened candidate claim;
10. evaluates requested bridges independently;
11. emits carrier diagnostics with applicability;
12. emits total-success derivations where supported;
13. emits the first canonical property or policy witness for every disproof; and
14. computes `certification.eligible`, `granted`, and canonical blocking reasons only after all prerequisites and requested property statuses are known.

The checker MUST NOT replace selected native equality with carrier equality as an optimization unless a proved bridge report is an explicit dependency of that result.

### 15.2 Deterministic witnesses

Witness selection uses:

1. canonical source-value order;
2. canonical target-value order;
3. canonical adapter-path order; and
4. canonical dimension ID.

The same normalized inputs and tool build therefore produce byte-identical semantic reports.

### 15.3 Unknown and limits

An unsupported observer, relation rule, or requested derivation returns `unknown` with a stable reason. Exhaustion or memory limits that prevent complete enumeration return `tool-error` for Core; the tool MUST NOT present a sampled result as exhaustive.

Limits are fixed only by the canonical Core run configuration in §13.4. Every semantic report and request binds `run_configuration_sha256`; `tool_build_sha256` is not a substitute. Fixtures are chosen to complete within those pinned limits, but this contract imposes no performance-ranking claim.

### 15.4 Evaluation order and non-masking

An invalid specification prevents semantic property verdicts. A failed round trip does not erase directly computed diagnostics, but it makes any certificate requiring that law ineligible and makes a transformation that was required to preserve that law `law-breaking-or-inapplicable`. A failed bridge does not mask direct native-mode comparison results. A failed policy soundness result does not prevent adequacy or precision from being evaluated.

This non-masking rule is important: the report must distinguish exactly which obligation failed.

---

## 16. Structural transformation analyzer

### 16.1 Enumeration

The Core analyzer recursively enumerates the finite structural family in §6.1. It:

- groups sum variants by compatible payload signature;
- groups product fields by identical type signature;
- treats compatible object-`Result` branches as a two-element structural family;
- composes nested structural choices in canonical path order; and
- removes duplicate normalized transformations.

It does not invoke a general graph-automorphism engine.

### 16.2 Classification

For every admitted normalized \(\sigma\), classification follows this fixed order:

1. validate the transformation IR, action domain, inverse IR, and exhaustive two-sided inverse identities;
2. construct—not accept as an independent input—the transformed four-map context by target-side carrier conjugation;
3. verify the normalized four-map construction and its hashes against the base context;
4. type-check the transformed context;
5. directly evaluate selected-comparator definedness on all \(U\);
6. directly evaluate every law explicitly selected by the hashed request over its complete declared domain;
7. whenever a well-typed transformed context exists and the selected comparator is defined, evaluate \(Sound_\chi\) even if a requested law failed; use that result to choose `lawful-safe` or `lawful-harmful` only when all lawfulness premises passed, and otherwise retain it solely as a non-classifying diagnostic; and
8. evaluate requested adequacy, precision, faithfulness, TNA, carrier diagnostics, and canonical witnesses wherever their own prerequisites hold, also as non-classifying diagnostics.

Failure of any lawfulness premise produces `law-breaking-or-inapplicable` with the ordered reasons in §6.3, regardless of any separately computed soundness result. This is the rule exercised by T02. An `unknown` or `tool-error` result produces no three-way classification and propagates unchanged. A generated structural candidate whose declared construction or inverse check fails is additionally a transformation-family conformance `tool-error`; it cannot be evidence about the family. A manually admitted scalar candidate may validly be inapplicable.

A transformation is not labeled harmful merely because it changes a carrier label or because an ill-formed or law-breaking candidate induces an unsafe pair. Direct native-comparator classification never depends on a bridge.

### 16.3 Required generation evidence

The analyzer MUST automatically generate the harmful transformations underlying A01, A02, and A05. Their `generation_mode` is `enumerated`. It MUST pass the nested structural-family conformance test in §10.4. A03 may use a manually declared, type-checked `BoundedComplement` with `generation_mode=declared-candidate`; automatic scalar discovery is not required.

The target maps are constructed in normalized IR as:

```text
transformed_source_encode = base_source_encode
transformed_source_decode = base_source_decode
transformed_target_encode = Compose(first=base_target_encode, second=transformation)
transformed_target_decode = Compose(first=inverse, second=base_target_decode)
```

where `Compose(first=f, second=g)` denotes \(g\circ f\). Reproduction independently reconstructs these four maps, checks their canonical hashes and exhaustive finite semantics, and rejects a user-supplied transformed context that is not the mechanical result.

Every report states that completeness is relative to the finite Core structural family. If a carrier diagnostic is asserted to apply to the transformed native comparator, that assertion requires a bridge recomputed for the transformed context and binds `transformed_bridge_report_sha256`; direct target-native classification itself remains bridge-free.

---

## 17. Report and witness contracts

### 17.1 Required check report

The check report contains at least:

```text
comparison: {
  comparator_kind
  comparator_spec_sha256
  universe_pair_count
  induced_equality_pair_count
  comparator_definedness: {
    status
    checked_input_count
    witness_sha256
  }
}

bridges: {
  carrier_target_bridge: {
    status
    report_sha256
  }
  carrier_source_bridge: {
    status
    report_sha256
  }
  selected_carrier_bridge_status
}

policy: {
  safe_dimension_count
  safe_pair_count
  unsafe_pair_count
  safe_is_universal
  policy_contract_status
  policy_vacuity_warning
  match_dimension_count
  match_pair_count
  match_coverage: {
    mode
    status
    source_comparison_domain_sha256
    target_comparison_domain_sha256
    source_comparison_domain_count
    target_comparison_domain_count
    matched_source_count
    matched_target_count
    empty_match_witness_sha256
    unmatched_source_witness_sha256
    unmatched_target_witness_sha256
  }
  match_shape_compatibility
  safe_anchor_coverage
  match_anchor_coverage
}

properties: {
  policy_soundness
  comparison_adequacy
  comparison_precision
  faithful_comparison
  target_non_amplification: {
    aggregate_status
    checked_dimension_count
    checked_pair_count
    dimensions: [
      {
        dimension_id
        preorder_sha256
        status
        checked_pair_count
        witness_sha256
      }
    ]
  }
}

certification: {
  requested_profile
  profile_property_consistency_status
  minimum_required_property_kinds
  explicit_required_property_kinds
  extra_required_property_kinds
  explicit_required_law_ids
  safe_match_equality_status
  safe_match_equality_witness_sha256
  eligible
  granted
  blocking_reasons
}
```

Each property result contains status, checked count, and zero or one canonical witness hash. TNA records both aggregate dimension/pair counts and each per-dimension pair count; a `not-requested` TNA result uses zero counts and typed `not-applicable` witness fields.

### 17.2 Round-trip report

The round-trip report contains one entry for each of the six law IDs:

```text
laws: [
  {
    law_id =
        source-native
      | target-native
      | source-carrier
      | target-carrier
      | source-full-transport
      | target-full-transport
    domain_sha256
    declared_input_count
    checked_input_count
    status
    transport_coverage_status
    final_equality_status
    execution_trace_table_sha256
    first_failing_input
    first_failure_trace
    witness_sha256
  }
]
```

The canonical execution-trace table records every checked input and the ordered intermediate `Result` after each adapter stage. Native and carrier laws use their two stages; full transport laws use all four. `transport_coverage_status` and `final_equality_status` are separate for full transport and `not-applicable` where appropriate. A disproof embeds the first canonical failing trace and binds the ordinary `roundtrip-failure` witness.

### 17.3 Comparator-specific evidence

Every comparison witness contains:

```text
witness_kind
source_value
target_value
comparator_kind
comparator_spec_sha256
comparator_evidence
violated_or_missing_dimensions
adapter_path
replay_command
```

`comparator_evidence` is a discriminated union:

```text
CarrierExact {
  source_encoding
  target_encoding
  common_carrier
}

TargetNativeExact {
  source_encoding
  target_decode_result
  compared_target_value
}

SourceNativeExact {
  target_encoding
  source_decode_result
  compared_source_value
}
```

A common carrier is mandatory only for `CarrierExact`. Non-selected encodings may appear as diagnostic fields but do not determine the witness.

Required witness kinds are:

```text
unsafe-false-agreement
missing-required-match
extra-safe-equality
comparator-undefined
bridge-divergence
roundtrip-failure
match-coverage-empty
match-coverage-source-gap
match-coverage-target-gap
safe-match-divergence
```

The three `match-coverage-*` records and `safe-match-divergence` are policy-well-formedness witnesses, not comparator witnesses. Coverage witnesses bind `coverage_mode`, both comparison-domain hashes, and either `match_pair_count=0` or the first unmatched endpoint value in canonical order. A Safe/Match divergence binds the first symmetric-difference pair and its two membership bits. Their comparator-evidence field is typed `not-applicable`; all other kinds above use the comparator-specific union.

### 17.4 Bridge report

A bridge report contains:

```text
bridge_kind = carrier-target | carrier-source
universe_pair_count
status
checked_pair_count
counterexample_pair
carrier_comparator_evidence
native_comparator_evidence
sufficient_rule_coverage
```

`sufficient_rule_coverage` is explanatory. Exhaustive relation equality owns the `proved-exhaustive` status.

### 17.5 Carrier summary

The carrier summary contains:

```text
source_successful_image
target_successful_image
shared_carrier_classes
class_endpoint_pairs
class_observation_conflicts
evidence_basis
applicability_to_selected_comparator
bridge_report_sha256
```

It MUST NOT contain a user-authored carrier-to-policy mapping in default derive mode.

### 17.6 Total derivation report

The derivation report contains only:

```text
judgment_kind = total-success
relation_kind = exact | target-no-amplification
adapter_path
observer_paths
input_domain_sha256
output_domain_sha256
children
relation_bridge
status
exhaustive_crosscheck_sha256
```

There is no Core outcome-judgment, allowed-error, failure-vocabulary, or error-relation report.

### 17.7 Transformation, baseline, and aggregate reports

Transformation reports bind:

```text
transformation_family_sha256
generation_mode = enumerated | declared-candidate
generation_rule_id
generation_parent_path
generation_ordinal
transformation_ir
transformation_sha256
inverse_ir
inverse_sha256
inverse_check_status
action_domain
action_domain_sha256
twist_side = target
twist_construction = carrier-conjugation
comparator_spec_sha256
candidate_context_sha256
base_check_report_sha256
base_alignment_status
base_source_encode_sha256
base_source_decode_sha256
base_target_encode_sha256
base_target_decode_sha256
transformed_context_sha256
transformed_check_report_sha256
candidate_binding_status
transformed_source_encode_sha256
transformed_source_decode_sha256
transformed_target_encode_sha256
transformed_target_decode_sha256
four_map_construction_status
four_map_semantics_check_sha256
well_typed_status
comparator_definedness_status
requested_law_ids
roundtrip_statuses
lawfulness_status
classification = lawful-safe | lawful-harmful | law-breaking-or-inapplicable
inapplicability_reasons
selected_property_statuses
harmful_witness_sha256
transformed_bridge_report_sha256
family_completeness_statement
```

`transformation_ir` and `inverse_ir` are the §7.2 normalized \(C\to C\) Adapter terms. `action_domain` is the derived exhaustive `All(C)` value table, never an authored subset. All three are embedded normalized values as well as separately hashed evidence. `candidate_context_sha256` names the base context, and the transformation report’s common-envelope `candidate_sha256` MUST equal that same base hash. The four base-map hashes MUST equal that context’s normalized maps; the four transformed-map hashes MUST equal the mechanically reconstructed maps in §16.3. `four_map_construction_status` is `proved-exhaustive` only when both the normalized IR and exhaustive finite map semantics agree.

For every required attack, `base_check_report_sha256` MUST resolve to a check report whose `candidate_sha256` equals `candidate_context_sha256`, whose comparator/scope/policy/request/run-configuration hashes equal the transformed check’s hashes, and whose comparator definedness, six laws, soundness, adequacy, precision, and faithfulness are all `proved-exhaustive`. Exactly then `base_alignment_status=proved-exhaustive`. `transformed_check_report_sha256` MUST resolve to the attack’s top-level check report, and that report’s `candidate_sha256` MUST equal `transformed_context_sha256`; exactly then `candidate_binding_status=proved-exhaustive`. Either failed equality or dependency is a `tool-error`, not a semantic attack result.
Non-attack transformation regressions type `base_check_report_sha256` and `base_alignment_status` as `not-required`; every transformation result paired with a top-level candidate still requires proved candidate binding, while an unpaired extra family diagnostic types its transformed-check and candidate-binding fields as `not-required`.

`harmful_witness_sha256` is mandatory exactly for `lawful-harmful` and is `not-applicable` otherwise. `inapplicability_reasons` is nonempty exactly for `law-breaking-or-inapplicable`. A transformed bridge hash is mandatory only when carrier-derived evidence is claimed to apply to the transformed native comparator; it is `not-required-direct-native` for direct target-native classification and MUST NOT reuse the base context’s bridge report.
When evaluation is `unknown` or `tool-error`, the error envelope uses §13.3’s typed `not-applicable` representation for `classification`; `not-applicable` is not a fourth classification value and cannot support a transformation claim.

The evidence DAG orders the base context before the transformed context, the transformed context before any transformed bridge, and those artifacts before the transformation report. Reproduction independently checks every edge, inverse identity, action-domain binding, four-map construction, and classification.

BL4 reports bind the same normalized semantic input hashes as the paired GlueRift report and contain a machine-checked parity field.

The canonical aggregate table contains:

```text
run_id
check_report_sha256
validation_request_sha256
candidate_sha256
comparator_kind
profile: {
  requested_profile
  profile_property_consistency_status
  safe_match_equality_status
  safe_match_equality_witness_sha256
}
match_coverage: {
  mode
  status
  source_comparison_domain_sha256
  target_comparison_domain_sha256
  source_comparison_domain_count
  target_comparison_domain_count
  matched_source_count
  matched_target_count
  empty_match_witness_sha256
  unmatched_source_witness_sha256
  unmatched_target_witness_sha256
}
six_roundtrip_statuses
comparator_definedness
bridge_statuses: {
  carrier_target
  carrier_source
  selected_carrier_bridge
}
policy_contract_status
policy_vacuity_warning
policy_witnesses
property_statuses: {
  policy_soundness
  comparison_adequacy
  comparison_precision
  faithful_comparison
  target_non_amplification_aggregate
  target_non_amplification_by_dimension
}
property_witnesses: [
  {
    property_id
    witness_kind
    witness_sha256
  }
]
certification: {
  eligible
  granted
  blocking_reasons
}
BL2_result: {
  law_statuses
}
BL4_result: {
  report_sha256
  common_validity_statuses: {
    profile_property_consistency
    match_subset_safe
    safe_anchor_coverage
    match_anchor_coverage
    match_shape_compatibility
    comparator_definedness
  }
  match_coverage_status
  policy_contract_status
  policy_witnesses
  property_statuses
  target_non_amplification_aggregate
  target_non_amplification_by_dimension
  property_witnesses
  validity_parity_status
  coverage_parity_status
  policy_parity_status
  property_parity_status
  witness_parity_status
}
transformation_results: [
 {
  transformation_report_sha256
  base_context_sha256
  base_check_report_sha256
  base_alignment_status
  transformed_context_sha256
  transformed_check_report_sha256
  candidate_binding_status
  transformation_sha256
  inverse_sha256
  action_domain_sha256
  lawfulness_status
  classification
  harmful_witness_sha256
 }
]
native_replay_result: {
  replay_report_sha256
  reference_run_id
  reference_check_report_sha256
  reference_candidate_sha256
  reference_candidate_binding_status
  ordinary_comparator_result
}
```

The arrays preserve multiple witnesses and transformations from one run, including both the unsafe-equality and missing-match witnesses required by every attack and all three asserted T01 candidates. A human paper table may flatten this object into deterministic columns, but the canonical owner MUST NOT collapse policy/coverage evidence, property-specific evidence, base/candidate binding, or the comparator-appropriate bridge.

---

## 18. CLI contract

The executable is `gluerift`.

### 18.1 Commands

```text
gluerift check
  --context <path>
  --scope <path>
  --policy <path>
  --request <path>
  --out <path>

gluerift roundtrip
  --context <path>
  --scope <path>
  --policy <path>
  --request <path>
  --out <path>

gluerift derive-carrier
  --context <path>
  --scope <path>
  --policy <path>
  --request <path>
  --out <path>

gluerift transformations
  --context <path>
  --scope <path>
  --policy <path>
  --request <path>
  --family spec/transformation-families/core-structural-v0.3.1a.json
  --out <path>

gluerift run-fixtures
  --registry <path>
  --out-dir <path>

gluerift run-baselines
  --registry <path>
  --baselines BL2,BL4
  --out-dir <path>

gluerift replay-native
  --manifest <path>
  --out <path>

gluerift reproduce
  --profile core
  --out-dir <path>
```

The comparator is read only from the hashed `ValidationScope`. No command-line comparator override is permitted for a certification run.

`derive-carrier` always emits bridge status and selected-comparator applicability. `transformations` validates the descriptor against `required_transformation_family_sha256`, binds the selected comparator, constructs the transformed four-map context from the base context plus normalized transformation/inverse IR, and executes the ordered §16.2 classification. It accepts no independent transformed-context input. The report may serialize the constructed context as evidence, but reproduction reconstructs it rather than trusting it.

### 18.2 Exit codes

```text
0  all requested obligations have their expected status
1  unexpected semantic disproof
2  invalid specification or schema
3  unexpected unknown
4  tool, build, resource, hash, or native-process error
```

Fixture runners compare actual typed statuses with registry expectations, so an expected attack disproof does not make reproduction exit with code 1.

Core exposes no code-generation command and no outcome-contract option.

---

## 19. Native Go/Rust/Protobuf replay

### 19.1 Architecture-faithful path

The formal carrier \(C\) denotes a decoded logical Protobuf value shared across generated Go and Rust representations. The required output path is:

```text
Go source-native output
  -> Go source encoder
  -> logical Protobuf carrier
  -> serialized process boundary
  -> Rust logical carrier
  -> Rust target decoder
  -> transported Rust-native value
  -> Rust target-native equality
  -> target program's Rust-native output
```

The primary result is `TargetNativeExact`. Carrier equality and wire bytes are explanatory only. The artifact MUST NOT assume Protobuf has a unique raw byte encoding.

### 19.2 Process isolation

The Go source producer and the Rust target/comparator execute as separate OS processes. The Go adapter is linked only into the Go-side role; the Rust adapter and ordinary target-native comparator are linked only into the Rust-side role. A separate harness process orchestrates them. Each process uses:

- fixed length-delimited or canonical JSON-line protocol;
- bounded stdin, stdout, and stderr;
- explicit timeout and exit status;
- malformed-message rejection;
- no crash-to-semantic-value coercion;
- an empty environment plus an exact whitelist;
- a pinned Darwin host/toolchain descriptor that hashes the actual compiler, linker, and tool executables used by this profile; and
- an enforced network-disabled execution context.

The default protocol is declared in the manifest and versioned. Every request and response carries fixture ID, operation ID, and canonical typed payload.

### 19.3 Native replay manifest

Each replay manifest contains:

```text
schema = "gluerift.native-manifest/v0.3.1a"
fixture_id
comparator_kind = target-native-exact
comparator_spec_sha256
types_sha256
context_sha256
validation_scope_sha256
endpoint_policy_sha256
validation_request_sha256
run_configuration_sha256
proto_schema_sha256
stdin_or_fixture_logical_path
stdin_or_fixture_sha256
build_manifest_set_sha256
dynamic_dependency_manifest_set_sha256
host_toolchain_descriptor_sha256
protocol
ordinary_comparator_role
expected_comparator_output

environment_mode = empty-plus-whitelist
environment = {
  LANG: "C.UTF-8",
  LC_ALL: "C.UTF-8",
  TZ: "UTC",
  SOURCE_DATE_EPOCH: "<fixed decimal>"
}
runtime_environment_sha256
network_mode = disabled

executables: [
  {
    role
    logical_path
    sha256
    argv
    working_directory
    build_manifest_sha256
    dynamic_dependency_manifest_sha256
    timeout
    stdin_limit
    stdout_limit
    stderr_limit
  }
]
```

`logical_path` and `working_directory` are repository-relative POSIX paths. `argv` uses logical placeholders resolved by the harness. A host temporary directory or checkout path is never serialized into canonical evidence.

The harness clears the ambient environment before adding exactly the declared map. Undeclared environment additions are a replay error.

`runtime_environment_sha256` is the SHA-256 of the canonical subobject consisting exactly of `environment_mode` and the sorted `environment` map. It has no separate mutable owner.

`build_manifest_set_sha256` hashes the canonical role-sorted list of `(role, build_manifest_sha256)` pairs from `executables`. `dynamic_dependency_manifest_set_sha256` is defined analogously. The two set hashes bind cardinality and role association; no singular top-level build hash stands in for multiple native processes. `stdin_or_fixture_logical_path` is a repository-relative fixture path whose canonical bytes must match `stdin_or_fixture_sha256`.

### 19.4 Build manifest

Each executable’s content-addressed build manifest binds:

```text
schema = "gluerift.build-manifest/v0.3.1a"
host_toolchain_descriptor_sha256
source_tree_sha256
source_inputs_manifest_sha256
source_file_hashes
lockfile_hashes
proto_schema_sha256
network_mode = disabled

build_environment_mode = empty-plus-whitelist
build_environment
build_environment_sha256

compiler_absolute_path
compiler_executable_sha256
compiler_version
compiler_flags
target_triple

linker_absolute_path
linker_executable_sha256
linker_version
linker_flags

output_logical_path
output_executable_sha256
dynamic_dependency_manifest_sha256

build_steps: [
  {
    step_id
    tool_absolute_path
    tool_executable_sha256
    argv
    working_directory
    environment_sha256
    declared_input_hashes
    declared_output_logical_paths
  }
]
```

Compiler flags and build steps are ordered and include path-remapping/reproducibility options. `working_directory` and all declared input/output paths are repository-relative logical paths. Each build tool is invoked through a canonical logical path bound to the hashed executable in the pinned Darwin host/toolchain descriptor; no ambient `PATH` lookup or inherited build environment is permitted.

`build_environment_sha256` hashes the canonical mode/map subobject just as the runtime environment hash does. The harness executes every step from the declared cwd with exactly that environment and with networking disabled. A build that cannot enforce `network_mode=disabled` is nonconformant.

The dynamic-dependency manifest is canonical and sorted. For each runtime library it records:

```text
library_identity
image_internal_path
sha256
```

The path records the Darwin library identity reported by the loader tooling. The harness verifies the host/toolchain descriptor, compilers, linker, sources, lockfiles, generated code inputs, output binaries, and dynamic dependencies before process launch. This profile does not claim to capture a complete OS image.

### 19.5 Backend conformance

For each finite native fixture, the checker emits a content-addressed native-reference bundle containing the four Adapter-IR truth tables, the complete target-native relation table on \(U\), all six staged round-trip tables, and a canonical unsafe witness/path. The native harness reads that bundle as its only semantic authority and exhaustively compares the manually written Go and Rust operations with it. No second hand-written Rust reference model participates in conformance. A negative regression mutates a simulated native result and legacy local model together while leaving the checker bundle unchanged, and requires conformance failure.

The backend-conformance report contains:

```text
fixture_id
comparator_kind
comparator_spec_sha256
native_source_tree_sha256
native_target_tree_sha256
build_manifests: [
  { role, build_manifest_sha256 }
]
build_manifest_set_sha256
runtime_environment_sha256
stdin_or_fixture_logical_path
stdin_or_fixture_sha256
dynamic_dependency_manifests: [
  { role, dynamic_dependency_manifest_sha256 }
]
dynamic_dependency_manifest_set_sha256
context_sha256
validation_scope_sha256
checked_adapter_value_count
adapter_value_mismatches
checked_comparator_pair_count
comparator_truth_table_mismatches
status
```

Zero mismatches are required. This is exhaustive finite backend-conformance testing, not a formal proof of the native code.

### 19.6 Native replay report

The replay report includes:

- `reference_run_id`, `reference_check_report_sha256`, `reference_candidate_sha256`, and `reference_candidate_binding_status`;
- every specification and comparator hash;
- build, environment, input, image, and dependency hashes;
- per-process logical cwd, argv, executable, and environment bindings;
- process exit and protocol results;
- all six round-trip statuses;
- selected-comparator definedness and ordinary comparison result;
- carrier bridge status as a separate diagnostic;
- GlueRift property statuses and witness IDs; and
- backend-conformance evidence ID.

A failed carrier bridge does not make a correct target-native attack result unknown.
`reference_candidate_binding_status=proved-exhaustive` exactly when the §10.6 E01→A01 or E02→A02 context and all common semantic-input hashes agree.

---

## 20. Reproduction and evidence ownership

### 20.1 Top-level command

The release provides:

```text
./artifact/reproduce
```

It runs Core only unless an explicit approved profile manifest says otherwise. It mounts the source-input tree read-only, writes every build cache and output to a clean staging directory outside that tree, and never overwrites checked-in evidence.

After pinned tools and dependencies are provisioned, semantic reproduction runs with enforced network isolation and read-only source enforcement. `network_mode=disabled` and the isolation result are bound into the reproduction report; inability to enforce isolation is a tool error.

### 20.2 Required sequence

The command:

1. verifies the reproduction graph, source-input manifest, source-entry hashes, schema, lockfile, pinned Darwin host/toolchain descriptor, run-configuration, transformation-family, and native manifest hashes;
2. records `source_tree_sha256` from the canonical immutable-entry array and creates external build/cache/output roots;
3. builds the Lean core and checks proof hygiene;
4. builds the Rust checker from the pinned lockfile and runs its unit/conformance suite, including symmetric microtests for all three comparator kinds;
5. validates schemas, canonical hashes, registry declaration bindings, and native manifests; an intentionally semantic-invalid regression such as V02 must still parse and bind, then produce the registry-declared `invalid` policy result rather than aborting the whole reproduction;
6. runs the aligned base and transformed rows for A01, A02, A03, and A05, then H01, H02, H04, V01, V02, V06, V10, T01, T02, and C01;
7. runs the policy-vacuity conformance test;
8. generates and classifies Core structural twists, including A01, A02, A05, and the nested-family conformance case; verifies the lawful three-way partition and T01/T02 classifications;
9. runs BL2 and BL4;
10. verifies the exact E01→A01 and E02→A02 checker bundle, reference-candidate/context, and transformation hash bindings, then builds and replays E01 and E02 with the pinned tools;
11. runs native backend conformance against the checker-emitted four-map, \(E_A^T\), six-law, and canonical-witness truth tables;
12. replays every required property, Match-coverage policy, round-trip, and bridge witness;
13. verifies all evidence IDs, hashes, parity edges, and applicability guards; independently checks each transformation/inverse pair, action domain, conjugated four-map construction, requested-law status, three-way classification, and transformed bridge edge when applicable; for each required attack it also rechecks aligned-base statuses, equality of the base check’s candidate hash with the transformation base hash, and equality of the attack check’s candidate hash with the generated transformed-context hash;
14. creates one canonical staging `results.json`;
15. generates the categorical paper table from that owner;
16. byte-compares the staging owner and table with the checked-in release artifacts; and
17. re-verifies every immutable source entry and confirms the same `source_tree_sha256`.

Expected attack disproofs are successful fixture outcomes because the registry binds their expected typed statuses.

### 20.3 Claim manifest

Every paper claim is registered as:

```text
claim_id
permitted_wording
forbidden_overstatement
required_evidence_ids
required_validation_request_sha256
required_profile
required_profile_property_consistency_status
required_policy_status
required_comparator_kind
required_match_coverage_mode
required_match_coverage_status
required_safe_match_equality_status
required_policy_witness_ids
required_property_statuses
required_certification_eligible
required_certification_granted
required_transformation_classification
required_base_alignment_status
required_candidate_binding_status
result
```

The release check rejects a claim if:

- its evidence uses another comparator;
- a required bridge is absent for carrier-derived native evidence;
- it cites `policy-unconstrained`;
- it cites an unknown or tool-error result;
- its request, profile, Match-coverage, Safe=Match, property-status, or certification guards do not match the canonical aggregate row;
- it promotes nonrequired Extended evidence;
- it describes BL4 as logically weaker;
- it calls native code formally verified;
- a native operational claim lacks the proved E01→A01 or E02→A02 reference-candidate binding;
- it calls a law-breaking or inapplicable transformation a harmful policy-laundering twist;
- a generated-transformation claim lacks the normalized transformation, inverse, action domain, four-map construction, requested-law, or classification evidence;
- a generated-laundering claim lacks `base_alignment_status=proved-exhaustive` or `candidate_binding_status=proved-exhaustive`, or its base/transformed check hashes do not resolve to the exact contexts in the transformation report; or
- it exceeds the literal limitations in §22.

### 20.4 Determinism

Canonical fixture order, exhaustive value order, witness selection, reports, aggregate verdicts, and paper tables are deterministic.

Descriptive runtime or artifact-size counts MAY be reported in a separate telemetry table. There is no performance RQ, inferential statistic, annotation-effort claim, or performance acceptance threshold.

---

## 21. Extended profiles

### 21.1 Promotion rule

An Extended profile:

- has its own schema and manifest;
- cannot alter Core semantics;
- reports `unknown` rather than silently weakening a property;
- is excluded from Core acceptance;
- is excluded from paper claims unless explicitly promoted by a later contract revision; and
- cannot repair a failed Core attack, theorem, baseline, or native replay.

### 21.2 `partial-errors`

This profile may add:

- `Restrict`, `CheckedCast`, and partial affine adapters;
- `OutcomeContract`;
- an input-indexed \(AllowedFail_f\);
- failure vocabularies and stage paths; and
- effectful Kleisli composition satisfying §8.4.

It MUST NOT use the observer quotient \(R_e^f\) as the primary permission relation without proving contract separation.

### 21.3 `rich-ir`

This profile may add:

- Option, bounded lists, and list reversal;
- rich predicate domains;
- generic field, arithmetic, comparison, conjunction, and join observers;
- failure-class observations; and
- supported registered rules for selected global observations.

### 21.4 `units-affine`

This profile owns:

- A04 epoch or unit affine twist;
- H03 exact unit conversion;
- rational affine quantities;
- unit and dimension registries; and
- exact overflow/rounding contracts.

### 21.5 `extra-fixtures` and `extra-baselines`

This profile owns:

- the reserved identifiers V03–V05, V07–V09, and E03;
- additional native library-backed wrappers; and
- executable BL0 or BL3.

These identifiers have no normative fixture semantics in v0.3.1a, and no definition is inherited from v0.3. An implementation may use them only after a versioned Extended profile manifest defines their types, candidate, policy, scope, comparator, expected statuses, and claim boundary. They may be useful context but are not required to close v0.3.1a.

### 21.6 Other Extended profiles

The following remain Extended:

```text
smt
scalar-template-discovery
graph-automorphisms
mutation-study
opaque-subjects
large-corpus
performance
custom-relation-composition
code-generation
```

Code generation, if later implemented, MUST distinguish unverified candidates from certified outputs. No code-generation lifecycle is part of this contract.

---

## 22. Trusted base and literal limitations

### 22.1 Trusted base

| Component | Trusted role | What is checked |
|---|---|---|
| Endpoint policy owner | supplies `Safe`, `Match`, observers, scope, comparator | schema, typing, coverage, inclusion, vacuity |
| Validation request | selects required laws/properties | hash binding and profile rules |
| Rust reference checker | executes finite semantics | fixture oracles, BL4 parity, witness replay |
| Lean kernel and libraries | checks formal core | pinned build and axiom audit |
| Rust/Go/Protobuf toolchains | build native evidence | pinned host/toolchain descriptor, versions, hashes, flags |
| Native fixture source | realizes IR and comparator | exhaustive backend conformance |
| Darwin runtime layer | supplies the tested host ABI and loader behavior | host/toolchain descriptor and dependency hashes; no complete-OS-image claim |

The Direct-Relation baseline is not treated as an adversarial strawman. It shares the trusted semantic inputs and common evaluator.

### 22.2 Literal limitations

A passing Core result means only:

> For the selected comparator, supported finite IR, declared nonempty universe, checked total-success scopes, and trusted endpoint policy, the requested set-theoretic properties hold exhaustively.

It does not establish:

- correctness or completeness of endpoint policy;
- correctness of the translated program;
- observational equivalence beyond the selected policy;
- safety outside \(U\);
- behavior of arbitrary handwritten adapters;
- completeness outside the declared transformation family;
- semantic identity of raw Protobuf bytes;
- formal verification of native binaries;
- absence of natural vulnerabilities; or
- ecosystem prevalence.

Carrier diagnostics are evidence about `CarrierExact` unless a proved bridge explicitly transfers them.

A `policy-sound` result alone MUST be paraphrased only as:

> No declared unsafe false agreement exists for the selected comparator in the supported, completely enumerated scope.

It is not faithful or complete comparison evidence unless the separately requested adequacy and precision obligations also pass.

---

## 23. Implementation sequence and research kill gates

No phase begins until the external reviewer approves v0.3.1a or returns a bounded correction.

### Phase 0 — design approval

Freeze:

- comparator-indexed definitions;
- request-scoped lawful/safe/harmful/inapplicable transformation partition;
- quantified Match-coverage semantics and profile/property consistency;
- generated-transformation provenance;
- total-success Core;
- Minimal fixture matrix;
- BL4 parity contract;
- native manifest binding; and
- allowed paper claims.

**Gate:** any unresolved P0 keeps implementation at No-Go.

### Phase 1 — Lean core

Mechanize L1–L9 before relying on the corresponding prose theorem. L8 includes the three-way partition, T01 lawful non-closure, and T02 sound-but-inapplicable counterexample.

**Scope-reduction or kill conditions:**

- the direct target-native laundering witness does not type-check;
- full transport preservation requires assuming the security conclusion;
- the bridge theorem needs broader premises than the checker records;
- comparator-specific shape claims are false; or
- composition requires the desired observer relation as an unproved premise.

If only composition fails to provide useful reuse, demote composition from the main contribution and retain direct finite checking. If the target-native laundering theorem fails, the project is No-Go.

### Phase 2 — finite checker

Implement the minimal IR and first run A01, A02, A03, A05, H01, H02, H04, V01, V02, V06, V10, T01, and T02.

**Kill conditions:**

- V01 does not distinguish carrier and target-native relations exactly as specified;
- a direct target-native unsafe pair is missed;
- `safe_dimensions=[]` receives a certificate;
- a profile/property mismatch or disproved required Match coverage receives a candidate verdict or certificate;
- a sound but request-law-breaking T02 transformation is classified `lawful-safe`;
- a harmful fixture is falsely accepted under its declared policy;
- a supported benign run is rejected for reasons not justified by its chosen relation; or
- a witness cannot identify the selected comparator path and violated observation.

### Phase 3 — structural diagnostics and composition

Generate A01/A02/A05 twists, pass the nested-family microtest, reconstruct and verify every inverse/action-domain/four-map provenance record, confirm T01’s lawful non-closure and T02’s inapplicability, and build C01 total derivations.

**Mechanism-demotion condition:** if generated twists, carrier diagnostics, and derivations add no useful information over BL4, remove the “new mechanism” claim. The operational attack remains viable.

### Phase 4 — strongest baseline

Run BL2 and BL4 with shared normalized inputs.

**Kill condition:** any top-level BL4/GlueRift verdict mismatch under identical inputs is an implementation or fairness defect and blocks evaluation. BL4 correctly rejecting every attack is expected, not a failure.

### Phase 5 — native replay

Implement E01 first, then E02.

**Project kill condition:** if E01 does not pass its six declared round trips and make the actual ordinary target-native comparator report the unsafe equality, the operational paper claim is No-Go.

**Core-completeness condition:** E02 or an externally approved nontrivial replacement must close before the artifact is submission-ready.

### Phase 6 — deterministic release

Bind manifests, replay witnesses, generate the single result owner and paper table, and verify clean reproduction.

No natural vulnerability, corpus result, SMT extension, third native fixture, or performance result is required before this phase can succeed.

---

## 24. Final Core acceptance gate

The artifact is ready to support paper writing only when all of the following hold:

- [ ] the external expert approved v0.3.1a;
- [ ] `lake build` checks L1–L9 without placeholders;
- [ ] the Rust checker implements exactly the Core IR;
- [ ] all required scopes bind a comparator and comparator hash;
- [ ] every request not designated as an invalid regression passes the §3.13 profile/property table, every non-`none` Match-coverage formula has its quantified §3.10 result, and V02 alone matches its expected invalid-coverage oracle;
- [ ] A01/A02/A03/A05 match the categorical matrix;
- [ ] H01/H02/H04 match the categorical matrix;
- [ ] V01’s two runs prove comparator divergence;
- [ ] V02 is invalid, V06 is unknown, and V10 distinguishes precision;
- [ ] T01 classifies safe, safe, harmful while all three candidates preserve every requested law;
- [ ] T02 is sound but `law-breaking-or-inapplicable`, never `lawful-safe`;
- [ ] the policy-vacuity conformance test withholds certification;
- [ ] C01 proves only total exact/preorder composition;
- [ ] A01, A02, and A05 harmful twists are automatically generated;
- [ ] every A01/A02/A03/A05 base check passes definedness, all six laws, soundness, adequacy, precision, and faithfulness under the identical comparator/scope/policy/request used by its twist;
- [ ] every required attack reports `base_alignment_status=proved-exhaustive` and `candidate_binding_status=proved-exhaustive`, including base-check/base-context and transformed-check/generated-context hash equality;
- [ ] every transformation report binds normalized transformation/inverse IR, the action domain, conjugated four-map hashes, inverse/construction checks, requested laws, classification, and any applicable transformed bridge;
- [ ] all three comparator directions, `ModularAffine`, and nested structural enumeration pass their Core microtests;
- [ ] BL2 accepts the lawful attacks at the round-trip layer;
- [ ] BL4 matches every common validity, Match-coverage, policy, TNA/property, and shared-witness status required by the exact `bl4_paired=true` registry;
- [ ] E01 and E02 reproduce ordinary target-native false agreement and bind the exact candidate hashes of their corresponding A01/A02 reference checks;
- [ ] native adapters and comparator truth tables have zero backend mismatches;
- [ ] every workspace/artifact path is repository-relative and the only absolute paths are bound image-internal manifest fields;
- [ ] the source/evidence hash graph is acyclic, validates topologically, and all builds use a read-only source tree with external output roots;
- [ ] native builds and runs are bound to image, environment, input, toolchain, and dependencies;
- [ ] all required witnesses replay;
- [ ] one command regenerates and byte-checks `results.json` and the paper table, whose rows retain request/check hashes, profile, coverage, policy, certification, and transformation-binding evidence;
- [ ] the claim manifest rejects forbidden overstatement; and
- [ ] no Core claim relies on an Extended artifact.

The §2.6 disclosure classification is a paper-submission gate, not authorization for an implementation agent to send an external message and not a prerequisite for beginning an externally approved Core implementation.

---

## 25. Questions for the external v0.3.1a reviewer

The reviewer is asked to answer:

1. Does §6.3 now restrict `lawful-safe` and `lawful-harmful` to well-typed, inverse-valid, mechanically conjugated, comparator-defined transformations that pass every law explicitly requested by the hashed request?
2. Do T01 and T02 exclude both residual errors: policy non-closure caused merely by law breakage, and a sound but law-breaking candidate mislabeled safe?
3. Do the quantified §3.10 formulas give `nonempty`, `source-total`, `target-total`, and `bidirectional-total` unambiguous meaning over the full comparison domains?
4. Does the §3.13 profile/property table prevent silent property insertion, silent law insertion, and certification under missing Match coverage?
5. Does §17.7 bind enough provenance to reconstruct and independently check the aligned base, transformation, inverse, complete action domain, target-side carrier conjugation, generated candidate, four maps, requested laws, classification, witness, native reference, and any transformed bridge?
6. Do these bounded changes preserve the already accepted comparator-indexed, total-success, BL4-parity, and native-replay Core?
7. May implementation begin with v0.3.1a as the sole source of truth?

### 25.1 Requested decision

Requested approval wording:

> **Go for implementation of the v0.3.1a GlueRift Minimal Core.** Comparator-indexed checking and the total-success Core remain coherent. Transformation analysis now classifies only well-typed, inverse-valid, mechanically conjugated, comparator-defined, request-law-preserving candidates as lawful; Match coverage has explicit quantified semantics; profile/property requirements are checked; and generated transformations bind their aligned base, transformation, inverse, complete action domain, four-map construction, generated candidate, requested laws, classification, native reference, and applicable bridge evidence. No further research redesign is required.

Possible reviewer outcomes:

```text
GO
CONDITIONAL GO — list bounded corrections
NO-GO — identify the failed theorem, construct, or contribution
```

---

# Appendix A — Required related-work anchors

The final paper MUST cite and compare at least the following primary sources. This list is a minimum, not a substitute for the final novelty search.

1. H. Zhang et al., [“Validated Code Translation for Projects with External Libraries.”](https://arxiv.org/abs/2602.18534), 2026.  
   Relevance: the motivating recent Go/Rust/Protobuf adapter-validation architecture and target-native differential comparison.

2. D. Patterson, N. Mushtak, A. Wagner, and A. Ahmed, [“Semantic Soundness for Language Interoperability.”](https://pldi22.sigplan.org/details/pldi-2022-pldi/36/Semantic-Soundness-for-Language-Interoperability), PLDI 2022, [DOI 10.1145/3519939.3523703](https://doi.org/10.1145/3519939.3523703).  
   Relevance: cross-language type convertibility, target-level glue conversion, and soundness relative to a semantic relation. GlueRift does not claim this defense basis as new.

3. M. S. Abid et al., [“GlueTest: Testing Code Translation via Language Interoperability.”](https://www.cs.cornell.edu/~saikatd/papers/gluetest-icsme-nier24.pdf), ICSME NIER 2024.  
   Relevance: interoperability glue as an additional source of translation inconsistency.

4. J. N. Foster et al., [“Combinators for Bi-Directional Tree Transformations: A Linguistic Approach to the View-Update Problem.”](https://www.cis.upenn.edu/~bcpierce/papers/lenses-toplas-final.pdf), TOPLAS 2007.  
   Relevance: classic lens laws and compositional bidirectional programming.

5. M. Hofmann, B. C. Pierce, and D. Wagner, [“Symmetric Lenses.”](https://dl.acm.org/doi/10.1145/1925844.1926428), POPL 2011.  
   Relevance: explicit consistency relations between endpoint domains.

6. J. N. Foster, A. Pilkiewicz, and B. C. Pierce, [“Quotient Lenses.”](https://doi.org/10.1145/1411204.1411257), ICFP 2008.  
   Relevance: well-behavedness modulo declared equivalence.

7. H. Zhang et al., [“Contract Lenses: Reasoning about Bidirectional Programs via Calculation.”](https://www.cambridge.org/core/journals/journal-of-functional-programming/article/contract-lenses-reasoning-about-bidirectional-programs-via-calculation/43F612938DAA399A9D35193FB6278F56), JFP 2023.  
   Relevance: predicates, partial lenses, and modular calculation.

8. J. Carette, B. A. Yorgey, and A. Sabry, [“Optics and Type Equivalences.”](https://www.cas.mcmaster.ca/~carette/publications/OpticsAndTypeEquivalences.pdf), 2019.  
   Relevance: automorphisms of finite type equivalences.

9. N. Moriakov, J. Adler, and J. Teuwen, [“Kernel of CycleGAN as a Principal Homogeneous Space.”](https://arxiv.org/abs/2001.09061), 2020.  
   Relevance: automorphism invariance of exact cycle-consistent solutions.

10. L.-Y. Xia, D. Orchard, and M. Wang, [“Composing Bidirectional Programs Monadically.”](https://link.springer.com/chapter/10.1007/978-3-030-17184-1_6), ESOP 2019.  
    Relevance: effectful bidirectional composition and the scope of round-trip properties. Effectful composition is Extended here.

### Administrative naming note

The former working name is retired because Lee et al. published [“RefLens: End-to-End Evidence-Grounded Citation Verification with LLM Agents”](https://ojs.aaai.org/index.php/AAAI/article/view/42361) at AAAI 2026. The fields differ, but the collision is avoidable. `GlueRift` is a working name and may still undergo repository/trademark screening before public release without changing the formal contract.

---

# Appendix B — Required positioning language

The final paper SHOULD use wording substantively equivalent to:

> Bidirectional-transformation research already provides round-trip laws, explicit consistency relations, partial-domain contracts, and compositional calculi. Cycle-consistency research also shows automorphism ambiguity, and language-interoperability research verifies glue conversion relative to semantic relations. We do not claim those ideas as new. We instantiate the ambiguity as an operational target-native false-agreement attack on a recent translation-validation workflow, distinguish unsafe equality from vacuous non-comparison, and provide a finite diagnostic and native replay artifact.

For the motivating architecture:

> A recent 2026 preprint describes a Go-to-Rust architecture that checks adapter round trips before reusing the adapters for differential I/O comparison. Our architecture-faithful reconstruction preserves that comparison direction and checks `TargetNativeExact` directly. We do not claim that an untested public implementation is vulnerable.

For the checker:

> GlueRift does not discover endpoint meaning. It accepts trusted endpoint observers, a safety relation, a required-match relation, a comparison universe, and the actual comparator kind. It checks the selected induced relation exhaustively in a restricted finite scope and reports unsupported semantics explicitly.

For carrier evidence:

> Carrier equality is one possible comparator, not a universal model of native comparison. A carrier summary applies to a native comparator only when a separately checked bridge is proved over the declared universe.

For BL4:

> A complete Direct-Relation author can express the same top-level obligations and receives the same comparator, semantic inputs, totality, validity, coverage, and finite evaluator. GlueRift’s evaluated delta is limited to demonstrated differences in structured derivations, carrier diagnostics, harmful-twist generation, and native evidence binding—not greater logical expressiveness.

---

# Appendix C — External-review traceability

| Review item | Binding resolution |
|---|---|
| P0-1 actual comparator differs from carrier equality | §§3.3–3.4, 4, 10.4 V01, 17, 19 |
| All properties must be comparator-indexed | §§3.9, 6.3, 11.2, 17 |
| Bridge must be optional evidence | §§4.1–4.3, 6.4, 15.1 |
| Comparator-specific Match shape | §3.12 and Lean L7 |
| P0-2 failure permission loses input | effectful calculus removed from Core; corrected future rule in §8.4 and §21.2 |
| Empty safety dimensions | §3.11, §10.4, §§17.1 and 20.3 |
| Direct interoperability prior work | §§2.3–2.4 and Appendix A.2 |
| RefLens name collision | §§0.1 and 1.1; Appendix A naming note |
| Native environment/build binding | §§19.2–19.6 |
| Overbroad “commonly” language | §§1.2–1.3, 2.1, Appendix B |
| Minimal Core reduction | §§10–12 and §21 |
| P0 transformation lawfulness missing from safe/harmful sets | §§5.4, 6.3, 10.4 T01/T02, 14.2 L8, 16, 17.7, 20 |
| P1 Match coverage modes lack quantified meaning | §§3.10, 7.5–7.7, 10.1, 17.1 |
| P1 generated-transformation provenance incomplete | §§6.3, 13.4, 16.2–16.3, 17.7, 20 |
| Profile/property and attack-law consistency | §§3.13, 7.6–7.7, 10.1–10.2, 10.6, 17.1 |
| Venue disclosure handling | §§2.6 and 24 |

---

# Appendix D — Normative delta from v0.3

The following v0.3 concepts are expressly deleted from Core:

```text
unqualified E_A
carrier equality as the default operational comparator
bridge inference from six round trips
OutcomeContract
observer-quotiented primary error relation
effectful composition theorem
partial Core adapters
Option and BoundedList
general Predicate IR
AffineQuantity and unit/lattice registries
CheckedCast and Restrict
A04 and H03
E03
BL0 and BL3 release gates
ten-regression requirement
automatic scalar-template discovery
general graph automorphism
SMT
code generation
```

No implementation may reintroduce one of these as a prerequisite for a v0.3.1a Core claim without a new reviewed contract version.

---

# Appendix E — Normative delta from v0.3.1

Version 0.3.1a makes only the following bounded changes to the already accepted v0.3.1 Core:

1. it replaces soundness-only transformation buckets with the request-scoped lawful three-way partition in §6.3;
2. it fixes T01 and T02 as conformance counterexamples for lawful asymmetric non-closure and sound-but-law-breaking inapplicability;
3. it gives every Match-coverage mode a quantified finite-domain definition and typed status;
4. it makes profile/property compatibility and explicit all-six attack law requests machine-checked;
5. it expands generated-transformation evidence to include normalized transformation and inverse IR, complete action domain, exact four-map conjugation, aligned-base proof, generated-candidate binding, lawfulness, classification, witness, exact native-reference binding, and any transformed bridge; and
6. it adds a pre-submission disclosure record without authorizing external contact.

No comparator definition, comparison property, bridge rule, total-success composition rule, baseline parity obligation, native fixture, or Core/Extended boundary is expanded by this delta.
