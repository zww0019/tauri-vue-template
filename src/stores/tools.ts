import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { ref, computed } from 'vue'

export interface ToolInfo {
  id: string
  name: string
  version: string
  description: string
  author: string
  icon: string
  required_auth: boolean
  has_dependencies: boolean
  hidden: boolean
}

export interface DependencyCheckResult {
  installed: boolean
  missing: string[]
  details: Record<string, boolean>
}

export const useToolsStore = defineStore('tools', () => {
  // 状态
  const tools = ref<ToolInfo[]>([])
  const selectedTool = ref<ToolInfo | null>(null)
  const loading = ref(false)
  const dependencyStatus = ref<Record<string, DependencyCheckResult>>({})
  
  // Getters
  const toolsCount = computed(() => tools.value.length)
  
  const authenticatedCount = computed(() => {
    // 这个需要结合auth store来计算
    return 0 // 临时返回0
  })
  
  const getToolById = computed(() => (id: string) => {
    return tools.value.find(t => t.id === id)
  })
  
  // Actions
  async function loadTools() {
    try {
      loading.value = true
      const toolsList = await invoke<ToolInfo[]>('get_tools_list')
      tools.value = toolsList
      return toolsList
    } catch (error) {
      console.error('加载工具列表失败:', error)
      tools.value = []
      return []
    } finally {
      loading.value = false
    }
  }
  
  async function getToolInfo(toolId: string) {
    try {
      const info = await invoke<ToolInfo | null>('get_tool_info', { toolId })
      return info
    } catch (error) {
      console.error('获取工具信息失败:', error)
      return null
    }
  }
  
  async function selectTool(tool: ToolInfo) {
    selectedTool.value = tool
  }
  
  async function checkToolDependencies(toolId: string) {
    try {
      const result = await invoke<DependencyCheckResult>('check_tool_dependencies', { toolId })
      dependencyStatus.value[toolId] = result
      return result
    } catch (error) {
      console.error('检查依赖失败:', error)
      return {
        installed: false,
        missing: [],
        details: {}
      }
    }
  }
  
  async function installToolDependencies(toolId: string) {
    try {
      const result = await invoke<{success: boolean, message: string}>('install_tool_dependencies', { toolId })
      
      if (result.success) {
        // 重新检查依赖状态
        await checkToolDependencies(toolId)
      }
      
      return result
    } catch (error) {
      console.error('安装依赖失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '安装依赖失败'
      }
    }
  }
  
  async function reloadTools() {
    try {
      await invoke('reload_tools')
      await loadTools()
      return { success: true }
    } catch (error) {
      console.error('重新加载工具失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '重新加载失败'
      }
    }
  }
  
  return {
    // 状态
    tools,
    selectedTool,
    loading,
    dependencyStatus,
    // Getters
    toolsCount,
    authenticatedCount,
    getToolById,
    // Actions
    loadTools,
    getToolInfo,
    selectTool,
    checkToolDependencies,
    installToolDependencies,
    reloadTools,
  }
})

