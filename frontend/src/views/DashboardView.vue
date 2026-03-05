<script setup lang="ts">
import { ref, onMounted } from 'vue'

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
  return `${days}天 ${hours}小时 ${minutes}分钟`
}

const fetchStats = async () => {
  try {
    const token = localStorage.getItem('token')
    const response = await fetch('/api/stats', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    if (response.ok) {
      stats.value = await response.json()
    }
  } catch (e) {
    console.error('Failed to fetch stats:', e)
  }
}

const fetchSystemStatus = async () => {
  try {
    const token = localStorage.getItem('token')
    const response = await fetch('/api/system/status', {
      headers: { 'Authorization': `Bearer ${token}` }
    })
    if (response.ok) {
      systemStatus.value = await response.json()
    }
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

const logout = () => {
  localStorage.removeItem('token')
  localStorage.removeItem('user')
  window.location.href = '/login'
}
</script>

<template>
  <div class="dashboard">
    <header class="header">
      <h1>🚀 LX-Proxy 仪表盘</h1>
      <button @click="logout" class="logout-btn">退出登录</button>
    </header>
    
    <main class="main" v-if="!loading">
      <!-- 统计卡片 -->
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-icon">👥</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.total_users || 0 }}</div>
            <div class="stat-label">用户总数</div>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">📡</div>
          <div class="stat-info">
            <div class="stat-value">{{ stats?.enabled_inbounds || 0 }}/{{ stats?.total_inbounds || 0 }}</div>
            <div class="stat-label">入站配置</div>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">📊</div>
          <div class="stat-info">
            <div class="stat-value">{{ formatBytes(stats?.total_traffic_used || 0) }}</div>
            <div class="stat-label">已用流量</div>
          </div>
        </div>
        
        <div class="stat-card">
          <div class="stat-icon">💾</div>
          <div class="stat-info">
            <div class="stat-value">{{ formatBytes(systemStatus?.memory_used || 0) }}</div>
            <div class="stat-label">内存使用</div>
          </div>
        </div>
      </div>
      
      <!-- 系统状态 -->
      <div class="section">
        <h2>系统状态</h2>
        <div class="status-grid">
          <div class="status-item">
            <span class="status-label">CPU 使用率</span>
            <div class="progress-bar">
              <div class="progress" :style="{ width: (systemStatus?.cpu_usage || 0) + '%' }"></div>
            </div>
            <span class="status-value">{{ (systemStatus?.cpu_usage || 0).toFixed(1) }}%</span>
          </div>
          
          <div class="status-item">
            <span class="status-label">内存使用</span>
            <div class="progress-bar">
              <div class="progress" :style="{ width: ((systemStatus?.memory_used || 0) / (systemStatus?.memory_total || 1) * 100) + '%' }"></div>
            </div>
            <span class="status-value">{{ formatBytes(systemStatus?.memory_used || 0) }} / {{ formatBytes(systemStatus?.memory_total || 0) }}</span>
          </div>
          
          <div class="status-item">
            <span class="status-label">运行时间</span>
            <span class="status-value">{{ formatUptime(systemStatus?.uptime || 0) }}</span>
          </div>
          
          <div class="status-item">
            <span class="status-label">Xray 状态</span>
            <span class="status-badge" :class="{ active: systemStatus?.xray_running }">
              {{ systemStatus?.xray_running ? '✅ 运行中' : '❌ 已停止' }}
            </span>
          </div>
        </div>
      </div>
      
      <!-- 快捷操作 -->
      <div class="section">
        <h2>快捷操作</h2>
        <div class="actions-grid">
          <router-link to="/users" class="action-btn">👥 用户管理</router-link>
          <router-link to="/inbounds" class="action-btn">📡 入站配置</router-link>
          <router-link to="/traffic" class="action-btn">📊 流量统计</router-link>
          <router-link to="/settings" class="action-btn">⚙️ 系统设置</router-link>
        </div>
      </div>
    </main>
    
    <div v-else class="loading">加载中...</div>
  </div>
</template>

<style scoped>
.dashboard {
  min-height: 100vh;
  background: #f5f7fa;
}

.header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 1.5rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header h1 {
  font-size: 1.5rem;
  margin: 0;
}

.logout-btn {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 0.5rem;
  cursor: pointer;
  transition: background 0.3s;
}

.logout-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.main {
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: white;
  padding: 1.5rem;
  border-radius: 1rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  display: flex;
  align-items: center;
  gap: 1rem;
}

.stat-icon {
  font-size: 2.5rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: #333;
}

.stat-label {
  color: #666;
  font-size: 0.9rem;
}

.section {
  background: white;
  padding: 1.5rem;
  border-radius: 1rem;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 2rem;
}

.section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.25rem;
}

.status-grid {
  display: grid;
  gap: 1.5rem;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.status-label {
  min-width: 100px;
  color: #666;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
}

.progress {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  transition: width 0.3s;
}

.status-value {
  min-width: 150px;
  text-align: right;
  color: #333;
  font-weight: 500;
}

.status-badge {
  padding: 0.25rem 0.75rem;
  border-radius: 1rem;
  font-size: 0.9rem;
  background: #fee;
  color: #c00;
}

.status-badge.active {
  background: #efe;
  color: #0a0;
}

.actions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.action-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  text-decoration: none;
  padding: 1rem;
  border-radius: 0.5rem;
  text-align: center;
  font-weight: 500;
  transition: transform 0.2s;
}

.action-btn:hover {
  transform: translateY(-2px);
}

.loading {
  text-align: center;
  padding: 4rem;
  font-size: 1.25rem;
  color: #666;
}
</style>
