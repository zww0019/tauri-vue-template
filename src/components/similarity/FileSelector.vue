<script setup lang="ts">
import { useSimilarityStore, type FileInfo } from '@/stores/similarityStore';
import { open } from '@tauri-apps/plugin-dialog';

const props = defineProps<{
  label: string
  file: FileInfo | null
  type: 'source' | 'target'
}>()

const emit = defineEmits<{
  select: [file: FileInfo]
  clear: []
}>()

const store = useSimilarityStore()
const isLoading = ref(false)
const isDragOver = ref(false)

// 打开文件选择对话框
async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: '支持的文档',
          extensions: ['txt', 'docx', 'pdf'],
        },
      ],
    })

    if (selected && typeof selected === 'string') {
      await loadFile(selected)
    }
  } catch (e) {
    console.error('选择文件失败:', e)
  }
}

// 加载文件
async function loadFile(filePath: string) {
  isLoading.value = true
  
  const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || 'unknown'
  const ext = fileName.split('.').pop()?.toLowerCase() || ''
  
  const fileInfo: FileInfo = {
    path: filePath,
    name: fileName,
    size: 0,
    type: ext,
    status: 'parsing',
  }
  
  emit('select', fileInfo)

  try {
    // 验证格式
    console.log('[FileSelector] 开始验证文件格式:', filePath)
    const isValid = await store.validateFile(filePath)
    console.log('[FileSelector] 验证结果:', isValid)
    if (!isValid) {
      fileInfo.status = 'error'
      fileInfo.error = '不支持的文件格式'
      emit('select', fileInfo)
      return
    }

    // 解析文档
    console.log('[FileSelector] 开始解析文档')
    const parsed = await store.parseDocument(filePath)
    console.log('[FileSelector] 解析结果:', parsed)
    if (parsed) {
      fileInfo.status = 'ready'
      fileInfo.parsed = parsed
      fileInfo.size = parsed.metadata.file_size
      console.log('[FileSelector] 文件信息已更新为 ready')
    } else {
      fileInfo.status = 'error'
      fileInfo.error = '解析失败'
      console.log('[FileSelector] 解析返回 null')
    }
    
    emit('select', fileInfo)
    console.log('[FileSelector] 已发送 select 事件')
  } catch (e) {
    console.error('[FileSelector] 捕获到错误:', e)
    fileInfo.status = 'error'
    fileInfo.error = String(e)
    emit('select', fileInfo)
  } finally {
    isLoading.value = false
    console.log('[FileSelector] isLoading 设置为 false')
  }
}

// 处理拖放
function handleDrop(e: DragEvent) {
  isDragOver.value = false
  const files = e.dataTransfer?.files
  if (files && files.length > 0) {
    // Tauri 中需要通过其他方式获取文件路径
    // 这里简化处理，提示用户使用选择按钮
    const firstFile = files[0]
    if (firstFile) {
      console.log('拖放文件:', firstFile.name)
    }
  }
}

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
</script>

<template>
  <div
    @dragover.prevent="isDragOver = true"
    @dragleave="isDragOver = false"
    @drop.prevent="handleDrop"
    :class="[
      'relative rounded-xl border-2 border-dashed transition-all duration-200',
      isDragOver 
        ? 'border-blue-400 bg-blue-50' 
        : file?.status === 'ready'
          ? 'border-green-300 bg-green-50'
          : file?.status === 'error'
            ? 'border-red-300 bg-red-50'
            : 'border-gray-200 bg-white hover:border-gray-300'
    ]"
  >
    <!-- 标签 -->
    <div class="absolute -top-3 left-4 px-2 bg-white">
      <span class="text-sm font-medium text-gray-600">{{ label }}</span>
    </div>

    <!-- 未选择状态 -->
    <div 
      v-if="!file" 
      class="p-8 text-center cursor-pointer"
      @click="selectFile"
    >
      <div class="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
        <svg class="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
        </svg>
      </div>
      <p class="text-gray-600 font-medium mb-1">点击选择文件</p>
      <p class="text-gray-400 text-sm">或拖拽文件到此处</p>
      <p class="text-gray-400 text-xs mt-2">支持 DOCX、TXT、PDF</p>
    </div>

    <!-- 加载中状态 -->
    <div v-else-if="file.status === 'parsing'" class="p-8 text-center">
      <div class="w-16 h-16 mx-auto mb-4 flex items-center justify-center">
        <div class="w-10 h-10 border-4 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
      </div>
      <p class="text-gray-600 font-medium">正在解析文档...</p>
      <p class="text-gray-400 text-sm">{{ file.name }}</p>
    </div>

    <!-- 已选择状态 -->
    <div v-else class="p-6">
      <div class="flex items-start gap-4">
        <!-- 文件图标 -->
        <div class="text-4xl">{{ getFileIcon(file.type) }}</div>
        
        <!-- 文件信息 -->
        <div class="flex-1 min-w-0">
          <p class="font-medium text-gray-900 truncate" :title="file.name">{{ file.name }}</p>
          
          <template v-if="file.status === 'ready' && file.parsed">
            <div class="mt-2 grid grid-cols-2 gap-2 text-sm">
              <div class="flex items-center gap-1 text-gray-500">
                <span>📊</span>
                <span>{{ file.parsed.word_count.toLocaleString() }} 词</span>
              </div>
              <div class="flex items-center gap-1 text-gray-500">
                <span>📝</span>
                <span>{{ file.parsed.char_count.toLocaleString() }} 字符</span>
              </div>
              <div class="flex items-center gap-1 text-gray-500">
                <span>📑</span>
                <span>{{ file.parsed.paragraphs.length }} 段落</span>
              </div>
              <div class="flex items-center gap-1 text-gray-500">
                <span>🖼️</span>
                <span>{{ file.parsed.images.length }} 图片</span>
              </div>
            </div>
            <p class="mt-2 text-xs text-gray-400">{{ formatSize(file.size) }}</p>
          </template>
          
          <template v-else-if="file.status === 'error'">
            <p class="mt-2 text-sm text-red-500">{{ file.error }}</p>
          </template>
        </div>

        <!-- 操作按钮 -->
        <div class="flex flex-col gap-2">
          <button
            @click="selectFile"
            class="p-2 text-gray-400 hover:text-blue-500 hover:bg-blue-50 rounded-lg transition-colors"
            title="重新选择"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            @click="emit('clear')"
            class="p-2 text-gray-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-colors"
            title="清除"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- 状态指示 -->
      <div 
        v-if="file.status === 'ready'"
        class="mt-4 flex items-center gap-2 text-green-600 text-sm"
      >
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
        </svg>
        <span>文档已就绪</span>
      </div>
    </div>
  </div>
</template>
