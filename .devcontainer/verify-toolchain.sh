#!/usr/bin/env bash
# Asserts the container toolchain matches the versions pinned in devcontainer.json,
# then smoke-builds each runtime. Exits non-zero on the first mismatch.
#
# Usage (inside the devcontainer): bash .devcontainer/verify-toolchain.sh [--no-build]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEVCONTAINER_JSON="${SCRIPT_DIR}/devcontainer.json"
RUN_BUILDS=true
[ "${1:-}" = "--no-build" ] && RUN_BUILDS=false

fail() { echo "FAIL: $*" >&2; exit 1; }
ok()   { printf 'ok   %-22s %s\n' "$1" "$2"; }
warn() { echo "WARN: $*" >&2; }

# name, actual, expected. Exact match only.
expect_eq() {
  [ -n "$2" ] || fail "$1: empty value"
  [ "$2" = "$3" ] || fail "$1: expected '$3', got '$2'"
  ok "$1" "$2"
}

pinned() {
  local value
  value="$(jq -r --arg k "$1" '.build.args[$k] // empty' "${DEVCONTAINER_JSON}")"
  [ -n "${value}" ] || fail "devcontainer.json has no build arg $1"
  echo "${value}"
}

[ "${DEVCONTAINER:-}" = "true" ] || fail "run this inside the devcontainer (DEVCONTAINER != true)"
command -v jq >/dev/null || fail "jq missing"

echo "== pinned versions"
expect_eq "claude"          "$(claude --version | awk '{print $1}')"                        "$(pinned CLAUDE_CODE_VERSION)"
expect_eq "forge"           "$(forge --version | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)" "$(pinned FOUNDRY_VERSION)"
expect_eq "solana"          "$(solana --version | awk '{print $2}')"                        "$(pinned SOLANA_VERSION)"
expect_eq "anchor"          "$(anchor --version | awk '{print $2}')"                        "$(pinned ANCHOR_VERSION)"
expect_eq "aptos"           "$(aptos --version | awk '{print $2}')"                         "$(pinned APTOS_VERSION)"
expect_eq "rustup"          "$(rustup --version 2>/dev/null | head -1 | awk '{print $2}')"  "$(pinned RUSTUP_VERSION)"
expect_eq "rustc (default)" "$(rustc --version | awk '{print $2}')"                         "$(pinned RUST_VERSION)"
expect_eq "rustc (extra)"   "$(rustup run "$(pinned RUST_EXTRA_VERSION)" rustc --version | awk '{print $2}')" "$(pinned RUST_EXTRA_VERSION)"
expect_eq "bun"             "$(bun --version)"                                               "$(pinned BUN_VERSION)"
expect_eq "just"            "$(just --version | awk '{print $2}')"                           "$(pinned JUST_VERSION)"
expect_eq "surfpool"        "$(surfpool --version | awk '{print $2}')"                       "$(pinned SURFPOOL_VERSION)"
# Each SVM program pins its own toolchain; check the ones present in this checkout.
for svm_dir in relay_cost_protection post-vaa-relay; do
  toolchain_file="${REPO_ROOT}/svm/${svm_dir}/rust-toolchain.toml"
  if [ -f "${toolchain_file}" ]; then
    expect_eq "rustc (${svm_dir})" "$(cd "${REPO_ROOT}/svm/${svm_dir}" && rustc --version | awk '{print $2}')" \
                                 "$(sed -n 's/^channel = "\(.*\)"/\1/p' "${toolchain_file}")"
  fi
done
expect_eq "platform-tools"  "$(cargo build-sbf --version | awk '/^platform-tools/ {print $2}')" "$(pinned PLATFORM_TOOLS_VERSION)"
expect_eq "castellan"       "$(castellan --version | awk '{print $2}')"                     "$(pinned CASTELLAN_VERSION)"

echo "== presence"
for bin in cast anvil chisel solana-keygen solana-test-validator cargo-test-sbf yarn node git gh delta bun just surfpool; do
  command -v "${bin}" >/dev/null || fail "${bin} missing from PATH"
done
ok "binaries" "cast anvil chisel solana-keygen solana-test-validator cargo-test-sbf yarn node git gh delta bun just surfpool"

PT_DIR="${HOME}/.cache/solana/$(pinned PLATFORM_TOOLS_VERSION)/platform-tools"
[ -x "${PT_DIR}/rust/bin/rustc" ] || fail "platform-tools not pre-seeded at ${PT_DIR}"
ok "platform-tools cache" "${PT_DIR}"

echo "== firewall"
sudo -n castellan verify >/dev/null || fail "castellan verify failed"
ok "castellan verify" "active"

echo "== submodules"
cd "${REPO_ROOT}"
for sub in lib/forge-std lib/openzeppelin-contracts lib/example-messaging-executor; do
  [ -n "$(ls -A "${sub}" 2>/dev/null)" ] || fail "submodule ${sub} not initialised"
done
ok "submodules" "lib/*"

if [ "${RUN_BUILDS}" = "false" ]; then
  echo "== builds skipped (--no-build)"
  exit 0
fi

echo "== evm: forge build"
(cd "${REPO_ROOT}" && forge build >/dev/null) || fail "forge build"
ok "forge build" "ok"

if [ -d "${REPO_ROOT}/svm/relay_cost_protection" ]; then
  echo "== svm: anchor build"
  (cd "${REPO_ROOT}/svm/relay_cost_protection" && anchor build >/dev/null) || fail "anchor build"
  [ -f "${REPO_ROOT}/svm/relay_cost_protection/target/deploy/relay_cost_protection.so" ] || fail "anchor build produced no .so"
  ok "anchor build" "target/deploy/relay_cost_protection.so"
else
  warn "svm/relay_cost_protection absent in this checkout; skipping anchor build"
fi

if [ -d "${REPO_ROOT}/svm/post-vaa-relay" ]; then
  echo "== svm: post-vaa-relay build"
  (cd "${REPO_ROOT}/svm/post-vaa-relay" && just build testnet >/dev/null) || fail "just build testnet"
  [ -f "${REPO_ROOT}/svm/post-vaa-relay/target/deploy/post_vaa_relay.so" ] || fail "post-vaa-relay build produced no .so"
  ok "post-vaa-relay build" "target/deploy/post_vaa_relay.so"
else
  warn "svm/post-vaa-relay absent in this checkout; skipping its build"
fi

echo "== aptos: move compile"
APTOS_DIR="${REPO_ROOT}/aptos/cctp_v1_receive_with_gas_drop_off"
# aptos-cctp's own submodules (stablecoin-aptos) use git@ URLs; aptos/README.md documents the manual init.
if [ -f "${APTOS_DIR}/lib/aptos-cctp/stablecoin-aptos/packages/aptos_extensions/Move.toml" ]; then
  (cd "${APTOS_DIR}" && aptos move compile --dev --named-addresses cctp_v1_receive_with_gas_drop_off=0xcafe >/dev/null) || fail "aptos move compile"
  ok "aptos move compile" "ok"
else
  warn "aptos-cctp nested submodules not initialised; skipping move compile (see aptos/README.md)"
fi

echo "all checks passed"
