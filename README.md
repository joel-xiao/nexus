# Nexus Workspace

包含三个可独立发布的 Rust 工具的工作空间。

## 📦 项目组成

- **llm-adapter**: LLM 适配器框架（可独立发布）
- **agentflow**: 多代理协作框架（可独立发布）
- **nexus**: 业务应用（集成示例）

## 🚀 快速开始

```bash
# 编译所有项目
cargo build --workspace

# 运行 Nexus 应用
cd nexus && cargo run

# 或直接运行
cargo run --package nexus
```

## 📚 文档

- [AI 使用指南](./AI_USAGE_GUIDE.md) - **给 AI 看的完整项目指南**
- [llm-adapter](./llm-adapter/README.md) - LLM 适配器文档
- [agentflow](./agentflow/README.md) - 多代理协作框架文档
- [agentflow 扩展指南](./agentflow/EXTENDING.md) - 如何扩展 AgentFlow
- [nexus](./nexus/README.md) - Nexus 应用文档

## 🎯 核心功能

### llm-adapter
- 统一的多 LLM 提供商适配器
- 支持 OpenAI, DeepSeek, Qwen, Zhipu, Doubao 等
- 限流、计费、并发控制
- 可扩展的架构设计

### agentflow
- 多智能体协作框架
- 支持多种角色（Planner, Executor, Reviewer 等）
- 发言者选择策略（RoundRobin, Random, Auto）
- 可扩展的 LLM Provider 抽象

### nexus
- HTTP API 网关
- LLM 调用接口
- 多智能体对话
- 配置管理（适配器、路由、提示模板、功能开关）
- 热重载配置

## 🔧 开发

```bash
# 格式化代码
cargo fmt --all

# 检查代码
cargo clippy --workspace

# 运行测试
cargo test --workspace
```

## 📝 许可证

MIT
