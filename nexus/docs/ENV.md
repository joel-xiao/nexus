# 环境变量配置

## 必需环境变量

无（所有环境变量都是可选的）

## 可选环境变量

### 服务配置

- `PORT`: 服务监听端口（默认: 3000）
- `RUST_LOG`: 日志级别（默认: info）
  - 可选值: `trace`, `debug`, `info`, `warn`, `error`
- `LOG_FORMAT`: 日志格式（默认: json）
  - 可选值: `json`, `pretty`

### 外部服务

- `REDIS_URL`: Redis 连接 URL（可选）
  - 格式: `redis://host:port` 或 `redis://user:password@host:port`
  - 示例: `redis://localhost:6379` 或 `redis://redis:6379`
  - 如果不设置，将使用内存缓存（不持久化）

- `JAEGER_ENDPOINT`: Jaeger/OTLP 追踪端点（可选）
  - 格式: `http://host:port`
  - 示例: `http://jaeger:4317`
  - 如果不设置，将不启用分布式追踪

### 示例

```bash
# 开发环境
export PORT=3000
export RUST_LOG=debug
export LOG_FORMAT=pretty
export REDIS_URL=redis://localhost:6379

# 生产环境
export PORT=3000
export RUST_LOG=info
export LOG_FORMAT=json
export REDIS_URL=redis://redis:6379
export JAEGER_ENDPOINT=http://jaeger:4317
```

## Docker Compose 环境变量

在 `docker-compose.yml` 中设置：

```yaml
environment:
  - PORT=3000
  - RUST_LOG=info
  - LOG_FORMAT=json
  - REDIS_URL=redis://redis:6379
  - JAEGER_ENDPOINT=http://jaeger:4317
```

## .env 文件

创建 `.env` 文件（不会被提交到 Git）：

```bash
PORT=3000
RUST_LOG=info
LOG_FORMAT=json
REDIS_URL=redis://localhost:6379
```

然后使用：
```bash
docker-compose --env-file .env up
```

