<template>
  <div class="foldersync-tool min-h-screen bg-gray-50 p-6">
    <div class="max-w-7xl mx-auto">
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- 主要内容区域 -->
        <div class="lg:col-span-2 space-y-6">
          <!-- 添加新同步任务 -->
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">添加同步任务RUST</h2>
            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">源文件夹</label>
                <div class="flex gap-2">
                  <input
                    v-model="newTask.sourceDir"
                    type="text"
                    readonly
                    class="flex-1 px-3 py-2 border border-gray-300 rounded-lg bg-gray-50"
                    placeholder="请选择源文件夹"
                  />
                  <button
                    @click="selectSourceDir"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    选择文件夹
                  </button>
                </div>
              </div>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-2">目标文件夹</label>
                <div class="flex gap-2">
                  <input
                    v-model="newTask.targetDir"
                    type="text"
                    readonly
                    class="flex-1 px-3 py-2 border border-gray-300 rounded-lg bg-gray-50"
                    placeholder="请选择目标文件夹"
                  />
                  <button
                    @click="selectTargetDir"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                  >
                    选择文件夹
                  </button>
                </div>
              </div>

              <button
                @click="addSyncTask"
                :disabled="!newTask.sourceDir || !newTask.targetDir"
                class="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
              >
                添加同步任务
              </button>
            </div>
          </div>

          <!-- 同步任务列表 -->
          <div v-if="syncTasks.length > 0" class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">同步任务列表 ({{ syncTasks.length }})</h2>
            <div class="space-y-4">
              <div
                v-for="task in syncTasks"
                :key="task.id"
                class="border border-gray-200 rounded-lg p-4 bg-gray-50"
              >
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center gap-3">
                    <span
                      :class="{
                        'bg-yellow-100 text-yellow-800': task.status === 'initializing',
                        'bg-green-100 text-green-800': task.status === 'syncing',
                        'bg-gray-100 text-gray-800': task.status === 'stopped',
                        'bg-red-100 text-red-800': task.status === 'error'
                      }"
                      class="px-2 py-1 rounded text-xs font-medium"
                    >
                      {{ getStatusText(task.status) }}
                    </span>
                    <span class="text-sm text-gray-600">
                      <strong>源:</strong> {{ task.sourceDir }} → 
                      <strong>目标:</strong> {{ task.targetDir }}
                    </span>
                  </div>
                  <button
                    @click="stopTask(task.id)"
                    :disabled="task.status === 'stopped'"
                    class="px-3 py-1 bg-red-600 text-white rounded hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors text-sm"
                  >
                    停止
                  </button>
                </div>

                <!-- 任务进度 -->
                <div v-if="task.progress" class="mt-3">
                  <div v-if="task.progress.message" class="text-sm text-blue-600 mb-2">
                    {{ task.progress.message }}
                  </div>
                  <div v-if="task.progress.progress !== undefined" class="w-full bg-gray-200 rounded-full h-2 mb-2">
                    <div
                      class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                      :style="{ width: task.progress.progress + '%' }"
                    ></div>
                  </div>
                  <div v-if="task.progress.processedFiles !== undefined" class="text-xs text-gray-600">
                    <span>文件: {{ task.progress.processedFiles }} / {{ task.progress.totalFiles }}</span>
                    <span v-if="task.progress.processedSize !== undefined && task.progress.totalSize !== undefined" class="ml-4">
                      已处理: {{ formatSize(task.progress.processedSize) }} / {{ formatSize(task.progress.totalSize) }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 侧边栏：使用说明 -->
        <div class="lg:col-span-1">
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6 sticky top-6">
            <h2 class="text-lg font-semibold text-gray-900 mb-4">使用说明</h2>
            <div class="space-y-4 text-sm text-gray-600">
              <div>
                <h3 class="font-medium text-gray-900 mb-2">功能特点：</h3>
                <ul class="list-disc list-inside space-y-1">
                  <li>单向同步：将源文件夹内容同步到目标文件夹</li>
                  <li>自动对齐：初始化阶段自动对齐两个文件夹</li>
                  <li>增量同步：后续实时监控并同步文件变化</li>
                  <li>多任务支持：可同时监控多个文件夹</li>
                  <li>实时日志：每个任务有独立的日志窗口</li>
                </ul>
              </div>
              <div>
                <h3 class="font-medium text-gray-900 mb-2">使用步骤：</h3>
                <ol class="list-decimal list-inside space-y-1">
                  <li>选择源文件夹和目标文件夹</li>
                  <li>点击"添加同步任务"开始同步</li>
                  <li>初始化阶段会自动对齐文件夹内容</li>
                  <li>初始化完成后，自动开始实时监控</li>
                  <li>查看下方日志窗口了解同步情况</li>
                </ol>
              </div>
              <div>
                <h3 class="font-medium text-gray-900 mb-2">注意事项：</h3>
                <ul class="list-disc list-inside space-y-1">
                  <li>目标文件夹中的文件可能会被覆盖</li>
                  <li>删除源文件夹中的文件，目标文件夹中对应文件也会被删除</li>
                  <li>请确保有足够的磁盘空间</li>
                  <li>同步过程中请勿关闭应用</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 日志窗口 -->
      <div v-if="syncTasks.length > 0" class="mt-6 bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-900">同步日志</h2>
          <button
            @click="clearAllLogs"
            class="text-sm text-blue-600 hover:text-blue-700"
          >
            清空所有日志
          </button>
        </div>
        <div class="border border-gray-200 rounded-lg overflow-hidden">
          <div class="flex border-b border-gray-200">
            <button
              v-for="task in syncTasks"
              :key="task.id"
              @click="activeLogTab = task.id"
              :class="{
                'bg-blue-600 text-white': activeLogTab === task.id,
                'bg-gray-50 text-gray-700 hover:bg-gray-100': activeLogTab !== task.id
              }"
              class="px-4 py-2 text-sm font-medium transition-colors"
            >
              {{ getTaskLabel(task) }}
            </button>
          </div>
          <div
            v-for="task in syncTasks"
            :key="task.id"
            v-show="activeLogTab === task.id"
            class="h-96 overflow-y-auto bg-gray-900 text-gray-100 p-4 font-mono text-sm"
          >
            <div class="space-y-1">
              <div
                v-for="(log, index) in task.logs"
                :key="index"
                :class="{
                  'text-gray-400': log.type === 'info',
                  'text-green-400': log.type === 'success',
                  'text-red-400': log.type === 'error',
                  'text-blue-400': log.type === 'sync'
                }"
              >
                <span class="text-gray-500">[{{ formatTime(log.time) }}]</span>
                <span class="ml-2">{{ log.message }}</span>
              </div>
              <div v-if="task.logs.length === 0" class="text-gray-500 text-center py-8">
                暂无日志
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { onMounted, onUnmounted, ref } from 'vue'

interface SyncTask {
  id: string
  sourceDir: string
  targetDir: string
  status: 'initializing' | 'syncing' | 'stopped' | 'error'
  logs: Array<{
    time: string
    type: 'info' | 'success' | 'error' | 'sync'
    message: string
  }>
  progress: {
    status?: string
    message?: string
    progress?: number
    processedFiles?: number
    totalFiles?: number
    processedSize?: number
    totalSize?: number
  } | null
}

const newTask = ref({
  sourceDir: '',
  targetDir: ''
})

const syncTasks = ref<SyncTask[]>([])
const activeLogTab = ref<string>('')
let unlistenProgress: (() => void) | null = null

async function selectSourceDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择源文件夹'
    })
    if (selected && typeof selected === 'string') {
      newTask.value.sourceDir = selected
    }
  } catch (error) {
    console.error('选择文件夹失败:', error)
  }
}

async function selectTargetDir() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择目标文件夹'
    })
    if (selected && typeof selected === 'string') {
      newTask.value.targetDir = selected
    }
  } catch (error) {
    console.error('选择文件夹失败:', error)
  }
}

async function addSyncTask() {
  if (!newTask.value.sourceDir || !newTask.value.targetDir) {
    alert('请选择源文件夹和目标文件夹')
    return
  }

  try {
    console.log('开始添加同步任务...', {
      sourceDir: newTask.value.sourceDir,
      targetDir: newTask.value.targetDir
    })

    const result = await invoke<{success: boolean, message: string, data?: any}>('execute_tool', {
      toolId: 'foldersync',
      method: 'addSyncTask',
      args: {
        sourceDir: newTask.value.sourceDir,
        targetDir: newTask.value.targetDir
      }
    })

    console.log('执行结果:', result)

    if (result && result.success && result.data) {
      const data = result.data
      const taskId = data.taskId || data.task_id
      
      if (taskId) {
        console.log('[前端] 创建任务对象，taskId:', taskId)
        const task: SyncTask = {
          id: taskId,
          sourceDir: newTask.value.sourceDir,
          targetDir: newTask.value.targetDir,
          status: 'initializing',
          logs: [],
          progress: null
        }
        syncTasks.value.push(task)
        console.log('[前端] 任务已添加到列表，当前任务数量:', syncTasks.value.length)
        console.log('[前端] 任务列表:', syncTasks.value.map(t => ({ id: t.id, status: t.status })))
        activeLogTab.value = taskId
        newTask.value.sourceDir = ''
        newTask.value.targetDir = ''
        console.log('[前端] 任务添加成功:', taskId)
      } else {
        console.error('返回数据中没有 taskId:', data)
        alert('任务添加失败：返回数据格式错误')
      }
    } else {
      console.error('执行失败:', result)
      alert(result?.message || '添加任务失败')
    }
  } catch (error) {
    console.error('添加同步任务失败:', error)
    alert(error instanceof Error ? error.message : '添加任务失败')
  }
}

async function stopTask(taskId: string) {
  try {
    const result = await invoke<{success: boolean, message?: string}>('execute_tool', {
      toolId: 'foldersync',
      method: 'stopSyncTask',
      args: { taskId }
    })

    if (result && result.success) {
      const task = syncTasks.value.find(t => t.id === taskId)
      if (task) {
        task.status = 'stopped'
      }
    }
  } catch (error) {
    console.error('停止任务失败:', error)
    alert(error instanceof Error ? error.message : '停止任务失败')
  }
}

function handleExecutionProgress(event: any) {
  console.log('[前端] 收到执行进度事件:', event)
  const progress = event.payload
  console.log('[前端] 进度数据:', progress)
  
  if (progress.toolId === 'foldersync' || progress.toolName === 'foldersync') {
    const taskId = progress.taskId
    console.log('[前端] 任务ID:', taskId)
    
    if (!taskId) {
      console.warn('[前端] 缺少 taskId，忽略事件')
      return
    }

    const task = syncTasks.value.find(t => t.id === taskId)
    if (!task) {
      console.warn('[前端] 未找到任务:', taskId, '当前任务列表:', syncTasks.value.map(t => t.id))
      return
    }

    console.log('[前端] 找到任务:', task.id)

    if (progress.status) {
      task.status = progress.status
    }

    if (progress.progress !== undefined || progress.message) {
      task.progress = {
        status: progress.status || task.status,
        message: progress.message || '',
        progress: progress.progress,
        processedFiles: progress.processedFiles,
        totalFiles: progress.totalFiles,
        processedSize: progress.processedSize,
        totalSize: progress.totalSize
      }
    }

    // 处理实时日志事件
    if (progress.log) {
      console.log('[前端] 收到日志事件:', progress.log)
      const logEntry = {
        time: progress.log.time || new Date().toISOString(),
        type: progress.log.type || progress.log.level || 'info',
        message: progress.log.message || ''
      }
      console.log('[前端] 添加日志条目:', logEntry)
      task.logs.push(logEntry)
      if (task.logs.length > 1000) {
        task.logs = task.logs.slice(-1000)
      }
      console.log('[前端] 任务日志数量:', task.logs.length)
      scrollLogToBottom(taskId)
    } else {
      console.log('[前端] 进度事件中没有日志字段')
    }
  } else {
    console.log('[前端] 工具ID不匹配:', progress.toolId, '期望: foldersync')
  }
}

// 已移除轮询逻辑，改为基于事件的实时日志推送
// 日志通过 tool:execution-progress 事件实时接收

function scrollLogToBottom(taskId: string) {
  // 自动滚动到底部的逻辑可以在需要时添加
}

function clearTaskLogs(taskId: string) {
  const task = syncTasks.value.find(t => t.id === taskId)
  if (task) {
    task.logs = []
  }
}

function clearAllLogs() {
  syncTasks.value.forEach(task => {
    task.logs = []
  })
}

function getStatusText(status: string): string {
  const statusMap: Record<string, string> = {
    'initializing': '初始化中',
    'syncing': '同步中',
    'stopped': '已停止',
    'error': '错误'
  }
  return statusMap[status] || status
}

function getTaskLabel(task: SyncTask): string {
  const sourceName = task.sourceDir.split(/[/\\]/).pop() || task.sourceDir
  return `${sourceName} (${getStatusText(task.status)})`
}

function formatTime(timeStr: string): string {
  if (!timeStr) return ''
  const date = new Date(timeStr)
  return date.toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

function formatSize(bytes: number | undefined): string {
  if (!bytes || bytes < 0) return '0 B'
  if (bytes < 1024) {
    return `${bytes} B`
  } else if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(2)} KB`
  } else if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / 1024 / 1024).toFixed(2)} MB`
  } else {
    return `${(bytes / 1024 / 1024 / 1024).toFixed(2)} GB`
  }
}

onMounted(async () => {
  console.log('[前端] 开始监听 tool:execution-progress 事件')
  // 监听工具执行进度（包含实时日志事件）
  unlistenProgress = await listen('tool:execution-progress', handleExecutionProgress)
  console.log('[前端] 事件监听已设置')
  
  // 不再需要轮询，日志通过事件实时推送
})

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
  
  // 停止所有正在运行的任务
  syncTasks.value.forEach(async (task) => {
    if (task.status === 'initializing' || task.status === 'syncing') {
      try {
        await invoke('execute_tool', {
          toolId: 'foldersync',
          method: 'stopSyncTask',
          args: { taskId: task.id }
        })
      } catch (error) {
        // 忽略错误
      }
    }
  })
})
</script>

<style scoped>
.foldersync-tool {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>

