# Nexus 测试套件

本项目使用 Rust 内置测试框架和 `axum-test` 进行测试。

## 📁 测试目录结构

新的测试目录按照三层架构组织：

```
tests/
├── common/              # 通用测试工具
│   ├── helpers.rs      # 测试辅助函数
│   ├── fixtures.rs     # 测试数据和夹具
│   ├── utils.rs        # 测试工具函数
│   └── mocks.rs        # Mock 对象定义
│
├── unit/                # 单元测试（按架构分层）
│   ├── domain/         # Domain Layer 测试
│   │   └── config/     # 配置管理领域测试
│   │
│   │   注：适配器测试现在在 llm-adapter crate 中
│   ├── infrastructure/ # Infrastructure Layer 测试
│   │   └── queue/      # 队列基础设施测试
│   └── application/    # Application Layer 测试
│       ├── postprocessor_test.rs
│       ├── kb_test.rs
│       └── prompt_test.rs
│
├── integration/         # 集成测试
│   ├── api/           # API 集成测试
│   │   ├── health_test.rs
│   │   ├── invoke_test.rs
│   │   └── config/    # 配置 API 测试
│   ├── end_to_end/    # 端到端测试
│   └── services/      # 服务集成测试
│
└── performance/        # 性能测试
    ├── api_performance_test.rs
    └── concurrency_test.rs
```

## 🚀 运行测试

### 运行所有测试

```bash
cargo test
```

或使用测试脚本：

```bash
# 使用符号链接（推荐）
./run_tests.sh

# 或使用完整路径
./scripts/test/run_tests.sh

# 包含性能测试
./run_tests.sh --performance
```

### 运行特定类型的测试

```bash
# 只运行单元测试（库测试）
cargo test --lib

# 运行所有测试（包括集成测试和性能测试）
cargo test --all

# 运行特定测试文件
cargo test --test health_test

# 运行单个测试
cargo test test_health_endpoint
```

### 使用测试脚本

测试脚本提供了更友好的输出和选项：

```bash
# 运行常规测试
./scripts/test/run_tests.sh

# 运行性能测试
./scripts/test/run_tests.sh --performance

# 生成覆盖率报告（如果安装了 cargo-tarpaulin）
./scripts/test/run_tests.sh

# 运行其他 crate 的脚本（在 workspace 根目录执行）
../llm-adapter/scripts/test/run_tests.sh
../agentflow/scripts/test/run_tests.sh
```

## 📚 测试框架

### axum-test

用于测试 Axum Web 应用：
- 提供 `TestServer` 用于创建测试服务器
- 支持 HTTP 请求/响应断言
- 自动处理 JSON 序列化/反序列化

示例：
```rust
use axum_test::TestServer;
use nexus::create_test_app;

#[tokio::test]
async fn test_endpoint() {
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();
    
    let response = server.get("/health").await;
    response.assert_status_ok();
}
```

### 使用测试辅助函数

每个测试文件可以使用 `common` 模块的辅助函数：

```rust
mod common;
use common::wait_for_adapters;
use common::create_test_server;
use common::create_test_invoke_payload;

#[tokio::test]
async fn test_something() {
    wait_for_adapters().await;
    let server = create_test_server();
    // ...
}
```

## ✅ 测试覆盖的功能

### API 端点测试

- ✅ 健康检查 (`/health`)
- ✅ 就绪检查 (`/ready`)
- ✅ 指标端点 (`/metrics`)
- ✅ 调用端点 (`/api/invoke`)
- ✅ 配置管理端点
- ✅ 功能标志端点
- ✅ 路由规则端点
- ✅ 适配器管理端点

### 单元测试

- ✅ 应用状态初始化
- ✅ Mock 适配器
- ✅ 适配器注册表
- ✅ 配置管理器
- ✅ 功能标志存储
- ✅ 路由规则
- ✅ 任务队列
- ✅ 后处理器链
- ✅ 知识库
- ✅ 提示存储

### 性能测试

- ✅ 健康检查性能
- ✅ 并发请求处理
- ✅ 吞吐量测试

## 📝 编写新测试

### 单元测试示例

```rust
use nexus::domain::config::manager::ConfigManager;

#[tokio::test]
async fn test_config_manager() {
    let manager = ConfigManager::new();
    // 测试逻辑...
}
```

### 集成测试示例

```rust
mod common;
use common::wait_for_adapters;

use axum_test::TestServer;
use serde_json::json;
use nexus::create_test_app;

#[tokio::test]
async fn test_my_endpoint() {
    wait_for_adapters().await;
    let app = create_test_app();
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .post("/api/my-endpoint")
        .json(&json!({"key": "value"}))
        .await;
    
    response.assert_status_ok();
    let json: serde_json::Value = response.json();
    assert_eq!(json["status"], "ok");
}
```

## 🔍 测试组织原则

1. **按架构分层**：单元测试按照 Domain、Infrastructure、Application 分层
2. **模块化**：每个模块有独立的测试文件
3. **共享工具**：通用测试工具放在 `common/` 目录
4. **清晰命名**：测试文件命名清晰描述其测试内容

## 📊 测试覆盖率

可以使用 `cargo-tarpaulin` 生成覆盖率报告：

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir ./target/coverage
```

报告将生成在 `./target/coverage/tarpaulin-report.html`

## 🐛 故障排查

### 测试失败

如果测试失败，检查：
1. 导入路径是否正确（新架构使用 `nexus::domain::*` 等路径）
2. 是否添加了 `mod common;` 声明（如果需要使用 common 模块）
3. 异步测试是否正确使用 `#[tokio::test]`

### 编译错误

如果出现编译错误：
1. 确保所有导入路径已更新到新架构
2. 检查 `mod.rs` 文件是否正确导出模块
3. 运行 `cargo clean && cargo test` 清理并重新编译

## 📖 更多信息

- 测试设计文档：`docs/TESTS_DESIGN.md`
- 迁移状态：`docs/MIGRATION_STATUS.md`


### Workspace 脚本

如果在 workspace 根目录运行所有项目测试，可使用：

```bash
./scripts/run_tests.sh
```
