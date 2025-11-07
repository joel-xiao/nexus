#!/bin/bash

set -e

echo "🚀 部署 Nexus 到生产环境..."

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装，请先安装 Docker"
    exit 1
fi

# 检查 docker-compose 是否安装
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose 未安装，请先安装 docker-compose"
    exit 1
fi

# 构建镜像
echo "📦 构建 Docker 镜像..."
cd "$(dirname "$0")/../.."
docker-compose -f docker-compose.prod.yml build --no-cache

# 停止旧容器
echo "🛑 停止旧容器..."
docker-compose -f docker-compose.prod.yml down

# 启动新容器
echo "▶️  启动服务..."
docker-compose -f docker-compose.prod.yml up -d

# 等待服务启动
echo "⏳ 等待服务启动..."
sleep 10

# 检查健康状态
if curl -f http://localhost:${PORT:-3000}/health > /dev/null 2>&1; then
    echo "✅ Nexus 服务已成功启动"
    echo "📋 健康检查: http://localhost:${PORT:-3000}/health"
    echo "📋 API 文档: http://localhost:${PORT:-3000}/docs"
else
    echo "❌ 服务启动失败，请检查日志: docker-compose -f docker-compose.prod.yml logs"
    exit 1
fi

