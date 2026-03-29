<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from 'vue-i18n'
import type { Inbound } from '../api'
import { inboundsAPI } from '../api'
import { Plus, Edit, Delete, Refresh, MoreFilled } from '@element-plus/icons-vue'

const { t } = useI18n()

interface InboundForm {
  protocol: 'vmess' | 'vless' | 'trojan' | 'shadowsocks'
  port: number
  enable: boolean
  remark: string
  total: number
  expiry_time: number | null
}

const inbounds = ref<Inbound[]>([])
const loading = ref(true)
const dialogVisible = ref(false)
const editingInbound = ref<Inbound | null>(null)
const form = ref<InboundForm>({
  protocol: 'vmess',
  port: 1080,
  enable: true,
  remark: '',
  total: 0,
  expiry_time: null
})

// 响应式状态
const isMobile = ref(false)
const viewMode = ref<'table' | 'card'>('table')

// 检测设备类型
const checkDevice = () => {
  const width = window.innerWidth
  isMobile.value = width <= 768
  viewMode.value = isMobile.value ? 'card' : 'table'
}

// 监听窗口大小变化
onMounted(() => {
  checkDevice()
  window.addEventListener('resize', checkDevice)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkDevice)
})

const protocolOptions = [
  { label: 'Vmess', value: 'vmess' },
  { label: 'Vless', value: 'vless' },
  { label: 'Trojan', value: 'trojan' },
  { label: 'Shadowsocks', value: 'shadowsocks' }
]

const fetchInbounds = async () => {
  try {
    const response = await inboundsAPI.list()
    inbounds.value = response.data
  } catch (error: any) {
    ElMessage.error(t('inbounds.fetchFailed') + ': ' + (error.message || t('common.error')))
  } finally {
    loading.value = false
  }
}

const openCreateDialog = () => {
  editingInbound.value = null
  form.value = {
    protocol: 'vmess',
    port: Math.floor(Math.random() * 10000) + 10000,
    enable: true,
    remark: '',
    total: 0,
    expiry_time: null
  }
  dialogVisible.value = true
}

const openEditDialog = (inbound: Inbound) => {
  editingInbound.value = inbound
  form.value = {
    protocol: inbound.protocol,
    port: inbound.port,
    enable: inbound.enable,
    remark: inbound.remark,
    total: inbound.total,
    expiry_time: inbound.expiry_time
  }
  dialogVisible.value = true
}

const handleSubmit = async () => {
  try {
    if (editingInbound.value) {
      await inboundsAPI.update(editingInbound.value.id, form.value)
      ElMessage.success(t('inbounds.updateSuccess'))
    } else {
      await inboundsAPI.create(form.value)
      ElMessage.success(t('inbounds.createSuccess'))
    }
    dialogVisible.value = false
    fetchInbounds()
  } catch (error: any) {
    const baseMsg = editingInbound.value ? t('inbounds.updateFailed') : t('inbounds.createFailed')
    ElMessage.error(baseMsg + ': ' + (error.response?.data || error.message))
  }
}

const handleDelete = async (inbound: Inbound) => {
  try {
    await ElMessageBox.confirm(
      t('inbounds.deleteConfirm', { remark: inbound.remark }),
      t('inbounds.confirmDelete'),
      {
        confirmButtonText: t('common.confirm'),
        cancelButtonText: t('common.cancel'),
        type: 'warning'
      }
    )
    await inboundsAPI.delete(inbound.id)
    ElMessage.success(t('inbounds.deleteSuccess'))
    fetchInbounds()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('inbounds.deleteFailed') + ': ' + (error.message || t('common.error')))
    }
  }
}

const handleReset = async (inbound: Inbound) => {
  try {
    await ElMessageBox.confirm(
      t('inbounds.resetConfirm', { remark: inbound.remark }),
      t('inbounds.confirmReset'),
      {
        confirmButtonText: t('common.confirm'),
        cancelButtonText: t('common.cancel'),
        type: 'warning'
      }
    )
    await inboundsAPI.reset(inbound.id)
    ElMessage.success(t('inbounds.resetSuccess'))
    fetchInbounds()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('inbounds.resetFailed') + ': ' + (error.message || t('common.error')))
    }
  }
}

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatExpiry = (timestamp: number | null) => {
  if (!timestamp) return t('inbounds.permanent')
  return new Date(timestamp).toLocaleDateString()
}

onMounted(() => {
  fetchInbounds()
})
</script>

<template>
  <div class="inbounds-page" :class="{ 'mobile-page': isMobile }">
    <div class="header">
      <h1>📡 {{ t('inbounds.title') }}</h1>
      <div class="header-actions">
        <!-- 视图切换按钮（移动端） -->
        <el-button 
          v-if="isMobile" 
          circle 
          @click="viewMode = viewMode === 'table' ? 'card' : 'table'"
          class="view-toggle-btn"
        >
          <el-icon>
            <MoreFilled v-if="viewMode === 'table'" />
            <Edit v-else />
          </el-icon>
        </el-button>
        
        <el-button type="primary" @click="openCreateDialog" class="add-btn">
          <el-icon><Plus /></el-icon>
          <span v-if="!isMobile">{{ t('inbounds.addInbound') }}</span>
        </el-button>
      </div>
    </div>

    <!-- 表格视图 -->
    <el-card class="table-card" v-if="viewMode === 'table'">
      <div class="table-wrapper" :class="{ 'mobile-table': isMobile }">
        <el-table :data="inbounds" v-loading="loading" stripe>
          <el-table-column prop="id" :label="t('inbounds.id')" width="70" />
          <el-table-column prop="remark" :label="t('inbounds.remark')" min-width="120" />
          <el-table-column prop="protocol" :label="t('inbounds.protocol')" width="100">
            <template #default="{ row }">
              <el-tag size="small">{{ row.protocol.toUpperCase() }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="port" :label="t('inbounds.port')" width="80" />
          <el-table-column :label="t('inbounds.status')" width="90">
            <template #default="{ row }">
              <el-tag :type="row.enable ? 'success' : 'danger'" size="small">
                {{ row.enable ? t('inbounds.enabled') : t('inbounds.disabled') }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column :label="t('inbounds.trafficUsed')" width="140">
            <template #default="{ row }">
              <div class="traffic-display">
                {{ formatBytes(row.up + row.down) }}
                <span v-if="row.total > 0" class="traffic-limit">
                  / {{ formatBytes(row.total) }}
                </span>
              </div>
            </template>
          </el-table-column>
          <el-table-column :label="t('inbounds.expiryTime')" width="110">
            <template #default="{ row }">
              {{ formatExpiry(row.expiry_time) }}
            </template>
          </el-table-column>
          <el-table-column :label="t('inbounds.actions')" width="240" fixed="right">
            <template #default="{ row }">
              <div class="action-buttons" :class="{ 'mobile-actions': isMobile }">
                <el-button size="small" @click="openEditDialog(row)">
                  {{ t('inbounds.edit') }}
                </el-button>
                <el-button size="small" @click="handleReset(row)">
                  {{ t('inbounds.resetTraffic') }}
                </el-button>
                <el-button size="small" type="danger" @click="handleDelete(row)">
                  {{ t('inbounds.delete') }}
                </el-button>
              </div>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>

    <!-- 卡片视图（移动端） -->
    <div class="cards-container" v-if="viewMode === 'card'">
      <el-card 
        v-for="inbound in inbounds" 
        :key="inbound.id" 
        shadow="hover"
        class="inbound-card"
        v-loading="loading"
      >
        <div class="card-header">
          <div class="card-header-left">
            <div class="card-id">#{{ inbound.id }}</div>
            <div class="card-remark">{{ inbound.remark || t('inbounds.noRemark') }}</div>
          </div>
          <div class="card-header-right">
            <el-tag :type="inbound.enable ? 'success' : 'danger'" size="small">
              {{ inbound.enable ? t('inbounds.enabled') : t('inbounds.disabled') }}
            </el-tag>
            <el-tag size="small">{{ inbound.protocol.toUpperCase() }}</el-tag>
          </div>
        </div>
        
        <div class="card-body">
          <div class="card-row">
            <span class="card-label">🔌 {{ t('inbounds.port') }}：</span>
            <span class="card-value">{{ inbound.port }}</span>
          </div>
          <div class="card-row">
            <span class="card-label">📊 {{ t('inbounds.trafficUsed') }}：</span>
            <span class="card-value">
              {{ formatBytes(inbound.up + inbound.down) }}
              <span v-if="inbound.total > 0" class="traffic-limit">
                / {{ formatBytes(inbound.total) }}
              </span>
            </span>
          </div>
          <div class="card-row">
            <span class="card-label">📅 {{ t('inbounds.expiryTime') }}：</span>
            <span class="card-value">{{ formatExpiry(inbound.expiry_time) }}</span>
          </div>
        </div>
        
        <div class="card-actions">
          <el-button type="primary" @click="openEditDialog(inbound)" class="action-btn">
            <el-icon><Edit /></el-icon>
            {{ t('inbounds.edit') }}
          </el-button>
          <el-button @click="handleReset(inbound)" class="action-btn">
            <el-icon><Refresh /></el-icon>
            {{ t('inbounds.resetTraffic') }}
          </el-button>
          <el-button type="danger" @click="handleDelete(inbound)" class="action-btn">
            <el-icon><Delete /></el-icon>
            {{ t('inbounds.delete') }}
          </el-button>
        </div>
      </el-card>
      
      <!-- 空状态 -->
      <el-empty v-if="inbounds.length === 0 && !loading" :description="t('inbounds.noData')" />
    </div>

    <!-- 添加/编辑入站对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingInbound ? t('inbounds.editInbound') : t('inbounds.addInbound')"
      width="600px"
    >
      <el-form :model="form" label-width="100px">
        <el-form-item :label="t('inbounds.protocol')" required>
          <el-select v-model="form.protocol" style="width: 100%">
            <el-option
              v-for="opt in protocolOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item :label="t('inbounds.port')" required>
          <el-input-number v-model="form.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
        <el-form-item :label="t('inbounds.remark')">
          <el-input v-model="form.remark" :placeholder="t('inbounds.remarkPlaceholder')" />
        </el-form-item>
        <el-form-item :label="t('inbounds.enable')" required>
          <el-switch v-model="form.enable" />
        </el-form-item>
        <el-form-item :label="t('inbounds.trafficLimit')">
          <el-input-number v-model="form.total" :min="0" style="width: 100%" />
          <div class="form-tip">{{ t('inbounds.trafficLimitHint') }}</div>
        </el-form-item>
        <el-form-item :label="t('inbounds.expiryTime')">
          <el-date-picker
            v-model="form.expiry_time"
            type="datetime"
            :placeholder="t('inbounds.selectExpiryTime')"
            style="width: 100%"
            clearable
          />
          <div class="form-tip">{{ t('inbounds.expiryHint') }}</div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('inbounds.cancel') }}</el-button>
        <el-button type="primary" @click="handleSubmit">{{ t('inbounds.submit') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.inbounds-page {
  padding: 2rem;
  max-width: 1400px;
  margin: 0 auto;
}

.mobile-page {
  padding: 12px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
  flex-wrap: wrap;
  gap: 12px;
}

.header h1 {
  font-size: 1.5rem;
  color: #333;
  margin: 0;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.view-toggle-btn {
  width: 40px;
  height: 40px;
  padding: 0;
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* 表格包装器 - 支持横向滚动 */
.table-wrapper {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}

.mobile-table {
  margin: 0 -20px;
}

.mobile-table :deep(.el-table) {
  min-width: 900px;
}

/* 移动端操作按钮优化 */
.action-buttons {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.mobile-actions {
  flex-direction: column;
  gap: 8px;
}

.mobile-actions .el-button {
  width: 100%;
}

/* 流量显示 */
.traffic-display {
  white-space: nowrap;
}

.traffic-limit {
  color: #999;
  font-size: 0.85em;
}

/* ========== 卡片视图样式 ========== */
.cards-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.mobile-page .cards-container {
  grid-template-columns: 1fr;
}

.inbound-card {
  transition: transform 0.3s, box-shadow 0.3s;
}

.inbound-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
}

.card-header-left {
  flex: 1;
}

.card-header-right {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.card-id {
  font-size: 12px;
  color: #909399;
  margin-bottom: 4px;
}

.card-remark {
  font-size: 15px;
  color: #303133;
  font-weight: 500;
}

.card-body {
  margin-bottom: 16px;
}

.card-row {
  display: flex;
  flex-direction: column;
  margin-bottom: 12px;
  gap: 4px;
}

.card-label {
  font-size: 12px;
  color: #909399;
  font-weight: 500;
}

.card-value {
  font-size: 14px;
  color: #303133;
}

.card-actions {
  display: flex;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
}

.card-actions .action-btn {
  flex: 1;
  min-height: 44px;
}

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .inbounds-page {
    padding: 12px;
  }

  .header h1 {
    font-size: 1.25rem;
  }

  .table-card {
    margin: 0 -12px 1rem;
    border-radius: 0;
  }

  .table-card :deep(.el-card__body) {
    padding: 12px;
  }

  .mobile-table :deep(.el-table) {
    font-size: 13px;
  }

  .mobile-table :deep(.el-table th) {
    padding: 8px 0;
    font-size: 12px;
  }

  .mobile-table :deep(.el-table td) {
    padding: 10px 0;
  }

  /* 卡片视图优化 */
  .cards-container {
    margin: 0 -12px;
  }

  .inbound-card {
    margin: 0 12px 12px;
    border-radius: 12px;
  }

  .inbound-card :deep(.el-card__body) {
    padding: 16px;
  }

  .card-remark {
    font-size: 14px;
  }

  .card-label {
    font-size: 11px;
  }

  .card-value {
    font-size: 13px;
  }

  .card-actions .action-btn {
    min-height: 48px;
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

.inbounds-page {
  max-width: 100vw;
  overflow-x: hidden;
}

.form-tip {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}
</style>
