use std::collections::BTreeMap;

use crate::adapter_ir::{Adapter, AdapterContext};
use crate::canonical::{CanonicalError, canonical_sha256};
use crate::domain::ResolvedScope;
use crate::report::{
    ExecutionTraceRow, LawId, NOT_APPLICABLE, RoundTripLawReport, RoundTripReport, StageTrace,
    Status,
};
use crate::type_ir::Value;

#[derive(Clone, Debug)]
pub struct RoundTripEvaluation {
    pub laws: BTreeMap<LawId, RoundTripLawReport>,
    pub tables: BTreeMap<LawId, Vec<ExecutionTraceRow>>,
}

type LawSpec<'a> = (LawId, &'a [Value], Vec<(&'static str, &'a Adapter)>);

pub fn evaluate_roundtrips(
    context: &AdapterContext,
    scope: &ResolvedScope,
) -> Result<RoundTripEvaluation, CanonicalError> {
    let specs: [LawSpec<'_>; 6] = [
        (
            LawId::SourceNative,
            &scope.source_domain,
            vec![
                ("source_encode", &context.source_encode),
                ("source_decode", &context.source_decode),
            ],
        ),
        (
            LawId::TargetNative,
            &scope.target_domain,
            vec![
                ("target_encode", &context.target_encode),
                ("target_decode", &context.target_decode),
            ],
        ),
        (
            LawId::SourceCarrier,
            &scope.source_carrier_domain,
            vec![
                ("source_decode", &context.source_decode),
                ("source_encode", &context.source_encode),
            ],
        ),
        (
            LawId::TargetCarrier,
            &scope.target_carrier_domain,
            vec![
                ("target_decode", &context.target_decode),
                ("target_encode", &context.target_encode),
            ],
        ),
        (
            LawId::SourceFullTransport,
            &scope.source_full_transport_domain,
            vec![
                ("source_encode", &context.source_encode),
                ("target_decode", &context.target_decode),
                ("target_encode", &context.target_encode),
                ("source_decode", &context.source_decode),
            ],
        ),
        (
            LawId::TargetFullTransport,
            &scope.target_full_transport_domain,
            vec![
                ("target_encode", &context.target_encode),
                ("source_decode", &context.source_decode),
                ("source_encode", &context.source_encode),
                ("target_decode", &context.target_decode),
            ],
        ),
    ];
    let mut laws = BTreeMap::new();
    let mut tables = BTreeMap::new();
    for (law_id, domain, stages) in specs {
        let mut rows = Vec::new();
        let mut failure: Option<(Value, Vec<StageTrace>)> = None;
        let mut all_stages_succeeded = true;
        let mut all_final_equal = true;
        for input in domain {
            let mut current = Ok(input.clone());
            let mut traces = Vec::new();
            for (name, adapter) in &stages {
                current = current.and_then(|value| adapter.eval(&value));
                traces.push(StageTrace {
                    stage: (*name).into(),
                    result: current.clone(),
                });
                if current.is_err() {
                    break;
                }
            }
            let final_matches_input = current.as_ref().is_ok_and(|value| value == input);
            all_stages_succeeded &= current.is_ok();
            all_final_equal &= final_matches_input;
            if !final_matches_input && failure.is_none() {
                failure = Some((input.clone(), traces.clone()));
            }
            rows.push(ExecutionTraceRow {
                input: input.clone(),
                stages: traces,
                final_matches_input,
            });
        }
        let coverage_status = if all_stages_succeeded {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        };
        let equality_status = if all_final_equal {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        };
        let status = if coverage_status == Status::ProvedExhaustive
            && equality_status == Status::ProvedExhaustive
        {
            Status::ProvedExhaustive
        } else {
            Status::Disproved
        };
        let trace_hash = canonical_sha256(&rows)?;
        let is_full = matches!(
            law_id,
            LawId::SourceFullTransport | LawId::TargetFullTransport
        );
        let (first_failing_input, first_failure_trace) =
            failure.unwrap_or((Value::Unit, Vec::new()));
        let report = RoundTripLawReport {
            law_id,
            domain_sha256: canonical_sha256(domain)?,
            declared_input_count: domain.len(),
            checked_input_count: rows.len(),
            status,
            transport_coverage_status: if is_full {
                crate::report::EvidenceValue::Present(coverage_status)
            } else {
                crate::report::EvidenceValue::Absent(crate::report::AbsentEvidence::NotApplicable)
            },
            final_equality_status: equality_status,
            execution_trace_table_sha256: trace_hash,
            first_failing_input: crate::report::EvidenceValue::from_option(
                (status == Status::Disproved).then_some(first_failing_input),
            ),
            first_failure_trace,
            // The semantic evaluator returns the failing input/trace seed.
            // `comparison::check` installs its canonical witness before this
            // report is ever materialized or serialized.
            witness_sha256: NOT_APPLICABLE.into(),
        };
        laws.insert(law_id, report);
        tables.insert(law_id, rows);
    }
    Ok(RoundTripEvaluation { laws, tables })
}

pub fn materialize_roundtrip_report(
    envelope: crate::report::CommonEnvelope,
    evaluation: &RoundTripEvaluation,
) -> RoundTripReport {
    RoundTripReport {
        envelope,
        laws: evaluation.laws.values().cloned().collect(),
    }
}
