import axios from 'axios'

const API_BASE = '/api'

// 认证相关
export const authAPI = {
  login: (username: string, password: string) => 
    axios.post(`${API_BASE}/auth/login`, { username, password }),
  logout: () => axios.post(`${API_BASE}/auth/logout`),
  getCurrentUser: () => axios.get(`${API_BASE}/auth/me`)
}

// 用户管理
export interface User {
  id: number
  username: string
  role: 'admin' | 'user'
  created_at: string
  updated_at: string
}

export const usersAPI = {
  list: () => axios.get<User[]>(`${API_BASE}/users`),
  get: (id: number) => axios.get<User>(`${API_BASE}/users/${id}`),
  create: (data: { username: string; password: string; role: string }) => 
    axios.post<User>(`${API_BASE}/users`, data),
  update: (id: number, data: Partial<User>) => 
    axios.put<User>(`${API_BASE}/users/${id}`, data),
  delete: (id: number) => axios.delete(`${API_BASE}/users/${id}`)
}

// 入站配置
export interface Inbound {
  id: number
  user_id: number
  protocol: 'vmess' | 'vless' | 'trojan' | 'shadowsocks'
  port: number
  enable: boolean
  remark: string
  total: number
  up: number
  down: number
  expiry_time: number | null
  created_at: string
  updated_at: string
}

export const inboundsAPI = {
  list: () => axios.get<Inbound[]>(`${API_BASE}/inbounds`),
  get: (id: number) => axios.get<Inbound>(`${API_BASE}/inbounds/${id}`),
  create: (data: Partial<Inbound>) => axios.post<Inbound>(`${API_BASE}/inbounds`, data),
  update: (id: number, data: Partial<Inbound>) => 
    axios.put<Inbound>(`${API_BASE}/inbounds/${id}`, data),
  delete: (id: number) => axios.delete(`${API_BASE}/inbounds/${id}`),
  reset: (id: number) => axios.post(`${API_BASE}/inbounds/${id}/reset`),
  getLinks: (id: number) => axios.get(`${API_BASE}/inbounds/${id}/links`)
}

// 统计
export interface Stats {
  total_users: number
  total_inbounds: number
  enabled_inbounds: number
  total_traffic_used: number
  total_traffic_limit: number | null
}

export interface SystemStatus {
  cpu_usage: number
  memory_total: number
  memory_used: number
  memory_free: number
  uptime: number
  xray_running: boolean
  connections: number
}

export const statsAPI = {
  getStats: () => axios.get<Stats>(`${API_BASE}/stats`),
  getSystemStatus: () => axios.get<SystemStatus>(`${API_BASE}/system/status`)
}
