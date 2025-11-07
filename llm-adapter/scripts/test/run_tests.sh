#!/usr/bin/env bash
set -euo pipefail

BLUE="[0;34m"
YELLOW="[1;33m"
GREEN="[0;32m"
NC="[0m"

function info() {
  echo -e "${BLUE}➤${NC} $1"
}

function success() {
  echo -e "${GREEN}✔${NC} $1"
}

info "运行 llm-adapter 单元测试"
cargo test --lib -- --nocapture

info "运行 llm-adapter 集成测试"
cargo test --tests -- --nocapture

if [[ "${1:-}" == "--performance" ]]; then
  info "运行性能测试"
  found=false
  if [[ -d tests/performance ]]; then
    for file in tests/performance/*_test.rs; do
      if [[ -f $file ]]; then
        found=true
        name=$(basename "$file" .rs)
        info "  → $name"
        cargo test --test "$name" -- --nocapture || true
      fi
    done
  fi
  if ! $found; then
    info "  (未找到性能测试文件)"
  fi
fi

if command -v cargo-tarpaulin >/dev/null 2>&1; then
  info "生成覆盖率报告"
  cargo tarpaulin --out Html --output-dir ./target/coverage
  success "覆盖率报告: target/coverage/tarpaulin-report.html"
fi

success "llm-adapter 测试全部通过"
