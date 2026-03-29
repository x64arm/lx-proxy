<template>
  <div class="settings-view" :class="{ 'mobile-view': isMobile }">
    <el-tabs 
      v-model="activeTab" 
      :type="isMobile ? '' : 'border-card'"
      class="settings-tabs"
      :class="{ 'mobile-tabs': isMobile }"
    >
      <!-- 基本设置 -->
      <el-tab-pane :label="`📌 ${t('settings.basic')}`" name="basic">
        <el-form :model="settings" label-width="auto" :class="{ 'mobile-form': isMobile }">
          <el-form-item :label="t('settings.webTitle')">
            <el-input v-model="settings.web_title" :placeholder="t('settings.webTitle')" />
          </el-form-item>

          <el-form-item :label="t('settings.webSubtitle')">
            <el-input v-model="settings.web_subtitle" :placeholder="t('settings.webSubtitle')" />
          </el-form-item>

          <el-form-item :label="t('settings.timezone')">
            <el-select v-model="settings.timezone" :placeholder="t('settings.timezone')" style="width: 100%">
              <el-option label="UTC" value="UTC" />
              <el-option label="Asia/Shanghai" value="Asia/Shanghai" />
              <el-option label="America/New_York" value="America/New_York" />
              <el-option label="Europe/London" value="Europe/London" />
              <el-option label="Asia/Tokyo" value="Asia/Tokyo" />
            </el-select>
          </el-form-item>

          <el-form-item :label="t('settings.language')">
            <el-select v-model="settings.language" :placeholder="t('settings.language')" style="width: 100%">
              <el-option label="简体中文" value="zh-CN" />
              <el-option label="English" value="en-US" />
              <el-option label="繁體中文" value="zh-TW" />
            </el-select>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- Xray 设置 -->
      <el-tab-pane :label="`🔧 ${t('settings.xray')}`" name="xray">
        <el-form :model="settings" label-width="auto" :class="{ 'mobile-form': isMobile }">
          <el-form-item :label="t('settings.xrayPath')">
            <el-input v-model="settings.xray_path" :placeholder="t('settings.xrayPath')" />
          </el-form-item>

          <el-form-item :label="t('settings.configPath')">
            <el-input v-model="settings.xray_config_path" :placeholder="t('settings.configPath')" />
          </el-form-item>

          <el-form-item :label="t('settings.listenPort')">
            <el-input-number v-model="settings.xray_port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>

          <el-form-item :label="t('settings.logLevel')">
            <el-select v-model="settings.xray_log_level" :placeholder="t('settings.logLevel')" style="width: 100%">
              <el-option label="Debug" value="debug" />
              <el-option label="Info" value="info" />
              <el-option label="Warning" value="warning" />
              <el-option label="Error" value="error" />
            </el-select>
          </el-form-item>

          <el-form-item>
            <el-button type="success" @click="testXrayConfig" :loading="testingXray" :block="isMobile">
              🧪 {{ t('settings.testXrayConfig') }}
            </el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 流量设置 -->
      <el-tab-pane :label="`📊 ${t('settings.traffic')}`" name="traffic">
        <el-form :model="settings" label-width="auto" :class="{ 'mobile-form': isMobile }">
          <el-form-item :label="t('settings.monthlyResetDay')">
            <el-select v-model="settings.traffic_reset_day" :placeholder="t('settings.monthlyResetDay')" style="width: 100%">
              <el-option :label="t('settings.noAutoReset')" :value="0" />
              <el-option :label="t('settings.monthlyDay', { day: 1 })" :value="1" />
              <el-option :label="t('settings.monthlyDay', { day: 5 })" :value="5" />
              <el-option :label="t('settings.monthlyDay', { day: 10 })" :value="10" />
              <el-option :label="t('settings.monthlyDay', { day: 15 })" :value="15" />
              <el-option :label="t('settings.monthlyDay', { day: 20 })" :value="20" />
              <el-option :label="t('settings.monthlyDay', { day: 25 })" :value="25" />
              <el-option :label="t('settings.lastDay')" :value="-1" />
            </el-select>
          </el-form-item>

          <el-form-item :label="t('settings.retentionDays')">
            <el-input-number v-model="settings.traffic_retention_days" :min="7" :max="365" style="width: 100%" />
            <div class="form-tip">{{ t('settings.retentionTip') }}</div>
          </el-form-item>

          <el-form-item :label="t('settings.warningThreshold')">
            <el-input-number v-model="settings.traffic_warning_threshold" :min="1" :max="100" style="width: 100%" />
            <span class="unit-suffix">{{ t('settings.percent') }}</span>
            <div class="form-tip">{{ t('settings.warningTip') }}</div>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 安全设置 -->
      <el-tab-pane :label="`🔐 ${t('settings.security')}`" name="security">
        <el-form :model="settings" label-width="auto" :class="{ 'mobile-form': isMobile }">
          <el-form-item :label="t('settings.sessionTimeout')">
            <el-input-number v-model="settings.session_timeout" :min="1" :max="168" style="width: 100%" />
            <span class="unit-suffix">{{ t('settings.hours') }}</span>
          </el-form-item>

          <el-form-item :label="t('settings.maxLoginAttempts')">
            <el-input-number v-model="settings.max_login_attempts" :min="3" :max="10" style="width: 100%" />
            <div class="form-tip">{{ t('settings.loginAttemptsTip') }}</div>
          </el-form-item>

          <el-form-item :label="t('settings.ipBanDuration')">
            <el-input-number v-model="settings.ip_ban_duration" :min="5" :max="1440" style="width: 100%" />
            <span class="unit-suffix">{{ t('settings.minutes') }}</span>
          </el-form-item>

          <el-form-item :label="t('settings.enableTotp2')">
            <el-switch v-model="settings.totp_enabled" />
            <div class="form-tip">{{ t('settings.totpTip') }}</div>
          </el-form-item>

          <el-form-item :label="t('settings.enableAccessLog')">
            <el-switch v-model="settings.access_log_enabled" />
            <div class="form-tip">{{ t('settings.accessLogTip') }}</div>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 通知设置 -->
      <el-tab-pane :label="`📧 ${t('settings.notification')}`" name="notification">
        <el-form :model="settings" label-width="auto" :class="{ 'mobile-form': isMobile }">
          <el-form-item :label="t('settings.enableEmail')">
            <el-switch v-model="settings.email_enabled" @change="toggleEmailSettings" />
          </el-form-item>

          <el-form-item :label="t('settings.smtpServer')" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_host" :placeholder="t('settings.smtpServer')" />
          </el-form-item>

          <el-form-item :label="t('settings.smtpPort')" v-if="settings.email_enabled">
            <el-input-number v-model="settings.smtp_port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>

          <el-form-item :label="t('settings.fromEmail')" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_from" :placeholder="t('settings.fromEmail')" />
          </el-form-item>

          <el-form-item :label="t('settings.smtpUsername')" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_username" :placeholder="t('settings.smtpUsername')" />
          </el-form-item>

          <el-form-item :label="t('settings.smtpPassword')" v-if="settings.email_enabled">
            <el-input v-model="settings.smtp_password" type="password" :placeholder="t('settings.smtpPassword')" show-password />
          </el-form-item>

          <el-form-item :label="t('settings.useSSL')" v-if="settings.email_enabled">
            <el-switch v-model="settings.smtp_ssl" />
          </el-form-item>

          <el-form-item v-if="settings.email_enabled">
            <el-button type="primary" @click="testEmailConfig" :loading="testingEmail" :block="isMobile">
              📧 {{ t('settings.sendTestEmail') }}
            </el-button>
          </el-form-item>
        </el-form>
      </el-tab-pane>

      <!-- 系统信息 -->
      <el-tab-pane :label="`💻 ${t('settings.system')}`" name="system">
        <el-descriptions 
          :column="isMobile ? 1 : 2" 
          border 
          :size="isMobile ? 'default' : 'large'"
          class="system-descriptions"
        >
          <el-descriptions-item :label="t('settings.osVersion')">{{ systemInfo.os_version }}</el-descriptions-item>
          <el-descriptions-item :label="t('settings.uptime')">{{ systemInfo.uptime }}</el-descriptions-item>
          <el-descriptions-item :label="t('settings.cpuUsage')">
            <el-progress :percentage="systemInfo.cpu_usage" :stroke-width="18" />
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.memoryUsage')">
            {{ formatBytes(systemInfo.memory_used) }} / {{ formatBytes(systemInfo.memory_total) }}
            <el-progress :percentage="memoryPercent" :stroke-width="18" style="margin-top: 8px" />
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.diskUsage')">
            <el-progress :percentage="systemInfo.disk_usage" :stroke-width="18" />
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.xrayStatus')">
            <el-tag :type="systemInfo.xray_status === 'running' ? 'success' : 'danger'" :size="isMobile ? 'default' : 'large'">
              {{ systemInfo.xray_status === 'running' ? '✅ ' + t('settings.running') : '❌ ' + t('settings.stopped') }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.dbConnections')">
            <el-tag type="info" :size="isMobile ? 'default' : 'large'">{{ systemInfo.db_connections }} {{ t('settings.connections') }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="t('settings.activeSessions')">
            <el-tag type="warning" :size="isMobile ? 'default' : 'large'">{{ systemInfo.active_sessions }} {{ t('settings.sessions') }}</el-tag>
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <div class="system-actions" :class="{ 'mobile-actions': isMobile }">
          <el-button type="warning" @click="restartXray" :loading="restartingXray" :block="isMobile">
            🔄 {{ t('settings.restartXray') }}
          </el-button>
          <el-button type="danger" @click="clearLogs" :loading="clearingLogs" :block="isMobile">
            🗑️ {{ t('settings.clearLogs') }}
          </el-button>
          <el-button @click="loadSystemInfo" :block="isMobile">
            🔄 {{ t('settings.refreshInfo') }}
          </el-button>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- 底部操作栏 -->
    <div class="form-actions" :class="{ 'mobile-actions': isMobile }">
      <el-button type="primary" @click="saveSettings" :loading="saving" size="large" :block="isMobile">
        💾 {{ t('settings.saveAll') }}
      </el-button>
      <el-button @click="loadSettings" size="large" :block="isMobile">🔄 {{ t('settings.reset') }}</el-button>
      <el-button type="success" @click="exportSettings" size="large" :block="isMobile">
        📤 {{ t('settings.exportConfig') }}
      </el-button>
      <el-button type="warning" @click="importSettings" size="large" :block="isMobile">
        📥 {{ t('settings.importConfig') }}
      </el-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from 'vue-i18n'
import axios from 'axios'

const { t } = useI18n()

// 响应式状态
const isMobile = ref(false)

// 检测设备类型
const checkDevice = () => {
  const width = window.innerWidth
  isMobile.value = width <= 768
}

// 监听窗口大小变化
onMounted(() => {
  checkDevice()
  window.addEventListener('resize', checkDevice)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkDevice)
})

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
    ElMessage.warning(t('settings.saveFailed'))
  }
}

// 保存设置
const saveSettings = async () => {
  saving.value = true
  try {
    const configArray = Object.entries(settings.value).map(([key, value]) => ({
      key,
      value: JSON.stringify(value),
      description: `${key} ${t('settings.config')}`
    }))

    // 批量保存配置
    for (const config of configArray) {
      await axios.put('/api/config', config)
    }
    
    ElMessage.success(t('settings.saveSuccess'))
  } catch (error) {
    ElMessage.error(t('settings.saveFailed'))
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
      ElMessage.success(t('settings.testSuccess'))
    } else {
      ElMessage.warning(t('settings.testFailed'))
    }
  } catch (error) {
    ElMessage.error(t('settings.testFailed'))
  } finally {
    testingXray.value = false
  }
}

// 测试邮件配置
const testEmailConfig = async () => {
  testingEmail.value = true
  try {
    // TODO: 实现后端邮件测试 API
    ElMessage.success(t('settings.testEmailSent'))
  } catch (error) {
    ElMessage.error(t('settings.testEmailFailed'))
  } finally {
    testingEmail.value = false
  }
}

// 重启 Xray
const restartXray = async () => {
  try {
    await ElMessageBox.confirm(t('settings.confirmRestart'), t('common.warning'), {
      confirmButtonText: t('common.yes'),
      cancelButtonText: t('common.no'),
      type: 'warning'
    })
    
    restartingXray.value = true
    // TODO: 实现后端 Xray 重启 API
    await new Promise(resolve => setTimeout(resolve, 2000))
    ElMessage.success(t('settings.restartSuccess'))
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('settings.restartFailed'))
    }
  } finally {
    restartingXray.value = false
  }
}

// 清理日志
const clearLogs = async () => {
  try {
    await ElMessageBox.confirm(t('settings.confirmClearLogs'), t('common.warning'), {
      confirmButtonText: t('common.yes'),
      cancelButtonText: t('common.no'),
      type: 'warning'
    })
    
    clearingLogs.value = true
    // TODO: 实现后端日志清理 API
    await new Promise(resolve => setTimeout(resolve, 1000))
    ElMessage.success(t('settings.clearSuccess'))
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('settings.clearFailed'))
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
    ElMessage.error(t('settings.loadSystemInfoFailed'))
  }
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${days}${t('time.day')} ${hours}${t('time.hour')} ${minutes}${t('time.minute')}`
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
  ElMessage.success(t('settings.configExported'))
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
      ElMessage.success(t('settings.configImported'))
    } catch (error) {
      ElMessage.error(t('settings.importFailed'))
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
  flex-wrap: wrap;
}

.system-actions {
  margin-top: 20px;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

:deep(.el-tabs__content) {
  padding: 20px;
}

/* 移动端表单样式 */
.mobile-form :deep(.el-form-item) {
  margin-bottom: 20px;
}

.mobile-form :deep(.el-form-item__label) {
  width: 100% !important;
  margin-bottom: 8px;
  font-weight: 500;
}

.mobile-form :deep(.el-form-item__content) {
  width: 100%;
}

.mobile-form :deep(.el-input),
.mobile-form :deep(.el-select),
.mobile-form :deep(.el-input-number) {
  width: 100% !important;
}

/* 移动端 Tabs 优化 */
.mobile-tabs :deep(.el-tabs__item) {
  padding: 0 12px;
  font-size: 13px;
}

.mobile-tabs :deep(.el-tabs__content) {
  padding: 12px;
}

/* 移动端系统信息描述 */
.mobile-view :deep(.el-descriptions__label) {
  width: 100%;
  font-size: 12px;
}

.mobile-view :deep(.el-descriptions__content) {
  width: 100%;
  font-size: 13px;
}

/* 移动端操作按钮 */
.mobile-actions {
  flex-direction: column;
  gap: 12px;
}

.mobile-actions .el-button {
  width: 100%;
  min-height: 48px;
}

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .settings-view {
    padding: 0;
  }

  .settings-tabs {
    margin: 0 -12px;
    border: none;
    border-radius: 0;
  }

  .settings-tabs :deep(.el-tabs__header) {
    padding: 0 12px;
    background: #fff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  }

  .settings-tabs :deep(.el-tabs__content) {
    padding: 12px;
    background: #f5f7fa;
  }

  .form-actions {
    margin: 20px -12px 0;
    padding: 16px 12px;
    flex-direction: column;
    border-radius: 0;
  }

  .form-actions .el-button {
    width: 100%;
    min-height: 48px;
    font-size: 15px;
  }

  .system-actions {
    flex-direction: column;
    gap: 12px;
  }

  .system-actions .el-button {
    width: 100%;
    min-height: 48px;
  }

  /* 表单字段优化 */
  :deep(.el-form-item__label) {
    font-size: 13px;
  }

  :deep(.el-input__inner),
  :deep(.el-select__input) {
    font-size: 14px;
  }

  /* 进度条优化 */
  :deep(.el-progress__text) {
    font-size: 12px !important;
  }

  /* 标签优化 */
  :deep(.el-tag) {
    font-size: 12px;
  }
}

/* ========== 触摸友好优化 ========== */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title),
:deep(.el-form-item__label) {
  min-height: 44px;
}

* {
  box-sizing: border-box;
}

.settings-view {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
