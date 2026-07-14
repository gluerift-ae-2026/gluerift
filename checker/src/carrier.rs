use std::collections::BTreeSet;

use crate::adapter_ir::AdapterContext;
use crate::domain::{ComparatorSpec, ResolvedScope};
use crate::report::{CarrierClassPair, CarrierSummary, CommonEnvelope};

pub fn derive_carrier_summary(
    envelope: CommonEnvelope,
    context: &AdapterContext,
    scope: &ResolvedScope,
    selected_bridge_status: crate::report::Status,
    selected_bridge_hash: String,
) -> CarrierSummary {
    let source_domain: BTreeSet<_> = scope
        .comparison_universe
        .iter()
        .map(|pair| pair.source.clone())
        .collect();
    let target_domain: BTreeSet<_> = scope
        .comparison_universe
        .iter()
        .map(|pair| pair.target.clone())
        .collect();
    let source_map: Vec<_> = source_domain
        .iter()
        .filter_map(|source| {
            context
                .source_encode
                .eval(source)
                .ok()
                .map(|carrier| (carrier, source.clone()))
        })
        .collect();
    let target_map: Vec<_> = target_domain
        .iter()
        .filter_map(|target| {
            context
                .target_encode
                .eval(target)
                .ok()
                .map(|carrier| (carrier, target.clone()))
        })
        .collect();
    let source_successful_image: BTreeSet<_> = source_map
        .iter()
        .map(|(carrier, _)| carrier.clone())
        .collect();
    let target_successful_image: BTreeSet<_> = target_map
        .iter()
        .map(|(carrier, _)| carrier.clone())
        .collect();
    let shared: BTreeSet<_> = source_successful_image
        .intersection(&target_successful_image)
        .cloned()
        .collect();
    let mut pairs = Vec::new();
    for carrier in &shared {
        for (_, source) in source_map
            .iter()
            .filter(|(candidate, _)| candidate == carrier)
        {
            for (_, target) in target_map
                .iter()
                .filter(|(candidate, _)| candidate == carrier)
            {
                pairs.push(CarrierClassPair {
                    carrier: carrier.clone(),
                    source: source.clone(),
                    target: target.clone(),
                });
            }
        }
    }
    pairs.sort_by(|a, b| {
        (&a.carrier, &a.source, &a.target).cmp(&(&b.carrier, &b.source, &b.target))
    });
    let (evidence_basis, applicability, bridge_hash) = match scope.comparator {
        ComparatorSpec::CarrierExact => ("carrier-exact", "direct", "not-required".into()),
        _ if selected_bridge_status == crate::report::Status::ProvedExhaustive => (
            "selected-via-proved-bridge",
            "applicable",
            selected_bridge_hash,
        ),
        _ => (
            "explanatory-only",
            "not-applicable-to-selected-comparator",
            selected_bridge_hash,
        ),
    };
    CarrierSummary {
        envelope,
        source_successful_image: source_successful_image.into_iter().collect(),
        target_successful_image: target_successful_image.into_iter().collect(),
        shared_carrier_classes: shared.into_iter().collect(),
        class_endpoint_pairs: pairs,
        class_observation_conflicts: Vec::new(),
        evidence_basis: evidence_basis.into(),
        applicability_to_selected_comparator: applicability.into(),
        bridge_report_sha256: bridge_hash,
    }
}
