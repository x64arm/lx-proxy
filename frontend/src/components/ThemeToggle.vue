<template>
  <el-dropdown trigger="click" @command="handleCommand">
    <el-button circle :title="`当前主题：${themeNames[currentTheme]}`">
      <el-icon v-if="isDark">
        <Moon />
      </el-icon>
      <el-icon v-else-if="currentTheme === 'light'">
        <Sunny />
      </el-icon>
      <el-icon v-else>
        <Monitor />
      </el-icon>
    </el-button>
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="light" :disabled="currentTheme === 'light'">
          <el-icon><Sunny /></el-icon>
          明亮模式
        </el-dropdown-item>
        <el-dropdown-item command="dark" :disabled="currentTheme === 'dark'">
          <el-icon><Moon /></el-icon>
          暗黑模式
        </el-dropdown-item>
        <el-dropdown-item command="auto" :disabled="currentTheme === 'auto'">
          <el-icon><Monitor /></el-icon>
          跟随系统
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Sunny, Moon, Monitor } from '@element-plus/icons-vue'
import { useTheme, type ThemeMode } from '@/stores/theme'

const { currentTheme, isDark, setTheme, initTheme } = useTheme()

const themeNames: Record<ThemeMode, string> = {
  light: '明亮',
  dark: '暗黑',
  auto: '自动'
}

onMounted(() => {
  initTheme()
})

const handleCommand = (command: ThemeMode) => {
  setTheme(command)
}
</script>

<style scoped>
.el-button {
  border: none;
  background: transparent;
  color: inherit;
}

.el-button:hover {
  background: rgba(255, 255, 255, 0.1);
}
</style>
