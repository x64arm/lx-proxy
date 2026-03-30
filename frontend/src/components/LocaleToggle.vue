<template>
  <el-dropdown trigger="click" @command="handleLocaleChange">
    <el-button circle :title="`当前语言：${currentLocaleName}`">
      <span class="locale-flag">{{ currentLocale.flag }}</span>
    </el-button>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item
          v-for="locale in supportedLocales"
          :key="locale.code"
          :command="locale.code"
          :disabled="locale.code === currentLocale.code"
        >
          <span class="locale-flag">{{ locale.flag }}</span>
          {{ locale.name }}
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { setLocale, getCurrentLocale, getSupportedLocales } from '../i18n'
import { useI18n } from 'vue-i18n'

const { locale } = useI18n()
const supportedLocales = getSupportedLocales()

const currentLocale = ref(supportedLocales[0])

onMounted(() => {
  const code = getCurrentLocale()
  const found = supportedLocales.find(l => l.code === code)
  if (found) {
    currentLocale.value = found
  }
})

const currentLocaleName = computed(() => currentLocale.value.name)

const handleLocaleChange = (code: string) => {
  const newLocale = supportedLocales.find(l => l.code === code)
  if (newLocale) {
    currentLocale.value = newLocale
    setLocale(code)
    locale.value = code as any
  }
}
</script>

<style scoped>
.locale-flag {
  margin-right: 8px;
  font-size: 16px;
}

.el-button {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 20px;
}

.el-button:hover {
  background: rgba(255, 255, 255, 0.1);
}
</style>
