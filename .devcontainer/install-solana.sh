#!/usr/bin/env bash
# Produces /opt/solana/solana-release in the layout of the official release tarball.
# amd64: download and SHA-verify the official archive.
# arm64: build agave from the pinned tag (anza publishes x86_64 Linux archives only).
#
# Inputs (env): SOLANA_VERSION, SOLANA_CHECKSUM_AMD64, AGAVE_COMMIT, AGAVE_RUST_VERSION
# Runs as the build user; cargo caches come from BuildKit cache mounts.
set -euo pipefail

: "${SOLANA_VERSION:?}" "${SOLANA_CHECKSUM_AMD64:?}" "${AGAVE_COMMIT:?}" "${AGAVE_RUST_VERSION:?}"

OUT=/opt/solana/solana-release
ARCH="$(dpkg --print-architecture)"

if [ "${ARCH}" = "amd64" ]; then
  curl -fsSLo /tmp/solana.tar.bz2 \
    "https://release.anza.xyz/v${SOLANA_VERSION}/solana-release-x86_64-unknown-linux-gnu.tar.bz2"
  echo "${SOLANA_CHECKSUM_AMD64}  /tmp/solana.tar.bz2" | sha256sum -c -
  tar -xjf /tmp/solana.tar.bz2 -C /opt/solana
  rm /tmp/solana.tar.bz2
else
  SRC=/tmp/agave-src
  export CARGO_TARGET_DIR=/tmp/agave-target
  git clone --depth 1 --branch "v${SOLANA_VERSION}" https://github.com/anza-xyz/agave.git "${SRC}"
  cd "${SRC}"
  actual="$(git rev-parse HEAD)"
  if [ "${actual}" != "${AGAVE_COMMIT}" ]; then
    echo "agave v${SOLANA_VERSION} resolved to ${actual}, expected ${AGAVE_COMMIT}" >&2
    exit 1
  fi

  rustup-init -y --no-modify-path --profile minimal --default-toolchain "${AGAVE_RUST_VERSION}"
  export PATH="${HOME}/.cargo/bin:${PATH}"

  # Same --exclude set as scripts/cargo-install-all.sh: keeps dev-context-only-utils from
  # being unified into the production binaries.
  # shellcheck source=/dev/null
  . scripts/dcou-tainted-packages.sh
  exclude_args=()
  for p in "${dcou_tainted_packages[@]}"; do
    exclude_args+=(--exclude "${p}")
  done

  bins=(
    solana solana-keygen solana-test-validator solana-faucet solana-genesis
    cargo-build-sbf cargo-test-sbf agave-validator agave-install agave-install-init
  )
  bin_args=()
  for b in "${bins[@]}"; do
    bin_args+=(--bin "${b}")
  done

  cargo build --release --locked --workspace "${exclude_args[@]}" "${bin_args[@]}"

  install -d "${OUT}/bin/platform-tools-sdk/sbf" "${OUT}/bin/deps"
  for b in "${bins[@]}"; do
    install -m 0755 "${CARGO_TARGET_DIR}/release/${b}" "${OUT}/bin/"
  done
  cp -a platform-tools-sdk/sbf/. "${OUT}/bin/platform-tools-sdk/sbf/"
  printf 'channel: v%s\ncommit: %s\n' "${SOLANA_VERSION}" "${AGAVE_COMMIT}" > "${OUT}/version.yml"
fi

actual_version="$("${OUT}/bin/solana" --version | awk '{print $2}')"
if [ "${actual_version}" != "${SOLANA_VERSION}" ]; then
  echo "built solana ${actual_version}, expected ${SOLANA_VERSION}" >&2
  exit 1
fi
test -x "${OUT}/bin/cargo-build-sbf"
test -f "${OUT}/bin/platform-tools-sdk/sbf/env.sh"
echo "solana ${SOLANA_VERSION} ready in ${OUT}"
