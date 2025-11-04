# Nexus

一个高性能的 LLM（大语言模型）代理/网关系统，提供统一的 API 接口来调用多种 LLM 提供商。

## 核心特性

- 🎯 **统一接口**：为多种 LLM 提供商提供统一的调用接口
- 🚀 **智能路由**：支持多种路由策略（轮询、随机、加权、基于用户等）
- ⚙️ **配置管理**：功能标志、路由规则、适配器配置的运行时管理
- 📊 **监控可观测**：日志、指标、审计、追踪
- 🔄 **异步处理**：任务队列、并发控制
- 💾 **缓存支持**：会话缓存、嵌入缓存（Redis + 内存）

## 🚀 快速开始

### 启动服务

```bash
# 使用脚本启动
./start.sh

# 或使用 Docker Compose
docker-compose up -d
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 或使用测试脚本
./run_tests.sh
```

详细测试文档请参阅：[`tests/README.md`](tests/README.md)

## 📚 文档

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - 架构快速参考
- [`docs/ARCHITECTURE_DESIGN.md`](docs/ARCHITECTURE_DESIGN.md) - 详细架构设计
- [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) - 部署指南
- [`tests/README.md`](tests/README.md) - 测试文档

## 📁 项目结构

```
.
├── src/                    # 源代码
│   ├── routes/            # Presentation Layer（路由 + OpenAPI）
│   ├── routes/handlers/   # Application Layer（业务逻辑）
│   ├── domain/            # Domain Layer（领域模型）
│   └── infrastructure/    # Infrastructure Layer（基础设施）
├── tests/                 # 测试文件（按架构分层）
├── scripts/               # 脚本文件
├── config/                # 配置文件
├── docs/                  # 文档
└── deploy/                # 部署配置
```

## 技术栈

- **Web 框架**：Axum
- **OpenAPI**：utoipa
- **异步运行时**：Tokio
- **缓存**：Redis + 内存缓存

## 许可证

[许可证信息]
