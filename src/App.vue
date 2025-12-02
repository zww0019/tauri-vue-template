<script setup lang="ts">
import { onMounted } from 'vue'
import { useAuthStore, useToolsStore } from './stores'

const authStore = useAuthStore()
const toolsStore = useToolsStore()

onMounted(async () => {
  // 初始化应用
  console.log('应用初始化...')
  
  // 加载工具列表
  await toolsStore.loadTools()
  
  // 加载已注册工具
  await authStore.loadRegisteredTools()
  
  console.log('应用初始化完成')
})
</script>

<template>
  <div id="app" class="min-h-screen bg-gray-50">
    <router-view />
  </div>
</template>

<style scoped>
#app {
  width: 100%;
  height: 100vh;
  overflow: auto;
}
</style>
