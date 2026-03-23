<template>
  <div class="batch-actions" v-if="selectedItems.length > 0">
    <el-card shadow="hover" class="batch-card">
      <div class="batch-header">
        <el-checkbox
          :model-value="allSelected"
          @change="toggleSelectAll"
        >
          已选择 {{ selectedItems.length }} 项
        </el-checkbox>
        
        <div class="batch-buttons">
          <slot name="actions">
            <!-- 默认批量操作按钮 -->
            <el-button
              v-if="showEnable"
              type="success"
              size="small"
              @click="handleBatchEnable"
            >
              批量启用
            </el-button>
            
            <el-button
              v-if="showDisable"
              type="warning"
              size="small"
              @click="handleBatchDisable"
            >
              批量禁用
            </el-button>
            
            <el-button
              v-if="showDelete"
              type="danger"
              size="small"
              @click="handleBatchDelete"
            >
              批量删除
            </el-button>
            
            <el-button
              v-if="showExport"
              type="primary"
              size="small"
              @click="handleBatchExport"
            >
              批量导出
            </el-button>
          </slot>
          
          <el-button size="small" @click="clearSelection">
            取消选择
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

interface BatchActionsProps {
  selectedItems: any[]
  allSelected: boolean
  showEnable?: boolean
  showDisable?: boolean
  showDelete?: boolean
  showExport?: boolean
}

const props = withDefaults(defineProps<BatchActionsProps>(), {
  showEnable: true,
  showDisable: true,
  showDelete: true,
  showExport: true,
})

const emit = defineEmits<{
  'update:selectedItems': [value: any[]]
  'select-all': []
  'batch-enable': [ids: string[]]
  'batch-disable': [ids: string[]]
  'batch-delete': [ids: string[]]
  'batch-export': [ids: string[]]
}>()

// 切换全选
const toggleSelectAll = () => {
  emit('select-all')
}

// 清空选择
const clearSelection = () => {
  emit('update:selectedItems', [])
}

// 批量启用
const handleBatchEnable = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要启用选中的 ${selectedItems.value.length} 项吗？`,
      '批量启用',
      { type: 'warning' }
    )
    
    const ids = selectedItems.value.map(item => item.id)
    emit('batch-enable', ids)
    ElMessage.success('批量启用成功')
  } catch {
    // 取消操作
  }
}

// 批量禁用
const handleBatchDisable = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要禁用选中的 ${selectedItems.value.length} 项吗？`,
      '批量禁用',
      { type: 'warning' }
    )
    
    const ids = selectedItems.value.map(item => item.id)
    emit('batch-disable', ids)
    ElMessage.success('批量禁用成功')
  } catch {
    // 取消操作
  }
}

// 批量删除
const handleBatchDelete = async () => {
  try {
    await ElMessageBox.confirm(
      `确定要删除选中的 ${selectedItems.value.length} 项吗？此操作不可恢复！`,
      '批量删除',
      {
        type: 'error',
        confirmButtonText: '确定删除',
        confirmButtonClass: 'el-button--danger'
      }
    )
    
    const ids = selectedItems.value.map(item => item.id)
    emit('batch-delete', ids)
    ElMessage.success('批量删除成功')
  } catch {
    // 取消操作
  }
}

// 批量导出
const handleBatchExport = () => {
  const ids = selectedItems.value.map(item => item.id)
  emit('batch-export', ids)
  ElMessage.success('开始导出，请稍候...')
}

// 监听选择变化
watch(() => props.selectedItems, (newVal) => {
  emit('update:selectedItems', newVal)
}, { deep: true })
</script>

<style scoped>
.batch-actions {
  position: sticky;
  top: 0;
  z-index: 100;
  margin-bottom: 16px;
}

.batch-card {
  border: 2px solid var(--el-color-primary);
}

.batch-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}

.batch-buttons {
  display: flex;
  gap: 8px;
}

:deep(.el-checkbox) {
  font-weight: bold;
  font-size: 14px;
}
</style>
