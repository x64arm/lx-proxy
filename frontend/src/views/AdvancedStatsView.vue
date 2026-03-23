<template>
  <div class="advanced-stats">
    <el-row :gutter="20">
      <!-- 流量趋势图表 -->
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>📈 流量趋势分析</span>
              <div class="header-actions">
                <el-radio-group v-model="timeRange" size="small" @change="loadTrafficData">
                  <el-radio-button label="7d">7 天</el-radio-button>
                  <el-radio-button label="30d">30 天</el-radio-button>
                  <el-radio-button label="90d">90 天</el-radio-button>
                </el-radio-group>
              </div>
            </div>
          </template>
          
          <div ref="trafficChartRef" style="height: 400px;"></div>
        </el-card>
      </el-col>

      <!-- 用户流量排行 -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <span>👥 用户流量排行 (Top 10)</span>
          </template>
          
          <el-table :data="topUsers" size="small" stripe>
            <el-table-column type="index" label="#" width="50" />
            <el-table-column prop="username" label="用户" />
            <el-table-column prop="totalTraffic" label="总流量">
              <template #default="{ row }">
                {{ formatBytes(row.totalTraffic) }}
              </template>
            </el-table-column>
            <el-table-column prop="percentage" label="占比">
              <template #default="{ row }">
                <el-progress :percentage="row.percentage" :stroke-width="10" />
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>

      <!-- 协议分布 -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <span>🔧 协议分布</span>
          </template>
          
          <div ref="protocolChartRef" style="height: 300px;"></div>
        </el-card>
      </el-col>

      <!-- 入站配置统计 -->
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <span>📡 入站配置使用率</span>
          </template>
          
          <el-table :data="inboundStats" size="small" stripe>
            <el-table-column prop="tag" label="标签" />
            <el-table-column prop="protocol" label="协议" width="100" />
            <el-table-column prop="trafficUsed" label="已用流量">
              <template #default="{ row }">
                {{ formatBytes(row.trafficUsed) }}
              </template>
            </el-table-column>
            <el-table-column prop="trafficLimit" label="限制流量">
              <template #default="{ row }">
                {{ row.trafficLimit ? formatBytes(row.trafficLimit) : '无限制' }}
              </template>
            </el-table-column>
            <el-table-column prop="usagePercent" label="使用率" width="150">
              <template #default="{ row }">
                <el-progress 
                  :percentage="row.usagePercent" 
                  :status="row.usagePercent > 90 ? 'exception' : row.usagePercent > 70 ? 'warning' : ''"
                />
              </template>
            </el-table-column>
            <el-table-column prop="expireAt" label="过期时间" width="180">
              <template #default="{ row }">
                {{ row.expireAt ? formatDate(row.expireAt) : '永不过期' }}
                <el-tag 
                  v-if="row.expireAt && new Date(row.expireAt) < new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)"
                  type="danger" 
                  size="small"
                >
                  即将到期
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>

      <!-- 活跃时段分析 -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <span>🕐 活跃时段分析</span>
          </template>
          
          <div ref="hourlyChartRef" style="height: 300px;"></div>
        </el-card>
      </el-col>

      <!-- 流量预测 -->
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <span>🔮 流量预测 (未来 7 天)</span>
          </template>
          
          <div ref="forecastChartRef" style="height: 300px;"></div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, nextTick } from 'vue'
import * as echarts from 'echarts'
import type { EChartsOption } from 'echarts'
import api from '@/api'

const timeRange = ref('7d')
const trafficChartRef = ref<HTMLElement>()
const protocolChartRef = ref<HTMLElement>()
const hourlyChartRef = ref<HTMLElement>()
const forecastChartRef = ref<HTMLElement>()

// 数据
const topUsers = ref<any[]>([])
const inboundStats = ref<any[]>([])

// 格式化字节
const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 格式化日期
const formatDate = (date: string) => {
  return new Date(date).toLocaleDateString('zh-CN')
}

// 加载流量数据
const loadTrafficData = async () => {
  try {
    const res = await api.get('/api/stats/advanced', {
      params: { range: timeRange.value }
    })
    
    // 更新图表数据
    updateTrafficChart(res.data.traffic)
    updateTopUsers(res.data.topUsers)
    updateProtocolChart(res.data.protocols)
    updateHourlyChart(res.data.hourly)
    updateForecastChart(res.data.forecast)
    inboundStats.value = res.data.inbounds
  } catch (error) {
    console.error('加载统计数据失败:', error)
  }
}

// 更新流量趋势图表
const updateTrafficChart = (data: any[]) => {
  if (!trafficChartRef.value) return
  
  const chart = echarts.init(trafficChartRef.value)
  
  const option: EChartsOption = {
    tooltip: {
      trigger: 'axis',
      axisPointer: { type: 'shadow' }
    },
    legend: {
      data: ['上传', '下载']
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '3%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: data.map(item => item.date)
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (value: number) => formatBytes(value)
      }
    },
    series: [
      {
        name: '上传',
        type: 'bar',
        stack: 'total',
        data: data.map(item => item.upload),
        itemStyle: { color: '#67C23A' }
      },
      {
        name: '下载',
        type: 'bar',
        stack: 'total',
        data: data.map(item => item.download),
        itemStyle: { color: '#409EFF' }
      }
    ]
  }
  
  chart.setOption(option)
  
  // 响应式调整
  window.addEventListener('resize', () => chart.resize())
}

// 更新用户排行
const updateTopUsers = (users: any[]) => {
  const total = users.reduce((sum, u) => sum + u.totalTraffic, 0)
  topUsers.value = users.map(u => ({
    ...u,
    percentage: total > 0 ? Math.round((u.totalTraffic / total) * 100) : 0
  }))
}

// 更新协议分布图表
const updateProtocolChart = (protocols: any[]) => {
  if (!protocolChartRef.value) return
  
  const chart = echarts.init(protocolChartRef.value)
  
  const option: EChartsOption = {
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c} ({d}%)'
    },
    legend: {
      orient: 'vertical',
      left: 'left'
    },
    series: [
      {
        name: '协议',
        type: 'pie',
        radius: '50%',
        data: protocols.map(p => ({
          name: p.protocol,
          value: p.count
        })),
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)'
          }
        }
      }
    ]
  }
  
  chart.setOption(option)
  window.addEventListener('resize', () => chart.resize())
}

// 更新活跃时段图表
const updateHourlyChart = (hourly: any[]) => {
  if (!hourlyChartRef.value) return
  
  const chart = echarts.init(hourlyChartRef.value)
  
  const option: EChartsOption = {
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: hourly.map(h => `${h.hour}:00`)
    },
    yAxis: {
      type: 'value'
    },
    series: [
      {
        name: '活跃用户数',
        type: 'line',
        smooth: true,
        data: hourly.map(h => h.activeUsers),
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(64, 158, 255, 0.5)' },
            { offset: 1, color: 'rgba(64, 158, 255, 0.05)' }
          ])
        },
        itemStyle: {
          color: '#409EFF'
        }
      }
    ]
  }
  
  chart.setOption(option)
  window.addEventListener('resize', () => chart.resize())
}

// 更新流量预测图表
const updateForecastChart = (forecast: any[]) => {
  if (!forecastChartRef.value) return
  
  const chart = echarts.init(forecastChartRef.value)
  
  const option: EChartsOption = {
    tooltip: {
      trigger: 'axis'
    },
    xAxis: {
      type: 'category',
      data: forecast.map(f => f.date)
    },
    yAxis: {
      type: 'value',
      axisLabel: {
        formatter: (value: number) => formatBytes(value)
      }
    },
    series: [
      {
        name: '预测流量',
        type: 'line',
        smooth: true,
        data: forecast.map(f => f.predicted),
        lineStyle: {
          type: 'dashed',
          color: '#E6A23C'
        },
        itemStyle: {
          color: '#E6A23C'
        }
      }
    ]
  }
  
  chart.setOption(option)
  window.addEventListener('resize', () => chart.resize())
}

onMounted(() => {
  nextTick(() => {
    loadTrafficData()
  })
})
</script>

<style scoped>
.advanced-stats {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 8px;
}
</style>
