#!/bin/bash

# Nexus 服务停止脚本

echo "🛑 停止 Nexus 服务..."

# 查找并终止在端口 3000 上运行的进程
PID=$(lsof -ti:3000 2>/dev/null || true)

if [ -z "$PID" ]; then
    echo "ℹ️  没有运行中的 Nexus 服务"
    exit 0
fi

echo "找到进程 PID: $PID"
kill -9 $PID 2>/dev/null || true

# 等待一下确保进程已终止
sleep 1

# 再次检查是否还有进程在运行
if lsof -ti:3000 >/dev/null 2>&1; then
    echo "⚠️  警告: 可能还有进程在运行"
else
    echo "✅ Nexus 服务已停止"
fi

