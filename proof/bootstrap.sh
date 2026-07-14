#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
TOOLCHAIN="$ROOT/.toolchain"
EXPECTED_VERSION="4.19.0"
EXPECTED_COMMIT="6caaee842e94"

if [[ -x "$TOOLCHAIN/bin/lean" ]]; then
  actual="$($TOOLCHAIN/bin/lean --version)"
  if [[ "$actual" == *"version $EXPECTED_VERSION"* && "$actual" == *"commit $EXPECTED_COMMIT"* ]]; then
    exit 0
  fi
  echo "proof/.toolchain exists but is not the pinned Lean build: $actual" >&2
  exit 1
fi

os="$(uname -s)"
arch="$(uname -m)"
if [[ "$os" != "Darwin" || "$arch" != "arm64" ]]; then
  echo "No workspace-local archive is locked for $os/$arch." >&2
  echo "Install the exact toolchain from lean-toolchain or extend toolchain-lock.json with a verified official archive." >&2
  exit 1
fi

command -v curl >/dev/null
command -v zstd >/dev/null
command -v tar >/dev/null

url="https://github.com/leanprover/lean4/releases/download/v4.19.0/lean-4.19.0-darwin_aarch64.tar.zst"
expected="94d4246fd90a152a4819419498d3e45c941a31638a57a0a8e32561494ab6cea7"
archive="$(mktemp "${TMPDIR:-/tmp}/gluerift-lean.XXXXXX.tar.zst")"
staging="$ROOT/.toolchain.staging.$$"
cleanup() {
  rm -f "$archive"
  rm -rf "$staging"
}
trap cleanup EXIT

curl -fL --retry 3 -o "$archive" "$url"
if command -v shasum >/dev/null; then
  actual_hash="$(shasum -a 256 "$archive" | awk '{print $1}')"
else
  actual_hash="$(sha256sum "$archive" | awk '{print $1}')"
fi
if [[ "$actual_hash" != "$expected" ]]; then
  echo "Lean archive checksum mismatch: expected $expected, got $actual_hash" >&2
  exit 1
fi

mkdir -p "$staging"
zstd -q -d -c "$archive" | tar -xf - -C "$staging" --strip-components=1
mv "$staging" "$TOOLCHAIN"
trap - EXIT
rm -f "$archive"

"$TOOLCHAIN/bin/lean" --version
"$TOOLCHAIN/bin/lake" --version

