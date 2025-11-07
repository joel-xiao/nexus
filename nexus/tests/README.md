# Nexus 测试

## 运行测试

### 基本测试
```bash
# 运行所有测试（Mock 模式）
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test '*'

# 运行特定测试
cargo test test_name
```

### 使用测试脚本
```bash
# Mock 模式（默认，无需外部依赖）
./scripts/test/run_tests.sh

# 真实模式（需要配置环境变量）
./scripts/test/run_tests.sh --real

# 只运行单元测试
./scripts/test/run_tests.sh --unit

# 只运行集成测试
./scripts/test/run_tests.sh --integration

# 运行性能测试
./scripts/test/run_tests.sh --performance
```

## 测试模式

### Mock 模式（默认）
- 使用 Mock 适配器，不依赖外部服务
- 适合快速测试和 CI/CD
- 无需配置环境变量

### 真实模式
需要设置环境变量：
```bash
export NEXUS_TEST_MODE=real
export NEXUS_TEST_ADAPTER_NAME=openai
export NEXUS_TEST_API_KEY=sk-...
export NEXUS_TEST_MODEL=gpt-3.5-turbo
export NEXUS_TEST_BASE_URL=https://api.openai.com/v1
```

## 测试结构

```
tests/
├── common/              # 通用测试工具
│   ├── helpers.rs       # 测试辅助函数
│   ├── fixtures.rs      # 测试数据和配置
│   ├── mocks.rs         # Mock 对象
│   └── utils.rs         # 工具函数
├── unit/                # 单元测试
│   ├── application/     # 应用服务测试
│   ├── domain/          # 领域模型测试
│   └── infrastructure/  # 基础设施测试
├── integration/         # 集成测试
│   ├── api/             # API 端点测试
│   ├── services/        # 服务集成测试
│   └── end_to_end/      # 端到端测试
└── performance/         # 性能测试
    ├── api_performance_test.rs
    └── concurrency_test.rs
```

## 编写测试

### 单元测试示例
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // 测试代码
    }
}
```

### 集成测试示例
```rust
mod common;
use common::*;

#[tokio::test]
async fn test_api_endpoint() {
    let server = create_test_server().await;
    let response = server
        .post("/api/invoke")
        .json(&json!({"input": "test", "adapter": "mock"}))
        .await;
    response.assert_status_ok();
}
```

## 测试辅助函数

`tests/common/helpers.rs` 提供：
- `TestMode` - 测试模式枚举
- `create_test_server()` - 创建测试服务器
- `wait_for_adapters()` - 等待适配器注册
- `create_test_app_state()` - 创建测试状态

## 注意事项

1. **Mock 模式优先**: 默认使用 Mock 模式，确保测试快速可靠
2. **真实测试隔离**: 真实模式测试需要单独运行，避免影响其他测试
3. **环境变量**: 真实模式需要正确配置环境变量
4. **测试数据**: 使用 `fixtures.rs` 中的测试数据，避免硬编码
