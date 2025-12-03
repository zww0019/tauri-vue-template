import { acceptHMRUpdate, defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

const versionString =
  import.meta.env.MODE === 'development' ? `${import.meta.env.VITE_APP_VERSION}-dev` : import.meta.env.VITE_APP_VERSION

export interface LicenseInfo {
  license_id: string
  tool_id: string
  licensee_id: string
  issued_date: string
  expiry_date: string
  max_devices: number
  features: string[]
  version: string
}

export const useStore = defineStore('main', {
  state: () => ({
    debug: import.meta.env.MODE === 'development',
    version: versionString,
    isInitialized: false,
    name: '',
    // 授权相关
    isLicenseRegistered: false,
    licenseInfo: null as LicenseInfo | null,
    showLicenseDialog: false,
    showExpiryWarning: false,
    expiryDaysRemaining: 0,
    // 更新相关
    showUpdateDialog: false,
    updateInfo: {
      hasUpdate: false,
      latestVersion: '',
      releaseNotes: '',
      downloadUrl: '',
    },
  }),

  actions: {
    async initApp() {
      // 检查授权状态
      await this.checkLicenseStatus()
      
      // 如果已注册，检查有效期
      if (this.isLicenseRegistered) {
        await this.checkLicenseExpiry()
      }
      
      // 检查软件更新
      await this.checkForUpdate()
      
      this.isInitialized = true
      console.log('app initialized!')
    },

    async checkLicenseStatus() {
      try {
        const registered = await invoke<boolean>('check_license_registered')
        this.isLicenseRegistered = registered
        
        if (registered) {
          try {
            const licenseInfo = await invoke<LicenseInfo>('get_license_info')
            this.licenseInfo = licenseInfo
          } catch (error) {
            console.error('获取授权信息失败:', error)
            this.isLicenseRegistered = false
          }
        } else {
          this.showLicenseDialog = true
        }
      } catch (error) {
        console.error('检查授权状态失败:', error)
        this.showLicenseDialog = true
      }
    },

    async checkLicenseExpiry() {
      try {
        const daysRemaining = await invoke<number>('check_license_expiry')
        this.expiryDaysRemaining = daysRemaining
        
        // 如果剩余天数不足10天，显示警告
        if (daysRemaining > 0 && daysRemaining < 10) {
          this.showExpiryWarning = true
        }
      } catch (error) {
        console.error('检查授权有效期失败:', error)
      }
    },

    async checkForUpdate() {
      try {
        const updateInfo = await invoke<{
          hasUpdate: boolean
          latestVersion: string
          releaseNotes: string
          downloadUrl: string
        }>('check_update', { currentVersion: this.version })
        
        this.updateInfo = updateInfo
        if (updateInfo.hasUpdate) {
          this.showUpdateDialog = true
        }
      } catch (error) {
        console.error('检查更新失败:', error)
      }
    },

    onLicenseVerified() {
      this.isLicenseRegistered = true
      this.showLicenseDialog = false
      this.checkLicenseStatus()
    },

    onLicenseDialogCancel() {
      // 关闭授权弹窗（如果用户有有效授权）
      this.showLicenseDialog = false
    },

    closeExpiryWarning() {
      this.showExpiryWarning = false
    },

    closeUpdateDialog() {
      this.showUpdateDialog = false
    },

    async resetLicense() {
      try {
        await invoke('reset_license')
        this.isLicenseRegistered = false
        this.licenseInfo = null
        this.showLicenseDialog = true
      } catch (error) {
        console.error('重置授权码失败:', error)
      }
    },
  },

  getters: {
    isReady: (state) => {
      return state.isInitialized && state.isLicenseRegistered
    },

    storeGreet: (state) => {
      if (state.name.length > 0) {
        return `Greetings from Pinia store, ${state.name}!`
      }
      return ''
    },
  },
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot))
}
