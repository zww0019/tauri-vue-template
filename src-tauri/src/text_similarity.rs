// 文本相似度计算模块
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::document_parser::{ParsedDocument, Paragraph};

// 全局 Jieba 实例，避免重复初始化词典
static JIEBA: LazyLock<jieba_rs::Jieba> = LazyLock::new(jieba_rs::Jieba::new);

/// 相似度等级
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SimilarityLevel {
    VeryLow,   // 0-29%
    Low,       // 30-49%
    Medium,    // 50-69%
    High,      // 70-89%
    VeryHigh,  // 90-100%
}

impl SimilarityLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 0.9 => SimilarityLevel::VeryHigh,
            s if s >= 0.7 => SimilarityLevel::High,
            s if s >= 0.5 => SimilarityLevel::Medium,
            s if s >= 0.3 => SimilarityLevel::Low,
            _ => SimilarityLevel::VeryLow,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SimilarityLevel::VeryHigh => "极高相似",
            SimilarityLevel::High => "高度相似",
            SimilarityLevel::Medium => "中等相似",
            SimilarityLevel::Low => "低度相似",
            SimilarityLevel::VeryLow => "极低相似",
        }
    }
}

/// 匹配的段落对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedParagraph {
    pub source_index: usize,
    pub source_text: String,
    pub target_index: usize,
    pub target_text: String,
    pub similarity: f64,
    pub level: SimilarityLevel,
}

/// 关键词信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordInfo {
    pub word: String,
    pub frequency: usize,
    pub tfidf_score: f64,
}

/// 【Phase 2.3】段落特征（用于快速过滤）
#[derive(Debug, Clone)]
struct ParagraphFeatures {
    char_count: usize,
    vocab_size: usize,
    vocab: HashSet<String>,
}


/// 文本相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSimilarityResult {
    /// 整体相似度 (0-1)
    pub overall_similarity: f64,
    /// 整体相似度等级
    pub overall_level: SimilarityLevel,
    
    /// 余弦相似度
    pub cosine_similarity: f64,
    /// N-gram 相似度
    pub ngram_similarity: f64,
    /// 编辑距离相似度（归一化，仅短文本）
    pub edit_distance_similarity: f64,
    /// 关键词相似度
    pub keyword_similarity: f64,
    /// 匹配的段落
    pub matched_paragraphs: Vec<MatchedParagraph>,
    
    /// 源文档关键词
    pub source_keywords: Vec<KeywordInfo>,
    /// 目标文档关键词
    pub target_keywords: Vec<KeywordInfo>,
    /// 共同关键词
    pub common_keywords: Vec<String>,
    
    /// 置信度 (0-1)
    pub confidence: f64,
    
    /// 各算法执行耗时（微秒）
    pub tokenization_time_us: u64,
    pub cosine_time_us: u64,
    pub ngram_time_us: u64,
    pub edit_distance_time_us: u64,
    pub keyword_extraction_time_us: u64,
    pub keyword_similarity_time_us: u64,
    pub paragraph_matching_time_us: u64,
}

/// 文本相似度计算器
pub struct TextSimilarityCalculator {
    /// 段落匹配阈值
    paragraph_threshold: f64,
    /// 是否使用分词
    use_segmentation: bool,
    /// 关键词数量
    keyword_count: usize,
    /// 短文本阈值（字符数）
    short_text_threshold: usize,
}

impl Default for TextSimilarityCalculator {
    fn default() -> Self {
        Self {
            paragraph_threshold: 0.5,
            use_segmentation: true,
            keyword_count: 20,
            short_text_threshold: 1000,
        }
    }
}

impl TextSimilarityCalculator {
    pub fn new(
        paragraph_threshold: f64,
        use_segmentation: bool,
        keyword_count: usize,
    ) -> Self {
        Self {
            paragraph_threshold,
            use_segmentation,
            keyword_count,
            short_text_threshold: 1000,
        }
    }

    /// 计算两个文档的文本相似度（带时间记录和多线程优化）
    pub fn calculate(
        &self,
        doc1: &ParsedDocument,
        doc2: &ParsedDocument,
    ) -> TextSimilarityResult {
        use std::time::Instant;
        
        // 1. 分词（记录时间）
        let tokenization_start = Instant::now();
        let tokens1 = self.tokenize(&doc1.full_text);
        let tokens2 = self.tokenize(&doc2.full_text);
        let tokenization_time_us = tokenization_start.elapsed().as_micros() as u64;

        // 构建语料库用于计算 IDF
        let corpus = vec![tokens1.clone(), tokens2.clone()];
        
        // 2. 计算各种相似度指标（记录每个算法的执行时间）
        let cosine_start = Instant::now();
        let cosine_similarity = self.cosine_similarity(&tokens1, &tokens2);
        let cosine_time_us = cosine_start.elapsed().as_micros() as u64;
        
        // N-gram 相似度
        let ngram_start = Instant::now();
        let ngram_similarity = self.ngram_similarity(&tokens1, &tokens2);
        let ngram_time_us = ngram_start.elapsed().as_micros() as u64;
        
        // 编辑距离（仅用于短文本）
        let edit_start = Instant::now();
        let avg_len = (doc1.full_text.len() + doc2.full_text.len()) / 2;
        let edit_distance_similarity = if avg_len <= self.short_text_threshold {
            self.normalized_edit_distance(&doc1.full_text, &doc2.full_text)
        } else {
            0.0 // 长文本跳过编辑距离计算
        };
        let edit_distance_time_us = edit_start.elapsed().as_micros() as u64;
        
        // 3. 关键词提取和相似度计算
        let keyword_extraction_start = Instant::now();
        let source_keywords = self.extract_keywords_with_tfidf(&tokens1, &corpus);
        let target_keywords = self.extract_keywords_with_tfidf(&tokens2, &corpus);
        let keyword_extraction_time_us = keyword_extraction_start.elapsed().as_micros() as u64;
        
        let keyword_similarity_start = Instant::now();
        let keyword_similarity = self.keyword_similarity(&source_keywords, &target_keywords);
        let common_keywords = self.find_common_keywords(&source_keywords, &target_keywords);
        let keyword_similarity_time_us = keyword_similarity_start.elapsed().as_micros() as u64;
        
        // 4. 段落匹配（记录时间）
        let paragraph_matching_start = Instant::now();
        let matched_paragraphs = self.match_paragraphs(&doc1.paragraphs, &doc2.paragraphs);
        let paragraph_matching_time_us = paragraph_matching_start.elapsed().as_micros() as u64;

        // 5. 动态权重计算综合相似度
        let overall_similarity = self.compute_overall_similarity_new(
            cosine_similarity,
            ngram_similarity,
            edit_distance_similarity,
            keyword_similarity,
            doc1,
            doc2,
        );

        // 6. 改进的置信度计算
        let confidence = self.compute_confidence_improved(
            doc1,
            doc2,
            cosine_similarity,
            ngram_similarity,
            &matched_paragraphs,
        );
        
        // 【性能日志】输出各阶段耗时
        let total_time_ms = tokenization_time_us + cosine_time_us + ngram_time_us 
            + edit_distance_time_us + keyword_extraction_time_us + keyword_similarity_time_us 
            + paragraph_matching_time_us;
        
        println!("=== 文本相似度计算性能统计 ===");
        println!("分词: {}ms", tokenization_time_us / 1000);
        println!("余弦相似度: {}ms", cosine_time_us / 1000);
        println!("N-gram相似度: {}ms", ngram_time_us / 1000);
        println!("编辑距离: {}ms", edit_distance_time_us / 1000);
        println!("关键词提取: {}ms", keyword_extraction_time_us / 1000);
        println!("关键词相似度: {}ms", keyword_similarity_time_us / 1000);
        println!("段落匹配({}x{}): {}ms", doc1.paragraphs.len(), doc2.paragraphs.len(), paragraph_matching_time_us / 1000);
        println!("总耗时: {}ms", total_time_ms / 1000);
        println!("匹配结果: {}个段落", matched_paragraphs.len());
        println!("==============================");

        TextSimilarityResult {
            overall_similarity,
            overall_level: SimilarityLevel::from_score(overall_similarity),
            cosine_similarity,
            ngram_similarity,
            edit_distance_similarity,
            keyword_similarity,
            matched_paragraphs,
            source_keywords,
            target_keywords,
            common_keywords,
            confidence,
            tokenization_time_us,
            cosine_time_us,
            ngram_time_us,
            edit_distance_time_us,
            keyword_extraction_time_us,
            keyword_similarity_time_us,
            paragraph_matching_time_us,
        }
    }

    /// 分词（使用全局 Jieba 实例避免重复初始化）
    fn tokenize(&self, text: &str) -> Vec<String> {
        if self.use_segmentation {
            // 使用全局结巴分词实例
            JIEBA.cut(text, false)
                .into_iter()
                .map(|s| s.to_lowercase())
                .filter(|s| !s.trim().is_empty() && s.chars().any(|c| c.is_alphanumeric()))
                .collect()
        } else {
            // 简单分词（按空格和标点）
            text.split(|c: char| !c.is_alphanumeric())
                .map(|s| s.to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        }
    }

    /// 余弦相似度
    fn cosine_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        let freq1 = self.term_frequency(tokens1);
        let freq2 = self.term_frequency(tokens2);

        let all_terms: HashSet<_> = freq1.keys().chain(freq2.keys()).collect();
        
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;

        for term in all_terms {
            let f1 = *freq1.get(term).unwrap_or(&0) as f64;
            let f2 = *freq2.get(term).unwrap_or(&0) as f64;
            
            dot_product += f1 * f2;
            norm1 += f1 * f1;
            norm2 += f2 * f2;
        }

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1.sqrt() * norm2.sqrt())
    }

    /// N-gram 相似度（组合2-gram和3-gram）
    fn ngram_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }
        
        // 2-gram
        let bigrams1 = self.extract_ngrams(tokens1, 2);
        let bigrams2 = self.extract_ngrams(tokens2, 2);
        let bigram_sim = if !bigrams1.is_empty() && !bigrams2.is_empty() {
            self.set_similarity(&bigrams1, &bigrams2)
        } else {
            0.0
        };
        
        // 3-gram
        let trigrams1 = self.extract_ngrams(tokens1, 3);
        let trigrams2 = self.extract_ngrams(tokens2, 3);
        let trigram_sim = if !trigrams1.is_empty() && !trigrams2.is_empty() {
            self.set_similarity(&trigrams1, &trigrams2)
        } else {
            0.0
        };
        
        // 加权组合：2-gram权重更高
        bigram_sim * 0.6 + trigram_sim * 0.4
    }
    
    /// 集合相似度（Jaccard）
    fn set_similarity(&self, set1: &[String], set2: &[String]) -> f64 {
        let s1: HashSet<_> = set1.iter().collect();
        let s2: HashSet<_> = set2.iter().collect();
        
        let intersection = s1.intersection(&s2).count();
        let union = s1.union(&s2).count();
        
        if union == 0 {
            return 0.0;
        }
        
        intersection as f64 / union as f64
    }

    /// 归一化编辑距离
    fn normalized_edit_distance(&self, text1: &str, text2: &str) -> f64 {
        strsim::normalized_levenshtein(text1, text2)
    }
    
    /// 词频统计
    fn term_frequency(&self, tokens: &[String]) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for token in tokens {
            *freq.entry(token.clone()).or_insert(0) += 1;
        }
        freq
    }

    /// 关键词相似度
    fn keyword_similarity(
        &self,
        keywords1: &[KeywordInfo],
        keywords2: &[KeywordInfo],
    ) -> f64 {
        let set1: HashSet<_> = keywords1.iter().map(|k| &k.word).collect();
        let set2: HashSet<_> = keywords2.iter().map(|k| &k.word).collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f64 / union as f64
    }

    /// 找出共同关键词
    fn find_common_keywords(
        &self,
        keywords1: &[KeywordInfo],
        keywords2: &[KeywordInfo],
    ) -> Vec<String> {
        let set1: HashSet<_> = keywords1.iter().map(|k| k.word.clone()).collect();
        let set2: HashSet<_> = keywords2.iter().map(|k| k.word.clone()).collect();

        set1.intersection(&set2).cloned().collect()
    }

    /// 匹配段落（优化：降低阈值从50到30）
    fn match_paragraphs(
        &self,
        paragraphs1: &[Paragraph],
        paragraphs2: &[Paragraph],
    ) -> Vec<MatchedParagraph> {
        // 对于大文档，使用优化的匹配算法（降低阈值从50到30）
        if paragraphs1.len() > 30 || paragraphs2.len() > 30 {
            self.match_paragraphs_optimized(paragraphs1, paragraphs2)
        } else {
            self.match_paragraphs_basic(paragraphs1, paragraphs2)
        }
    }

    /// 【Phase 2.3】基础段落匹配（适用于小文档）
    fn match_paragraphs_basic(
        &self,
        paragraphs1: &[Paragraph],
        paragraphs2: &[Paragraph],
    ) -> Vec<MatchedParagraph> {
        let mut matches = Vec::new();

        for p1 in paragraphs1 {
            for p2 in paragraphs2 {
                let similarity = self.text_similarity(&p1.text, &p2.text);
                
                if similarity >= self.paragraph_threshold {
                    matches.push(MatchedParagraph {
                        source_index: p1.index,
                        source_text: p1.text.clone(),
                        target_index: p2.index,
                        target_text: p2.text.clone(),
                        similarity,
                        level: SimilarityLevel::from_score(similarity),
                    });
                }
            }
        }

        // 按相似度降序排序
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        matches
    }

    /// 【Phase 2.3】优化的段落匹配（适用于大文档）
    /// 使用预过滤策略减少不必要的比较
    fn match_paragraphs_optimized(
        &self,
        paragraphs1: &[Paragraph],
        paragraphs2: &[Paragraph],
    ) -> Vec<MatchedParagraph> {
        use rayon::prelude::*;
        
        // 1. 为每个段落计算简单特征（用于快速过滤）
        let features1: Vec<_> = paragraphs1.par_iter()
            .map(|p| self.compute_paragraph_features(&p.text))
            .collect();
        
        let features2: Vec<_> = paragraphs2.par_iter()
            .map(|p| self.compute_paragraph_features(&p.text))
            .collect();
        
        // 2. 并行计算匹配
        let matches: Vec<MatchedParagraph> = paragraphs1.par_iter()
            .enumerate()
            .flat_map(|(i, p1)| {
                let f1 = &features1[i];
                
                paragraphs2.iter()
                    .enumerate()
                    .filter_map(|(j, p2)| {
                        let f2 = &features2[j];
                        
                        // 快速预过滤：长度差异太大或特征差异太大则跳过
                        if !self.should_compare_paragraphs(f1, f2) {
                            return None;
                        }
                        
                        // 详细相似度计算
                        let similarity = self.text_similarity(&p1.text, &p2.text);
                        
                        if similarity >= self.paragraph_threshold {
                            Some(MatchedParagraph {
                                source_index: p1.index,
                                source_text: p1.text.clone(),
                                target_index: p2.index,
                                target_text: p2.text.clone(),
                                similarity,
                                level: SimilarityLevel::from_score(similarity),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        
        // 按相似度降序排序
        let mut sorted_matches = matches;
        sorted_matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        sorted_matches
    }

    /// 【Phase 2.3】计算段落特征（用于快速过滤）
    fn compute_paragraph_features(&self, text: &str) -> ParagraphFeatures {
        let tokens = self.tokenize(text);
        let char_count = text.chars().count();
        
        // 计算词汇集合（用于快速 Jaccard 估算）
        let vocab: HashSet<String> = tokens.into_iter().collect();
        
        ParagraphFeatures {
            char_count,
            vocab_size: vocab.len(),
            vocab,
        }
    }

    /// 【Phase 2.3】判断是否应该比较两个段落
    fn should_compare_paragraphs(&self, f1: &ParagraphFeatures, f2: &ParagraphFeatures) -> bool {
        // 1. 长度过滤：长度差异超过3倍则跳过
        let len_ratio = f1.char_count.min(f2.char_count) as f64 / f1.char_count.max(f2.char_count).max(1) as f64;
        if len_ratio < 0.33 {
            return false;
        }
        
        // 2. 词汇量过滤
        let vocab_ratio = f1.vocab_size.min(f2.vocab_size) as f64 / f1.vocab_size.max(f2.vocab_size).max(1) as f64;
        if vocab_ratio < 0.2 {
            return false;
        }
        
        // 3. 快速 Jaccard 估算（只用词汇集合）
        let intersection = f1.vocab.intersection(&f2.vocab).count();
        let union = f1.vocab.union(&f2.vocab).count();
        let quick_jaccard = if union > 0 {
            intersection as f64 / union as f64
        } else {
            0.0
        };
        
        // 如果快速 Jaccard 太低，跳过详细计算
        quick_jaccard >= self.paragraph_threshold * 0.5
    }

    /// 【重构】计算两段文本的相似度（用于段落匹配）
    /// 权重配置：
    /// - 短文本(<500字符): 余弦40% + N-gram30% + 编辑距离30%
    /// - 长文本(≥500字符): 余弦60% + N-gram40%
    fn text_similarity(&self, text1: &str, text2: &str) -> f64 {
        let tokens1 = self.tokenize(text1);
        let tokens2 = self.tokenize(text2);

        let cosine = self.cosine_similarity(&tokens1, &tokens2);
        let ngram = self.ngram_similarity(&tokens1, &tokens2);
        
        // 对于短文本（<500字符），使用编辑距离提高精度
        let edit_dist = if text1.len() < 500 && text2.len() < 500 {
            strsim::normalized_levenshtein(text1, text2)
        } else {
            0.0
        };

        // 加权平均（推荐配置）
        if edit_dist > 0.0 {
            // 短文本：余弦40% + N-gram30% + 编辑距离30%
            cosine * 0.40 + ngram * 0.30 + edit_dist * 0.30
        } else {
            // 长文本：余弦60% + N-gram40%
            cosine * 0.60 + ngram * 0.40
        }
    }

    /// 【重构】综合相似度计算 - 按照推荐的算法比例优化
    /// 权重配置：
    /// - 长文本(>1000字): 余弦45% + N-gram30% + 关键词25%
    /// - 短文本(≤1000字): 余弦35% + N-gram25% + 关键词15% + 编辑距离25%
    fn compute_overall_similarity_new(
        &self,
        cosine: f64,
        ngram: f64,
        edit_dist: f64,
        keyword: f64,
        doc1: &ParsedDocument,
        doc2: &ParsedDocument,
    ) -> f64 {
        let avg_len = (doc1.char_count + doc2.char_count) / 2;
        
        // 根据文档长度动态调整权重
        let (w_cosine, w_ngram, w_edit, w_keyword) = if avg_len <= self.short_text_threshold {
            // 短文本(≤1000字): 启用编辑距离
            // 推荐配置：余弦35% + N-gram25% + 关键词15% + 编辑距离25%
            (0.35, 0.25, 0.25, 0.15)
        } else {
            // 长文本(>1000字): 禁用编辑距离(太慢)
            // 推荐配置：余弦45% + N-gram30% + 关键词25%
            (0.45, 0.30, 0.0, 0.25)
        };
        
        // 加权平均计算最终相似度
        let overall = cosine * w_cosine 
            + ngram * w_ngram 
            + edit_dist * w_edit 
            + keyword * w_keyword;
        
        // 确保结果在[0, 1]范围内
        overall.clamp(0.0, 1.0)
    }


    // ==================== Phase 1 & 2 优化方法 ====================

    /// 【Phase 1.1】使用完整 TF-IDF 提取关键词
    fn extract_keywords_with_tfidf(&self, tokens: &[String], corpus: &[Vec<String>]) -> Vec<KeywordInfo> {
        let tf = self.term_frequency(tokens);
        let total_tokens = tokens.len() as f64;
        let total_docs = corpus.len() as f64;
        
        // 停用词列表
        let stopwords: HashSet<&str> = [
            "的", "了", "是", "在", "我", "有", "和", "就", "不", "人", "都", "一", "一个",
            "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好",
            "自己", "这", "那", "些", "个", "们", "为", "与", "及", "其", "中", "而", "或",
            "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "can", "to", "of", "in", "for", "on", "with",
            "at", "by", "from", "or", "and", "but", "if", "then", "else", "when", "up",
            "down", "out", "as", "so", "it", "its", "this", "that", "these", "those",
        ].iter().cloned().collect();
        
        // 计算 IDF
        let mut idf_map = HashMap::new();
        for term in tf.keys() {
            let doc_count = corpus.iter()
                .filter(|doc| doc.contains(term))
                .count() as f64;
            // IDF = log(总文档数 / (包含该词的文档数 + 1))
            let idf = (total_docs / (1.0 + doc_count)).ln();
            idf_map.insert(term.clone(), idf);
        }
        
        // 计算 TF-IDF
        let mut keywords: Vec<KeywordInfo> = tf
            .into_iter()
            .filter(|(word, _)| {
                word.chars().count() > 1 && 
                !stopwords.contains(word.as_str()) &&
                word.chars().any(|c| c.is_alphanumeric())
            })
            .map(|(word, count)| {
                let tf_score = count as f64 / total_tokens;
                let idf_score = idf_map.get(&word).unwrap_or(&0.0);
                let tfidf = tf_score * idf_score;
                
                KeywordInfo {
                    word,
                    frequency: count,
                    tfidf_score: tfidf,
                }
            })
            .collect();
        
        // 按 TF-IDF 分数排序并取前N个
        keywords.sort_by(|a, b| b.tfidf_score.partial_cmp(&a.tfidf_score).unwrap_or(std::cmp::Ordering::Equal));
        keywords.truncate(self.keyword_count);
        
        keywords
    }

    /// 【Phase 1.2】改进的置信度计算
    fn compute_confidence_improved(
        &self,
        doc1: &ParsedDocument,
        doc2: &ParsedDocument,
        cosine: f64,
        ngram: f64,
        matched_paragraphs: &[MatchedParagraph],
    ) -> f64 {
        // 1. 文档长度相似性
        let len_similarity = {
            let len1 = doc1.char_count as f64;
            let len2 = doc2.char_count as f64;
            if len1 == 0.0 || len2 == 0.0 {
                0.0
            } else {
                len1.min(len2) / len1.max(len2)
            }
        };
        
        // 2. 指标一致性（多个指标结果接近说明可信）
        let metrics = [cosine, ngram];
        let mean = metrics.iter().sum::<f64>() / metrics.len() as f64;
        let variance = metrics.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / metrics.len() as f64;
        let consistency = (1.0 - variance.sqrt()).max(0.0);
        
        // 3. 匹配质量（高相似度匹配的比例）
        let match_quality = if matched_paragraphs.is_empty() {
            0.5 // 没有匹配时给中等置信度
        } else {
            let high_quality_matches = matched_paragraphs.iter()
                .filter(|m| m.similarity > 0.8)
                .count() as f64;
            high_quality_matches / matched_paragraphs.len() as f64
        };
        
        // 4. 结构相似性（段落数量接近）
        let structure_similarity = {
            let p1 = doc1.paragraphs.len() as f64;
            let p2 = doc2.paragraphs.len() as f64;
            if p1 == 0.0 || p2 == 0.0 {
                0.5
            } else {
                p1.min(p2) / p1.max(p2)
            }
        };
        
        // 综合置信度（加权平均）
        (len_similarity * 0.2 + consistency * 0.3 + match_quality * 0.3 + structure_similarity * 0.2)
            .clamp(0.0, 1.0)
    }


    /// 【Phase 2.1】提取 N-gram 特征
    fn extract_ngrams(&self, tokens: &[String], n: usize) -> Vec<String> {
        if tokens.len() < n {
            return Vec::new();
        }
        
        tokens.windows(n)
            .map(|window| window.join(" "))
            .collect()
    }

}

/// 快速计算两个文本的相似度（不返回详细信息）
pub fn quick_similarity(text1: &str, text2: &str) -> f64 {
    let calculator = TextSimilarityCalculator::default();
    let tokens1 = calculator.tokenize(text1);
    let tokens2 = calculator.tokenize(text2);
    
    let cosine = calculator.cosine_similarity(&tokens1, &tokens2);
    let ngram = calculator.ngram_similarity(&tokens1, &tokens2);
    
    cosine * 0.6 + ngram * 0.4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_level() {
        assert_eq!(SimilarityLevel::from_score(0.95), SimilarityLevel::VeryHigh);
        assert_eq!(SimilarityLevel::from_score(0.75), SimilarityLevel::High);
        assert_eq!(SimilarityLevel::from_score(0.55), SimilarityLevel::Medium);
        assert_eq!(SimilarityLevel::from_score(0.35), SimilarityLevel::Low);
        assert_eq!(SimilarityLevel::from_score(0.15), SimilarityLevel::VeryLow);
    }

    #[test]
    fn test_cosine_similarity() {
        let calculator = TextSimilarityCalculator::default();
        let tokens1 = vec!["hello".to_string(), "world".to_string()];
        let tokens2 = vec!["hello".to_string(), "world".to_string()];
        
        let similarity = calculator.cosine_similarity(&tokens1, &tokens2);
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ngram_similarity() {
        let calculator = TextSimilarityCalculator::default();
        let tokens1 = vec!["hello".to_string(), "world".to_string(), "test".to_string()];
        let tokens2 = vec!["hello".to_string(), "world".to_string(), "demo".to_string()];
        
        let similarity = calculator.ngram_similarity(&tokens1, &tokens2);
        assert!(similarity > 0.0);
    }
}
