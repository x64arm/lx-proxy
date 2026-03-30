// 主题 Store 测试
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useThemeStore } from '../src/stores/theme'

describe('ThemeStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('should initialize with default theme', () => {
    const store = useThemeStore()
    expect(store.theme).toBeDefined()
  })

  it('should set theme to light', () => {
    const store = useThemeStore()
    store.setTheme('light')
    expect(store.theme).toBe('light')
  })

  it('should set theme to dark', () => {
    const store = useThemeStore()
    store.setTheme('dark')
    expect(store.theme).toBe('dark')
  })

  it('should set theme to auto', () => {
    const store = useThemeStore()
    store.setTheme('auto')
    expect(store.theme).toBe('auto')
  })

  it('should toggle theme', () => {
    const store = useThemeStore()
    const initialTheme = store.theme
    
    store.toggleTheme()
    expect(store.theme).not.toBe(initialTheme)
  })

  it('should persist theme to localStorage', () => {
    const store = useThemeStore()
    store.setTheme('dark')
    
    const savedTheme = localStorage.getItem('theme')
    expect(savedTheme).toBe('dark')
  })

  it('should load theme from localStorage on init', () => {
    localStorage.setItem('theme', 'dark')
    
    // 创建新的 store 实例
    setActivePinia(createPinia())
    const store = useThemeStore()
    
    // 应该从 localStorage 加载主题
    expect(localStorage.getItem('theme')).toBe('dark')
  })

  it('should apply dark mode class to document', () => {
    const store = useThemeStore()
    store.setTheme('dark')
    
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('should remove dark mode class when switching to light', () => {
    const store = useThemeStore()
    store.setTheme('dark')
    store.setTheme('light')
    
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })
})
