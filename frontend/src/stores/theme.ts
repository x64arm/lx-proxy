/// 主题状态管理（暗黑模式）

import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'auto'

const THEME_KEY = 'lx-proxy-theme'

// 主题状态
const currentTheme = ref<ThemeMode>('light')
const isDark = ref(false)

// 初始化主题
export function initTheme() {
  const saved = localStorage.getItem(THEME_KEY) as ThemeMode | null
  currentTheme.value = saved || 'light'
  applyTheme(currentTheme.value)
}

// 应用主题
function applyTheme(mode: ThemeMode) {
  const html = document.documentElement
  
  if (mode === 'auto') {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    isDark.value = prefersDark
  } else {
    isDark.value = mode === 'dark'
  }

  if (isDark.value) {
    html.classList.add('dark')
    html.setAttribute('data-theme', 'dark')
  } else {
    html.classList.remove('dark')
    html.setAttribute('data-theme', 'light')
  }

  // 更新 Element Plus 主题
  updateElementPlusTheme()
}

// 更新 Element Plus 主题
function updateElementPlusTheme() {
  const cssVars = isDark.value ? {
    // 暗黑模式变量
    '--el-color-primary': '#409EFF',
    '--el-color-success': '#67C23A',
    '--el-color-warning': '#E6A23C',
    '--el-color-danger': '#F56C6C',
    '--el-color-info': '#909399',
    '--el-bg-color': '#141414',
    '--el-bg-color-page': '#0a0a0a',
    '--el-bg-color-overlay': '#1d1e1f',
    '--el-text-color-primary': '#E5EAF3',
    '--el-text-color-regular': '#CFD3DC',
    '--el-text-color-secondary': '#A3A6AD',
    '--el-text-color-placeholder': '#8D9095',
    '--el-text-color-disabled': '#6C6C6C',
    '--el-border-color': '#434343',
    '--el-border-color-light': '#4C4C4C',
    '--el-fill-color': '#262727',
    '--el-fill-color-light': '#1F2020',
  } : {
    // 明亮模式变量
    '--el-color-primary': '#409EFF',
    '--el-color-success': '#67C23A',
    '--el-color-warning': '#E6A23C',
    '--el-color-danger': '#F56C6C',
    '--el-color-info': '#909399',
  }

  Object.entries(cssVars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value)
  })
}

// 设置主题
export function setTheme(mode: ThemeMode) {
  currentTheme.value = mode
  localStorage.setItem(THEME_KEY, mode)
  applyTheme(mode)
}

// 切换暗黑/明亮
export function toggleTheme() {
  setTheme(isDark.value ? 'light' : 'dark')
}

// 监听系统主题变化
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    if (currentTheme.value === 'auto') {
      isDark.value = e.matches
      applyTheme('auto')
    }
  })
}

// 导出供组件使用
export function useTheme() {
  return {
    currentTheme,
    isDark,
    setTheme,
    toggleTheme,
    initTheme
  }
}
