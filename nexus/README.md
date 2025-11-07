# Nexus - LLM API Gateway

生产就绪的多模型 LLM 统一网关，集成 llm-adapter 和 agentflow。

## 🚀 快速开始

```bash
# 编译
cargo build --release

# 运行
./target/release/nexus

# 或开发模式
cargo run
```

服务将在 `http://localhost:3000` 启动。

## 📚 文档

- [快速启动](./docs/QUICKSTART.md) - 快速上手指南
- [部署指南](./docs/DEPLOYMENT.md) - 生产环境部署
- [环境变量](./docs/ENV.md) - 环境变量配置
- [架构说明](./docs/ARCHITECTURE.md) - 项目架构
- [前端 API 指南](./docs/FRONTEND_API_GUIDE.md) - 前端开发指南
- [API 文档](http://localhost:3000/docs) - Swagger UI（运行后访问）

## 🎯 核心功能

### 1. LLM 调用
- 统一调用接口 `/api/invoke`
- 支持路由规则自动选择模型
- 支持提示模板
- 支持知识库检索
- 支持后处理链

### 2. 多智能体对话
- `/api/agents/conversation` - 启动多角色对话
- `/api/agents/orchestrate` - 编排多个 Agent
- 支持 8 种角色类型
- 支持发言者选择策略
- 支持终止条件

### 3. 配置管理
- 适配器管理（CRUD、统计、按模型查询）
- 提示模板管理
- 路由规则管理
- 功能开关管理
- 配置热重载
- 配置导入导出

## 🔌 API 端点

### 核心 API
- `POST /api/invoke` - 调用 LLM
- `POST /api/agents/conversation` - 多角色对话
- `POST /api/agents/orchestrate` - 编排 Agent
- `GET /api/agents` - 列出 Agent

### 配置 API
- `GET /api/config/adapters` - 适配器管理
- `GET /api/config/prompts` - 提示模板管理
- `GET /api/config/flags` - 功能开关管理
- `GET /api/config/routing` - 路由规则管理
- `POST /api/config/reload/*` - 热重载
- `POST /api/config/import-export` - 导入导出

完整 API 文档：运行服务后访问 `http://localhost:3000/docs`

## ⚙️ 配置

### 环境变量
- `PORT` - 服务端口（默认: 3000）
- `RUST_LOG` - 日志级别（默认: info）
- `REDIS_URL` - Redis 连接（可选）
- `JAEGER_ENDPOINT` - 追踪端点（可选）

详见 [环境变量文档](./docs/ENV.md)

### 配置文件
配置文件示例：`config/config.example.json`

可以通过 API 热加载配置，无需重启服务。

## 🧪 测试

```bash
# 运行所有测试（Mock 模式）
cargo test

# 使用测试脚本（支持真实模式）
./scripts/test/run_tests.sh --real
```

## 📦 部署

### Docker Compose
```bash
docker-compose up -d
```

### Kubernetes
详见 [部署指南](./docs/DEPLOYMENT.md)

## 📝 许可证

MIT
