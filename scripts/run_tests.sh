#!/usr/bin/env bash
# 统一测试脚本

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

RUN_AGENTFLOW=true
RUN_LLM_ADAPTER=true
RUN_NEXUS=true
TEST_MODE="${NEXUS_TEST_MODE:-mock}"

PASSED_ARGS=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --agentflow-only) RUN_LLM_ADAPTER=false; RUN_NEXUS=false; shift ;;
        --llm-only) RUN_AGENTFLOW=false; RUN_NEXUS=false; shift ;;
        --nexus-only) RUN_AGENTFLOW=false; RUN_LLM_ADAPTER=false; shift ;;
        --skip-agentflow) RUN_AGENTFLOW=false; shift ;;
        --skip-llm) RUN_LLM_ADAPTER=false; shift ;;
        --skip-nexus) RUN_NEXUS=false; shift ;;
        --mock) TEST_MODE="mock"; shift ;;
        --real) TEST_MODE="real"; shift ;;
        --help|-h)
            echo "用法: $0 [选项]"
            echo "选项: --agentflow-only, --llm-only, --nexus-only, --skip-*, --mock, --real"
            exit 0
            ;;
        *)
            PASSED_ARGS+=("$1")
            shift
            ;;
    esac
done

export NEXUS_TEST_MODE="$TEST_MODE"
export LLM_TEST_MODE="$TEST_MODE"
cd "$PROJECT_ROOT"

exit_code=0

[[ "$RUN_AGENTFLOW" == "true" ]] && {
    echo "=== agentflow ==="
    if [[ ${#PASSED_ARGS[@]} -gt 0 ]]; then
        bash "${PROJECT_ROOT}/agentflow/scripts/test/run_tests.sh" "${PASSED_ARGS[@]}" || exit_code=1
    else
        bash "${PROJECT_ROOT}/agentflow/scripts/test/run_tests.sh" || exit_code=1
    fi
}

[[ "$RUN_LLM_ADAPTER" == "true" ]] && {
    echo "=== llm-adapter ==="
    if [[ ${#PASSED_ARGS[@]} -gt 0 ]]; then
        bash "${PROJECT_ROOT}/llm-adapter/scripts/test/run_tests.sh" "${PASSED_ARGS[@]}" || exit_code=1
    else
        bash "${PROJECT_ROOT}/llm-adapter/scripts/test/run_tests.sh" || exit_code=1
    fi
}

[[ "$RUN_NEXUS" == "true" ]] && {
    echo "=== nexus ==="
    if [[ ${#PASSED_ARGS[@]} -gt 0 ]]; then
        bash "${PROJECT_ROOT}/nexus/scripts/test/run_tests.sh" "${PASSED_ARGS[@]}" || exit_code=1
    else
        bash "${PROJECT_ROOT}/nexus/scripts/test/run_tests.sh" || exit_code=1
    fi
}

exit $exit_code
