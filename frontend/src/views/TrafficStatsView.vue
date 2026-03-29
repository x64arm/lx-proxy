<template>
  <div class="traffic-stats-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>📊 {{ t('traffic.title') }}</span>
          <div class="header-actions">
            <el-select v-model="selectedInbound" :placeholder="t('traffic.selectInbound')" @change="loadTrafficData" style="width: 200px;">
              <el-option :label="t('traffic.allInbounds')" value="all" />
              <el-option
                v-for="inbound in inbounds"
                :key="inbound.id"
                :label="`${inbound.name} (${inbound.protocol})`"
                :value="inbound.id"
              />
            </el-select>
            <el-date-picker
              v-model="dateRange"
              type="daterange"
              :range-separator="t('traffic.to')"
              :start-placeholder="t('traffic.startDate')"
              :end-placeholder="t('traffic.endDate')"
              @change="loadTrafficData"
              style="margin-left: 10px;"
            />
          </div>
        </div>
      </template>

      <!-- 总流量概览 -->
      <el-row :gutter="20" class="mb-4">
        <el-col :xs="24" :sm="8">
          <el-statistic :title="t('traffic.totalUpload')" :value="totalUpload" precision="2">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :xs="24" :sm="8">
          <el-statistic :title="t('traffic.totalDownload')" :value="totalDownload" precision="2">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :xs="24" :sm="8">
          <el-statistic :title="t('traffic.total')" :value="totalTraffic" precision="2">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
      </el-row>

      <!-- 流量图表 -->
      <div ref="chartRef" style="height: 400px; width: 100%"></div>
    </el-card>

    <!-- 流量历史记录 -->
    <el-card class="box-card mt-4">
      <template #header>
        <div class="card-header">
          <span>📋 {{ t('traffic.trafficHistory') }}</span>
          <div class="header-right">
            <!-- 视图切换按钮（移动端） -->
            <el-button 
              v-if="isMobile" 
              circle 
              size="small"
              @click="viewMode = viewMode === 'table' ? 'card' : 'table'"
              class="view-toggle-btn"
            >
              <el-icon><MoreFilled /></el-icon>
            </el-button>
            <el-button size="small" @click="loadTrafficData">
              <el-icon><Refresh /></el-icon>
              <span v-if="!isMobile">{{ t('common.refresh') }}</span>
            </el-button>
          </div>
        </div>
      </template>
      
      <!-- 表格视图 -->
      <div class="table-wrapper" v-if="viewMode === 'table'">
        <el-table :data="trafficHistory" stripe style="width: 100%" v-loading="loading">
          <el-table-column prop="date" :label="t('traffic.date')" width="120" sortable />
          <el-table-column prop="inbound_name" :label="t('traffic.inboundName')" min-width="150" />
          <el-table-column :label="`${t('traffic.upload')} (GB)`" width="110" sortable>
            <template #default="{ row }">
              {{ (row.upload / 1073741824).toFixed(2) }}
            </template>
          </el-table-column>
          <el-table-column :label="`${t('traffic.download')} (GB)`" width="120" sortable>
            <template #default="{ row }">
              {{ (row.download / 1073741824).toFixed(2) }}
            </template>
          </el-table-column>
          <el-table-column :label="`${t('traffic.total')} (GB)`" width="110" sortable>
            <template #default="{ row }">
              {{ ((row.upload + row.download) / 1073741824).toFixed(2) }}
            </template>
          </el-table-column>
        </el-table>
      </div>
      
      <!-- 卡片视图（移动端） -->
      <div class="cards-container" v-if="viewMode === 'card'">
        <el-card 
          v-for="record in trafficHistory" 
          :key="record.date + record.inbound_id" 
          shadow="hover"
          class="traffic-card"
          v-loading="loading"
        >
          <div class="card-header">
            <div class="card-date">📅 {{ record.date }}</div>
            <div class="card-inbound">{{ record.inbound_name }}</div>
          </div>
          
          <div class="card-body">
            <div class="stat-row">
              <span class="stat-label">⬆️ {{ t('traffic.upload') }}：</span>
              <span class="stat-value upload">{{ (record.upload / 1073741824).toFixed(2) }} GB</span>
            </div>
            <div class="stat-row">
              <span class="stat-label">⬇️ {{ t('traffic.download') }}：</span>
              <span class="stat-value download">{{ (record.download / 1073741824).toFixed(2) }} GB</span>
            </div>
            <div class="stat-row total">
              <span class="stat-label">📊 {{ t('traffic.total') }}：</span>
              <span class="stat-value">{{ ((record.upload + record.download) / 1073741824).toFixed(2) }} GB</span>
            </div>
          </div>
        </el-card>
        
        <!-- 空状态 -->
        <el-empty v-if="trafficHistory.length === 0 && !loading" :description="t('traffic.noData')" />
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import axios from 'axios'
import { useI18n } from 'vue-i18n'
import { Refresh, MoreFilled } from '@element-plus/icons-vue'

const { t } = useI18n()

interface Inbound {
  id: string
  name: string
  protocol: string
}

interface TrafficRecord {
  date: string
  inbound_id: string
  inbound_name: string
  upload: number
  download: number
}

const loading = ref(false)
const chartRef = ref<HTMLElement | null>(null)
const selectedInbound = ref<string>('all')
const dateRange = ref<[Date, Date] | null>(null)
const inbounds = ref<Inbound[]>([])
const trafficHistory = ref<TrafficRecord[]>([])
const totalUpload = ref<number>(0)
const totalDownload = ref<number>(0)
let chartInstance: echarts.ECharts | null = null

// 响应式状态
const isMobile = ref(false)
const viewMode = ref<'table' | 'card'>('table')

// 检测设备类型
const checkDevice = () => {
  const width = window.innerWidth
  isMobile.value = width <= 768
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

// 加载入站列表
const loadInbounds = async () => {
  try {
    const res = await axios.get('/api/inbounds')
    inbounds.value = res.data.data || []
  } catch (error) {
    ElMessage.error(t('traffic.loadInboundsFailed'))
  }
}

// 加载流量数据
const loadTrafficData = async () => {
  loading.value = true
  try {
    let url = '/api/traffic?limit=60'
    
    // 添加入站筛选
    if (selectedInbound.value !== 'all') {
      url += `&inbound_id=${selectedInbound.value}`
    }
    
    // 添加日期范围
    if (dateRange.value) {
      const startDate = dateRange.value[0].toISOString().split('T')[0]
      const endDate = dateRange.value[1].toISOString().split('T')[0]
      url += `&start_date=${startDate}&end_date=${endDate}`
    }
    
    const res = await axios.get(url)
    const data = res.data || []
    
    trafficHistory.value = data
    
    // 计算总量
    totalUpload.value = data.reduce((sum: number, r: TrafficRecord) => sum + r.upload, 0) / 1073741824
    totalDownload.value = data.reduce((sum: number, r: TrafficRecord) => sum + r.download, 0) / 1073741824
    
    // 更新图表
    updateChart(data)
  } catch (error) {
    ElMessage.error(t('traffic.loadTrafficDataFailed'))
    trafficHistory.value = []
  } finally {
    loading.value = false
  }
}

// 更新图表
const updateChart = (data: TrafficRecord[]) => {
  if (!chartRef.value) return
  
  if (!chartInstance) {
    chartInstance = echarts.init(chartRef.value)
    window.addEventListener('resize', () => chartInstance?.resize())
  }
  
  // 按日期聚合数据
  const dateMap = new Map<string, { upload: number; download: number }>()
  data.forEach((record: TrafficRecord) => {
    const existing = dateMap.get(record.date) || { upload: 0, download: 0 }
    dateMap.set(record.date, {
      upload: existing.upload + record.upload,
      download: existing.download + record.download
    })
  })
  
  const sortedDates = Array.from(dateMap.keys()).sort()
  const uploads = sortedDates.map(date => (dateMap.get(date)?.upload || 0) / 1073741824)
  const downloads = sortedDates.map(date => (dateMap.get(date)?.download || 0) / 1073741824)
  
  chartInstance.setOption({
    title: { text: t('traffic.trafficTrend') + ' (GB)', left: 'center' },
    tooltip: { 
      trigger: 'axis',
      axisPointer: { type: 'shadow' }
    },
    legend: { 
      data: [t('traffic.upload'), t('traffic.download')],
      top: '10%'
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      top: '20%',
      containLabel: true
    },
    xAxis: { 
      type: 'category', 
      data: sortedDates,
      axisLabel: { rotate: 45 }
    },
    yAxis: { 
      type: 'value', 
      name: 'GB',
      axisLabel: { formatter: '{value} GB' }
    },
    dataZoom: [
      { type: 'slider', start: 0, end: 100 },
      { type: 'inside', start: 0, end: 100 }
    ],
    series: [
      { 
        name: t('traffic.upload'), 
        type: 'bar', 
        data: uploads, 
        itemStyle: { 
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: '#83bff6' },
            { offset: 1, color: '#188df0' }
          ])
        }
      },
      { 
        name: t('traffic.download'), 
        type: 'bar', 
        data: downloads, 
        itemStyle: { 
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: '#a8e063' },
            { offset: 1, color: '#56ab2f' }
          ])
        }
      }
    ]
  })
}

const totalTraffic = computed(() => {
  return totalUpload.value + totalDownload.value
})

onMounted(async () => {
  await loadInbounds()
  await loadTrafficData()
})
</script>

<style scoped>
.traffic-stats-view {
  padding: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.header-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.view-toggle-btn {
  width: 32px;
  height: 32px;
  padding: 0;
}

.mb-4 {
  margin-bottom: 20px;
}

.mt-4 {
  margin-top: 20px;
}

/* 表格包装器 - 支持横向滚动 */
.table-wrapper {
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}

.table-wrapper :deep(.el-table) {
  min-width: 800px;
}

/* ========== 卡片视图样式 ========== */
.cards-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.traffic-card {
  transition: transform 0.3s, box-shadow 0.3s;
}

.traffic-card:hover {
  transform: translateY(-2px);
}

.traffic-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
  flex-wrap: nowrap;
}

.card-date {
  font-size: 14px;
  color: #909399;
  font-weight: 500;
}

.card-inbound {
  font-size: 13px;
  color: #303133;
  background: #f5f7fa;
  padding: 4px 8px;
  border-radius: 4px;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stat-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: #909399;
  font-weight: 500;
}

.stat-value {
  font-size: 16px;
  font-weight: 600;
}

.stat-value.upload {
  color: #67c23a;
}

.stat-value.download {
  color: #409eff;
}

.stat-row.total .stat-value {
  color: #e6a23c;
  font-size: 18px;
}

/* ========== 响应式适配 ========== */
@media (max-width: 768px) {
  .traffic-stats-view {
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

  .header-actions {
    width: 100%;
    flex-direction: column;
  }

  .header-actions .el-select,
  .header-actions .el-date-picker {
    width: 100% !important;
    margin-left: 0 !important;
  }

  .header-right {
    width: 100%;
    justify-content: space-between;
  }

  /* 统计卡片响应式 */
  .mb-4 :deep(.el-row) {
    flex-direction: column;
    gap: 16px;
  }

  .mb-4 :deep(.el-col) {
    width: 100% !important;
  }

  /* 图表容器优化 */
  .box-card :deep([style*="height: 400px"]) {
    height: 300px !important;
  }

  /* 卡片视图优化 */
  .cards-container {
    margin: 0 -12px;
    grid-template-columns: 1fr;
  }

  .traffic-card {
    margin: 0 12px 12px;
    border-radius: 12px;
  }

  .traffic-card :deep(.el-card__body) {
    padding: 16px;
  }

  .card-date {
    font-size: 13px;
  }

  .card-inbound {
    font-size: 12px;
    max-width: 150px;
  }

  .stat-value {
    font-size: 15px;
  }

  .stat-row.total .stat-value {
    font-size: 17px;
  }
}

/* ========== 触摸友好优化 ========== */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title) {
  min-height: 44px;
}

* {
  box-sizing: border-box;
}

.traffic-stats-view {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
