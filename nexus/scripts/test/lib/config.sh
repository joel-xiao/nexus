#!/usr/bin/env bash
# nexus 模块特定配置

# 检查必需的环境变量
check_required_env_vars() {
    local missing_vars=()
    
    if [[ -z "${NEXUS_TEST_ADAPTER_NAME:-}" ]]; then
        missing_vars+=("NEXUS_TEST_ADAPTER_NAME")
    fi
    
    if [[ -z "${NEXUS_TEST_API_KEY:-}" ]]; then
        missing_vars+=("NEXUS_TEST_API_KEY")
    fi
    
    if [[ ${#missing_vars[@]} -gt 0 ]]; then
        warning "以下环境变量未设置（真实测试模式需要）:"
        for var in "${missing_vars[@]}"; do
            warning "  - $var"
        done
        warning ""
        warning "示例配置:"
        warning "  export NEXUS_TEST_ADAPTER_NAME=qwen"
        warning "  export NEXUS_TEST_API_KEY=your-api-key"
        warning "  export NEXUS_TEST_MODEL=qwen-turbo"
        warning "  export NEXUS_TEST_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        warning ""
        warning "继续运行测试，但某些真实测试可能会失败..."
    else
        success "环境变量检查通过"
    fi
}

# 显示帮助信息
show_help() {
    cat << EOF
用法: $0 [选项]

测试模式:
    --mock             使用 Mock 模式（默认，不依赖外部服务）
    --real             使用真实测试模式（需要配置环境变量）

测试类型:
    --unit             只运行单元测试
    --integration      只运行集成测试
    --performance      运行性能测试

其他选项:
    --coverage         生成覆盖率报告
    --verbose          详细输出
    --filter PATTERN   过滤测试名称
    --help             显示此帮助信息

环境变量（真实测试模式）:
    NEXUS_TEST_ADAPTER_NAME    适配器名称
    NEXUS_TEST_API_KEY          API 密钥
    NEXUS_TEST_MODEL            模型名称
    NEXUS_TEST_BASE_URL         基础 URL
    NEXUS_TEST_ADAPTER          默认适配器名称

示例:
    # Mock 模式测试（默认）
    $0
    $0 --mock

    # 真实测试模式
    $0 --real
    NEXUS_TEST_ADAPTER_NAME=qwen NEXUS_TEST_API_KEY=xxx $0 --real

    # 只运行单元测试
    $0 --unit

    # 运行性能测试
    $0 --performance

    # 生成覆盖率报告
    $0 --coverage
EOF
}

