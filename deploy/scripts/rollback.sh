#!/bin/bash

# Forerunner 回滚脚本

set -e

NAMESPACE="nexus"
DEPLOYMENT="nexus"

echo "🔄 开始回滚 Forerunner..."

# 获取当前版本
CURRENT_REVISION=$(kubectl rollout history deployment/$DEPLOYMENT -n $NAMESPACE | tail -2 | head -1 | awk '{print $1}')
PREVIOUS_REVISION=$(kubectl rollout history deployment/$DEPLOYMENT -n $NAMESPACE | tail -3 | head -1 | awk '{print $1}')

echo "当前版本: $CURRENT_REVISION"
echo "回滚到: $PREVIOUS_REVISION"

# 执行回滚
kubectl rollout undo deployment/$DEPLOYMENT -n $NAMESPACE

# 等待回滚完成
echo "⏳ 等待回滚完成..."
kubectl rollout status deployment/$DEPLOYMENT -n $NAMESPACE --timeout=300s

echo "✅ 回滚完成！"
echo "📊 检查服务状态："
kubectl get pods -n $NAMESPACE -l app=nexus

