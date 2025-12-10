<script setup lang="ts">
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref } from 'vue'
import ExpiryWarning from './components/ExpiryWarning.vue'
import LicenseDialog from './components/LicenseDialog.vue'
import SimilarityApp from './components/similarity/SimilarityApp.vue'
import UpdateDialog from './components/UpdateDialog.vue'

const store = useStore()
const showTest = ref(false)

// 定时检测授权有效期（每天检测一次）
let expiryCheckInterval: number | null = null
let unlistenResetLicense: UnlistenFn | null = null

onMounted(async () => {
  await store.initApp()
  
  // 监听菜单重置授权事件
  unlistenResetLicense = await listen('menu-reset-license', async () => {
    await store.resetLicense()
  })
  
  // 设置定时检测（每24小时检测一次）
  expiryCheckInterval = window.setInterval(async () => {
    if (store.isLicenseRegistered) {
      await store.checkLicenseExpiry()
    }
  }, 24 * 60 * 60 * 1000) // 24小时
})

onUnmounted(() => {
  if (expiryCheckInterval !== null) {
    clearInterval(expiryCheckInterval)
  }
  if (unlistenResetLicense !== null) {
    unlistenResetLicense()
  }
})
</script>

<template>
  <div class="min-h-screen">
    <!-- 授权注册弹窗 -->
    <LicenseDialog
      :visible="store.showLicenseDialog"
      @verified="store.onLicenseVerified"
      @cancel="store.onLicenseDialogCancel"
    />
    
    <!-- 过期警告提示 -->
    <ExpiryWarning
      :visible="store.showExpiryWarning"
      :days-remaining="store.expiryDaysRemaining"
      @close="store.closeExpiryWarning"
    />
    
    <!-- 更新提示弹窗 -->
    <UpdateDialog
      :visible="store.showUpdateDialog"
      :update-info="store.updateInfo"
      :current-version="store.version"
      @close="store.closeUpdateDialog"
      @download="store.closeUpdateDialog"
    />
    
    <!-- 主界面（只有授权通过后才显示） -->
    <main v-if="store.isLicenseRegistered">
      <div class="p-4">
        <button 
          @click="showTest = !showTest"
          class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600"
        >
          {{ showTest ? '返回主界面' : '测试命令' }}
        </button>
      </div>
      
      <TestCommand v-if="showTest" />
      <SimilarityApp v-else />
    </main>
  </div>
</template>
