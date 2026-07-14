import Gluerift.Core

/-!
L6: comparator bridges are proved pointwise and only under the exact native or
carrier-domain facts required by contract §4.2.  No full-transport premise is
used as a substitute.
-/

namespace GlueRift.L6

/-- Carrier equality implies target-native equality at a target native-RT point. -/
theorem carrier_implies_target_native_at_target_native_roundtrip_point
    {eS : S → C} {eT : T → C} {dT : C → T} {s : S} {t : T}
    (targetNativeAtT : dT (eT t) = t)
    (carrier : CarrierExact eS eT s t) : TargetNativeExact eS dT s t := by
  unfold CarrierExact at carrier
  unfold TargetNativeExact
  rw [carrier]
  exact targetNativeAtT

/--
Target-native equality implies carrier equality only when the exact
source-encoded carrier has a proved target carrier-domain round trip.
-/
theorem target_native_implies_carrier_at_source_encoded_carrier_coverage
    {eS : S → C} {eT : T → C} {dT : C → T} {s : S} {t : T}
    (targetCarrierAtEncodedSource : eT (dT (eS s)) = eS s)
    (native : TargetNativeExact eS dT s t) : CarrierExact eS eT s t := by
  unfold TargetNativeExact at native
  unfold CarrierExact
  calc
    eS s = eT (dT (eS s)) := targetCarrierAtEncodedSource.symm
    _ = eT t := congrArg eT native

/-- Carrier equality implies source-native equality at a source native-RT point. -/
theorem carrier_implies_source_native_at_source_native_roundtrip_point
    {eS : S → C} {dS : C → S} {eT : T → C} {s : S} {t : T}
    (sourceNativeAtS : dS (eS s) = s)
    (carrier : CarrierExact eS eT s t) : SourceNativeExact eT dS s t := by
  unfold CarrierExact at carrier
  unfold SourceNativeExact
  rw [← carrier]
  exact sourceNativeAtS

/--
Source-native equality implies carrier equality only when the exact
target-encoded carrier has a proved source carrier-domain round trip.
-/
theorem source_native_implies_carrier_at_target_encoded_carrier_coverage
    {eS : S → C} {dS : C → S} {eT : T → C} {s : S} {t : T}
    (sourceCarrierAtEncodedTarget : eS (dS (eT t)) = eT t)
    (native : SourceNativeExact eT dS s t) : CarrierExact eS eT s t := by
  unfold SourceNativeExact at native
  unfold CarrierExact
  calc
    eS s = eS (dS (eT t)) := congrArg eS native.symm
    _ = eT t := sourceCarrierAtEncodedTarget

/-- §4.2 target bridge implication with domain membership made explicit. -/
theorem carrier_implies_target_native_on_declared_native_domain
    {eS : S → C} {eT : T → C} {dT : C → T}
    {targetDomain : T → Prop} {s : S} {t : T}
    (targetNativeRoundTrip : NativeRoundTrip eT dT targetDomain)
    (targetCovered : targetDomain t)
    (carrier : CarrierExact eS eT s t) : TargetNativeExact eS dT s t :=
  carrier_implies_target_native_at_target_native_roundtrip_point
    (targetNativeRoundTrip t targetCovered) carrier

/-- §4.2 reverse target bridge implication with exact carrier coverage explicit. -/
theorem target_native_implies_carrier_on_declared_target_carrier_domain
    {eS : S → C} {eT : T → C} {dT : C → T}
    {targetCarrierDomain : C → Prop} {s : S} {t : T}
    (targetCarrierRoundTrip : CarrierRoundTrip eT dT targetCarrierDomain)
    (encodedSourceCovered : targetCarrierDomain (eS s))
    (native : TargetNativeExact eS dT s t) : CarrierExact eS eT s t :=
  target_native_implies_carrier_at_source_encoded_carrier_coverage
    (targetCarrierRoundTrip (eS s) encodedSourceCovered) native

/-- §4.2 source bridge implication with domain membership made explicit. -/
theorem carrier_implies_source_native_on_declared_native_domain
    {eS : S → C} {dS : C → S} {eT : T → C}
    {sourceDomain : S → Prop} {s : S} {t : T}
    (sourceNativeRoundTrip : NativeRoundTrip eS dS sourceDomain)
    (sourceCovered : sourceDomain s)
    (carrier : CarrierExact eS eT s t) : SourceNativeExact eT dS s t :=
  carrier_implies_source_native_at_source_native_roundtrip_point
    (sourceNativeRoundTrip s sourceCovered) carrier

/-- §4.2 reverse source bridge implication with exact carrier coverage explicit. -/
theorem source_native_implies_carrier_on_declared_source_carrier_domain
    {eS : S → C} {dS : C → S} {eT : T → C}
    {sourceCarrierDomain : C → Prop} {s : S} {t : T}
    (sourceCarrierRoundTrip : CarrierRoundTrip eS dS sourceCarrierDomain)
    (encodedTargetCovered : sourceCarrierDomain (eT t))
    (native : SourceNativeExact eT dS s t) : CarrierExact eS eT s t :=
  source_native_implies_carrier_at_target_encoded_carrier_coverage
    (sourceCarrierRoundTrip (eT t) encodedTargetCovered) native

/-- Scoped bridge equivalence for CarrierExact and TargetNativeExact. -/
theorem carrier_target_bridge_on_scope
    {eS : S → C} {eT : T → C} {dT : C → T} {scope : S → T → Prop}
    (targetNativeCoverage : ∀ s t, scope s t → dT (eT t) = t)
    (targetCarrierCoverage : ∀ s t, scope s t → eT (dT (eS s)) = eS s) :
    ∀ s t, scope s t →
      (CarrierExact eS eT s t ↔ TargetNativeExact eS dT s t) := by
  intro s t hScope
  constructor
  · exact carrier_implies_target_native_at_target_native_roundtrip_point
      (targetNativeCoverage s t hScope)
  · exact target_native_implies_carrier_at_source_encoded_carrier_coverage
      (targetCarrierCoverage s t hScope)

/-- Scoped bridge equivalence for CarrierExact and SourceNativeExact. -/
theorem carrier_source_bridge_on_scope
    {eS : S → C} {dS : C → S} {eT : T → C} {scope : S → T → Prop}
    (sourceNativeCoverage : ∀ s t, scope s t → dS (eS s) = s)
    (sourceCarrierCoverage : ∀ s t, scope s t → eS (dS (eT t)) = eT t) :
    ∀ s t, scope s t →
      (CarrierExact eS eT s t ↔ SourceNativeExact eT dS s t) := by
  intro s t hScope
  constructor
  · exact carrier_implies_source_native_at_source_native_roundtrip_point
      (sourceNativeCoverage s t hScope)
  · exact source_native_implies_carrier_at_target_encoded_carrier_coverage
      (sourceCarrierCoverage s t hScope)

/-- Full target bridge over a scope, retaining both independently owned domains. -/
theorem carrier_target_bridge_on_declared_domains
    {eS : S → C} {eT : T → C} {dT : C → T}
    {scope : S → T → Prop} {targetDomain : T → Prop}
    {targetCarrierDomain : C → Prop}
    (targetNativeRoundTrip : NativeRoundTrip eT dT targetDomain)
    (targetCarrierRoundTrip : CarrierRoundTrip eT dT targetCarrierDomain)
    (targetCovered : ∀ s t, scope s t → targetDomain t)
    (encodedSourceCovered : ∀ s t, scope s t → targetCarrierDomain (eS s)) :
    ∀ s t, scope s t →
      (CarrierExact eS eT s t ↔ TargetNativeExact eS dT s t) := by
  intro s t hScope
  constructor
  · exact carrier_implies_target_native_on_declared_native_domain
      targetNativeRoundTrip (targetCovered s t hScope)
  · exact target_native_implies_carrier_on_declared_target_carrier_domain
      targetCarrierRoundTrip (encodedSourceCovered s t hScope)

/-- Full source bridge over a scope, retaining both independently owned domains. -/
theorem carrier_source_bridge_on_declared_domains
    {eS : S → C} {dS : C → S} {eT : T → C}
    {scope : S → T → Prop} {sourceDomain : S → Prop}
    {sourceCarrierDomain : C → Prop}
    (sourceNativeRoundTrip : NativeRoundTrip eS dS sourceDomain)
    (sourceCarrierRoundTrip : CarrierRoundTrip eS dS sourceCarrierDomain)
    (sourceCovered : ∀ s t, scope s t → sourceDomain s)
    (encodedTargetCovered : ∀ s t, scope s t → sourceCarrierDomain (eT t)) :
    ∀ s t, scope s t →
      (CarrierExact eS eT s t ↔ SourceNativeExact eT dS s t) := by
  intro s t hScope
  constructor
  · exact carrier_implies_source_native_on_declared_native_domain
      sourceNativeRoundTrip (sourceCovered s t hScope)
  · exact source_native_implies_carrier_on_declared_source_carrier_domain
      sourceCarrierRoundTrip (encodedTargetCovered s t hScope)

end GlueRift.L6
