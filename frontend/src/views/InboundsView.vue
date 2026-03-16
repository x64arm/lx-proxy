<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { Inbound } from '../api'
import { inboundsAPI } from '../api'

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
    ElMessage.error('获取入站列表失败：' + (error.message || '未知错误'))
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
      ElMessage.success('入站配置更新成功')
    } else {
      await inboundsAPI.create(form.value)
      ElMessage.success('入站配置创建成功')
    }
    dialogVisible.value = false
    fetchInbounds()
  } catch (error: any) {
    ElMessage.error(editingInbound.value ? '更新失败' : '创建失败' + ': ' + (error.response?.data || error.message))
  }
}

const handleDelete = async (inbound: Inbound) => {
  try {
    await ElMessageBox.confirm(`确定要删除入站 "${inbound.remark}" 吗？`, '确认删除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await inboundsAPI.delete(inbound.id)
    ElMessage.success('删除成功')
    fetchInbounds()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败：' + (error.message || '未知错误'))
    }
  }
}

const handleReset = async (inbound: Inbound) => {
  try {
    await ElMessageBox.confirm(`确定要重置 "${inbound.remark}" 的流量统计吗？`, '确认重置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await inboundsAPI.reset(inbound.id)
    ElMessage.success('流量重置成功')
    fetchInbounds()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('重置失败：' + (error.message || '未知错误'))
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
  if (!timestamp) return '永久有效'
  return new Date(timestamp).toLocaleDateString('zh-CN')
}

onMounted(() => {
  fetchInbounds()
})
</script>

<template>
  <div class="inbounds-page">
    <div class="header">
      <h1>📡 入站配置</h1>
      <el-button type="primary" @click="openCreateDialog">
        <el-icon><Plus /></el-icon>
        添加入站
      </el-button>
    </div>

    <el-card class="table-card">
      <el-table :data="inbounds" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="remark" label="备注" min-width="150" />
        <el-table-column prop="protocol" label="协议" width="120">
          <template #default="{ row }">
            <el-tag>{{ row.protocol.toUpperCase() }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="port" label="端口" width="100" />
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.enable ? 'success' : 'danger'">
              {{ row.enable ? '已启用' : '已禁用' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="流量" width="150">
          <template #default="{ row }">
            {{ formatBytes(row.up + row.down) }}
            <span v-if="row.total > 0" class="traffic-limit">
              / {{ formatBytes(row.total) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="到期时间" width="120">
          <template #default="{ row }">
            {{ formatExpiry(row.expiry_time) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="280" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEditDialog(row)">
              编辑
            </el-button>
            <el-button size="small" @click="handleReset(row)">
              重置流量
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑入站对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingInbound ? '编辑入站' : '添加入站'"
      width="600px"
    >
      <el-form :model="form" label-width="100px">
        <el-form-item label="协议" required>
          <el-select v-model="form.protocol" style="width: 100%">
            <el-option
              v-for="opt in protocolOptions"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="端口" required>
          <el-input-number v-model="form.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" placeholder="可选，用于标识此入站" />
        </el-form-item>
        <el-form-item label="启用" required>
          <el-switch v-model="form.enable" />
        </el-form-item>
        <el-form-item label="流量上限 (字节)">
          <el-input-number v-model="form.total" :min="0" style="width: 100%" />
          <div class="form-tip">0 表示无限制</div>
        </el-form-item>
        <el-form-item label="到期时间">
          <el-date-picker
            v-model="form.expiry_time"
            type="datetime"
            placeholder="选择到期时间"
            style="width: 100%"
            clearable
          />
          <div class="form-tip">留空表示永久有效</div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSubmit">确定</el-button>
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

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.header h1 {
  font-size: 1.5rem;
  color: #333;
  margin: 0;
}

.table-card {
  margin-bottom: 1rem;
}

.traffic-limit {
  color: #999;
  font-size: 0.9em;
}

.form-tip {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}
</style>
