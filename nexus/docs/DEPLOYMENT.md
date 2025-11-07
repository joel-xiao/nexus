# Nexus 部署快速指南

## 🚀 快速开始

### 1. Docker Compose（推荐用于开发/测试）

```bash
# 克隆项目
git clone <repository>
cd dragonchain-nexus

# 启动所有服务
docker-compose up -d

# 查看日志
docker-compose logs -f nexus

# 访问服务
curl http://localhost:3000/health
```

### 2. Kubernetes 生产部署

#### 前置要求
- Kubernetes 集群（1.20+）
- kubectl 配置正确
- Docker 已安装

#### 快速部署

```bash
# 使用 Makefile
make deploy

# 或使用脚本
./deploy/deploy.sh

# 查看状态
make status
```

#### 手动部署步骤

```bash
# 1. 创建命名空间
kubectl apply -f deploy/k8s/namespace.yaml

# 2. 创建配置
kubectl apply -f deploy/k8s/configmap.yaml

# 3. 部署 Redis
kubectl apply -f deploy/k8s/redis-deployment.yaml

# 4. 部署 Nexus
kubectl apply -f deploy/k8s/deployment.yaml

# 5. 部署 Ingress（可选）
kubectl apply -f deploy/k8s/ingress.yaml

# 6. 部署 HPA（自动扩缩容）
kubectl apply -f deploy/k8s/hpa.yaml
```

## 📋 部署架构

```
┌─────────────────────────────────────────┐
│         Ingress (Nginx)                │
└──────────────┬──────────────────────────┘
               │
    ┌──────────▼──────────┐
    │   Nexus             │ 主服务（3+ 副本）
    │   Orchestrator      │
    └──────────┬───────────┘
               │
    ┌──────────┼───────────┐
    │          │           │
┌───▼───┐ ┌───▼────┐ ┌───▼─────┐
│ Redis │ │ Task   │ │Adapter  │
│ Cache │ │ Queue  │ │Sidecar  │
└───────┘ └────────┘ └─────────┘
```

## 🔄 升级/回滚

### 滚动升级（零停机）

```bash
# 更新镜像
kubectl set image deployment/nexus \
  nexus=nexus:v1.2.0 \
  -n nexus

# 查看升级状态
kubectl rollout status deployment/nexus -n nexus
```

### 金丝雀发布（灰度发布）

```bash
# 10% 流量
make canary VERSION=v1.2.0 PERCENTAGE=10

# 50% 流量
make canary VERSION=v1.2.0 PERCENTAGE=50

# 100% 流量（完全切换）
kubectl rollout promote nexus-rollout -n nexus
```

### 快速回滚

```bash
# 使用 Makefile
make rollback

# 或使用脚本
./deploy/rollback.sh

# 或手动
kubectl rollout undo deployment/nexus -n nexus
```

## 📊 监控和告警

### Prometheus

```bash
# 端口转发
kubectl port-forward svc/prometheus 9090:9090 -n monitoring

# 访问 UI
open http://localhost:9090
```

### Grafana

```bash
# 端口转发
kubectl port-forward svc/grafana 3001:3000 -n monitoring

# 访问 UI
open http://localhost:3001
# 默认用户名/密码: admin/admin
```

### 告警规则

已配置的告警：
- ⚠️ 高错误率（> 0.1 errors/s）
- ⚠️ 高延迟（95分位 > 5s）
- 🚨 服务宕机
- ⚠️ 内存使用率过高（> 90%）
- ⚠️ CPU 使用率过高（> 80%）

## 🔧 配置

### 环境变量

通过 ConfigMap 配置：

```bash
# 编辑配置
kubectl edit configmap nexus-config -n nexus

# 应用更改
kubectl rollout restart deployment/nexus -n nexus
```

### 动态配置适配器

```bash
# 导入配置
curl -X POST http://nexus.example.com/api/config/import \
  -H "Content-Type: application/json" \
  -d @adapter_config_example.json

# 热重载适配器
curl -X PUT http://nexus.example.com/api/config/reload/adapter \
  -H "Content-Type: application/json" \
  -d '{
    "name": "new-model",
    "api_key": "sk-xxx",
    "model": "gpt-4",
    "base_url": "https://api.openai.com",
    "enabled": true
  }'
```

## 🏥 健康检查

### 存活探针（Liveness）
- 端点: `GET /health`
- 检查应用是否运行

### 就绪探针（Readiness）
- 端点: `GET /ready`
- 检查：
  - Redis 连接
  - 适配器注册状态
  - 任务队列状态

```bash
# 测试健康检查
curl http://localhost:3000/api/health
curl http://localhost:3000/api/ready
```

## 🔐 安全

### Secrets 管理

```bash
# 创建 Secret
kubectl create secret generic nexus-secrets \
  --from-literal=OPENAI_API_KEY=sk-xxx \
  -n nexus

# 在 Deployment 中引用
# env:
# - name: OPENAI_API_KEY
#   valueFrom:
#     secretKeyRef:
#       name: nexus-secrets
#       key: OPENAI_API_KEY
```

### 网络策略

```bash
# 应用网络隔离策略
kubectl apply -f deploy/k8s/network-policy.yaml
```

## 📈 扩缩容

### 手动扩缩容

```bash
# 扩展到 5 个副本
kubectl scale deployment/nexus --replicas=5 -n nexus

# 查看 HPA 状态
kubectl get hpa -n nexus
```

### 自动扩缩容（HPA）

HPA 已配置：
- 最小副本: 3
- 最大副本: 10
- CPU 阈值: 70%
- 内存阈值: 80%

## 🐛 故障排查

```bash
# 查看 Pod 状态
kubectl get pods -n nexus

# 查看日志
kubectl logs -f deployment/nexus -n nexus

# 查看事件
kubectl get events -n nexus --sort-by='.lastTimestamp'

# 进入 Pod 调试
kubectl exec -it deployment/nexus -n nexus -- /bin/sh

# 查看资源使用
kubectl top pods -n nexus
```

## 📚 更多信息

详细文档请参考：
- [deploy/README.md](deploy/README.md) - 完整部署文档
- [adapter_config_example.json](adapter_config_example.json) - 适配器配置示例

