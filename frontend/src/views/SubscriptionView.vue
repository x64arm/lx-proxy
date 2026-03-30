<template>
  <div class="subscription-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🔗 {{ t('subscription.title') }}</span>
          <el-button type="primary" @click="refreshLinks" :loading="loading">
            🔄 {{ t('subscription.refresh') }}
          </el-button>
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
      </el-alert>

      <!-- 入站选择 -->
      <el-form :inline="true" class="mb-4">
        <el-form-item :label="t('subscription.selectInbound')">
          <el-select v-model="selectedInbound" :placeholder="t('subscription.allInbounds')" @change="loadLinks">
            <el-option :label="t('subscription.allInbounds')" value="all" />
            <el-option
              v-for="inbound in inbounds"
              :key="inbound.id"
              :label="`${inbound.name} (${inbound.protocol})`"
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
        <el-table-column :label="t('subscription.qrCode')" width="120" align="center">
          <template #default="{ row }">
            <el-button type="info" size="small" @click="showQRCode(row)">
              📱 {{ t('subscription.view') }}
            </el-button>
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
        <el-button @click="qrDialogVisible = false">{{ t('subscription.close') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import axios from 'axios'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Inbound {
  id: number
  name: string
  protocol: string
  port: number
}

interface SubscriptionLink {
  id: number
  name: string
  protocol: string
  subscription_url: string
}

const loading = ref(false)
const selectedInbound = ref<string>('all')
const inbounds = ref<Inbound[]>([])
const subscriptionLinks = ref<SubscriptionLink[]>([])
const qrDialogVisible = ref(false)
const currentQRCode = ref<string>('')

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
    const res = await axios.get('/api/inbounds')
    inbounds.value = res.data.data || []
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
    
    const res = await axios.get(url)
    const data = res.data.data || []
    
    if (selectedInbound.value === 'all') {
      // 全部入站，为每个入站生成订阅链接
      subscriptionLinks.value = data.map((inbound: Inbound) => ({
        id: inbound.id,
        name: inbound.name,
        protocol: inbound.protocol,
        subscription_url: `${window.location.origin}/api/inbounds/${inbound.id}/links`
      }))
    } else {
      // 单个入站
      subscriptionLinks.value = [{
        id: data.id,
        name: data.name,
        protocol: data.protocol,
        subscription_url: `${window.location.origin}/api/inbounds/${data.id}/links`
      }]
    }
  } catch (error) {
    ElMessage.error(t('subscription.loadLinksFailed'))
  } finally {
    loading.value = false
  }
}

// 刷新链接
const refreshLinks = () => {
  loadLinks()
  ElMessage.success(t('subscription.linksRefreshed'))
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

// 显示二维码
const showQRCode = (link: SubscriptionLink) => {
  // 使用二维码 API 生成二维码
  currentQRCode.value = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(link.subscription_url)}`
  qrDialogVisible.value = true
}

// 获取协议类型
const getProtocolType = (protocol: string) => {
  const types: Record<string, 'success' | 'warning' | 'info' | 'danger'> = {
    'vmess': 'success',
    'vless': 'success',
    'trojan': 'warning',
    'shadowsocks': 'info'
  }
  return types[protocol.toLowerCase()] || 'info'
}

onMounted(async () => {
  await loadInbounds()
  await loadLinks()
})
</script>

<style scoped>
.subscription-view {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.mb-4 {
  margin-bottom: 20px;
}

.link-container {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}

.link-container .el-input {
  flex: 1;
  min-width: 200px;
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

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .subscription-view {
    padding: 12px;
  }

  .box-card {
    margin: 0 -12px;
    border-radius: 0;
  }

  .box-card :deep(.el-card__body) {
    padding: 12px;
  }

  .card-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .card-header .el-button {
    width: 100%;
    min-height: 48px;
  }

  /* 表单移动端优化 */
  :deep(.el-form--inline) {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  :deep(.el-form-item) {
    width: 100%;
  }

  :deep(.el-form-item__label) {
    width: 100% !important;
    margin-bottom: 8px;
  }

  :deep(.el-select) {
    width: 100% !important;
  }

  /* 表格移动端优化 */
  :deep(.el-table) {
    font-size: 13px;
  }

  :deep(.el-table th) {
    padding: 8px 0;
    font-size: 12px;
  }

  :deep(.el-table td) {
    padding: 10px 0;
  }

  /* 链接容器优化 */
  .link-container {
    flex-direction: column;
    gap: 12px;
  }

  .link-container .el-input {
    width: 100%;
    min-width: auto;
  }

  .link-container .el-button {
    width: 100%;
    min-height: 48px;
  }

  /* QR 码对话框优化 */
  :deep(.el-dialog) {
    width: 90% !important;
    max-width: 320px !important;
  }

  .qr-image {
    max-width: 240px;
  }

  /* 分隔线优化 */
  :deep(.el-divider__text) {
    font-size: 14px;
  }

  /* 警告框优化 */
  :deep(.el-alert) {
    padding: 12px;
  }

  :deep(.el-alert__content) {
    font-size: 13px;
  }
}

/* ========== 触摸友好优化 ========== */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title) {
  min-height: 44px;
}

* {
  box-sizing: border-box;
}

.subscription-view {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
