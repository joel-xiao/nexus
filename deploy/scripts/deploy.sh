#!/bin/bash

# Forerunner 部署脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🚀 开始部署 Forerunner..."

# 检查环境
command -v kubectl >/dev/null 2>&1 || { echo "❌ kubectl 未安装"; exit 1; }
command -v docker >/dev/null 2>&1 || { echo "❌ docker 未安装"; exit 1; }

# 构建镜像
echo "📦 构建 Docker 镜像..."
cd "$PROJECT_ROOT"
docker build -t nexus:latest -f Dockerfile .

# 如果有 Docker registry，推送镜像
if [ -n "$DOCKER_REGISTRY" ]; then
    echo "📤 推送镜像到 registry..."
    docker tag nexus:latest "$DOCKER_REGISTRY/nexus:latest"
    docker push "$DOCKER_REGISTRY/nexus:latest"
fi

# 部署到 Kubernetes
echo "☸️  部署到 Kubernetes..."
kubectl apply -f "$SCRIPT_DIR/k8s/namespace.yaml"
kubectl apply -f "$SCRIPT_DIR/k8s/configmap.yaml"
kubectl apply -f "$SCRIPT_DIR/k8s/redis-deployment.yaml"
kubectl apply -f "$SCRIPT_DIR/k8s/deployment.yaml"
kubectl apply -f "$SCRIPT_DIR/k8s/ingress.yaml"
kubectl apply -f "$SCRIPT_DIR/k8s/hpa.yaml"

# 等待部署完成
echo "⏳ 等待部署完成..."
kubectl rollout status deployment/nexus -n nexus --timeout=300s

echo "✅ 部署完成！"
echo "📊 检查服务状态："
kubectl get pods -n nexus
kubectl get svc -n nexus

