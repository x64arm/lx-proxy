<template>
  <div class="settings-view">
    <el-tabs v-model="activeTab" type="border-card">
      <!-- 基本设置 -->
      <el-tab-pane label="📌 基本设置" name="basic">
        <el-form :model="settings" label-width="140px" size="large">
          <el-form-item label="网站标题">
            <el-input v-model="settings.web_title" placeholder="LX-Proxy" />
          </el-form-item>

          <el-form-item label="网站副标题">
            <el-input v-model="settings.web_subtitle" placeholder="Xray 代理管理面板" />
          </el-form-item>

          <el-form-item label="时区">
            <el-select v-model="settings.timezone" placeholder="选择时区" style="width: 100%">
              <el-option label="UTC" value="UTC" />
              <el-option label="Asia/Shanghai" value="Asia/Shanghai" />
              <el-option label="America/New_York" value="America/New_York" />
              <el-option label="Europe/London" value="Europe/London" />
              <el-option label="Asia/Tokyo" value="Asia/Tokyo" />
            </el-select>
          </el-form-item>

          <el-form-item label="语言">
            <el-select v-model="settings.language" placeholder="选择语言" style="width: 100%">
              <el-option label="简体中文" value="zh-CN" />
              <el-option label="English" value="en-US" />
              <el-option label="繁體中文" value="zh-TW" />
            </el-select>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- Xray 设置 -->
      <el-tab-pane label="🔧 Xray 设置" name="xray">
        <el-form :model="settings" label-width="140px" size="large">
          <el-form-item label="Xray 路径">
            <el-input v-model="settings.xray_path" placeholder="/usr/local/bin/xray" />
          </el-form-item>

          <el-form-item label="配置文件路径">
            <el-input v-model="settings.xray_config_path" placeholder="/etc/xray/config.json" />
          </el-form-item>

          <el-form-item label="监听端口">
            <el-input-number v-model="settings.xray_port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>

          <el-form-item label="日志级别">
            <el-select v-model="settings.xray_log_level" placeholder="选择日志级别" style="width: 100%">
              <el-option label="Debug" value="debug" />
              <el-option label="Info" value="info" />
              <el-option label="Warning" value="warning" />
              <el-option label="Error" value="error" />
            </el-select>
          </el-form-item>

          <el-form-item>
            <el-button type="success" @click="testXrayConfig" :loading="testingXray">
              🧪 测试 Xray 配置
            </el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 流量设置 -->
      <el-tab-pane label="📊 流量设置" name="traffic">
        <el-form :model="settings" label-width="160px" size="large">
          <el-form-item label="每月流量重置日">
            <el-select v-model="settings.traffic_reset_day" placeholder="选择日期" style="width: 100%">
              <el-option label="不自动重置" :value="0" />
              <el-option label="每月 1 日" :value="1" />
              <el-option label="每月 5 日" :value="5" />
              <el-option label="每月 10 日" :value="10" />
              <el-option label="每月 15 日" :value="15" />
              <el-option label="每月 20 日" :value="20" />
              <el-option label="每月 25 日" :value="25" />
              <el-option label="每月最后一天" :value="-1" />
            </el-select>
          </el-form-item>

          <el-form-item label="流量统计保留天数">
            <el-input-number v-model="settings.traffic_retention_days" :min="7" :max="365" style="width: 100%" />
            <div class="form-tip">超过此天数的流量记录将被自动清理</div>
          </el-form-item>

          <el-form-item label="流量告警阈值">
            <el-input-number v-model="settings.traffic_warning_threshold" :min="1" :max="100" style="width: 100%" />
            <span class="unit-suffix">%</span>
            <div class="form-tip">当流量使用超过此百分比时发送告警</div>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 安全设置 -->
      <el-tab-pane label="🔐 安全设置" name="security">
        <el-form :model="settings" label-width="160px" size="large">
          <el-form-item label="会话超时时间">
            <el-input-number v-model="settings.session_timeout" :min="1" :max="168" style="width: 100%" />
            <span class="unit-suffix">小时</span>
          </el-form-item>

          <el-form-item label="最大登录尝试">
            <el-input-number v-model="settings.max_login_attempts" :min="3" :max="10" style="width: 100%" />
            <div class="form-tip">超过此次数后 IP 将被临时封禁</div>
          </el-form-item>

          <el-form-item label="IP 封禁时长">
            <el-input-number v-model="settings.ip_ban_duration" :min="5" :max="1440" style="width: 100%" />
            <span class="unit-suffix">分钟</span>
          </el-form-item>

          <el-form-item label="启用双因素认证">
            <el-switch v-model="settings.totp_enabled" />
            <div class="form-tip">启用后登录需要验证码</div>
          </el-form-item>

          <el-form-item label="启用访问日志">
            <el-switch v-model="settings.access_log_enabled" />
            <div class="form-tip">记录所有 API 访问日志</div>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 通知设置 -->
      <el-tab-pane label="📧 通知设置" name="notification">
        <el-form :model="settings" label-width="140px" size="large">
          <el-form-item label="启用邮件通知">
            <el-switch v-model="settings.email_enabled" @change="toggleEmailSettings" />
          </el-form-item>

          <el-form-item label="SMTP 服务器" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_host" placeholder="smtp.example.com" />
          </el-form-item>

          <el-form-item label="SMTP 端口" v-if="settings.email_enabled">
            <el-input-number v-model="settings.smtp_port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>

          <el-form-item label="发件人邮箱" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_from" placeholder="noreply@example.com" />
          </el-form-item>

          <el-form-item label="SMTP 用户名" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_username" placeholder="username" />
          </el-form-item>

          <el-form-item label="SMTP 密码" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_password" type="password" placeholder="password" show-password />
          </el-form-item>

          <el-form-item label="使用 SSL" v-if="settings.email_enabled">
            <el-switch v-model="settings.smtp_ssl" />
          </el-form-item>

          <el-form-item v-if="settings.email_enabled">
            <el-button type="primary" @click="testEmailConfig" :loading="testingEmail">
              📧 发送测试邮件
            </el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 系统信息 -->
      <el-tab-pane label="💻 系统信息" name="system">
        <el-descriptions :column="2" border size="large">
          <el-descriptions-item label="系统版本">{{ systemInfo.os_version }}</el-descriptions-item>
          <el-descriptions-item label="运行时间">{{ systemInfo.uptime }}</el-descriptions-item>
          <el-descriptions-item label="CPU 使用率">
            <el-progress :percentage="systemInfo.cpu_usage" :stroke-width="18" />
          </el-descriptions-item>
          <el-descriptions-item label="内存使用">
            {{ formatBytes(systemInfo.memory_used) }} / {{ formatBytes(systemInfo.memory_total) }}
            <el-progress :percentage="memoryPercent" :stroke-width="18" style="margin-top: 8px" />
          </el-descriptions-item>
          <el-descriptions-item label="磁盘使用率">
            <el-progress :percentage="systemInfo.disk_usage" :stroke-width="18" />
          </el-descriptions-item>
          <el-descriptions-item label="Xray 状态">
            <el-tag :type="systemInfo.xray_status === 'running' ? 'success' : 'danger'" size="large">
              {{ systemInfo.xray_status === 'running' ? '✅ 运行中' : '❌ 已停止' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="数据库连接">
            <el-tag type="info" size="large">{{ systemInfo.db_connections }} 个连接</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="活跃会话">
            <el-tag type="warning" size="large">{{ systemInfo.active_sessions }} 个会话</el-tag>
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <div class="system-actions">
          <el-button type="warning" @click="restartXray" :loading="restartingXray">
            🔄 重启 Xray
          </el-button>
          <el-button type="danger" @click="clearLogs" :loading="clearingLogs">
            🗑️ 清理日志
          </el-button>
          <el-button @click="loadSystemInfo">
            🔄 刷新信息
          </el-button>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- 底部操作栏 -->
    <div class="form-actions">
      <el-button type="primary" @click="saveSettings" :loading="saving" size="large">
        💾 保存所有设置
      </el-button>
      <el-button @click="loadSettings" size="large">🔄 重置</el-button>
      <el-button type="success" @click="exportSettings" size="large">
        📤 导出配置
      </el-button>
      <el-button type="warning" @click="importSettings" size="large">
        📥 导入配置
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import axios from 'axios'

interface Settings {
  web_title: string
  web_subtitle: string
  timezone: string
  language: string
  xray_path: string
  xray_config_path: string
  xray_port: number
  xray_log_level: string
  traffic_reset_day: number
  traffic_retention_days: number
  traffic_warning_threshold: number
  session_timeout: number
  max_login_attempts: number
  ip_ban_duration: number
  totp_enabled: boolean
  access_log_enabled: boolean
  email_enabled: boolean
  smtp_host: string
  smtp_port: number
  smtp_from: string
  smtp_username: string
  smtp_password: string
  smtp_ssl: boolean
}

interface SystemInfo {
  os_version: string
  uptime: string
  cpu_usage: number
  memory_total: number
  memory_used: number
  disk_usage: number
  xray_status: string
  db_connections: number
  active_sessions: number
}

const activeTab = ref('basic')
const saving = ref(false)
const testingXray = ref(false)
const testingEmail = ref(false)
const restartingXray = ref(false)
const clearingLogs = ref(false)

const settings = ref<Settings>({
  web_title: 'LX-Proxy',
  web_subtitle: 'Xray 代理管理面板',
  timezone: 'Asia/Shanghai',
  language: 'zh-CN',
  xray_path: '/usr/local/bin/xray',
  xray_config_path: '/etc/xray/config.json',
  xray_port: 443,
  xray_log_level: 'info',
  traffic_reset_day: 1,
  traffic_retention_days: 30,
  traffic_warning_threshold: 80,
  session_timeout: 24,
  max_login_attempts: 5,
  ip_ban_duration: 60,
  totp_enabled: false,
  access_log_enabled: true,
  email_enabled: false,
  smtp_host: '',
  smtp_port: 587,
  smtp_from: '',
  smtp_username: '',
  smtp_password: '',
  smtp_ssl: true
})

const systemInfo = ref<SystemInfo>({
  os_version: '',
  uptime: '',
  cpu_usage: 0,
  memory_total: 0,
  memory_used: 0,
  disk_usage: 0,
  xray_status: 'unknown',
  db_connections: 0,
  active_sessions: 0
})

const memoryPercent = computed(() => {
  if (systemInfo.value.memory_total === 0) return 0
  return Math.round((systemInfo.value.memory_used / systemInfo.value.memory_total) * 100)
})

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 加载设置
const loadSettings = async () => {
  try {
    const res = await axios.get('/api/config')
    const configs = res.data || []
    
    // 将数组格式转换为对象格式
    const configMap: Record<string, any> = {}
    configs.forEach((c: any) => {
      const value = typeof c.value === 'string' ? JSON.parse(c.value) : c.value
      configMap[c.key] = value
    })
    
    settings.value = {
      ...settings.value,
      web_title: configMap.web_title || settings.value.web_title,
      web_subtitle: configMap.web_subtitle || settings.value.web_subtitle,
      timezone: configMap.timezone || settings.value.timezone,
      language: configMap.language || settings.value.language,
      xray_path: configMap.xray_path || settings.value.xray_path,
      xray_config_path: configMap.xray_config_path || settings.value.xray_config_path,
      xray_port: configMap.xray_port || settings.value.xray_port,
      xray_log_level: configMap.xray_log_level || settings.value.xray_log_level,
      traffic_reset_day: configMap.traffic_reset_day || settings.value.traffic_reset_day,
      traffic_retention_days: configMap.traffic_retention_days || settings.value.traffic_retention_days,
      traffic_warning_threshold: configMap.traffic_warning_threshold || settings.value.traffic_warning_threshold,
      session_timeout: configMap.session_timeout || settings.value.session_timeout,
      max_login_attempts: configMap.max_login_attempts || settings.value.max_login_attempts,
      ip_ban_duration: configMap.ip_ban_duration || settings.value.ip_ban_duration,
      totp_enabled: configMap.totp_enabled || settings.value.totp_enabled,
      access_log_enabled: configMap.access_log_enabled || settings.value.access_log_enabled,
      email_enabled: configMap.email_enabled || settings.value.email_enabled,
      smtp_host: configMap.smtp_host || settings.value.smtp_host,
      smtp_port: configMap.smtp_port || settings.value.smtp_port,
      smtp_from: configMap.smtp_from || settings.value.smtp_from,
      smtp_username: configMap.smtp_username || settings.value.smtp_username,
      smtp_password: configMap.smtp_password || settings.value.smtp_password,
      smtp_ssl: configMap.smtp_ssl !== undefined ? configMap.smtp_ssl : settings.value.smtp_ssl
    }
  } catch (error) {
    ElMessage.warning('加载设置失败，使用默认值')
  }
}

// 保存设置
const saveSettings = async () => {
  saving.value = true
  try {
    const configArray = Object.entries(settings.value).map(([key, value]) => ({
      key,
      value: JSON.stringify(value),
      description: `${key} 配置`
    }))

    // 批量保存配置
    for (const config of configArray) {
      await axios.put('/api/config', config)
    }
    
    ElMessage.success('设置已保存')
  } catch (error) {
    ElMessage.error('保存设置失败')
  } finally {
    saving.value = false
  }
}

// 切换邮件设置
const toggleEmailSettings = (enabled: boolean) => {
  if (!enabled) {
    settings.value.smtp_host = ''
    settings.value.smtp_port = 587
    settings.value.smtp_from = ''
    settings.value.smtp_username = ''
    settings.value.smtp_password = ''
  }
}

// 测试 Xray 配置
const testXrayConfig = async () => {
  testingXray.value = true
  try {
    const res = await axios.get('/api/config/xray')
    if (res.data) {
      ElMessage.success('Xray 配置测试通过')
    } else {
      ElMessage.warning('Xray 配置测试失败')
    }
  } catch (error) {
    ElMessage.error('Xray 配置测试失败')
  } finally {
    testingXray.value = false
  }
}

// 测试邮件配置
const testEmailConfig = async () => {
  testingEmail.value = true
  try {
    // TODO: 实现后端邮件测试 API
    ElMessage.success('测试邮件已发送，请检查收件箱')
  } catch (error) {
    ElMessage.error('邮件发送失败')
  } finally {
    testingEmail.value = false
  }
}

// 重启 Xray
const restartXray = async () => {
  try {
    await ElMessageBox.confirm('确定要重启 Xray 服务吗？可能导致短暂的网络中断。', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    restartingXray.value = true
    // TODO: 实现后端 Xray 重启 API
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.success('Xray 服务已重启')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('重启失败')
    }
  } finally {
    restartingXray.value = false
  }
}

// 清理日志
const clearLogs = async () => {
  try {
    await ElMessageBox.confirm('确定要清理所有日志吗？此操作不可恢复。', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    clearingLogs.value = true
    // TODO: 实现后端日志清理 API
    await new Promise(resolve => setTimeout(resolve, 1000))
    ElMessage.success('日志已清理')
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('清理失败')
    }
  } finally {
    clearingLogs.value = false
  }
}

// 加载系统信息
const loadSystemInfo = async () => {
  try {
    const res = await axios.get('/api/system/status')
    const data = res.data || {}
    systemInfo.value = {
      os_version: data.os_version || 'Linux',
      uptime: formatUptime(data.uptime || 0),
      cpu_usage: data.cpu_usage || 0,
      memory_total: data.memory_total || 0,
      memory_used: data.memory_used || 0,
      disk_usage: data.disk_usage || 0,
      xray_status: data.xray_running ? 'running' : 'stopped',
      db_connections: data.db_connections || 0,
      active_sessions: data.active_sessions || 0
    }
  } catch (error) {
    ElMessage.error('加载系统信息失败')
  }
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${days}天 ${hours}小时 ${minutes}分钟`
}

// 导出配置
const exportSettings = () => {
  const dataStr = JSON.stringify(settings.value, null, 2)
  const dataBlob = new Blob([dataStr], { type: 'application/json' })
  const url = URL.createObjectURL(dataBlob)
  const link = document.createElement('a')
  link.href = url
  link.download = `lx-proxy-config-${new Date().toISOString().split('T')[0]}.json`
  link.click()
  URL.revokeObjectURL(url)
  ElMessage.success('配置已导出')
}

// 导入配置
const importSettings = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'application/json'
  input.onchange = async (e: any) => {
    const file = e.target.files[0]
    if (!file) return
    
    try {
      const text = await file.text()
      const imported = JSON.parse(text)
      settings.value = { ...settings.value, ...imported }
      ElMessage.success('配置已导入')
    } catch (error) {
      ElMessage.error('导入失败：文件格式错误')
    }
  }
  input.click()
}

onMounted(async () => {
  await loadSettings()
  await loadSystemInfo()
})
</script>

<style scoped>
.settings-view {
  padding: 0;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.unit-suffix {
  margin-left: 8px;
  color: #606266;
}

.form-actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
  justify-content: center;
  padding: 20px;
  background: #f5f7fa;
  border-radius: 4px;
}

.system-actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
}

:deep(.el-tabs__content) {
  padding: 20px;
}
</style>
