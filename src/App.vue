<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import GreetComponent from './components/GreetComponent.vue'
import LicenseDialog from './components/LicenseDialog.vue'
import UpdateDialog from './components/UpdateDialog.vue'
import ExpiryWarning from './components/ExpiryWarning.vue'

const store = useStore()

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
    <main v-if="store.isLicenseRegistered" class="flex-1 flex flex-col items-center justify-center min-h-screen">
      <h1>Welcome to Tauri 2 + Vue</h1>

      <div class="flex flex-row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://vuejs.org/" target="_blank">
          <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>

      <GreetComponent />
    </main>
  </div>
</template>
