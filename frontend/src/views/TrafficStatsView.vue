<template>
  <div class="traffic-stats-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>📊 流量统计</span>
          <div class="header-actions">
            <el-select v-model="selectedInbound" placeholder="选择入站" @change="loadTrafficData" style="width: 200px;">
              <el-option label="全部入站" value="all" />
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
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              @change="loadTrafficData"
              style="margin-left: 10px;"
            />
          </div>
        </div>
      </template>

      <!-- 总流量概览 -->
      <el-row :gutter="20" class="mb-4">
        <el-col :xs="24" :sm="8">
          <el-statistic title="总上传" :value="totalUpload" precision="2">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :xs="24" :sm="8">
          <el-statistic title="总下载" :value="totalDownload" precision="2">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :xs="24" :sm="8">
          <el-statistic title="总计" :value="totalTraffic" precision="2">
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
          <span>📋 流量历史记录</span>
          <el-button size="small" @click="loadTrafficData">🔄 刷新</el-button>
        </div>
      </template>
      <el-table :data="trafficHistory" stripe style="width: 100%" v-loading="loading">
        <el-table-column prop="date" label="日期" width="120" sortable />
        <el-table-column prop="inbound_name" label="入站名称" width="200" />
        <el-table-column label="上传 (GB)" width="120" sortable>
          <template #default="{ row }">
            {{ (row.upload / 1073741824).toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column label="下载 (GB)" width="120" sortable>
          <template #default="{ row }">
            {{ (row.download / 1073741824).toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column label="总计 (GB)" width="120" sortable>
          <template #default="{ row }">
            {{ ((row.upload + row.download) / 1073741824).toFixed(2) }}
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { ElMessage } from 'element-plus'
import * as echarts from 'echarts'
import axios from 'axios'

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

// 加载入站列表
const loadInbounds = async () => {
  try {
    const res = await axios.get('/api/inbounds')
    inbounds.value = res.data.data || []
  } catch (error) {
    ElMessage.error('加载入站列表失败')
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
    ElMessage.error('加载流量数据失败')
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
    title: { text: '流量趋势 (GB)', left: 'center' },
    tooltip: { 
      trigger: 'axis',
      axisPointer: { type: 'shadow' }
    },
    legend: { 
      data: ['上传', '下载'],
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
        name: '上传', 
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
        name: '下载', 
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

.mb-4 {
  margin-bottom: 20px;
}

.mt-4 {
  margin-top: 20px;
}

@media (max-width: 768px) {
  .card-header {
    flex-direction: column;
    align-items: flex-start;
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
}
</style>
