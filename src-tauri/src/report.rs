// 报告生成模块 - 生成PDF和其他格式的报告
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::comparison::{ComparisonResult, BatchComparisonResult};

/// 报告格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    PDF,
    HTML,
    JSON,
    CSV,
}

/// 报告配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub format: String, // "pdf" | "html" | "json" | "csv"
    pub include_details: bool,
    pub include_matched_paragraphs: bool,
    pub include_matched_images: bool,
    pub include_statistics: bool,
    pub title: Option<String>,
    pub author: Option<String>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            format: "pdf".to_string(),
            include_details: true,
            include_matched_paragraphs: true,
            include_matched_images: true,
            include_statistics: true,
            title: None,
            author: None,
        }
    }
}

/// 报告生成器
pub struct ReportGenerator;

impl ReportGenerator {
    /// 生成单次对比报告
    pub fn generate_single_report(
        result: &ComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        match config.format.as_str() {
            "pdf" => Self::generate_pdf_report(result, output_path, config),
            "html" => Self::generate_html_report(result, output_path, config),
            "json" => Self::generate_json_report(result, output_path),
            "csv" => Self::generate_csv_report(result, output_path),
            _ => Err(format!("不支持的报告格式: {}", config.format)),
        }
    }

    /// 生成批量对比报告
    pub fn generate_batch_report(
        result: &BatchComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        match config.format.as_str() {
            "pdf" => Self::generate_batch_pdf_report(result, output_path, config),
            "html" => Self::generate_batch_html_report(result, output_path, config),
            "json" => Self::generate_batch_json_report(result, output_path),
            "csv" => Self::generate_batch_csv_report(result, output_path),
            _ => Err(format!("不支持的报告格式: {}", config.format)),
        }
    }

    /// 生成PDF报告（简化版，使用HTML转PDF的方式）
    fn generate_pdf_report(
        result: &ComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        // 由于printpdf库使用复杂，这里先生成HTML报告
        // 实际项目中可以使用 wkhtmltopdf 或 puppeteer 等工具转换
        let html_content = Self::generate_html_content(result, config);
        
        // 保存为HTML文件（暂时替代PDF）
        let html_path = output_path.replace(".pdf", ".html");
        let file = File::create(&html_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(html_content.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(html_path)
    }

    /// 生成HTML报告
    fn generate_html_report(
        result: &ComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        let html_content = Self::generate_html_content(result, config);
        
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(html_content.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 生成HTML内容
    fn generate_html_content(result: &ComparisonResult, config: &ReportConfig) -> String {
        let title = config.title.clone().unwrap_or_else(|| "文档相似度检测报告".to_string());
        let similarity_color = Self::get_similarity_color(result.overall_similarity);
        
        let mut html = format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); padding: 30px; }}
        h1 {{ color: #1a1a1a; margin-bottom: 10px; }}
        h2 {{ color: #333; margin: 30px 0 15px; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        h3 {{ color: #555; margin: 20px 0 10px; }}
        .header {{ text-align: center; margin-bottom: 30px; padding-bottom: 20px; border-bottom: 2px solid #eee; }}
        .timestamp {{ color: #888; font-size: 14px; }}
        .score-card {{ display: flex; justify-content: center; gap: 30px; margin: 30px 0; flex-wrap: wrap; }}
        .score-item {{ text-align: center; padding: 20px 40px; background: #f8f9fa; border-radius: 10px; min-width: 150px; }}
        .score-value {{ font-size: 36px; font-weight: bold; color: {similarity_color}; }}
        .score-label {{ color: #666; font-size: 14px; margin-top: 5px; }}
        .score-level {{ font-size: 12px; padding: 3px 10px; border-radius: 12px; margin-top: 8px; display: inline-block; background: {similarity_color}; color: white; }}
        .file-info {{ display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0; }}
        .file-card {{ background: #f8f9fa; padding: 15px; border-radius: 8px; }}
        .file-card h4 {{ color: #333; margin-bottom: 10px; }}
        .file-card p {{ color: #666; font-size: 14px; margin: 5px 0; }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin: 20px 0; }}
        .stat-item {{ background: #f8f9fa; padding: 15px; border-radius: 8px; }}
        .stat-value {{ font-size: 24px; font-weight: bold; color: #333; }}
        .stat-label {{ color: #666; font-size: 12px; }}
        .match-list {{ list-style: none; }}
        .match-item {{ background: #f8f9fa; padding: 15px; border-radius: 8px; margin: 10px 0; border-left: 4px solid #4CAF50; }}
        .match-item.high {{ border-left-color: #f44336; }}
        .match-item.medium {{ border-left-color: #ff9800; }}
        .match-item.low {{ border-left-color: #4CAF50; }}
        .match-text {{ color: #555; font-size: 14px; margin: 5px 0; padding: 10px; background: white; border-radius: 4px; }}
        .match-score {{ font-weight: bold; color: #333; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 12px; text-align: left; border-bottom: 1px solid #eee; }}
        th {{ background: #f8f9fa; font-weight: 600; }}
        .footer {{ text-align: center; margin-top: 40px; padding-top: 20px; border-top: 2px solid #eee; color: #888; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{title}</h1>
            <p class="timestamp">生成时间：{}</p>
        </div>
"#, result.timestamp);

        // 相似度得分卡片
        html.push_str(&format!(r#"
        <div class="score-card">
            <div class="score-item">
                <div class="score-value">{:.1}%</div>
                <div class="score-label">综合相似度</div>
                <span class="score-level">{}</span>
            </div>
            <div class="score-item">
                <div class="score-value" style="color: #2196F3;">{:.1}%</div>
                <div class="score-label">文本相似度</div>
            </div>
            <div class="score-item">
                <div class="score-value" style="color: #9C27B0;">{:.1}%</div>
                <div class="score-label">图像相似度</div>
            </div>
        </div>
"#, 
            result.overall_similarity * 100.0,
            result.overall_level,
            result.text_similarity * 100.0,
            result.image_similarity * 100.0,
        ));

        // 文件信息
        html.push_str(&format!(r#"
        <h2>📁 文件信息</h2>
        <div class="file-info">
            <div class="file-card">
                <h4>源文件</h4>
                <p><strong>文件名：</strong>{}</p>
                <p><strong>字数：</strong>{} 词</p>
                <p><strong>字符数：</strong>{}</p>
                <p><strong>图片数：</strong>{}</p>
            </div>
            <div class="file-card">
                <h4>目标文件</h4>
                <p><strong>文件名：</strong>{}</p>
                <p><strong>字数：</strong>{} 词</p>
                <p><strong>字符数：</strong>{}</p>
                <p><strong>图片数：</strong>{}</p>
            </div>
        </div>
"#,
            result.source_file_name,
            result.source_word_count,
            result.source_char_count,
            result.source_image_count,
            result.target_file_name,
            result.target_word_count,
            result.target_char_count,
            result.target_image_count,
        ));

        // 详细指标
        if config.include_statistics {
            html.push_str(&format!(r#"
        <h2>📊 详细指标</h2>
        <div class="stats-grid">
            <div class="stat-item">
                <div class="stat-value">{:.1}%</div>
                <div class="stat-label">余弦相似度</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{:.1}%</div>
                <div class="stat-label">Jaccard相似度</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{:.1}%</div>
                <div class="stat-label">编辑距离相似度</div>
            </div>
            <div class="stat-item">
                <div class="stat-value">{:.1}%</div>
                <div class="stat-label">关键词相似度</div>
            </div>
        </div>
"#,
                result.text_result.cosine_similarity * 100.0,
                result.text_result.jaccard_similarity * 100.0,
                result.text_result.edit_distance_similarity * 100.0,
                result.text_result.keyword_similarity * 100.0,
            ));
        }

        // 相似段落
        if config.include_matched_paragraphs && !result.text_result.matched_paragraphs.is_empty() {
            html.push_str(r#"
        <h2>📝 相似段落</h2>
        <ul class="match-list">
"#);
            for (idx, para) in result.text_result.matched_paragraphs.iter().take(10).enumerate() {
                let level_class = if para.similarity >= 0.8 { "high" } 
                    else if para.similarity >= 0.6 { "medium" } 
                    else { "low" };
                
                html.push_str(&format!(r#"
            <li class="match-item {level_class}">
                <p class="match-score">匹配 #{} - 相似度: {:.1}% ({})</p>
                <div class="match-text"><strong>源文本：</strong>{}</div>
                <div class="match-text"><strong>目标文本：</strong>{}</div>
            </li>
"#,
                    idx + 1,
                    para.similarity * 100.0,
                    para.level.description(),
                    Self::truncate_text(&para.source_text, 200),
                    Self::truncate_text(&para.target_text, 200),
                ));
            }
            html.push_str("        </ul>\n");
            
            if result.text_result.matched_paragraphs.len() > 10 {
                html.push_str(&format!(
                    "        <p style=\"color: #888; font-style: italic;\">... 还有 {} 个相似段落未显示</p>\n",
                    result.text_result.matched_paragraphs.len() - 10
                ));
            }
        }

        // 共同关键词
        if !result.text_result.common_keywords.is_empty() {
            html.push_str(r#"
        <h2>🔑 共同关键词</h2>
        <p style="line-height: 2;">
"#);
            for keyword in &result.text_result.common_keywords {
                html.push_str(&format!(
                    "            <span style=\"display: inline-block; background: #e3f2fd; padding: 4px 12px; border-radius: 15px; margin: 3px; font-size: 14px;\">{}</span>\n",
                    keyword
                ));
            }
            html.push_str("        </p>\n");
        }

        // 页脚
        html.push_str(&format!(r#"
        <div class="footer">
            <p>报告ID: {} | 处理耗时: {}ms | 置信度: {:.1}%</p>
            <p>本报告由文档相似度检测工具自动生成</p>
        </div>
    </div>
</body>
</html>
"#,
            result.id,
            result.processing_time_ms,
            result.confidence * 100.0,
        ));

        html
    }

    /// 生成JSON报告
    fn generate_json_report(result: &ComparisonResult, output_path: &str) -> Result<String, String> {
        let json = serde_json::to_string_pretty(result).map_err(|e| e.to_string())?;
        
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 生成CSV报告
    fn generate_csv_report(result: &ComparisonResult, output_path: &str) -> Result<String, String> {
        let mut csv = String::from("指标,数值\n");
        csv.push_str(&format!("源文件,{}\n", result.source_file_name));
        csv.push_str(&format!("目标文件,{}\n", result.target_file_name));
        csv.push_str(&format!("综合相似度,{:.2}%\n", result.overall_similarity * 100.0));
        csv.push_str(&format!("文本相似度,{:.2}%\n", result.text_similarity * 100.0));
        csv.push_str(&format!("图像相似度,{:.2}%\n", result.image_similarity * 100.0));
        csv.push_str(&format!("余弦相似度,{:.2}%\n", result.text_result.cosine_similarity * 100.0));
        csv.push_str(&format!("Jaccard相似度,{:.2}%\n", result.text_result.jaccard_similarity * 100.0));
        csv.push_str(&format!("相似段落数,{}\n", result.text_result.matched_paragraphs.len()));
        csv.push_str(&format!("相似句子数,{}\n", result.text_result.matched_sentences.len()));
        csv.push_str(&format!("处理时间(ms),{}\n", result.processing_time_ms));
        
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(csv.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 生成批量PDF报告
    fn generate_batch_pdf_report(
        result: &BatchComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        // 同样使用HTML
        let html_path = output_path.replace(".pdf", ".html");
        Self::generate_batch_html_report(result, &html_path, config)
    }

    /// 生成批量HTML报告
    fn generate_batch_html_report(
        result: &BatchComparisonResult,
        output_path: &str,
        config: &ReportConfig,
    ) -> Result<String, String> {
        let title = config.title.clone().unwrap_or_else(|| "批量文档相似度检测报告".to_string());
        
        let mut html = format!(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>{title}</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }}
        .container {{ max-width: 1400px; margin: 0 auto; background: white; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); padding: 30px; }}
        h1 {{ text-align: center; margin-bottom: 10px; }}
        h2 {{ margin: 30px 0 15px; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        .timestamp {{ text-align: center; color: #888; margin-bottom: 30px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin: 20px 0; }}
        .summary-item {{ background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }}
        .summary-value {{ font-size: 28px; font-weight: bold; }}
        .summary-label {{ color: #666; font-size: 12px; }}
        .matrix {{ overflow-x: auto; margin: 20px 0; }}
        .matrix table {{ border-collapse: collapse; font-size: 12px; }}
        .matrix th, .matrix td {{ padding: 8px; text-align: center; border: 1px solid #ddd; min-width: 60px; }}
        .matrix th {{ background: #f8f9fa; }}
        .results-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .results-table th, .results-table td {{ padding: 12px; text-align: left; border-bottom: 1px solid #eee; }}
        .results-table th {{ background: #f8f9fa; }}
        .results-table tr:hover {{ background: #f8f9fa; }}
        .badge {{ padding: 4px 8px; border-radius: 4px; font-size: 12px; color: white; }}
        .badge-high {{ background: #f44336; }}
        .badge-medium {{ background: #ff9800; }}
        .badge-low {{ background: #4CAF50; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>{title}</h1>
        <p class="timestamp">生成时间：{} | 模式：{}</p>
"#, result.timestamp, result.mode);

        // 汇总信息
        html.push_str(&format!(r#"
        <h2>📊 汇总统计</h2>
        <div class="summary">
            <div class="summary-item">
                <div class="summary-value">{}</div>
                <div class="summary-label">文件数量</div>
            </div>
            <div class="summary-item">
                <div class="summary-value">{}</div>
                <div class="summary-label">对比对数</div>
            </div>
            <div class="summary-item">
                <div class="summary-value">{:.1}%</div>
                <div class="summary-label">平均相似度</div>
            </div>
            <div class="summary-item">
                <div class="summary-value">{:.1}%</div>
                <div class="summary-label">最高相似度</div>
            </div>
            <div class="summary-item">
                <div class="summary-value" style="color: #f44336;">{}</div>
                <div class="summary-label">高相似度对</div>
            </div>
            <div class="summary-item">
                <div class="summary-value" style="color: #ff9800;">{}</div>
                <div class="summary-label">中等相似度对</div>
            </div>
        </div>
"#,
            result.file_count,
            result.total_pairs,
            result.average_similarity * 100.0,
            result.max_similarity * 100.0,
            result.high_similarity_pairs,
            result.medium_similarity_pairs,
        ));

        // 相似度矩阵
        if result.similarity_matrix.len() <= 20 {
            html.push_str(r#"
        <h2>🔥 相似度热力图</h2>
        <div class="matrix">
            <table>
                <tr><th></th>
"#);
            for name in &result.file_names {
                html.push_str(&format!("<th>{}</th>", Self::truncate_text(name, 15)));
            }
            html.push_str("</tr>\n");
            
            for (i, row) in result.similarity_matrix.iter().enumerate() {
                html.push_str(&format!("<tr><th>{}</th>", Self::truncate_text(&result.file_names[i], 15)));
                for val in row {
                    let color = Self::get_similarity_color(*val);
                    let bg_opacity = val * 0.3 + 0.1;
                    html.push_str(&format!(
                        "<td style=\"background: rgba({}); color: {}\">{:.0}%</td>",
                        Self::hex_to_rgba(&color, bg_opacity),
                        if *val > 0.5 { "white" } else { "#333" },
                        val * 100.0
                    ));
                }
                html.push_str("</tr>\n");
            }
            html.push_str("            </table>\n        </div>\n");
        }

        // 详细结果表格
        html.push_str(r#"
        <h2>📋 详细结果</h2>
        <table class="results-table">
            <tr>
                <th>序号</th>
                <th>源文件</th>
                <th>目标文件</th>
                <th>综合相似度</th>
                <th>文本相似度</th>
                <th>图像相似度</th>
                <th>等级</th>
            </tr>
"#);

        for (idx, r) in result.results.iter().enumerate() {
            let badge_class = if r.overall_similarity >= 0.7 { "badge-high" }
                else if r.overall_similarity >= 0.5 { "badge-medium" }
                else { "badge-low" };
            
            html.push_str(&format!(r#"
            <tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td><strong>{:.1}%</strong></td>
                <td>{:.1}%</td>
                <td>{:.1}%</td>
                <td><span class="badge {badge_class}">{}</span></td>
            </tr>
"#,
                idx + 1,
                r.source_file_name,
                r.target_file_name,
                r.overall_similarity * 100.0,
                r.text_similarity * 100.0,
                r.image_similarity * 100.0,
                r.overall_level,
            ));
        }
        html.push_str("        </table>\n");

        // 页脚
        html.push_str(&format!(r#"
        <div style="text-align: center; margin-top: 40px; padding-top: 20px; border-top: 2px solid #eee; color: #888; font-size: 12px;">
            <p>报告ID: {} | 总处理耗时: {}ms</p>
            <p>本报告由文档相似度检测工具自动生成</p>
        </div>
    </div>
</body>
</html>
"#, result.id, result.total_processing_time_ms));

        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(html.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 生成批量JSON报告
    fn generate_batch_json_report(result: &BatchComparisonResult, output_path: &str) -> Result<String, String> {
        let json = serde_json::to_string_pretty(result).map_err(|e| e.to_string())?;
        
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 生成批量CSV报告
    fn generate_batch_csv_report(result: &BatchComparisonResult, output_path: &str) -> Result<String, String> {
        let mut csv = String::from("序号,源文件,目标文件,综合相似度,文本相似度,图像相似度,等级\n");
        
        for (idx, r) in result.results.iter().enumerate() {
            csv.push_str(&format!(
                "{},{},{},{:.2}%,{:.2}%,{:.2}%,{}\n",
                idx + 1,
                r.source_file_name,
                r.target_file_name,
                r.overall_similarity * 100.0,
                r.text_similarity * 100.0,
                r.image_similarity * 100.0,
                r.overall_level,
            ));
        }
        
        let file = File::create(output_path).map_err(|e| e.to_string())?;
        let mut writer = BufWriter::new(file);
        writer.write_all(csv.as_bytes()).map_err(|e| e.to_string())?;
        
        Ok(output_path.to_string())
    }

    /// 获取相似度对应的颜色
    fn get_similarity_color(similarity: f64) -> String {
        match similarity {
            s if s >= 0.9 => "#d32f2f".to_string(),
            s if s >= 0.7 => "#f57c00".to_string(),
            s if s >= 0.5 => "#fbc02d".to_string(),
            s if s >= 0.3 => "#388e3c".to_string(),
            _ => "#1976d2".to_string(),
        }
    }

    /// 将hex颜色转换为rgba
    fn hex_to_rgba(hex: &str, alpha: f64) -> String {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            format!("{}, {}, {}, {:.2}", r, g, b, alpha)
        } else {
            format!("128, 128, 128, {:.2}", alpha)
        }
    }

    /// 截断文本
    fn truncate_text(text: &str, max_len: usize) -> String {
        let chars: Vec<char> = text.chars().collect();
        if chars.len() <= max_len {
            text.to_string()
        } else {
            chars[..max_len].iter().collect::<String>() + "..."
        }
    }
}

// ==================== Tauri Commands ====================

/// 生成单次对比报告
#[tauri::command]
pub async fn generate_report(
    result: ComparisonResult,
    output_path: String,
    config: Option<ReportConfig>,
) -> Result<String, String> {
    let config = config.unwrap_or_default();
    ReportGenerator::generate_single_report(&result, &output_path, &config)
}

/// 生成批量对比报告
#[tauri::command]
pub async fn generate_batch_report(
    result: BatchComparisonResult,
    output_path: String,
    config: Option<ReportConfig>,
) -> Result<String, String> {
    let config = config.unwrap_or_default();
    ReportGenerator::generate_batch_report(&result, &output_path, &config)
}
