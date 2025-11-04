#!/bin/bash

# Nexus 测试运行脚本

set -e

echo "🧪 运行 Nexus 测试套件..."

# 颜色定义
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 运行单元测试（库测试）
echo -e "${BLUE}📦 运行单元测试...${NC}"
cargo test --lib -- --nocapture

# 运行集成测试（如果存在旧的测试文件）
if [ -f tests/integration_test.rs.bak ] || [ -f tests/integration_test.rs ]; then
    echo -e "${BLUE}🔗 运行集成测试...${NC}"
    # 注意：需要根据实际测试文件调整
    cargo test --test integration_test -- --nocapture 2>/dev/null || echo "集成测试文件已迁移"
fi

# 运行性能测试（如果需要）
if [ "$1" == "--performance" ]; then
    echo -e "${YELLOW}⚡ 运行性能测试...${NC}"
    cargo test --test performance_test -- --nocapture 2>/dev/null || echo "性能测试文件已迁移"
fi

# 运行所有测试（包括新的测试结构）
echo -e "${BLUE}🧪 运行所有测试...${NC}"
cargo test --all -- --nocapture

# 运行所有测试并显示覆盖率（如果安装了 cargo-tarpaulin）
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${BLUE}📊 生成测试覆盖率报告...${NC}"
    cargo tarpaulin --out Html --output-dir ./target/coverage
    echo -e "${GREEN}✅ 覆盖率报告已生成: ./target/coverage/tarpaulin-report.html${NC}"
fi

echo -e "${GREEN}✅ 所有测试完成！${NC}"

