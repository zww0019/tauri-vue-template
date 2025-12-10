<script setup lang="ts">
import { useSimilarityStore, type BatchMode, type FileInfo } from '@/stores/similarityStore'
import { open } from '@tauri-apps/plugin-dialog'

const store = useSimilarityStore()
const isDragOver = ref(false)

// 选择多个文件
async function selectFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: '支持的文档',
          extensions: ['txt', 'docx', 'pdf'],
        },
      ],
    })

    if (selected && Array.isArray(selected)) {
      for (const filePath of selected) {
        await addFile(filePath)
      }
    }
  } catch (e) {
    console.error('选择文件失败:', e)
  }
}

// 添加文件
async function addFile(filePath: string) {
  const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'unknown'
  const ext = fileName.split('.').pop()?.toLowerCase() || ''
  
  // 检查是否已存在
  if (store.batchFiles.some(f => f.path === filePath)) {
    return
  }
  
  const fileInfo: FileInfo = {
    path: filePath,
    name: fileName,
    size: 0,
    type: ext,
    status: 'parsing',
  }
  
  store.addBatchFiles([fileInfo])
  const index = store.batchFiles.findIndex(f => f.path === filePath)

  try {
    const isValid = await store.validateFile(filePath)
    const file = index >= 0 ? store.batchFiles[index] : null
    if (!isValid) {
      if (file) {
        file.status = 'error'
        file.error = '不支持的文件格式'
      }
      return
    }

    const parsed = await store.parseDocument(filePath)
    if (file) {
      if (parsed) {
        file.status = 'ready'
        file.parsed = parsed
        file.size = parsed.metadata.file_size
      } else {
        file.status = 'error'
        file.error = '解析失败'
      }
    }
  } catch (e) {
    const file = index >= 0 ? store.batchFiles[index] : null
    if (file) {
      file.status = 'error'
      file.error = String(e)
    }
  }
}

// 移除文件
function removeFile(index: number) {
  store.removeBatchFile(index)
}

// 清空所有文件
function clearAllFiles() {
  store.clearBatchFiles()
}

// 开始批量对比
async function startBatchComparison() {
  await store.compareBatch()
}

// 获取就绪文件数量
const readyFileCount = computed(() => {
  return store.batchFiles.filter(f => f.status === 'ready').length
})

// 是否可以开始对比
const canStartComparison = computed(() => {
  return readyFileCount.value >= 2 && !store.isProcessing
})

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

// 获取文件类型图标
function getFileIcon(type: string): string {
  switch (type) {
    case 'docx': return '📝'
    case 'pdf': return '📕'
    case 'txt': return '📄'
    default: return '📁'
  }
}

// 批量模式选项
const batchModeOptions: { value: BatchMode; label: string; description: string }[] = [
  { value: 'all_pairs', label: '全两两对比', description: '所有文件相互比较' },
  { value: 'with_first', label: '与第一个对比', description: '其他文件都与第一个文件比较' },
  { value: 'with_baseline', label: '与基准文件对比', description: '指定一个基准文件进行对比' },
]

// 查看结果详情
function viewResultDetail(index: number) {
  store.selectBatchResult(index)
}

// 获取相似度颜色
function getSimilarityColor(similarity: number): string {
  return store.getSimilarityColor(similarity)
}
</script>

<template>
  <div class="space-y-6">
    <!-- 文件列表区域 -->
    <div class="bg-white rounded-2xl shadow-lg p-6">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-bold text-gray-900">文件列表</h2>
        <div class="flex items-center gap-2">
          <span class="text-sm text-gray-500">
            {{ readyFileCount }} / {{ store.batchFiles.length }} 文件就绪
          </span>
          <button
            v-if="store.batchFiles.length > 0"
            @click="clearAllFiles"
            class="text-sm text-red-500 hover:text-red-600"
          >
            清空全部
          </button>
        </div>
      </div>

      <!-- 文件选择区域 -->
      <div
        @dragover.prevent="isDragOver = true"
        @dragleave="isDragOver = false"
        @drop.prevent
        @click="selectFiles"
        :class="[
          'border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all',
          isDragOver ? 'border-blue-400 bg-blue-50' : 'border-gray-200 hover:border-gray-300'
        ]"
      >
        <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
          <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
        </div>
        <p class="text-gray-600 font-medium mb-1">点击添加文件</p>
        <p class="text-gray-400 text-sm">支持选择多个 DOCX、TXT、PDF 文件</p>
      </div>

      <!-- 已选文件列表 -->
      <div v-if="store.batchFiles.length > 0" class="mt-4 space-y-2">
        <div
          v-for="(file, index) in store.batchFiles"
          :key="file.path"
          :class="[
            'flex items-center justify-between p-3 rounded-lg border transition-colors',
            file.status === 'ready' ? 'border-green-200 bg-green-50' :
            file.status === 'error' ? 'border-red-200 bg-red-50' :
            file.status === 'parsing' ? 'border-blue-200 bg-blue-50' :
            'border-gray-200'
          ]"
        >
          <div class="flex items-center gap-3 min-w-0">
            <span class="text-2xl">{{ getFileIcon(file.type) }}</span>
            <div class="min-w-0">
              <p class="font-medium text-gray-900 truncate">{{ file.name }}</p>
              <p v-if="file.status === 'ready' && file.parsed" class="text-xs text-gray-500">
                {{ file.parsed.word_count }} 词 · {{ file.parsed.paragraphs.length }} 段落 · {{ formatSize(file.size) }}
              </p>
              <p v-else-if="file.status === 'parsing'" class="text-xs text-blue-500">
                正在解析...
              </p>
              <p v-else-if="file.status === 'error'" class="text-xs text-red-500">
                {{ file.error }}
              </p>
            </div>
          </div>
          
          <div class="flex items-center gap-2">
            <!-- 状态图标 -->
            <span v-if="file.status === 'ready'" class="text-green-500">
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
              </svg>
            </span>
            <span v-else-if="file.status === 'parsing'" class="text-blue-500">
              <div class="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
            </span>
            <span v-else-if="file.status === 'error'" class="text-red-500">
              <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
              </svg>
            </span>
            
            <!-- 基准标记 -->
            <span 
              v-if="store.batchMode === 'with_baseline' && index === store.baselineIndex"
              class="px-2 py-0.5 bg-yellow-100 text-yellow-700 text-xs rounded"
            >
              基准
            </span>
            
            <!-- 设为基准按钮 -->
            <button
              v-if="store.batchMode === 'with_baseline' && index !== store.baselineIndex && file.status === 'ready'"
              @click="store.setBaselineIndex(index)"
              class="text-xs text-gray-500 hover:text-blue-500"
            >
              设为基准
            </button>
            
            <!-- 删除按钮 -->
            <button
              @click="removeFile(index)"
              class="p-1 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 对比模式选择 -->
    <div class="bg-white rounded-2xl shadow-lg p-6">
      <h2 class="text-lg font-bold text-gray-900 mb-4">对比模式</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div
          v-for="option in batchModeOptions"
          :key="option.value"
          @click="store.setBatchMode(option.value)"
          :class="[
            'p-4 border-2 rounded-xl cursor-pointer transition-all',
            store.batchMode === option.value
              ? 'border-blue-500 bg-blue-50'
              : 'border-gray-200 hover:border-gray-300'
          ]"
        >
          <div class="flex items-center gap-2 mb-2">
            <div :class="[
              'w-4 h-4 rounded-full border-2',
              store.batchMode === option.value
                ? 'border-blue-500 bg-blue-500'
                : 'border-gray-300'
            ]">
              <div v-if="store.batchMode === option.value" class="w-full h-full flex items-center justify-center">
                <div class="w-2 h-2 bg-white rounded-full"></div>
              </div>
            </div>
            <span class="font-medium text-gray-900">{{ option.label }}</span>
          </div>
          <p class="text-sm text-gray-500 ml-6">{{ option.description }}</p>
        </div>
      </div>
    </div>

    <!-- 开始对比按钮 -->
    <div class="flex items-center justify-center gap-4">
      <button
        @click="startBatchComparison"
        :disabled="!canStartComparison"
        :class="[
          'px-8 py-3 rounded-xl font-medium text-white transition-all',
          canStartComparison
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
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
          开始批量对比
        </span>
      </button>
    </div>

    <!-- 进度显示 -->
    <div v-if="store.isProcessing && store.progress" class="bg-white rounded-2xl shadow-lg p-6">
      <h2 class="text-lg font-bold text-gray-900 mb-4">处理进度</h2>
      <div class="space-y-4">
        <div class="flex items-center justify-between text-sm">
          <span>{{ store.progress.completed_pairs }} / {{ store.progress.total_pairs }} 对</span>
          <span>{{ store.progress.percentage.toFixed(1) }}%</span>
        </div>
        <div class="w-full bg-gray-200 rounded-full h-3">
          <div
            class="bg-gradient-to-r from-blue-500 to-purple-600 h-3 rounded-full transition-all duration-300"
            :style="{ width: store.progress.percentage + '%' }"
          ></div>
        </div>
        <div class="flex items-center justify-between text-sm text-gray-500">
          <span>当前: {{ store.progress.current_source }} ↔ {{ store.progress.current_target }}</span>
          <span>预计剩余: {{ Math.ceil(store.progress.estimated_remaining_seconds) }}s</span>
        </div>
      </div>
    </div>

    <!-- 批量结果展示 -->
    <div v-if="store.batchResult" class="space-y-6">
      <!-- 汇总统计 -->
      <div class="bg-white rounded-2xl shadow-lg p-6">
        <div class="flex items-center justify-between mb-6">
          <h2 class="text-lg font-bold text-gray-900">批量对比结果</h2>
          <span class="text-sm text-gray-500">{{ store.batchResult.timestamp }}</span>
        </div>

        <div class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
          <div class="text-center p-4 bg-gray-50 rounded-xl">
            <div class="text-3xl font-bold text-gray-800">{{ store.batchResult.file_count }}</div>
            <p class="text-sm text-gray-500">文件数</p>
          </div>
          <div class="text-center p-4 bg-gray-50 rounded-xl">
            <div class="text-3xl font-bold text-gray-800">{{ store.batchResult.total_pairs }}</div>
            <p class="text-sm text-gray-500">对比对数</p>
          </div>
          <div class="text-center p-4 bg-blue-50 rounded-xl">
            <div class="text-3xl font-bold text-blue-600">{{ (store.batchResult.average_similarity * 100).toFixed(1) }}%</div>
            <p class="text-sm text-gray-500">平均相似度</p>
          </div>
          <div class="text-center p-4 bg-orange-50 rounded-xl">
            <div class="text-3xl font-bold text-orange-600">{{ (store.batchResult.max_similarity * 100).toFixed(1) }}%</div>
            <p class="text-sm text-gray-500">最高相似度</p>
          </div>
          <div class="text-center p-4 bg-red-50 rounded-xl">
            <div class="text-3xl font-bold text-red-600">{{ store.batchResult.high_similarity_pairs }}</div>
            <p class="text-sm text-gray-500">高相似对</p>
          </div>
          <div class="text-center p-4 bg-green-50 rounded-xl">
            <div class="text-3xl font-bold text-green-600">{{ store.batchResult.total_processing_time_ms }}ms</div>
            <p class="text-sm text-gray-500">处理耗时</p>
          </div>
        </div>
      </div>

      <!-- 热力图 -->
      <div v-if="store.batchResult.similarity_matrix.length <= 10" class="bg-white rounded-2xl shadow-lg p-6">
        <h3 class="text-lg font-bold text-gray-900 mb-4">相似度热力图</h3>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr>
                <th class="p-2"></th>
                <th 
                  v-for="(name, index) in store.batchResult.file_names" 
                  :key="index"
                  class="p-2 text-center font-medium text-gray-600 truncate max-w-[100px]"
                  :title="name"
                >
                  {{ name.slice(0, 8) }}{{ name.length > 8 ? '...' : '' }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, i) in store.batchResult.similarity_matrix" :key="i">
                <td class="p-2 font-medium text-gray-600 truncate max-w-[100px]" :title="store.batchResult.file_names[i]">
                  {{ store.batchResult.file_names[i]?.slice(0, 8) }}{{ (store.batchResult.file_names[i]?.length || 0) > 8 ? '...' : '' }}
                </td>
                <td 
                  v-for="(val, j) in row" 
                  :key="j"
                  class="p-2 text-center text-white font-medium rounded"
                  :style="{ 
                    backgroundColor: i === j ? '#e5e7eb' : getSimilarityColor(val),
                    color: i === j ? '#6b7280' : 'white'
                  }"
                >
                  {{ i === j ? '-' : (val * 100).toFixed(0) + '%' }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 结果列表 -->
      <div class="bg-white rounded-2xl shadow-lg p-6">
        <h3 class="text-lg font-bold text-gray-900 mb-4">详细结果列表</h3>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-gray-50">
              <tr>
                <th class="p-3 text-left font-medium text-gray-600">#</th>
                <th class="p-3 text-left font-medium text-gray-600">源文件</th>
                <th class="p-3 text-left font-medium text-gray-600">目标文件</th>
                <th class="p-3 text-center font-medium text-gray-600">综合相似度</th>
                <th class="p-3 text-center font-medium text-gray-600">文本</th>
                <th class="p-3 text-center font-medium text-gray-600">图像</th>
                <th class="p-3 text-center font-medium text-gray-600">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr 
                v-for="(result, index) in store.batchResult.results" 
                :key="result.id"
                class="border-t border-gray-100 hover:bg-gray-50"
              >
                <td class="p-3 text-gray-500">{{ index + 1 }}</td>
                <td class="p-3 truncate max-w-[150px]" :title="result.source_file_name">{{ result.source_file_name }}</td>
                <td class="p-3 truncate max-w-[150px]" :title="result.target_file_name">{{ result.target_file_name }}</td>
                <td class="p-3 text-center">
                  <span 
                    class="px-2 py-1 rounded text-white text-xs font-medium"
                    :style="{ backgroundColor: getSimilarityColor(result.overall_similarity) }"
                  >
                    {{ (result.overall_similarity * 100).toFixed(1) }}%
                  </span>
                </td>
                <td class="p-3 text-center text-gray-600">{{ (result.text_similarity * 100).toFixed(1) }}%</td>
                <td class="p-3 text-center text-gray-600">{{ (result.image_similarity * 100).toFixed(1) }}%</td>
                <td class="p-3 text-center">
                  <button
                    @click="viewResultDetail(index)"
                    class="text-blue-600 hover:text-blue-700 font-medium"
                  >
                    详情
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
