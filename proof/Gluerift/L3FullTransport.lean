import Gluerift.Core

/-! L3: both full transports survive the clean total/shared-domain twist. -/

namespace GlueRift.L3

/-- Contract Theorem 3, source-target-source direction. -/
theorem clean_total_twist_preserves_source_target_source_transport
    (source : Iso S C) (target : Iso T C) (sigma : Iso C C) (s : S) :
    transportSTS source.toFun source.invFun
      (twistedTargetEncode target.toFun sigma)
      (twistedTargetDecode target.invFun sigma) s = s := by
  change source.invFun
    (sigma (target (target.invFun (sigma.invFun (source s))))) = s
  calc
    source.invFun (sigma (target (target.invFun (sigma.invFun (source s))))) =
        source.invFun (sigma (sigma.invFun (source s))) :=
      congrArg source.invFun
        (congrArg sigma.toFun (target.rightInv (sigma.invFun (source s))))
    _ = source.invFun (source s) :=
      congrArg source.invFun (sigma.rightInv (source s))
    _ = s := source.leftInv s

/-- Contract Theorem 3, target-source-target direction. -/
theorem clean_total_twist_preserves_target_source_target_transport
    (source : Iso S C) (target : Iso T C) (sigma : Iso C C) (t : T) :
    transportTST source.toFun source.invFun
      (twistedTargetEncode target.toFun sigma)
      (twistedTargetDecode target.invFun sigma) t = t := by
  change target.invFun
    (sigma.invFun (source (source.invFun (sigma (target t))))) = t
  calc
    target.invFun (sigma.invFun (source (source.invFun (sigma (target t))))) =
        target.invFun (sigma.invFun (sigma (target t))) :=
      congrArg target.invFun
        (congrArg sigma.invFun (source.rightInv (sigma (target t))))
    _ = target.invFun (target t) :=
      congrArg target.invFun (sigma.leftInv (target t))
    _ = t := target.leftInv t

end GlueRift.L3
