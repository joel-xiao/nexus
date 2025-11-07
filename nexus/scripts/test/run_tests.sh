#!/usr/bin/env bash
# Nexus 测试脚本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${SCRIPT_DIR}/lib/config.sh" 2>/dev/null || true

TEST_MODE="${NEXUS_TEST_MODE:-mock}"
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_PERFORMANCE=false
VERBOSE=false
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit) RUN_INTEGRATION=false; RUN_PERFORMANCE=false; shift ;;
        --integration) RUN_UNIT=false; RUN_PERFORMANCE=false; shift ;;
        --performance) RUN_UNIT=false; RUN_INTEGRATION=false; RUN_PERFORMANCE=true; shift ;;
        --mock) TEST_MODE="mock"; shift ;;
        --real) TEST_MODE="real"; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --filter) FILTER="$2"; shift 2 ;;
        --help|-h) show_help 2>/dev/null || cat <<EOF
用法: $0 [选项]
选项: --unit, --integration, --performance, --mock, --real, --verbose, --filter PATTERN
EOF
            exit 0 ;;
        *) shift ;;
    esac
done

export NEXUS_TEST_MODE="$TEST_MODE"
cd "$PROJECT_ROOT"

[[ "$TEST_MODE" == "real" ]] && check_required_env_vars 2>/dev/null || true

CARGO_TEST_ARGS=()
[[ "$VERBOSE" == "true" ]] && CARGO_TEST_ARGS+=(-- --nocapture)
[[ -n "$FILTER" ]] && CARGO_TEST_ARGS+=(-- --exact "$FILTER")

exit_code=0

[[ "$RUN_UNIT" == "true" ]] && {
    echo "运行单元测试..."
    if [[ ${#CARGO_TEST_ARGS[@]} -gt 0 ]]; then
        cargo test --lib "${CARGO_TEST_ARGS[@]}" || exit_code=1
    else
        cargo test --lib || exit_code=1
    fi
}

[[ "$RUN_INTEGRATION" == "true" ]] && {
    echo "运行集成测试..."
    if [[ ${#CARGO_TEST_ARGS[@]} -gt 0 ]]; then
        cargo test --test integration_test "${CARGO_TEST_ARGS[@]}" || exit_code=1
    else
        cargo test --test integration_test || exit_code=1
    fi
}

[[ "$RUN_PERFORMANCE" == "true" ]] && {
    echo "运行性能测试..."
    if [[ ${#CARGO_TEST_ARGS[@]} -gt 0 ]]; then
        cargo test --test performance_test "${CARGO_TEST_ARGS[@]}" || exit_code=1
    else
        cargo test --test performance_test || exit_code=1
    fi
}

exit $exit_code
