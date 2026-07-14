import Gluerift.Core

/-!
L4: the clean witness is directly target-native.  No carrier/native bridge is
assumed or used.
-/

namespace GlueRift.L4

variable (source : Iso S C) (target : Iso T C) (sigma : Iso C C)

/-- Source endpoint selected by carrier witness `c`. -/
def witnessSource (c : C) : S := source.invFun (sigma c)

/-- Target endpoint selected by carrier witness `c`. -/
def witnessTarget (c : C) : T := target.invFun c

/-- The §5.3 witness lies in carrier-exact equality after the twist. -/
theorem witness_is_carrier_exact (c : C) :
    CarrierExact source.toFun (twistedTargetEncode target.toFun sigma)
      (witnessSource source sigma c) (witnessTarget target c) := by
  change source (source.invFun (sigma c)) = sigma (target (target.invFun c))
  calc
    source (source.invFun (sigma c)) = sigma c := source.rightInv (sigma c)
    _ = sigma (target (target.invFun c)) :=
      congrArg sigma.toFun (target.rightInv c).symm

/-- The §5.3 witness lies directly in target-native equality after the twist. -/
theorem witness_is_direct_target_native (c : C) :
    TargetNativeExact source.toFun (twistedTargetDecode target.invFun sigma)
      (witnessSource source sigma c) (witnessTarget target c) := by
  change target.invFun (sigma.invFun (source (source.invFun (sigma c)))) =
    target.invFun c
  calc
    target.invFun (sigma.invFun (source (source.invFun (sigma c)))) =
        target.invFun (sigma.invFun (sigma c)) :=
      congrArg target.invFun
        (congrArg sigma.invFun (source.rightInv (sigma c)))
    _ = target.invFun c := congrArg target.invFun (sigma.leftInv c)

/-- The same §5.3 witness also lies in source-native equality. -/
theorem witness_is_source_native (c : C) :
    SourceNativeExact (twistedTargetEncode target.toFun sigma) source.invFun
      (witnessSource source sigma c) (witnessTarget target c) := by
  change source.invFun (sigma (target (target.invFun c))) =
    source.invFun (sigma c)
  exact congrArg source.invFun
    (congrArg sigma.toFun (target.rightInv c))

/--
Contract Corollary 4's load-bearing semantic implication for the actual
target-native comparator: an in-universe unsafe clean witness is laundering.
Family admission and requested-law lawfulness are deliberately separate
premises of the request-scoped L8 classification.
-/
theorem unsafe_clean_witness_is_direct_target_native_laundering
    (scope safe : S → T → Prop) (c : C)
    (inUniverse : scope (witnessSource source sigma c) (witnessTarget target c))
    (hUnsafe : ¬ safe (witnessSource source sigma c) (witnessTarget target c)) :
    ∃ s t,
      scope s t ∧
      TargetNativeExact source.toFun (twistedTargetDecode target.invFun sigma) s t ∧
      ¬ safe s t := by
  exact ⟨witnessSource source sigma c, witnessTarget target c,
    inUniverse, witness_is_direct_target_native source target sigma c, hUnsafe⟩

/-- The same direct witness is a concrete disproof of target-native soundness. -/
theorem unsafe_clean_witness_disproves_target_native_soundness
    (scope safe : S → T → Prop) (c : C)
    (inScope : scope (witnessSource source sigma c) (witnessTarget target c))
    (hUnsafe : ¬ safe (witnessSource source sigma c) (witnessTarget target c)) :
    ¬ Sound
      (TargetNativeExact source.toFun (twistedTargetDecode target.invFun sigma))
      safe scope := by
  intro sound
  exact hUnsafe (sound _ _ inScope
    (witness_is_direct_target_native source target sigma c))

end GlueRift.L4
