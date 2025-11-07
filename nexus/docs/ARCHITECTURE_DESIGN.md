# Nexus 架构设计文档

## 📋 文档导航

- **[快速开始](#快速开始)** - 5 分钟了解架构
- **[系统概述](#1-系统概述)** - 项目定位和技术栈
- **[架构层次](#2-架构层次)** - 详细的三层架构说明
- **[模块职责](#3-模块职责详解)** - 各模块详细说明
- **[核心数据流](#4-核心数据流)** - 典型业务流程
- **[扩展指南](#5-业务功能扩展指南)** - 添加新业务的完整指南

---

## 快速开始

### 核心架构（30 秒）

```
Presentation Layer (routes/)     → HTTP API + OpenAPI
         ↓
Application Layer (handlers/)    → 业务处理逻辑
         ↓
Domain Layer (domain/)           → 领域模型和服务
         ↓
Infrastructure (infrastructure/) → 缓存、队列、数据库
```

### 添加新业务模块（3 步）

1. **Domain Layer**：`src/domain/<domain>/` - 领域模型和服务
2. **Application Layer**：`src/routes/handlers/<domain>/` - 处理逻辑
3. **Presentation Layer**：`src/routes/<domain>/` - API 路由

详细步骤见：[扩展指南](#5-业务功能扩展指南)

---

## 1. 系统概述

### 1.1 项目定位
Nexus 是一个 **LLM（大语言模型）代理/网关系统**，提供以下核心能力：
- 🎯 **统一接口**：为多种 LLM 提供商（OpenAI、DeepSeek、豆包、智谱等）提供统一的调用接口
- 🚀 **智能路由**：支持多种路由策略（轮询、随机、加权、基于用户等）
- ⚙️ **配置管理**：功能标志、路由规则、适配器配置的运行时管理
- 📊 **监控可观测**：日志、指标、审计、追踪
- 🔄 **异步处理**：任务队列、并发控制
- 💾 **缓存支持**：会话缓存、嵌入缓存（Redis + 内存）
- 🔧 **可扩展性**：模块化设计，易于添加新的适配器和功能

### 1.2 技术栈
- **Web 框架**：Axum
- **OpenAPI**：utoipa
- **异步运行时**：Tokio
- **序列化**：serde
- **配置管理**：serde_json
- **缓存**：Redis + 内存缓存
- **监控**：tracing、prometheus

---

## 2. 架构层次

### 2.1 三层架构

#### Presentation Layer (routes/)
**职责**：
- HTTP 路由定义
- OpenAPI 文档注解
- 请求验证和响应格式化
- 调用 Application Layer

**位置**：`src/routes/`

**特点**：
- 路由和 OpenAPI 注解在一起
- 业务逻辑在 handlers 中
- 使用 DTO（Data Transfer Object）进行数据传输

#### Application Layer (routes/handlers/)
**职责**：
- HTTP 请求处理
- 业务逻辑编排
- DTO ↔ Domain 转换
- 调用 Domain Layer 服务

**位置**：`src/routes/handlers/`

**特点**：
- 纯业务逻辑，不包含路由定义
- 通过 `AppState` 访问 Domain 服务
- 处理事务和错误

#### Domain Layer (domain/)
**职责**：
- 领域模型定义
- 业务规则实现
- 领域服务
- 不依赖框架和基础设施

**位置**：`src/domain/`

**特点**：
- 核心业务逻辑
- 可独立测试
- 通过 trait 定义接口

#### Infrastructure Layer (infrastructure/)
**职责**：
- 缓存实现
- 队列实现
- 数据库访问
- 外部服务集成

**位置**：`src/infrastructure/`

**特点**：
- 技术实现细节
- 可替换实现
- 通过 trait 为 Domain 提供服务

---

## 3. 模块职责详解

### 3.1 Domain Layer

#### domain/config/
- **ConfigManager**：配置管理器
- **FeatureFlagStore**：功能标志存储
- **ModelRouter**：模型路由器
- **RoutingRule**：路由规则定义

**注意**：LLM 适配器功能现在由独立的 `llm-adapter` crate 提供

### 3.2 Infrastructure Layer

#### infrastructure/cache/
- **RedisCache**：Redis 缓存实现
- **SessionCache**：会话缓存
- **EmbeddingCache**：嵌入缓存

#### infrastructure/queue/
- **TaskQueue**：任务队列
- **QueueManager**：队列管理器
- **Worker**：任务处理工作器

#### infrastructure/messaging/
- **MCP Message Bus**：MCP 消息总线

**注意**：限流、计费、并发控制现在由 `llm-adapter` crate 的 wrapper 提供

### 3.3 Integration Layer（新增）

#### integration/llm_agent.rs
- **LLMAgent**：桥接 llm-adapter 的 Adapter 和 AgentFlow 的 Agent
- 展示如何集成两个独立工具

### 3.4 Application Layer

#### application/postprocessor/
- **PostprocessorChain**：后处理链
- **ProcessingContext**：处理上下文

#### application/prompt/
- **PromptStore**：提示存储和管理

#### application/kb/
- **KnowledgeBase**：知识库

#### application/planner/
- **Planner**：任务规划器

### 3.5 Presentation Layer

#### routes/
- **config/**：配置管理 API
- **health.rs**：健康检查
- **invoke.rs**：LLM 调用
- **api_doc.rs**：OpenAPI 文档组合

#### routes/handlers/
- **config/**：配置处理逻辑
- **invoke.rs**：调用处理逻辑

---

## 4. 核心数据流

### 4.1 LLM 调用流程

```
1. Client → HTTP Request (/api/invoke)
   ↓
2. routes/invoke.rs → 路由定义 + OpenAPI 注解
   ↓
3. handlers/invoke.rs → 业务处理逻辑
   ↓
4. domain/adapters/ → 选择适配器
   ↓
5. infrastructure/adapter/ → 速率限制、计费跟踪
   ↓
6. domain/adapters/implementations/ → 具体适配器实现
   ↓
7. External LLM API
   ↓
8. Response 返回（经过后处理链）
```

### 4.2 配置管理流程

```
1. Client → HTTP Request (/api/config/*)
   ↓
2. routes/config/*.rs → 路由定义
   ↓
3. handlers/config/*.rs → 处理逻辑
   ↓
4. domain/config/* → 配置管理服务
   ↓
5. 更新配置（内存 + 持久化）
```

---

## 5. 业务功能扩展指南

### 5.1 添加新业务领域（例如：用户管理）

#### 步骤 1：Domain Layer

创建 `src/domain/user/`：

```rust
// src/domain/user/mod.rs
pub mod model;
pub mod service;
pub mod repository;

// src/domain/user/model.rs
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
}

// src/domain/user/service.rs
pub struct UserService {
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn create_user(&self, email: String, name: String) -> Result<User> {
        // 业务逻辑
    }
}

// src/domain/user/repository.rs
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<()>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>>;
}
```

#### 步骤 2：Infrastructure Layer（如果需要）

创建 `src/infrastructure/user/`：

```rust
// src/infrastructure/user/repository.rs
pub struct UserRepositoryImpl {
    // 数据库连接等
}

#[async_trait]
impl domain::user::repository::UserRepository for UserRepositoryImpl {
    // 实现具体逻辑
}
```

#### 步骤 3：Application Layer

创建 `src/routes/handlers/user/`：

```rust
// src/routes/handlers/user/create.rs
pub async fn create_user(
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>> {
    let user = state.user_service.create_user(request.email, request.name).await?;
    Ok(Json(UserResponse::from(user)))
}
```

#### 步骤 4：Presentation Layer

创建 `src/routes/user/`：

```rust
// src/routes/user/mod.rs
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(create_user),
    components(schemas(UserResponse, CreateUserRequest)),
    tags((name = "user", description = "用户管理 API"))
)]
pub struct UserApiDoc;

pub fn user_routes() -> Router {
    Router::new()
        .route("/users", post(handlers::user::create::create_user))
}

// src/routes/user/create.rs
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "创建成功", body = UserResponse)
    ),
    tag = "user"
)]
pub async fn create_user(/* ... */) {
    handlers::user::create::create_user(/* ... */).await
}
```

#### 步骤 5：注册路由

在 `src/routes/mod.rs` 中：

```rust
pub fn app_routes(state: AppState) -> Router {
    Router::new()
        .merge(config_routes())
        .merge(user::user_routes()) // 新增
        .layer(Extension(state))
}
```

在 `src/routes/api_doc.rs` 中：

```rust
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    // ...
    tags((name = "config", description = "配置管理 API")),
    tags((name = "user", description = "用户管理 API")), // 新增
)]
pub struct ApiDoc;

impl OpenApi for ApiDoc {
    fn openapi() -> utoipa::openapi::OpenApi {
        let mut openapi = <ApiDoc as utoipa::OpenApi>::openapi();
        openapi.merge(config::FlagsApiDoc::openapi());
        openapi.merge(user::UserApiDoc::openapi()); // 新增
        openapi
    }
}
```

### 5.2 目录结构示例

完整的新业务模块结构：

```
src/
├── domain/
│   └── user/              # ✨ 新增
│       ├── mod.rs
│       ├── model.rs
│       ├── service.rs
│       └── repository.rs
│
├── infrastructure/
│   └── user/              # ✨ 新增（如果需要）
│       └── repository.rs
│
├── routes/
│   ├── user/              # ✨ 新增
│   │   ├── mod.rs
│   │   └── create.rs
│   │
│   └── handlers/
│       └── user/          # ✨ 新增
│           └── create.rs
```

---

## 6. 最佳实践

### 6.1 路由和 OpenAPI

- ✅ 路由定义和 OpenAPI 注解在 `routes/` 中
- ✅ 业务逻辑在 `routes/handlers/` 中
- ✅ 使用 DTO 进行 API 数据传输
- ✅ 每个业务模块有独立的 OpenAPI 文档结构

### 6.2 Domain Layer

- ✅ 定义 trait 接口，不依赖具体实现
- ✅ 领域模型独立于框架
- ✅ 业务规则在 Domain 层实现

### 6.3 Infrastructure Layer

- ✅ 实现 Domain 层定义的 trait
- ✅ 技术细节隔离在 Infrastructure 层
- ✅ 可替换实现（例如：内存实现 → 数据库实现）

### 6.4 错误处理

- ✅ 使用统一的错误类型
- ✅ Domain 层返回领域错误
- ✅ Presentation 层转换为 HTTP 错误

### 6.5 测试

- ✅ 单元测试：测试 Domain 层逻辑
- ✅ 集成测试：测试 API 端点
- ✅ 使用 Mock 隔离外部依赖

---

## 7. 目录结构总结

```
src/
├── lib.rs                    # 库入口
├── main.rs                   # 二进制入口
├── state.rs                  # 应用状态管理
│
├── routes/                   # Presentation Layer
│   ├── mod.rs               # 路由组装
│   ├── api_doc.rs           # OpenAPI 文档组合
│   ├── common.rs            # 通用响应辅助函数
│   ├── config/              # 配置管理 API
│   ├── health.rs            # 健康检查
│   └── invoke.rs            # LLM 调用
│
├── routes/handlers/          # Application Layer
│   ├── config/              # 配置处理逻辑
│   └── invoke.rs            # 调用处理逻辑
│
├── domain/                   # Domain Layer
│   ├── adapters/           # LLM 适配器领域
│   └── config/             # 配置管理领域
│
└── infrastructure/           # Infrastructure Layer
    ├── adapter/            # 适配器基础设施
    ├── cache/              # 缓存
    ├── queue/              # 队列
    └── messaging/          # 消息总线
```

---

详细架构快速参考请参阅：[`ARCHITECTURE.md`](ARCHITECTURE.md)
