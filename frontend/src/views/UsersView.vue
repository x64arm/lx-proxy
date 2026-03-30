<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from 'vue-i18n'
import type { User } from '../api'
import { usersAPI } from '../api'
import { Plus, Edit, Delete, MoreFilled } from '@element-plus/icons-vue'

const { t } = useI18n()

interface UserForm {
  username: string
  password: string
  role: 'admin' | 'user'
}

const users = ref<User[]>([])
const loading = ref(true)
const dialogVisible = ref(false)
const editingUser = ref<User | null>(null)
const form = ref<UserForm>({
  username: '',
  password: '',
  role: 'user'
})

// 响应式状态
const isMobile = ref(false)
const viewMode = ref<'table' | 'card'>('table')

// 检测设备类型
const checkDevice = () => {
  const width = window.innerWidth
  isMobile.value = width <= 768
  // 移动端默认卡片视图，桌面端默认表格视图
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

const fetchUsers = async () => {
  try {
    const response = await usersAPI.list()
    users.value = response.data
  } catch (error: any) {
    ElMessage.error(t('users.fetchFailed') + ': ' + (error.message || t('common.error')))
  } finally {
    loading.value = false
  }
}

const openCreateDialog = () => {
  editingUser.value = null
  form.value = {
    username: '',
    password: '',
    role: 'user'
  }
  dialogVisible.value = true
}

const openEditDialog = (user: User) => {
  editingUser.value = user
  form.value = {
    username: user.username,
    password: '',
    role: user.role
  }
  dialogVisible.value = true
}

const handleSubmit = async () => {
  try {
    if (editingUser.value) {
      // 更新用户
      const updateData: any = {
        username: form.value.username,
        role: form.value.role
      }
      if (form.value.password) {
        updateData.password = form.value.password
      }
      await usersAPI.update(editingUser.value.id, updateData)
      ElMessage.success(t('users.updateSuccess'))
    } else {
      // 创建用户
      await usersAPI.create(form.value)
      ElMessage.success(t('users.createSuccess'))
    }
    dialogVisible.value = false
    fetchUsers()
  } catch (error: any) {
    const baseMsg = editingUser.value ? t('users.updateFailed') : t('users.createFailed')
    ElMessage.error(baseMsg + ': ' + (error.response?.data || error.message))
  }
}

const handleDelete = async (user: User) => {
  try {
    await ElMessageBox.confirm(
      t('users.deleteConfirm', { username: user.username }),
      t('users.confirmDelete'),
      {
        confirmButtonText: t('common.confirm'),
        cancelButtonText: t('common.cancel'),
        type: 'warning'
      }
    )
    await usersAPI.delete(user.id)
    ElMessage.success(t('users.deleteSuccess'))
    fetchUsers()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(t('users.deleteFailed') + ': ' + (error.message || t('common.error')))
    }
  }
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString('zh-CN')
}

onMounted(() => {
  fetchUsers()
})
</script>

<template>
  <div class="users-page" :class="{ 'mobile-page': isMobile }">
    <div class="header">
      <h1>👥 {{ t('users.title') }}</h1>
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
          <span v-if="!isMobile">{{ t('users.addUser') }}</span>
        </el-button>
      </div>
    </div>

    <!-- 表格视图 -->
    <el-card class="table-card" v-if="viewMode === 'table'">
      <div class="table-wrapper" :class="{ 'mobile-table': isMobile }">
        <el-table :data="users" v-loading="loading" stripe>
          <el-table-column prop="id" :label="t('users.id')" width="80" />
          <el-table-column prop="username" :label="t('users.username')" min-width="120" />
          <el-table-column prop="role" :label="t('users.role')" width="100">
            <template #default="{ row }">
              <el-tag :type="row.role === 'admin' ? 'danger' : 'primary'">
                {{ row.role === 'admin' ? t('users.admin') : t('users.normalUser') }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="created_at" :label="t('users.createdAt')" width="160">
            <template #default="{ row }">
              {{ formatDate(row.created_at) }}
            </template>
          </el-table-column>
          <el-table-column prop="updated_at" :label="t('users.updatedAt')" width="160">
            <template #default="{ row }">
              {{ formatDate(row.updated_at) }}
            </template>
          </el-table-column>
          <el-table-column :label="t('users.actions')" width="180" fixed="right">
            <template #default="{ row }">
              <div class="action-buttons" :class="{ 'mobile-actions': isMobile }">
                <el-button size="small" @click="openEditDialog(row)">
                  {{ t('users.edit') }}
                </el-button>
                <el-button size="small" type="danger" @click="handleDelete(row)">
                  {{ t('users.delete') }}
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
        v-for="user in users" 
        :key="user.id" 
        shadow="hover"
        class="user-card"
        v-loading="loading"
      >
        <div class="card-header">
          <div class="card-id">#{{ user.id }}</div>
          <el-tag :type="user.role === 'admin' ? 'danger' : 'primary'" size="small">
            {{ user.role === 'admin' ? t('users.admin') : t('users.normalUser') }}
          </el-tag>
        </div>
        
        <div class="card-body">
          <div class="card-row">
            <span class="card-label">👤 {{ t('users.username') }}：</span>
            <span class="card-value">{{ user.username }}</span>
          </div>
          <div class="card-row">
            <span class="card-label">📅 {{ t('users.createdAt') }}：</span>
            <span class="card-value">{{ formatDate(user.created_at) }}</span>
          </div>
          <div class="card-row">
            <span class="card-label">🔄 {{ t('users.updatedAt') }}：</span>
            <span class="card-value">{{ formatDate(user.updated_at) }}</span>
          </div>
        </div>
        
        <div class="card-actions">
          <el-button type="primary" @click="openEditDialog(user)" class="action-btn">
            <el-icon><Edit /></el-icon>
            {{ t('users.edit') }}
          </el-button>
          <el-button type="danger" @click="handleDelete(user)" class="action-btn">
            <el-icon><Delete /></el-icon>
            {{ t('users.delete') }}
          </el-button>
        </div>
      </el-card>
      
      <!-- 空状态 -->
      <el-empty v-if="users.length === 0 && !loading" :description="t('users.noData')" />
    </div>

    <!-- 添加/编辑用户对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingUser ? t('users.editUser') : t('users.addUser')"
      width="500px"
    >
      <el-form :model="form" label-width="80px">
        <el-form-item :label="t('users.username')" required>
          <el-input v-model="form.username" :placeholder="t('users.pleaseEnterUsername')" />
        </el-form-item>
        <el-form-item 
          :label="editingUser ? t('users.newPassword') : t('users.password')" 
          :required="!editingUser"
        >
          <el-input 
            v-model="form.password" 
            type="password" 
            :placeholder="editingUser ? t('users.keepPasswordEmpty') : t('users.pleaseEnterPassword')" 
            show-password
          />
        </el-form-item>
        <el-form-item :label="t('users.role')" required>
          <el-radio-group v-model="form.role">
            <el-radio label="user">{{ t('users.normalUser') }}</el-radio>
            <el-radio label="admin">{{ t('users.admin') }}</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ t('users.cancel') }}</el-button>
        <el-button type="primary" @click="handleSubmit">{{ t('users.submit') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.users-page {
  padding: 2rem;
  max-width: 1200px;
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
  min-width: 800px;
}

/* 移动端操作按钮优化 */
.action-buttons {
  display: flex;
  gap: 4px;
}

.mobile-actions {
  flex-direction: column;
  gap: 8px;
}

.mobile-actions .el-button {
  width: 100%;
}

/* ========== 卡片视图样式 ========== */
.cards-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.mobile-page .cards-container {
  grid-template-columns: 1fr;
}

.user-card {
  transition: transform 0.3s, box-shadow 0.3s;
}

.user-card:hover {
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
}

.card-id {
  font-size: 14px;
  color: #909399;
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
  word-break: break-all;
}

.card-actions {
  display: flex;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid #ebeef5;
}

.card-actions .action-btn {
  flex: 1;
  min-height: 44px; /* 触摸友好的最小高度 */
}

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .users-page {
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

  /* 移动端表格样式优化 */
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

  .user-card {
    margin: 0 12px 12px;
    border-radius: 12px;
  }

  .user-card :deep(.el-card__body) {
    padding: 16px;
  }

  .card-label {
    font-size: 11px;
  }

  .card-value {
    font-size: 13px;
  }

  .card-actions .action-btn {
    min-height: 48px; /* 移动端更大的点击区域 */
    font-size: 14px;
  }
}

/* ========== 触摸友好优化 ========== */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title) {
  min-height: 44px;
}

/* 防止水平滚动 */
* {
  box-sizing: border-box;
}

.users-page {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
