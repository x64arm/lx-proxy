<template>
  <div class="settings-view">
    <el-card class="box-card">
      <template #header>
        <span>⚙️ 系统设置</span>
      </template>

      <el-form :model="settings" label-width="120px" size="large">
        <!-- 基本设置 -->
        <el-divider content-position="left">📌 基本设置</el-divider>
        
        <el-form-item label="网站标题">
          <el-input v-model="settings.web_title" placeholder="LX-Proxy" />
        </el-form-item>

        <el-form-item label="网站副标题">
          <el-input v-model="settings.web_subtitle" placeholder="Xray 代理管理面板" />
        </el-form-item>

        <el-form-item label="时区">
          <el-select v-model="settings.timezone" placeholder="选择时区">
            <el-option label="UTC" value="UTC" />
            <el-option label="Asia/Shanghai" value="Asia/Shanghai" />
            <el-option label="America/New_York" value="America/New_York" />
            <el-option label="Europe/London" value="Europe/London" />
          </el-select>
        </el-form-item>

        <!-- Xray 设置 -->
        <el-divider content-position="left">🔧 Xray 设置</el-divider>

        <el-form-item label="Xray 路径">
          <el-input v-model="settings.xray_path" placeholder="/usr/local/bin/xray" />
        </el-form-item>

        <el-form-item label="配置文件路径">
          <el-input v-model="settings.xray_config_path" placeholder="/etc/xray/config.json" />
        </el-form-item>

        <el-form-item label="监听端口">
          <el-input-number v-model="settings.xray_port" :min="1" :max="65535" />
        </el-form-item>

        <!-- 流量设置 -->
        <el-divider content-position="left">📊 流量设置</el-divider>

        <el-form-item label="流量重置日">
          <el-select v-model="settings.traffic_reset_day" placeholder="选择日期">
            <el-option label="每月 1 日" :value="1" />
            <el-option label="每月 15 日" :value="15" />
            <el-option label="不自动重置" :value="0" />
          </el-select>
        </el-form-item>

        <el-form-item label="流量统计保留天数">
          <el-input-number v-model="settings.traffic_retention_days" :min="7" :max="365" />
        </el-form-item>

        <!-- 安全设置 -->
        <el-divider content-position="left">🔐 安全设置</el-divider>

        <el-form-item label="会话超时 (小时)">
          <el-input-number v-model="settings.session_timeout" :min="1" :max="168" />
        </el-form-item>

        <el-form-item label="最大登录尝试">
          <el-input-number v-model="settings.max_login_attempts" :min="3" :max="10" />
        </el-form-item>

        <el-form-item label="IP 封禁时长 (分钟)">
          <el-input-number v-model="settings.ip_ban_duration" :min="5" :max="1440" />
        </el-form-item>

        <!-- 通知设置 -->
        <el-divider content-position="left">📧 通知设置</el-divider>

        <el-form-item label="启用邮件通知">
          <el-switch v-model="settings.email_enabled" />
        </el-form-item>

        <el-form-item label="SMTP 服务器">
          <el-input v-model="settings.smtp_host" placeholder="smtp.example.com" :disabled="!settings.email_enabled" />
        </el-form-item>

        <el-form-item label="SMTP 端口">
          <el-input-number v-model="settings.smtp_port" :min="1" :max="65535" :disabled="!settings.email_enabled" />
        </el-form-item>

        <el-form-item label="发件人邮箱">
          <el-input v-model="settings.smtp_from" placeholder="noreply@example.com" :disabled="!settings.email_enabled" />
        </el-form-item>

        <!-- 操作按钮 -->
        <el-form-item class="mt-4">
          <el-button type="primary" @click="saveSettings" :loading="saving">
            💾 保存设置
          </el-button>
          <el-button @click="loadSettings">🔄 重置</el-button>
          <el-button type="success" @click="testXrayConfig">
            🧪 测试 Xray 配置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 系统信息 -->
    <el-card class="box-card mt-4">
      <template #header>
        <span>💻 系统信息</span>
      </template>
      <el-descriptions :column="2" border>
        <el-descriptions-item label="系统版本">{{ systemInfo.os_version }}</el-descriptions-item>
        <el-descriptions-item label="CPU 使用率">{{ systemInfo.cpu_usage }}%</el-descriptions-item>
        <el-descriptions-item label="内存使用率">{{ systemInfo.memory_usage }}%</el-descriptions-item>
        <el-descriptions-item label="磁盘使用率">{{ systemInfo.disk_usage }}%</el-descriptions-item>
        <el-descriptions-item label="Xray 状态">
          <el-tag :type="systemInfo.xray_status === 'running' ? 'success' : 'danger'">
            {{ systemInfo.xray_status }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="运行时间">{{ systemInfo.uptime }}</el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import axios from 'axios'

interface Settings {
  web_title: string
  web_subtitle: string
  timezone: string
  xray_path: string
  xray_config_path: string
  xray_port: number
  traffic_reset_day: number
  traffic_retention_days: number
  session_timeout: number
  max_login_attempts: number
  ip_ban_duration: number
  email_enabled: boolean
  smtp_host: string
  smtp_port: number
  smtp_from: string
}

interface SystemInfo {
  os_version: string
  cpu_usage: number
  memory_usage: number
  disk_usage: number
  xray_status: string
  uptime: string
}

const saving = ref(false)
const settings = ref<Settings>({
  web_title: 'LX-Proxy',
  web_subtitle: 'Xray 代理管理面板',
  timezone: 'Asia/Shanghai',
  xray_path: '/usr/local/bin/xray',
  xray_config_path: '/etc/xray/config.json',
  xray_port: 443,
  traffic_reset_day: 1,
  traffic_retention_days: 30,
  session_timeout: 24,
  max_login_attempts: 5,
  ip_ban_duration: 60,
  email_enabled: false,
  smtp_host: '',
  smtp_port: 587,
  smtp_from: ''
})

const systemInfo = ref<SystemInfo>({
  os_version: '',
  cpu_usage: 0,
  memory_usage: 0,
  disk_usage: 0,
  xray_status: 'unknown',
  uptime: ''
})

// 加载设置
const loadSettings = async () => {
  try {
    const res = await axios.get('/api/config')
    settings.value = res.data.data || settings.value
  } catch (error) {
    ElMessage.warning('加载设置失败，使用默认值')
  }
}

// 保存设置
const saveSettings = async () => {
  saving.value = true
  try {
    await axios.put('/api/config', settings.value)
    ElMessage.success('设置已保存')
  } catch (error) {
    ElMessage.error('保存设置失败')
  } finally {
    saving.value = false
  }
}

// 测试 Xray 配置
const testXrayConfig = async () => {
  try {
    const res = await axios.get('/api/config/xray')
    if (res.data.success) {
      ElMessage.success('Xray 配置测试通过')
    } else {
      ElMessage.warning('Xray 配置测试失败：' + res.data.message)
    }
  } catch (error) {
    ElMessage.error('Xray 配置测试失败')
  }
}

// 加载系统信息
const loadSystemInfo = async () => {
  try {
    const res = await axios.get('/api/system/status')
    systemInfo.value = res.data.data || systemInfo.value
  } catch (error) {
    ElMessage.error('加载系统信息失败')
  }
}

onMounted(async () => {
  await loadSettings()
  await loadSystemInfo()
})
</script>

<style scoped>
.settings-view {
  padding: 20px;
}

.mt-4 {
  margin-top: 20px;
}
</style>
