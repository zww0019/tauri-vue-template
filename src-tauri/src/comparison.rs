// 综合比较模块 - 整合文本和图像相似度分析
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Local;

use crate::document_parser::{DocumentParser, ParsedDocument};
use crate::text_similarity::{TextSimilarityCalculator, TextSimilarityResult, SimilarityLevel};
use crate::image_similarity::{ImageSimilarityCalculator, ImageSimilarityResult};

/// 比较配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    /// 文本权重 (0-1)
    pub text_weight: f64,
    /// 图像权重 (0-1)
    pub image_weight: f64,
    /// 段落匹配阈值
    pub paragraph_threshold: f64,
    /// 图像匹配阈值
    pub image_threshold: f64,
    /// 相似度显示阈值（低于此值不显示）
    pub display_threshold: f64,
    /// 对比粒度: "overall" | "paragraph"
    pub granularity: String,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            text_weight: 0.7,
            image_weight: 0.3,
            paragraph_threshold: 0.5,
            image_threshold: 0.7,
            display_threshold: 0.3,
            granularity: "paragraph".to_string(),
        }
    }
}

/// 单次对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub id: String,
    pub timestamp: String,
    
    /// 文件信息
    pub source_file: String,
    pub target_file: String,
    pub source_file_name: String,
    pub target_file_name: String,
    
    /// 综合相似度
    pub overall_similarity: f64,
    pub overall_level: String,
    
    /// 文本相似度
    pub text_similarity: f64,
    pub text_level: String,
    
    /// 图像相似度
    pub image_similarity: f64,
    pub image_level: String,
    
    /// 详细结果
    pub text_result: TextSimilarityResult,
    pub image_result: ImageSimilarityResult,
    
    /// 统计信息
    pub source_word_count: usize,
    pub target_word_count: usize,
    pub source_char_count: usize,
    pub target_char_count: usize,
    pub source_image_count: usize,
    pub target_image_count: usize,
    
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    
    /// 置信度
    pub confidence: f64,
    
    /// 使用的配置
    pub config: ComparisonConfig,
}

/// 批量比较模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchMode {
    /// 全两两对比
    AllPairs,
    /// 与第一个对比
    WithFirst,
    /// 与指定基准对比
    WithBaseline(usize),
}

/// 批量比较进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total_pairs: usize,
    pub completed_pairs: usize,
    pub current_source: String,
    pub current_target: String,
    pub percentage: f64,
    pub estimated_remaining_seconds: f64,
    pub pairs_per_minute: f64,
    pub high_similarity_count: usize,
    pub error_count: usize,
    pub status: String, // "running" | "paused" | "completed" | "cancelled"
}

/// 批量比较结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchComparisonResult {
    pub id: String,
    pub timestamp: String,
    pub mode: String,
    pub file_count: usize,
    pub total_pairs: usize,
    pub results: Vec<ComparisonResult>,
    
    /// 统计信息
    pub average_similarity: f64,
    pub max_similarity: f64,
    pub min_similarity: f64,
    pub high_similarity_pairs: usize, // > 70%
    pub medium_similarity_pairs: usize, // 50-70%
    pub low_similarity_pairs: usize, // < 50%
    
    /// 处理时间
    pub total_processing_time_ms: u64,
    
    /// 相似度矩阵（用于热力图）
    pub similarity_matrix: Vec<Vec<f64>>,
    pub file_names: Vec<String>,
}

/// 文档比较器
pub struct DocumentComparator {
    parser: DocumentParser,
    text_calculator: TextSimilarityCalculator,
    image_calculator: ImageSimilarityCalculator,
}

impl Default for DocumentComparator {
    fn default() -> Self {
        Self::new(ComparisonConfig::default())
    }
}

impl DocumentComparator {
    pub fn new(config: ComparisonConfig) -> Self {
        Self {
            parser: DocumentParser::default(),
            text_calculator: TextSimilarityCalculator::new(
                config.paragraph_threshold,
                true,
                20,
            ),
            image_calculator: ImageSimilarityCalculator::new(
                config.image_threshold,
                true,
                true,
            ),
        }
    }

    /// 比较两个文档
    pub fn compare(
        &self,
        file1: &str,
        file2: &str,
        config: &ComparisonConfig,
    ) -> Result<ComparisonResult, String> {
        let start_time = std::time::Instant::now();
        // 打印日志
        println!("开始比较文档: {} 和 {}", file1, file2);
        // 解析文档
        let doc1 = self.parser.parse(file1).map_err(|e| e.to_string())?;
        let doc2 = self.parser.parse(file2).map_err(|e| e.to_string())?;

        // 计算文本相似度
        let text_result = self.text_calculator.calculate(&doc1, &doc2);
        println!("文本相似度: {}", text_result.overall_similarity);
        // 计算图像相似度
        let image_result = self.image_calculator.calculate(&doc1.images, &doc2.images);
        println!("图像相似度: {}", image_result.overall_similarity);

        // 计算综合相似度
        let overall_similarity = if doc1.images.is_empty() && doc2.images.is_empty() {
            // 没有图片时，只考虑文本
            text_result.overall_similarity
        } else {
            text_result.overall_similarity * config.text_weight
                + image_result.overall_similarity * config.image_weight
        };
        println!("综合相似度: {}", overall_similarity);
        let processing_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(ComparisonResult {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            source_file: file1.to_string(),
            target_file: file2.to_string(),
            source_file_name: doc1.metadata.file_name.clone(),
            target_file_name: doc2.metadata.file_name.clone(),
            overall_similarity,
            overall_level: SimilarityLevel::from_score(overall_similarity).description().to_string(),
            text_similarity: text_result.overall_similarity,
            text_level: text_result.overall_level.description().to_string(),
            image_similarity: image_result.overall_similarity,
            image_level: image_result.overall_level.description().to_string(),
            text_result,
            image_result,
            source_word_count: doc1.word_count,
            target_word_count: doc2.word_count,
            source_char_count: doc1.char_count,
            target_char_count: doc2.char_count,
            source_image_count: doc1.images.len(),
            target_image_count: doc2.images.len(),
            processing_time_ms,
            confidence: 0.8, // 简化的置信度
            config: config.clone(),
        })
    }

    /// 批量比较文档
    pub fn batch_compare(
        &self,
        files: &[String],
        mode: BatchMode,
        config: &ComparisonConfig,
        progress_callback: Option<Box<dyn Fn(BatchProgress) + Send + Sync>>,
    ) -> Result<BatchComparisonResult, String> {
        let start_time = std::time::Instant::now();
        
        // 根据模式生成文件对
        let pairs: Vec<(usize, usize)> = match &mode {
            BatchMode::AllPairs => {
                let mut pairs = Vec::new();
                for i in 0..files.len() {
                    for j in (i + 1)..files.len() {
                        pairs.push((i, j));
                    }
                }
                pairs
            }
            BatchMode::WithFirst => {
                (1..files.len()).map(|i| (0, i)).collect()
            }
            BatchMode::WithBaseline(baseline) => {
                (0..files.len())
                    .filter(|&i| i != *baseline)
                    .map(|i| (*baseline, i))
                    .collect()
            }
        };

        let total_pairs = pairs.len();
        
        // 获取文件名
        let file_names: Vec<String> = files
            .iter()
            .map(|f| {
                std::path::Path::new(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            })
            .collect();

        // 使用 Rayon 并行处理文件对，同时支持进度回调
        use rayon::prelude::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let completed_count = Arc::new(AtomicUsize::new(0));
        let error_count_atomic = Arc::new(AtomicUsize::new(0));
        let high_similarity_count_atomic = Arc::new(AtomicUsize::new(0));
        
        let results: Vec<Result<ComparisonResult, String>> = pairs
            .par_iter()
            .map(|(i, j)| {
                let result = self.compare(&files[*i], &files[*j], config);
                
                // 更新计数器
                let completed = completed_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                // 更新统计
                match &result {
                    Ok(r) => {
                        if r.overall_similarity >= 0.7 {
                            high_similarity_count_atomic.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                    Err(_) => {
                        error_count_atomic.fetch_add(1, Ordering::SeqCst);
                    }
                }
                
                // 调用进度回调（每完成一个任务就回调一次）
                if let Some(ref callback) = progress_callback {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let pairs_per_second = if elapsed > 0.0 { completed as f64 / elapsed } else { 0.0 };
                    let remaining = if pairs_per_second > 0.0 {
                        (total_pairs - completed) as f64 / pairs_per_second
                    } else {
                        0.0
                    };
                    
                    callback(BatchProgress {
                        total_pairs,
                        completed_pairs: completed,
                        current_source: file_names[*i].clone(),
                        current_target: file_names[*j].clone(),
                        percentage: completed as f64 / total_pairs as f64 * 100.0,
                        estimated_remaining_seconds: remaining,
                        pairs_per_minute: pairs_per_second * 60.0,
                        high_similarity_count: high_similarity_count_atomic.load(Ordering::SeqCst),
                        error_count: error_count_atomic.load(Ordering::SeqCst),
                        status: "running".to_string(),
                    });
                }
                
                result
            })
            .collect();
        
        // 收集成功的结果
        let results: Vec<ComparisonResult> = results.into_iter().flatten().collect();

        // 计算统计信息
        let similarities: Vec<f64> = results.iter().map(|r| r.overall_similarity).collect();
        let average_similarity = if similarities.is_empty() {
            0.0
        } else {
            similarities.iter().sum::<f64>() / similarities.len() as f64
        };
        let max_similarity = similarities.iter().cloned().fold(0.0, f64::max);
        let min_similarity = similarities.iter().cloned().fold(1.0, f64::min);

        let high_similarity_pairs = similarities.iter().filter(|&&s| s >= 0.7).count();
        let medium_similarity_pairs = similarities.iter().filter(|&&s| (0.5..0.7).contains(&s)).count();
        let low_similarity_pairs = similarities.iter().filter(|&&s| s < 0.5).count();

        // 构建相似度矩阵
        let n = files.len();
        let mut similarity_matrix = vec![vec![0.0; n]; n];
        
        // 对角线为1
        for (i, row) in similarity_matrix.iter_mut().enumerate().take(n) {
            row[i] = 1.0;
        }
        
        // 填充结果
        for result in &results {
            if let (Some(i), Some(j)) = (
                files.iter().position(|f| f == &result.source_file),
                files.iter().position(|f| f == &result.target_file),
            ) {
                similarity_matrix[i][j] = result.overall_similarity;
                similarity_matrix[j][i] = result.overall_similarity;
            }
        }

        let mode_str = match mode {
            BatchMode::AllPairs => "all_pairs",
            BatchMode::WithFirst => "with_first",
            BatchMode::WithBaseline(_) => "with_baseline",
        };

        Ok(BatchComparisonResult {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            mode: mode_str.to_string(),
            file_count: files.len(),
            total_pairs,
            results,
            average_similarity,
            max_similarity,
            min_similarity,
            high_similarity_pairs,
            medium_similarity_pairs,
            low_similarity_pairs,
            total_processing_time_ms: start_time.elapsed().as_millis() as u64,
            similarity_matrix,
            file_names,
        })
    }
}

// ==================== Tauri Commands ====================

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// 全局比较任务管理
static COMPARISON_TASKS: Lazy<Arc<Mutex<HashMap<String, BatchProgress>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// 解析单个文档
#[tauri::command]
pub async fn parse_document(file_path: String) -> Result<ParsedDocument, String> {
    println!("开始解析文档: {}", file_path);
    let parser = DocumentParser::default();
    let result = tokio::task::spawn_blocking(move || {
        parser.parse(&file_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?;
    
    match &result {
        Ok(doc) => {
            println!("文档解析完成 - 字数: {}, 段落: {}, 图片: {}", 
                doc.word_count, doc.paragraphs.len(), doc.images.len());
        }
        Err(e) => {
            println!("文档解析失败: {}", e);
        }
    }
    
    result
}

/// 比较两个文档
#[tauri::command]
pub async fn compare_documents(
    file1: String,
    file2: String,
    config: Option<ComparisonConfig>,
) -> Result<ComparisonResult, String> {
    println!("========================================");
    println!("开始文档对比");
    println!("源文件: {}", file1);
    println!("目标文件: {}", file2);
    println!("========================================");
    
    let config = config.unwrap_or_default();
    println!("配置参数: text_weight={}, image_weight={}", config.text_weight, config.image_weight);
    
    let comparator = DocumentComparator::new(config.clone());
    
    println!("正在执行对比计算...");
    let result = tokio::task::spawn_blocking(move || {
        comparator.compare(&file1, &file2, &config)
    })
    .await
    .map_err(|e| {
        println!("对比任务执行失败: {}", e);
        e.to_string()
    })?;
    
    match &result {
        Ok(res) => {
            println!("========================================");
            println!("对比完成！");
            println!("综合相似度: {:.2}%", res.overall_similarity * 100.0);
            println!("文本相似度: {:.2}%", res.text_similarity * 100.0);
            println!("图像相似度: {:.2}%", res.image_similarity * 100.0);
            println!("处理耗时: {} ms", res.processing_time_ms);
            println!("========================================");
        }
        Err(e) => {
            println!("对比失败: {}", e);
        }
    }
    
    result
}

/// 批量比较文档
#[tauri::command]
pub async fn batch_compare_documents(
    files: Vec<String>,
    mode: String,
    baseline_index: Option<usize>,
    config: Option<ComparisonConfig>,
) -> Result<BatchComparisonResult, String> {
    let config = config.unwrap_or_default();
    let comparator = DocumentComparator::new(config.clone());
    
    let batch_mode = match mode.as_str() {
        "all_pairs" => BatchMode::AllPairs,
        "with_first" => BatchMode::WithFirst,
        "with_baseline" => BatchMode::WithBaseline(baseline_index.unwrap_or(0)),
        _ => BatchMode::AllPairs,
    };
    
    tokio::task::spawn_blocking(move || {
        comparator.batch_compare(&files, batch_mode, &config, None)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// 获取批量比较进度
#[tauri::command]
pub async fn get_batch_progress(task_id: String) -> Result<Option<BatchProgress>, String> {
    let tasks = COMPARISON_TASKS.lock().await;
    Ok(tasks.get(&task_id).cloned())
}

/// 验证文件格式
#[tauri::command]
pub fn validate_file_format(file_path: String) -> Result<bool, String> {
    println!("验证文件格式: {}", file_path);
    let result = crate::document_parser::is_supported_format(&file_path);
    println!("验证结果: {}", result);
    Ok(result)
}

/// 获取支持的文件格式
#[tauri::command]
pub fn get_supported_formats() -> Vec<String> {
    vec!["txt".to_string(), "docx".to_string(), "pdf".to_string()]
}

/// 获取默认配置
#[tauri::command]
pub fn get_default_config() -> ComparisonConfig {
    ComparisonConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ComparisonConfig::default();
        assert!((config.text_weight + config.image_weight - 1.0).abs() < 0.001);
    }
}
