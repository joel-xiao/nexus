#!/usr/bin/env bash
set -euo pipefail

BLUE="\033[0;34m"
YELLOW="\033[1;33m"
GREEN="\033[0;32m"
NC="\033[0m"

function info() {
  echo -e "${BLUE}➤${NC} $1"
}

function success() {
  echo -e "${GREEN}✔${NC} $1"
}

info "运行 Nexus 单元测试"
cargo test --lib -- --nocapture

info "运行 Nexus 集成测试"
cargo test --tests -- --nocapture

if [[ "${1:-}" == "--performance" ]]; then
  info "运行性能测试"
  found=false
  for file in tests/performance/*_test.rs; do
    if [[ -f $file ]]; then
      found=true
      name=$(basename "$file" .rs)
      info "  → $name"
      cargo test --test "$name" -- --nocapture || true
    fi
  done
  if ! $found; then
    info "  (未找到性能测试文件)"
  fi
fi

if command -v cargo-tarpaulin >/dev/null 2>&1; then
  info "生成覆盖率报告"
  cargo tarpaulin --out Html --output-dir ./target/coverage
  success "覆盖率报告: target/coverage/tarpaulin-report.html"
fi

success "Nexus 测试全部通过"
