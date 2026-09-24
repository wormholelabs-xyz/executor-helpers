# syntax=docker/dockerfile:1
#
# Devcontainer for executor-helpers: Foundry (EVM), Rust + Solana + Anchor + surfpool (SVM),
# Aptos CLI (Move), Bun and just, plus Claude Code behind the castellan egress firewall. Every third-party artifact is pinned by
# version and SHA-256 in devcontainer.json; nothing follows "latest".
#
# The `solana` and `surfpool` stages download the official x86_64 tarball on amd64 and build from
# the pinned commit on arm64 (neither project publishes Linux arm64 archives).

# Common base stage for downloading tools
FROM debian:bookworm-slim@sha256:4724b8cc51e33e398f0e2e15e18d5ec2851ff0c2280647e1310bc1642182655d AS downloader
RUN apt-get update && apt-get install -y --no-install-recommends \
  curl \
  unzip \
  bzip2 \
  git \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Stage: Download and extract git-delta
FROM downloader AS git-delta
ARG GIT_DELTA_VERSION
ARG GIT_DELTA_CHECKSUM_AMD64
ARG GIT_DELTA_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    EXPECTED_CHECKSUM="${GIT_DELTA_CHECKSUM_AMD64}"; \
  else \
    EXPECTED_CHECKSUM="${GIT_DELTA_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/git-delta.deb "https://github.com/dandavison/delta/releases/download/${GIT_DELTA_VERSION}/git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/git-delta.deb" | sha256sum -c - && \
  dpkg-deb -x /tmp/git-delta.deb /opt/delta

# Stage: Download oh-my-zsh at pinned commit
FROM downloader AS oh-my-zsh
ARG OHMYZSH_COMMIT
RUN git init /opt/ohmyzsh \
    && git -C /opt/ohmyzsh remote add origin https://github.com/ohmyzsh/ohmyzsh.git \
    && git -C /opt/ohmyzsh fetch --depth 1 origin ${OHMYZSH_COMMIT} \
    && git -C /opt/ohmyzsh checkout FETCH_HEAD \
    && rm -rf /opt/ohmyzsh/.git

# Stage: Download powerlevel10k at pinned commit
FROM downloader AS powerlevel10k
ARG POWERLEVEL10K_COMMIT
RUN git init /opt/powerlevel10k \
    && git -C /opt/powerlevel10k remote add origin https://github.com/romkatv/powerlevel10k.git \
    && git -C /opt/powerlevel10k fetch --depth 1 origin ${POWERLEVEL10K_COMMIT} \
    && git -C /opt/powerlevel10k checkout FETCH_HEAD \
    && rm -rf /opt/powerlevel10k/.git

# Stage: download and verify the castellan firewall binary
# Pre-built release from https://github.com/wormholelabs-xyz/castellan/releases.
FROM downloader AS castellan-download
ARG CASTELLAN_VERSION
ARG CASTELLAN_CHECKSUM_AMD64
ARG CASTELLAN_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    TARBALL="castellan-linux-x86_64.tar.gz"; EXPECTED_CHECKSUM="${CASTELLAN_CHECKSUM_AMD64}"; \
  else \
    TARBALL="castellan-linux-aarch64.tar.gz"; EXPECTED_CHECKSUM="${CASTELLAN_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/castellan.tar.gz \
    "https://github.com/wormholelabs-xyz/castellan/releases/download/v${CASTELLAN_VERSION}/${TARBALL}" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/castellan.tar.gz" | sha256sum -c - && \
  mkdir -p /opt/castellan && \
  tar -xzf /tmp/castellan.tar.gz -C /opt/castellan castellan && \
  chmod +x /opt/castellan/castellan

# Stage: Download and verify the Claude Code native binary
# Mirrors https://downloads.claude.ai/claude-code-releases/bootstrap.sh but pins the version and
# per-arch checksum. Refresh from the manifest when bumping:
#   curl -fsSL https://downloads.claude.ai/claude-code-releases/${CLAUDE_CODE_VERSION}/manifest.json
FROM downloader AS claude-code
ARG CLAUDE_CODE_VERSION
ARG CLAUDE_CODE_CHECKSUM_AMD64
ARG CLAUDE_CODE_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    PLATFORM="linux-x64"; EXPECTED_CHECKSUM="${CLAUDE_CODE_CHECKSUM_AMD64}"; \
  else \
    PLATFORM="linux-arm64"; EXPECTED_CHECKSUM="${CLAUDE_CODE_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/claude \
    "https://downloads.claude.ai/claude-code-releases/${CLAUDE_CODE_VERSION}/${PLATFORM}/claude" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/claude" | sha256sum -c - && \
  mkdir -p /opt/claude && \
  mv /tmp/claude /opt/claude/claude && \
  chmod +x /opt/claude/claude

# Stage: Download and verify Foundry (forge, cast, anvil, chisel)
# Checksums computed from the release archives at
# https://github.com/foundry-rs/foundry/releases/tag/v${FOUNDRY_VERSION}
# 1.3.6 is the last release whose `forge fmt` matches the committed Solidity; CI pins the same.
FROM downloader AS foundry
ARG FOUNDRY_VERSION
ARG FOUNDRY_CHECKSUM_AMD64
ARG FOUNDRY_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    EXPECTED_CHECKSUM="${FOUNDRY_CHECKSUM_AMD64}"; \
  else \
    EXPECTED_CHECKSUM="${FOUNDRY_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/foundry.tar.gz \
    "https://github.com/foundry-rs/foundry/releases/download/v${FOUNDRY_VERSION}/foundry_v${FOUNDRY_VERSION}_linux_${ARCH}.tar.gz" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/foundry.tar.gz" | sha256sum -c - && \
  mkdir -p /opt/foundry && \
  tar -xzf /tmp/foundry.tar.gz -C /opt/foundry && \
  test -x /opt/foundry/forge && test -x /opt/foundry/cast && test -x /opt/foundry/anvil && test -x /opt/foundry/chisel

# Stage: Download and verify the Aptos CLI
# Checksums computed from the release zips at
# https://github.com/aptos-labs/aptos-core/releases/tag/aptos-cli-v${APTOS_VERSION}
FROM downloader AS aptos
ARG APTOS_VERSION
ARG APTOS_CHECKSUM_AMD64
ARG APTOS_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    APTOS_ARCH="x86_64"; EXPECTED_CHECKSUM="${APTOS_CHECKSUM_AMD64}"; \
  else \
    APTOS_ARCH="aarch64"; EXPECTED_CHECKSUM="${APTOS_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/aptos.zip \
    "https://github.com/aptos-labs/aptos-core/releases/download/aptos-cli-v${APTOS_VERSION}/aptos-cli-${APTOS_VERSION}-Linux-${APTOS_ARCH}.zip" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/aptos.zip" | sha256sum -c - && \
  unzip -j -d /opt/aptos /tmp/aptos.zip aptos && \
  chmod +x /opt/aptos/aptos

# Stage: Download and verify Bun (runs scripts/*.ts without a package install)
# Checksums from SHASUMS256.txt at https://github.com/oven-sh/bun/releases/tag/bun-v${BUN_VERSION}
FROM downloader AS bun
ARG BUN_VERSION
ARG BUN_CHECKSUM_AMD64
ARG BUN_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    BUN_ARCH="x64"; EXPECTED_CHECKSUM="${BUN_CHECKSUM_AMD64}"; \
  else \
    BUN_ARCH="aarch64"; EXPECTED_CHECKSUM="${BUN_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/bun.zip \
    "https://github.com/oven-sh/bun/releases/download/bun-v${BUN_VERSION}/bun-linux-${BUN_ARCH}.zip" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/bun.zip" | sha256sum -c - && \
  unzip -j -d /opt/bun /tmp/bun.zip "bun-linux-${BUN_ARCH}/bun" && \
  chmod +x /opt/bun/bun

# Stage: Download and verify just (task runner used by svm/post-vaa-relay)
# Checksums from SHA256SUMS at https://github.com/casey/just/releases/tag/${JUST_VERSION}
FROM downloader AS just
ARG JUST_VERSION
ARG JUST_CHECKSUM_AMD64
ARG JUST_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    TRIPLE="x86_64-unknown-linux-musl"; EXPECTED_CHECKSUM="${JUST_CHECKSUM_AMD64}"; \
  else \
    TRIPLE="aarch64-unknown-linux-musl"; EXPECTED_CHECKSUM="${JUST_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/just.tar.gz \
    "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-${TRIPLE}.tar.gz" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/just.tar.gz" | sha256sum -c - && \
  mkdir -p /opt/just && tar -xzf /tmp/just.tar.gz -C /opt/just just && \
  chmod +x /opt/just/just

# Stage: Download and verify Solana platform-tools (SBF rustc/clang used by cargo build-sbf)
# Checksums computed from https://github.com/anza-xyz/platform-tools/releases/tag/${PLATFORM_TOOLS_VERSION}
FROM downloader AS platform-tools
ARG PLATFORM_TOOLS_VERSION
ARG PLATFORM_TOOLS_CHECKSUM_AMD64
ARG PLATFORM_TOOLS_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    PT_ARCH="x86_64"; EXPECTED_CHECKSUM="${PLATFORM_TOOLS_CHECKSUM_AMD64}"; \
  else \
    PT_ARCH="aarch64"; EXPECTED_CHECKSUM="${PLATFORM_TOOLS_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/platform-tools.tar.bz2 \
    "https://github.com/anza-xyz/platform-tools/releases/download/${PLATFORM_TOOLS_VERSION}/platform-tools-linux-${PT_ARCH}.tar.bz2" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/platform-tools.tar.bz2" | sha256sum -c - && \
  mkdir -p /opt/platform-tools && \
  tar -xjf /tmp/platform-tools.tar.bz2 -C /opt/platform-tools && \
  test -x /opt/platform-tools/rust/bin/rustc

# Stage: Download and verify rustup-init (shared by the Rust and Solana stages)
# Checksums from https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/<triple>/rustup-init.sha256
FROM downloader AS rustup-init
ARG RUSTUP_VERSION
ARG RUSTUP_CHECKSUM_AMD64
ARG RUSTUP_CHECKSUM_ARM64
RUN ARCH=$(dpkg --print-architecture) && \
  if [ "${ARCH}" = "amd64" ]; then \
    TRIPLE="x86_64-unknown-linux-gnu"; EXPECTED_CHECKSUM="${RUSTUP_CHECKSUM_AMD64}"; \
  else \
    TRIPLE="aarch64-unknown-linux-gnu"; EXPECTED_CHECKSUM="${RUSTUP_CHECKSUM_ARM64}"; \
  fi && \
  curl -fsSLo /tmp/rustup-init \
    "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${TRIPLE}/rustup-init" && \
  echo "${EXPECTED_CHECKSUM}  /tmp/rustup-init" | sha256sum -c - && \
  mkdir -p /opt/rustup && mv /tmp/rustup-init /opt/rustup/rustup-init && chmod +x /opt/rustup/rustup-init

# Stage: Rust dev toolchain + Anchor CLI (built from the pinned tag with its lockfile)
FROM node:24@sha256:bb20cf73b3ad7212834ec48e2174cdcb5775f6550510a5336b842ae32741ce6c AS rust-toolchain
RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential pkg-config libssl-dev libudev-dev \
  && rm -rf /var/lib/apt/lists/*
COPY --from=rustup-init /opt/rustup/rustup-init /usr/local/bin/rustup-init
USER node
ARG RUST_VERSION
ARG RUST_EXTRA_VERSION
RUN rustup-init -y --no-modify-path --profile minimal \
      --default-toolchain "${RUST_VERSION}" \
      --component rustfmt --component clippy --component rust-src
ENV PATH="/home/node/.cargo/bin:${PATH}"
# Second toolchain for svm/post-vaa-relay (its mollusk/agave dev-dependencies need a newer rustc).
RUN rustup toolchain install "${RUST_EXTRA_VERSION}" --profile minimal \
      --component rustfmt --component clippy
ARG ANCHOR_VERSION
ARG ANCHOR_COMMIT
RUN --mount=type=cache,target=/home/node/.cargo/registry,uid=1000,gid=1000 \
    --mount=type=cache,target=/home/node/.cargo/git,uid=1000,gid=1000 \
    cargo install --locked --git https://github.com/solana-foundation/anchor --rev "${ANCHOR_COMMIT}" anchor-cli && \
    test "$(anchor --version)" = "anchor-cli ${ANCHOR_VERSION}"

# Stage: Solana CLI (agave)
# amd64: official SHA-verified tarball. arm64: compile the pinned tag with agave's own Rust version
# and reproduce the tarball's bin/ layout (binaries + platform-tools-sdk).
FROM node:24@sha256:bb20cf73b3ad7212834ec48e2174cdcb5775f6550510a5336b842ae32741ce6c AS solana
RUN apt-get update && apt-get install -y --no-install-recommends \
  bzip2 build-essential pkg-config libssl-dev libudev-dev zlib1g-dev llvm clang cmake make \
  libprotobuf-dev protobuf-compiler libclang-dev \
  && rm -rf /var/lib/apt/lists/*
COPY --from=rustup-init /opt/rustup/rustup-init /usr/local/bin/rustup-init
COPY install-solana.sh /usr/local/bin/install-solana.sh
# ~/.cargo must exist node-owned before BuildKit mounts the caches beneath it as root.
RUN mkdir -p /opt/solana /home/node/.cargo && chown node:node /opt/solana /home/node/.cargo && chmod +x /usr/local/bin/install-solana.sh
USER node
ARG SOLANA_VERSION
ARG SOLANA_CHECKSUM_AMD64
ARG AGAVE_COMMIT
ARG AGAVE_RUST_VERSION
RUN --mount=type=cache,target=/home/node/.cargo/registry,uid=1000,gid=1000 \
    --mount=type=cache,target=/home/node/.cargo/git,uid=1000,gid=1000 \
    --mount=type=cache,target=/tmp/agave-target,uid=1000,gid=1000 \
    install-solana.sh

# Stage: surfpool (local SVM test network used by svm/post-vaa-relay end-to-end tests)
# amd64: official SHA-verified tarball. arm64: cargo install from the pinned commit with the
# repository's lockfile, using the newer toolchain its agave dependencies need.
FROM node:24@sha256:bb20cf73b3ad7212834ec48e2174cdcb5775f6550510a5336b842ae32741ce6c AS surfpool
RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential pkg-config libssl-dev libclang-dev clang libudev-dev ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=rustup-init /opt/rustup/rustup-init /usr/local/bin/rustup-init
COPY install-surfpool.sh /usr/local/bin/install-surfpool.sh
RUN mkdir -p /opt/surfpool /home/node/.cargo && chown node:node /opt/surfpool /home/node/.cargo && chmod +x /usr/local/bin/install-surfpool.sh
USER node
ARG SURFPOOL_VERSION
ARG SURFPOOL_COMMIT
ARG SURFPOOL_CHECKSUM_AMD64
ARG RUST_EXTRA_VERSION
RUN --mount=type=cache,target=/home/node/.cargo/registry,uid=1000,gid=1000 \
    --mount=type=cache,target=/home/node/.cargo/git,uid=1000,gid=1000 \
    install-surfpool.sh

# Base stage for VS Code extension downloads
FROM downloader AS vscode-downloader
COPY download-vsix.sh /usr/local/bin/download-vsix.sh
RUN chmod +x /usr/local/bin/download-vsix.sh

# VS Code extension stages: one per extension so each bump only busts its own layer.
# Run .devcontainer/fetch-vscode-extension-shas.sh to update versions and hashes.

FROM vscode-downloader AS vscode-ext-claude-code
ARG CLAUDE_CODE_EXT_VERSION
ARG CLAUDE_CODE_EXT_SHA256
RUN download-vsix.sh anthropic claude-code "${CLAUDE_CODE_EXT_VERSION}" "${CLAUDE_CODE_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-gitlens
ARG GITLENS_EXT_VERSION
ARG GITLENS_EXT_SHA256
RUN download-vsix.sh eamodio gitlens "${GITLENS_EXT_VERSION}" "${GITLENS_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-prettier
ARG PRETTIER_EXT_VERSION
ARG PRETTIER_EXT_SHA256
RUN download-vsix.sh esbenp prettier-vscode "${PRETTIER_EXT_VERSION}" "${PRETTIER_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-spellchecker
ARG SPELLCHECKER_EXT_VERSION
ARG SPELLCHECKER_EXT_SHA256
RUN download-vsix.sh streetsidesoftware code-spell-checker "${SPELLCHECKER_EXT_VERSION}" "${SPELLCHECKER_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-markdown-mermaid
ARG MARKDOWN_MERMAID_EXT_VERSION
ARG MARKDOWN_MERMAID_EXT_SHA256
RUN download-vsix.sh bierner markdown-mermaid "${MARKDOWN_MERMAID_EXT_VERSION}" "${MARKDOWN_MERMAID_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-rust-analyzer
ARG RUST_ANALYZER_EXT_VERSION
ARG RUST_ANALYZER_EXT_SHA256
RUN download-vsix.sh rust-lang rust-analyzer "${RUST_ANALYZER_EXT_VERSION}" "${RUST_ANALYZER_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-even-better-toml
ARG EVEN_BETTER_TOML_EXT_VERSION
ARG EVEN_BETTER_TOML_EXT_SHA256
RUN download-vsix.sh tamasfe even-better-toml "${EVEN_BETTER_TOML_EXT_VERSION}" "${EVEN_BETTER_TOML_EXT_SHA256}"

FROM vscode-downloader AS vscode-ext-solidity
ARG SOLIDITY_EXT_VERSION
ARG SOLIDITY_EXT_SHA256
RUN download-vsix.sh JuanBlanco solidity "${SOLIDITY_EXT_VERSION}" "${SOLIDITY_EXT_SHA256}"

# Main stage
FROM node:24@sha256:bb20cf73b3ad7212834ec48e2174cdcb5775f6550510a5336b842ae32741ce6c

ARG TZ
ENV TZ="$TZ"

# Set the locale
ENV LC_ALL=C.UTF-8 LANG=C.UTF-8

# Basic development tools, nftables (castellan firewall), and the native build deps that
# `cargo build` / `anchor build` need on the host side (cc, pkg-config, openssl, libudev).
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y --no-install-recommends \
  less \
  git \
  procps \
  sudo \
  fzf \
  zsh \
  man-db \
  unzip \
  bzip2 \
  gnupg2 \
  gh \
  iptables \
  nftables \
  iproute2 \
  dnsutils \
  jq \
  nano \
  vim \
  build-essential \
  pkg-config \
  libssl-dev \
  libudev-dev \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY --from=git-delta /opt/delta/usr/bin/delta /usr/local/bin/delta
COPY --from=foundry /opt/foundry/forge /opt/foundry/cast /opt/foundry/anvil /opt/foundry/chisel /usr/local/bin/
COPY --from=aptos /opt/aptos/aptos /usr/local/bin/aptos
COPY --from=bun /opt/bun/bun /usr/local/bin/bun
COPY --from=just /opt/just/just /usr/local/bin/just
COPY --from=surfpool /opt/surfpool/surfpool /usr/local/bin/surfpool

ARG USERNAME=node

# Persist bash history.
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Set `DEVCONTAINER` environment variable to help with orientation
ENV DEVCONTAINER=true

# Workspace, Claude config, and the directories that devcontainer.json mounts as named volumes.
# They must exist and be node-owned in the image; Docker seeds a new volume from the image
# directory on first mount and keeps its ownership.
RUN mkdir -p /workspace /home/node/.claude /home/node/.svm /home/node/.cargo/registry /home/node/.cargo/git /home/node/.cache/solana && \
  chown -R node:node /workspace /home/node/.claude /home/node/.svm /home/node/.cargo /home/node/.cache

WORKDIR /workspace

# Solana CLI in the layout agave-install uses, so `solana-install`-aware tooling finds it.
ARG SOLANA_VERSION
COPY --from=solana --chown=node:node /opt/solana/solana-release /home/node/.local/share/solana/install/releases/${SOLANA_VERSION}/solana-release
RUN ln -s /home/node/.local/share/solana/install/releases/${SOLANA_VERSION}/solana-release \
          /home/node/.local/share/solana/install/active_release && \
  chown -R node:node /home/node/.local

# Rust toolchain and Anchor CLI from the rust-toolchain stage
COPY --from=rust-toolchain --chown=node:node /home/node/.rustup /home/node/.rustup
COPY --from=rust-toolchain --chown=node:node /home/node/.cargo/bin /home/node/.cargo/bin

# Pre-seed platform-tools where cargo-build-sbf looks (~/.cache/solana/<ver>/platform-tools) and
# link it into the SDK dependencies dir, so `anchor build` uses the pinned compiler from the first run.
ARG PLATFORM_TOOLS_VERSION
COPY --from=platform-tools --chown=node:node /opt/platform-tools /home/node/.cache/solana/${PLATFORM_TOOLS_VERSION}/platform-tools
RUN mkdir -p /home/node/.local/share/solana/install/active_release/bin/platform-tools-sdk/sbf/dependencies && \
  ln -s /home/node/.cache/solana/${PLATFORM_TOOLS_VERSION}/platform-tools \
        /home/node/.local/share/solana/install/active_release/bin/platform-tools-sdk/sbf/dependencies/platform-tools && \
  chown -h node:node /home/node/.local/share/solana/install/active_release/bin/platform-tools-sdk/sbf/dependencies/platform-tools

# Set up non-root user
USER node

ENV PATH="/home/node/.cargo/bin:/home/node/.local/share/solana/install/active_release/bin:${PATH}"

# Set the default shell to zsh rather than sh
ENV SHELL=/bin/zsh

# Set the default editor and visual
ENV EDITOR=nano
ENV VISUAL=nano

# Install oh-my-zsh and powerlevel10k theme from pinned stages
COPY --chown=node:node --from=oh-my-zsh /opt/ohmyzsh /home/node/.oh-my-zsh
COPY --chown=node:node --from=powerlevel10k /opt/powerlevel10k /home/node/.oh-my-zsh/custom/themes/powerlevel10k
COPY --chown=node:node node-zshrc /home/node/.zshrc

# Install Claude Code from the SHA-verified native binary. Root-owned on PATH rather than
# `claude install`, which would set up a user-writable auto-updating launcher.
COPY --from=claude-code /opt/claude/claude /usr/local/bin/claude

# Install the castellan firewall: binary from the download stage, scripts and allow-list from
# .devcontainer. The allow-list is baked in at the path castellan reads by default; edits require
# a devcontainer rebuild.
#
# Security: the sudoers grant below gives effective root to anyone who can write
# init-firewall.sh, so it (and the supervisor and the full /usr/local/share/castellan chain)
# must stay root-owned and non-writable. The root COPYs already ensure this.
USER root
COPY --from=castellan-download /opt/castellan/castellan /usr/local/bin/castellan
COPY init-firewall.sh castellan-supervisor.sh /usr/local/bin/
RUN mkdir -p /usr/local/share/castellan
COPY allowed-domains.txt /usr/local/share/castellan/allowed-domains.txt
RUN chmod +x /usr/local/bin/init-firewall.sh /usr/local/bin/castellan-supervisor.sh && \
  echo "node ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh, /usr/local/bin/castellan verify" > /etc/sudoers.d/node-firewall && \
  chmod 0440 /etc/sudoers.d/node-firewall

# VS Code extensions (baked in for version pinning; installed at folderOpen via .vscode/tasks.json)
RUN mkdir -p /opt/vscode-extensions
COPY --from=vscode-ext-claude-code       /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-gitlens           /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-prettier          /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-spellchecker      /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-markdown-mermaid  /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-rust-analyzer     /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-even-better-toml  /opt/ext /opt/vscode-extensions/
COPY --from=vscode-ext-solidity          /opt/ext /opt/vscode-extensions/

USER node
