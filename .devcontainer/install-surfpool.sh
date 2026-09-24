#!/usr/bin/env bash
# Produces /opt/surfpool/surfpool.
# amd64: download and SHA-verify the official release tarball.
# arm64: cargo install from the pinned commit (surfpool publishes no Linux arm64 archive).
#
# Inputs (env): SURFPOOL_VERSION, SURFPOOL_COMMIT, SURFPOOL_CHECKSUM_AMD64, RUST_EXTRA_VERSION
set -euo pipefail
: "${SURFPOOL_VERSION:?}" "${SURFPOOL_COMMIT:?}" "${SURFPOOL_CHECKSUM_AMD64:?}" "${RUST_EXTRA_VERSION:?}"

OUT=/opt/surfpool
ARCH="$(dpkg --print-architecture)"

if [ "${ARCH}" = "amd64" ]; then
  curl -fsSLo /tmp/surfpool.tar.gz \
    "https://github.com/solana-foundation/surfpool/releases/download/v${SURFPOOL_VERSION}/surfpool-linux-x64.tar.gz"
  echo "${SURFPOOL_CHECKSUM_AMD64}  /tmp/surfpool.tar.gz" | sha256sum -c -
  tar -xzf /tmp/surfpool.tar.gz -C "${OUT}" surfpool
  rm /tmp/surfpool.tar.gz
else
  rustup-init -y --no-modify-path --profile minimal --default-toolchain "${RUST_EXTRA_VERSION}"
  export PATH="${HOME}/.cargo/bin:${PATH}"
  cargo install --locked --git https://github.com/solana-foundation/surfpool \
    --rev "${SURFPOOL_COMMIT}" surfpool-cli --root /tmp/surfpool-install
  install -m 0755 /tmp/surfpool-install/bin/surfpool "${OUT}/surfpool"
fi

actual="$("${OUT}/surfpool" --version | awk '{print $2}')"
if [ "${actual}" != "${SURFPOOL_VERSION}" ]; then
  echo "built surfpool ${actual}, expected ${SURFPOOL_VERSION}" >&2
  exit 1
fi
echo "surfpool ${SURFPOOL_VERSION} ready in ${OUT}"
