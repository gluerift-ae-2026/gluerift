import Std
import Gluerift.Core

/-!
L8: exact stabilizer closure, the request-scoped three-way transformation
partition, and executable T01/T02 witnesses.  `lawful-safe` below means only
request-lawful plus selected-comparator sound, exactly as in contract §6.3.
-/

namespace GlueRift.L8

/-! ## Exact observer stabilizer -/

/-- A total permutation with a reported, two-sided inverse. -/
structure Permutation (α : Type u) where
  forward : α → α
  backward : α → α
  backward_forward : ∀ x, backward (forward x) = x
  forward_backward : ∀ x, forward (backward x) = x

namespace Permutation

/-- Identity permutation. -/
def identity (α : Type u) : Permutation α where
  forward := id
  backward := id
  backward_forward := by intro x; rfl
  forward_backward := by intro x; rfl

/-- Right-to-left permutation composition. -/
def compose (p q : Permutation α) : Permutation α where
  forward := fun x => p.forward (q.forward x)
  backward := fun x => q.backward (p.backward x)
  backward_forward := by
    intro x
    change q.backward (p.backward (p.forward (q.forward x))) = x
    calc
      q.backward (p.backward (p.forward (q.forward x))) =
          q.backward (q.forward x) :=
        congrArg q.backward (p.backward_forward (q.forward x))
      _ = x := q.backward_forward x
  forward_backward := by
    intro x
    change p.forward (q.forward (q.backward (p.backward x))) = x
    calc
      p.forward (q.forward (q.backward (p.backward x))) =
          p.forward (p.backward x) :=
        congrArg p.forward (q.forward_backward (p.backward x))
      _ = x := p.forward_backward x

/-- Permutation inverse. -/
def inverse (p : Permutation α) : Permutation α where
  forward := p.backward
  backward := p.forward
  backward_forward := p.forward_backward
  forward_backward := p.backward_forward

end Permutation

/-- Exact observation preservation. -/
def Stabilizes (observer : C → L) (p : Permutation C) : Prop :=
  ∀ c, observer (p.forward c) = observer c

/-- The subgroup closure obligations inside the finite permutation group. -/
structure IsPermutationSubgroup (members : Permutation C → Prop) : Prop where
  identity_mem : members (Permutation.identity C)
  compose_mem : ∀ p q, members p → members q → members (Permutation.compose p q)
  inverse_mem : ∀ p, members p → members (Permutation.inverse p)

/-- Contract Theorem 5: an exact observer's stabilizer is a subgroup. -/
theorem exact_observer_stabilizer_is_subgroup (observer : C → L) :
    IsPermutationSubgroup (Stabilizes observer) := by
  constructor
  · intro c
    rfl
  · intro p q hp hq c
    calc
      observer ((Permutation.compose p q).forward c) = observer (q.forward c) :=
        hp (q.forward c)
      _ = observer c := hq c
  · intro p hp c
    have h := hp (p.backward c)
    rw [p.forward_backward] at h
    exact h.symm

/-! ## Request-scoped lawful partition -/

def LawfulSafe (lawful sound : Candidate → Prop) (candidate : Candidate) : Prop :=
  lawful candidate ∧ sound candidate

def LawfulHarmful (lawful sound : Candidate → Prop) (candidate : Candidate) : Prop :=
  lawful candidate ∧ ¬ sound candidate

def LawBreakingOrInapplicable (lawful : Candidate → Prop) (candidate : Candidate) : Prop :=
  ¬ lawful candidate

/-- Lawful candidates split exhaustively into the sound and harmful classes. -/
theorem lawful_is_disjoint_union_of_safe_and_harmful
    (lawful sound : Candidate → Prop) [DecidablePred sound]
    (candidate : Candidate) :
    lawful candidate ↔
      LawfulSafe lawful sound candidate ∨ LawfulHarmful lawful sound candidate := by
  constructor
  · intro hLawful
    by_cases hSound : sound candidate
    · exact Or.inl ⟨hLawful, hSound⟩
    · exact Or.inr ⟨hLawful, hSound⟩
  · intro classified
    cases classified with
    | inl safe => exact safe.1
    | inr harmful => exact harmful.1

/-- Constructive finite-Core partition, parameterized by decidable complete checks. -/
theorem request_scoped_sets_form_exhaustive_partition
    (lawful sound : Candidate → Prop)
    [DecidablePred lawful] [DecidablePred sound]
    (candidate : Candidate) :
    LawfulSafe lawful sound candidate ∨
    LawfulHarmful lawful sound candidate ∨
    LawBreakingOrInapplicable lawful candidate := by
  by_cases hLawful : lawful candidate
  · by_cases hSound : sound candidate
    · exact Or.inl ⟨hLawful, hSound⟩
    · exact Or.inr (Or.inl ⟨hLawful, hSound⟩)
  · exact Or.inr (Or.inr hLawful)

theorem lawful_safe_and_lawful_harmful_are_disjoint
    {lawful sound : Candidate → Prop} {candidate : Candidate}
    (safe : LawfulSafe lawful sound candidate)
    (harmful : LawfulHarmful lawful sound candidate) : False :=
  harmful.2 safe.2

theorem lawful_safe_and_inapplicable_are_disjoint
    {lawful sound : Candidate → Prop} {candidate : Candidate}
    (safe : LawfulSafe lawful sound candidate)
    (inapplicable : LawBreakingOrInapplicable lawful candidate) : False :=
  inapplicable safe.1

theorem lawful_harmful_and_inapplicable_are_disjoint
    {lawful sound : Candidate → Prop} {candidate : Candidate}
    (harmful : LawfulHarmful lawful sound candidate)
    (inapplicable : LawBreakingOrInapplicable lawful candidate) : False :=
  inapplicable harmful.1

inductive Classification where
  | lawfulSafe
  | lawfulHarmful
  | lawBreakingOrInapplicable
  deriving DecidableEq, Repr

def classify (lawful sound : Bool) : Classification :=
  if lawful then
    if sound then .lawfulSafe else .lawfulHarmful
  else
    .lawBreakingOrInapplicable

/-! ## T01: lawful asymmetric non-closure -/

namespace T01

inductive Atom where
  | a | b | c
  deriving DecidableEq, Repr

inductive PolicyLevel where
  | deny | allow
  deriving DecidableEq, Repr

def atoms : List Atom := [.a, .b, .c]

def sigma1 : Permutation Atom where
  forward
    | .a => .b
    | .b => .a
    | .c => .c
  backward
    | .a => .b
    | .b => .a
    | .c => .c
  backward_forward := by intro x; cases x <;> rfl
  forward_backward := by intro x; cases x <;> rfl

def sigma2 : Permutation Atom where
  forward
    | .a => .a
    | .b => .c
    | .c => .b
  backward
    | .a => .a
    | .b => .c
    | .c => .b
  backward_forward := by intro x; cases x <;> rfl
  forward_backward := by intro x; cases x <;> rfl

/-- Right-to-left `sigma1 ∘ sigma2`, as fixed by contract §6.3. -/
def composite : Permutation Atom := Permutation.compose sigma1 sigma2

def sourcePolicy : Atom → PolicyLevel
  | .a => .deny
  | .b | .c => .allow

def targetPolicy : Atom → PolicyLevel
  | .a | .b => .deny
  | .c => .allow

def noAmplification : PolicyLevel → PolicyLevel → Bool
  | .deny, _ => true
  | .allow, .allow => true
  | .allow, .deny => false

def safeB (s t : Atom) : Bool := noAmplification (targetPolicy t) (sourcePolicy s)

/-- Four aligned identity-base maps and their mechanically conjugated context. -/
def eSBase (x : Atom) : Atom := x
def dSBase (x : Atom) : Atom := x
def eTBase (x : Atom) : Atom := x
def dTBase (x : Atom) : Atom := x
def eSTwisted (_p : Permutation Atom) : Atom → Atom := eSBase
def dSTwisted (_p : Permutation Atom) : Atom → Atom := dSBase
def eTTwisted (p : Permutation Atom) (x : Atom) : Atom := p.forward (eTBase x)
def dTTwisted (p : Permutation Atom) (x : Atom) : Atom := dTBase (p.backward x)

/-- Target-native graph for identity base maps twisted by `p`. -/
def targetNativeB (p : Permutation Atom) (s t : Atom) : Bool :=
  decide (dTTwisted p (eSTwisted p s) = t)

def sourceNativeLaw (p : Permutation Atom) : Bool :=
  atoms.all fun s => decide (dSTwisted p (eSTwisted p s) = s)

def sourceCarrierLaw (p : Permutation Atom) : Bool :=
  atoms.all fun x => decide (eSTwisted p (dSTwisted p x) = x)

def targetNativeLaw (p : Permutation Atom) : Bool :=
  atoms.all fun t => decide (dTTwisted p (eTTwisted p t) = t)

def targetCarrierLaw (p : Permutation Atom) : Bool :=
  atoms.all fun x => decide (eTTwisted p (dTTwisted p x) = x)

def sourceFullTransportLaw (p : Permutation Atom) : Bool :=
  atoms.all fun s => decide
    (transportSTS (eSTwisted p) (dSTwisted p) (eTTwisted p) (dTTwisted p) s = s)

def targetFullTransportLaw (p : Permutation Atom) : Bool :=
  atoms.all fun t => decide
    (transportTST (eSTwisted p) (dSTwisted p) (eTTwisted p) (dTTwisted p) t = t)

def allSixRequestedLaws (p : Permutation Atom) : Bool :=
  sourceNativeLaw p && targetNativeLaw p &&
  sourceCarrierLaw p && targetCarrierLaw p &&
  sourceFullTransportLaw p && targetFullTransportLaw p

def completeInverseCheck (p : Permutation Atom) : Bool :=
  atoms.all fun x =>
    decide (p.backward (p.forward x) = x) &&
    decide (p.forward (p.backward x) = x)

def targetNativeDefined (_p : Permutation Atom) : Bool := true
def wellTyped (_p : Permutation Atom) : Bool := true

/-- Exhaustive four-map check for the identity base and target-side conjugation. -/
def constructedByConjugation (p : Permutation Atom) : Bool :=
  atoms.all fun x =>
    decide (eSTwisted p x = eSBase x) &&
    decide (dSTwisted p x = dSBase x) &&
    decide (eTTwisted p x = p.forward (eTBase x)) &&
    decide (dTTwisted p x = dTBase (p.backward x))

def lawful (p : Permutation Atom) : Bool :=
  wellTyped p && completeInverseCheck p && constructedByConjugation p &&
  targetNativeDefined p && allSixRequestedLaws p

def sound (p : Permutation Atom) : Bool :=
  atoms.all fun s => atoms.all fun t =>
    if targetNativeB p s t then safeB s t else true

def classification (p : Permutation Atom) : Classification :=
  classify (lawful p) (sound p)

theorem sigma1_preserves_every_requested_law : allSixRequestedLaws sigma1 = true := by
  rfl

theorem sigma2_preserves_every_requested_law : allSixRequestedLaws sigma2 = true := by
  rfl

theorem composite_preserves_every_requested_law :
    allSixRequestedLaws composite = true := by
  rfl

theorem sigma1_is_lawful_safe : classification sigma1 = .lawfulSafe := by
  rfl

theorem sigma2_is_lawful_safe : classification sigma2 = .lawfulSafe := by
  rfl

theorem composite_is_lawful_harmful :
    classification composite = .lawfulHarmful := by
  rfl

theorem composite_passes_every_lawfulness_condition_and_is_only_policy_unsound :
    wellTyped composite = true ∧
    completeInverseCheck composite = true ∧
    constructedByConjugation composite = true ∧
    targetNativeDefined composite = true ∧
    allSixRequestedLaws composite = true ∧
    sound composite = false := by
  decide

/-- The exact policy-only witness: composite(c)=a pairs target allow with source deny. -/
theorem composite_harm_is_policy_not_law_failure :
    composite.forward .c = .a ∧
    targetNativeB composite .a .c = true ∧
    safeB .a .c = false ∧
    allSixRequestedLaws composite = true := by
  decide

def executableReport : List (String × Classification × Bool) :=
  [ ("sigma1", classification sigma1, allSixRequestedLaws sigma1)
  , ("sigma2", classification sigma2, allSixRequestedLaws sigma2)
  , ("normalize(sigma1-compose-sigma2)", classification composite,
      allSixRequestedLaws composite)
  ]

#eval executableReport

end T01

/-! ## T02: sound but request-law-breaking is inapplicable -/

namespace T02

inductive Source where
  | x
  deriving DecidableEq, Repr

inductive Target where
  | a | b | c
  deriving DecidableEq, Repr

inductive Carrier where
  | zero | one | two
  deriving DecidableEq, Repr

def targetNativeDomain : List Target := [.a, .b]
def allTargets : List Target := [.a, .b, .c]
def sourceCarrierDomain : List Carrier := [.zero]
def targetCarrierDomain : List Carrier := [.zero, .one]
def sourceFullDomain : List Source := [.x]
def targetFullDomain : List Target := [.a]
def comparisonSources : List Source := [.x]
def comparisonTargets : List Target := [.a, .c]
def actionDomain : List Carrier := [.zero, .one, .two]

def eS : Source → Carrier | .x => .zero
def dS : Carrier → Source | _ => .x

def eT : Target → Carrier
  | .a => .zero
  | .b => .one
  | .c => .two

def dT : Carrier → Target
  | .zero => .a
  | .one => .b
  | .two => .a

def swapZeroTwo : Permutation Carrier where
  forward
    | .zero => .two
    | .one => .one
    | .two => .zero
  backward
    | .zero => .two
    | .one => .one
    | .two => .zero
  backward_forward := by intro x; cases x <;> rfl
  forward_backward := by intro x; cases x <;> rfl

def identityCarrier : Permutation Carrier := Permutation.identity Carrier

def eTTwisted (p : Permutation Carrier) (t : Target) : Carrier := p.forward (eT t)
def dTTwisted (p : Permutation Carrier) (c : Carrier) : Target := dT (p.backward c)
def eSTwisted (_p : Permutation Carrier) : Source → Carrier := eS
def dSTwisted (_p : Permutation Carrier) : Carrier → Source := dS

def sourceNativeLaw : Bool := decide (dS (eS .x) = .x)

def targetNativeLaw (p : Permutation Carrier) : Bool :=
  targetNativeDomain.all fun t => decide (dTTwisted p (eTTwisted p t) = t)

def sourceCarrierLaw : Bool :=
  sourceCarrierDomain.all fun c => decide (eS (dS c) = c)

def targetCarrierLaw (p : Permutation Carrier) : Bool :=
  targetCarrierDomain.all fun c => decide (eTTwisted p (dTTwisted p c) = c)

def sourceFullTransportLaw (p : Permutation Carrier) : Bool :=
  sourceFullDomain.all fun s =>
    decide (dS (eTTwisted p (dTTwisted p (eS s))) = s)

def targetFullTransportLaw (p : Permutation Carrier) : Bool :=
  targetFullDomain.all fun t =>
    decide (dTTwisted p (eS (dS (eTTwisted p t))) = t)

def allSixRequestedLaws (p : Permutation Carrier) : Bool :=
  sourceNativeLaw && targetNativeLaw p &&
  sourceCarrierLaw && targetCarrierLaw p &&
  sourceFullTransportLaw p && targetFullTransportLaw p

def completeInverseCheck (p : Permutation Carrier) : Bool :=
  actionDomain.all fun c =>
    decide (p.backward (p.forward c) = c) &&
    decide (p.forward (p.backward c) = c)

def safeB : Source → Target → Bool
  | .x, .a => true
  | .x, .c => false
  | .x, .b => false

def inUniverseB : Source → Target → Bool
  | .x, .a | .x, .c => true
  | _, _ => false

def targetNativeB (p : Permutation Carrier) (s : Source) (t : Target) : Bool :=
  decide (dTTwisted p (eS s) = t)

def sound (p : Permutation Carrier) : Bool :=
  comparisonSources.all fun s => comparisonTargets.all fun t =>
    if inUniverseB s t && targetNativeB p s t then safeB s t else true

def wellTyped (_p : Permutation Carrier) : Bool := true

/-- Exhaustive target four-map conjugation check on every native/carrier value. -/
def constructedByConjugation (p : Permutation Carrier) : Bool :=
  comparisonSources.all (fun s =>
    decide (eSTwisted p s = eS s)) &&
  actionDomain.all (fun c =>
    decide (dSTwisted p c = dS c)) &&
  allTargets.all (fun t =>
    decide (eTTwisted p t = p.forward (eT t))) &&
  actionDomain.all (fun c =>
    decide (dTTwisted p c = dT (p.backward c)))
def targetNativeDefined (_p : Permutation Carrier) : Bool := true

def lawful (p : Permutation Carrier) : Bool :=
  wellTyped p && completeInverseCheck p && constructedByConjugation p &&
  targetNativeDefined p && allSixRequestedLaws p

def classification (p : Permutation Carrier) : Classification :=
  classify (lawful p) (sound p)

theorem base_context_passes_all_six_requested_laws :
    allSixRequestedLaws identityCarrier = true := by
  rfl

theorem swap_is_selected_comparator_sound : sound swapZeroTwo = true := by
  rfl

theorem swap_fails_target_carrier_roundtrip_at_zero :
    eTTwisted swapZeroTwo (dTTwisted swapZeroTwo .zero) = .two ∧
    eTTwisted swapZeroTwo (dTTwisted swapZeroTwo .zero) ≠ .zero := by
  decide

theorem swap_breaks_a_requested_law :
    allSixRequestedLaws swapZeroTwo = false := by
  rfl

theorem swap_passes_nonlaw_policy_prerequisites_but_fails_requested_laws :
    wellTyped swapZeroTwo = true ∧
    completeInverseCheck swapZeroTwo = true ∧
    constructedByConjugation swapZeroTwo = true ∧
    targetNativeDefined swapZeroTwo = true ∧
    sound swapZeroTwo = true ∧
    allSixRequestedLaws swapZeroTwo = false := by
  decide

theorem sound_but_law_breaking_swap_is_inapplicable_not_safe :
    classification swapZeroTwo = .lawBreakingOrInapplicable := by
  rfl

def executableReport : List (String × Bool) :=
  [ ("base-all-six", allSixRequestedLaws identityCarrier)
  , ("twist-inverse-complete", completeInverseCheck swapZeroTwo)
  , ("twist-target-native-defined", targetNativeDefined swapZeroTwo)
  , ("twist-selected-sound", sound swapZeroTwo)
  , ("twist-target-carrier-law", targetCarrierLaw swapZeroTwo)
  , ("twist-all-six", allSixRequestedLaws swapZeroTwo)
  , ("twist-inapplicable",
      decide (classification swapZeroTwo = .lawBreakingOrInapplicable))
  ]

#eval executableReport

end T02
end GlueRift.L8
