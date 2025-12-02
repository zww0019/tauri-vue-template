<template>
  <div class="home min-h-screen bg-gray-50">
    <header class="bg-white shadow-sm border-b border-gray-200">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-4">
            <div class="text-3xl">⚡</div>
            <h1 class="text-2xl font-bold text-gray-900">ZToolSet</h1>
          </div>
          <div v-if="toolsStore.tools.length > 0" class="flex items-center space-x-6 text-sm">
            <div class="flex items-center space-x-2">
              <span class="text-2xl font-bold text-blue-600">{{ toolsStore.tools.length }}</span>
              <span class="text-gray-600">工具</span>
            </div>
            <div class="h-6 w-px bg-gray-300"></div>
            <div class="flex items-center space-x-2">
              <span class="text-2xl font-bold text-green-600">{{ authStore.registeredTools.length }}</span>
              <span class="text-gray-600">已认证</span>
            </div>
          </div>
        </div>
      </div>
    </header>

    <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <section class="mb-8">
        <div class="mb-6">
          <div class="flex items-center space-x-3 mb-2">
            <span class="text-2xl">🛠️</span>
            <h2 class="text-xl font-semibold text-gray-900">可用工具</h2>
          </div>
          <p class="text-gray-600">选择一个工具开始使用</p>
        </div>

        <div class="mb-6">
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <span class="text-gray-400">🔍</span>
            </div>
            <input
              v-model="searchQuery"
              type="text"
              class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="搜索工具名称、描述或作者..."
            />
          </div>
          <div v-if="searchQuery" class="mt-2 text-sm text-gray-600">
            找到 {{ filteredTools.length }} 个工具
          </div>
        </div>

        <div v-if="toolsStore.loading" class="text-center py-12">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
          <p class="mt-2 text-gray-600">加载中...</p>
        </div>

        <div v-else-if="filteredTools.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <ToolCard
            v-for="tool in filteredTools"
            :key="tool.id"
            :tool="tool"
            :is-authenticated="authStore.isToolAuthenticated(tool.id)"
            @click="navigateToTool(tool)"
          />
        </div>

        <div v-else-if="searchQuery" class="text-center py-12">
          <div class="text-6xl mb-4">🔍</div>
          <p class="text-xl text-gray-700 mb-2">未找到匹配的工具</p>
          <p class="text-gray-500">尝试使用其他关键词搜索</p>
        </div>

        <div v-else class="text-center py-12">
          <div class="text-6xl mb-4">📦</div>
          <p class="text-xl text-gray-700">暂无可用工具</p>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore, useToolsStore } from '../stores'
import ToolCard from '../components/ToolCard.vue'
import type { ToolInfo } from '../stores/tools'

const router = useRouter()
const authStore = useAuthStore()
const toolsStore = useToolsStore()

const searchQuery = ref('')

const filteredTools = computed(() => {
  if (!searchQuery.value) {
    return toolsStore.tools
  }
  
  const query = searchQuery.value.toLowerCase()
  return toolsStore.tools.filter(tool => 
    tool.name.toLowerCase().includes(query) ||
    tool.description.toLowerCase().includes(query) ||
    tool.author.toLowerCase().includes(query) ||
    tool.id.toLowerCase().includes(query)
  )
})

function navigateToTool(tool: ToolInfo) {
  router.push({ name: 'ToolDetail', params: { id: tool.id } })
}

onMounted(async () => {
  // 页面加载时检查所有工具的认证状态
  for (const tool of toolsStore.tools) {
    if (tool.required_auth) {
      await authStore.checkToolAuthStatus(tool.id)
    }
  }
})
</script>

<style scoped>
.home {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>

