import Std
import Gluerift.Core

/-!
L5: the two comparison vacuities, the carrier-only consequence of disjoint
encoder images, and the executable V01 comparator-divergence model.
-/

namespace GlueRift.L5

/-- Empty selected equality makes one-sided policy soundness vacuous. -/
theorem selected_relation_emptiness_makes_soundness_vacuous
    {induced safe scope : S → T → Prop}
    (empty : ∀ s t, ¬ induced s t) : Sound induced safe scope := by
  intro s t _ hInduced
  exact (empty s t hInduced).elim

/-- A nonempty Match defeats adequacy when selected equality is empty. -/
theorem nonempty_match_defeats_adequacy_of_empty_relation
    {induced matchRel scope : S → T → Prop}
    (empty : ∀ s t, ¬ induced s t)
    (nonempty : ∃ s t, matchRel s t) : ¬ Adequate induced matchRel scope := by
  intro adequate
  obtain ⟨s, t, hMatch⟩ := nonempty
  exact empty s t (adequate s t hMatch).2

/-- Disjoint encoder images prove only that carrier-exact equality is empty. -/
theorem disjoint_encoder_images_make_carrier_exact_empty
    {eS : S → C} {eT : T → C}
    (disjoint : ∀ s t, eS s ≠ eT t) :
    ∀ s t, ¬ CarrierExact eS eT s t := by
  intro s t hCarrier
  exact disjoint s t hCarrier

namespace V01

inductive Source where
  | s0 | s1
  deriving DecidableEq, Repr

inductive Target where
  | t0 | t1
  deriving DecidableEq, Repr

inductive Carrier where
  | l0 | l1 | r0 | r1
  deriving DecidableEq, Repr

def sources : List Source := [.s0, .s1]
def targets : List Target := [.t0, .t1]
def carriers : List Carrier := [.l0, .l1, .r0, .r1]
def sourceCarrierDomain : List Carrier := [.l0, .l1]
def targetCarrierDomain : List Carrier := [.r0, .r1]

def eS : Source → Carrier
  | .s0 => .l0
  | .s1 => .l1

def eT : Target → Carrier
  | .t0 => .r0
  | .t1 => .r1

def dS : Carrier → Source
  | .l0 | .r0 => .s0
  | .l1 | .r1 => .s1

def dT : Carrier → Target
  | .l0 | .r0 => .t0
  | .l1 | .r1 => .t1

def sourceNativePass : Bool :=
  sources.all fun s => decide (dS (eS s) = s)

def targetNativePass : Bool :=
  targets.all fun t => decide (dT (eT t) = t)

def sourceCarrierPass : Bool :=
  sourceCarrierDomain.all fun c => decide (eS (dS c) = c)

def targetCarrierPass : Bool :=
  targetCarrierDomain.all fun c => decide (eT (dT c) = c)

def sourceFullTransportPass : Bool :=
  sources.all fun s => decide (transportSTS eS dS eT dT s = s)

def targetFullTransportPass : Bool :=
  targets.all fun t => decide (transportTST eS dS eT dT t = t)

def allSixPass : Bool :=
  sourceNativePass && targetNativePass &&
  sourceCarrierPass && targetCarrierPass &&
  sourceFullTransportPass && targetFullTransportPass

def carrierRel (s : Source) (t : Target) : Prop := CarrierExact eS eT s t
def targetRel (s : Source) (t : Target) : Prop := TargetNativeExact eS dT s t
def sourceRel (s : Source) (t : Target) : Prop := SourceNativeExact eT dS s t

def aligned : Source → Target → Prop
  | .s0, .t0 => True
  | .s1, .t1 => True
  | _, _ => False

/-- Non-universal safety table frozen by §4.4. -/
def safe : Source → Target → Prop
  | .s0, .t1 => True
  | .s1, .t0 => True
  | _, _ => False

theorem all_six_roundtrips_pass : allSixPass = true := by decide

theorem source_and_target_encoder_images_are_disjoint :
    ∀ s t, eS s ≠ eT t := by
  intro s t
  cases s <;> cases t <;> decide

theorem carrier_relation_is_empty : ∀ s t, ¬ carrierRel s t := by
  exact disjoint_encoder_images_make_carrier_exact_empty
    source_and_target_encoder_images_are_disjoint

theorem target_native_relation_is_exactly_aligned (s : Source) (t : Target) :
    targetRel s t ↔ aligned s t := by
  cases s <;> cases t <;>
    simp [targetRel, TargetNativeExact, eS, dT, aligned]

theorem source_native_relation_is_exactly_aligned (s : Source) (t : Target) :
    sourceRel s t ↔ aligned s t := by
  cases s <;> cases t <;>
    simp [sourceRel, SourceNativeExact, eT, dS, aligned]

/-- Disjoint carrier images do not imply empty target-native equality. -/
theorem target_native_relation_is_nonempty : ∃ s t, targetRel s t := by
  exact ⟨.s0, .t0, rfl⟩

/-- The V01 target-native run is policy-unsafe despite all six round trips. -/
theorem target_native_soundness_fails :
    ¬ Sound targetRel safe (fun _ _ => True) := by
  intro hSound
  exact hSound .s0 .t0 trivial rfl

def carrierRelB (s : Source) (t : Target) : Bool := decide (eS s = eT t)

def targetRelB (s : Source) (t : Target) : Bool := decide (dT (eS s) = t)

def sourceRelB (s : Source) (t : Target) : Bool := decide (dS (eT t) = s)

def alignedB : Source → Target → Bool
  | .s0, .t0 => true
  | .s1, .t1 => true
  | _, _ => false

/-- Machine-reducible categorical V01 output. -/
def executableReport : List (String × Bool) :=
  [ ("source-native-roundtrip", sourceNativePass)
  , ("target-native-roundtrip", targetNativePass)
  , ("source-carrier-roundtrip", sourceCarrierPass)
  , ("target-carrier-roundtrip", targetCarrierPass)
  , ("source-full-transport", sourceFullTransportPass)
  , ("target-full-transport", targetFullTransportPass)
  , ("carrier-relation-empty",
      sources.all fun s => targets.all fun t => !(carrierRelB s t))
  , ("target-native-aligned",
      sources.all fun s => targets.all fun t => targetRelB s t == alignedB s t)
  , ("source-native-aligned",
      sources.all fun s => targets.all fun t => sourceRelB s t == alignedB s t)
  ]

theorem executable_report_is_categorical :
    executableReport.all (fun entry => entry.2) = true := by rfl

#eval executableReport

end V01
end GlueRift.L5
