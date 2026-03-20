#!/bin/bash
# LX-Proxy 早间进度汇报脚本
# 修复：添加完整 PATH 以支持 cron 环境

# 设置完整 PATH (cron 环境 PATH 很精简)
export PATH="/root/.nvm/versions/node/v24.14.0/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# 验证 openclaw 可用
if ! command -v openclaw &> /dev/null; then
    echo "ERROR: openclaw command not found" >> /root/.openclaw/workspace/lx-proxy/logs/morning-report.log
    exit 1
fi

MESSAGE="☀️ 早间开发进度汇报 - $(date '+%Y-%m-%d')

📋 LX-Proxy 项目状态：
- 后端：Rust + Axum + PostgreSQL ✅
- 前端：Vue 3 + TypeScript 🔄
- Xray 集成：配置生成 ✅

📊 今日计划：
1. 继续前端页面开发
2. 数据库初始化
3. Docker 部署配置

💡 当前状态：开发进行中

如有问题会随时联系您。"

/root/.nvm/versions/node/v24.14.0/bin/openclaw message send \
  --channel feishu \
  --target "ou_c2b0bc829616f15c46c319f32ecf0fb3" \
  --message "$MESSAGE"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Morning report sent successfully" >> /root/.openclaw/workspace/lx-proxy/logs/morning-report.log
