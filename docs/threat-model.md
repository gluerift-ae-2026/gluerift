# Threat model

GlueRift studies a cross-language validation seam in which source and target
program values are transported through adapter glue and compared in a selected
native or carrier representation. The adversary may supply or influence adapter
maps while preserving the admission workflow's requested round-trip laws. The
attack succeeds when the validator's selected induced equality contains a pair
that the independently owned endpoint policy marks unsafe.

The endpoint types, finite validation domains, comparator selection, comparison
universe, Safe and Match observations, validation request, and transformation
family descriptor are trusted policy inputs. The candidate adapter cannot narrow
them or define them from its own outputs. GlueRift checks only the finite,
total-success Core and reports conclusions relative to those inputs.

The implementation does not claim that endpoint policy is correct, that programs
are equivalent, that arbitrary handwritten adapters are verified, or that the
declared transformation family is semantically complete.

