# Nexus 部署指南

## 部署方式

### 1. Docker Compose（开发/测试环境）

```bash
# 启动所有服务
docker-compose up -d

# 启动 Adapter Sidecar（可选）
docker-compose --profile adapter up -d adapter-sidecar

# 查看日志
docker-compose logs -f nexus

# 停止服务
docker-compose down
```

### 2. Kubernetes（生产环境）

#### 快速部署

```bash
# 创建命名空间和配置
kubectl apply -f deploy/k8s/namespace.yaml
kubectl apply -f deploy/k8s/configmap.yaml

# 部署 Redis
kubectl apply -f deploy/k8s/redis-deployment.yaml

# 部署 Nexus
kubectl apply -f deploy/k8s/deployment.yaml

# 部署 Ingress（可选）
kubectl apply -f deploy/k8s/ingress.yaml

# 部署 HPA（自动扩缩容）
kubectl apply -f deploy/k8s/hpa.yaml
```

#### 使用部署脚本

```bash
# 完整部署
./deploy/deploy.sh

# 回滚
./deploy/rollback.sh

# 金丝雀部署
./deploy/canary-deploy.sh v1.2.0 10  # 10% 流量
```

## 服务分离

### Adapter Sidecar 部署

Adapter 可以作为独立的 Sidecar 或独立服务部署：

```bash
# 部署 Adapter Sidecar
kubectl apply -f deploy/k8s/adapter-sidecar.yaml
```

### 微服务架构

```
┌─────────────────┐
│  Nexus          │ 主服务（Orchestrator）
│  (Orchestrator) │
└────────┬────────┘
         │
    ┌────┴────┬─────────────────┐
    │         │                 │
┌───▼───┐ ┌──▼────┐      ┌──────▼─────┐
│Redis  │ │Task   │      │Adapter     │
│Cache  │ │Queue  │      │Sidecar     │
└───────┘ └───────┘      └────────────┘
```

## 升级/回滚策略

### 1. 蓝绿部署（零停机）

使用 Argo Rollouts：

```bash
# 应用 Rollout 配置
kubectl apply -f deploy/k8s/rollout-strategy.yaml

# 触发新版本部署
kubectl argo rollouts set image nexus-rollout nexus=nexus:v1.2.0 -n nexus

# 手动批准升级
kubectl argo rollouts promote nexus-rollout -n nexus
```

### 2. 金丝雀发布（灰度发布）

结合 Feature Flag：

```bash
# 第一步：10% 流量
./deploy/canary-deploy.sh v1.2.0 10

# 第二步：50% 流量
./deploy/increase-canary.sh v1.2.0 50

# 第三步：100% 流量（完全切换）
./deploy/promote-canary.sh v1.2.0
```

### 3. 基于 Prometheus 告警的自动回滚

配置了以下告警规则：
- 高错误率告警
- 高延迟告警
- 服务宕机告警

当告警触发时，可以自动回滚：

```bash
# 监控告警并自动回滚
kubectl apply -f deploy/k8s/prometheus-alerts.yaml
```

## 健康检查

### 存活探针（Liveness）
- 路径: `/api/health`
- 间隔: 30秒
- 超时: 3秒

### 就绪探针（Readiness）
- 路径: `/api/ready`
- 检查项：
  - Redis 连接状态
  - 适配器注册状态
  - 任务队列状态

## 监控

### Prometheus

```bash
# 访问 Prometheus UI
http://localhost:9090

# 查看指标
curl http://nexus:3000/metrics
```

### Grafana

```bash
# 访问 Grafana
http://localhost:3001
# 用户名/密码: admin/admin
```

### Jaeger

```bash
# 访问 Jaeger UI
http://localhost:16686
```

## 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `REDIS_URL` | Redis 连接地址 | - |
| `JAEGER_ENDPOINT` | Jaeger OTLP 端点 | - |
| `LOG_FORMAT` | 日志格式 (json/pretty) | json |
| `RUST_LOG` | 日志级别 | info |

## 扩缩容

HPA 配置：
- 最小副本数: 3
- 最大副本数: 10
- CPU 阈值: 70%
- 内存阈值: 80%

手动扩缩容：
```bash
kubectl scale deployment/nexus --replicas=5 -n nexus
```

## 故障排查

```bash
# 查看 Pod 状态
kubectl get pods -n nexus

# 查看日志
kubectl logs -f deployment/nexus -n nexus

# 查看事件
kubectl get events -n nexus --sort-by='.lastTimestamp'

# 进入 Pod 调试
kubectl exec -it deployment/nexus -n nexus -- /bin/sh
```

## 备份和恢复

### Redis 备份

```bash
# 备份
kubectl exec -n nexus nexus-redis-0 -- redis-cli BGSAVE

# 导出数据
kubectl exec -n nexus nexus-redis-0 -- redis-cli --rdb /data/dump.rdb
```

### 配置备份

```bash
# 导出配置
kubectl get configmap nexus-config -n nexus -o yaml > config-backup.yaml
```

