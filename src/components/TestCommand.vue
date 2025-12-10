<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'

const testPath = ref('/Users/wwz/temp/IBM Granite 4.0 Micro.docx')
const result = ref<any>(null)
const error = ref<string | null>(null)
const isLoading = ref(false)

async function testParseDocument() {
  isLoading.value = true
  error.value = null
  result.value = null
  
  console.log('=== 开始测试 ===')
  console.log('文件路径:', testPath.value)
  
  try {
    console.log('调用 parse_document...')
    const data = await invoke('parse_document', { filePath: testPath.value })
    console.log('调用成功，返回数据:', data)
    result.value = data
  } catch (e) {
    console.error('调用失败:', e)
    error.value = String(e)
  } finally {
    isLoading.value = false
    console.log('=== 测试结束 ===')
  }
}
</script>

<template>
  <div class="p-8">
    <h2 class="text-2xl font-bold mb-4">命令测试</h2>
    
    <div class="mb-4">
      <label class="block mb-2">文件路径:</label>
      <input 
        v-model="testPath" 
        class="w-full px-3 py-2 border rounded"
        type="text"
      />
    </div>
    
    <button 
      @click="testParseDocument"
      :disabled="isLoading"
      class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-400"
    >
      {{ isLoading ? '测试中...' : '测试 parse_document' }}
    </button>
    
    <div v-if="error" class="mt-4 p-4 bg-red-100 text-red-700 rounded">
      <h3 class="font-bold">错误:</h3>
      <pre>{{ error }}</pre>
    </div>
    
    <div v-if="result" class="mt-4 p-4 bg-green-100 rounded">
      <h3 class="font-bold mb-2">成功!</h3>
      <pre class="text-sm overflow-auto">{{ JSON.stringify(result, null, 2) }}</pre>
    </div>
  </div>
</template>
