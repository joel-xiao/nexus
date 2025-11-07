# Nexus 文档索引

## 📚 快速导航

### 🚀 开始使用
- **[QUICKSTART.md](QUICKSTART.md)** - 5分钟快速开始

### 🏗️ 架构文档
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - 架构快速参考
- **[ARCHITECTURE_DESIGN.md](ARCHITECTURE_DESIGN.md)** - 详细架构设计

### 🧪 测试文档
- **[../tests/README.md](../tests/README.md)** - 测试指南
- **[TESTS_DESIGN.md](TESTS_DESIGN.md)** - 测试设计

### 🚢 部署文档
- **[DEPLOYMENT.md](DEPLOYMENT.md)** - 部署指南
- **[DEPLOY_DESIGN.md](DEPLOY_DESIGN.md)** - 部署设计

### 🌐 API 文档
- **[FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md)** - 前端 API 指南

---

## 🔗 外部工具文档

Nexus 使用了两个独立工具：

- **[llm-adapter](../../llm-adapter/README.md)** - LLM 适配器框架
- **[AgentFlow](../../agentflow/README.md)** - 多代理协作框架

---

## 📋 文档说明

### 架构变更（重要！）

Nexus 已重构为使用两个独立工具：
1. **llm-adapter** - 替代了原 `domain/adapters` 和 `infrastructure/adapter`
2. **AgentFlow** - 提供多代理协作功能（新增）

集成方式见 `src/integration/llm_agent.rs`

---

## 🎯 快速查找

- **想快速启动？** → [QUICKSTART.md](QUICKSTART.md)
- **想了解架构？** → [ARCHITECTURE.md](ARCHITECTURE.md)
- **想添加功能？** → [ARCHITECTURE_DESIGN.md](ARCHITECTURE_DESIGN.md)
- **想部署项目？** → [DEPLOYMENT.md](DEPLOYMENT.md)
- **想调用 API？** → [FRONTEND_API_GUIDE.md](FRONTEND_API_GUIDE.md)
