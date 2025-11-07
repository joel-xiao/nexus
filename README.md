# Nexus Workspace

包含两个可独立发布的 Rust 工具的工作空间。

## 📦 项目组成

### 🎁 llm-adapter - LLM 适配器框架

统一的 LLM API 适配器框架，支持多种提供商。

**特性**:
- 🎯 统一接口（OpenAI、DeepSeek、Doubao、千问、智谱等）
- 🚀 限流、计费、并发控制
- ⚙️ 配置化创建
- 🔌 易于扩展

📖 [完整文档](./llm-adapter/README.md)

### 🎁 agentflow - 多代理协作框架

独立的轻量级多代理协作和工作流编排框架。

**特性**:
- 🤖 多代理定义和管理
- 🔄 对话编排
- 📊 工作流引擎
- 💬 消息协议和上下文管理
- 🔌 纯抽象，不绑定任何 LLM 实现

📖 [完整文档](./agentflow/README.md)

### 🏗️ nexus - 业务应用示例

展示如何集成上述两个工具的完整示例。

📖 [完整文档](./nexus/README.md)

---

## 🚀 快速开始

### 编译所有项目

```bash
cargo build --workspace
```

### 独立编译工具

```bash
cd llm-adapter && cargo build
cd agentflow && cargo build
```

### 运行 Nexus

```bash
cd nexus && cargo run
# 访问 http://localhost:3000
```

### 运行测试

```bash
# 工作空间整体测试
./scripts/run_tests.sh

# 单独测试 llm-adapter / AgentFlow / Nexus
./llm-adapter/scripts/test/run_tests.sh
./agentflow/scripts/test/run_tests.sh
./nexus/scripts/test/run_tests.sh

# 附加性能测试或覆盖率
./nexus/scripts/test/run_tests.sh --performance
```

---

## 📖 文档导航

- [项目总结](./PROJECT_SUMMARY.md) - 详细的项目说明
- [项目状态](./PROJECT_STATUS.md) - 当前状态和检查清单
- [llm-adapter 文档](./llm-adapter/README.md)
- [agentflow 文档](./agentflow/README.md)
- [Nexus 文档](./nexus/docs/)

---

## 🏗️ 架构

```
nexus-workspace/
├── llm-adapter/      # 🎁 独立工具1
├── agentflow/          # 🎁 独立工具2
└── nexus/            # 业务应用（集成示例）
```

**依赖关系**:
```
nexus → llm-adapter (独立)
nexus → agentflow (独立)

llm-adapter ⊥ agentflow (互不依赖)
```

---

## ✅ 验证状态

- ✅ 两个工具可独立编译
- ✅ 两个工具互不依赖
- ✅ 代码零重复
- ✅ 文档完整

---

## 📝 发布

### 发布 llm-adapter

```bash
cd llm-adapter
cargo publish
```

### 发布 agentflow

```bash
cd agentflow
cargo publish
```

---

## License

MIT
