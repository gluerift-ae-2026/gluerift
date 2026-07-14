import Gluerift.Core

/-!
L7: comparator-specific native-relation shape.  The unrestricted native graph
gets only its deterministic direction; the opposite direction is proved solely
for adequate Match pairs covered by the corresponding full transport law.
-/

namespace GlueRift.L7

/-- TargetNativeExact is functional because it is the graph of deterministic transport. -/
theorem target_native_exact_is_functional
    (eS : S → C) (dT : C → T) :
    Functional (TargetNativeExact eS dT) := by
  intro s t₁ t₂ h₁ h₂
  exact h₁.symm.trans h₂

/-- SourceNativeExact is inverse-functional because it is a converse graph. -/
theorem source_native_exact_is_inverse_functional
    (eT : T → C) (dS : C → S) :
    InverseFunctional (SourceNativeExact eT dS) := by
  intro s₁ s₂ t h₁ h₂
  exact h₁.symm.trans h₂

/--
Under adequacy and source full-transport coverage of Match's source projection,
Match is inverse-functional.  This does not assert injectivity of unrestricted
TargetNativeExact.
-/
theorem adequate_target_native_match_is_inverse_functional_on_covered_projection
    {eS : S → C} {dT : C → T} {back : T → S}
    {matchRel scope : S → T → Prop}
    (adequate : Adequate (TargetNativeExact eS dT) matchRel scope)
    (sourceFullTransportOnMatch :
      ∀ s, (∃ t, matchRel s t) → back (dT (eS s)) = s) :
    InverseFunctional matchRel := by
  intro s₁ s₂ t h₁ h₂
  have hf₁ : dT (eS s₁) = t := (adequate s₁ t h₁).2
  have hf₂ : dT (eS s₂) = t := (adequate s₂ t h₂).2
  calc
    s₁ = back (dT (eS s₁)) :=
      (sourceFullTransportOnMatch s₁ ⟨t, h₁⟩).symm
    _ = back t := congrArg back hf₁
    _ = back (dT (eS s₂)) := congrArg back hf₂.symm
    _ = s₂ := sourceFullTransportOnMatch s₂ ⟨t, h₂⟩

/--
Source-native dual: adequacy and target full-transport coverage of Match's target
projection make Match functional, without asserting unrestricted injectivity.
-/
theorem adequate_source_native_match_is_functional_on_covered_projection
    {eT : T → C} {dS : C → S} {forward : S → T}
    {matchRel scope : S → T → Prop}
    (adequate : Adequate (SourceNativeExact eT dS) matchRel scope)
    (targetFullTransportOnMatch :
      ∀ t, (∃ s, matchRel s t) → forward (dS (eT t)) = t) :
    Functional matchRel := by
  intro s t₁ t₂ h₁ h₂
  have hg₁ : dS (eT t₁) = s := (adequate s t₁ h₁).2
  have hg₂ : dS (eT t₂) = s := (adequate s t₂ h₂).2
  calc
    t₁ = forward (dS (eT t₁)) :=
      (targetFullTransportOnMatch t₁ ⟨s, h₁⟩).symm
    _ = forward s := congrArg forward hg₁
    _ = forward (dS (eT t₂)) := congrArg forward hg₂.symm
    _ = t₂ := targetFullTransportOnMatch t₂ ⟨s, h₂⟩

end GlueRift.L7
