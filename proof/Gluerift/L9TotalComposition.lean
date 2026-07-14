import Gluerift.Core

/-!
L9: composition for the sole Core judgment: typed, total-success, domain-
contained value transformations.  There is no meta-level allowed-error branch.
-/

namespace GlueRift.L9

/-- Closed total-success value judgment from contract §8.1. -/
def ValueJudgment
    (f : X → Y) (domainX : X → Prop) (domainY : Y → Prop)
    (observeX : X → OX) (observeY : Y → OY)
    (relation : OX → OY → Prop) : Prop :=
  ∀ x, domainX x →
    domainY (f x) ∧ relation (observeX x) (observeY (f x))

/-- Contract Theorem 6 with explicit intermediate-domain and relation premises. -/
theorem total_success_value_judgments_compose
    {f : X → Y} {g : Y → Z}
    {domainX : X → Prop} {domainY : Y → Prop} {domainZ : Z → Prop}
    {observeX : X → OX} {observeY : Y → OY} {observeZ : Z → OZ}
    {r₁ : OX → OY → Prop} {r₂ : OY → OZ → Prop} {r : OX → OZ → Prop}
    (hf : ValueJudgment f domainX domainY observeX observeY r₁)
    (hg : ValueJudgment g domainY domainZ observeY observeZ r₂)
    (relationComposition :
      ∀ ox oy oz, r₁ ox oy → r₂ oy oz → r ox oz) :
    ValueJudgment (fun x => g (f x)) domainX domainZ observeX observeZ r := by
  intro x hx
  obtain ⟨hy, hr₁⟩ := hf x hx
  obtain ⟨hz, hr₂⟩ := hg (f x) hy
  exact ⟨hz, relationComposition _ _ _ hr₁ hr₂⟩

/-- Exact-observer judgments compose by equality transitivity. -/
theorem exact_total_success_value_judgments_compose
    {f : X → Y} {g : Y → Z}
    {domainX : X → Prop} {domainY : Y → Prop} {domainZ : Z → Prop}
    {observeX : X → O} {observeY : Y → O} {observeZ : Z → O}
    (hf : ValueJudgment f domainX domainY observeX observeY Eq)
    (hg : ValueJudgment g domainY domainZ observeY observeZ Eq) :
    ValueJudgment (fun x => g (f x)) domainX domainZ observeX observeZ Eq := by
  apply total_success_value_judgments_compose hf hg
  intro ox oy oz hxy hyz
  exact hxy.trans hyz

/-- Proof object emitted after exhaustive reflexivity/transitivity validation. -/
structure VerifiedFinitePreorder (L : Type u) where
  le : L → L → Prop
  reflexive : ∀ x, le x x
  transitive : ∀ x y z, le x y → le y z → le x z

/-- Target-non-amplification relation orientation: target is below source. -/
def TargetNoAmplification (order : VerifiedFinitePreorder L) (source target : L) : Prop :=
  order.le target source

/-- Verified-preorder non-amplification judgments compose by transitivity. -/
theorem verified_preorder_total_success_value_judgments_compose
    (order : VerifiedFinitePreorder L)
    {f : X → Y} {g : Y → Z}
    {domainX : X → Prop} {domainY : Y → Prop} {domainZ : Z → Prop}
    {observeX : X → L} {observeY : Y → L} {observeZ : Z → L}
    (hf : ValueJudgment f domainX domainY observeX observeY
      (TargetNoAmplification order))
    (hg : ValueJudgment g domainY domainZ observeY observeZ
      (TargetNoAmplification order)) :
    ValueJudgment (fun x => g (f x)) domainX domainZ observeX observeZ
      (TargetNoAmplification order) := by
  apply total_success_value_judgments_compose hf hg
  intro ox oy oz hyx hzy
  exact order.transitive oz oy ox hzy hyx

end GlueRift.L9
