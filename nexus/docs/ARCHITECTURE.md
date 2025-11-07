# Nexus 架构

## 架构层次

```
Presentation Layer (routes/)     → HTTP API + OpenAPI
         ↓
Application Layer (handlers/)    → 业务处理逻辑
         ↓
Domain Layer (domain/)           → 领域模型和服务
         ↓
Infrastructure (infrastructure/) → 缓存、队列、消息总线
```

## 目录结构

```
src/
├── routes/          # Presentation Layer
│   ├── invoke.rs    # LLM 调用路由
│   ├── agents.rs    # 多智能体对话路由
│   ├── config/      # 配置管理路由
│   └── api_doc.rs   # OpenAPI 文档
├── routes/handlers/ # Application Layer
│   ├── invoke.rs    # LLM 调用处理
│   ├── agents.rs    # 多智能体对话处理
│   └── config/      # 配置管理处理
├── domain/          # Domain Layer
│   └── config/      # 配置领域模型
├── application/     # Application Services
│   ├── kb/          # 知识库
│   ├── planner.rs   # 任务规划器
│   ├── postprocessor.rs  # 后处理器链
│   └── prompt.rs    # 提示模板管理
├── infrastructure/  # Infrastructure Layer
│   ├── cache/       # 缓存（Redis/内存）
│   ├── queue/       # 任务队列
│   └── messaging/   # 消息总线（MCP）
└── integration/     # 桥接层
    ├── llm_agent.rs           # LLM Agent 实现
    └── llm_adapter_provider.rs # LLM Adapter Provider 桥接
```

## 核心组件

### 1. 状态管理 (`state.rs`)
- `AppState` - 应用全局状态
- 包含所有服务的引用（适配器注册表、配置管理器、缓存等）

### 2. 适配器集成
- `llm-adapter` 提供统一的 LLM 调用接口
- `AdapterRegistry` 管理所有适配器实例
- 支持热重载适配器配置

### 3. 多智能体集成
- `agentflow` 提供多智能体协作框架
- `LLMAgent` 桥接 llm-adapter 和 agentflow
- `LLMAdapterProvider` 将 Adapter 包装为 LLMProvider
- 支持多角色、发言者选择、终止条件

### 4. 配置管理
- `ConfigManager` 管理所有配置（适配器、路由、提示、功能开关）
- 支持热重载（无需重启）
- 支持导入导出

### 5. 应用服务
- **知识库** (`kb/`) - 关键词检索
- **规划器** (`planner.rs`) - 任务拆分
- **后处理器** (`postprocessor.rs`) - 审计、PII 脱敏、格式化
- **提示模板** (`prompt.rs`) - Handlebars 模板渲染

## 数据流

### LLM 调用流程
```
HTTP Request → invoke_handler → 
  1. 路由规则选择适配器
  2. 知识库检索（如果启用）
  3. 提示模板渲染
  4. 后处理链预处理
  5. 调用适配器
  6. 后处理链后处理
  7. 返回结果
```

### 多智能体对话流程
```
HTTP Request → agents_handler →
  1. 创建 AgentOrchestrator
  2. 注册多个 LLMAgent（不同角色）
  3. 执行多轮对话
  4. 根据发言者选择策略选择 Agent
  5. 检查终止条件
  6. 返回对话结果
```

## 依赖关系

```
nexus
├── llm-adapter (独立工具)
│   └── 提供 Adapter trait 和 AdapterRegistry
├── agentflow (独立工具)
│   └── 提供 Agent trait 和 AgentOrchestrator
└── 桥接层 (integration/)
    └── 连接两个独立工具
```

## 扩展点

### 添加新的 LLM 提供商
1. 在 `llm-adapter/src/providers/` 实现 `Adapter` trait
2. 在 `AdapterFactory` 中注册

### 添加新的 Agent 角色
1. 在 `agentflow/src/agent.rs` 添加 `AgentRole` 枚举值
2. 在 `LLMAgent::build_prompt` 中添加角色特定的 prompt 构建逻辑
3. 在 `LLMAgent::can_handle` 中添加角色特定的消息处理逻辑

### 添加新的应用服务
1. 在 `src/application/` 创建新模块
2. 在 `AppState` 中添加服务实例
3. 在相应的 handler 中使用

## 添加新业务模块

1. **Domain Layer**: `src/domain/<domain>/`
2. **Application Layer**: `src/routes/handlers/<domain>/`
3. **Presentation Layer**: `src/routes/<domain>/`
