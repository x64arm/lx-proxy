<template>
  <div class="totp-settings">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🔐 双因素认证 (TOTP)</span>
          <el-tag :type="totpStatus.enabled ? 'success' : 'info'">
            {{ totpStatus.enabled ? '已启用' : '未启用' }}
          </el-tag>
        </div>
      </template>

      <!-- 未启用状态 -->
      <div v-if="!totpStatus.enabled" class="setup-section">
        <el-alert
          title="增强账户安全性"
          description="启用双因素认证后，登录时需要输入 6 位验证码，有效提升账户安全性。"
          type="info"
          :closable="false"
          show-icon
        />

        <div class="setup-steps">
          <h3>设置步骤：</h3>
          
          <el-steps :active="currentStep" finish-status="success" align-center>
            <el-step title="1. 生成密钥" />
            <el-step title="2. 扫描二维码" />
            <el-step title="3. 验证启用" />
          </el-steps>

          <!-- 步骤 1：生成密钥 -->
          <div v-if="currentStep === 0" class="step-content">
            <el-button type="primary" @click="initSetup" :loading="loading">
              生成 TOTP 密钥
            </el-button>
          </div>

          <!-- 步骤 2：扫描二维码 -->
          <div v-if="currentStep === 1" class="step-content">
            <el-alert
              title="请使用 Authenticator 应用扫描二维码"
              description="推荐使用：Google Authenticator、Microsoft Authenticator 或 Authy"
              type="warning"
              :closable="false"
              show-icon
              class="mb-4"
            />

            <div class="qr-section">
              <div v-if="qrCodeImage" class="qr-code">
                <img :src="qrCodeImage" alt="TOTP QR Code" />
              </div>
              
              <el-divider>或手动输入密钥</el-divider>
              
              <el-input
                v-model="totpSecret"
                readonly
                class="secret-input"
              >
                <template #prepend>密钥</template>
                <template #append>
                  <el-button @click="copySecret">
                    <el-icon><CopyDocument /></el-icon>
                  </el-button>
                </template>
              </el-input>
            </div>

            <div class="step-actions">
              <el-button @click="currentStep = 0">上一步</el-button>
              <el-button type="primary" @click="currentStep = 2">下一步</el-button>
            </div>
          </div>

          <!-- 步骤 3：验证启用 -->
          <div v-if="currentStep === 2" class="step-content">
            <el-alert
              title="输入 6 位验证码"
              description="打开 Authenticator 应用，输入当前显示的 6 位验证码"
              type="success"
              :closable="false"
              show-icon
              class="mb-4"
            />

            <el-form :model="verifyForm" label-width="100px" class="verify-form">
              <el-form-item label="验证码">
                <el-input
                  v-model="verifyForm.code"
                  placeholder="请输入 6 位验证码"
                  maxlength="6"
                  style="width: 200px"
                />
              </el-form-item>

              <el-form-item>
                <el-button @click="currentStep = 1">上一步</el-button>
                <el-button 
                  type="primary" 
                  @click="verifyAndEnable"
                  :loading="verifying"
                >
                  验证并启用
                </el-button>
              </el-form-item>
            </el-form>
          </div>
        </div>

        <!-- 备用代码 -->
        <div v-if="backupCodes.length > 0" class="backup-codes-section">
          <el-alert
            title="⚠️ 请妥善保存备用代码"
            description="这些代码只能在紧急情况下使用一次，一旦使用将失效。建议打印或保存到安全的地方。"
            type="warning"
            :closable="false"
            show-icon
            class="mb-4"
          />

          <div class="backup-codes">
            <div v-for="(code, index) in backupCodes" :key="index" class="backup-code">
              {{ code }}
            </div>
          </div>

          <el-button @click="copyBackupCodes" class="mt-4">
            <el-icon><CopyDocument /></el-icon>
            复制所有备用代码
          </el-button>
        </div>
      </div>

      <!-- 已启用状态 -->
      <div v-else class="enabled-section">
        <el-alert
          title="✅ 双因素认证已启用"
          description="您的账户已受到双重保护，登录时需要输入验证码。"
          type="success"
          :closable="false"
          show-icon
          class="mb-4"
        />

        <el-descriptions title="TOTP 状态" :column="1" border>
          <el-descriptions-item label="状态">
            <el-tag type="success">已启用</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="剩余备用代码">
            {{ totpStatus.backup_codes_remaining }} 个
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <el-button type="danger" @click="showDisableDialog = true">
          禁用双因素认证
        </el-button>
      </div>
    </el-card>

    <!-- 禁用确认对话框 -->
    <el-dialog
      v-model="showDisableDialog"
      title="禁用双因素认证"
      width="400px"
    >
      <el-alert
        title="⚠️ 警告"
        description="禁用后将降低账户安全性，确定要继续吗？"
        type="warning"
        :closable="false"
        show-icon
      />

      <el-form :model="disableForm" label-width="80px" class="mt-4">
        <el-form-item label="密码">
          <el-input
            v-model="disableForm.password"
            type="password"
            placeholder="请输入密码确认"
            show-password
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showDisableDialog = false">取消</el-button>
        <el-button type="danger" @click="disableTotp" :loading="disabling">
          确认禁用
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElNotification } from 'element-plus'
import { CopyDocument } from '@element-plus/icons-vue'
import { useAuthStore } from '@/stores/auth'
import api from '@/api'

const authStore = useAuthStore()
const currentUser = authStore.currentUser

// 状态
const loading = ref(false)
const verifying = ref(false)
const disabling = ref(false)
const currentStep = ref(0)
const showDisableDialog = ref(false)

// TOTP 状态
const totpStatus = reactive({
  enabled: false,
  verified: false,
  backup_codes_remaining: 0
})

// 设置相关
const totpSecret = ref('')
const qrCodeImage = ref('')
const backupCodes = ref<string[]>([])

// 表单
const verifyForm = reactive({
  code: ''
})

const disableForm = reactive({
  password: ''
})

// 初始化 TOTP 设置
const initSetup = async () => {
  if (!currentUser.value) {
    ElMessage.error('未登录')
    return
  }

  loading.value = true
  try {
    const res = await api.post('/api/totp/setup', {
      user_id: currentUser.value.id
    })

    totpSecret.value = res.data.secret
    qrCodeImage.value = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(res.data.qr_code_url)}`
    backupCodes.value = res.data.backup_codes

    currentStep.value = 1
    ElMessage.success('密钥生成成功')
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '生成失败')
  } finally {
    loading.value = false
  }
}

// 验证并启用
const verifyAndEnable = async () => {
  if (!currentUser.value || !verifyForm.code) {
    ElMessage.warning('请输入验证码')
    return
  }

  if (verifyForm.code.length !== 6) {
    ElMessage.warning('验证码必须是 6 位数字')
    return
  }

  verifying.value = true
  try {
    await api.post(`/api/totp/${currentUser.value.id}/verify`, {
      code: verifyForm.code
    })

    ElNotification({
      title: '✅ 启用成功',
      message: '双因素认证已启用，请妥善保存备用代码',
      type: 'success',
      duration: 5000
    })

    totpStatus.enabled = true
    totpStatus.backup_codes_remaining = backupCodes.value.length
    currentStep.value = 3 // 完成
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '验证失败，请重试')
  } finally {
    verifying.value = false
  }
}

// 禁用 TOTP
const disableTotp = async () => {
  if (!currentUser.value || !disableForm.password) {
    ElMessage.warning('请输入密码')
    return
  }

  disabling.value = true
  try {
    await api.post(`/api/totp/${currentUser.value.id}/disable`, {
      password: disableForm.password
    })

    ElNotification({
      title: '⚠️ 已禁用',
      message: '双因素认证已禁用',
      type: 'warning'
    })

    totpStatus.enabled = false
    totpStatus.backup_codes_remaining = 0
    showDisableDialog.value = false
    disableForm.password = ''
    currentStep.value = 0
    backupCodes.value = []
    totpSecret.value = ''
    qrCodeImage.value = ''
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || '禁用失败')
  } finally {
    disabling.value = false
  }
}

// 复制密钥
const copySecret = () => {
  navigator.clipboard.writeText(totpSecret.value)
  ElMessage.success('密钥已复制')
}

// 复制备用代码
const copyBackupCodes = () => {
  navigator.clipboard.writeText(backupCodes.value.join('\n'))
  ElMessage.success('备用代码已复制')
}

// 获取 TOTP 状态
const fetchTotpStatus = async () => {
  if (!currentUser.value) return

  try {
    const res = await api.get(`/api/totp/${currentUser.value.id}/status`)
    totpStatus.enabled = res.data.enabled
    totpStatus.verified = res.data.verified
    totpStatus.backup_codes_remaining = res.data.backup_codes_remaining
  } catch (error) {
    console.error('Failed to fetch TOTP status:', error)
  }
}

onMounted(() => {
  fetchTotpStatus()
})
</script>

<style scoped>
.totp-settings {
  max-width: 800px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.setup-section {
  margin-top: 20px;
}

.setup-steps h3 {
  margin-bottom: 20px;
}

.step-content {
  margin-top: 30px;
  text-align: center;
}

.qr-section {
  margin: 30px 0;
}

.qr-code {
  display: inline-block;
  padding: 20px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.qr-code img {
  width: 200px;
  height: 200px;
}

.secret-input {
  max-width: 400px;
  margin: 0 auto;
}

.step-actions {
  margin-top: 20px;
}

.backup-codes-section {
  margin-top: 30px;
  padding: 20px;
  background: #f5f7fa;
  border-radius: 8px;
}

.backup-codes {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 10px;
  margin: 20px 0;
}

.backup-code {
  padding: 10px;
  background: white;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  text-align: center;
  font-family: monospace;
  font-size: 14px;
  color: #333;
}

.enabled-section {
  text-align: center;
}

.verify-form {
  max-width: 400px;
  margin: 0 auto;
}

.mb-4 {
  margin-bottom: 16px;
}

.mt-4 {
  margin-top: 16px;
}
</style>
