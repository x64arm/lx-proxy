#!/bin/bash
# LX-Proxy 晚间进度汇报脚本
# 修复：添加完整 PATH 以支持 cron 环境

# 设置完整 PATH (cron 环境 PATH 很精简)
export PATH="/root/.nvm/versions/node/v24.14.0/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

# 验证 openclaw 可用
if ! command -v openclaw &> /dev/null; then
    echo "ERROR: openclaw command not found" >> /root/.openclaw/workspace/lx-proxy/logs/evening-report.log
    exit 1
fi

MESSAGE="🌙 晚间开发进度汇报 - $(date '+%Y-%m-%d')

✅ 今日完成：
- 待更新

🔄 进行中：
- 待更新

⚠️ 困难/阻塞：
- 无（如有会及时汇报）

📌 明日计划：
- 继续推进剩余工作"

/root/.nvm/versions/node/v24.14.0/bin/openclaw message send \
  --channel feishu \
  --target "ou_c2b0bc829616f15c46c319f32ecf0fb3" \
  --message "$MESSAGE"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Evening report sent successfully" >> /root/.openclaw/workspace/lx-proxy/logs/evening-report.log
