<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import axios from 'axios'

const { t } = useI18n()

interface AuditLog {
  id: string
  user_id?: string
  username?: string
  action: string
  resource_type?: string
  resource_id?: string
  ip_address?: string
  user_agent?: string
  request_method?: string
  request_path?: string
  status: string
  error_message?: string
  duration_ms?: number
  created_at: string
}

interface AuditStats {
  total_actions: number
  successful_actions: number
  failed_actions: number
  unique_users: number
  unique_ips: number
  top_actions: { action: string; count: number }[]
  actions_by_day: { date: string; count: number }[]
}

const loading = ref(false)
const logs = ref<AuditLog[]>([])
const stats = ref<AuditStats | null>(null)
const total = ref(0)
const page = ref(1)
const pageSize = ref(20)

// 过滤条件
const filters = ref({
  user_id: undefined as string | undefined,
  action: undefined as string | undefined,
  status: undefined as string | undefined,
  start_date: undefined as string | undefined,
  end_date: undefined as string | undefined
})

// 统计时间范围
const statsDays = ref(30)

// 加载审计日志
const loadLogs = async () => {
  loading.value = true
  try {
    const params: any = {
      page: page.value,
      page_size: pageSize.value
    }
    
    if (filters.value.user_id) params.user_id = filters.value.user_id
    if (filters.value.action) params.action = filters.value.action
    if (filters.value.status) params.status = filters.value.status
    if (filters.value.start_date) params.start_date = filters.value.start_date
    if (filters.value.end_date) params.end_date = filters.value.end_date

    const res = await axios.get('/api/audit/logs', { params })
    logs.value = res.data.logs || []
    total.value = res.data.total || 0
  } catch (error) {
    ElMessage.error(t('audit.loadFailed'))
  } finally {
    loading.value = false
  }
}

// 加载统计信息
const loadStats = async () => {
  try {
    const res = await axios.get('/api/audit/stats', {
      params: { days: statsDays.value }
    })
    stats.value = res.data
  } catch (error) {
    console.error('Failed to load stats:', error)
  }
}

// 搜索
const handleSearch = () => {
  page.value = 1
  loadLogs()
}

// 重置过滤
const resetFilters = () => {
  filters.value = {
    user_id: undefined,
    action: undefined,
    status: undefined,
    start_date: undefined,
    end_date: undefined
  }
  page.value = 1
  loadLogs()
}

// 分页变化
const handlePageChange = (newPage: number) => {
  page.value = newPage
  loadLogs()
}

// 获取状态标签类型
const getStatusType = (status: string) => {
  const types: Record<string, 'success' | 'danger' | 'warning' | 'info'> = {
    'success': 'success',
    'failure': 'danger',
    'error': 'danger'
  }
  return types[status] || 'info'
}

// 获取操作类型图标
const getActionIcon = (action: string) => {
  const icons: Record<string, string> = {
    'create': '➕',
    'update': '✏️',
    'delete': '🗑️',
    'login': '🔑',
    'logout': '🚪',
    'sync': '🔄',
    'export': '📥',
    'import': '📤'
  }
  return icons[action] || '📝'
}

// 格式化时间
const formatTime = (date: string) => {
  return new Date(date).toLocaleString()
}

// 格式化持续时间
const formatDuration = (ms?: number) => {
  if (ms === undefined || ms === null) return '-'
  if (ms < 1000) return `${ms}ms`
  return `${(ms / 1000).toFixed(2)}s`
}

// 导出日志
const exportLogs = async () => {
  try {
    const res = await axios.get('/api/audit/logs', {
      params: { ...filters.value, page: 1, page_size: 1000 },
      responseType: 'json'
    })
    
    const csv = [
      ['时间', '用户', '操作', '资源类型', '资源 ID', 'IP 地址', '状态', '持续时间', '错误信息'].join(','),
      ...res.data.logs.map((log: AuditLog) => [
        log.created_at,
        log.username || '-',
        log.action,
        log.resource_type || '-',
        log.resource_id || '-',
        log.ip_address || '-',
        log.status,
        log.duration_ms?.toString() || '-',
        log.error_message || '-'
      ].map(field => `"${field}"`).join(','))
    ].join('\n')
    
    const blob = new Blob([csv], { type: 'text/csv' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `audit-logs-${new Date().toISOString().split('T')[0]}.csv`
    a.click()
    URL.revokeObjectURL(url)
    
    ElMessage.success(t('audit.exportSuccess'))
  } catch (error) {
    ElMessage.error(t('audit.exportFailed'))
  }
}

// 清理旧日志
const cleanupLogs = async () => {
  try {
    await axios.post('/api/audit/cleanup', null, {
      params: { days: 90 }
    })
    ElMessage.success(t('audit.cleanupSuccess'))
    loadLogs()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('audit.cleanupFailed'))
  }
}

onMounted(() => {
  loadLogs()
  loadStats()
})
</script>

<template>
  <div class="audit-logs-view">
    <el-card class="box-card mb-4">
      <template #header>
        <div class="card-header">
          <span>📋 {{ t('audit.title') }}</span>
          <div class="header-actions">
            <el-button type="success" @click="exportLogs">
              📥 {{ t('audit.export') }}
            </el-button>
            <el-button type="warning" @click="cleanupLogs">
              🧹 {{ t('audit.cleanup') }}
            </el-button>
          </div>
        </div>
      </template>

      <!-- 统计卡片 -->
      <el-row :gutter="20" v-if="stats">
        <el-col :span="6">
          <el-statistic :title="t('audit.totalActions')" :value="stats.total_actions" />
        </el-col>
        <el-col :span="6">
          <el-statistic :title="t('audit.successfulActions')" :value="stats.successful_actions">
            <template #suffix>
              <el-tag type="success" size="small">✅</el-tag>
            </template>
          </el-statistic>
        </el-col>
        <el-col :span="6">
          <el-statistic :title="t('audit.failedActions')" :value="stats.failed_actions">
            <template #suffix>
              <el-tag type="danger" size="small">❌</el-tag>
            </template>
          </el-statistic>
        </el-col>
        <el-col :span="6">
          <el-statistic :title="t('audit.uniqueUsers')" :value="stats.unique_users" />
        </el-col>
      </el-row>
    </el-card>

    <!-- 过滤条件 -->
    <el-card class="box-card mb-4">
      <el-form :inline="true" :model="filters" class="filter-form">
        <el-form-item :label="t('audit.action')">
          <el-input v-model="filters.action" :placeholder="t('audit.actionPlaceholder')" clearable />
        </el-form-item>

        <el-form-item :label="t('audit.status')">
          <el-select v-model="filters.status" :placeholder="t('audit.allStatus')" clearable>
            <el-option label="成功" value="success" />
            <el-option label="失败" value="failure" />
            <el-option label="错误" value="error" />
          </el-select>
        </el-form-item>

        <el-form-item :label="t('audit.startDate')">
          <el-date-picker
            v-model="filters.start_date"
            type="datetime"
            :placeholder="t('audit.startDatePlaceholder')"
            value-format="YYYY-MM-DD HH:mm:ss"
          />
        </el-form-item>

        <el-form-item :label="t('audit.endDate')">
          <el-date-picker
            v-model="filters.end_date"
            type="datetime"
            :placeholder="t('audit.endDatePlaceholder')"
            value-format="YYYY-MM-DD HH:mm:ss"
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="handleSearch">{{ t('audit.search') }}</el-button>
          <el-button @click="resetFilters">{{ t('audit.reset') }}</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 日志列表 -->
    <el-card class="box-card">
      <el-table :data="logs" stripe style="width: 100%" v-loading="loading">
        <el-table-column prop="created_at" :label="t('audit.time')" width="180">
          <template #default="{ row }">
            {{ formatTime(row.created_at) }}
          </template>
        </el-table-column>

        <el-table-column prop="username" :label="t('audit.user')" width="120">
          <template #default="{ row }">
            {{ row.username || 'system' }}
          </template>
        </el-table-column>

        <el-table-column prop="action" :label="t('audit.action')" width="120">
          <template #default="{ row }">
            <span>{{ getActionIcon(row.action) }} {{ row.action }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="resource_type" :label="t('audit.resource')" width="120">
          <template #default="{ row }">
            {{ row.resource_type || '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="ip_address" :label="t('audit.ipAddress')" width="140">
          <template #default="{ row }">
            {{ row.ip_address || '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="request_path" :label="t('audit.path')" min-width="200" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.request_path || '-' }}
          </template>
        </el-table-column>

        <el-table-column prop="status" :label="t('audit.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ row.status === 'success' ? '✅ 成功' : row.status === 'failure' ? '❌ 失败' : '⚠️ 错误' }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="duration_ms" :label="t('audit.duration')" width="90" align="right">
          <template #default="{ row }">
            {{ formatDuration(row.duration_ms) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('audit.error')" min-width="150" show-overflow-tooltip>
          <template #default="{ row }">
            <span v-if="row.error_message" class="text-danger">{{ row.error_message }}</span>
            <span v-else class="text-muted">-</span>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-container">
        <el-pagination
          v-model:current-page="page"
          :page-size="pageSize"
          :total="total"
          :page-sizes="[20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next, jumper"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<style scoped>
.audit-logs-view {
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

.filter-form {
  margin-bottom: 0;
}

.pagination-container {
  margin-top: 20px;
  display: flex;
  justify-content: flex-end;
}

.text-danger {
  color: #f56c6c;
}

.text-muted {
  color: #909399;
}

.mb-4 {
  margin-bottom: 16px;
}
</style>
