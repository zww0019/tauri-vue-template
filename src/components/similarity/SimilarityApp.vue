<script setup lang="ts">
import { useSimilarityStore } from '@/stores/similarityStore';

const store = useSimilarityStore()

// 初始化加载默认配置
onMounted(async () => {
  await store.loadDefaultConfig()
})
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- 顶部导航 -->
    <header class="bg-white border-b border-gray-200 px-6 py-4">
      <div class="max-w-7xl mx-auto flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 bg-gradient-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
            <span class="text-white text-xl">📄</span>
          </div>
          <div>
            <h1 class="text-xl font-bold text-gray-900">文档相似度检测</h1>
            <p class="text-sm text-gray-500">支持 DOCX、TXT、PDF 格式</p>
          </div>
        </div>
        
        <!-- 模式切换 -->
        <div class="flex items-center gap-4">
          <div class="flex bg-gray-100 rounded-lg p-1">
            <button
              @click="store.setMode('single')"
              :class="[
                'px-4 py-2 rounded-md text-sm font-medium transition-all',
                store.mode === 'single' 
                  ? 'bg-white text-blue-600 shadow-sm' 
                  : 'text-gray-600 hover:text-gray-900'
              ]"
            >
              单文件对比
            </button>
            <button
              @click="store.setMode('batch')"
              :class="[
                'px-4 py-2 rounded-md text-sm font-medium transition-all',
                store.mode === 'batch' 
                  ? 'bg-white text-blue-600 shadow-sm' 
                  : 'text-gray-600 hover:text-gray-900'
              ]"
            >
              批量对比
            </button>
          </div>
          
          <button
            @click="store.showSettings = true"
            class="p-2 text-gray-500 hover:text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
            title="设置"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <!-- 主内容区 -->
    <main class="max-w-7xl mx-auto px-6 py-6">
      <!-- 错误提示 -->
      <div v-if="store.error" class="mb-6 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg flex items-center justify-between">
        <span>{{ store.error }}</span>
        <button @click="store.error = null" class="text-red-500 hover:text-red-700">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 单文件对比模式 -->
      <template v-if="store.mode === 'single'">
        <SingleComparison />
      </template>

      <!-- 批量对比模式 -->
      <template v-else>
        <BatchComparison />
      </template>
    </main>

    <!-- 设置面板 -->
    <SettingsPanel 
      v-if="store.showSettings" 
      @close="store.showSettings = false" 
    />

    <!-- 详细对比视图 -->
    <DetailView 
      v-if="store.showDetailView && store.selectedResult" 
      @close="store.showDetailView = false" 
    />
  </div>
</template>
