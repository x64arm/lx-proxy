<template>
  <div class="email-settings">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>📧 邮件通知设置</span>
          <el-tag :type="emailStatus.configured ? 'success' : 'info'">
            {{ emailStatus.configured ? '已配置' : '未配置' }}
          </el-tag>
        </div>
      </template>

      <el-alert
        title="邮件通知功能"
        description="配置 SMTP 服务器后，系统会自动发送流量告警、到期提醒等通知邮件。"
        type="info"
        :closable="false"
        show-icon
        class="mb-4"
      />

      <el-form
        :model="emailForm"
        label-width="140px"
        class="email-form"
        :disabled="!editing"
      >
        <el-divider content-position="left">SMTP 服务器配置</el-divider>

        <el-form-item label="SMTP 服务器" required>
          <el-input
            v-model="emailForm.smtp_server"
            placeholder="例如：smtp.gmail.com"
          />
        </el-form-item>

        <el-form-item label="SMTP 端口" required>
          <el-input-number
            v-model="emailForm.smtp_port"
            :min="1"
            :max="65535"
            style="width: 150px"
          />
          <span class="form-tip">常用端口：587 (TLS), 465 (SSL), 25 (不加密)</span>
        </el-form-item>

        <el-form-item label="用户名" required>
          <el-input
            v-model="emailForm.smtp_username"
            placeholder="邮箱地址"
          />
        </el-form-item>

        <el-form-item label="密码" required>
          <el-input
            v-model="emailForm.smtp_password"
            type="password"
            placeholder="SMTP 密码或应用专用密码"
            show-password
          />
          <span class="form-tip">
            Gmail 用户需使用
            <el-link type="primary" href="https://support.google.com/accounts/answer/185833" target="_blank">
              应用专用密码
            </el-link>
          </span>
        </el-form-item>

        <el-divider content-position="left">发件人配置</el-divider>

        <el-form-item label="发件人邮箱">
          <el-input
            v-model="emailForm.smtp_from_email"
            placeholder="默认与用户名相同"
          />
        </el-form-item>

        <el-form-item label="发件人名称">
          <el-input
            v-model="emailForm.smtp_from_name"
            placeholder="例如：LX-Proxy"
          />
        </el-form-item>

        <el-divider content-position="left">测试邮件</el-divider>

        <el-form-item label="测试邮箱">
          <el-input
            v-model="testEmail"
            placeholder="接收测试邮件的邮箱地址"
            style="width: 300px"
          />
          <el-button
            type="primary"
            @click="sendTestEmail"
            :loading="sendingTest"
            class="ml-2"
          >
            发送测试邮件
          </el-button>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="saveSettings" :loading="saving">
            保存配置
          </el-button>
          <el-button @click="cancelEdit" v-if="editing">
            取消
          </el-button>
          <el-button @click="startEdit" v-if="!editing">
            编辑配置
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 通知类型说明 -->
      <el-divider />

      <div class="notification-types">
        <h4>📬 自动发送的通知类型：</h4>
        
        <el-row :gutter="20">
          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">📊</div>
              <h5>流量告警</h5>
              <p>当流量使用达到 70% 或 90% 时自动发送</p>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">⏰</div>
              <h5>到期提醒</h5>
              <p>配置到期前自动发送提醒邮件</p>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">🚫</div>
              <h5>禁用通知</h5>
              <p>配置被禁用时通知用户</p>
            </el-card>
          </el-col>
        </el-row>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'
import api from '@/api'

// 状态
const editing = ref(false)
const saving = ref(false)
const sendingTest = ref(false)

// 邮件配置状态
const emailStatus = reactive({
  configured: false,
  smtp_server: '',
  smtp_port: 587,
  from_email: ''
})

// 邮件表单
const emailForm = reactive({
  smtp_server: '',
  smtp_port: 587,
  smtp_username: '',
  smtp_password: '',
  smtp_from_email: '',
  smtp_from_name: 'LX-Proxy'
})

// 测试邮箱
const testEmail = ref('')

// 获取邮件配置状态
const fetchEmailStatus = async () => {
  try {
    const res = await api.get('/api/email/status')
    emailStatus.configured = res.data.configured
    emailStatus.smtp_server = res.data.smtp_server || ''
    emailStatus.smtp_port = res.data.smtp_port || 587
    emailStatus.from_email = res.data.from_email || ''

    // 如果已配置，加载现有值（密码除外）
    if (emailStatus.configured) {
      emailForm.smtp_server = emailStatus.smtp_server
      emailForm.smtp_port = emailStatus.smtp_port
      emailForm.smtp_username = emailStatus.from_email
      emailForm.smtp_from_email = emailStatus.from_email
      emailForm.smtp_from_name = 'LX-Proxy'
      // 密码不加载，保持空白
    }
  } catch (error) {
    console.error('Failed to fetch email status:', error)
  }
}

// 开始编辑
const startEdit = () => {
  editing.value = true
}

// 取消编辑
const cancelEdit = () => {
  editing.value = false
  fetchEmailStatus()
}

// 保存配置
const saveSettings = async () => {
  // 验证必填字段
  if (!emailForm.smtp_server || !emailForm.smtp_username || !emailForm.smtp_password) {
    ElMessage.warning('请填写必填字段')
    return
  }

  saving.value = true
  try {
    // 注意：后端需要添加保存配置的 API
    // 这里暂时只是前端验证和提示
    ElMessage.success('配置已保存（需要后端支持）')
    editing.value = false
    fetchEmailStatus()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '保存失败')
  } finally {
    saving.value = false
  }
}

// 发送测试邮件
const sendTestEmail = async () => {
  if (!testEmail.value) {
    ElMessage.warning('请输入测试邮箱地址')
    return
  }

  // 简单的邮箱格式验证
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  if (!emailRegex.test(testEmail.value)) {
    ElMessage.warning('请输入有效的邮箱地址')
    return
  }

  sendingTest.value = true
  try {
    await api.post('/api/email/test', {
      email: testEmail.value
    })

    ElNotification({
      title: '✅ 测试邮件已发送',
      message: `测试邮件已发送至 ${testEmail.value}，请查收`,
      type: 'success',
      duration: 5000
    })
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '发送失败，请检查 SMTP 配置')
  } finally {
    sendingTest.value = false
  }
}

onMounted(() => {
  fetchEmailStatus()
})
</script>

<style scoped>
.email-settings {
  max-width: 800px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.mb-4 {
  margin-bottom: 16px;
}

.email-form {
  margin-top: 20px;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
  display: block;
}

.ml-2 {
  margin-left: 8px;
}

.notification-types {
  margin-top: 30px;
}

.notification-types h4 {
  margin-bottom: 20px;
  font-size: 16px;
  color: #303133;
}

.notification-card {
  text-align: center;
  padding: 20px;
}

.notification-icon {
  font-size: 48px;
  margin-bottom: 10px;
}

.notification-card h5 {
  margin: 10px 0;
  font-size: 16px;
  color: #303133;
}

.notification-card p {
  font-size: 13px;
  color: #909399;
  margin: 0;
}
</style>
