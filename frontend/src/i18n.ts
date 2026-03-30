import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN.json'
import enUS from './locales/en-US.json'
import zhTW from './locales/zh-TW.json'

const messages = {
  'zh-CN': zhCN,
  'en-US': enUS,
  'zh-TW': zhTW,
}

// 获取用户语言偏好
function getUserLanguage() {
  const saved = localStorage.getItem('locale')
  if (saved) return saved
  
  const browserLang = navigator.language
  if (browserLang.startsWith('zh-CN')) return 'zh-CN'
  if (browserLang.startsWith('zh-TW') || browserLang.startsWith('zh-HK')) return 'zh-TW'
  return 'en-US'
}

const i18n = createI18n({
  legacy: false,
  locale: getUserLanguage(),
  fallbackLocale: 'en-US',
  messages,
  globalInjection: true,
})

// 切换语言
export function setLocale(locale: string) {
  i18n.global.locale.value = locale as any
  localStorage.setItem('locale', locale)
  document.documentElement.lang = locale
}

// 获取当前语言
export function getCurrentLocale() {
  return i18n.global.locale.value
}

// 获取支持的语言列表
export function getSupportedLocales() {
  return [
    { code: 'zh-CN', name: '简体中文', flag: '🇨🇳' },
    { code: 'zh-TW', name: '繁體中文', flag: '🇹🇼' },
    { code: 'en-US', name: 'English', flag: '🇺🇸' },
  ]
}

export default i18n
