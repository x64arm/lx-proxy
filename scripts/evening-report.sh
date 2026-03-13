#!/bin/bash
# LX-Proxy 晚间进度汇报脚本
# 使用方法：将此脚本添加到 crontab: 0 18 * * * /path/to/evening-report.sh

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
