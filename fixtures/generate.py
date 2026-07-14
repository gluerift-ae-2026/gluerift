#!/usr/bin/env python3
"""Generate the canonical GlueRift v0.3.1a Core fixture inputs.

The expected-oracle registry is emitted beside the authored semantic inputs,
but no expected field is copied into an evaluator input.  Transformed adapter
contexts are deliberately referenced under artifact/staging and are produced
by the Rust transformation engine during reproduction.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Iterable


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "fixtures"
RUN_CONFIG_PATH = ROOT / "spec/run-config/core-v0.3.1a.json"
FAMILY_PATH = ROOT / "spec/transformation-families/core-structural-v0.3.1a.json"

ALL_LAWS = [
    "source-native",
    "target-native",
    "source-carrier",
    "target-carrier",
    "source-full-transport",
    "target-full-transport",
]
ALL_LAW_FLAGS = {
    "source_native_roundtrip": True,
    "target_native_roundtrip": True,
    "source_carrier_roundtrip": True,
    "target_carrier_roundtrip": True,
    "source_full_transport": True,
    "target_full_transport": True,
}
LAW_FLAGS = [
    ("source-native", "source_native_roundtrip"),
    ("target-native", "target_native_roundtrip"),
    ("source-carrier", "source_carrier_roundtrip"),
    ("target-carrier", "target_carrier_roundtrip"),
    ("source-full-transport", "source_full_transport"),
    ("target-full-transport", "target_full_transport"),
]
ALL_PROPERTIES = [
    {"kind": "policy-soundness"},
    {"kind": "comparison-adequacy"},
    {"kind": "comparison-precision"},
    {"kind": "faithful-comparison"},
]
PROPERTY_KEYS = [
    "policy-soundness",
    "comparison-adequacy",
    "comparison-precision",
    "faithful-comparison",
    "target-non-amplification",
]


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def read(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write(relative: str, value: Any) -> None:
    path = ROOT / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_bytes(value))


def logical(path: Path | str) -> str:
    return Path(path).as_posix()


def t_unit() -> dict[str, Any]:
    return {"kind": "unit"}


def t_bool() -> dict[str, Any]:
    return {"kind": "bool"}


def t_int(minimum: int = 0, maximum: int = 2) -> dict[str, Any]:
    return {"kind": "bounded-int", "min": minimum, "max": maximum}


def t_sum(names: Iterable[str]) -> dict[str, Any]:
    return {
        "kind": "sum",
        "variants": [{"name": name, "payload": t_unit()} for name in names],
    }


def t_product(fields: Iterable[tuple[str, dict[str, Any]]]) -> dict[str, Any]:
    return {
        "kind": "product",
        "fields": [{"name": name, "type": ty} for name, ty in fields],
    }


def t_result() -> dict[str, Any]:
    return {"kind": "object-result", "ok": t_unit(), "err": t_unit()}


def v_unit() -> dict[str, Any]:
    return {"kind": "unit"}


def v_bool(value: bool) -> dict[str, Any]:
    return {"kind": "bool", "value": value}


def v_int(value: int) -> dict[str, Any]:
    return {"kind": "bounded-int", "value": value}


def v_sum(variant: str) -> dict[str, Any]:
    return {"kind": "sum", "variant": variant, "payload": v_unit()}


def v_product(fields: dict[str, dict[str, Any]]) -> dict[str, Any]:
    return {"kind": "product", "fields": fields}


def v_result(branch: str) -> dict[str, Any]:
    return {"kind": "object-result", "branch": branch, "value": v_unit()}


def identity() -> dict[str, Any]:
    return {"kind": "identity"}


def compose(first: dict[str, Any], second: dict[str, Any]) -> dict[str, Any]:
    return {"kind": "compose", "first": first, "second": second}


def enum_map(mapping: dict[str, str]) -> dict[str, Any]:
    return {"kind": "enum-permutation", "mapping": mapping}


def sum_map_adapter(mapping: dict[str, str]) -> dict[str, Any]:
    return {
        "kind": "sum-map",
        "variants": {
            source: {"target": target, "adapter": identity()}
            for source, target in mapping.items()
        },
    }


def field_map(mapping: dict[str, str]) -> dict[str, Any]:
    return {"kind": "field-permutation", "mapping": mapping}


def product_map(fields: dict[str, tuple[str, dict[str, Any]]]) -> dict[str, Any]:
    return {
        "kind": "product-map",
        "fields": {
            target: {"source": source, "adapter": adapter}
            for target, (source, adapter) in fields.items()
        },
    }


def result_swap() -> dict[str, Any]:
    return {
        "kind": "result-map",
        "branch_mapping": "swap",
        "ok": identity(),
        "err": identity(),
    }


def obs_atom(value: str) -> dict[str, Any]:
    return {"kind": "atom", "value": value}


def observer_constructor(table: dict[str, str]) -> dict[str, Any]:
    return {"kind": "constructor-role", "path": [], "table": table}


def observer_finite(
    entries: list[tuple[dict[str, Any], str]],
    path: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "kind": "finite-policy-map",
        "path": path or [],
        "table": [{"value": value, "atom": atom} for value, atom in entries],
    }


def relation_exact() -> dict[str, Any]:
    return {"kind": "exact"}


def relation_table(
    left_atoms: Iterable[str],
    right_atoms: Iterable[str],
    allowed: Iterable[tuple[str, str]],
) -> dict[str, Any]:
    return {
        "kind": "finite-table",
        "left_codomain": [obs_atom(value) for value in sorted(left_atoms)],
        "right_codomain": [obs_atom(value) for value in sorted(right_atoms)],
        "allowed_pairs": [
            {"left": obs_atom(left), "right": obs_atom(right)}
            for left, right in sorted(allowed)
        ],
    }


def relation_tna() -> dict[str, Any]:
    return {
        "kind": "target-no-amplification",
        "elements": [obs_atom("allow"), obs_atom("deny")],
        "preorder_edges": [
            {"left": obs_atom("allow"), "right": obs_atom("allow")},
            {"left": obs_atom("deny"), "right": obs_atom("allow")},
            {"left": obs_atom("deny"), "right": obs_atom("deny")},
        ],
    }


def policy(
    *,
    dimension_id: str,
    source_observer: dict[str, Any],
    target_observer: dict[str, Any],
    source_atoms: Iterable[str],
    target_atoms: Iterable[str],
    safe_relation: dict[str, Any],
    match_relation: dict[str, Any] | None,
    match_coverage: str,
    safe_active: bool = True,
    match_active: bool = True,
) -> dict[str, Any]:
    dimension: dict[str, Any] = {
        "id": dimension_id,
        "source_codomain": [obs_atom(value) for value in sorted(set(source_atoms))],
        "target_codomain": [obs_atom(value) for value in sorted(set(target_atoms))],
        "source_observer": source_observer,
        "target_observer": target_observer,
        "safe_relation": safe_relation,
    }
    if match_relation is not None:
        dimension["match_relation"] = match_relation
    return {
        "schema": "gluerift.policy/v0.3.1a",
        "match_coverage": match_coverage,
        "dimensions": [dimension],
        "safe_dimensions": [dimension_id] if safe_active else [],
        "match_dimensions": [dimension_id] if match_active else [],
        "explicitly_irrelevant_paths": [],
    }


def policy_unconstrained() -> dict[str, Any]:
    return {
        "schema": "gluerift.policy/v0.3.1a",
        "match_coverage": "none",
        "dimensions": [],
        "safe_dimensions": [],
        "match_dimensions": [],
        "explicitly_irrelevant_paths": [],
    }


def context(
    source_type: dict[str, Any],
    target_type: dict[str, Any],
    carrier_type: dict[str, Any],
    source_encode: dict[str, Any],
    source_decode: dict[str, Any],
    target_encode: dict[str, Any],
    target_decode: dict[str, Any],
) -> dict[str, Any]:
    return {
        "schema": "gluerift.adapter-context/v0.3.1a",
        "source_type": source_type,
        "target_type": target_type,
        "carrier_type": carrier_type,
        "source_encode": source_encode,
        "source_decode": source_decode,
        "target_encode": target_encode,
        "target_decode": target_decode,
    }


def domain_all() -> dict[str, Any]:
    return {"kind": "all"}


def domain_values(values: list[dict[str, Any]]) -> dict[str, Any]:
    return {"kind": "finite-set", "values": values}


def pairs(values: list[tuple[dict[str, Any], dict[str, Any]]]) -> dict[str, Any]:
    return {
        "kind": "finite-pair-set",
        "pairs": [{"source": source, "target": target} for source, target in values],
    }


def scope(
    comparator: str,
    *,
    source_domain: dict[str, Any] | None = None,
    target_domain: dict[str, Any] | None = None,
    source_comparison_domain: dict[str, Any] | None = None,
    target_comparison_domain: dict[str, Any] | None = None,
    universe: dict[str, Any] | None = None,
    source_carrier_domain: dict[str, Any] | None = None,
    target_carrier_domain: dict[str, Any] | None = None,
    source_full_transport_domain: dict[str, Any] | None = None,
    target_full_transport_domain: dict[str, Any] | None = None,
) -> dict[str, Any]:
    all_domain = domain_all()
    source_cmp = source_comparison_domain or all_domain
    target_cmp = target_comparison_domain or all_domain
    return {
        "schema": "gluerift.validation-scope/v0.3.1a",
        "source_domain": source_domain or all_domain,
        "target_domain": target_domain or all_domain,
        "source_comparison_domain": source_cmp,
        "target_comparison_domain": target_cmp,
        "comparison_universe": universe
        or {"kind": "product", "source": source_cmp, "target": target_cmp},
        "source_carrier_domain": source_carrier_domain or all_domain,
        "target_carrier_domain": target_carrier_domain or all_domain,
        "source_full_transport_domain": source_full_transport_domain or all_domain,
        "target_full_transport_domain": target_full_transport_domain or all_domain,
        "comparator": comparator,
    }


def validation_request(
    request_id: str,
    profile: str,
    fixture_scope: dict[str, Any],
    fixture_policy: dict[str, Any],
    properties: list[dict[str, Any]],
    family_hash: str,
    run_config_hash: str,
    *,
    laws: dict[str, bool] | None = None,
    bridges: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "schema": "gluerift.validation-request/v0.3.1a",
        "request_id": request_id,
        "profile": profile,
        "validation_scope_sha256": digest(fixture_scope),
        "endpoint_policy_sha256": digest(fixture_policy),
        "run_configuration_sha256": run_config_hash,
        "required_laws": laws if laws is not None else dict(ALL_LAW_FLAGS),
        "required_properties": properties,
        "required_bridges": bridges or [],
        "required_transformation_family_sha256": family_hash,
    }


def enum_context(
    source_names: list[str],
    target_names: list[str],
    carrier_names: list[str],
    source_to_carrier: dict[str, str],
    target_to_carrier: dict[str, str],
    source_decode: dict[str, str] | None = None,
    target_decode: dict[str, str] | None = None,
) -> dict[str, Any]:
    source_decode = source_decode or {value: key for key, value in source_to_carrier.items()}
    target_decode = target_decode or {value: key for key, value in target_to_carrier.items()}
    return context(
        t_sum(source_names),
        t_sum(target_names),
        t_sum(carrier_names),
        sum_map_adapter(source_to_carrier),
        sum_map_adapter(source_decode),
        sum_map_adapter(target_to_carrier),
        sum_map_adapter(target_decode),
    )


def nested_type(field_order: tuple[str, str]) -> dict[str, Any]:
    bounds = t_product([(name, t_int()) for name in field_order])
    return t_product([("output", t_product([("policy", t_product([("bounds", bounds)]))]))])


def nested_value(minimum: int, maximum: int, *, carrier: bool = False) -> dict[str, Any]:
    if carrier:
        bounds = {
            "maximum_slot": v_int(maximum),
            "minimum_slot": v_int(minimum),
        }
    else:
        bounds = {"maximum": v_int(maximum), "minimum": v_int(minimum)}
    return v_product(
        {
            "output": v_product(
                {"policy": v_product({"bounds": v_product(bounds)})}
            )
        }
    )


def nested_adapter(to_carrier: bool) -> dict[str, Any]:
    leaf = (
        field_map({"maximum_slot": "maximum", "minimum_slot": "minimum"})
        if to_carrier
        else field_map({"maximum": "maximum_slot", "minimum": "minimum_slot"})
    )
    return product_map(
        {
            "output": (
                "output",
                product_map(
                    {
                        "policy": (
                            "policy",
                            product_map({"bounds": ("bounds", leaf)}),
                        )
                    }
                ),
            )
        }
    )


def nested_slot_swap() -> dict[str, Any]:
    leaf = field_map(
        {"maximum_slot": "minimum_slot", "minimum_slot": "maximum_slot"}
    )
    return product_map(
        {
            "output": (
                "output",
                product_map(
                    {
                        "policy": (
                            "policy",
                            product_map({"bounds": ("bounds", leaf)}),
                        )
                    }
                ),
            )
        }
    )


def nested_policy(match_coverage: str = "nonempty") -> dict[str, Any]:
    entries = [(v_int(value), str(value)) for value in range(3)]
    codomain = [obs_atom(str(value)) for value in range(3)]
    dimensions = []
    for dimension_id, role in [
        ("00-minimum-role", "minimum"),
        ("01-maximum-role", "maximum"),
    ]:
        observer = observer_finite(
            entries,
            ["output", "policy", "bounds", role],
        )
        dimensions.append(
            {
                "id": dimension_id,
                "source_codomain": codomain,
                "target_codomain": codomain,
                "source_observer": observer,
                "target_observer": observer,
                "safe_relation": relation_exact(),
                "match_relation": relation_exact(),
            }
        )
    active = [dimension["id"] for dimension in dimensions]
    return {
        "schema": "gluerift.policy/v0.3.1a",
        "match_coverage": match_coverage,
        "dimensions": dimensions,
        "safe_dimensions": active,
        "match_dimensions": active,
        "explicitly_irrelevant_paths": [],
    }


def candidate(
    transformation: dict[str, Any],
    inverse: dict[str, Any],
    *,
    mode: str,
    rule: str,
    parent_path: list[str],
    ordinal: int = 0,
) -> dict[str, Any]:
    return {
        "generation_mode": mode,
        "generation_rule_id": rule,
        "generation_parent_path": parent_path,
        "generation_ordinal": ordinal,
        "transformation_ir": transformation,
        "inverse_ir": inverse,
    }


def property_statuses(
    sound: str,
    adequate: str,
    precise: str,
    faithful: str,
    tna: str = "not-requested",
) -> dict[str, str]:
    return dict(
        zip(PROPERTY_KEYS, [sound, adequate, precise, faithful, tna], strict=True)
    )


def law_statuses(target_carrier: str = "proved-exhaustive") -> dict[str, str]:
    result = {law: "proved-exhaustive" for law in ALL_LAWS}
    result["target-carrier"] = target_carrier
    return result


def make_request_files(
    directory: str,
    request_id: str,
    fixture_context: dict[str, Any],
    fixture_scope: dict[str, Any],
    fixture_policy: dict[str, Any],
    profile: str,
    properties: list[dict[str, Any]],
    family_hash: str,
    run_config_hash: str,
    *,
    bridges: list[str] | None = None,
    laws: dict[str, bool] | None = None,
) -> tuple[str, str, str, str, dict[str, Any]]:
    context_path = f"{directory}/context.json"
    scope_path = f"{directory}/scope.json"
    policy_path = f"{directory}/policy.json"
    request_path = f"{directory}/request.json"
    request = validation_request(
        request_id,
        profile,
        fixture_scope,
        fixture_policy,
        properties,
        family_hash,
        run_config_hash,
        laws=laws,
        bridges=bridges,
    )
    write(context_path, fixture_context)
    write(scope_path, fixture_scope)
    write(policy_path, fixture_policy)
    write(request_path, request)
    return context_path, scope_path, policy_path, request_path, request


def registry_row(
    *,
    run_id: str,
    fixture_kind: str,
    context_path: str,
    base_context_path: str,
    scope_path: str,
    policy_path: str,
    request_path: str,
    request: dict[str, Any],
    fixture_scope: dict[str, Any],
    family_hash: str,
    run_config_hash: str,
    profile_consistency: str,
    coverage_mode: str,
    coverage_status: str,
    safe_match_status: str,
    eligible: bool,
    granted: bool,
    definedness: str,
    laws: dict[str, str],
    properties: dict[str, str],
    bridges: dict[str, str],
    policy_status: str,
    transformation_required: bool = False,
    transformation: dict[str, Any] | None = None,
    transformation_classification: str = "not-applicable",
    candidate_binding: str = "not-applicable",
    base_alignment: str = "not-applicable",
    witness_kinds: list[str] | None = None,
    bl2: bool = False,
    bl4: bool = False,
    native_replay_id: str = "not-applicable",
) -> dict[str, Any]:
    required_laws = [
        law for law, field in LAW_FLAGS if request["required_laws"][field]
    ]
    transformation_hash = (
        digest(transformation["transformation_ir"])
        if transformation is not None
        else "not-applicable"
    )
    return {
        "run_id": run_id,
        "fixture_kind": fixture_kind,
        "context_logical_path": context_path,
        "transformation_base_context_logical_path": base_context_path,
        "scope_logical_path": scope_path,
        "policy_logical_path": policy_path,
        "request_logical_path": request_path,
        "request_id": request["request_id"],
        "validation_request_sha256": digest(request),
        "profile": request["profile"],
        "required_law_ids": required_laws,
        "required_properties": request["required_properties"],
        "required_properties_sha256": digest(request["required_properties"]),
        "required_bridge_ids": request["required_bridges"],
        "required_transformation_family_sha256": family_hash,
        "comparator_spec_sha256": digest(fixture_scope["comparator"]),
        "run_configuration_sha256": run_config_hash,
        "expected_profile_property_consistency": profile_consistency,
        "match_coverage_mode": coverage_mode,
        "expected_match_coverage_status": coverage_status,
        "expected_safe_match_equality_status": safe_match_status,
        "expected_certificate_eligibility": eligible,
        "expected_certificate_granted": granted,
        "expected_comparator_definedness_status": definedness,
        "expected_law_statuses": laws,
        "expected_property_statuses": properties,
        "expected_bridge_statuses": bridges,
        "expected_policy_contract_status": policy_status,
        "transformation_report_required": transformation_required,
        "transformation_sha256": transformation_hash,
        "expected_transformation_classification": transformation_classification,
        "expected_candidate_binding_status": candidate_binding,
        "expected_base_alignment_status": base_alignment,
        "required_witness_kinds": sorted(witness_kinds or []),
        "bl2_paired": bl2,
        "bl4_paired": bl4,
        "native_replay_id": native_replay_id,
    }


def generate() -> None:
    run_config = read(RUN_CONFIG_PATH)
    family = read(FAMILY_PATH)
    run_config_hash = digest(run_config)
    family_hash = digest(family)
    rows: list[dict[str, Any]] = []

    # A01 and H01: endpoint constructor names differ, with a carrier enum swap.
    a01_context = enum_context(
        ["ALLOW", "DENY"],
        ["Blocked", "Permitted"],
        ["DECISION_CARRIER_ALLOW", "DECISION_CARRIER_DENY"],
        {"ALLOW": "DECISION_CARRIER_ALLOW", "DENY": "DECISION_CARRIER_DENY"},
        {"Blocked": "DECISION_CARRIER_DENY", "Permitted": "DECISION_CARRIER_ALLOW"},
    )
    a01_policy = policy(
        dimension_id="decision-role",
        source_observer=observer_constructor({"ALLOW": "allow", "DENY": "deny"}),
        target_observer=observer_constructor({"Blocked": "deny", "Permitted": "allow"}),
        source_atoms=["allow", "deny"],
        target_atoms=["allow", "deny"],
        safe_relation=relation_exact(),
        match_relation=relation_exact(),
        match_coverage="nonempty",
    )
    a01_scope = scope("target-native-exact")
    a01_dir = "fixtures/attacks/A01"
    a01_paths = make_request_files(
        a01_dir,
        "A01",
        a01_context,
        a01_scope,
        a01_policy,
        "policy-sound-adequate",
        ALL_PROPERTIES,
        family_hash,
        run_config_hash,
    )
    a01_swap = enum_map(
        {
            "DECISION_CARRIER_ALLOW": "DECISION_CARRIER_DENY",
            "DECISION_CARRIER_DENY": "DECISION_CARRIER_ALLOW",
        }
    )
    a01_candidate = candidate(
        a01_swap,
        a01_swap,
        mode="enumerated",
        rule="core.enum.payload-compatible",
        parent_path=[],
    )
    write(f"{a01_dir}/transformation.json", a01_candidate)
    ctxp, scp, polp, reqp, req = a01_paths
    common_attack_base = dict(
        fixture_scope=a01_scope,
        family_hash=family_hash,
        run_config_hash=run_config_hash,
        profile_consistency="proved-exhaustive",
        coverage_mode="nonempty",
        coverage_status="proved-exhaustive",
        safe_match_status="not-requested",
        eligible=True,
        definedness="proved-exhaustive",
        laws=law_statuses(),
        bridges={"carrier-source": "not-requested", "carrier-target": "proved-exhaustive"},
        policy_status="constrained",
    )
    rows.append(
        registry_row(
            run_id="A01.base",
            fixture_kind="attack-base",
            context_path=ctxp,
            base_context_path="not-applicable",
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            granted=True,
            properties=property_statuses(*(["proved-exhaustive"] * 4)),
            **common_attack_base,
        )
    )
    rows.append(
        registry_row(
            run_id="A01",
            fixture_kind="attack",
            context_path="artifact/staging/generated-contexts/A01.json",
            base_context_path=ctxp,
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            granted=False,
            properties=property_statuses(*(["disproved"] * 4)),
            transformation_required=True,
            transformation=a01_candidate,
            transformation_classification="lawful-harmful",
            candidate_binding="proved-exhaustive",
            base_alignment="proved-exhaustive",
            witness_kinds=["missing-required-match", "unsafe-false-agreement"],
            bl2=True,
            bl4=True,
            native_replay_id="E01",
            **common_attack_base,
        )
    )

    h01_dir = "fixtures/benign/H01"
    h01_policy = dict(a01_policy)
    h01_policy["match_coverage"] = "bidirectional-total"
    h01_paths = make_request_files(
        h01_dir,
        "H01",
        a01_context,
        a01_scope,
        h01_policy,
        "faithful-exact",
        ALL_PROPERTIES,
        family_hash,
        run_config_hash,
    )
    ctxp, scp, polp, reqp, req = h01_paths
    rows.append(
        registry_row(
            run_id="H01",
            fixture_kind="benign",
            context_path=ctxp,
            base_context_path="not-applicable",
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            fixture_scope=a01_scope,
            family_hash=family_hash,
            run_config_hash=run_config_hash,
            profile_consistency="proved-exhaustive",
            coverage_mode="bidirectional-total",
            coverage_status="proved-exhaustive",
            safe_match_status="proved-exhaustive",
            eligible=True,
            granted=True,
            definedness="proved-exhaustive",
            laws=law_statuses(),
            properties=property_statuses(*(["proved-exhaustive"] * 4)),
            bridges={"carrier-source": "not-requested", "carrier-target": "proved-exhaustive"},
            policy_status="constrained",
            bl4=True,
        )
    )

    # A02 and H02: nested repeated-type fields at the exact native witness path.
    endpoint_nested = nested_type(("maximum", "minimum"))
    carrier_nested = nested_type(("maximum_slot", "minimum_slot"))
    a02_context = context(
        endpoint_nested,
        endpoint_nested,
        carrier_nested,
        nested_adapter(True),
        nested_adapter(False),
        nested_adapter(True),
        nested_adapter(False),
    )
    a02_policy = nested_policy()
    a02_scope = scope("target-native-exact")
    a02_dir = "fixtures/attacks/A02"
    a02_paths = make_request_files(
        a02_dir,
        "A02",
        a02_context,
        a02_scope,
        a02_policy,
        "policy-sound-adequate",
        ALL_PROPERTIES,
        family_hash,
        run_config_hash,
    )
    a02_swap = nested_slot_swap()
    a02_candidate = candidate(
        a02_swap,
        a02_swap,
        mode="enumerated",
        rule="core.nested.canonical-path-product",
        parent_path=["output", "policy", "bounds"],
    )
    write(f"{a02_dir}/transformation.json", a02_candidate)
    ctxp, scp, polp, reqp, req = a02_paths
    a02_common = dict(common_attack_base)
    a02_common["fixture_scope"] = a02_scope
    rows.append(
        registry_row(
            run_id="A02.base",
            fixture_kind="attack-base",
            context_path=ctxp,
            base_context_path="not-applicable",
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            granted=True,
            properties=property_statuses(*(["proved-exhaustive"] * 4)),
            **a02_common,
        )
    )
    rows.append(
        registry_row(
            run_id="A02",
            fixture_kind="attack",
            context_path="artifact/staging/generated-contexts/A02.json",
            base_context_path=ctxp,
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            granted=False,
            properties=property_statuses(*(["disproved"] * 4)),
            transformation_required=True,
            transformation=a02_candidate,
            transformation_classification="lawful-harmful",
            candidate_binding="proved-exhaustive",
            base_alignment="proved-exhaustive",
            witness_kinds=["missing-required-match", "unsafe-false-agreement"],
            bl2=True,
            bl4=True,
            native_replay_id="E02",
            **a02_common,
        )
    )

    h02_dir = "fixtures/benign/H02"
    h02_source = nested_type(("maximum", "minimum"))
    h02_target = nested_type(("minimum", "maximum"))
    h02_context = context(
        h02_source,
        h02_target,
        carrier_nested,
        nested_adapter(True),
        nested_adapter(False),
        nested_adapter(True),
        nested_adapter(False),
    )
    h02_policy = nested_policy("bidirectional-total")
    h02_scope = scope("target-native-exact")
    h02_paths = make_request_files(
        h02_dir,
        "H02",
        h02_context,
        h02_scope,
        h02_policy,
        "faithful-exact",
        ALL_PROPERTIES,
        family_hash,
        run_config_hash,
    )
    ctxp, scp, polp, reqp, req = h02_paths
    rows.append(
        registry_row(
            run_id="H02",
            fixture_kind="benign",
            context_path=ctxp,
            base_context_path="not-applicable",
            scope_path=scp,
            policy_path=polp,
            request_path=reqp,
            request=req,
            fixture_scope=h02_scope,
            family_hash=family_hash,
            run_config_hash=run_config_hash,
            profile_consistency="proved-exhaustive",
            coverage_mode="bidirectional-total",
            coverage_status="proved-exhaustive",
            safe_match_status="proved-exhaustive",
            eligible=True,
            granted=True,
            definedness="proved-exhaustive",
            laws=law_statuses(),
            properties=property_statuses(*(["proved-exhaustive"] * 4)),
            bridges={"carrier-source": "not-requested", "carrier-target": "proved-exhaustive"},
            policy_status="constrained",
            bl4=True,
        )
    )

    # A03: declared bounded-complement candidate.
    a03_context = context(t_int(0, 3), t_int(0, 3), t_int(0, 3), identity(), identity(), identity(), identity())
    a03_entries = [(v_int(value), f"risk-{value}") for value in range(4)]
    a03_atoms = [atom for _, atom in a03_entries]
    a03_policy = policy(
        dimension_id="risk-order",
        source_observer=observer_finite(a03_entries),
        target_observer=observer_finite(a03_entries),
        source_atoms=a03_atoms,
        target_atoms=a03_atoms,
        safe_relation=relation_exact(),
        match_relation=relation_exact(),
        match_coverage="nonempty",
    )
    a03_scope = scope("target-native-exact")
    a03_dir = "fixtures/attacks/A03"
    a03_paths = make_request_files(a03_dir, "A03", a03_context, a03_scope, a03_policy, "policy-sound-adequate", ALL_PROPERTIES, family_hash, run_config_hash)
    complement = {"kind": "bounded-complement", "min": 0, "max": 3}
    a03_candidate = candidate(complement, complement, mode="declared-candidate", rule="core.scalar.declared-bounded-complement", parent_path=[])
    write(f"{a03_dir}/transformation.json", a03_candidate)
    ctxp, scp, polp, reqp, req = a03_paths
    a03_common = dict(common_attack_base); a03_common["fixture_scope"] = a03_scope
    rows.append(registry_row(run_id="A03.base", fixture_kind="attack-base", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, granted=True, properties=property_statuses(*(["proved-exhaustive"] * 4)), **a03_common))
    rows.append(registry_row(run_id="A03", fixture_kind="attack", context_path="artifact/staging/generated-contexts/A03.json", base_context_path=ctxp, scope_path=scp, policy_path=polp, request_path=reqp, request=req, granted=False, properties=property_statuses(*(["disproved"] * 4)), transformation_required=True, transformation=a03_candidate, transformation_classification="lawful-harmful", candidate_binding="proved-exhaustive", base_alignment="proved-exhaustive", witness_kinds=["missing-required-match", "unsafe-false-agreement"], bl2=True, bl4=True, **a03_common))

    # A05: object-language Result branch laundering.
    a05_context = context(t_result(), t_result(), t_result(), identity(), identity(), identity(), identity())
    a05_policy = policy(
        dimension_id="result-role",
        source_observer=observer_constructor({"Err": "denied", "Ok": "success"}),
        target_observer=observer_constructor({"Err": "denied", "Ok": "success"}),
        source_atoms=["denied", "success"], target_atoms=["denied", "success"],
        safe_relation=relation_exact(), match_relation=relation_exact(), match_coverage="nonempty",
    )
    a05_scope = scope("target-native-exact")
    a05_dir = "fixtures/attacks/A05"
    a05_paths = make_request_files(a05_dir, "A05", a05_context, a05_scope, a05_policy, "policy-sound-adequate", ALL_PROPERTIES, family_hash, run_config_hash)
    swap_result = result_swap()
    a05_candidate = candidate(swap_result, swap_result, mode="enumerated", rule="core.result.compatible-branches", parent_path=[])
    write(f"{a05_dir}/transformation.json", a05_candidate)
    ctxp, scp, polp, reqp, req = a05_paths
    a05_common = dict(common_attack_base); a05_common["fixture_scope"] = a05_scope
    rows.append(registry_row(run_id="A05.base", fixture_kind="attack-base", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, granted=True, properties=property_statuses(*(["proved-exhaustive"] * 4)), **a05_common))
    rows.append(registry_row(run_id="A05", fixture_kind="attack", context_path="artifact/staging/generated-contexts/A05.json", base_context_path=ctxp, scope_path=scp, policy_path=polp, request_path=reqp, request=req, granted=False, properties=property_statuses(*(["disproved"] * 4)), transformation_required=True, transformation=a05_candidate, transformation_classification="lawful-harmful", candidate_binding="proved-exhaustive", base_alignment="proved-exhaustive", witness_kinds=["missing-required-match", "unsafe-false-agreement"], bl2=True, bl4=True, **a05_common))

    # H04: conservative target relation versus exact endpoint matching.
    h04_context = enum_context(
        ["S0_DENY", "S1_ALLOW", "S2_ALLOW"],
        ["T0_DENY", "T1_DENY", "T2_ALLOW"],
        ["C0", "C1", "C2"],
        {"S0_DENY": "C0", "S1_ALLOW": "C1", "S2_ALLOW": "C2"},
        {"T0_DENY": "C0", "T1_DENY": "C1", "T2_ALLOW": "C2"},
    )
    h04_pairs = [
        (v_sum("S0_DENY"), v_sum("T0_DENY")),
        (v_sum("S0_DENY"), v_sum("T2_ALLOW")),
        (v_sum("S1_ALLOW"), v_sum("T1_DENY")),
        (v_sum("S2_ALLOW"), v_sum("T2_ALLOW")),
    ]
    h04_scope = scope("target-native-exact", universe=pairs(h04_pairs))
    h04_source_observer = observer_constructor({"S0_DENY": "deny", "S1_ALLOW": "allow", "S2_ALLOW": "allow"})
    h04_target_observer = observer_constructor({"T0_DENY": "deny", "T1_DENY": "deny", "T2_ALLOW": "allow"})
    tna = relation_tna()
    h04_tna_policy = policy(
        dimension_id="policy-level", source_observer=h04_source_observer, target_observer=h04_target_observer,
        source_atoms=["allow", "deny"], target_atoms=["allow", "deny"], safe_relation=tna,
        match_relation=tna, match_coverage="bidirectional-total",
    )
    h04_tna_properties = ALL_PROPERTIES + [{"kind": "target-non-amplification", "dimension_ids": ["policy-level"]}]
    h04_tna_dir = "fixtures/benign/H04/tna"
    h04_tna_paths = make_request_files(h04_tna_dir, "H04.tna", h04_context, h04_scope, h04_tna_policy, "policy-sound-adequate", h04_tna_properties, family_hash, run_config_hash)
    ctxp, scp, polp, reqp, req = h04_tna_paths
    rows.append(registry_row(run_id="H04.tna", fixture_kind="benign", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=h04_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="bidirectional-total", coverage_status="proved-exhaustive", safe_match_status="not-requested", eligible=True, granted=True, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses(*(["proved-exhaustive"] * 5)), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", bl4=True))

    h04_exact_policy = policy(
        dimension_id="policy-level", source_observer=h04_source_observer, target_observer=h04_target_observer,
        source_atoms=["allow", "deny"], target_atoms=["allow", "deny"], safe_relation=relation_exact(),
        match_relation=relation_exact(), match_coverage="nonempty",
    )
    h04_exact_dir = "fixtures/benign/H04/exact"
    h04_exact_paths = make_request_files(h04_exact_dir, "H04.exact", h04_context, h04_scope, h04_exact_policy, "faithful-exact", ALL_PROPERTIES, family_hash, run_config_hash)
    ctxp, scp, polp, reqp, req = h04_exact_paths
    rows.append(registry_row(run_id="H04.exact", fixture_kind="benign", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=h04_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="nonempty", coverage_status="proved-exhaustive", safe_match_status="proved-exhaustive", eligible=True, granted=False, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses("disproved","proved-exhaustive","disproved","disproved"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", witness_kinds=["extra-safe-equality","unsafe-false-agreement"], bl4=True))

    # V01 exact comparator divergence with disjoint encoder images.
    v01_context = enum_context(
        ["s0", "s1"], ["t0", "t1"], ["L0", "L1", "R0", "R1"],
        {"s0":"L0","s1":"L1"}, {"t0":"R0","t1":"R1"},
        source_decode={"L0":"s0","L1":"s1","R0":"s0","R1":"s1"},
        target_decode={"L0":"t0","L1":"t1","R0":"t0","R1":"t1"},
    )
    v01_policy = policy(
        dimension_id="off-diagonal-safety",
        source_observer=observer_constructor({"s0":"s0","s1":"s1"}),
        target_observer=observer_constructor({"t0":"t0","t1":"t1"}),
        source_atoms=["s0","s1"], target_atoms=["t0","t1"],
        safe_relation=relation_table(["s0","s1"],["t0","t1"],[("s0","t1"),("s1","t0")]),
        match_relation=None, match_coverage="none", match_active=False,
    )
    k_source = domain_values([v_sum("L0"),v_sum("L1")])
    k_target = domain_values([v_sum("R0"),v_sum("R1")])
    for suffix, comparator, sound, granted in [
        ("carrier", "carrier-exact", "proved-exhaustive", True),
        ("target", "target-native-exact", "disproved", False),
    ]:
        v01_scope = scope(comparator, source_carrier_domain=k_source, target_carrier_domain=k_target)
        directory = f"fixtures/regressions/V01/{suffix}"
        paths = make_request_files(directory, f"V01.{suffix}", v01_context, v01_scope, v01_policy, "policy-sound", [{"kind":"policy-soundness"}], family_hash, run_config_hash, bridges=["carrier-target"])
        ctxp, scp, polp, reqp, req = paths
        rows.append(registry_row(run_id=f"V01.{suffix}", fixture_kind="regression", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=v01_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="none", coverage_status="not-requested", safe_match_status="not-requested", eligible=True, granted=granted, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses(sound,"not-requested","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"disproved"}, policy_status="constrained", witness_kinds=["unsafe-false-agreement"] if suffix == "target" else ["bridge-divergence"], bl4=True))

    # V02, V06, V10 and the explicit policy-vacuity conformance row.
    v02_policy = dict(a01_policy); v02_policy["match_dimensions"] = []; v02_policy["match_coverage"] = "nonempty"
    v02_dir = "fixtures/regressions/V02"
    v02_paths = make_request_files(v02_dir,"V02",a01_context,a01_scope,v02_policy,"policy-sound-adequate",[{"kind":"policy-soundness"},{"kind":"comparison-adequacy"}],family_hash,run_config_hash)
    ctxp, scp, polp, reqp, req = v02_paths
    rows.append(registry_row(run_id="V02", fixture_kind="regression", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=a01_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="invalid", coverage_mode="nonempty", coverage_status="disproved", safe_match_status="not-requested", eligible=False, granted=False, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses("invalid","invalid","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", witness_kinds=["match-coverage-empty"], bl4=True))

    v06_policy = policy(
        dimension_id="unsupported-global",
        source_observer={"kind":"external-observer-ref","id":"global-policy-callback"},
        target_observer={"kind":"external-observer-ref","id":"global-policy-callback"},
        source_atoms=["allow","deny"], target_atoms=["allow","deny"],
        safe_relation=relation_exact(), match_relation=None, match_coverage="none", match_active=False,
    )
    v06_dir = "fixtures/regressions/V06"
    v06_paths = make_request_files(v06_dir,"V06",a01_context,a01_scope,v06_policy,"policy-sound",[{"kind":"policy-soundness"}],family_hash,run_config_hash)
    ctxp, scp, polp, reqp, req = v06_paths
    rows.append(registry_row(run_id="V06", fixture_kind="regression", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=a01_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="none", coverage_status="not-requested", safe_match_status="not-requested", eligible=False, granted=False, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses("unknown","not-requested","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", bl4=True))

    v10_context = enum_context(["s0","s1"],["t0","t1"],["c0","c1"],{"s0":"c0","s1":"c1"},{"t0":"c0","t1":"c1"})
    v10_scope = scope("target-native-exact")
    v10_policy = policy(
        dimension_id="identity-table",
        source_observer=observer_constructor({"s0":"s0","s1":"s1"}),
        target_observer=observer_constructor({"t0":"t0","t1":"t1"}),
        source_atoms=["s0","s1"], target_atoms=["t0","t1"],
        safe_relation=relation_table(["s0","s1"],["t0","t1"],[("s0","t0"),("s1","t1")]),
        match_relation=relation_table(["s0","s1"],["t0","t1"],[("s0","t0")]),
        match_coverage="nonempty",
    )
    v10_dir = "fixtures/regressions/V10"
    v10_paths = make_request_files(v10_dir,"V10",v10_context,v10_scope,v10_policy,"policy-sound-adequate",ALL_PROPERTIES,family_hash,run_config_hash)
    ctxp, scp, polp, reqp, req = v10_paths
    rows.append(registry_row(run_id="V10", fixture_kind="regression", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=v10_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="nonempty", coverage_status="proved-exhaustive", safe_match_status="not-requested", eligible=True, granted=False, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses("proved-exhaustive","proved-exhaustive","disproved","disproved"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", witness_kinds=["extra-safe-equality"], bl4=True))

    vacuity_dir = "fixtures/regressions/policy-vacuity-conformance"
    vacuity_policy = policy_unconstrained()
    vacuity_paths = make_request_files(vacuity_dir,"policy-vacuity-conformance",a01_context,a01_scope,vacuity_policy,"policy-sound",[{"kind":"policy-soundness"}],family_hash,run_config_hash)
    ctxp, scp, polp, reqp, req = vacuity_paths
    rows.append(registry_row(run_id="policy-vacuity-conformance", fixture_kind="conformance", context_path=ctxp, base_context_path="not-applicable", scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=a01_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="none", coverage_status="not-requested", safe_match_status="not-requested", eligible=False, granted=False, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses("proved-exhaustive","not-requested","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="policy-unconstrained", bl4=True))

    # T01: lawful asymmetric non-closure under right-to-left composition.
    t01_context = context(t_sum(["a","b","c"]),t_sum(["a","b","c"]),t_sum(["a","b","c"]),identity(),identity(),identity(),identity())
    t01_scope = scope("target-native-exact")
    t01_source = observer_constructor({"a":"deny","b":"allow","c":"allow"})
    t01_target = observer_constructor({"a":"deny","b":"deny","c":"allow"})
    t01_policy = policy(dimension_id="policy-level",source_observer=t01_source,target_observer=t01_target,source_atoms=["allow","deny"],target_atoms=["allow","deny"],safe_relation=relation_tna(),match_relation=None,match_coverage="none",match_active=False)
    t01_dir = "fixtures/transformations/T01"
    t01_base_context_path = f"{t01_dir}/context.json"
    write(t01_base_context_path, t01_context)
    sigma1 = enum_map({"a":"b","b":"a","c":"c"})
    sigma2 = enum_map({"a":"a","b":"c","c":"b"})
    t01_candidates = [
        ("sigma1", "T01.sigma1", candidate(sigma1,sigma1,mode="enumerated",rule="core.enum.payload-compatible",parent_path=[]), "lawful-safe", True, "proved-exhaustive"),
        ("sigma2", "T01.sigma2", candidate(sigma2,sigma2,mode="enumerated",rule="core.enum.payload-compatible",parent_path=[]), "lawful-safe", True, "proved-exhaustive"),
        ("sigma1-compose-sigma2", "T01.sigma1-compose-sigma2", candidate(compose(sigma2,sigma1),compose(sigma1,sigma2),mode="enumerated",rule="core.nested.canonical-path-product",parent_path=[]), "lawful-harmful", False, "disproved"),
    ]
    for filename, run_id, transform, classification, granted, sound in t01_candidates:
        candidate_dir = f"{t01_dir}/{filename}"
        _, scp, polp, reqp, req = make_request_files(candidate_dir,"T01",t01_context,t01_scope,t01_policy,"policy-sound",[{"kind":"policy-soundness"}],family_hash,run_config_hash)
        write(f"{candidate_dir}/transformation.json", transform)
        rows.append(registry_row(run_id=run_id, fixture_kind="transformation", context_path=f"artifact/staging/generated-contexts/{run_id}.json", base_context_path=t01_base_context_path, scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=t01_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="none", coverage_status="not-requested", safe_match_status="not-requested", eligible=True, granted=granted, definedness="proved-exhaustive", laws=law_statuses(), properties=property_statuses(sound,"not-requested","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"}, policy_status="constrained", transformation_required=True, transformation=transform, transformation_classification=classification, candidate_binding="proved-exhaustive", base_alignment="not-required", witness_kinds=["unsafe-false-agreement"] if not granted else [], bl4=False))

    # T02: sound target-native relation but transformed target-carrier RT failure.
    t02_context = enum_context(
        ["x","y"],["a","b","c"],["0","1","2"],
        {"x":"0","y":"1"},{"a":"0","b":"1","c":"2"},
        source_decode={"0":"x","1":"y","2":"x"}, target_decode={"0":"a","1":"b","2":"a"},
    )
    ds = domain_values([v_sum("x"),v_sum("y")]); dt = domain_values([v_sum("a"),v_sum("b")]); kc = domain_values([v_sum("0"),v_sum("1")])
    t02_scope = scope("target-native-exact",source_domain=ds,target_domain=dt,source_comparison_domain=ds,target_comparison_domain=dt,universe=pairs([(v_sum("x"),v_sum("a")),(v_sum("x"),v_sum("b")),(v_sum("y"),v_sum("b"))]),source_carrier_domain=kc,target_carrier_domain=kc,source_full_transport_domain=ds,target_full_transport_domain=dt)
    t02_policy = policy(dimension_id="diagonal-safety",source_observer=observer_constructor({"x":"x","y":"y"}),target_observer=observer_constructor({"a":"a","b":"b","c":"c"}),source_atoms=["x","y"],target_atoms=["a","b","c"],safe_relation=relation_table(["x","y"],["a","b","c"],[("x","a"),("y","b")]),match_relation=None,match_coverage="none",match_active=False)
    t02_dir = "fixtures/transformations/T02"
    t02_base_context_path = f"{t02_dir}/context.json"
    write(t02_base_context_path, t02_context)
    t02_candidate_dir = f"{t02_dir}/sigma"
    t02_paths = make_request_files(t02_candidate_dir,"T02",t02_context,t02_scope,t02_policy,"policy-sound",[{"kind":"policy-soundness"}],family_hash,run_config_hash)
    t02_sigma = enum_map({"0":"2","1":"1","2":"0"})
    t02_candidate = candidate(t02_sigma,t02_sigma,mode="enumerated",rule="core.enum.payload-compatible",parent_path=[])
    write(f"{t02_candidate_dir}/transformation.json", t02_candidate)
    ctxp, scp, polp, reqp, req = t02_paths
    rows.append(registry_row(run_id="T02.sigma", fixture_kind="transformation", context_path="artifact/staging/generated-contexts/T02.sigma.json", base_context_path=t02_base_context_path, scope_path=scp, policy_path=polp, request_path=reqp, request=req, fixture_scope=t02_scope, family_hash=family_hash, run_config_hash=run_config_hash, profile_consistency="proved-exhaustive", coverage_mode="none", coverage_status="not-requested", safe_match_status="not-requested", eligible=False, granted=False, definedness="proved-exhaustive", laws=law_statuses("disproved"), properties=property_statuses("proved-exhaustive","not-requested","not-requested","not-requested"), bridges={"carrier-source":"not-requested","carrier-target":"disproved"}, policy_status="constrained", transformation_required=True, transformation=t02_candidate, transformation_classification="law-breaking-or-inapplicable", candidate_binding="proved-exhaustive", base_alignment="not-required", witness_kinds=["roundtrip-failure"], bl4=False))

    # C01: total-success exact and preorder composition inputs, each with a
    # diagnostic semantic bundle so every registry declaration stays hashed.
    c_context = context(t_bool(),t_bool(),t_bool(),identity(),identity(),identity(),identity())
    c_scope = scope("target-native-exact")
    bool_entries = [(v_bool(False),"false"),(v_bool(True),"true")]
    c_exact_policy = policy(dimension_id="boolean",source_observer=observer_finite(bool_entries),target_observer=observer_finite(bool_entries),source_atoms=["false","true"],target_atoms=["false","true"],safe_relation=relation_exact(),match_relation=None,match_coverage="none",match_active=False)
    tna_relation = relation_tna()
    policy_entries = [(v_bool(False),"deny"),(v_bool(True),"allow")]
    c_tna_policy = policy(dimension_id="policy-level",source_observer=observer_finite(policy_entries),target_observer=observer_finite(policy_entries),source_atoms=["allow","deny"],target_atoms=["allow","deny"],safe_relation=tna_relation,match_relation=None,match_coverage="none",match_active=False)
    for suffix, relation, fixture_policy, entries in [("exact",relation_exact(),c_exact_policy,bool_entries),("tna",tna_relation,c_tna_policy,policy_entries)]:
        directory = f"fixtures/composition/C01/{suffix}"
        paths = make_request_files(directory,f"C01.{suffix}",c_context,c_scope,fixture_policy,"diagnostic",[],family_hash,run_config_hash,laws={key:False for key in ALL_LAW_FLAGS})
        observer = observer_finite(entries)
        judgment = {"input_type":t_bool(),"output_type":t_bool(),"input_domain":domain_all(),"output_domain":domain_all(),"input_observer":observer,"output_observer":observer,"adapter":identity(),"relation":relation}
        composition = {"first":judgment,"second":judgment,"composed_relation":relation}
        write(f"{directory}/composition.json", composition)
        ctxp, scp, polp, reqp, req = paths
        rows.append(registry_row(run_id=f"C01.{suffix}",fixture_kind="composition",context_path=ctxp,base_context_path="not-applicable",scope_path=scp,policy_path=polp,request_path=reqp,request=req,fixture_scope=c_scope,family_hash=family_hash,run_config_hash=run_config_hash,profile_consistency="proved-exhaustive",coverage_mode="none",coverage_status="not-requested",safe_match_status="not-requested",eligible=False,granted=False,definedness="proved-exhaustive",laws=law_statuses(),properties=property_statuses(*(["not-requested"]*4)),bridges={"carrier-source":"not-requested","carrier-target":"proved-exhaustive"},policy_status="constrained",bl4=True))

    rows.sort(key=lambda row: row["run_id"])
    registry = {
        "schema": "gluerift.fixture-registry/v0.3.1a",
        "semantic_contract_version": "0.3.1a",
        "runs": rows,
    }
    write("fixtures/registry.json", registry)

    bl2 = {
        "schema": "gluerift.baseline-configuration/v0.3.1a",
        "baseline_id": "BL2",
        "acceptance_rule": "all-explicitly-requested-round-trip-laws-proved-exhaustive",
        "receives_endpoint_policy": False,
        "paired_run_ids": ["A01","A02","A03","A05"],
        "shared_candidate_scope_totality": True,
    }
    write("baselines/BL2/config.json", bl2)
    bl4 = {
        "schema": "gluerift.baseline-configuration/v0.3.1a",
        "baseline_id": "BL4",
        "acceptance_rule": "direct-comparator-indexed-relation-check",
        "shared_semantic_kernel": True,
        "same_first_top_level_witness_required": True,
        "parity_fields": [
            "profile-property-consistency",
            "match-coverage",
            "policy-vacuity",
            "comparator-definedness",
            "target-non-amplification",
            "policy-soundness",
            "comparison-adequacy",
            "comparison-precision",
            "faithful-comparison",
        ],
        "paired_run_ids": sorted([row["run_id"] for row in rows if row["bl4_paired"]]),
    }
    write("baselines/BL4/config.json", bl4)

    output_index = {
        "schema": "gluerift.fixture-source-index/v0.3.1a",
        "generator_logical_path": "fixtures/generate.py",
        "run_configuration_sha256": run_config_hash,
        "transformation_family_sha256": family_hash,
        "fixture_registry_sha256": digest(registry),
        "run_count": len(rows),
        "run_ids": [row["run_id"] for row in rows],
    }
    write("fixtures/source-index.json", output_index)


if __name__ == "__main__":
    generate()
