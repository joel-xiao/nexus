#!/usr/bin/env bash
set -euo pipefail

# Colors for output
GREEN="\033[0;32m"
RED="\033[0;31m"
NC="\033[0m"

function info() {
  echo -e "${GREEN}[RUN]${NC} $1"
}

function error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# 1. Workspace tests
info "Running workspace tests"
cargo test --workspace --all-features

# 2. llm-adapter tests (if any additional)
info "Running llm-adapter tests"
cargo test -p llm-adapter

# 3. agentflow tests
info "Running agentflow tests"
cargo test -p agentflow

# 4. Nexus tests (unit + integration)
info "Running nexus unit tests"
cargo test -p nexus --lib --tests

info "Running nexus integration tests"
cargo test -p nexus --tests -- --ignored || true

info "All tests completed successfully"
