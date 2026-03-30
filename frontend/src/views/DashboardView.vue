<template>
  <div class="dashboard">
    <h2 class="page-title">📊 {{ t('dashboard.title') }}</h2>
    
    <div v-if="!loading" class="dashboard-content">
      <!-- 统计卡片 -->
      <el-row :gutter="20" class="stats-row">
        <el-col :xs="24" :sm="12" :md="6">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                👥
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ stats?.total_users || 0 }}</div>
                <div class="stat-label">{{ t('dashboard.totalUsers') }}</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :xs="24" :sm="12" :md="6">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-icon" style="background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%)">
                📡
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ stats?.enabled_inbounds || 0 }}/{{ stats?.total_inbounds || 0 }}</div>
                <div class="stat-label">{{ t('dashboard.totalInbounds') }}</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :xs="24" :sm="12" :md="6">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-icon" style="background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)">
                📊
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ formatBytes(stats?.total_traffic_used || 0) }}</div>
                <div class="stat-label">{{ t('dashboard.trafficUsed') }}</div>
              </div>
            </div>
          </el-card>
        </el-col>
        
        <el-col :xs="24" :sm="12" :md="6">
          <el-card shadow="hover" class="stat-card">
            <div class="stat-content">
              <div class="stat-icon" style="background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)">
                💾
              </div>
              <div class="stat-info">
                <div class="stat-value">{{ formatBytes(systemStatus?.memory_used || 0) }}</div>
                <div class="stat-label">{{ t('dashboard.memoryUsage') }}</div>
              </div>
            </div>
          </el-card>
        </el-col>
      </el-row>
      
      <!-- 系统状态 -->
      <el-row :gutter="20" class="mt-4">
        <el-col :span="24">
          <el-card>
            <template #header>
              <span>💻 {{ t('dashboard.systemStatus') }}</span>
            </template>
            <el-descriptions :column="4" border>
              <el-descriptions-item :label="t('dashboard.cpuUsage')">
                <el-progress :percentage="systemStatus?.cpu_usage || 0" :stroke-width="18" />
              </el-descriptions-item>
              <el-descriptions-item :label="t('dashboard.memoryUsage')">
                {{ formatBytes(systemStatus?.memory_used || 0) }} / {{ formatBytes(systemStatus?.memory_total || 0) }}
              </el-descriptions-item>
              <el-descriptions-item :label="t('dashboard.uptime')">
                {{ formatUptime(systemStatus?.uptime || 0) }}
              </el-descriptions-item>
              <el-descriptions-item :label="t('dashboard.xrayStatus')">
                <el-tag :type="systemStatus?.xray_running ? 'success' : 'danger'">
                  {{ systemStatus?.xray_running ? '✅ ' + t('dashboard.running') : '❌ ' + t('dashboard.stopped') }}
                </el-tag>
              </el-descriptions-item>
            </el-descriptions>
          </el-card>
        </el-col>
      </el-row>
      
      <!-- 快捷操作 -->
      <el-row :gutter="20" class="mt-4">
        <el-col :span="24">
          <el-card>
            <template #header>
              <span>⚡ {{ t('dashboard.quickActions') }}</span>
            </template>
            <div class="quick-actions">
              <router-link to="/users" class="action-btn">
                <el-icon><User /></el-icon>
                <span>{{ t('layout.users') }}</span>
              </router-link>
              <router-link to="/inbounds" class="action-btn">
                <el-icon><Connection /></el-icon>
                <span>{{ t('layout.inbounds') }}</span>
              </router-link>
              <router-link to="/traffic" class="action-btn">
                <el-icon><DataLine /></el-icon>
                <span>{{ t('layout.traffic') }}</span>
              </router-link>
              <router-link to="/subscription" class="action-btn">
                <el-icon><Link /></el-icon>
                <span>{{ t('layout.subscription') }}</span>
              </router-link>
              <router-link to="/settings" class="action-btn">
                <el-icon><Setting /></el-icon>
                <span>{{ t('layout.settings') }}</span>
              </router-link>
            </div>
          </el-card>
        </el-col>
      </el-row>
    </div>
    
    <div v-else class="loading-container">
      <el-skeleton :rows="10" animated />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { User, Connection, DataLine, Link, Setting } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import axios from 'axios'

const { t } = useI18n()

interface Stats {
  total_users: number
  total_inbounds: number
  enabled_inbounds: number
  total_traffic_used: number
  total_traffic_limit: number | null
}

interface SystemStatus {
  cpu_usage: number
  memory_total: number
  memory_used: number
  memory_free: number
  uptime: number
  xray_running: boolean
  connections: number
}

const stats = ref<Stats | null>(null)
const systemStatus = ref<SystemStatus | null>(null)
const loading = ref(true)

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatUptime = (seconds: number): string => {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${days} ${t('dashboard.days')} ${hours} ${t('dashboard.hours')} ${minutes} ${t('dashboard.minutes')}`
}

const fetchStats = async () => {
  try {
    const res = await axios.get('/api/stats')
    stats.value = res.data.data || stats.value
  } catch (e) {
    console.error('Failed to fetch stats:', e)
  }
}

const fetchSystemStatus = async () => {
  try {
    const res = await axios.get('/api/system/status')
    systemStatus.value = res.data.data || systemStatus.value
  } catch (e) {
    console.error('Failed to fetch system status:', e)
  }
}

onMounted(async () => {
  await Promise.all([fetchStats(), fetchSystemStatus()])
  loading.value = false
  
  // 每 30 秒刷新一次数据
  setInterval(async () => {
    await Promise.all([fetchStats(), fetchSystemStatus()])
  }, 30000)
})
</script>

<style scoped>
.dashboard {
  padding: 0;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 20px;
}

.stats-row {
  margin-bottom: 0;
}

.stat-card {
  margin-bottom: 20px;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 15px;
}

.stat-icon {
  width: 60px;
  height: 60px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: white;
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #303133;
  line-height: 1.2;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.mt-4 {
  margin-top: 20px;
}

.quick-actions {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 8px;
  text-decoration: none;
  transition: transform 0.2s, box-shadow 0.2s;
}

.action-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.action-btn .el-icon {
  font-size: 18px;
}

.loading-container {
  padding: 20px;
}
</style>
