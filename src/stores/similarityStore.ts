// 文档相似度检测 Store
import { invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'

// ==================== 类型定义 ====================

/** 相似度等级 */
export type SimilarityLevel = 'VeryLow' | 'Low' | 'Medium' | 'High' | 'VeryHigh'

/** 图像相似度等级 */
export type ImageSimilarityLevel = 'Identical' | 'HighlySimilar' | 'PartiallySimilar' | 'SimilarFeatures' | 'Different'

/** 文档元数据 */
export interface DocumentMetadata {
  file_name: string
  file_path: string
  file_size: number
  file_type: string
  page_count?: number
  created_time?: string
  modified_time?: string
}

/** 段落信息 */
export interface Paragraph {
  index: number
  text: string
  page?: number
  char_count: number
}

/** 句子信息 */
export interface Sentence {
  index: number
  text: string
  paragraph_index: number
}

/** 文档图片 */
export interface DocumentImage {
  index: number
  data?: number[]
  format: string
  width?: number
  height?: number
  page?: number
  hash: string
}

/** 解析后的文档 */
export interface ParsedDocument {
  metadata: DocumentMetadata
  full_text: string
  paragraphs: Paragraph[]
  sentences: Sentence[]
  images: DocumentImage[]
  word_count: number
  char_count: number
}

/** 匹配的段落对 */
export interface MatchedParagraph {
  source_index: number
  source_text: string
  target_index: number
  target_text: string
  similarity: number
  level: SimilarityLevel
}

/** 关键词信息 */
export interface KeywordInfo {
  word: string
  frequency: number
  tfidf_score: number
}

/** 文本相似度结果 */
export interface TextSimilarityResult {
  overall_similarity: number
  overall_level: SimilarityLevel
  cosine_similarity: number
  ngram_similarity: number
  edit_distance_similarity: number
  keyword_similarity: number
  matched_paragraphs: MatchedParagraph[]
  source_keywords: KeywordInfo[]
  target_keywords: KeywordInfo[]
  common_keywords: string[]
  confidence: number
  // 各算法执行耗时（微秒）
  tokenization_time_us: number
  cosine_time_us: number
  ngram_time_us: number
  edit_distance_time_us: number
  keyword_extraction_time_us: number
  keyword_similarity_time_us: number
  paragraph_matching_time_us: number
}

/** 匹配的图像对 */
export interface MatchedImage {
  source_index: number
  source_hash: string
  source_width?: number
  source_height?: number
  target_index: number
  target_hash: string
  target_width?: number
  target_height?: number
  similarity: number
  level: ImageSimilarityLevel
  match_type: string
}

/** 图像相似度结果 */
export interface ImageSimilarityResult {
  overall_similarity: number
  overall_level: ImageSimilarityLevel
  matched_images: MatchedImage[]
  source_image_count: number
  target_image_count: number
  matched_count: number
  identical_count: number
}

/** 比较配置 */
export interface ComparisonConfig {
  text_weight: number
  image_weight: number
  paragraph_threshold: number
  image_threshold: number
  display_threshold: number
  granularity: 'overall' | 'paragraph'
}

/** 单次对比结果 */
export interface ComparisonResult {
  id: string
  timestamp: string
  source_file: string
  target_file: string
  source_file_name: string
  target_file_name: string
  overall_similarity: number
  overall_level: string
  text_similarity: number
  text_level: string
  image_similarity: number
  image_level: string
  text_result: TextSimilarityResult
  image_result: ImageSimilarityResult
  source_word_count: number
  target_word_count: number
  source_char_count: number
  target_char_count: number
  source_image_count: number
  target_image_count: number
  processing_time_ms: number
  confidence: number
  config: ComparisonConfig
}

/** 批量比较进度 */
export interface BatchProgress {
  total_pairs: number
  completed_pairs: number
  current_source: string
  current_target: string
  percentage: number
  estimated_remaining_seconds: number
  pairs_per_minute: number
  high_similarity_count: number
  error_count: number
  status: 'running' | 'paused' | 'completed' | 'cancelled'
}

/** 批量比较结果 */
export interface BatchComparisonResult {
  id: string
  timestamp: string
  mode: string
  file_count: number
  total_pairs: number
  results: ComparisonResult[]
  average_similarity: number
  max_similarity: number
  min_similarity: number
  high_similarity_pairs: number
  medium_similarity_pairs: number
  low_similarity_pairs: number
  total_processing_time_ms: number
  similarity_matrix: number[][]
  file_names: string[]
}

/** 批量比较模式 */
export type BatchMode = 'all_pairs' | 'with_first' | 'with_baseline'

/** 文件信息 */
export interface FileInfo {
  path: string
  name: string
  size: number
  type: string
  status: 'pending' | 'parsing' | 'ready' | 'error'
  error?: string
  parsed?: ParsedDocument
}

/** 报告配置 */
export interface ReportConfig {
  format: 'pdf' | 'html' | 'json' | 'csv'
  include_details: boolean
  include_matched_paragraphs: boolean
  include_matched_images: boolean
  include_statistics: boolean
  title?: string
  author?: string
}

// ==================== Store 定义 ====================

export const useSimilarityStore = defineStore('similarity', {
  state: () => ({
    // 模式切换
    mode: 'single' as 'single' | 'batch',
    
    // 单文件模式
    sourceFile: null as FileInfo | null,
    targetFile: null as FileInfo | null,
    
    // 批量模式
    batchFiles: [] as FileInfo[],
    batchMode: 'all_pairs' as BatchMode,
    baselineIndex: 0,
    
    // 配置
    config: {
      text_weight: 0.7,
      image_weight: 0.3,
      paragraph_threshold: 0.5,
      image_threshold: 0.7,
      display_threshold: 0.3,
      granularity: 'paragraph',
    } as ComparisonConfig,
    
    // 结果
    singleResult: null as ComparisonResult | null,
    batchResult: null as BatchComparisonResult | null,
    selectedResultIndex: -1,
    
    // 进度
    isProcessing: false,
    progress: null as BatchProgress | null,
    
    // 历史记录
    history: [] as ComparisonResult[],
    
    // UI状态
    showSettings: false,
    showDetailView: false,
    activeTab: 'overview' as 'overview' | 'text' | 'image' | 'report',
    
    // 错误
    error: null as string | null,
  }),

  getters: {
    /** 是否已选择文件 */
    hasFiles: (state) => {
      if (state.mode === 'single') {
        return state.sourceFile !== null && state.targetFile !== null
      }
      return state.batchFiles.length >= 2
    },

    /** 是否有结果 */
    hasResult: (state) => {
      if (state.mode === 'single') {
        return state.singleResult !== null
      }
      return state.batchResult !== null
    },

    /** 当前选中的详细结果 */
    selectedResult: (state): ComparisonResult | null => {
      if (state.mode === 'single') {
        return state.singleResult
      }
      if (state.batchResult && state.selectedResultIndex >= 0 && state.selectedResultIndex < state.batchResult.results.length) {
        return state.batchResult.results[state.selectedResultIndex] ?? null
      }
      return null
    },

    /** 支持的文件格式 */
    supportedFormats: () => ['txt', 'docx', 'pdf'],

    /** 相似度等级颜色 */
    getSimilarityColor: () => (similarity: number): string => {
      if (similarity >= 0.9) return '#ef4444' // red
      if (similarity >= 0.7) return '#f97316' // orange
      if (similarity >= 0.5) return '#eab308' // yellow
      if (similarity >= 0.3) return '#22c55e' // green
      return '#3b82f6' // blue
    },

    /** 相似度等级文字 */
    getSimilarityLabel: () => (similarity: number): string => {
      if (similarity >= 0.9) return '极高相似'
      if (similarity >= 0.7) return '高度相似'
      if (similarity >= 0.5) return '中等相似'
      if (similarity >= 0.3) return '低度相似'
      return '极低相似'
    },
  },

  actions: {
    /** 切换模式 */
    setMode(mode: 'single' | 'batch') {
      this.mode = mode
      this.clearResults()
    },

    /** 设置源文件 */
    setSourceFile(file: FileInfo) {
      this.sourceFile = file
    },

    /** 设置目标文件 */
    setTargetFile(file: FileInfo) {
      this.targetFile = file
    },

    /** 添加批量文件 */
    addBatchFiles(files: FileInfo[]) {
      this.batchFiles.push(...files)
    },

    /** 移除批量文件 */
    removeBatchFile(index: number) {
      this.batchFiles.splice(index, 1)
    },

    /** 清空批量文件 */
    clearBatchFiles() {
      this.batchFiles = []
    },

    /** 设置批量模式 */
    setBatchMode(mode: BatchMode) {
      this.batchMode = mode
    },

    /** 设置基准文件索引 */
    setBaselineIndex(index: number) {
      this.baselineIndex = index
    },

    /** 更新配置 */
    updateConfig(config: Partial<ComparisonConfig>) {
      this.config = { ...this.config, ...config }
    },

    /** 清空结果 */
    clearResults() {
      this.singleResult = null
      this.batchResult = null
      this.selectedResultIndex = -1
      this.error = null
    },

    /** 验证文件格式 */
    async validateFile(filePath: string): Promise<boolean> {
      try {
        console.log('调用 validate_file_format，参数:', { filePath })
        const result = await invoke<boolean>('validate_file_format', { filePath })
        console.log('validate_file_format 返回:', result)
        return result
      } catch (e) {
        console.error('验证文件格式失败:', e)
        return false
      }
    },

    /** 解析文档 */
    async parseDocument(filePath: string): Promise<ParsedDocument | null> {
      try {
        console.log('开始解析文档:', filePath)
        console.log('调用参数:', { filePath })
        const result = await invoke<ParsedDocument>('parse_document', { filePath })
        console.log('文档解析成功，结果类型:', typeof result)
        console.log('文档解析成功，结果:', result)
        return result
      } catch (e) {
        console.error('解析文档失败，错误类型:', typeof e)
        console.error('解析文档失败，错误详情:', e)
        console.error('解析文档失败，错误字符串:', String(e))
        this.error = `解析文档失败: ${e}`
        return null
      }
    },

    /** 执行单文件对比 */
    async compareSingle(): Promise<boolean> {
      console.log('[compareSingle] 开始执行')
      console.log('[compareSingle] sourceFile:', this.sourceFile)
      console.log('[compareSingle] targetFile:', this.targetFile)
      
      if (!this.sourceFile || !this.targetFile) {
        this.error = '请选择两个文件进行对比'
        console.log('[compareSingle] 文件未选择')
        return false
      }

      console.log('[compareSingle] 设置 isProcessing = true')
      this.isProcessing = true
      this.error = null

      try {
        console.log('[compareSingle] 调用 compare_documents')
        console.log('[compareSingle] 参数:', {
          file1: this.sourceFile.path,
          file2: this.targetFile.path,
          config: this.config,
        })
        
        const result = await invoke<ComparisonResult>('compare_documents', {
          file1: this.sourceFile.path,
          file2: this.targetFile.path,
          config: this.config,
        })

        console.log('[compareSingle] 对比成功，结果:', result)
        this.singleResult = result
        this.addToHistory(result)
        return true
      } catch (e) {
        console.error('[compareSingle] 对比失败:', e)
        this.error = `对比失败: ${e}`
        return false
      } finally {
        console.log('[compareSingle] 设置 isProcessing = false')
        this.isProcessing = false
      }
    },

    /** 执行批量对比 */
    async compareBatch(): Promise<boolean> {
      if (this.batchFiles.length < 2) {
        this.error = '请至少选择两个文件进行对比'
        return false
      }

      this.isProcessing = true
      this.error = null
      this.progress = {
        total_pairs: 0,
        completed_pairs: 0,
        current_source: '',
        current_target: '',
        percentage: 0,
        estimated_remaining_seconds: 0,
        pairs_per_minute: 0,
        high_similarity_count: 0,
        error_count: 0,
        status: 'running',
      }

      try {
        const files = this.batchFiles.map(f => f.path)
        const result = await invoke<BatchComparisonResult>('batch_compare_documents', {
          files,
          mode: this.batchMode,
          baselineIndex: this.batchMode === 'with_baseline' ? this.baselineIndex : null,
          config: this.config,
        })

        this.batchResult = result
        this.progress = {
          ...this.progress!,
          status: 'completed',
          percentage: 100,
          completed_pairs: result.total_pairs,
        }
        return true
      } catch (e) {
        console.error('批量对比失败:', e)
        this.error = `批量对比失败: ${e}`
        if (this.progress) {
          this.progress.status = 'cancelled'
        }
        return false
      } finally {
        this.isProcessing = false
      }
    },

    /** 添加到历史记录 */
    addToHistory(result: ComparisonResult) {
      this.history.unshift(result)
      // 保留最近50条记录
      if (this.history.length > 50) {
        this.history = this.history.slice(0, 50)
      }
    },

    /** 清空历史记录 */
    clearHistory() {
      this.history = []
    },

    /** 选择批量结果详情 */
    selectBatchResult(index: number) {
      this.selectedResultIndex = index
      this.showDetailView = true
    },

    /** 生成报告 */
    async generateReport(outputPath: string, config?: ReportConfig): Promise<string | null> {
      const reportConfig = config || {
        format: 'html' as const,
        include_details: true,
        include_matched_paragraphs: true,
        include_matched_images: true,
        include_statistics: true,
      }

      try {
        if (this.mode === 'single' && this.singleResult) {
          return await invoke<string>('generate_report', {
            result: this.singleResult,
            outputPath,
            config: reportConfig,
          })
        } else if (this.mode === 'batch' && this.batchResult) {
          return await invoke<string>('generate_batch_report', {
            result: this.batchResult,
            outputPath,
            config: reportConfig,
          })
        }
        return null
      } catch (e) {
        console.error('生成报告失败:', e)
        this.error = `生成报告失败: ${e}`
        return null
      }
    },

    /** 获取默认配置 */
    async loadDefaultConfig() {
      try {
        const config = await invoke<ComparisonConfig>('get_default_config')
        this.config = config
      } catch (e) {
        console.error('获取默认配置失败:', e)
      }
    },

    /** 重置所有状态 */
    reset() {
      this.sourceFile = null
      this.targetFile = null
      this.batchFiles = []
      this.singleResult = null
      this.batchResult = null
      this.selectedResultIndex = -1
      this.isProcessing = false
      this.progress = null
      this.error = null
      this.showDetailView = false
    },
  },
})
