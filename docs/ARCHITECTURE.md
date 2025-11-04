# Nexus 架构快速参考

## 核心设计原则

1. **三层架构**：Presentation → Application → Domain
2. **服务分组**：AppState 按功能分组，避免臃肿
3. **模块独立**：每个业务领域独立模块，互不干扰
4. **统一抽象**：Repository、Service 等通用模式
5. **依赖倒置**：Domain Layer 不依赖上层

## 架构层次

```
┌─────────────────────────────────────────────────────────────┐
│              Presentation Layer (routes/)                     │
│  • HTTP 路由定义                                              │
│  • OpenAPI 文档注解                                           │
│  • 调用 Application Layer                                     │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│            Application Layer (routes/handlers/)              │
│  • HTTP 请求处理                                              │
│  • 业务逻辑编排                                               │
│  • DTO ↔ Domain 转换                                         │
│  • 调用 Domain Layer 服务                                    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│            Domain Layer (domain/)                            │
│  • 领域模型定义                                               │
│  • 业务规则实现                                               │
│  • 领域服务                                                   │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│            Infrastructure (infrastructure/)                   │
│  • 缓存实现                                                   │
│  • 队列实现                                                   │
│  • 数据库访问                                                 │
│  • 外部服务集成                                               │
└─────────────────────────────────────────────────────────────┘
```

## 目录结构

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

## 添加新业务模块（3 步）

1. **Domain Layer**：`src/domain/<domain>/` - 领域模型和服务
2. **Application Layer**：`src/routes/handlers/<domain>/` - 处理逻辑
3. **Presentation Layer**：`src/routes/<domain>/` - API 路由

## 关键设计模式

- **Repository Pattern**：数据访问抽象
- **Service Pattern**：业务逻辑封装
- **DTO Pattern**：API 数据传输
- **Factory Pattern**：对象创建
- **Strategy Pattern**：路由策略

详细架构设计请参阅：[`ARCHITECTURE_DESIGN.md`](ARCHITECTURE_DESIGN.md)
