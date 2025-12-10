<script setup lang="ts">
import { useSimilarityStore } from '@/stores/similarityStore';
import { save } from '@tauri-apps/plugin-dialog';

const emit = defineEmits<{
  close: []
}>()

const store = useSimilarityStore()
const activeTab = ref<'text' | 'image' | 'report'>('text')

// 当前结果
const result = computed(() => store.selectedResult)

// 获取相似度颜色
function getSimilarityColor(similarity: number): string {
  return store.getSimilarityColor(similarity)
}

// 导出报告
async function exportReport(format: 'html' | 'json' | 'csv') {
  try {
    const extension = format
    const selected = await save({
      filters: [{ name: format.toUpperCase(), extensions: [extension] }],
      defaultPath: `相似度报告_${new Date().toISOString().slice(0, 10)}.${extension}`,
    })

    if (selected) {
      const outputPath = await store.generateReport(selected, {
        format,
        include_details: true,
        include_matched_paragraphs: true,
        include_matched_images: true,
        include_statistics: true,
      })

      if (outputPath) {
        alert(`报告已保存到: ${outputPath}`)
      }
    }
  } catch (e) {
    console.error('导出报告失败:', e)
  }
}

// 截断文本
function truncateText(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text
  return text.slice(0, maxLen) + '...'
}
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <!-- 背景遮罩 -->
    <div class="absolute inset-0 bg-black/50" @click="emit('close')"></div>
    
    <!-- 详情面板 -->
    <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-6xl mx-4 max-h-[90vh] overflow-hidden flex flex-col">
      <!-- 头部 -->
      <div class="bg-white border-b border-gray-100 px-6 py-4 flex items-center justify-between flex-shrink-0">
        <div>
          <h2 class="text-xl font-bold text-gray-900">详细对比结果</h2>
          <p v-if="result" class="text-sm text-gray-500">
            {{ result.source_file_name }} ↔ {{ result.target_file_name }}
          </p>
        </div>
        <button
          @click="emit('close')"
          class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 标签页 -->
      <div class="border-b border-gray-100 px-6 flex-shrink-0">
        <div class="flex gap-6">
          <button
            @click="activeTab = 'text'"
            :class="[
              'py-3 border-b-2 font-medium text-sm transition-colors',
              activeTab === 'text'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            ]"
          >
            文本对比
          </button>
          <button
            @click="activeTab = 'image'"
            :class="[
              'py-3 border-b-2 font-medium text-sm transition-colors',
              activeTab === 'image'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            ]"
          >
            图像对比
          </button>
          <button
            @click="activeTab = 'report'"
            :class="[
              'py-3 border-b-2 font-medium text-sm transition-colors',
              activeTab === 'report'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700'
            ]"
          >
            导出报告
          </button>
        </div>
      </div>

      <!-- 内容区 -->
      <div class="flex-1 overflow-y-auto p-6" v-if="result">
        <!-- 文本对比标签页 -->
        <div v-if="activeTab === 'text'" class="space-y-6">
          <!-- 相似度概览 -->
          <div class="grid grid-cols-4 gap-4">
            <div class="text-center p-4 bg-blue-50 rounded-xl">
              <div class="text-2xl font-bold text-blue-600">
                {{ (result.text_result.cosine_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">余弦相似度</p>
            </div>
            <div class="text-center p-4 bg-purple-50 rounded-xl">
              <div class="text-2xl font-bold text-purple-600">
                {{ (result.text_result.jaccard_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">Jaccard相似度</p>
            </div>
            <div class="text-center p-4 bg-green-50 rounded-xl">
              <div class="text-2xl font-bold text-green-600">
                {{ (result.text_result.edit_distance_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">编辑距离</p>
            </div>
            <div class="text-center p-4 bg-orange-50 rounded-xl">
              <div class="text-2xl font-bold text-orange-600">
                {{ (result.text_result.keyword_similarity * 100).toFixed(1) }}%
              </div>
              <p class="text-xs text-gray-500">关键词相似度</p>
            </div>
          </div>

          <!-- 共同关键词 -->
          <div v-if="result.text_result.common_keywords.length > 0">
            <h3 class="text-sm font-medium text-gray-700 mb-3">共同关键词 ({{ result.text_result.common_keywords.length }})</h3>
            <div class="flex flex-wrap gap-2">
              <span 
                v-for="keyword in result.text_result.common_keywords" 
                :key="keyword"
                class="px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-sm"
              >
                {{ keyword }}
              </span>
            </div>
          </div>

          <!-- 相似段落 -->
          <div>
            <h3 class="text-sm font-medium text-gray-700 mb-3">
              相似段落 ({{ result.text_result.matched_paragraphs.length }})
            </h3>
            <div class="space-y-4 max-h-[400px] overflow-y-auto">
              <div 
                v-for="(para, index) in result.text_result.matched_paragraphs" 
                :key="index"
                class="border border-gray-200 rounded-lg overflow-hidden"
              >
                <div class="flex items-center justify-between px-4 py-2 bg-gray-50 border-b border-gray-200">
                  <span class="text-sm font-medium text-gray-600">匹配 #{{ index + 1 }}</span>
                  <span 
                    class="px-2 py-1 rounded text-xs font-medium text-white"
                    :style="{ backgroundColor: getSimilarityColor(para.similarity) }"
                  >
                    {{ (para.similarity * 100).toFixed(1) }}%
                  </span>
                </div>
                <div class="grid grid-cols-2 divide-x divide-gray-200">
                  <div class="p-4">
                    <p class="text-xs text-blue-600 mb-2 font-medium">源文件 - 段落 {{ para.source_index + 1 }}</p>
                    <p class="text-sm text-gray-700 whitespace-pre-wrap">{{ para.source_text }}</p>
                  </div>
                  <div class="p-4">
                    <p class="text-xs text-purple-600 mb-2 font-medium">目标文件 - 段落 {{ para.target_index + 1 }}</p>
                    <p class="text-sm text-gray-700 whitespace-pre-wrap">{{ para.target_text }}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 相似句子 -->
          <div v-if="result.text_result.matched_sentences.length > 0">
            <h3 class="text-sm font-medium text-gray-700 mb-3">
              相似句子 ({{ result.text_result.matched_sentences.length }})
            </h3>
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="p-3 text-left font-medium text-gray-600">#</th>
                    <th class="p-3 text-left font-medium text-gray-600">源句子</th>
                    <th class="p-3 text-left font-medium text-gray-600">目标句子</th>
                    <th class="p-3 text-center font-medium text-gray-600">相似度</th>
                  </tr>
                </thead>
                <tbody>
                  <tr 
                    v-for="(sent, index) in result.text_result.matched_sentences.slice(0, 20)" 
                    :key="index"
                    class="border-t border-gray-100 hover:bg-gray-50"
                  >
                    <td class="p-3 text-gray-500">{{ index + 1 }}</td>
                    <td class="p-3 text-gray-700">{{ truncateText(sent.source_text, 100) }}</td>
                    <td class="p-3 text-gray-700">{{ truncateText(sent.target_text, 100) }}</td>
                    <td class="p-3 text-center">
                      <span 
                        class="px-2 py-1 rounded text-xs font-medium text-white"
                        :style="{ backgroundColor: getSimilarityColor(sent.similarity) }"
                      >
                        {{ (sent.similarity * 100).toFixed(1) }}%
                      </span>
                    </td>
                  </tr>
                </tbody>
              </table>
              <p v-if="result.text_result.matched_sentences.length > 20" class="text-center text-sm text-gray-500 mt-2">
                仅显示前 20 条，共 {{ result.text_result.matched_sentences.length }} 条
              </p>
            </div>
          </div>
        </div>

        <!-- 图像对比标签页 -->
        <div v-else-if="activeTab === 'image'" class="space-y-6">
          <!-- 图像统计 -->
          <div class="grid grid-cols-4 gap-4">
            <div class="text-center p-4 bg-blue-50 rounded-xl">
              <div class="text-2xl font-bold text-blue-600">{{ result.image_result.source_image_count }}</div>
              <p class="text-xs text-gray-500">源文件图片</p>
            </div>
            <div class="text-center p-4 bg-purple-50 rounded-xl">
              <div class="text-2xl font-bold text-purple-600">{{ result.image_result.target_image_count }}</div>
              <p class="text-xs text-gray-500">目标文件图片</p>
            </div>
            <div class="text-center p-4 bg-green-50 rounded-xl">
              <div class="text-2xl font-bold text-green-600">{{ result.image_result.matched_count }}</div>
              <p class="text-xs text-gray-500">匹配图片对</p>
            </div>
            <div class="text-center p-4 bg-orange-50 rounded-xl">
              <div class="text-2xl font-bold text-orange-600">{{ result.image_result.identical_count }}</div>
              <p class="text-xs text-gray-500">完全相同</p>
            </div>
          </div>

          <!-- 匹配图片列表 -->
          <div v-if="result.image_result.matched_images.length > 0">
            <h3 class="text-sm font-medium text-gray-700 mb-3">匹配的图片</h3>
            <div class="space-y-4">
              <div 
                v-for="(img, index) in result.image_result.matched_images" 
                :key="index"
                class="border border-gray-200 rounded-lg p-4"
              >
                <div class="flex items-center justify-between mb-3">
                  <span class="text-sm font-medium text-gray-600">图片匹配 #{{ index + 1 }}</span>
                  <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">{{ img.match_type }}</span>
                    <span 
                      class="px-2 py-1 rounded text-xs font-medium text-white"
                      :style="{ backgroundColor: getSimilarityColor(img.similarity) }"
                    >
                      {{ (img.similarity * 100).toFixed(1) }}%
                    </span>
                  </div>
                </div>
                <div class="grid grid-cols-2 gap-4 text-sm text-gray-600">
                  <div class="p-3 bg-gray-50 rounded">
                    <p><strong>源图片 #{{ img.source_index + 1 }}</strong></p>
                    <p v-if="img.source_width && img.source_height">
                      尺寸: {{ img.source_width }} × {{ img.source_height }}
                    </p>
                  </div>
                  <div class="p-3 bg-gray-50 rounded">
                    <p><strong>目标图片 #{{ img.target_index + 1 }}</strong></p>
                    <p v-if="img.target_width && img.target_height">
                      尺寸: {{ img.target_width }} × {{ img.target_height }}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 无图片提示 -->
          <div v-else class="text-center py-12 text-gray-500">
            <div class="text-4xl mb-4">🖼️</div>
            <p>文档中没有发现匹配的图片</p>
          </div>
        </div>

        <!-- 导出报告标签页 -->
        <div v-else-if="activeTab === 'report'" class="space-y-6">
          <p class="text-gray-600 mb-6">选择导出格式来生成对比报告：</p>
          
          <div class="grid grid-cols-3 gap-4">
            <button
              @click="exportReport('html')"
              class="p-6 border-2 border-gray-200 rounded-xl hover:border-blue-300 hover:bg-blue-50 transition-all text-center group"
            >
              <div class="text-4xl mb-3">🌐</div>
              <p class="font-medium text-gray-900 group-hover:text-blue-600">HTML 报告</p>
              <p class="text-sm text-gray-500 mt-1">交互式网页报告</p>
            </button>
            
            <button
              @click="exportReport('json')"
              class="p-6 border-2 border-gray-200 rounded-xl hover:border-green-300 hover:bg-green-50 transition-all text-center group"
            >
              <div class="text-4xl mb-3">📊</div>
              <p class="font-medium text-gray-900 group-hover:text-green-600">JSON 数据</p>
              <p class="text-sm text-gray-500 mt-1">结构化数据导出</p>
            </button>
            
            <button
              @click="exportReport('csv')"
              class="p-6 border-2 border-gray-200 rounded-xl hover:border-purple-300 hover:bg-purple-50 transition-all text-center group"
            >
              <div class="text-4xl mb-3">📋</div>
              <p class="font-medium text-gray-900 group-hover:text-purple-600">CSV 表格</p>
              <p class="text-sm text-gray-500 mt-1">Excel 兼容格式</p>
            </button>
          </div>

          <!-- 报告预览信息 -->
          <div class="mt-8 p-4 bg-gray-50 rounded-xl">
            <h4 class="font-medium text-gray-700 mb-3">报告将包含：</h4>
            <ul class="space-y-2 text-sm text-gray-600">
              <li class="flex items-center gap-2">
                <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                文件基本信息和元数据
              </li>
              <li class="flex items-center gap-2">
                <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                综合相似度评分和各项指标
              </li>
              <li class="flex items-center gap-2">
                <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                相似段落对照表
              </li>
              <li class="flex items-center gap-2">
                <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                图像匹配信息
              </li>
              <li class="flex items-center gap-2">
                <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                </svg>
                共同关键词列表
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
