# AI 使用指南

本文档为 AI 助手提供项目概览和关键信息，帮助快速理解项目结构和开发规范。

## 📋 项目概览

### 项目结构
```
nexus-workspace/
├── llm-adapter/      # LLM 适配器框架（可独立发布）
├── agentflow/        # 多代理协作框架（可独立发布）
├── nexus/            # 业务应用（集成示例）
└── scripts/          # 测试和构建脚本
```

### 核心依赖关系
```
nexus → llm-adapter (独立工具)
nexus → agentflow (独立工具)

llm-adapter ⊥ agentflow (互不依赖)
```

## 🎯 项目模块说明

### 1. llm-adapter
**位置**: `llm-adapter/`  
**类型**: 独立可发布的库  
**功能**: 统一 LLM API 适配器框架

**核心模块**:
- `src/providers/` - LLM 提供商实现（OpenAI, DeepSeek, Qwen, Zhipu, Doubao 等）
- `src/registry.rs` - 适配器注册表，管理所有适配器实例
- `src/factory.rs` - 适配器工厂，创建适配器实例
- `src/wrapper.rs` - 限流、计费、并发控制包装器
- `src/generic.rs` - 通用 HTTP 适配器，支持自定义 API
- `src/billing.rs` - 计费跟踪
- `src/rate_limit.rs` - 速率限制
- `src/guard.rs` - 并发控制

**核心 Trait**:
- `Adapter` - 适配器接口，包含 `invoke` 和 `invoke_with_options` 方法
- `LLMProvider` - 在 `agentflow` 中定义的抽象层，支持扩展

**测试脚本**: `llm-adapter/scripts/test/run_tests.sh`
- 支持 Mock 和真实测试模式
- 使用 `--mock` 或 `--real` 选项
- 真实模式需要环境变量：`LLM_TEST_API_KEY`, `LLM_TEST_PROVIDER` 等

### 2. agentflow
**位置**: `agentflow/`  
**类型**: 独立可发布的库  
**功能**: 多代理协作和工作流编排框架

**核心模块**:
- `src/agent.rs` - Agent trait (`AgentFlowAgent`) 和消息协议
- `src/orchestrator.rs` - 代理编排器 (`AgentOrchestrator`)，实现多轮对话
- `src/workflow.rs` - 工作流引擎
- `src/config.rs` - 配置管理
- `src/llm_provider.rs` - LLM 提供者抽象 trait，支持扩展

**核心功能**:
- **多角色支持**: User, Assistant, Planner, Executor, Reviewer, Coordinator, Expert, Custom
- **发言者选择**: RoundRobin, Random, Manual, Auto
- **终止条件**: 可配置的对话终止条件
- **消息历史**: 完整的对话历史管理
- **自动路由**: Agent 可以根据响应内容自动指定下一个 Agent

**测试脚本**: `agentflow/scripts/test/run_tests.sh`
- 标准测试脚本，支持单元、集成、性能测试

**扩展性**:
- 通过 `LLMProvider` trait 支持任意 LLM 库
- 可以自定义 Agent 实现
- 详见 `agentflow/EXTENDING.md`

### 3. nexus
**位置**: `nexus/`  
**类型**: 业务应用（集成示例）  
**功能**: HTTP API 网关，集成 llm-adapter 和 agentflow

**架构层次**:
- `src/routes/` - Presentation Layer（HTTP 路由和 OpenAPI 文档）
- `src/routes/handlers/` - Application Layer（业务处理逻辑）
- `src/domain/` - Domain Layer（领域模型和服务）
- `src/infrastructure/` - Infrastructure Layer（缓存、队列、消息总线）
- `src/integration/` - 桥接层（连接 llm-adapter 和 agentflow）
- `src/application/` - 应用服务（知识库、规划器、后处理器、提示模板）

**核心功能**:
1. **LLM 调用** (`/api/invoke`)
   - 统一调用接口
   - 支持路由规则选择模型
   - 支持提示模板
   - 支持知识库检索
   - 支持后处理链（审计、PII 脱敏、格式化）

2. **多智能体对话** (`/api/agents/*`)
   - `/api/agents/conversation` - 启动多角色对话
   - `/api/agents/orchestrate` - 编排多个 Agent
   - `/api/agents` - 列出可用 Agent
   - 支持角色配置（Planner, Executor, Reviewer 等）
   - 支持发言者选择策略
   - 支持终止条件

3. **配置管理** (`/api/config/*`)
   - `/api/config/adapters` - 适配器管理（CRUD、统计、按模型查询）
   - `/api/config/prompts` - 提示模板管理
   - `/api/config/flags` - 功能开关管理
   - `/api/config/routing` - 路由规则管理
   - `/api/config/reload` - 热重载配置
   - `/api/config/import-export` - 配置导入导出

4. **健康检查** (`/health`, `/ready`)

**测试脚本**: `nexus/scripts/test/run_tests.sh`
- 支持 Mock 和真实测试模式
- 使用 `--mock` 或 `--real` 选项
- 真实模式需要环境变量：`NEXUS_TEST_ADAPTER_NAME`, `NEXUS_TEST_API_KEY` 等

## 🔌 API 端点

### 核心 API

#### 1. LLM 调用
```
POST /api/invoke
```
请求体：
```json
{
  "input": "Hello, world!",
  "adapter": "openai",
  "model": "gpt-4o-mini",
  "user_id": "user123",
  "prompt_name": "default",
  "temperature": 0.7
}
```

#### 2. 多智能体对话
```
POST /api/agents/conversation
```
请求体：
```json
{
  "message": "请帮我规划一个项目",
  "agent_configs": [
    {
      "agent_id": "adapter1",
      "role": "planner",
      "name": "规划师",
      "system_prompt": "你是一个专业的项目规划师..."
    },
    {
      "agent_id": "adapter2",
      "role": "executor",
      "name": "执行者"
    }
  ],
  "speaker_selection": "auto",
  "max_rounds": 10
}
```

```
POST /api/agents/orchestrate
```
请求体：
```json
{
  "initial_message": "请完成这个任务",
  "agent_configs": [
    {
      "id": "planner",
      "name": "规划者",
      "role": "planner",
      "system_prompt": "...",
      "adapter_name": "openai"
    }
  ]
}
```

#### 3. 配置管理
```
GET    /api/config/adapters              # 列出所有适配器
GET    /api/config/adapters/{name}       # 获取适配器详情
DELETE /api/config/adapters/{name}      # 删除适配器
GET    /api/config/adapters/stats       # 模型统计
GET    /api/config/adapters/by-model/{model_name}  # 按模型查询适配器

GET    /api/config/prompts              # 列出提示模板
POST   /api/config/prompts              # 创建提示模板
PUT    /api/config/prompts/{name}       # 更新提示模板
DELETE /api/config/prompts/{name}      # 删除提示模板

GET    /api/config/flags                # 列出功能开关
POST   /api/config/flags                # 创建功能开关
PUT    /api/config/flags/{name}         # 更新功能开关
DELETE /api/config/flags/{name}        # 删除功能开关

GET    /api/config/routing              # 列出路由规则
POST   /api/config/routing              # 创建路由规则
PUT    /api/config/routing/{id}         # 更新路由规则
DELETE /api/config/routing/{id}        # 删除路由规则

POST   /api/config/reload/adapter       # 热重载适配器
POST   /api/config/reload/prompt       # 热重载提示模板

POST   /api/config/export               # 导出配置
POST   /api/config/import               # 导入配置
```

**完整 API 文档**: 运行服务后访问 `http://localhost:3000/docs`

## 🧪 测试系统

### 测试模式
- **Mock 模式**（默认）: 使用 Mock 适配器，不依赖外部服务
- **真实模式**: 使用真实 API，需要配置环境变量

### 测试脚本位置
- 根目录: `scripts/run_tests.sh` - 运行所有项目测试
- llm-adapter: `llm-adapter/scripts/test/run_tests.sh`
- agentflow: `agentflow/scripts/test/run_tests.sh`
- nexus: `nexus/scripts/test/run_tests.sh`

### 测试选项
所有测试脚本支持：
- `--unit` - 只运行单元测试
- `--integration` - 只运行集成测试
- `--performance` - 运行性能测试
- `--coverage` - 生成覆盖率报告
- `--verbose` - 详细输出
- `--filter PATTERN` - 过滤测试名称

### 测试辅助模块
`nexus/tests/common/` 提供：
- `helpers.rs` - 测试辅助函数（TestMode, wait_for_adapters 等）
- `fixtures.rs` - 测试数据和配置
- `mocks.rs` - Mock 对象
- `utils.rs` - 工具函数

## 📚 关键文档

### 项目文档
- `README.md` - 项目总览
- `nexus/README.md` - Nexus 应用说明
- `llm-adapter/README.md` - LLM 适配器文档
- `agentflow/README.md` - AgentFlow 文档
- `agentflow/EXTENDING.md` - AgentFlow 扩展指南

### 架构文档
- `nexus/docs/ARCHITECTURE.md` - 架构说明
- `nexus/docs/DEPLOYMENT.md` - 部署指南
- `nexus/docs/QUICKSTART.md` - 快速开始
- `nexus/docs/ENV.md` - 环境变量说明
- `nexus/docs/FRONTEND_API_GUIDE.md` - 前端 API 指南

## 🔧 开发规范

### 代码组织
- **三层架构**: Presentation → Application → Domain
- **模块独立**: 每个业务领域独立模块
- **依赖倒置**: Domain Layer 不依赖上层
- **桥接层**: `nexus/src/integration/` 用于连接独立工具

### 错误处理
- 使用 `Result<T, E>` 而不是 `unwrap()`
- 统一使用 `anyhow::Result` 或自定义错误类型
- 提供清晰的错误消息
- API 响应使用 `ok_response` 和 `error_response` 统一格式

### 测试编写
- 单元测试放在 `src/` 文件中（`#[cfg(test)]`）
- 集成测试放在 `tests/` 目录
- 使用 `tests/common/` 中的辅助函数
- 支持 Mock 和真实测试模式
- 测试应该可以在没有外部依赖的情况下运行（Mock 模式）

### 代码质量
- 使用 `cargo clippy` 检查代码
- 使用 `cargo fmt` 格式化代码
- 避免代码重复，提取公共函数
- 删除冗余注释，保留必要的文档注释

### API 设计
- 使用 OpenAPI (utoipa) 自动生成文档
- 统一响应格式：`{"status": "ok", "data": {...}}` 或 `{"status": "error", "message": "..."}`
- 使用 RESTful 风格
- 支持热重载配置（无需重启服务）

## 🚀 常用命令

### 编译
```bash
# 编译所有项目
cargo build --workspace

# 编译单个项目
cargo build --package llm-adapter
cargo build --package agentflow
cargo build --package nexus

# 发布版本
cargo build --workspace --release
```

### 运行
```bash
# 运行 Nexus
cd nexus && cargo run

# 或直接运行
cargo run --package nexus
```

### 测试
```bash
# 运行所有测试（Mock 模式）
cargo test --workspace

# 运行单个项目测试
cargo test --package nexus
cargo test --package llm-adapter
cargo test --package agentflow

# 使用测试脚本（支持真实模式）
cd nexus && ./scripts/test/run_tests.sh --real
```

### 代码检查
```bash
# 格式化代码
cargo fmt --all

# 检查代码
cargo clippy --workspace -- -D warnings

# 运行测试
cargo test --workspace
```

## 📝 重要注意事项

### 1. 模块独立性
- `llm-adapter` 和 `agentflow` 必须保持独立
- 不能有相互依赖
- 通过 `nexus/src/integration/` 桥接层集成
- 两个工具都可以独立打包和发布

### 2. 扩展性
- `agentflow` 通过 `LLMProvider` trait 支持任意 LLM 库
- `llm-adapter` 通过 `GenericAdapter` 支持自定义 API
- 用户可以实现自定义 Agent 和 LLM Provider
- 详见 `agentflow/EXTENDING.md`

### 3. 配置管理
- 所有配置支持热重载（无需重启）
- 配置存储在内存中，支持导入导出
- 适配器、提示模板、路由规则都可以在线修改

### 4. 多角色支持
- 支持 8 种角色类型：User, Assistant, Planner, Executor, Reviewer, Coordinator, Expert, Custom
- 每个角色有默认的 system prompt
- 可以根据角色自动路由消息
- 支持发言者选择策略（RoundRobin, Random, Auto 等）

### 5. 测试模式
- 默认使用 Mock 模式（不依赖外部服务）
- 真实测试需要配置环境变量
- 测试脚本会自动检测模式
- Mock 适配器用于快速测试

## 🔍 快速定位

### 查找适配器相关代码
- `llm-adapter/src/providers/` - 提供商实现
- `llm-adapter/src/registry.rs` - 注册表
- `llm-adapter/src/factory.rs` - 工厂
- `nexus/src/routes/handlers/config/adapters.rs` - 适配器 API
- `nexus/src/routes/handlers/adapter_helpers.rs` - 适配器辅助函数

### 查找 Agent 相关代码
- `agentflow/src/agent.rs` - Agent trait
- `agentflow/src/orchestrator.rs` - 编排器（多轮对话核心）
- `agentflow/src/llm_provider.rs` - LLM Provider trait
- `nexus/src/integration/llm_agent.rs` - LLM Agent 桥接实现
- `nexus/src/integration/llm_adapter_provider.rs` - LLM Adapter Provider 桥接
- `nexus/src/routes/handlers/agents.rs` - Agent API

### 查找配置相关代码
- `nexus/src/domain/config/` - 配置领域模型
- `nexus/src/routes/config/` - 配置 API 路由
- `nexus/src/routes/handlers/config/` - 配置处理逻辑

### 查找测试相关代码
- `nexus/tests/common/` - 测试辅助模块
- `nexus/tests/unit/` - 单元测试
- `nexus/tests/integration/` - 集成测试
- `nexus/tests/performance/` - 性能测试

## 📊 项目统计

- **llm-adapter**: ~2000 行代码，9 个依赖，可独立发布
- **agentflow**: ~1500 行代码，8 个依赖，可独立发布
- **nexus**: ~6000 行代码，30+ 个依赖，业务应用

## ✅ 质量保证

- ✅ 所有项目可独立编译
- ✅ 零代码重复
- ✅ 完整的测试覆盖
- ✅ 统一的代码风格
- ✅ 完整的文档
- ✅ 支持热重载配置
- ✅ 支持多角色多智能体对话
- ✅ 可扩展的架构设计

---

**最后更新**: 2025-01-XX  
**维护者**: Nexus Team
