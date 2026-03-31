<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'
import axios from 'axios'

const { t } = useI18n()

interface Node {
  id: string
  name: string
  description?: string
  api_url: string
  status: string
  location?: string
  version?: string
  cpu_usage?: number
  memory_usage?: number
  disk_usage?: number
  connection_count?: number
  last_seen?: string
  is_primary: boolean
  is_active: boolean
  sync_status: string
  last_sync_at?: string
  created_at: string
  updated_at: string
}

const loading = ref(false)
const nodes = ref<Node[]>([])
const dialogVisible = ref(false)
const editMode = ref(false)
const currentNode = ref<Partial<Node>>({})

const stats = ref({
  total: 0,
  online: 0,
  offline: 0
})

// 表单验证规则
const rules = {
  name: [
    { required: true, message: '请输入节点名称', trigger: 'blur' },
    { min: 2, max: 50, message: '长度在 2 到 50 个字符', trigger: 'blur' }
  ],
  api_url: [
    { required: true, message: '请输入 API 地址', trigger: 'blur' },
    { type: 'url', message: '请输入有效的 URL', trigger: 'blur' }
  ],
  api_key: [
    { required: true, message: '请输入 API 密钥', trigger: 'blur' }
  ]
}

// 加载节点列表
const loadNodes = async () => {
  loading.value = true
  try {
    const res = await axios.get('/api/nodes')
    nodes.value = res.data.nodes || []
    stats.value = {
      total: res.data.total || 0,
      online: res.data.online_count || 0,
      offline: res.data.offline_count || 0
    }
  } catch (error) {
    ElMessage.error(t('nodes.loadFailed'))
  } finally {
    loading.value = false
  }
}

// 打开创建对话框
const openCreateDialog = () => {
  editMode.value = false
  currentNode.value = {
    is_active: true,
    is_primary: false
  }
  dialogVisible.value = true
}

// 打开编辑对话框
const openEditDialog = (node: Node) => {
  editMode.value = true
  currentNode.value = { ...node }
  dialogVisible.value = true
}

// 保存节点
const saveNode = async () => {
  try {
    if (editMode.value && currentNode.value.id) {
      await axios.put(`/api/nodes/${currentNode.value.id}`, currentNode.value)
      ElMessage.success(t('nodes.updateSuccess'))
    } else {
      await axios.post('/api/nodes', currentNode.value)
      ElMessage.success(t('nodes.createSuccess'))
    }
    dialogVisible.value = false
    loadNodes()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('nodes.saveFailed'))
  }
}

// 删除节点
const deleteNode = async (node: Node) => {
  try {
    await ElMessageBox.confirm(
      t('nodes.deleteConfirm', { name: node.name }),
      t('nodes.deleteTitle'),
      { type: 'warning' }
    )
    
    await axios.delete(`/api/nodes/${node.id}`)
    ElMessage.success(t('nodes.deleteSuccess'))
    loadNodes()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.response?.data?.message || t('nodes.deleteFailed'))
    }
  }
}

// 检查节点健康
const checkHealth = async (node: Node) => {
  try {
    const res = await axios.post(`/api/nodes/${node.id}/health`)
    const status = res.data.healthy ? 'online' : 'offline'
    node.status = status
    ElMessage.success(t('nodes.healthCheckSuccess'))
    loadNodes()
  } catch (error: any) {
    ElMessage.error(t('nodes.healthCheckFailed'))
  }
}

// 同步配置到节点
const syncNode = async (node: Node) => {
  try {
    const configData = { sync_type: 'full' }
    await axios.post(`/api/nodes/${node.id}/sync`, configData)
    ElMessage.success(t('nodes.syncSuccess'))
    loadNodes()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('nodes.syncFailed'))
  }
}

// 批量同步到所有节点
const batchSync = async () => {
  try {
    await ElMessageBox.confirm(
      t('nodes.batchSyncConfirm'),
      t('nodes.batchSyncTitle'),
      { type: 'warning' }
    )
    
    const res = await axios.post('/api/nodes/batch/sync', { sync_type: 'full' })
    const success = res.data.successful
    const failed = res.data.failed
    
    ElMessage.success(t('nodes.batchSyncSuccess', { success, failed }))
    loadNodes()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('nodes.batchSyncFailed'))
    }
  }
}

// 获取状态标签类型
const getStatusType = (status: string) => {
  const types: Record<string, 'success' | 'danger' | 'warning' | 'info'> = {
    'online': 'success',
    'offline': 'danger',
    'error': 'warning'
  }
  return types[status] || 'info'
}

// 获取同步状态标签类型
const getSyncStatusType = (status: string) => {
  const types: Record<string, 'success' | 'warning' | 'info' | 'danger'> = {
    'synced': 'success',
    'syncing': 'warning',
    'pending': 'info',
    'failed': 'danger'
  }
  return types[status] || 'info'
}

// 格式化最后活跃时间
const formatLastSeen = (date?: string) => {
  if (!date) return '从未'
  const d = new Date(date)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  const minutes = Math.floor(diff / 60000)
  
  if (minutes < 1) return '刚刚'
  if (minutes < 60) return `${minutes}分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}小时前`
  return d.toLocaleDateString()
}

onMounted(() => {
  loadNodes()
})
</script>

<template>
  <div class="nodes-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🌐 {{ t('nodes.title') }}</span>
          <div class="header-actions">
            <el-button type="primary" @click="batchSync" :loading="loading">
              🔄 {{ t('nodes.batchSync') }}
            </el-button>
            <el-button type="success" @click="openCreateDialog">
              ➕ {{ t('nodes.addNode') }}
            </el-button>
          </div>
        </div>
      </template>

      <!-- 统计卡片 -->
      <el-row :gutter="20" class="mb-4">
        <el-col :span="8">
          <el-statistic :title="t('nodes.totalNodes')" :value="stats.total" />
        </el-col>
        <el-col :span="8">
          <el-statistic :title="t('nodes.onlineNodes')" :value="stats.online">
            <template #suffix>
              <el-tag type="success" size="small">在线</el-tag>
            </template>
          </el-statistic>
        </el-col>
        <el-col :span="8">
          <el-statistic :title="t('nodes.offlineNodes')" :value="stats.offline">
            <template #suffix>
              <el-tag type="danger" size="small">离线</el-tag>
            </template>
          </el-statistic>
        </el-col>
      </el-row>

      <!-- 节点列表 -->
      <el-table :data="nodes" stripe style="width: 100%" v-loading="loading">
        <el-table-column prop="name" :label="t('nodes.name')" width="200">
          <template #default="{ row }">
            <div>
              <strong>{{ row.name }}</strong>
              <el-tag v-if="row.is_primary" type="danger" size="small" class="ml-2">主节点</el-tag>
              <el-tag v-if="!row.is_active" type="info" size="small" class="ml-2">禁用</el-tag>
            </div>
            <div class="text-secondary">{{ row.description || '-' }}</div>
          </template>
        </el-table-column>

        <el-table-column prop="status" :label="t('nodes.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ row.status === 'online' ? '🟢 在线' : row.status === 'offline' ? '🔴 离线' : '⚠️ 异常' }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="location" :label="t('nodes.location')" width="120">
          <template #default="{ row }">
            {{ row.location || '-' }}
          </template>
        </el-table-column>

        <el-table-column :label="t('nodes.resources')" width="180">
          <template #default="{ row }">
            <div v-if="row.cpu_usage !== null">
              <el-progress :percentage="row.cpu_usage" :stroke-width="6" :format="(p: number) => `CPU ${p}%`" />
            </div>
            <div v-if="row.memory_usage !== null" class="mt-1">
              <el-progress :percentage="row.memory_usage" :stroke-width="6" :format="(p: number) => `MEM ${p}%`" />
            </div>
          </template>
        </el-table-column>

        <el-table-column prop="connection_count" :label="t('nodes.connections')" width="100" align="center">
          <template #default="{ row }">
            {{ row.connection_count || 0 }}
          </template>
        </el-table-column>

        <el-table-column prop="sync_status" :label="t('nodes.syncStatus')" width="100">
          <template #default="{ row }">
            <el-tag :type="getSyncStatusType(row.sync_status)" size="small">
              {{ row.sync_status }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column :label="t('nodes.lastSeen')" width="120">
          <template #default="{ row }">
            {{ formatLastSeen(row.last_seen) }}
          </template>
        </el-table-column>

        <el-table-column :label="t('nodes.operations')" width="280" fixed="right">
          <template #default="{ row }">
            <el-button
              type="info"
              size="small"
              @click="checkHealth(row)"
            >
              🏥 {{ t('nodes.healthCheck') }}
            </el-button>
            <el-button
              type="primary"
              size="small"
              @click="syncNode(row)"
            >
              🔄 {{ t('nodes.sync') }}
            </el-button>
            <el-button
              type="warning"
              size="small"
              @click="openEditDialog(row)"
            >
              ✏️ {{ t('common.edit') }}
            </el-button>
            <el-button
              type="danger"
              size="small"
              @click="deleteNode(row)"
              :disabled="row.is_primary"
            >
              🗑️ {{ t('common.delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑节点对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editMode ? t('nodes.editNode') : t('nodes.addNode')"
      width="600px"
    >
      <el-form :model="currentNode" :rules="rules" ref="formRef" label-width="100px">
        <el-form-item :label="t('nodes.name')" prop="name">
          <el-input v-model="currentNode.name" :placeholder="t('nodes.namePlaceholder')" />
        </el-form-item>

        <el-form-item :label="t('nodes.description')" prop="description">
          <el-input
            v-model="currentNode.description"
            type="textarea"
            :rows="2"
            :placeholder="t('nodes.descriptionPlaceholder')"
          />
        </el-form-item>

        <el-form-item :label="t('nodes.apiUrl')" prop="api_url">
          <el-input v-model="currentNode.api_url" placeholder="https://node1.example.com:8080" />
        </el-form-item>

        <el-form-item :label="t('nodes.apiKey')" prop="api_key">
          <el-input
            v-model="currentNode.api_key"
            type="password"
            show-password
            :placeholder="t('nodes.apiKeyPlaceholder')"
          />
        </el-form-item>

        <el-form-item :label="t('nodes.location')" prop="location">
          <el-input v-model="currentNode.location" placeholder="例如：北京、上海、东京" />
        </el-form-item>

        <el-form-item :label="t('nodes.settings')">
          <el-checkbox v-model="currentNode.is_active">{{ t('nodes.enableNode') }}</el-checkbox>
          <el-checkbox v-model="currentNode.is_primary" :disabled="editMode">{{ t('nodes.setAsPrimary') }}</el-checkbox>
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="saveNode">{{ t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.nodes-view {
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

.text-secondary {
  color: #909399;
  font-size: 12px;
}

.mt-1 {
  margin-top: 4px;
}

.ml-2 {
  margin-left: 8px;
}
</style>
