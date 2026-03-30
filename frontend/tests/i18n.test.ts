// 国际化 (i18n) 测试
import { describe, it, expect } from 'vitest'
import { createI18n } from '../src/i18n'
import zhCN from '../src/i18n/locales/zh-CN.json'
import enUS from '../src/i18n/locales/en-US.json'
import zhTW from '../src/i18n/locales/zh-TW.json'

describe('i18n', () => {
  const i18n = createI18n({
    legacy: false,
    locale: 'zh-CN',
    fallbackLocale: 'en-US',
    messages: {
      'zh-CN': zhCN,
      'en-US': enUS,
      'zh-TW': zhTW,
    },
  })

  it('should have all required language packs', () => {
    expect(i18n.global.availableLocales).toContain('zh-CN')
    expect(i18n.global.availableLocales).toContain('en-US')
    expect(i18n.global.availableLocales).toContain('zh-TW')
  })

  it('should have default locale set to zh-CN', () => {
    expect(i18n.global.locale.value).toBe('zh-CN')
  })

  it('should have common translations in zh-CN', () => {
    expect(zhCN.common).toBeDefined()
    expect(zhCN.common.loading).toBeDefined()
    expect(zhCN.common.success).toBeDefined()
    expect(zhCN.common.error).toBeDefined()
  })

  it('should have common translations in en-US', () => {
    expect(enUS.common).toBeDefined()
    expect(enUS.common.loading).toBeDefined()
    expect(enUS.common.success).toBeDefined()
    expect(enUS.common.error).toBeDefined()
  })

  it('should have common translations in zh-TW', () => {
    expect(zhTW.common).toBeDefined()
    expect(zhTW.common.loading).toBeDefined()
    expect(zhTW.common.success).toBeDefined()
    expect(zhTW.common.error).toBeDefined()
  })

  it('should have navigation translations', () => {
    expect(zhCN.nav.dashboard).toBeDefined()
    expect(zhCN.nav.users).toBeDefined()
    expect(zhCN.nav.inbounds).toBeDefined()
    expect(zhCN.nav.traffic).toBeDefined()
    expect(zhCN.nav.settings).toBeDefined()
  })

  it('should have consistent translation structure across languages', () => {
    const zhKeys = Object.keys(zhCN).sort()
    const enKeys = Object.keys(enUS).sort()
    const twKeys = Object.keys(zhTW).sort()
    
    expect(zhKeys).toEqual(enKeys)
    expect(zhKeys).toEqual(twKeys)
  })

  it('should change locale', () => {
    i18n.global.locale.value = 'en-US'
    expect(i18n.global.locale.value).toBe('en-US')
    
    i18n.global.locale.value = 'zh-TW'
    expect(i18n.global.locale.value).toBe('zh-TW')
    
    // 恢复默认
    i18n.global.locale.value = 'zh-CN'
  })

  it('should translate with parameters', () => {
    const { t } = i18n.global
    
    // 测试带参数的翻译
    const message = t('common.confirm', { action: '删除' })
    expect(message).toBeDefined()
  })

  it('should handle missing translations gracefully', () => {
    const { t } = i18n.global
    
    // 应该返回键名或 fallback
    const missing = t('nonexistent.key')
    expect(missing).toBeDefined()
  })
})
