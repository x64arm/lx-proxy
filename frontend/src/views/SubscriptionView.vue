<template>
  <div class="subscription-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🔗 订阅链接</span>
          <el-button type="primary" @click="refreshLinks" :loading="loading">
            🔄 刷新
          </el-button>
        </div>
      </template>

      <el-alert
        title="订阅链接说明"
        type="info"
        :closable="false"
        class="mb-4"
      >
        <p>订阅链接可用于客户端一键导入所有可用节点配置。</p>
        <p>支持 Clash、V2RayN、Surge 等主流客户端。</p>
      </el-alert>

      <!-- 入站选择 -->
      <el-form :inline="true" class="mb-4">
        <el-form-item label="选择入站">
          <el-select v-model="selectedInbound" placeholder="全部入站" @change="loadLinks">
            <el-option label="全部入站" value="all" />
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
        <el-table-column prop="name" label="名称" width="200" />
        <el-table-column prop="protocol" label="协议" width="100">
          <template #default="{ row }">
            <el-tag :type="getProtocolType(row.protocol)">
              {{ row.protocol }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="订阅链接" show-overflow-tooltip>
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
                📋 复制
              </el-button>
            </div>
          </template>
        </el-table-column>
        <el-table-column label="二维码" width="120" align="center">
          <template #default="{ row }">
            <el-button type="info" size="small" @click="showQRCode(row)">
              📱 查看
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- Base64 订阅 -->
      <el-divider content-position="left">📦 Base64 订阅</el-divider>
      <el-form-item label="Base64 订阅链接">
        <div class="link-container">
          <el-input
            :model-value="base64SubscriptionUrl"
            readonly
            placeholder="选择入站后生成"
          />
          <el-button
            type="primary"
            @click="copyLink(base64SubscriptionUrl)"
            :disabled="!base64SubscriptionUrl"
          >
            📋 复制
          </el-button>
        </div>
      </el-form-item>
    </el-card>

    <!-- 二维码对话框 -->
    <el-dialog v-model="qrDialogVisible" title="订阅二维码" width="300px">
      <div v-if="currentQRCode" class="qr-container">
        <img :src="currentQRCode" alt="QR Code" class="qr-image" />
      </div>
      <template #footer>
        <el-button @click="qrDialogVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import axios from 'axios'

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
    ElMessage.error('加载入站列表失败')
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
    ElMessage.error('加载订阅链接失败')
  } finally {
    loading.value = false
  }
}

// 刷新链接
const refreshLinks = () => {
  loadLinks()
  ElMessage.success('链接已刷新')
}

// 复制链接
const copyLink = (text: string) => {
  if (!text) {
    ElMessage.warning('没有可复制的链接')
    return
  }
  navigator.clipboard.writeText(text).then(() => {
    ElMessage.success('已复制到剪贴板')
  }).catch(() => {
    ElMessage.error('复制失败')
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
}

.mb-4 {
  margin-bottom: 20px;
}

.link-container {
  display: flex;
  gap: 10px;
  align-items: center;
}

.link-container .el-input {
  flex: 1;
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
</style>
