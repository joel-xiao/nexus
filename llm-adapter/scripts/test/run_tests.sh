#!/usr/bin/env bash
# llm-adapter 测试脚本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODULE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

TEST_MODE="${LLM_TEST_MODE:-mock}"
RUN_UNIT=true
RUN_INTEGRATION=true
VERBOSE=false
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit) RUN_INTEGRATION=false; shift ;;
        --integration) RUN_UNIT=false; shift ;;
        --mock) TEST_MODE="mock"; shift ;;
        --real) TEST_MODE="real"; shift ;;
        --verbose) VERBOSE=true; shift ;;
        --filter) FILTER="$2"; shift 2 ;;
        *) shift ;;
    esac
done

export LLM_TEST_MODE="$TEST_MODE"
cd "$MODULE_DIR"

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
        cargo test --tests "${CARGO_TEST_ARGS[@]}" || exit_code=1
    else
        cargo test --tests || exit_code=1
    fi
}

exit $exit_code
