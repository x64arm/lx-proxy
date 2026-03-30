<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElNotification } from 'element-plus'
import api from '@/api'

const { t } = useI18n()

interface Inbound {
  id: number
  name: string
  protocol: string
  port: number
  enable: boolean
  remark: string
}

interface SubscriptionLink {
  id: number
  name: string
  protocol: string
  subscription_url: string
  encrypted_url?: string
  access_count?: number
}

interface AccessStats {
  total_access: number
  unique_ips: number
  last_24h_access: number
  last_7d_access: number
}

const loading = ref(false)
const selectedInbound = ref<string>('all')
const inbounds = ref<Inbound[]>([])
const subscriptionLinks = ref<SubscriptionLink[]>([])
const qrDialogVisible = ref(false)
const currentQRCode = ref<string>('')
const statsDialogVisible = ref(false)
const currentStats = ref<AccessStats | null>(null)
const configDialogVisible = ref(false)
const currentConfig = ref<string>('')
const configType = ref<'clash' | 'v2rayn' | 'singbox'>('clash')

const base64SubscriptionUrl = computed(() => {
  if (!selectedInbound.value) return ''
  const baseUrl = window.location.origin
  if (selectedInbound.value === 'all') {
    return `${baseUrl}/api/subscription/all`
  }
  return `${baseUrl}/api/subscription/${selectedInbound.value}`
})

// 加载入站列表
const loadInbounds = async () => {
  try {
    const res = await api.inbounds.list()
    inbounds.value = res.data || []
  } catch (error) {
    ElMessage.error(t('subscription.loadInboundsFailed'))
  }
}

// 加载订阅链接
const loadLinks = async () => {
  loading.value = true
  try {
    const url = selectedInbound.value === 'all'
      ? '/api/inbounds'
      : `/api/inbounds/${selectedInbound.value}/links`
    
    const res = await api.get(url)
    const data = res.data.data || []
    
    if (selectedInbound.value === 'all') {
      subscriptionLinks.value = data.map((inbound: Inbound) => ({
        id: inbound.id,
        name: inbound.name || inbound.remark,
        protocol: inbound.protocol,
        subscription_url: `${window.location.origin}/api/sub/${inbound.id}`
      }))
    } else {
      subscriptionLinks.value = [{
        id: data.id,
        name: data.name || data.remark,
        protocol: data.protocol,
        subscription_url: `${window.location.origin}/api/sub/${data.id}`
      }]
    }
  } catch (error) {
    ElMessage.error(t('subscription.loadLinksFailed'))
  } finally {
    loading.value = false
  }
}

// 生成加密链接
const generateEncryptedLink = async (link: SubscriptionLink) => {
  try {
    const res = await api.post(`/api/sub/${link.id}/encrypt`)
    const data = res.data
    if (data.encrypted_url) {
      link.encrypted_url = data.encrypted_url
      ElMessage.success(t('subscription.encryptedLinkGenerated'))
      ElNotification({
        title: t('subscription.encryptedLinkTitle'),
        message: t('subscription.encryptedLinkDesc'),
        type: 'success',
        duration: 5000
      })
    }
  } catch (error) {
    ElMessage.error(t('subscription.encryptFailed'))
  }
}

// 查看访问统计
const viewStats = async (link: SubscriptionLink) => {
  try {
    const res = await api.get(`/api/sub/${link.id}/stats`)
    currentStats.value = res.data
    statsDialogVisible.value = true
  } catch (error) {
    ElMessage.error(t('subscription.loadStatsFailed'))
  }
}

// 生成二维码
const showQRCode = (link: SubscriptionLink) => {
  currentQRCode.value = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(link.subscription_url)}`
  qrDialogVisible.value = true
}

// 批量生成二维码
const batchGenerateQR = async () => {
  try {
    const res = await api.get('/api/sub/qrcode/batch')
    const qrcodes = res.data
    // 创建一个新的对话框显示所有二维码
    ElNotification({
      title: t('subscription.batchQRTitle'),
      message: t('subscription.batchQRDesc', { count: qrcodes.length }),
      type: 'success',
      duration: 5000
    })
  } catch (error) {
    ElMessage.error(t('subscription.batchQRFailed'))
  }
}

// 导出客户端配置
const exportClientConfig = async (link: SubscriptionLink, type: 'clash' | 'v2rayn' | 'singbox') => {
  try {
    const res = await api.get(`/api/sub/${link.id}/${type}`)
    if (type === 'clash') {
      currentConfig.value = res.data
      configType.value = 'clash'
    } else {
      currentConfig.value = JSON.stringify(res.data, null, 2)
      configType.value = type
    }
    configDialogVisible.value = true
    ElMessage.success(t('subscription.configGenerated'))
  } catch (error) {
    ElMessage.error(t('subscription.configExportFailed'))
  }
}

// 复制配置
const copyConfig = () => {
  if (!currentConfig.value) return
  navigator.clipboard.writeText(currentConfig.value).then(() => {
    ElMessage.success(t('subscription.copied'))
  }).catch(() => {
    ElMessage.error(t('subscription.copyFailed'))
  })
}

// 下载配置文件
const downloadConfig = () => {
  if (!currentConfig.value) return
  const blob = new Blob([currentConfig.value], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `config.${configType.value === 'clash' ? 'yaml' : 'json'}`
  a.click()
  URL.revokeObjectURL(url)
  ElMessage.success(t('subscription.downloaded'))
}

// 复制链接
const copyLink = (text: string) => {
  if (!text) {
    ElMessage.warning(t('subscription.noLinkToCopy'))
    return
  }
  navigator.clipboard.writeText(text).then(() => {
    ElMessage.success(t('subscription.copied'))
  }).catch(() => {
    ElMessage.error(t('subscription.copyFailed'))
  })
}

// 刷新链接
const refreshLinks = () => {
  loadLinks()
  ElMessage.success(t('subscription.linksRefreshed'))
}

// 获取协议类型
const getProtocolType = (protocol: string) => {
  const types: Record<string, 'success' | 'warning' | 'info' | 'danger'> = {
    'vmess': 'success',
    'vless': 'warning',
    'trojan': 'info',
    'shadowsocks': 'danger'
  }
  return types[protocol] || 'info'
}

onMounted(() => {
  loadInbounds()
  loadLinks()
})
</script>

<template>
  <div class="subscription-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🔗 {{ t('subscription.title') }}</span>
          <div class="header-actions">
            <el-button type="success" @click="batchGenerateQR" :loading="loading">
              📱 {{ t('subscription.batchQR') }}
            </el-button>
            <el-button type="primary" @click="refreshLinks" :loading="loading">
              🔄 {{ t('subscription.refresh') }}
            </el-button>
          </div>
        </div>
      </template>

      <el-alert
        :title="t('subscription.instructionTitle')"
        type="info"
        :closable="false"
        class="mb-4"
      >
        <p>{{ t('subscription.instruction1') }}</p>
        <p>{{ t('subscription.instruction2') }}</p>
        <p>{{ t('subscription.instruction3') }}</p>
      </el-alert>

      <!-- 入站选择 -->
      <el-form :inline="true" class="mb-4">
        <el-form-item :label="t('subscription.selectInbound')">
          <el-select v-model="selectedInbound" :placeholder="t('subscription.allInbounds')" @change="loadLinks">
            <el-option :label="t('subscription.allInbounds')" value="all" />
            <el-option
              v-for="inbound in inbounds"
              :key="inbound.id"
              :label="`${inbound.remark} (${inbound.protocol})`"
              :value="inbound.id"
            />
          </el-select>
        </el-form-item>
      </el-form>

      <!-- 订阅链接列表 -->
      <el-table :data="subscriptionLinks" stripe style="width: 100%">
        <el-table-column prop="name" :label="t('subscription.name')" width="200" />
        <el-table-column prop="protocol" :label="t('subscription.protocol')" width="100">
          <template #default="{ row }">
            <el-tag :type="getProtocolType(row.protocol)">
              {{ row.protocol }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="access_count" :label="t('subscription.accessCount')" width="100" align="center">
          <template #default="{ row }">
            {{ row.access_count || 0 }}
          </template>
        </el-table-column>
        <el-table-column :label="t('subscription.subscriptionLink')" show-overflow-tooltip>
          <template #default="{ row }">
            <div class="link-container">
              <el-input
                :model-value="row.subscription_url"
                readonly
                size="small"
              />
              <el-button
                type="primary"
                size="small"
                @click="copyLink(row.subscription_url)"
              >
                📋 {{ t('subscription.copy') }}
              </el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column :label="t('subscription.operations')" width="280" fixed="right">
          <template #default="{ row }">
            <el-button
              type="success"
              size="small"
              @click="generateEncryptedLink(row)"
            >
              🔐 {{ t('subscription.encrypt') }}
            </el-button>
            <el-button
              type="info"
              size="small"
              @click="showQRCode(row)"
            >
              📱 {{ t('subscription.qr') }}
            </el-button>
            <el-button
              type="warning"
              size="small"
              @click="viewStats(row)"
            >
              📊 {{ t('subscription.stats') }}
            </el-button>
            <el-dropdown trigger="click" @command="(cmd) => exportClientConfig(row, cmd)">
              <el-button size="small">
                📥 {{ t('subscription.export') }} <el-icon class="el-icon--right"><arrow-down /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="clash">Clash</el-dropdown-item>
                  <el-dropdown-item command="v2rayn">V2RayN</el-dropdown-item>
                  <el-dropdown-item command="singbox">SingBox</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
        </el-table-column>
      </el-table>

      <!-- Base64 订阅 -->
      <el-divider content-position="left">📦 {{ t('subscription.base64Subscription') }}</el-divider>
      <el-form-item :label="t('subscription.base64SubscriptionLink')">
        <div class="link-container">
          <el-input
            :model-value="base64SubscriptionUrl"
            readonly
            :placeholder="t('subscription.selectInboundToGenerate')"
          />
          <el-button
            type="primary"
            @click="copyLink(base64SubscriptionUrl)"
            :disabled="!base64SubscriptionUrl"
          >
            📋 {{ t('subscription.copy') }}
          </el-button>
        </div>
      </el-form-item>
    </el-card>

    <!-- 二维码对话框 -->
    <el-dialog v-model="qrDialogVisible" :title="t('subscription.qrCodeTitle')" width="300px">
      <div v-if="currentQRCode" class="qr-container">
        <img :src="currentQRCode" alt="QR Code" class="qr-image" />
      </div>
      <template #footer>
        <el-button type="primary" @click="qrDialogVisible = false">{{ t('common.close') }}</el-button>
      </template>
    </el-dialog>

    <!-- 访问统计对话框 -->
    <el-dialog v-model="statsDialogVisible" :title="t('subscription.accessStats')" width="400px">
      <div v-if="currentStats" class="stats-container">
        <el-descriptions :column="1" border>
          <el-descriptions-item :label="t('subscription.totalAccess')">
            {{ currentStats.total_access }}
          </el-descriptions-item>
          <el-descriptions-item :label="t('subscription.uniqueIPs')">
            {{ currentStats.unique_ips }}
          </el-descriptions-item>
          <el-descriptions-item :label="t('subscription.last24hAccess')">
            {{ currentStats.last_24h_access }}
          </el-descriptions-item>
          <el-descriptions-item :label="t('subscription.last7dAccess')">
            {{ currentStats.last_7d_access }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      <template #footer>
        <el-button type="primary" @click="statsDialogVisible = false">{{ t('common.close') }}</el-button>
      </template>
    </el-dialog>

    <!-- 配置导出对话框 -->
    <el-dialog v-model="configDialogVisible" :title="t('subscription.clientConfig')" width="600px">
      <div class="config-container">
        <el-input
          v-model="currentConfig"
          type="textarea"
          :rows="15"
          readonly
          class="config-textarea"
        />
      </div>
      <template #footer>
        <el-button @click="copyConfig">📋 {{ t('subscription.copy') }}</el-button>
        <el-button type="primary" @click="downloadConfig">📥 {{ t('subscription.download') }}</el-button>
        <el-button @click="configDialogVisible = false">{{ t('common.close') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.subscription-view {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.link-container {
  display: flex;
  gap: 8px;
  align-items: center;
}

.qr-container {
  display: flex;
  justify-content: center;
  padding: 20px;
}

.qr-image {
  max-width: 100%;
  height: auto;
}

.stats-container {
  padding: 10px 0;
}

.config-container {
  margin-top: 10px;
}

.config-textarea {
  font-family: 'Courier New', monospace;
  font-size: 12px;
}
</style>
