<template>
  <div
    class="tool-card bg-white rounded-lg shadow-sm hover:shadow-md transition-shadow duration-200 cursor-pointer border border-gray-200 overflow-hidden"
    @click="$emit('click')"
  >
    <div class="p-6">
      <div class="flex items-start justify-between mb-4">
        <div class="flex items-center space-x-3">
          <div class="text-3xl">{{ tool.icon || '📦' }}</div>
          <div>
            <h3 class="text-lg font-semibold text-gray-900">{{ tool.name }}</h3>
            <p class="text-sm text-gray-500">v{{ tool.version }}</p>
          </div>
        </div>
        <div v-if="tool.required_auth" class="flex-shrink-0">
          <span
            v-if="isAuthenticated"
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800"
          >
            ✓ 已认证
          </span>
          <span
            v-else
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800"
          >
            🔒 需认证
          </span>
        </div>
      </div>

      <p class="text-gray-600 text-sm mb-4 line-clamp-2">
        {{ tool.description || '暂无描述' }}
      </p>

      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-2 text-xs text-gray-500">
          <span>👤 {{ tool.author || 'Unknown' }}</span>
        </div>
        <div v-if="tool.has_dependencies" class="text-xs text-blue-600">
          📦 有依赖
        </div>
      </div>
    </div>

    <div class="px-6 py-3 bg-gray-50 border-t border-gray-200">
      <button
        class="w-full text-sm font-medium text-blue-600 hover:text-blue-700 transition-colors"
      >
        打开工具 →
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ToolInfo } from '../stores/tools'

interface Props {
  tool: ToolInfo
  isAuthenticated: boolean
}

defineProps<Props>()
defineEmits<{
  click: []
}>()
</script>

<style scoped>
.tool-card:hover {
  transform: translateY(-2px);
}

.tool-card:active {
  transform: translateY(0);
}

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>

