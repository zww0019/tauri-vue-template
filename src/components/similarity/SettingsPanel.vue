<script setup lang="ts">
import { useSimilarityStore } from '@/stores/similarityStore';

const emit = defineEmits<{
  close: []
}>()

const store = useSimilarityStore()

// 本地配置副本
const localConfig = ref({ ...store.config })

// 保存配置
function saveConfig() {
  store.updateConfig(localConfig.value)
  emit('close')
}

// 重置为默认
async function resetToDefault() {
  await store.loadDefaultConfig()
  localConfig.value = { ...store.config }
}

// 权重联动
watch(() => localConfig.value.text_weight, (newVal) => {
  localConfig.value.image_weight = Number((1 - newVal).toFixed(2))
})

watch(() => localConfig.value.image_weight, (newVal) => {
  localConfig.value.text_weight = Number((1 - newVal).toFixed(2))
})
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- 背景遮罩 -->
    <div class="absolute inset-0 bg-black/50" @click="emit('close')"></div>
    
    <!-- 设置面板 -->
    <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-lg mx-4 max-h-[90vh] overflow-y-auto">
      <!-- 头部 -->
      <div class="sticky top-0 bg-white border-b border-gray-100 px-6 py-4 flex items-center justify-between">
        <h2 class="text-xl font-bold text-gray-900">参数设置</h2>
        <button
          @click="emit('close')"
          class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 内容 -->
      <div class="p-6 space-y-6">
        <!-- 权重设置 -->
        <div>
          <h3 class="text-sm font-medium text-gray-900 mb-4">权重设置</h3>
          
          <div class="space-y-4">
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">文本权重</label>
                <span class="text-sm font-medium text-blue-600">{{ (localConfig.text_weight * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.text_weight"
                min="0"
                max="1"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-500"
              />
            </div>

            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">图像权重</label>
                <span class="text-sm font-medium text-purple-600">{{ (localConfig.image_weight * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.image_weight"
                min="0"
                max="1"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-purple-500"
              />
            </div>
          </div>
        </div>

        <!-- 阈值设置 -->
        <div>
          <h3 class="text-sm font-medium text-gray-900 mb-4">匹配阈值</h3>
          
          <div class="space-y-4">
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">段落匹配阈值</label>
                <span class="text-sm font-medium text-gray-800">{{ (localConfig.paragraph_threshold * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.paragraph_threshold"
                min="0.1"
                max="0.9"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              />
              <p class="text-xs text-gray-400 mt-1">相似度低于此值的段落不会被标记为匹配</p>
            </div>

            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">句子匹配阈值</label>
                <span class="text-sm font-medium text-gray-800">{{ (localConfig.sentence_threshold * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.sentence_threshold"
                min="0.1"
                max="0.9"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              />
            </div>

            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">图像匹配阈值</label>
                <span class="text-sm font-medium text-gray-800">{{ (localConfig.image_threshold * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.image_threshold"
                min="0.3"
                max="0.9"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              />
            </div>

            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-sm text-gray-600">显示阈值</label>
                <span class="text-sm font-medium text-gray-800">{{ (localConfig.display_threshold * 100).toFixed(0) }}%</span>
              </div>
              <input
                type="range"
                v-model.number="localConfig.display_threshold"
                min="0"
                max="0.5"
                step="0.1"
                class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              />
              <p class="text-xs text-gray-400 mt-1">相似度低于此值的结果不会显示</p>
            </div>
          </div>
        </div>

        <!-- 对比粒度 -->
        <div>
          <h3 class="text-sm font-medium text-gray-900 mb-4">对比粒度</h3>
          
          <div class="grid grid-cols-3 gap-3">
            <button
              @click="localConfig.granularity = 'overall'"
              :class="[
                'p-3 rounded-lg border-2 transition-all text-center',
                localConfig.granularity === 'overall'
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-200 hover:border-gray-300'
              ]"
            >
              <span class="block text-sm font-medium">整体</span>
              <span class="block text-xs text-gray-500">仅计算整体分数</span>
            </button>
            <button
              @click="localConfig.granularity = 'paragraph'"
              :class="[
                'p-3 rounded-lg border-2 transition-all text-center',
                localConfig.granularity === 'paragraph'
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-200 hover:border-gray-300'
              ]"
            >
              <span class="block text-sm font-medium">段落级</span>
              <span class="block text-xs text-gray-500">逐段落对比</span>
            </button>
            <button
              @click="localConfig.granularity = 'sentence'"
              :class="[
                'p-3 rounded-lg border-2 transition-all text-center',
                localConfig.granularity === 'sentence'
                  ? 'border-blue-500 bg-blue-50 text-blue-700'
                  : 'border-gray-200 hover:border-gray-300'
              ]"
            >
              <span class="block text-sm font-medium">句子级</span>
              <span class="block text-xs text-gray-500">逐句对比</span>
            </button>
          </div>
        </div>
      </div>

      <!-- 底部操作 -->
      <div class="sticky bottom-0 bg-white border-t border-gray-100 px-6 py-4 flex items-center justify-between">
        <button
          @click="resetToDefault"
          class="px-4 py-2 text-gray-600 hover:text-gray-800 hover:bg-gray-100 rounded-lg transition-colors"
        >
          重置默认
        </button>
        <div class="flex items-center gap-3">
          <button
            @click="emit('close')"
            class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            @click="saveConfig"
            class="px-6 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition-colors font-medium"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
