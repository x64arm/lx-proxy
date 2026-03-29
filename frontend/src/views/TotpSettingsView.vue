<template>
  <div class="totp-settings">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>🔐 {{ t('security.totpTitle') }}</span>
          <el-tag :type="totpStatus.enabled ? 'success' : 'info'">
            {{ totpStatus.enabled ? t('security.enabled') : t('security.notEnabled') }}
          </el-tag>
        </div>
      </template>

      <!-- 未启用状态 -->
      <div v-if="!totpStatus.enabled" class="setup-section">
        <el-alert
          :title="t('security.enhanceSecurity')"
          :description="t('security.enhanceSecurityDesc')"
          type="info"
          :closable="false"
          show-icon
        />

        <div class="setup-steps">
          <h3>{{ t('security.setupSteps') }}</h3>
          
          <el-steps :active="currentStep" finish-status="success" align-center>
            <el-step :title="t('security.step1')" />
            <el-step :title="t('security.step2')" />
            <el-step :title="t('security.step3')" />
          </el-steps>

          <!-- 步骤 1：生成密钥 -->
          <div v-if="currentStep === 0" class="step-content">
            <el-button type="primary" @click="initSetup" :loading="loading">
              {{ t('security.generateSecret') }}
            </el-button>
          </div>

          <!-- 步骤 2：扫描二维码 -->
          <div v-if="currentStep === 1" class="step-content">
            <el-alert
              :title="t('security.scanWithAuthenticator')"
              :description="t('security.authenticatorRecommendation')"
              type="warning"
              :closable="false"
              show-icon
              class="mb-4"
            />

            <div class="qr-section">
              <div v-if="qrCodeImage" class="qr-code">
                <img :src="qrCodeImage" alt="TOTP QR Code" />
              </div>
              
              <el-divider>{{ t('security.orEnterManually') }}</el-divider>
              
              <el-input
                v-model="totpSecret"
                readonly
                class="secret-input"
              >
                <template #prepend>{{ t('security.secretKey') }}</template>
                <template #append>
                  <el-button @click="copySecret">
                    <el-icon><CopyDocument /></el-icon>
                  </el-button>
                </template>
              </el-input>
            </div>

            <div class="step-actions">
              <el-button @click="currentStep = 0">{{ t('security.previousStep') }}</el-button>
              <el-button type="primary" @click="currentStep = 2">{{ t('security.nextStep') }}</el-button>
            </div>
          </div>

          <!-- 步骤 3：验证启用 -->
          <div v-if="currentStep === 2" class="step-content">
            <el-alert
              :title="t('security.enter6DigitCode')"
              :description="t('security.enter6DigitCodeDesc')"
              type="success"
              :closable="false"
              show-icon
              class="mb-4"
            />

            <el-form :model="verifyForm" label-width="100px" class="verify-form">
              <el-form-item :label="t('security.verificationCode')">
                <el-input
                  v-model="verifyForm.code"
                  :placeholder="t('security.input6DigitCode')"
                  maxlength="6"
                  style="width: 200px"
                />
              </el-form-item>

              <el-form-item>
                <el-button @click="currentStep = 1">{{ t('security.previousStep') }}</el-button>
                <el-button 
                  type="primary" 
                  @click="verifyAndEnable"
                  :loading="verifying"
                >
                  {{ t('security.verifyAndEnable') }}
                </el-button>
              </el-form-item>
            </el-form>
          </div>
        </div>

        <!-- 备用代码 -->
        <div v-if="backupCodes.length > 0" class="backup-codes-section">
          <el-alert
            :title="t('security.backupCodesWarning')"
            :description="t('security.backupCodesDesc')"
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
            {{ t('security.copyAllBackupCodes') }}
          </el-button>
        </div>
      </div>

      <!-- 已启用状态 -->
      <div v-else class="enabled-section">
        <el-alert
          :title="t('security.totpEnabled')"
          :description="t('security.totpEnabledDesc')"
          type="success"
          :closable="false"
          show-icon
          class="mb-4"
        />

        <el-descriptions :title="t('security.totpStatus')" :column="1" border>
          <el-descriptions-item :label="t('common.status')">
            <el-tag type="success">{{ t('security.enabled') }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item :label="t('security.remainingBackupCodes')">
            {{ totpStatus.backup_codes_remaining }} {{ t('security.codes') }}
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <el-button type="danger" @click="showDisableDialog = true">
          {{ t('security.disableTotpButton') }}
        </el-button>
      </div>
    </el-card>

    <!-- 禁用确认对话框 -->
    <el-dialog
      v-model="showDisableDialog"
      :title="t('security.disableConfirmTitle')"
      width="400px"
    >
      <el-alert
        :title="t('security.disableWarning')"
        :description="t('security.disableWarningDesc')"
        type="warning"
        :closable="false"
        show-icon
      />

      <el-form :model="disableForm" label-width="80px" class="mt-4">
        <el-form-item :label="t('security.password')">
          <el-input
            v-model="disableForm.password"
            type="password"
            :placeholder="t('security.inputPasswordConfirm')"
            show-password
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <el-button @click="showDisableDialog = false">{{ t('security.cancel') }}</el-button>
        <el-button type="danger" @click="disableTotp" :loading="disabling">
          {{ t('security.confirmDisable') }}
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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
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
    ElMessage.error(t('security.notLoggedIn'))
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
    ElMessage.success(t('security.enableSuccess'))
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('security.enableFailed'))
  } finally {
    loading.value = false
  }
}

// 验证并启用
const verifyAndEnable = async () => {
  if (!currentUser.value || !verifyForm.code) {
    ElMessage.warning(t('security.input6DigitCode'))
    return
  }

  if (verifyForm.code.length !== 6) {
    ElMessage.warning(t('security.codeMustBe6Digits'))
    return
  }

  verifying.value = true
  try {
    await api.post(`/api/totp/${currentUser.value.id}/verify`, {
      code: verifyForm.code
    })

    ElNotification({
      title: t('security.enableSuccess'),
      message: t('security.enableSuccessMessage'),
      type: 'success',
      duration: 5000
    })

    totpStatus.enabled = true
    totpStatus.backup_codes_remaining = backupCodes.value.length
    currentStep.value = 3 // 完成
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t('security.verifyFailed'))
  } finally {
    verifying.value = false
  }
}

// 禁用 TOTP
const disableTotp = async () => {
  if (!currentUser.value || !disableForm.password) {
    ElMessage.warning(t('security.inputPasswordConfirm'))
    return
  }

  disabling.value = true
  try {
    await api.post(`/api/totp/${currentUser.value.id}/disable`, {
      password: disableForm.password
    })

    ElNotification({
      title: t('security.disableSuccess'),
      message: t('security.disableSuccessMessage'),
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
    ElMessage.error(error.response?.data?.message || t('security.disableFailed'))
  } finally {
    disabling.value = false
  }
}

// 复制密钥
const copySecret = () => {
  navigator.clipboard.writeText(totpSecret.value)
  ElMessage.success(t('security.secretCopied'))
}

// 复制备用代码
const copyBackupCodes = () => {
  navigator.clipboard.writeText(backupCodes.value.join('\n'))
  ElMessage.success(t('security.backupCodesCopied'))
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
  padding: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
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
  transition: transform 0.3s;
}

.qr-code:hover {
  transform: scale(1.02);
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

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .totp-settings {
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

  /* QR 码移动端优化 */
  .qr-code {
    padding: 16px;
    width: 100%;
    max-width: 280px;
  }

  .qr-code img {
    width: 100%;
    height: auto;
    max-width: 240px;
  }

  /* 备用码网格优化 */
  .backup-codes {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .backup-code {
    padding: 8px;
    font-size: 12px;
  }

  /* 表单移动端优化 */
  .verify-form :deep(.el-form-item) {
    margin-bottom: 20px;
  }

  .verify-form :deep(.el-form-item__label) {
    width: 100% !important;
    margin-bottom: 8px;
  }

  .verify-form :deep(.el-input) {
    width: 100% !important;
  }

  /* 步骤说明优化 */
  .setup-steps :deep(h3) {
    font-size: 15px;
  }

  .setup-steps :deep(p) {
    font-size: 13px;
  }

  /* 按钮全宽显示 */
  .step-actions .el-button,
  .enabled-section .el-button {
    width: 100%;
    min-height: 48px;
    margin-bottom: 12px;
  }

  /* 警告框优化 */
  :deep(.el-alert) {
    margin: 0 -12px;
    border-radius: 0;
  }

  :deep(.el-alert__content) {
    font-size: 13px;
  }

  /* 分隔线优化 */
  :deep(.el-divider__text) {
    font-size: 14px;
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

.totp-settings {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
