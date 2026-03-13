#!/bin/bash
# LX-Proxy 早间进度汇报脚本
# 使用方法：将此脚本添加到 crontab: 0 8 * * * /path/to/morning-report.sh

# 加载 NVM 和 PATH
export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
export PATH="$HOME/.nvm/versions/node/v24.14.0/bin:$PATH"

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

openclaw message send \
  --channel feishu \
  --target "ou_c2b0bc829616f15c46c319f32ecf0fb3" \
  --message "$MESSAGE"
