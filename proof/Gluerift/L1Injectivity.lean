import Gluerift.Core

/-! L1: admitted-domain native round trips imply admitted-domain injectivity. -/

namespace GlueRift.L1

/-- Contract Lemma 1, scoped to the explicitly admitted native domain. -/
theorem native_roundtrip_implies_encoder_injective_on_admitted_domain
    {encode : X → C} {decode : C → X} {domain : X → Prop}
    (roundtrip : NativeRoundTrip encode decode domain)
    {x₁ x₂ : X} (hx₁ : domain x₁) (hx₂ : domain x₂)
    (encoded : encode x₁ = encode x₂) : x₁ = x₂ := by
  calc
    x₁ = decode (encode x₁) := (roundtrip x₁ hx₁).symm
    _ = decode (encode x₂) := congrArg decode encoded
    _ = x₂ := roundtrip x₂ hx₂

end GlueRift.L1
