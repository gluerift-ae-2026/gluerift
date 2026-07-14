#!/usr/bin/env python3
"""Strict RFC 8785 canonical JSON for GlueRift's integer/string subset."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any


class CanonicalizationError(ValueError):
    pass


MAX_SAFE_INTEGER = 2**53 - 1


def _validate(value: Any, path: str = "$") -> None:
    if value is None or isinstance(value, (bool, str)):
        return
    if isinstance(value, int) and not isinstance(value, bool):
        if not -MAX_SAFE_INTEGER <= value <= MAX_SAFE_INTEGER:
            raise CanonicalizationError(f"integer outside the RFC 8785 interoperable range at {path}")
        return
    if isinstance(value, float):
        raise CanonicalizationError(f"floating-point value forbidden at {path}")
    if isinstance(value, list):
        for index, item in enumerate(value):
            _validate(item, f"{path}[{index}]")
        return
    if isinstance(value, dict):
        for key, item in value.items():
            if not isinstance(key, str):
                raise CanonicalizationError(f"non-string object key at {path}")
            _validate(item, f"{path}.{key}")
        return
    raise CanonicalizationError(f"unsupported JSON value {type(value).__name__} at {path}")


def _render(value: Any) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False, allow_nan=False)
    if isinstance(value, list):
        return "[" + ",".join(_render(item) for item in value) + "]"
    if isinstance(value, dict):
        # RFC 8785 follows ECMAScript's lexicographic ordering of UTF-16 code
        # units, which differs from Python's Unicode scalar ordering outside
        # the BMP.
        keys = sorted(value, key=lambda key: key.encode("utf-16-be", "surrogatepass"))
        return "{" + ",".join(
            f"{json.dumps(key, ensure_ascii=False)}:{_render(value[key])}" for key in keys
        ) + "}"
    raise CanonicalizationError(f"unsupported JSON value {type(value).__name__}")


def canonical_bytes(value: Any) -> bytes:
    _validate(value)
    return _render(value).encode("utf-8")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_sha256(value: Any) -> str:
    return sha256_bytes(canonical_bytes(value))


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        value = json.load(
            handle,
            parse_float=_reject_float,
            object_pairs_hook=_object_without_duplicates,
        )
    _validate(value)
    return value


def _reject_float(token: str) -> None:
    raise CanonicalizationError(f"floating-point token forbidden: {token}")


def _object_without_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CanonicalizationError(f"duplicate object key: {key}")
        result[key] = value
    return result


def write_canonical(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    # RFC 8785 defines the serialization bytes themselves.  A trailing line
    # feed would be valid surrounding JSON whitespace, but it is not part of
    # the JCS serialization and would give the Rust and Python emitters two
    # different byte owners.
    path.write_bytes(canonical_bytes(value))


def canonical_file_sha256(path: Path) -> str:
    return canonical_sha256(load_json(path))
