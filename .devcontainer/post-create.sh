#!/usr/bin/env bash
set -euo pipefail

# Bring up the egress firewall first (same as postStartCommand) so the rest of
# this script runs under the same network policy a normal session would.
sudo /usr/local/bin/init-firewall.sh

echo "Setting up development environment..."

# lib/ recursively: forge derives remappings from the nested checkouts, and an
# empty lib/example-messaging-executor/evm/lib/forge-std makes it pick
# `evm/src/` as that library's root, which breaks the imports. aptos-cctp's
# nested submodules use git@ URLs and need the manual rewrite in aptos/README.md,
# so the aptos submodule is initialised at the top level only.
echo "Initialising git submodules..."
git submodule update --init
git submodule update --init --recursive -- lib

# yarn.lock is audited before anything is installed from it. yarn 1 returns a
# severity bitmask: 8 = high, 16 = critical. Either aborts the install.
SVM_DIR="svm/relay_cost_protection"
if [ -f "${SVM_DIR}/yarn.lock" ]; then
  echo "Auditing ${SVM_DIR}/yarn.lock..."
  audit_rc=0
  (cd "${SVM_DIR}" && yarn audit --groups dependencies) || audit_rc=$?
  if (( audit_rc & 24 )); then
    echo "yarn audit reported high or critical advisories (exit ${audit_rc})." >&2
    echo "Review them, update yarn.lock, then run: (cd ${SVM_DIR} && yarn install --frozen-lockfile)" >&2
  else
    echo "Installing ${SVM_DIR} dependencies from the lockfile..."
    (cd "${SVM_DIR}" && yarn install --frozen-lockfile)
  fi
else
  echo "${SVM_DIR} absent in this checkout; skipping yarn."
fi

echo "Verifying pinned toolchain..."
bash .devcontainer/verify-toolchain.sh --no-build

echo ""
echo "Dev container setup complete. Full smoke test: bash .devcontainer/verify-toolchain.sh"
