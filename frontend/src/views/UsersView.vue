<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { User } from '../api'
import { usersAPI } from '../api'

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

const fetchUsers = async () => {
  try {
    const response = await usersAPI.list()
    users.value = response.data
  } catch (error: any) {
    ElMessage.error('获取用户列表失败：' + (error.message || '未知错误'))
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
      ElMessage.success('用户更新成功')
    } else {
      // 创建用户
      await usersAPI.create(form.value)
      ElMessage.success('用户创建成功')
    }
    dialogVisible.value = false
    fetchUsers()
  } catch (error: any) {
    ElMessage.error(editingUser.value ? '更新用户失败' : '创建用户失败' + ': ' + (error.response?.data || error.message))
  }
}

const handleDelete = async (user: User) => {
  try {
    await ElMessageBox.confirm(`确定要删除用户 "${user.username}" 吗？`, '确认删除', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    await usersAPI.delete(user.id)
    ElMessage.success('用户删除成功')
    fetchUsers()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error('删除用户失败：' + (error.message || '未知错误'))
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
  <div class="users-page">
    <div class="header">
      <h1>👥 用户管理</h1>
      <el-button type="primary" @click="openCreateDialog">
        <el-icon><Plus /></el-icon>
        添加用户
      </el-button>
    </div>

    <el-card class="table-card">
      <el-table :data="users" v-loading="loading" stripe>
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="username" label="用户名" />
        <el-table-column prop="role" label="角色" width="100">
          <template #default="{ row }">
            <el-tag :type="row.role === 'admin' ? 'danger' : 'primary'">
              {{ row.role === 'admin' ? '管理员' : '普通用户' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column prop="updated_at" label="更新时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.updated_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button size="small" @click="openEditDialog(row)">
              编辑
            </el-button>
            <el-button size="small" type="danger" @click="handleDelete(row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑用户对话框 -->
    <el-dialog
      v-model="dialogVisible"
      :title="editingUser ? '编辑用户' : '添加用户'"
      width="500px"
    >
      <el-form :model="form" label-width="80px">
        <el-form-item label="用户名" required>
          <el-input v-model="form.username" placeholder="请输入用户名" />
        </el-form-item>
        <el-form-item :label="editingUser ? '新密码' : '密码'" :required="!editingUser">
          <el-input 
            v-model="form.password" 
            type="password" 
            :placeholder="editingUser ? '留空则不修改密码' : '请输入密码'" 
            show-password
          />
        </el-form-item>
        <el-form-item label="角色" required>
          <el-radio-group v-model="form.role">
            <el-radio label="user">普通用户</el-radio>
            <el-radio label="admin">管理员</el-radio>
          </el-radio-group>
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
.users-page {
  padding: 2rem;
  max-width: 1200px;
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
</style>
