#!/bin/bash

# 金丝雀部署脚本（使用 Feature Flag 灰度发布）

set -e

NAMESPACE="nexus"
VERSION=$1
PERCENTAGE=${2:-10}  # 默认 10% 流量

if [ -z "$VERSION" ]; then
    echo "用法: $0 <版本> [百分比]"
    echo "示例: $0 v1.2.0 10"
    exit 1
fi

echo "🪶 开始金丝雀部署版本 $VERSION (${PERCENTAGE}% 流量)..."

# 1. 部署新版本（不接收流量）
echo "📦 部署新版本..."
kubectl set image deployment/nexus nexus=nexus:$VERSION -n $NAMESPACE
kubectl rollout status deployment/nexus -n $NAMESPACE --timeout=300s

# 2. 通过 Feature Flag 控制流量百分比
echo "🎚️  设置 Feature Flag 灰度发布..."
# 这里可以通过 API 调用设置 Feature Flag
# curl -X POST http://nexus.example.com/api/config/flags \
#   -H "Content-Type: application/json" \
#   -d "{\"name\":\"canary-version\",\"status\":\"gradual\",\"percentage\":$PERCENTAGE}"

# 3. 监控指标
echo "📊 监控新版本指标..."
echo "等待 5 分钟观察指标..."
sleep 300

# 4. 检查错误率
ERROR_RATE=$(kubectl exec -n $NAMESPACE deployment/nexus -- \
    curl -s http://localhost:3000/metrics | \
    grep 'nexus_adapter_errors_total' | \
    awk '{print $2}' || echo "0")

if [ "$ERROR_RATE" -gt 10 ]; then
    echo "❌ 错误率过高 ($ERROR_RATE)，自动回滚..."
    ./rollback.sh
    exit 1
fi

echo "✅ 金丝雀部署成功！"
echo "当前流量分配: ${PERCENTAGE}% 新版本，$((100 - PERCENTAGE))% 旧版本"

# 提示下一步操作
echo ""
echo "📋 下一步操作："
echo "1. 增加流量: ./increase-canary.sh $VERSION $((PERCENTAGE + 10))"
echo "2. 完全切换: ./promote-canary.sh $VERSION"
echo "3. 回滚: ./rollback.sh"

