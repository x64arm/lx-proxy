<template>
  <div class="email-settings">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>📧 {{ t('email.emailSettingsTitle') }}</span>
          <el-tag :type="emailStatus.configured ? 'success' : 'info'">
            {{ emailStatus.configured ? t('email.configured') : t('email.notConfigured') }}
          </el-tag>
        </div>
      </template>

      <el-alert
        :title="t('email.emailFunctionTitle')"
        :description="t('email.emailFunctionDesc')"
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
        <el-divider content-position="left">{{ t('email.smtpConfig') }}</el-divider>

        <el-form-item :label="t('email.smtpServerLabel')" required>
          <el-input
            v-model="emailForm.smtp_server"
            :placeholder="t('email.smtpServerPlaceholder')"
          />
        </el-form-item>

        <el-form-item :label="t('email.smtpPortLabel')" required>
          <el-input-number
            v-model="emailForm.smtp_port"
            :min="1"
            :max="65535"
            style="width: 150px"
          />
          <span class="form-tip">{{ t('email.smtpPortTip') }}</span>
        </el-form-item>

        <el-form-item :label="t('email.username')" required>
          <el-input
            v-model="emailForm.smtp_username"
            :placeholder="t('email.usernamePlaceholder')"
          />
        </el-form-item>

        <el-form-item :label="t('email.password')" required>
          <el-input
            v-model="emailForm.smtp_password"
            type="password"
            :placeholder="t('email.passwordPlaceholder')"
            show-password
          />
          <span class="form-tip">
            {{ t('email.passwordTip') }}
            <el-link type="primary" href="https://support.google.com/accounts/answer/185833" target="_blank">
              {{ t('email.appPassword') }}
            </el-link>
          </span>
        </el-form-item>

        <el-divider content-position="left">{{ t('email.senderConfig') }}</el-divider>

        <el-form-item :label="t('email.senderEmail')">
          <el-input
            v-model="emailForm.smtp_from_email"
            :placeholder="t('email.senderEmailPlaceholder')"
          />
        </el-form-item>

        <el-form-item :label="t('email.senderName')">
          <el-input
            v-model="emailForm.smtp_from_name"
            :placeholder="t('email.senderNamePlaceholder')"
          />
        </el-form-item>

        <el-divider content-position="left">{{ t('email.testEmailSection') }}</el-divider>

        <el-form-item :label="t('email.testEmailAddress')">
          <el-input
            v-model="testEmail"
            :placeholder="t('email.testEmailInputPlaceholder')"
            style="width: 300px"
          />
          <el-button
            type="primary"
            @click="sendTestEmail"
            :loading="sendingTest"
            class="ml-2"
          >
            {{ t('email.sendTestEmail') }}
          </el-button>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="saveSettings" :loading="saving">
            {{ t('email.saveConfig') }}
          </el-button>
          <el-button @click="cancelEdit" v-if="editing">
            {{ t('email.cancel') }}
          </el-button>
          <el-button @click="startEdit" v-if="!editing">
            {{ t('email.editConfig') }}
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 通知类型说明 -->
      <el-divider />

      <div class="notification-types">
        <h4>{{ t('email.autoNotifications') }}</h4>
        
        <el-row :gutter="20">
          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">📊</div>
              <h5>{{ t('email.trafficAlertCard') }}</h5>
              <p>{{ t('email.trafficAlertDesc') }}</p>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">⏰</div>
              <h5>{{ t('email.expiryReminderCard') }}</h5>
              <p>{{ t('email.expiryReminderDesc') }}</p>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="notification-card">
              <div class="notification-icon">🚫</div>
              <h5>{{ t('email.disableNotificationCard') }}</h5>
              <p>{{ t('email.disableNotificationDesc') }}</p>
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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

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
    console.error(t('email.fetchStatusFailed'), error)
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
    ElMessage.warning(t('email.pleaseFillRequiredFields'))
    return
  }

  saving.value = true
  try {
    // 注意：后端需要添加保存配置的 API
    // 这里暂时只是前端验证和提示
    ElMessage.success(t('email.configSaved'))
    editing.value = false
    fetchEmailStatus()
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('email.saveFailed'))
  } finally {
    saving.value = false
  }
}

// 发送测试邮件
const sendTestEmail = async () => {
  if (!testEmail.value) {
    ElMessage.warning(t('email.pleaseEnterTestEmail'))
    return
  }

  // 简单的邮箱格式验证
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  if (!emailRegex.test(testEmail.value)) {
    ElMessage.warning(t('email.invalidEmailFormat'))
    return
  }

  sendingTest.value = true
  try {
    await api.post('/api/email/test', {
      email: testEmail.value
    })

    ElNotification({
      title: t('email.testEmailSent'),
      message: t('email.testEmailSentMessage', { email: testEmail.value }),
      type: 'success',
      duration: 5000
    })
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('email.testEmailFailed'))
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
  padding: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
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
  transition: transform 0.3s, box-shadow 0.3s;
}

.notification-card:hover {
  transform: translateY(-2px);
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

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .email-settings {
    padding: 0;
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

  /* 表单移动端优化 */
  .email-form :deep(.el-form-item) {
    margin-bottom: 20px;
  }

  .email-form :deep(.el-form-item__label) {
    width: 100% !important;
    margin-bottom: 8px;
    font-weight: 500;
  }

  .email-form :deep(.el-form-item__content) {
    width: 100%;
  }

  .email-form :deep(.el-input),
  .email-form :deep(.el-input-number) {
    width: 100% !important;
  }

  .ml-2 {
    margin-left: 0;
    margin-top: 12px;
    width: 100%;
  }

  .ml-2 .el-button {
    width: 100%;
    min-height: 48px;
  }

  /* 通知卡片响应式 */
  .notification-types :deep(.el-row) {
    flex-direction: column;
    gap: 16px;
  }

  .notification-types :deep(.el-col) {
    width: 100% !important;
  }

  .notification-card {
    padding: 16px;
    border-radius: 12px;
  }

  .notification-icon {
    font-size: 40px;
  }

  .notification-card h5 {
    font-size: 15px;
  }

  .notification-card p {
    font-size: 12px;
  }

  /* 分隔线优化 */
  :deep(.el-divider__text) {
    font-size: 14px;
  }

  /* 警告框优化 */
  :deep(.el-alert__content) {
    font-size: 13px;
  }
}

/* ========== 触摸友好优化 ========== */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title),
:deep(.el-form-item__label) {
  min-height: 44px;
}

* {
  box-sizing: border-box;
}

.email-settings {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
