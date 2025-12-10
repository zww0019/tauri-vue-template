// 图像相似度计算模块
use serde::{Deserialize, Serialize};
use image::{DynamicImage, GenericImageView, ImageFormat};
use rayon::prelude::*;
// sha2 可用于未来扩展

use crate::document_parser::DocumentImage;

/// 图像相似度等级
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImageSimilarityLevel {
    Identical,       // 完全相同 (100%)
    HighlySimilar,   // 高度相似 (80-99%)
    PartiallySimilar, // 部分相似 (50-79%)
    SimilarFeatures,  // 相似特征 (30-49%)
    Different,        // 完全不同 (0-29%)
}

impl ImageSimilarityLevel {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 1.0 => ImageSimilarityLevel::Identical,
            s if s >= 0.8 => ImageSimilarityLevel::HighlySimilar,
            s if s >= 0.5 => ImageSimilarityLevel::PartiallySimilar,
            s if s >= 0.3 => ImageSimilarityLevel::SimilarFeatures,
            _ => ImageSimilarityLevel::Different,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ImageSimilarityLevel::Identical => "完全相同",
            ImageSimilarityLevel::HighlySimilar => "高度相似",
            ImageSimilarityLevel::PartiallySimilar => "部分相似",
            ImageSimilarityLevel::SimilarFeatures => "相似特征",
            ImageSimilarityLevel::Different => "完全不同",
        }
    }
}

/// 图像特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFeatures {
    /// 感知哈希值
    pub perceptual_hash: String,
    /// 平均颜色 (R, G, B)
    pub average_color: (u8, u8, u8),
    /// 颜色直方图 (简化为16个bin)
    pub color_histogram: Vec<f64>,
    /// 图像尺寸
    pub width: u32,
    pub height: u32,
    /// 纵横比
    pub aspect_ratio: f64,
}

/// 匹配的图像对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedImage {
    pub source_index: usize,
    pub source_hash: String,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
    pub target_index: usize,
    pub target_hash: String,
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
    pub similarity: f64,
    pub level: ImageSimilarityLevel,
    /// 相似类型说明
    pub match_type: String,
}

/// 图像相似度结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSimilarityResult {
    /// 整体图像相似度
    pub overall_similarity: f64,
    /// 整体相似度等级
    pub overall_level: ImageSimilarityLevel,
    /// 匹配的图像对
    pub matched_images: Vec<MatchedImage>,
    /// 源文档图像数量
    pub source_image_count: usize,
    /// 目标文档图像数量
    pub target_image_count: usize,
    /// 匹配图像数量
    pub matched_count: usize,
    /// 完全相同的图像数量
    pub identical_count: usize,
}

/// 图像相似度计算器
pub struct ImageSimilarityCalculator {
    /// 相似度阈值
    threshold: f64,
    /// 是否使用感知哈希
    use_perceptual_hash: bool,
    /// 是否使用颜色直方图
    use_color_histogram: bool,
}

impl Default for ImageSimilarityCalculator {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            use_perceptual_hash: true,
            use_color_histogram: true,
        }
    }
}

impl ImageSimilarityCalculator {
    pub fn new(threshold: f64, use_perceptual_hash: bool, use_color_histogram: bool) -> Self {
        Self {
            threshold,
            use_perceptual_hash,
            use_color_histogram,
        }
    }

    /// 计算两组图像的相似度
    pub fn calculate(
        &self,
        images1: &[DocumentImage],
        images2: &[DocumentImage],
    ) -> ImageSimilarityResult {
        if images1.is_empty() || images2.is_empty() {
            return ImageSimilarityResult {
                overall_similarity: 0.0,
                overall_level: ImageSimilarityLevel::Different,
                matched_images: vec![],
                source_image_count: images1.len(),
                target_image_count: images2.len(),
                matched_count: 0,
                identical_count: 0,
            };
        }

        // 提取特征
        let features1: Vec<Option<ImageFeatures>> = images1
            .par_iter()
            .map(|img| self.extract_features(img))
            .collect();
        
        let features2: Vec<Option<ImageFeatures>> = images2
            .par_iter()
            .map(|img| self.extract_features(img))
            .collect();

        // 查找匹配
        let mut matched_images = Vec::new();
        let mut identical_count = 0;

        for (i, (img1, feat1_opt)) in images1.iter().zip(features1.iter()).enumerate() {
            for (j, (img2, feat2_opt)) in images2.iter().zip(features2.iter()).enumerate() {
                // 首先检查hash是否相同（快速路径）
                if img1.hash == img2.hash {
                    matched_images.push(MatchedImage {
                        source_index: i,
                        source_hash: img1.hash.clone(),
                        source_width: img1.width,
                        source_height: img1.height,
                        target_index: j,
                        target_hash: img2.hash.clone(),
                        target_width: img2.width,
                        target_height: img2.height,
                        similarity: 1.0,
                        level: ImageSimilarityLevel::Identical,
                        match_type: "完全相同（数据哈希匹配）".to_string(),
                    });
                    identical_count += 1;
                    continue;
                }

                // 如果有特征，计算相似度
                if let (Some(feat1), Some(feat2)) = (feat1_opt, feat2_opt) {
                    let similarity = self.compute_similarity(feat1, feat2);
                    
                    if similarity >= self.threshold {
                        let match_type = self.determine_match_type(feat1, feat2, similarity);
                        
                        matched_images.push(MatchedImage {
                            source_index: i,
                            source_hash: img1.hash.clone(),
                            source_width: img1.width,
                            source_height: img1.height,
                            target_index: j,
                            target_hash: img2.hash.clone(),
                            target_width: img2.width,
                            target_height: img2.height,
                            similarity,
                            level: ImageSimilarityLevel::from_score(similarity),
                            match_type,
                        });
                    }
                }
            }
        }

        // 按相似度降序排序
        matched_images.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // 计算整体相似度
        let overall_similarity = if matched_images.is_empty() {
            0.0
        } else {
            let total_similarity: f64 = matched_images.iter().map(|m| m.similarity).sum();
            let max_possible = images1.len().max(images2.len()) as f64;
            (total_similarity / max_possible).clamp(0.0, 1.0)
        };

        ImageSimilarityResult {
            overall_similarity,
            overall_level: ImageSimilarityLevel::from_score(overall_similarity),
            matched_count: matched_images.len(),
            matched_images,
            source_image_count: images1.len(),
            target_image_count: images2.len(),
            identical_count,
        }
    }

    /// 提取图像特征
    fn extract_features(&self, image: &DocumentImage) -> Option<ImageFeatures> {
        // 尝试从数据加载图像
        let img = self.load_image(&image.data)?;

        let (width, height) = img.dimensions();
        let aspect_ratio = width as f64 / height as f64;

        // 计算感知哈希
        let perceptual_hash = if self.use_perceptual_hash {
            self.compute_perceptual_hash(&img)
        } else {
            String::new()
        };

        // 计算平均颜色和直方图
        let (average_color, color_histogram) = if self.use_color_histogram {
            self.compute_color_features(&img)
        } else {
            ((0, 0, 0), vec![])
        };

        Some(ImageFeatures {
            perceptual_hash,
            average_color,
            color_histogram,
            width,
            height,
            aspect_ratio,
        })
    }

    /// 加载图像
    fn load_image(&self, data: &[u8]) -> Option<DynamicImage> {
        // 尝试自动检测格式
        if let Ok(img) = image::load_from_memory(data) {
            return Some(img);
        }

        // 尝试常见格式
        for format in [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Gif, ImageFormat::Bmp] {
            if let Ok(img) = image::load_from_memory_with_format(data, format) {
                return Some(img);
            }
        }

        None
    }

    /// 计算感知哈希（简化实现：使用缩略图的像素灰度值）
    fn compute_perceptual_hash(&self, img: &DynamicImage) -> String {
        // 缩放到 8x8 并转为灰度
        let small = img.resize_exact(8, 8, image::imageops::FilterType::Lanczos3);
        let gray = small.to_luma8();
        
        // 计算平均亮度
        let pixels: Vec<u8> = gray.pixels().map(|p| p[0]).collect();
        let avg: u64 = pixels.iter().map(|&p| p as u64).sum::<u64>() / 64;
        
        // 生成哈希：每个像素与平均值比较
        let mut hash_bytes = Vec::with_capacity(8);
        for chunk in pixels.chunks(8) {
            let mut byte = 0u8;
            for (i, &pixel) in chunk.iter().enumerate() {
                if pixel as u64 > avg {
                    byte |= 1 << i;
                }
            }
            hash_bytes.push(byte);
        }
        
        // 转为十六进制字符串
        hash_bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// 计算颜色特征
    fn compute_color_features(&self, img: &DynamicImage) -> ((u8, u8, u8), Vec<f64>) {
        let rgb_img = img.to_rgb8();
        let pixels: Vec<_> = rgb_img.pixels().collect();
        let pixel_count = pixels.len() as f64;

        if pixels.is_empty() {
            return ((0, 0, 0), vec![]);
        }

        // 计算平均颜色
        let (sum_r, sum_g, sum_b) = pixels.iter().fold((0u64, 0u64, 0u64), |acc, p| {
            (acc.0 + p[0] as u64, acc.1 + p[1] as u64, acc.2 + p[2] as u64)
        });
        
        let avg_r = (sum_r as f64 / pixel_count) as u8;
        let avg_g = (sum_g as f64 / pixel_count) as u8;
        let avg_b = (sum_b as f64 / pixel_count) as u8;

        // 计算颜色直方图（简化为16个bin）
        let bin_count = 16;
        let mut histogram = vec![0.0; bin_count * 3]; // R, G, B各16个bin

        for pixel in &pixels {
            let r_bin = (pixel[0] as usize * bin_count / 256).min(bin_count - 1);
            let g_bin = (pixel[1] as usize * bin_count / 256).min(bin_count - 1);
            let b_bin = (pixel[2] as usize * bin_count / 256).min(bin_count - 1);

            histogram[r_bin] += 1.0;
            histogram[bin_count + g_bin] += 1.0;
            histogram[bin_count * 2 + b_bin] += 1.0;
        }

        // 归一化
        for val in &mut histogram {
            *val /= pixel_count;
        }

        ((avg_r, avg_g, avg_b), histogram)
    }

    /// 计算两个图像的相似度
    fn compute_similarity(&self, feat1: &ImageFeatures, feat2: &ImageFeatures) -> f64 {
        let mut similarity = 0.0;
        let mut weight_sum = 0.0;

        // 感知哈希相似度
        if self.use_perceptual_hash && !feat1.perceptual_hash.is_empty() && !feat2.perceptual_hash.is_empty() {
            let hash_sim = self.hash_similarity(&feat1.perceptual_hash, &feat2.perceptual_hash);
            similarity += hash_sim * 0.5;
            weight_sum += 0.5;
        }

        // 颜色直方图相似度
        if self.use_color_histogram && !feat1.color_histogram.is_empty() && !feat2.color_histogram.is_empty() {
            let hist_sim = self.histogram_similarity(&feat1.color_histogram, &feat2.color_histogram);
            similarity += hist_sim * 0.3;
            weight_sum += 0.3;
        }

        // 纵横比相似度
        let aspect_sim = 1.0 - (feat1.aspect_ratio - feat2.aspect_ratio).abs().min(1.0);
        similarity += aspect_sim * 0.1;
        weight_sum += 0.1;

        // 平均颜色相似度
        let color_sim = self.color_distance_similarity(feat1.average_color, feat2.average_color);
        similarity += color_sim * 0.1;
        weight_sum += 0.1;

        if weight_sum > 0.0 {
            similarity / weight_sum
        } else {
            0.0
        }
    }

    /// 感知哈希相似度
    fn hash_similarity(&self, hash1: &str, hash2: &str) -> f64 {
        if hash1.is_empty() || hash2.is_empty() || hash1.len() != hash2.len() {
            return 0.0;
        }

        // 解码十六进制并计算汉明距离
        let h1: Vec<u8> = (0..hash1.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(&hash1[i..i+2], 16).ok())
            .collect();
        let h2: Vec<u8> = (0..hash2.len())
            .step_by(2)
            .filter_map(|i| u8::from_str_radix(&hash2[i..i+2], 16).ok())
            .collect();

        if h1.len() != h2.len() || h1.is_empty() {
            return 0.0;
        }

        let total_bits = h1.len() * 8;
        let diff_bits: usize = h1.iter()
            .zip(h2.iter())
            .map(|(a, b)| (a ^ b).count_ones() as usize)
            .sum();

        1.0 - (diff_bits as f64 / total_bits as f64)
    }

    /// 直方图相似度（使用巴氏距离）
    fn histogram_similarity(&self, hist1: &[f64], hist2: &[f64]) -> f64 {
        if hist1.len() != hist2.len() {
            return 0.0;
        }

        let bc: f64 = hist1.iter()
            .zip(hist2.iter())
            .map(|(a, b)| (a * b).sqrt())
            .sum();

        bc
    }

    /// 颜色距离相似度
    fn color_distance_similarity(&self, c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f64 {
        let dr = (c1.0 as f64 - c2.0 as f64) / 255.0;
        let dg = (c1.1 as f64 - c2.1 as f64) / 255.0;
        let db = (c1.2 as f64 - c2.2 as f64) / 255.0;

        let distance = (dr * dr + dg * dg + db * db).sqrt() / 3.0_f64.sqrt();
        1.0 - distance
    }

    /// 确定匹配类型
    fn determine_match_type(&self, feat1: &ImageFeatures, feat2: &ImageFeatures, similarity: f64) -> String {
        if similarity >= 0.99 {
            return "完全相同".to_string();
        }

        let mut reasons = Vec::new();

        // 检查尺寸是否相同
        if feat1.width == feat2.width && feat1.height == feat2.height {
            reasons.push("尺寸相同");
        } else if (feat1.aspect_ratio - feat2.aspect_ratio).abs() < 0.1 {
            reasons.push("纵横比相似");
        }

        // 检查颜色
        let color_sim = self.color_distance_similarity(feat1.average_color, feat2.average_color);
        if color_sim > 0.9 {
            reasons.push("颜色相近");
        }

        if reasons.is_empty() {
            "结构相似".to_string()
        } else {
            reasons.join("，")
        }
    }
}

/// 快速比较两个图像是否相同（基于hash）
pub fn images_identical(img1: &DocumentImage, img2: &DocumentImage) -> bool {
    img1.hash == img2.hash
}

/// 快速计算两个图像的相似度
pub fn quick_image_similarity(data1: &[u8], data2: &[u8]) -> f64 {
    let calculator = ImageSimilarityCalculator::default();
    
    let img1 = DocumentImage {
        index: 0,
        data: data1.to_vec(),
        format: "unknown".to_string(),
        width: None,
        height: None,
        page: None,
        hash: String::new(),
    };
    
    let img2 = DocumentImage {
        index: 0,
        data: data2.to_vec(),
        format: "unknown".to_string(),
        width: None,
        height: None,
        page: None,
        hash: String::new(),
    };

    if let (Some(feat1), Some(feat2)) = (
        calculator.extract_features(&img1),
        calculator.extract_features(&img2),
    ) {
        calculator.compute_similarity(&feat1, &feat2)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_level() {
        assert_eq!(ImageSimilarityLevel::from_score(1.0), ImageSimilarityLevel::Identical);
        assert_eq!(ImageSimilarityLevel::from_score(0.85), ImageSimilarityLevel::HighlySimilar);
        assert_eq!(ImageSimilarityLevel::from_score(0.65), ImageSimilarityLevel::PartiallySimilar);
        assert_eq!(ImageSimilarityLevel::from_score(0.35), ImageSimilarityLevel::SimilarFeatures);
        assert_eq!(ImageSimilarityLevel::from_score(0.1), ImageSimilarityLevel::Different);
    }

    #[test]
    fn test_color_distance() {
        let calculator = ImageSimilarityCalculator::default();
        
        // 相同颜色
        let sim = calculator.color_distance_similarity((100, 100, 100), (100, 100, 100));
        assert!((sim - 1.0).abs() < 0.001);
        
        // 黑白对比
        let sim = calculator.color_distance_similarity((0, 0, 0), (255, 255, 255));
        assert!(sim < 0.1);
    }
}
