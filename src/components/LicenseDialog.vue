<template>
  <div
    v-if="visible"
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    @click.self="handleCancel"
  >
    <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
      <h2 class="text-2xl font-bold mb-4">软件授权</h2>
      <p class="text-gray-600 mb-4">请输入您的授权码以继续使用本软件</p>
      
      <div class="mb-4">
        <label class="block text-sm font-medium text-gray-700 mb-2">授权码</label>
        <textarea
          v-model="licenseCode"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          rows="4"
          placeholder="请输入授权码"
        ></textarea>
      </div>
      
      <div v-if="errorMessage" class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded">
        {{ errorMessage }}
      </div>
      
      <div class="flex justify-end gap-3">
        <button
          @click="handleCancel"
          class="px-4 py-2 text-gray-700 bg-gray-200 rounded-md hover:bg-gray-300"
        >
          取消
        </button>
        <button
          @click="handleVerify"
          :disabled="!licenseCode.trim() || loading"
          class="px-4 py-2 text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          {{ loading ? '验证中...' : '验证' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'verified'): void
  (e: 'cancel'): void
}>()

const licenseCode = ref('')
const errorMessage = ref('')
const loading = ref(false)

const handleVerify = async () => {
  if (!licenseCode.value.trim()) {
    errorMessage.value = '请输入授权码'
    return
  }

  loading.value = true
  errorMessage.value = ''

  try {
    await invoke<{
      license_id: string
      tool_id: string
      licensee_id: string
      issued_date: string
      expiry_date: string
      max_devices: number
      features: string[]
      version: string
    }>('verify_license', { licenseCode: licenseCode.value.trim() })
    
    emit('verified')
    licenseCode.value = ''
    errorMessage.value = ''
  } catch (error: any) {
    errorMessage.value = error || '授权码验证失败，请检查授权码是否正确'
  } finally {
    loading.value = false
  }
}

const handleCancel = () => {
  licenseCode.value = ''
  errorMessage.value = ''
  emit('cancel')
}
</script>

