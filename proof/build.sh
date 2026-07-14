#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
"$ROOT/bootstrap.sh"
export PATH="$ROOT/.toolchain/bin:$PATH"
cd "$ROOT"
lake build

