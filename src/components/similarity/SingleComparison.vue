<script setup lang="ts">
import { useSimilarityStore, type FileInfo } from '@/stores/similarityStore'

const store = useSimilarityStore()

// 设置源文件
function setSourceFile(file: FileInfo) {
  store.setSourceFile(file)
}

// 设置目标文件
function setTargetFile(file: FileInfo) {
  store.setTargetFile(file)
}

// 清除源文件
function clearSourceFile() {
  store.sourceFile = null
}

// 清除目标文件
function clearTargetFile() {
  store.targetFile = null
}

// 开始对比
async function startComparison() {
  await store.compareSingle()
}

// 查看详细结果
function viewDetails() {
  store.showDetailView = true
}

// 清除结果
function clearResult() {
  store.clearResults()
}

// 计算相似度颜色
function getSimilarityColor(similarity: number): string {
  return store.getSimilarityColor(similarity)
}

// 计算相似度标签
function getSimilarityLabel(similarity: number): string {
  return store.getSimilarityLabel(similarity)
}
</script>

<template>
  <div class="space-y-6">
    <!-- 文件选择区域 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <FileSelector
        label="源文件"
        :file="store.sourceFile"
        type="source"
        @select="setSourceFile"
        @clear="clearSourceFile"
      />
      <FileSelector
        label="目标文件"
        :file="store.targetFile"
        type="target"
        @select="setTargetFile"
        @clear="clearTargetFile"
      />
    </div>

    <!-- 操作按钮 -->
    <div class="flex items-center justify-center gap-4">
      <button
        @click="startComparison"
        :disabled="!store.hasFiles || store.isProcessing || store.sourceFile?.status !== 'ready' || store.targetFile?.status !== 'ready'"
        :class="[
          'px-8 py-3 rounded-xl font-medium text-white transition-all',
          store.hasFiles && !store.isProcessing && store.sourceFile?.status === 'ready' && store.targetFile?.status === 'ready'
            ? 'bg-gradient-to-r from-blue-500 to-purple-600 hover:from-blue-600 hover:to-purple-700 shadow-lg hover:shadow-xl'
            : 'bg-gray-300 cursor-not-allowed'
        ]"
      >
        <span v-if="store.isProcessing" class="flex items-center gap-2">
          <div class="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          正在分析...
        </span>
        <span v-else class="flex items-center gap-2">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
          </svg>
          开始对比
        </span>
      </button>
      
      <button
        v-if="store.singleResult"
        @click="clearResult"
        class="px-6 py-3 rounded-xl font-medium text-gray-600 bg-gray-100 hover:bg-gray-200 transition-colors"
      >
        清除结果
      </button>
    </div>

    <!-- 结果展示 -->
    <div v-if="store.singleResult" class="space-y-6">
      <!-- 综合评分卡片 -->
      <div class="bg-white rounded-2xl shadow-lg p-6">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-lg font-bold text-gray-900">对比结果</h2>
          <span class="text-sm text-gray-500">{{ store.singleResult.timestamp }}</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <!-- 综合相似度 -->
          <div class="text-center p-6 bg-gradient-to-br from-gray-50 to-gray-100 rounded-xl">
            <div 
              class="text-5xl font-bold mb-2"
              :style="{ color: getSimilarityColor(store.singleResult.overall_similarity) }"
            >
              {{ (store.singleResult.overall_similarity * 100).toFixed(1) }}%
            </div>
            <p class="text-gray-600 font-medium">综合相似度</p>
            <span 
              class="inline-block mt-2 px-3 py-1 rounded-full text-white text-sm"
              :style="{ backgroundColor: getSimilarityColor(store.singleResult.overall_similarity) }"
            >
              {{ store.singleResult.overall_level }}
            </span>
          </div>

          <!-- 文本相似度 -->
          <div class="text-center p-6 bg-blue-50 rounded-xl">
            <div class="text-4xl font-bold text-blue-600 mb-2">
              {{ (store.singleResult.text_similarity * 100).toFixed(1) }}%
            </div>
            <p class="text-gray-600 font-medium">文本相似度</p>
            <p class="text-sm text-gray-500 mt-1">{{ store.singleResult.text_level }}</p>
          </div>

          <!-- 图像相似度 -->
          <div class="text-center p-6 bg-purple-50 rounded-xl">
            <div class="text-4xl font-bold text-purple-600 mb-2">
              {{ (store.singleResult.image_similarity * 100).toFixed(1) }}%
            </div>
            <p class="text-gray-600 font-medium">图像相似度</p>
            <p class="text-sm text-gray-500 mt-1">{{ store.singleResult.image_level }}</p>
          </div>
        </div>

        <!-- 详细指标 -->
        <div class="mt-6 pt-6 border-t border-gray-100">
          <h3 class="text-sm font-medium text-gray-500 mb-4">详细指标</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-xl font-semibold text-gray-800">
                {{ (store.singleResult.text_result.cosine_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">余弦相似度</p>
            </div>
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-xl font-semibold text-gray-800">
                {{ (store.singleResult.text_result.ngram_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">N-gram相似度</p>
            </div>
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-xl font-semibold text-gray-800">
                {{ (store.singleResult.text_result.edit_distance_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">编辑距离</p>
            </div>
            <div class="text-center p-3 bg-gray-50 rounded-lg">
              <div class="text-xl font-semibold text-gray-800">
                {{ (store.singleResult.text_result.keyword_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">关键词相似度</p>
            </div>
          </div>
        </div>

        <!-- 统计信息 -->
        <div class="mt-6 pt-6 border-t border-gray-100">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <p class="text-gray-500">相似段落</p>
              <p class="font-semibold text-gray-800">{{ store.singleResult.text_result.matched_paragraphs.length }} 对</p>
            </div>
            <div>
              <p class="text-gray-500">共同关键词</p>
              <p class="font-semibold text-gray-800">{{ store.singleResult.text_result.common_keywords.length }} 个</p>
            </div>
            <div>
              <p class="text-gray-500">处理耗时</p>
              <p class="font-semibold text-gray-800">{{ store.singleResult.processing_time_ms }} ms</p>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="mt-6 flex items-center justify-end gap-4">
          <button
            @click="viewDetails"
            class="px-4 py-2 text-blue-600 hover:bg-blue-50 rounded-lg transition-colors font-medium"
          >
            查看详情
          </button>
        </div>
      </div>

      <!-- 快速预览：相似段落 -->
      <div v-if="store.singleResult.text_result.matched_paragraphs.length > 0" class="bg-white rounded-2xl shadow-lg p-6">
        <h3 class="text-lg font-bold text-gray-900 mb-4">相似段落预览</h3>
        <div class="space-y-4">
          <div 
            v-for="(para, index) in store.singleResult.text_result.matched_paragraphs.slice(0, 3)" 
            :key="index"
            class="p-4 border border-gray-100 rounded-lg"
            :class="{
              'border-l-4 border-l-red-500': para.similarity >= 0.8,
              'border-l-4 border-l-orange-500': para.similarity >= 0.6 && para.similarity < 0.8,
              'border-l-4 border-l-green-500': para.similarity < 0.6,
            }"
          >
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm font-medium text-gray-500">匹配 #{{ index + 1 }}</span>
              <span 
                class="px-2 py-1 rounded text-xs font-medium text-white"
                :style="{ backgroundColor: getSimilarityColor(para.similarity) }"
              >
                {{ (para.similarity * 100).toFixed(1) }}%
              </span>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div class="p-3 bg-blue-50 rounded-lg">
                <p class="text-xs text-blue-600 mb-1 font-medium">源文件</p>
                <p class="text-sm text-gray-700 line-clamp-3">{{ para.source_text }}</p>
              </div>
              <div class="p-3 bg-purple-50 rounded-lg">
                <p class="text-xs text-purple-600 mb-1 font-medium">目标文件</p>
                <p class="text-sm text-gray-700 line-clamp-3">{{ para.target_text }}</p>
              </div>
            </div>
          </div>
        </div>
        
        <div v-if="store.singleResult.text_result.matched_paragraphs.length > 3" class="mt-4 text-center">
          <button
            @click="viewDetails"
            class="text-blue-600 hover:text-blue-700 text-sm font-medium"
          >
            查看全部 {{ store.singleResult.text_result.matched_paragraphs.length }} 个相似段落 →
          </button>
        </div>
      </div>

      <!-- 共同关键词 -->
      <div v-if="store.singleResult.text_result.common_keywords.length > 0" class="bg-white rounded-2xl shadow-lg p-6">
        <h3 class="text-lg font-bold text-gray-900 mb-4">共同关键词</h3>
        <div class="flex flex-wrap gap-2">
          <span 
            v-for="keyword in store.singleResult.text_result.common_keywords" 
            :key="keyword"
            class="px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-sm font-medium"
          >
            {{ keyword }}
          </span>
        </div>
      </div>

    </div>
  </div>
</template>
