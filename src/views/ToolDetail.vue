<template>
  <div class="tool-detail min-h-screen bg-gray-50">
    <header class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <button
              @click="router.back()"
              class="text-gray-600 hover:text-gray-900 transition-colors"
            >
              ← 返回
            </button>
            <div class="h-6 w-px bg-gray-300"></div>
            <div class="text-3xl">{{ toolInfo?.icon || '📦' }}</div>
            <div>
              <h1 class="text-xl font-bold text-gray-900">{{ toolInfo?.name }}</h1>
              <p class="text-sm text-gray-500">v{{ toolInfo?.version }}</p>
            </div>
          </div>
        </div>
      </div>
    </header>

    <main class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div v-if="!toolInfo" class="text-center py-12">
        <div class="text-6xl mb-4">❌</div>
        <p class="text-xl text-gray-700">工具不存在</p>
      </div>

      <div v-else>
        <!-- 如果已认证且依赖已安装，直接显示工具界面 -->
        <div v-if="canUseTool" class="w-full">
          <component
            v-if="toolViewComponent"
            :is="toolViewComponent"
          />
          <div v-else-if="loadingToolView" class="text-center py-12">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
            <p class="mt-2 text-sm text-gray-600">加载工具界面中...</p>
          </div>
          <div v-else-if="toolViewError" class="text-center py-12">
            <div class="text-6xl mb-4">⚠️</div>
            <p class="text-xl text-gray-700 mb-2">工具界面加载失败</p>
            <p class="text-sm text-gray-500">{{ toolViewError }}</p>
          </div>
          <div v-else class="text-center py-12">
            <div class="text-6xl mb-4">📦</div>
            <p class="text-xl text-gray-700">工具界面未实现</p>
            <p class="text-sm text-gray-500 mt-2">该工具尚未实现用户界面</p>
          </div>
        </div>

        <!-- 如果需要认证或依赖未安装，显示设置页面 -->
        <div v-else class="space-y-6">
          <!-- 认证状态 -->
          <div v-if="toolInfo.required_auth && !isAuthenticated" class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">认证状态</h2>
            
            <div class="space-y-4">
              <div class="flex items-start space-x-3">
                <div class="flex-shrink-0">
                  <div class="w-10 h-10 bg-yellow-100 rounded-full flex items-center justify-center">
                    <span class="text-yellow-600 text-xl">🔒</span>
                  </div>
                </div>
                <div class="flex-1">
                  <p class="text-sm font-medium text-gray-900">需要认证</p>
                  <p class="text-sm text-gray-600 mt-1">此工具需要有效的许可证才能使用</p>
                </div>
              </div>

              <div class="space-y-3">
                <label class="block text-sm font-medium text-gray-700">
                  输入许可证代码
                </label>
                <textarea
                  v-model="licenseCode"
                  rows="4"
                  class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
                  placeholder="粘贴您的许可证代码..."
                ></textarea>
                <button
                  @click="registerLicense"
                  :disabled="!licenseCode || registering"
                  class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                >
                  {{ registering ? '注册中...' : '注册许可证' }}
                </button>
                <p v-if="registerError" class="text-sm text-red-600">{{ registerError }}</p>
                <p v-if="registerSuccess" class="text-sm text-green-600">{{ registerSuccess }}</p>
              </div>
            </div>
          </div>

          <!-- 依赖状态（始终显示，但安装按钮需要认证后才可用） -->
          <div v-if="toolInfo.has_dependencies" class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">依赖状态</h2>
            
            <div v-if="checkingDependencies" class="text-center py-4">
              <div class="inline-block animate-spin rounded-full h-6 w-6 border-4 border-blue-500 border-t-transparent"></div>
              <p class="mt-2 text-sm text-gray-600">检查依赖中...</p>
            </div>

            <div v-else-if="dependencyStatus" class="space-y-4">
              <div v-if="dependencyStatus.installed" class="flex items-start space-x-3">
                <div class="flex-shrink-0">
                  <div class="w-10 h-10 bg-green-100 rounded-full flex items-center justify-center">
                    <span class="text-green-600 text-xl">✓</span>
                  </div>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-900">依赖已安装</p>
                  <p class="text-sm text-gray-600 mt-1">所有必需的依赖包已安装完成</p>
                </div>
              </div>

              <div v-else class="space-y-3">
                <div class="flex items-start space-x-3">
                  <div class="flex-shrink-0">
                    <div class="w-10 h-10 bg-yellow-100 rounded-full flex items-center justify-center">
                      <span class="text-yellow-600 text-xl">⚠</span>
                    </div>
                  </div>
                  <div>
                    <p class="text-sm font-medium text-gray-900">缺少依赖</p>
                    <p class="text-sm text-gray-600 mt-1">需要安装以下依赖包:</p>
                    <ul class="mt-2 text-sm text-gray-600 list-disc list-inside">
                      <li v-for="dep in dependencyStatus.missing" :key="dep">{{ dep }}</li>
                    </ul>
                  </div>
                </div>

                <!-- 只有在已认证或不需要认证的情况下才显示安装按钮 -->
                <button
                  v-if="isAuthenticated || !toolInfo.required_auth"
                  @click="installDependencies"
                  :disabled="installingDependencies"
                  class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                >
                  {{ installingDependencies ? '安装中...' : '安装依赖' }}
                </button>
                <div v-else class="px-4 py-2 bg-gray-100 rounded-lg text-sm text-gray-600 text-center">
                  请先完成工具认证后再安装依赖
                </div>

                <div v-if="installingDependencies && installProgress" class="mt-4">
                  <div class="w-full bg-gray-200 rounded-full h-2">
                    <div
                      class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                      :style="{ width: installProgress.progress + '%' }"
                    ></div>
                  </div>
                  <p class="mt-2 text-sm text-gray-600 text-center">{{ installProgress.message }}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- 解绑确认对话框 -->
    <div
      v-if="showUnbindDialog"
      class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      @click.self="showUnbindDialog = false"
    >
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full mx-4 p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-2">确认解绑</h3>
        <p class="text-gray-600 mb-6">
          确定要解绑此工具的授权吗？解绑后将无法使用此工具，需要重新注册许可证。
        </p>
        <div class="flex justify-end space-x-3">
          <button
            @click="showUnbindDialog = false"
            class="px-4 py-2 text-gray-700 hover:text-gray-900 transition-colors"
          >
            取消
          </button>
          <button
            @click="unbindLicense"
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            确认解绑
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, shallowRef } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore, useToolsStore } from '../stores'
import { listen } from '@tauri-apps/api/event'
import type { ToolInfo, DependencyCheckResult } from '../stores/tools'
import type { Component } from 'vue'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const toolsStore = useToolsStore()

const toolId = computed(() => route.params.id as string)
const toolInfo = ref<ToolInfo | null>(null)
const authStatus = computed(() => authStore.toolAuthStatus[toolId.value])
const isAuthenticated = computed(() => authStatus.value?.is_authenticated || false)

// 判断是否可以正常使用工具（已认证且依赖已安装）
const canUseTool = computed(() => {
  if (!toolInfo.value) return false
  
  // 如果工具不需要认证，只需要检查依赖
  if (!toolInfo.value.required_auth) {
    // 如果没有依赖，直接可以使用
    if (!toolInfo.value.has_dependencies) return true
    // 如果有依赖，需要检查是否已安装
    return dependencyStatus.value?.installed ?? false
  }
  
  // 如果工具需要认证，需要同时满足认证和依赖条件
  if (!isAuthenticated.value) return false
  
  // 如果有依赖，需要检查是否已安装
  if (toolInfo.value.has_dependencies) {
    return dependencyStatus.value?.installed ?? false
  }
  
  return true
})

const licenseCode = ref('')
const registering = ref(false)
const registerError = ref('')
const registerSuccess = ref('')

const dependencyStatus = ref<DependencyCheckResult | null>(null)
const checkingDependencies = ref(false)
const installingDependencies = ref(false)
const installProgress = ref<{ message: string; progress: number } | null>(null)

const showUnbindDialog = ref(false)

// 工具视图相关
const toolViewComponent = shallowRef<Component | null>(null)
const loadingToolView = ref(false)
const toolViewError = ref<string | null>(null)

// 使用 import.meta.glob 预加载所有工具视图
// 注意：在 Vite 中，import.meta.glob 的路径是相对于项目根目录的
// 使用相对路径或 alias 都可以，但需要确保路径正确
const toolViewsModules = import.meta.glob('../../tools/*/views/ToolView.vue', { eager: false })

// 创建工具视图映射
const toolViewMap = new Map<string, () => Promise<any>>()
Object.keys(toolViewsModules).forEach((path) => {
  // 从路径中提取工具目录名
  // 例如: ../../tools/foldersync/views/ToolView.vue -> foldersync
  const match = path.match(/tools\/([^/]+)\/views\/ToolView\.vue$/)
  if (match) {
    const toolDirName = match[1]
    toolViewMap.set(toolDirName, toolViewsModules[path] as () => Promise<any>)
  }
})

console.log('已加载的工具视图:', Array.from(toolViewMap.keys()))

async function loadToolInfo() {
  const info = await toolsStore.getToolInfo(toolId.value)
  toolInfo.value = info
  
  if (info) {
    // 检查认证状态
    if (info.required_auth) {
      await authStore.checkToolAuthStatus(info.id)
    }
    
    // 检查依赖状态
    if (info.has_dependencies) {
      await checkDependencies()
    }
    
    // 加载工具视图
    await loadToolView()
  }
}

/**
 * 动态加载工具视图
 */
async function loadToolView() {
  if (!toolInfo.value) {
    toolViewComponent.value = null
    return
  }

  // 如果工具需要认证但未认证，不加载视图
  if (toolInfo.value.required_auth && !isAuthenticated.value) {
    toolViewComponent.value = null
    return
  }

  loadingToolView.value = true
  toolViewError.value = null
  toolViewComponent.value = null

  try {
    // 尝试使用工具ID查找视图
    let viewLoader = toolViewMap.get(toolInfo.value.id)
    
    // 如果找不到，尝试其他可能的名称
    if (!viewLoader) {
      const possibleNames = [
        toolInfo.value.id.toLowerCase(),
        toolInfo.value.id.replace(/-/g, ''),
        toolInfo.value.id.replace(/-/g, '_'),
      ]
      
      for (const name of possibleNames) {
        if (toolViewMap.has(name)) {
          viewLoader = toolViewMap.get(name)
          break
        }
      }
    }

    if (viewLoader) {
      const module = await viewLoader()
      // Vue 组件通常在 default 导出中
      toolViewComponent.value = module.default || module
      console.log(`工具视图加载成功: ${toolInfo.value.id}`)
    } else {
      console.warn(`工具视图未找到: ${toolInfo.value.id}`)
      console.log('可用的工具视图:', Array.from(toolViewMap.keys()))
      toolViewComponent.value = null
    }
  } catch (error) {
    console.error('加载工具视图失败:', error)
    toolViewError.value = error instanceof Error ? error.message : '未知错误'
    toolViewComponent.value = null
  } finally {
    loadingToolView.value = false
  }
}

// 监听认证状态变化，重新加载视图
watch(isAuthenticated, async (newVal) => {
  if (newVal && toolInfo.value?.required_auth) {
    await loadToolView()
  }
})

async function registerLicense() {
  if (!toolInfo.value || !licenseCode.value) return
  
  registering.value = true
  registerError.value = ''
  registerSuccess.value = ''
  
  try {
    const result = await authStore.registerToolLicense(toolInfo.value.id, licenseCode.value)
    
    if (result.success) {
      registerSuccess.value = '注册成功！'
      licenseCode.value = ''
      // 重新检查认证状态
      await authStore.checkToolAuthStatus(toolInfo.value.id)
    } else {
      registerError.value = result.message || '注册失败'
    }
  } catch (error) {
    registerError.value = error instanceof Error ? error.message : '注册失败'
  } finally {
    registering.value = false
  }
}

async function unbindLicense() {
  if (!toolInfo.value) return
  
  await authStore.resetToolAuth(toolInfo.value.id)
  showUnbindDialog.value = false
  
  // 重新检查认证状态
  await authStore.checkToolAuthStatus(toolInfo.value.id)
}

async function checkDependencies() {
  if (!toolInfo.value) return
  
  checkingDependencies.value = true
  try {
    const result = await toolsStore.checkToolDependencies(toolInfo.value.id)
    dependencyStatus.value = result
  } finally {
    checkingDependencies.value = false
  }
}

async function installDependencies() {
  if (!toolInfo.value) return
  
  installingDependencies.value = true
  installProgress.value = { message: '准备安装...', progress: 0 }
  
  try {
    await toolsStore.installToolDependencies(toolInfo.value.id)
    
    // 重新检查依赖状态
    await checkDependencies()
    
    installProgress.value = { message: '安装完成！', progress: 100 }
    
    setTimeout(() => {
      installingDependencies.value = false
      installProgress.value = null
    }, 2000)
  } catch (error) {
    installingDependencies.value = false
    installProgress.value = null
    alert('安装依赖失败: ' + (error instanceof Error ? error.message : '未知错误'))
  }
}

onMounted(async () => {
  await loadToolInfo()
  
  // 监听依赖安装进度
  const unlisten = await listen('tool:dependency-progress', (event: any) => {
    const data = event.payload
    if (data.toolId === toolId.value) {
      installProgress.value = {
        message: data.message || '安装中...',
        progress: data.progress || 0
      }
    }
  })
  
  // 组件卸载时取消监听
  return () => {
    unlisten()
  }
})
</script>

<style scoped>
.tool-detail {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>

