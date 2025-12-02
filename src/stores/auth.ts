import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

export interface LicenseData {
  toolId: string
  username: string
  expiryDate: string
  maxDevices: number
}

export interface AuthStatus {
  is_authenticated: boolean
  license_data: LicenseData | null
}

export const useAuthStore = defineStore('auth', () => {
  // 状态
  const toolAuthStatus = ref<Record<string, AuthStatus>>({})
  const registeredTools = ref<string[]>([])
  
  // Getters
  const isToolAuthenticated = computed(() => (toolId: string) => {
    return toolAuthStatus.value[toolId]?.is_authenticated || false
  })
  
  // Actions
  async function checkToolAuthStatus(toolId: string) {
    try {
      const status = await invoke<AuthStatus>('check_tool_auth_status', { toolId })
      toolAuthStatus.value[toolId] = status
      return status
    } catch (error) {
      console.error('检查认证状态失败:', error)
      toolAuthStatus.value[toolId] = {
        is_authenticated: false,
        license_data: null
      }
      return toolAuthStatus.value[toolId]
    }
  }
  
  async function registerToolLicense(toolId: string, licenseCode: string) {
    try {
      const result = await invoke<{success: boolean, message: string}>('register_tool_license', {
        toolId,
        licenseCode
      })
      
      if (result.success) {
        // 重新检查认证状态
        await checkToolAuthStatus(toolId)
        await loadRegisteredTools()
      }
      
      return result
    } catch (error) {
      console.error('注册License失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '注册失败'
      }
    }
  }
  
  async function loadRegisteredTools() {
    try {
      const tools = await invoke<string[]>('get_registered_tools')
      registeredTools.value = tools
    } catch (error) {
      console.error('加载已注册工具失败:', error)
      registeredTools.value = []
    }
  }
  
  async function resetToolAuth(toolId?: string) {
    try {
      await invoke('reset_tool_auth', { toolId })
      if (toolId) {
        delete toolAuthStatus.value[toolId]
      } else {
        toolAuthStatus.value = {}
      }
      await loadRegisteredTools()
      return { success: true }
    } catch (error) {
      console.error('重置认证失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '重置失败'
      }
    }
  }
  
  async function fetchPublicKey() {
    try {
      const result = await invoke<{success: boolean, message: string, data?: string}>('fetch_public_key')
      return result
    } catch (error) {
      console.error('获取公钥失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '获取公钥失败'
      }
    }
  }
  
  return {
    // 状态
    toolAuthStatus,
    registeredTools,
    // Getters
    isToolAuthenticated,
    // Actions
    checkToolAuthStatus,
    registerToolLicense,
    loadRegisteredTools,
    resetToolAuth,
    fetchPublicKey,
  }
})

