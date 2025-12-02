<script setup lang="ts">
import { onMounted, ref, onUnmounted } from 'vue'
import { useAuthStore, useToolsStore } from './stores'
import { listen } from '@tauri-apps/api/event'

const authStore = useAuthStore()
const toolsStore = useToolsStore()

// 下载进度状态
const downloadProgress = ref<Record<string, {
  toolId: string
  downloadedSize: number
  totalSize: number
  progress: number
  status: 'downloading' | 'installing' | 'completed' | 'failed'
  error?: string
}>>({})

// 监听工具更新进度事件
let unlistenProgress: (() => void) | null = null
let unlistenStatus: (() => void) | null = null

onMounted(async () => {
  // 初始化应用
  console.log('应用初始化...')
  
  // 加载工具列表
  await toolsStore.loadTools()
  
  // 加载已注册工具
  await authStore.loadRegisteredTools()
  
  // 监听工具更新进度
  unlistenProgress = await listen('tool:update-progress', (event: any) => {
    const data = event.payload
    if (data.toolId) {
      downloadProgress.value[data.toolId] = {
        toolId: data.toolId,
        downloadedSize: data.downloadedSize || 0,
        totalSize: data.totalSize || 0,
        progress: data.progress || 0,
        status: 'downloading'
      }
    }
  })
  
  // 监听工具更新状态
  unlistenStatus = await listen('tool:update-status', (event: any) => {
    const data = event.payload
    if (data.toolId) {
      if (!downloadProgress.value[data.toolId]) {
        downloadProgress.value[data.toolId] = {
          toolId: data.toolId,
          downloadedSize: 0,
          totalSize: 0,
          progress: 0,
          status: data.status
        }
      } else {
        downloadProgress.value[data.toolId].status = data.status
        if (data.error) {
          downloadProgress.value[data.toolId].error = data.error
        }
      }
      
      // 如果完成或失败，3秒后清除进度
      if (data.status === 'completed' || data.status === 'failed') {
        setTimeout(() => {
          delete downloadProgress.value[data.toolId]
        }, 3000)
      }
    }
  })
  
  // 监听工具自动更新完成事件，重新加载工具列表
  const unlistenUpdated = await listen('tool:auto-updated', async () => {
    await toolsStore.loadTools()
  })
  
  console.log('应用初始化完成')
})

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
  if (unlistenStatus) {
    unlistenStatus()
  }
})

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<template>
  <div id="app" class="min-h-screen bg-gray-50">
    <router-view />
    
    <!-- 全局下载进度提示 -->
    <div
      v-if="Object.keys(downloadProgress).length > 0"
      class="fixed bottom-4 right-4 z-50 space-y-2 max-w-md"
    >
      <div
        v-for="(progress, toolId) in downloadProgress"
        :key="toolId"
        class="bg-white rounded-lg shadow-lg border border-gray-200 p-4"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-gray-900">
            {{ progress.status === 'downloading' ? '下载中' : 
               progress.status === 'installing' ? '安装中' : 
               progress.status === 'completed' ? '完成' : '失败' }}
          </span>
          <span class="text-xs text-gray-500">{{ toolId }}</span>
        </div>
        
        <div v-if="progress.status === 'downloading' || progress.status === 'installing'" class="space-y-1">
          <div class="w-full bg-gray-200 rounded-full h-2">
            <div
              class="bg-blue-600 h-2 rounded-full transition-all duration-300"
              :style="{ width: progress.progress + '%' }"
            ></div>
          </div>
          <div class="flex justify-between text-xs text-gray-600">
            <span>{{ formatBytes(progress.downloadedSize) }} / {{ formatBytes(progress.totalSize) }}</span>
            <span>{{ Math.round(progress.progress) }}%</span>
          </div>
        </div>
        
        <div v-else-if="progress.status === 'completed'" class="text-sm text-green-600">
          ✓ 工具更新完成
        </div>
        
        <div v-else-if="progress.status === 'failed'" class="text-sm text-red-600">
          ✗ {{ progress.error || '更新失败' }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
#app {
  width: 100%;
  height: 100vh;
  overflow: auto;
}
</style>
