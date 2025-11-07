#!/bin/bash

# Nexus 服务启动脚本

set -e

echo "🚀 启动 Nexus 服务..."

# 检查是否已经在运行
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "❌ 端口 3000 已被占用，请先运行 stop.sh 停止服务"
    exit 1
fi

# 先编译
echo "📦 编译项目..."
cargo build

# 后台运行
echo "▶️  启动服务..."
nohup ./target/debug/nexus > nexus.log 2>&1 &
PID=$!

# 等待一下确保服务启动
sleep 2

# 检查服务是否成功启动
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "✅ Nexus 服务已启动 (PID: $PID)"
    echo "📋 日志文件: ./nexus.log"
else
    echo "❌ 服务启动失败，请查看日志: ./nexus.log"
    exit 1
fi

