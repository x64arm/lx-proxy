import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import axios from 'axios'
import './style.css'
import App from './App.vue'

// 视图组件
import LoginView from './views/LoginView.vue'
import DashboardView from './views/DashboardView.vue'
import UsersView from './views/UsersView.vue'
import InboundsView from './views/InboundsView.vue'

// API 请求拦截器
axios.interceptors.request.use(config => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

axios.interceptors.response.use(
  response => response,
  error => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

// 路由配置
const routes = [
  { path: '/login', component: LoginView, meta: { title: '登录' } },
  { 
    path: '/dashboard', 
    component: DashboardView, 
    meta: { title: '仪表盘', requiresAuth: true } 
  },
  { 
    path: '/users', 
    component: UsersView, 
    meta: { title: '用户管理', requiresAuth: true } 
  },
  { 
    path: '/inbounds', 
    component: InboundsView, 
    meta: { title: '入站配置', requiresAuth: true } 
  },
  { path: '/', redirect: '/dashboard' }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫
router.beforeEach((to, _from, next) => {
  const token = localStorage.getItem('token')
  
  // 设置页面标题
  if (to.meta.title) {
    document.title = `${to.meta.title} - LX-Proxy`
  }
  
  // 需要认证的检查
  if (to.meta.requiresAuth && !token) {
    next('/login')
  } else if (to.path === '/login' && token) {
    next('/dashboard')
  } else {
    next()
  }
})

const app = createApp(App)

// 注册 Pinia
app.use(createPinia())

// 注册路由
app.use(router)

// 注册 Element Plus
app.use(ElementPlus)

// 注册所有图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.mount('#app')
