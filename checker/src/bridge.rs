use crate::adapter_ir::AdapterContext;
use crate::comparator::evaluate_pair;
use crate::domain::{ComparatorSpec, ResolvedScope};
use crate::report::{BridgeKind, BridgeReport, CommonEnvelope, Status};
use crate::witness::ComparatorEvidence;

pub fn evaluate_bridge(
    envelope: CommonEnvelope,
    context: &AdapterContext,
    scope: &ResolvedScope,
    kind: BridgeKind,
) -> BridgeReport {
    let native = match kind {
        BridgeKind::CarrierTarget => ComparatorSpec::TargetNativeExact,
        BridgeKind::CarrierSource => ComparatorSpec::SourceNativeExact,
    };
    let mut counterexample = None;
    let mut carrier_evidence = None;
    let mut native_evidence = None;
    for pair in &scope.comparison_universe {
        let carrier = evaluate_pair(context, ComparatorSpec::CarrierExact, pair);
        let native_eval = evaluate_pair(context, native, pair);
        if carrier.equal != native_eval.equal {
            counterexample = Some(pair.clone());
            carrier_evidence = Some(carrier.evidence);
            native_evidence = Some(native_eval.evidence);
            break;
        }
    }
    let status = if counterexample.is_some() {
        Status::Disproved
    } else {
        Status::ProvedExhaustive
    };
    BridgeReport {
        envelope: CommonEnvelope { status, ..envelope },
        bridge_kind: kind,
        universe_pair_count: scope.comparison_universe.len(),
        checked_pair_count: counterexample
            .as_ref()
            .and_then(|pair| {
                scope
                    .comparison_universe
                    .iter()
                    .position(|candidate| candidate == pair)
            })
            .map_or(scope.comparison_universe.len(), |index| index + 1),
        counterexample_pair: crate::report::EvidenceValue::from_option(counterexample),
        carrier_comparator_evidence: carrier_evidence.unwrap_or(ComparatorEvidence::NotApplicable),
        native_comparator_evidence: native_evidence.unwrap_or(ComparatorEvidence::NotApplicable),
        sufficient_rule_coverage: Vec::new(),
    }
}

pub fn not_requested_bridge(envelope: CommonEnvelope, kind: BridgeKind) -> BridgeReport {
    BridgeReport {
        envelope: CommonEnvelope {
            status: Status::NotRequested,
            ..envelope
        },
        bridge_kind: kind,
        universe_pair_count: 0,
        checked_pair_count: 0,
        counterexample_pair: crate::report::EvidenceValue::Absent(
            crate::report::AbsentEvidence::NotApplicable,
        ),
        carrier_comparator_evidence: ComparatorEvidence::NotApplicable,
        native_comparator_evidence: ComparatorEvidence::NotApplicable,
        sufficient_rule_coverage: Vec::new(),
    }
}
