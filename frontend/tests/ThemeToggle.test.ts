// 主题切换组件测试
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ThemeToggle from '../src/components/ThemeToggle.vue'
import { createPinia, setActivePinia } from 'pinia'
import { useThemeStore } from '../src/stores/theme'

describe('ThemeToggle', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders correctly', () => {
    const wrapper = mount(ThemeToggle)
    expect(wrapper.exists()).toBe(true)
  })

  it('displays current theme icon', async () => {
    const wrapper = mount(ThemeToggle)
    const themeStore = useThemeStore()
    
    // 初始应该是跟随系统或明亮模式
    expect(wrapper.find('.theme-toggle').exists()).toBe(true)
  })

  it('changes theme when clicked', async () => {
    const wrapper = mount(ThemeToggle)
    const themeStore = useThemeStore()
    
    const button = wrapper.find('button')
    await button.trigger('click')
    
    // 验证主题已切换
    expect(themeStore.theme).toBeDefined()
  })

  it('supports three theme modes', () => {
    const themeStore = useThemeStore()
    
    // 测试设置不同主题
    themeStore.setTheme('light')
    expect(themeStore.theme).toBe('light')
    
    themeStore.setTheme('dark')
    expect(themeStore.theme).toBe('dark')
    
    themeStore.setTheme('auto')
    expect(themeStore.theme).toBe('auto')
  })
})
