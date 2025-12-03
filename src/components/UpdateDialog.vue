<template>
  <div
    v-if="visible"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    @click.self="handleClose"
  >
    <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-2xl max-h-[80vh] overflow-y-auto">
      <h2 class="text-2xl font-bold mb-4">软件更新</h2>
      
      <div v-if="updateInfo.hasUpdate" class="mb-4">
        <p class="text-gray-600 mb-2">
          发现新版本: <span class="font-semibold">{{ updateInfo.latestVersion }}</span>
        </p>
        <p class="text-gray-600 mb-4">当前版本: {{ currentVersion }}</p>
        
        <div v-if="updateInfo.releaseNotes" class="mb-4">
          <h3 class="font-semibold mb-2">更新内容：</h3>
          <div class="bg-gray-50 p-4 rounded border border-gray-200 whitespace-pre-wrap">
            {{ updateInfo.releaseNotes }}
          </div>
        </div>
        
        <div v-if="downloading" class="mb-4">
          <div class="flex items-center gap-2">
            <div class="flex-1 bg-gray-200 rounded-full h-2">
              <div
                class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                :style="{ width: `${downloadProgress}%` }"
              ></div>
            </div>
            <span class="text-sm text-gray-600">{{ downloadProgress }}%</span>
          </div>
        </div>
      </div>
      
      <div v-else class="mb-4">
        <p class="text-gray-600">当前已是最新版本</p>
      </div>
      
      <div class="flex justify-end gap-3">
        <button
          v-if="!updateInfo.hasUpdate"
          @click="handleClose"
          class="px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700"
        >
          确定
        </button>
        <template v-else>
          <button
            @click="handleClose"
            class="px-4 py-2 text-gray-700 bg-gray-200 rounded-md hover:bg-gray-300"
          >
            稍后
          </button>
          <button
            @click="handleDownload"
            :disabled="downloading"
            class="px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            {{ downloading ? '下载中...' : '立即更新' }}
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  visible: boolean
  updateInfo: {
    hasUpdate: boolean
    latestVersion: string
    releaseNotes: string
    downloadUrl: string
  }
  currentVersion: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'download'): void
}>()

const downloading = ref(false)
const downloadProgress = ref(0)

import { invoke } from '@tauri-apps/api/core'

const handleDownload = async () => {
  downloading.value = true
  downloadProgress.value = 0
  
  try {
    // 模拟下载进度
    const progressInterval = setInterval(() => {
      downloadProgress.value += 10
      if (downloadProgress.value >= 90) {
        clearInterval(progressInterval)
      }
    }, 500)
    
    // 实际下载更新
    if (props.updateInfo.downloadUrl) {
      const installerPath = await invoke<string>('download_update', {
        downloadUrl: props.updateInfo.downloadUrl
      })
      
      downloadProgress.value = 100
      clearInterval(progressInterval)
      
      // 安装更新
      await invoke('install_update', { installerPath })
      
      downloading.value = false
      emit('download')
    } else {
      // 如果没有下载链接，只模拟下载
      setTimeout(() => {
        downloadProgress.value = 100
        downloading.value = false
        emit('download')
      }, 2000)
    }
  } catch (error: any) {
    console.error('下载更新失败:', error)
    downloading.value = false
    alert('下载更新失败: ' + (error || '未知错误'))
  }
}

const handleClose = () => {
  emit('close')
}
</script>

