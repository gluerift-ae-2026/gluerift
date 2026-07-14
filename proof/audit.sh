#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
"$ROOT/bootstrap.sh"
export PATH="$ROOT/.toolchain/bin:$PATH"
cd "$ROOT"

forbidden='(^|[[:space:]])(sorry|admit)([[:space:]]|$)|^[[:space:]]*unsafe[[:space:]]+(def|theorem|opaque)'
if /usr/bin/grep -R -n -E --include='*.lean' "$forbidden" Gluerift Gluerift.lean; then
  echo "Forbidden Lean proof escape found." >&2
  exit 1
fi

/bin/mkdir -p "$ROOT/.lake"
build_output="$ROOT/.lake/build-output.txt"
if ! lake build >"$build_output" 2>&1; then
  /bin/cat "$build_output" >&2
  exit 1
fi
echo "Lean build completed for L1 L2 L3 L4 L5 L6 L7 L8 L9."
audit_output="$ROOT/.lake/audit-output.txt"
lake env lean Gluerift/AxiomAudit.lean 2>&1 | tee "$audit_output"

if /usr/bin/grep -n -E 'sorryAx|declaration uses \.unsafe' "$audit_output"; then
  echo "Untracked or unsafe axiom dependency found." >&2
  exit 1
fi

echo "Lean proof hygiene and axiom audit passed."
