// Vitest 测试设置文件
import { config } from '@vue/test-utils'
import { createI18n } from './src/i18n'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'

// 全局配置
config.global.plugins = [
  createPinia(),
  createI18n({
    legacy: false,
    locale: 'zh-CN',
    fallbackLocale: 'en-US',
    messages: {},
  }),
  ElementPlus,
]

// 全局 mock
global.fetch = vi.fn()

// 重置 mocks
beforeEach(() => {
  vi.clearAllMocks()
})
