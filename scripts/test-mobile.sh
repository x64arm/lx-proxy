#!/bin/bash

# P12 移动端适配测试运行脚本
# 用法：./scripts/test-mobile.sh

set -e

cd "$(dirname "$0")/.."

echo "🚀 开始运行 P12 移动端适配测试..."
echo ""

# 检查依赖
if ! command -v npx &> /dev/null; then
    echo "❌ 错误：npx 未安装，请先安装 Node.js"
    exit 1
fi

# 进入前端目录
cd frontend

# 安装依赖（如果需要）
if [ ! -d "node_modules" ]; then
    echo "📦 安装依赖..."
    npm install
fi

# 检查 Playwright 浏览器
echo "🔍 检查 Playwright 浏览器..."
npx playwright install --with-deps chromium 2>/dev/null || true

# 运行移动端测试
echo ""
echo "📱 运行移动端适配测试..."
echo ""

# 创建测试报告目录
mkdir -p test-results

# 运行测试（只运行移动端测试文件）
npx playwright test e2e/mobile-responsive.spec.ts \
    --reporter=html,list \
    --output=test-results \
    --timeout=30000 \
    --retries=1

# 生成测试报告
echo ""
echo "📊 生成测试报告..."
npx playwright show-report test-results/report.html 2>/dev/null || true

echo ""
echo "✅ 测试完成！"
echo ""
echo "📁 测试报告位置：frontend/test-results/report.html"
echo ""
echo "💡 提示："
echo "   - 查看 HTML 报告：npx playwright show-report test-results/report.html"
echo "   - 运行特定设备：npx playwright test --project='Mobile Safari'"
echo "   - 调试模式：npx playwright test --debug"
echo ""
