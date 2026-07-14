import Gluerift.Core

/-! L2: carrier conjugation preserves target native and total carrier round trips. -/

namespace GlueRift.L2

/-- Contract Theorem 2, target native half. -/
theorem target_automorphism_inverse_twist_preserves_native_roundtrip
    (target : Iso T C) (sigma : Iso C C) (t : T) :
    twistedTargetDecode target.invFun sigma
      (twistedTargetEncode target.toFun sigma t) = t := by
  change target.invFun (sigma.invFun (sigma (target t))) = t
  calc
    target.invFun (sigma.invFun (sigma (target t))) = target.invFun (target t) :=
      congrArg target.invFun (sigma.leftInv (target t))
    _ = t := target.leftInv t

/-- Contract Theorem 2, total target-carrier half. -/
theorem target_automorphism_inverse_twist_preserves_total_carrier_roundtrip
    (target : Iso T C) (sigma : Iso C C) (c : C) :
    twistedTargetEncode target.toFun sigma
      (twistedTargetDecode target.invFun sigma c) = c := by
  change sigma (target (target.invFun (sigma.invFun c))) = c
  calc
    sigma (target (target.invFun (sigma.invFun c))) = sigma (sigma.invFun c) :=
      congrArg sigma.toFun (target.rightInv (sigma.invFun c))
    _ = c := sigma.rightInv c

end GlueRift.L2
