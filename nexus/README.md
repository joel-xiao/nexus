# Nexus

一个高性能的 LLM（大语言模型）网关系统，集成了 llm-adapter 和 AgentFlow 两个独立工具。

## 核心特性

- 🎯 **统一接口** - 为多种 LLM 提供商提供统一的调用接口（基于 llm-adapter）
- 🤖 **多代理协作** - 支持多代理编排和工作流（基于 AgentFlow）
- 🚀 **智能路由** - 多种路由策略（轮询、随机、加权等）
- ⚙️ **配置管理** - 运行时配置管理
- 📊 **监控可观测** - 日志、指标、审计、追踪
- 💾 **缓存支持** - Redis + 内存缓存

## 🚀 快速开始

### 启动服务

```bash
# 方法 1: 直接运行
cargo run

# 方法 2: 使用 Docker Compose
docker-compose up -d
```

服务将在 `http://localhost:3000` 启动。

### 配置 API Key

```bash
curl -X PUT http://localhost:3000/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "openai",
    "api_key": "sk-your-key",
    "model": "gpt-3.5-turbo",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }'
```

### 调用模型

```bash
curl -X POST http://localhost:3000/api/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "input": "你好，介绍一下你自己",
    "adapter": "openai"
  }'
```

## 📖 文档

- [快速开始](./docs/QUICKSTART.md)
- [架构设计](./docs/ARCHITECTURE.md)
- [API 文档](./docs/FRONTEND_API_GUIDE.md)
- [部署指南](./docs/DEPLOYMENT.md)
- [测试文档](./tests/README.md)

## 🏗️ 架构

Nexus 通过集成层使用两个独立工具：

```
Nexus
 ├─→ llm-adapter（LLM 调用）
 └─→ AgentFlow（多代理协作）

集成层: src/integration/llm_agent.rs
```

## 🔧 技术栈

- **Web 框架**: Axum
- **LLM 适配器**: llm-adapter
- **多代理框架**: AgentFlow
- **异步运行时**: Tokio
- **缓存**: Redis
- **监控**: Prometheus, Tracing

## 📝 API 文档

访问 Swagger UI：`http://localhost:3000/docs`

## License

MIT

