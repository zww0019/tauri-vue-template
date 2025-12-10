// 文本相似度计算模块
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use rayon::prelude::*;

use crate::document_parser::{ParsedDocument, Paragraph, Sentence};

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

/// 匹配的句子对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedSentence {
    pub source_index: usize,
    pub source_text: String,
    pub source_paragraph: usize,
    pub target_index: usize,
    pub target_text: String,
    pub target_paragraph: usize,
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

/// 文本相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSimilarityResult {
    /// 整体相似度 (0-1)
    pub overall_similarity: f64,
    /// 整体相似度等级
    pub overall_level: SimilarityLevel,
    
    /// 余弦相似度
    pub cosine_similarity: f64,
    /// Jaccard 相似度
    pub jaccard_similarity: f64,
    /// 编辑距离相似度（归一化）
    pub edit_distance_similarity: f64,
    /// 关键词相似度
    pub keyword_similarity: f64,
    
    /// 匹配的段落
    pub matched_paragraphs: Vec<MatchedParagraph>,
    /// 匹配的句子
    pub matched_sentences: Vec<MatchedSentence>,
    
    /// 源文档关键词
    pub source_keywords: Vec<KeywordInfo>,
    /// 目标文档关键词
    pub target_keywords: Vec<KeywordInfo>,
    /// 共同关键词
    pub common_keywords: Vec<String>,
    
    /// 置信度 (0-1)
    pub confidence: f64,
}

/// 文本相似度计算器
pub struct TextSimilarityCalculator {
    /// 段落匹配阈值
    paragraph_threshold: f64,
    /// 句子匹配阈值
    sentence_threshold: f64,
    /// 是否使用分词
    use_segmentation: bool,
    /// 关键词数量
    keyword_count: usize,
}

impl Default for TextSimilarityCalculator {
    fn default() -> Self {
        Self {
            paragraph_threshold: 0.5,
            sentence_threshold: 0.6,
            use_segmentation: true,
            keyword_count: 20,
        }
    }
}

impl TextSimilarityCalculator {
    pub fn new(
        paragraph_threshold: f64,
        sentence_threshold: f64,
        use_segmentation: bool,
        keyword_count: usize,
    ) -> Self {
        Self {
            paragraph_threshold,
            sentence_threshold,
            use_segmentation,
            keyword_count,
        }
    }

    /// 计算两个文档的文本相似度
    pub fn calculate(
        &self,
        doc1: &ParsedDocument,
        doc2: &ParsedDocument,
    ) -> TextSimilarityResult {
        // 分词
        let tokens1 = self.tokenize(&doc1.full_text);
        let tokens2 = self.tokenize(&doc2.full_text);

        // 计算各种相似度指标
        let cosine_similarity = self.cosine_similarity(&tokens1, &tokens2);
        let jaccard_similarity = self.jaccard_similarity(&tokens1, &tokens2);
        let edit_distance_similarity = self.normalized_edit_distance(&doc1.full_text, &doc2.full_text);
        
        // 计算关键词
        let source_keywords = self.extract_keywords(&tokens1);
        let target_keywords = self.extract_keywords(&tokens2);
        let keyword_similarity = self.keyword_similarity(&source_keywords, &target_keywords);
        let common_keywords = self.find_common_keywords(&source_keywords, &target_keywords);

        // 计算段落匹配
        let matched_paragraphs = self.match_paragraphs(&doc1.paragraphs, &doc2.paragraphs);
        
        // 计算句子匹配
        let matched_sentences = self.match_sentences(&doc1.sentences, &doc2.sentences);

        // 综合相似度计算
        let overall_similarity = self.compute_overall_similarity(
            cosine_similarity,
            jaccard_similarity,
            edit_distance_similarity,
            keyword_similarity,
        );

        // 计算置信度
        let confidence = self.compute_confidence(
            &doc1,
            &doc2,
            cosine_similarity,
            jaccard_similarity,
        );

        TextSimilarityResult {
            overall_similarity,
            overall_level: SimilarityLevel::from_score(overall_similarity),
            cosine_similarity,
            jaccard_similarity,
            edit_distance_similarity,
            keyword_similarity,
            matched_paragraphs,
            matched_sentences,
            source_keywords,
            target_keywords,
            common_keywords,
            confidence,
        }
    }

    /// 分词
    fn tokenize(&self, text: &str) -> Vec<String> {
        if self.use_segmentation {
            // 使用结巴分词
            let jieba = jieba_rs::Jieba::new();
            jieba.cut(text, false)
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

    /// Jaccard 相似度
    fn jaccard_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        let set1: HashSet<_> = tokens1.iter().collect();
        let set2: HashSet<_> = tokens2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f64 / union as f64
    }

    /// 归一化编辑距离相似度
    fn normalized_edit_distance(&self, text1: &str, text2: &str) -> f64 {
        // 对于长文本，采样比较以提高性能
        let (s1, s2) = if text1.len() > 10000 || text2.len() > 10000 {
            // 采样前10000个字符
            let t1: String = text1.chars().take(10000).collect();
            let t2: String = text2.chars().take(10000).collect();
            (t1, t2)
        } else {
            (text1.to_string(), text2.to_string())
        };

        let max_len = s1.len().max(s2.len());
        if max_len == 0 {
            return 1.0;
        }

        // 使用 strsim 的归一化编辑距离
        strsim::normalized_levenshtein(&s1, &s2)
    }

    /// 词频统计
    fn term_frequency(&self, tokens: &[String]) -> HashMap<String, usize> {
        let mut freq = HashMap::new();
        for token in tokens {
            *freq.entry(token.clone()).or_insert(0) += 1;
        }
        freq
    }

    /// 提取关键词
    fn extract_keywords(&self, tokens: &[String]) -> Vec<KeywordInfo> {
        let freq = self.term_frequency(tokens);
        let total_tokens = tokens.len() as f64;
        
        // 停用词列表（简化版）
        let stopwords: HashSet<&str> = [
            "的", "了", "是", "在", "我", "有", "和", "就", "不", "人", "都", "一", "一个",
            "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好",
            "自己", "这", "the", "a", "an", "is", "are", "was", "were", "be", "been",
            "being", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "can", "to", "of", "in", "for", "on", "with",
            "at", "by", "from", "or", "and", "but", "if", "then", "else", "when", "up",
            "down", "out", "as", "so", "it", "its", "this", "that", "these", "those",
        ].iter().cloned().collect();

        let mut keywords: Vec<KeywordInfo> = freq
            .into_iter()
            .filter(|(word, _)| {
                word.chars().count() > 1 && 
                !stopwords.contains(word.as_str()) &&
                word.chars().any(|c| c.is_alphanumeric())
            })
            .map(|(word, count)| {
                let tf = count as f64 / total_tokens;
                KeywordInfo {
                    word,
                    frequency: count,
                    tfidf_score: tf, // 简化的TF-IDF（没有IDF部分）
                }
            })
            .collect();

        // 按频率排序并取前N个
        keywords.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        keywords.truncate(self.keyword_count);
        
        keywords
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

    /// 匹配段落
    fn match_paragraphs(
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

    /// 匹配句子
    fn match_sentences(
        &self,
        sentences1: &[Sentence],
        sentences2: &[Sentence],
    ) -> Vec<MatchedSentence> {
        // 使用并行计算提高性能
        let matches: Vec<MatchedSentence> = sentences1
            .par_iter()
            .flat_map(|s1| {
                sentences2
                    .iter()
                    .filter_map(|s2| {
                        let similarity = self.text_similarity(&s1.text, &s2.text);
                        
                        if similarity >= self.sentence_threshold {
                            Some(MatchedSentence {
                                source_index: s1.index,
                                source_text: s1.text.clone(),
                                source_paragraph: s1.paragraph_index,
                                target_index: s2.index,
                                target_text: s2.text.clone(),
                                target_paragraph: s2.paragraph_index,
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

    /// 计算两段文本的相似度
    fn text_similarity(&self, text1: &str, text2: &str) -> f64 {
        // 使用多种方法的加权平均
        let tokens1 = self.tokenize(text1);
        let tokens2 = self.tokenize(text2);

        let cosine = self.cosine_similarity(&tokens1, &tokens2);
        let jaccard = self.jaccard_similarity(&tokens1, &tokens2);
        
        // 对于短文本，使用编辑距离
        let edit_dist = if text1.len() < 500 && text2.len() < 500 {
            strsim::normalized_levenshtein(text1, text2)
        } else {
            // 对于长文本，跳过编辑距离计算
            (cosine + jaccard) / 2.0
        };

        // 加权平均
        cosine * 0.4 + jaccard * 0.3 + edit_dist * 0.3
    }

    /// 计算综合相似度
    fn compute_overall_similarity(
        &self,
        cosine: f64,
        jaccard: f64,
        edit_dist: f64,
        keyword: f64,
    ) -> f64 {
        // 加权平均
        cosine * 0.35 + jaccard * 0.25 + edit_dist * 0.25 + keyword * 0.15
    }

    /// 计算置信度
    fn compute_confidence(
        &self,
        doc1: &ParsedDocument,
        doc2: &ParsedDocument,
        cosine: f64,
        jaccard: f64,
    ) -> f64 {
        // 基于文档长度和相似度指标的一致性计算置信度
        let len_ratio = {
            let len1 = doc1.char_count as f64;
            let len2 = doc2.char_count as f64;
            if len1 == 0.0 || len2 == 0.0 {
                0.0
            } else {
                let ratio = len1.min(len2) / len1.max(len2);
                ratio
            }
        };

        // 相似度指标一致性
        let similarity_consistency = 1.0 - (cosine - jaccard).abs();

        // 综合置信度
        (len_ratio * 0.3 + similarity_consistency * 0.7).clamp(0.0, 1.0)
    }
}

/// 快速计算两个文本的相似度（不返回详细信息）
pub fn quick_similarity(text1: &str, text2: &str) -> f64 {
    let calculator = TextSimilarityCalculator::default();
    let tokens1 = calculator.tokenize(text1);
    let tokens2 = calculator.tokenize(text2);
    
    let cosine = calculator.cosine_similarity(&tokens1, &tokens2);
    let jaccard = calculator.jaccard_similarity(&tokens1, &tokens2);
    
    (cosine + jaccard) / 2.0
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
    fn test_jaccard_similarity() {
        let calculator = TextSimilarityCalculator::default();
        let tokens1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let tokens2 = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        
        let similarity = calculator.jaccard_similarity(&tokens1, &tokens2);
        assert!((similarity - 0.5).abs() < 0.001);
    }
}
