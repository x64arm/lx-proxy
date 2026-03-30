<template>
  <el-container class="layout-container">
    <!-- 移动端侧边栏抽屉 -->
    <el-drawer
      v-model="mobileSidebarVisible"
      direction="ltr"
      :size="280"
      class="mobile-sidebar-drawer"
      :with-header="false"
      v-if="isMobile"
    >
      <div class="mobile-sidebar">
        <div class="logo">
          <span>🚀 LX-Proxy</span>
        </div>
        
        <el-menu
          :default-active="activeMenu"
          :collapse-transition="false"
          router
          background-color="#304156"
          text-color="#bfcbd9"
          active-text-color="#409EFF"
          @select="mobileSidebarVisible = false"
        >
          <el-menu-item index="/dashboard">
            <el-icon><HomeFilled /></el-icon>
            <template #title>{{ t('layout.dashboard') }}</template>
          </el-menu-item>
          
          <el-menu-item index="/users">
            <el-icon><User /></el-icon>
            <template #title>{{ t('layout.users') }}</template>
          </el-menu-item>
          
          <el-menu-item index="/inbounds">
            <el-icon><Connection /></el-icon>
            <template #title>{{ t('layout.inbounds') }}</template>
          </el-menu-item>
          
          <el-menu-item index="/traffic">
            <el-icon><DataLine /></el-icon>
            <template #title>{{ t('layout.traffic') }}</template>
          </el-menu-item>
          
          <el-menu-item index="/subscription">
            <el-icon><Link /></el-icon>
            <template #title>{{ t('layout.subscription') }}</template>
          </el-menu-item>
          
          <el-sub-menu index="security">
            <template #title>
              <el-icon><Lock /></el-icon>
              <span>{{ t('layout.totpSettings') }}</span>
            </template>
            <el-menu-item index="/totp">
              <el-icon><Key /></el-icon>
              <template #title>{{ t('layout.totpSettings') }}</template>
            </el-menu-item>
          </el-sub-menu>
          
          <el-sub-menu index="notifications">
            <template #title>
              <el-icon><Bell /></el-icon>
              <span>{{ t('layout.emailSettings') }}</span>
            </template>
            <el-menu-item index="/email-settings">
              <el-icon><Message /></el-icon>
              <template #title>{{ t('layout.emailSettings') }}</template>
            </el-menu-item>
          </el-sub-menu>
          
          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <template #title>{{ t('layout.settings') }}</template>
          </el-menu-item>
        </el-menu>
      </div>
    </el-drawer>

    <!-- 桌面端侧边栏 -->
    <el-aside 
      :width="isCollapse ? '64px' : '220px'" 
      class="sidebar"
      :class="{ 'desktop-sidebar': !isMobile }"
      v-show="!isMobile"
    >
      <div class="logo">
        <span v-if="!isCollapse">🚀 LX-Proxy</span>
        <span v-else>🚀</span>
      </div>
      
      <el-menu
        :default-active="activeMenu"
        :collapse="isCollapse"
        :collapse-transition="false"
        router
        background-color="#304156"
        text-color="#bfcbd9"
        active-text-color="#409EFF"
      >
        <el-menu-item index="/dashboard">
          <el-icon><HomeFilled /></el-icon>
          <template #title>{{ t('layout.dashboard') }}</template>
        </el-menu-item>
        
        <el-menu-item index="/users">
          <el-icon><User /></el-icon>
          <template #title>{{ t('layout.users') }}</template>
        </el-menu-item>
        
        <el-menu-item index="/inbounds">
          <el-icon><Connection /></el-icon>
          <template #title>{{ t('layout.inbounds') }}</template>
        </el-menu-item>
        
        <el-menu-item index="/traffic">
          <el-icon><DataLine /></el-icon>
          <template #title>{{ t('layout.traffic') }}</template>
        </el-menu-item>
        
        <el-menu-item index="/subscription">
          <el-icon><Link /></el-icon>
          <template #title>{{ t('layout.subscription') }}</template>
        </el-menu-item>
        
        <el-sub-menu index="security">
          <template #title>
            <el-icon><Lock /></el-icon>
            <span>{{ t('layout.totpSettings') }}</span>
          </template>
          <el-menu-item index="/totp">
            <el-icon><Key /></el-icon>
            <template #title>{{ t('layout.totpSettings') }}</template>
          </el-menu-item>
        </el-sub-menu>
        
        <el-sub-menu index="notifications">
          <template #title>
            <el-icon><Bell /></el-icon>
            <span>{{ t('layout.emailSettings') }}</span>
          </template>
          <el-menu-item index="/email-settings">
            <el-icon><Message /></el-icon>
            <template #title>{{ t('layout.emailSettings') }}</template>
          </el-menu-item>
        </el-sub-menu>
        
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <template #title>{{ t('layout.settings') }}</template>
        </el-menu-item>
      </el-menu>
    </el-aside>

    <el-container>
      <!-- 顶部导航 -->
      <el-header class="header">
        <div class="header-left">
          <!-- 移动端汉堡菜单按钮 -->
          <el-icon 
            v-if="isMobile" 
            class="mobile-menu-btn" 
            @click="mobileSidebarVisible = true"
          >
            <Menu />
          </el-icon>
          
          <!-- 桌面端折叠按钮 -->
          <el-icon 
            v-else 
            class="collapse-btn" 
            @click="toggleCollapse"
          >
            <Fold v-if="!isCollapse" />
            <Expand v-else />
          </el-icon>
          
          <el-breadcrumb separator="/" class="breadcrumb">
            <el-breadcrumb-item :to="{ path: '/dashboard' }">
              {{ t('layout.dashboard') }}
            </el-breadcrumb-item>
            <el-breadcrumb-item v-if="currentTitle">
              {{ currentTitle }}
            </el-breadcrumb-item>
          </el-breadcrumb>
        </div>
        
        <div class="header-right">
          <!-- 语言切换 - 移动端隐藏图标，只显示文字 -->
          <LocaleToggle class="locale-toggle" />
          
          <!-- 主题切换 -->
          <ThemeToggle class="theme-toggle" />
          
          <el-dropdown @command="handleCommand">
            <span class="user-info">
              <el-avatar :size="isMobile ? 28 : 32" :icon="UserFilled" />
              <span class="username" :class="{ 'mobile-username': isMobile }">
                {{ currentUser?.username || t('layout.user') }}
              </span>
            </span>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item command="profile">
                  👤 {{ t('users.title') }}
                </el-dropdown-item>
                <el-dropdown-item command="logout" divided>
                  🚪 {{ t('login.logout') }}
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </el-header>

      <!-- 主内容区 -->
      <el-main class="main-content" :class="{ 'mobile-main': isMobile }">
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessageBox } from 'element-plus'
import {
  HomeFilled,
  User,
  UserFilled,
  Connection,
  DataLine,
  Link,
  Setting,
  Fold,
  Expand,
  Lock,
  Key,
  Bell,
  Message,
  Menu
} from '@element-plus/icons-vue'
import ThemeToggle from './ThemeToggle.vue'
import LocaleToggle from './LocaleToggle.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const router = useRouter()
const route = useRoute()

// 响应式状态
const isMobile = ref(false)
const isTablet = ref(false)
const isCollapse = ref(false)
const mobileSidebarVisible = ref(false)
const currentUser = ref<any>(null)

// 检测设备类型
const checkDevice = () => {
  const width = window.innerWidth
  isMobile.value = width <= 768
  isTablet.value = width > 768 && width <= 1024
}

// 监听窗口大小变化
onMounted(() => {
  checkDevice()
  window.addEventListener('resize', checkDevice)
  
  const userStr = localStorage.getItem('user')
  if (userStr) {
    try {
      currentUser.value = JSON.parse(userStr)
    } catch (e) {
      console.error('Failed to parse user info:', e)
    }
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', checkDevice)
})

const activeMenu = computed(() => route.path)
const currentTitle = computed(() => route.meta.title as string || '')

const toggleCollapse = () => {
  isCollapse.value = !isCollapse.value
}

const handleCommand = (command: string) => {
  if (command === 'logout') {
    ElMessageBox.confirm(t('layout.confirmLogout'), t('layout.tip'), {
      confirmButtonText: t('common.yes'),
      cancelButtonText: t('common.no'),
      type: 'warning'
    }).then(() => {
      localStorage.removeItem('token')
      localStorage.removeItem('user')
      router.push('/login')
    }).catch(() => {})
  } else if (command === 'profile') {
    ElMessageBox.alert(
      t('layout.currentUser') + ': ' + currentUser.value?.username,
      t('layout.profile')
    )
  }
}
</script>

<style scoped>
.layout-container {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

/* ========== 桌面端侧边栏 ========== */
.desktop-sidebar {
  background-color: #304156;
  transition: width 0.3s;
  overflow-x: hidden;
}

.sidebar {
  background-color: #304156;
  transition: width 0.3s;
  overflow-x: hidden;
}

.logo {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 20px;
  font-weight: bold;
  background-color: #2b3a4b;
  white-space: nowrap;
}

.el-menu {
  border-right: none;
}

/* ========== 移动端侧边栏 ========== */
.mobile-sidebar-drawer {
  background-color: #304156;
}

.mobile-sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.mobile-sidebar .logo {
  height: 60px;
  background-color: #2b3a4b;
}

.mobile-sidebar .el-menu {
  flex: 1;
  overflow-y: auto;
}

/* 移动端菜单项点击区域优化 */
:deep(.mobile-sidebar .el-menu-item) {
  min-height: 50px;
}

:deep(.mobile-sidebar .el-sub-menu__title) {
  min-height: 50px;
}

/* ========== 顶部导航 ========== */
.header {
  background-color: #fff;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  height: 60px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.collapse-btn,
.mobile-menu-btn {
  font-size: 20px;
  cursor: pointer;
  transition: color 0.3s;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 44px;
  min-height: 44px;
}

.collapse-btn:hover,
.mobile-menu-btn:hover {
  color: #409EFF;
}

.breadcrumb {
  flex: 1;
}

/* 移动端隐藏面包屑 */
@media (max-width: 768px) {
  .breadcrumb {
    display: none;
  }
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.locale-toggle,
.theme-toggle {
  margin-right: 8px;
}

/* 移动端隐藏语言和主题切换器 */
@media (max-width: 768px) {
  .locale-toggle,
  .theme-toggle {
    display: none;
  }
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 8px;
  border-radius: 4px;
  transition: background-color 0.3s;
}

.user-info:hover {
  background-color: #f5f7fa;
}

.username {
  color: #606266;
  font-size: 14px;
  white-space: nowrap;
}

/* 移动端用户名隐藏 */
@media (max-width: 768px) {
  .username {
    display: none;
  }
}

/* ========== 主内容区 ========== */
.main-content {
  background-color: #f0f2f5;
  padding: 20px;
  overflow-y: auto;
  height: calc(100vh - 60px);
}

/* 移动端主内容区优化 */
.mobile-main {
  padding: 12px;
}

@media (max-width: 768px) {
  .header {
    padding: 0 12px;
  }
  
  .main-content {
    padding: 12px;
    height: calc(100vh - 60px);
  }
}

/* ========== 响应式工具类 ========== */
/* 触摸友好的最小点击区域 */
:deep(.el-button),
:deep(.el-menu-item),
:deep(.el-sub-menu__title) {
  min-height: 44px;
}

/* 防止水平滚动 */
* {
  box-sizing: border-box;
}

.layout-container {
  max-width: 100vw;
  overflow-x: hidden;
}
</style>
