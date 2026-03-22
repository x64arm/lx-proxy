<template>
  <div class="traffic-stats-view">
    <el-card class="box-card">
      <template #header>
        <div class="card-header">
          <span>📊 流量统计</span>
          <el-select v-model="selectedInbound" placeholder="选择入站" @change="loadTrafficData">
            <el-option label="全部" value="all" />
            <el-option
              v-for="inbound in inbounds"
              :key="inbound.id"
              :label="inbound.name"
              :value="inbound.id"
            />
          </el-select>
        </div>
      </template>

      <!-- 总流量概览 -->
      <el-row :gutter="20" class="mb-4">
        <el-col :span="8">
          <el-statistic title="总上传" :value="totalUpload">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :span="8">
          <el-statistic title="总下载" :value="totalDownload">
            <template #suffix>GB</template>
          </el-statistic>
        </el-col>
        <el-col :span="8">
          <el-statistic title="总计" :value="totalTraffic">
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
        <span>📋 流量历史记录</span>
      </template>
      <el-table :data="trafficHistory" stripe style="width: 100%">
        <el-table-column prop="date" label="日期" width="120" />
        <el-table-column prop="inbound_name" label="入站名称" width="150" />
        <el-table-column label="上传 (GB)">
          <template #default="{ row }">
            {{ (row.upload / 1073741824).toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column label="下载 (GB)">
          <template #default="{ row }">
            {{ (row.download / 1073741824).toFixed(2) }}
          </template>
        </el-table-column>
        <el-table-column label="总计 (GB)">
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
  id: number
  name: string
  protocol: string
}

interface TrafficRecord {
  date: string
  inbound_name: string
  upload: number
  download: number
}

const chartRef = ref<HTMLElement | null>(null)
const selectedInbound = ref<string>('all')
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
  try {
    const url = selectedInbound.value === 'all'
      ? '/api/traffic'
      : `/api/traffic/${selectedInbound.value}`
    
    const res = await axios.get(url)
    const data = res.data.data || []
    
    trafficHistory.value = data
    
    // 计算总量
    totalUpload.value = data.reduce((sum: number, r: TrafficRecord) => sum + r.upload, 0) / 1073741824
    totalDownload.value = data.reduce((sum: number, r: TrafficRecord) => sum + r.download, 0) / 1073741824
    
    // 更新图表
    updateChart(data)
  } catch (error) {
    ElMessage.error('加载流量数据失败')
  }
}

// 更新图表
const updateChart = (data: TrafficRecord[]) => {
  if (!chartRef.value || !chartInstance) return
  
  const dates = data.map((r: TrafficRecord) => r.date)
  const uploads = data.map((r: TrafficRecord) => (r.upload / 1073741824).toFixed(2))
  const downloads = data.map((r: TrafficRecord) => (r.download / 1073741824).toFixed(2))
  
  chartInstance.setOption({
    title: { text: '流量趋势 (GB)' },
    tooltip: { trigger: 'axis' },
    legend: { data: ['上传', '下载'] },
    xAxis: { type: 'category', data: dates },
    yAxis: { type: 'value', name: 'GB' },
    series: [
      { name: '上传', type: 'bar', data: uploads, itemStyle: { color: '#409EFF' } },
      { name: '下载', type: 'bar', data: downloads, itemStyle: { color: '#67C23A' } }
    ]
  })
}

const totalTraffic = computed(() => {
  return (totalUpload.value + totalDownload.value).toFixed(2)
})

onMounted(async () => {
  await loadInbounds()
  await loadTrafficData()
  
  // 初始化图表
  if (chartRef.value) {
    chartInstance = echarts.init(chartRef.value)
    window.addEventListener('resize', () => chartInstance?.resize())
  }
})
</script>

<style scoped>
.traffic-stats-view {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.mb-4 {
  margin-bottom: 20px;
}

.mt-4 {
  margin-top: 20px;
}
</style>
