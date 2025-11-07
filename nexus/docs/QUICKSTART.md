# Nexus 快速开始指南

## 🚀 快速启动

### 方法 1: 直接运行（推荐用于开发测试）

```bash
# 1. 编译并运行
cargo run

# 或者后台运行
cargo run &
```

服务将在 `http://localhost:3000` 启动。

### 方法 2: 使用启动脚本

```bash
./scripts/run/start.sh
```

### 方法 3: 使用 Docker Compose

```bash
docker-compose up -d
```

## 📝 配置大模型 API Key

### 方式 1: 通过 API 热加载配置（推荐）

服务启动后，默认已经有一个 `mock` 适配器（用于测试）。要使用真实的大模型，需要通过 API 添加配置：

#### 添加 OpenAI 适配器

```bash
curl -X PUT http://localhost:3000/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "openai",
    "api_key": "sk-your-openai-api-key-here",
    "model": "gpt-3.5-turbo",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }'
```

#### 添加 DeepSeek 适配器

```bash
curl -X PUT http://localhost:3000/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "deepseek",
    "api_key": "your-deepseek-api-key",
    "model": "deepseek-chat",
    "base_url": "https://api.deepseek.com",
    "enabled": true
  }'
```

#### 添加豆包（Doubao）适配器

```bash
curl -X PUT http://localhost:3000/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "doubao",
    "api_key": "your-doubao-api-key",
    "model": "doubao-pro-4k",
    "base_url": "https://ark.cn-beijing.volces.com/api/v3",
    "enabled": true
  }'
```

### 方式 2: 导入配置文件

```bash
# 1. 编辑配置文件（复制示例文件）
cp config/examples/adapter_config_example.json config/adapters.json

# 2. 编辑 config/adapters.json，填入你的 API keys

# 3. 通过 API 导入配置
curl -X POST http://localhost:3000/api/config/import \
  -H "Content-Type: application/json" \
  -d @config/adapters.json
```

## 🎯 体验大模型

### 1. 使用 Mock 适配器（无需 API Key）

```bash
curl -X POST http://localhost:3000/api/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "input": "你好，介绍一下你自己",
    "adapter": "mock"
  }'
```

### 2. 调用真实的大模型

```bash
# 调用 OpenAI
curl -X POST http://localhost:3000/api/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "input": "请用中文解释什么是 Rust 编程语言",
    "adapter": "openai"
  }'
```

### 3. 使用路由策略（自动选择模型）

```bash
curl -X POST http://localhost:3000/api/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "input": "写一首关于春天的诗",
    "user_id": "user123"
  }'
```

## 📊 查看 API 文档

启动服务后，访问 Swagger UI：
```
http://localhost:3000/docs
```

这里可以：
- 查看所有 API 接口
- 在线测试 API
- 查看请求/响应示例

## 🔍 常用操作

### 查看已注册的适配器

```bash
curl http://localhost:3000/api/config/adapters
```

### 查看健康状态

```bash
curl http://localhost:3000/health
```

### 查看就绪状态

```bash
curl http://localhost:3000/ready
```

### 查看 Prometheus 指标

```bash
curl http://localhost:3000/metrics
```

## 🌐 完整的测试示例

### 1. 启动服务

```bash
cargo run
```

### 2. 添加适配器配置

```bash
# 替换 YOUR_API_KEY 为你的真实 API Key
curl -X PUT http://localhost:3000/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "openai",
    "api_key": "YOUR_API_KEY",
    "model": "gpt-3.5-turbo",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }'
```

### 3. 调用大模型

```bash
curl -X POST http://localhost:3000/api/invoke \
  -H "Content-Type: application/json" \
  -d '{
    "input": "用一句话介绍人工智能",
    "adapter": "openai"
  }'
```

### 4. 查看响应

成功的话，你会收到类似这样的响应：

```json
{
  "result": "人工智能是模拟人类智能的计算机系统...",
  "tasks": [],
  "adapter_used": "openai"
}
```

## 💡 提示

1. **Mock 适配器**：默认已经注册，无需配置即可测试 API 功能
2. **API Key 安全**：不要将 API Key 提交到代码仓库
3. **日志查看**：日志会输出到 `nexus.log` 文件和控制台
4. **端口修改**：默认端口是 3000，可以在代码中修改（`src/main.rs`）

## ❓ 故障排查

### 服务无法启动

- 检查端口 3000 是否被占用：`lsof -i :3000`
- 检查 Rust 环境：`rustc --version`

### API 调用失败

- 检查适配器是否已注册：`curl http://localhost:3000/api/config/adapters`
- 查看服务日志：`tail -f nexus.log`
- 检查 API Key 是否正确

### 适配器注册失败

- 确保 API Key 格式正确
- 检查 `base_url` 是否正确
- 查看日志了解详细错误信息
