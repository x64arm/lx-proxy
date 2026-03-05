import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import './style.css'
import App from './App.vue'

// 视图组件
import LoginView from './views/LoginView.vue'
import DashboardView from './views/DashboardView.vue'

// 简单的路由守卫
const routes = [
  { path: '/login', component: LoginView },
  { path: '/dashboard', component: DashboardView },
  { path: '/', redirect: '/dashboard' }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 简单的登录检查
router.beforeEach((to, from, next) => {
  const token = localStorage.getItem('token')
  if (to.path !== '/login' && !token) {
    next('/login')
  } else {
    next()
  }
})

const app = createApp(App)
app.use(router)
app.mount('#app')
