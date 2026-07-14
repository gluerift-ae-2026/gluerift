/-!
# GlueRift total finite/shared-domain core

This module formalizes only the total mathematical layer specified by contract
v0.3.1a §§3--8 and §14.  Meta-level conversion failures, the Rust evaluator,
native backends, Protobuf, and Extended profiles are deliberately outside this
model.
-/

namespace GlueRift

/-- A total isomorphism, used for the clean shared-domain attack spine. -/
structure Iso (α : Type u) (β : Type v) where
  toFun : α → β
  invFun : β → α
  leftInv : ∀ x, invFun (toFun x) = x
  rightInv : ∀ y, toFun (invFun y) = y

namespace Iso

instance : CoeFun (Iso α β) (fun _ => α → β) := ⟨Iso.toFun⟩

@[simp] theorem inv_to (e : Iso α β) (x : α) : e.invFun (e x) = x :=
  e.leftInv x

@[simp] theorem to_inv (e : Iso α β) (y : β) : e (e.invFun y) = y :=
  e.rightInv y

/-- Identity isomorphism. -/
def refl (α : Type u) : Iso α α where
  toFun := id
  invFun := id
  leftInv := by intro x; rfl
  rightInv := by intro x; rfl

/-- Inverse isomorphism. -/
def symm (e : Iso α β) : Iso β α where
  toFun := e.invFun
  invFun := e.toFun
  leftInv := e.rightInv
  rightInv := e.leftInv

/-- Right-to-left function composition of isomorphisms. -/
def trans (e : Iso α β) (f : Iso β γ) : Iso α γ where
  toFun := fun x => f (e x)
  invFun := fun z => e.invFun (f.invFun z)
  leftInv := by intro x; simp
  rightInv := by intro z; simp

end Iso

/-- Native round trip on an independently supplied admitted domain. -/
def NativeRoundTrip (encode : X → C) (decode : C → X) (domain : X → Prop) : Prop :=
  ∀ x, domain x → decode (encode x) = x

/-- Carrier-domain round trip on an independently supplied carrier domain. -/
def CarrierRoundTrip (encode : X → C) (decode : C → X) (domain : C → Prop) : Prop :=
  ∀ c, domain c → encode (decode c) = c

/-- Source-to-target-to-source transport in the total mathematical core. -/
def transportSTS (eS : S → C) (dS : C → S) (eT : T → C) (dT : C → T)
    (s : S) : S :=
  dS (eT (dT (eS s)))

/-- Target-to-source-to-target transport in the total mathematical core. -/
def transportTST (eS : S → C) (dS : C → S) (eT : T → C) (dT : C → T)
    (t : T) : T :=
  dT (eS (dS (eT t)))

/-- Carrier-exact induced equality. -/
def CarrierExact (eS : S → C) (eT : T → C) (s : S) (t : T) : Prop :=
  eS s = eT t

/-- Target-native induced equality: the graph of source-to-target transport. -/
def TargetNativeExact (eS : S → C) (dT : C → T) (s : S) (t : T) : Prop :=
  dT (eS s) = t

/-- Source-native induced equality: the converse graph of target-to-source transport. -/
def SourceNativeExact (eT : T → C) (dS : C → S) (s : S) (t : T) : Prop :=
  dS (eT t) = s

/-- A relation is policy-sound within an independently supplied universe. -/
def Sound (induced safe scope : S → T → Prop) : Prop :=
  ∀ s t, scope s t → induced s t → safe s t

/-- A relation is adequate for every policy-owned required Match pair. -/
def Adequate (induced matchRel scope : S → T → Prop) : Prop :=
  ∀ s t, matchRel s t → scope s t ∧ induced s t

/-- A scoped induced equality contains no pair outside required Match. -/
def Precise (induced matchRel scope : S → T → Prop) : Prop :=
  ∀ s t, scope s t → induced s t → matchRel s t

/-- The scoped induced equality is exactly required Match. -/
def Faithful (induced matchRel scope : S → T → Prop) : Prop :=
  ∀ s t, (scope s t ∧ induced s t) ↔ matchRel s t

/-- Contract §3.9: faithfulness is exactly adequacy plus precision. -/
theorem faithful_iff_adequate_and_precise
    (induced matchRel scope : S → T → Prop) :
    Faithful induced matchRel scope ↔
      Adequate induced matchRel scope ∧ Precise induced matchRel scope := by
  constructor
  · intro faithful
    constructor
    · intro s t hMatch
      exact (faithful s t).2 hMatch
    · intro s t hScope hInduced
      exact (faithful s t).1 ⟨hScope, hInduced⟩
  · intro both s t
    constructor
    · intro hScoped
      exact both.2 s t hScoped.1 hScoped.2
    · intro hMatch
      exact both.1 s t hMatch

/-- A binary relation is functional from its first endpoint. -/
def Functional (r : A → B → Prop) : Prop :=
  ∀ a b₁ b₂, r a b₁ → r a b₂ → b₁ = b₂

/-- A binary relation is inverse-functional from its second endpoint. -/
def InverseFunctional (r : A → B → Prop) : Prop :=
  ∀ a₁ a₂ b, r a₁ b → r a₂ b → a₁ = a₂

/-- Target encoder after carrier conjugation. -/
def twistedTargetEncode (eT : T → C) (sigma : Iso C C) : T → C :=
  fun t => sigma (eT t)

/-- Target decoder after carrier conjugation. -/
def twistedTargetDecode (dT : C → T) (sigma : Iso C C) : C → T :=
  fun c => dT (sigma.invFun c)

end GlueRift
