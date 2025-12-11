// 文档解析模块 - 支持 DOCX, TXT, PDF 格式
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("不支持的文件格式: {0}")]
    UnsupportedFormat(String),
    #[error("文件读取失败: {0}")]
    IoError(#[from] std::io::Error),
    #[error("文件解析失败: {0}")]
    ParseFailed(String),
    #[error("文件损坏或无法读取")]
    CorruptedFile,
    #[error("无法提取文本内容（可能是扫描版PDF）")]
    NoTextContent,
    #[error("文件过大: {0} MB，超过限制 {1} MB")]
    FileTooLarge(u64, u64),
}

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub file_type: String,
    pub page_count: Option<u32>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
}

/// 段落信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub index: usize,
    pub text: String,
    pub page: Option<u32>,
    pub char_count: usize,
}

/// 句子信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub index: usize,
    pub text: String,
    pub paragraph_index: usize,
}

/// 文档中的图片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentImage {
    pub index: usize,
    #[serde(skip_serializing)]
    pub data: Vec<u8>,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub page: Option<u32>,
    pub hash: String,
}

/// 解析后的文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub metadata: DocumentMetadata,
    pub full_text: String,
    pub paragraphs: Vec<Paragraph>,
    pub sentences: Vec<Sentence>,
    pub images: Vec<DocumentImage>,
    pub word_count: usize,
    pub char_count: usize,
}

/// 文档解析器
pub struct DocumentParser {
    max_file_size_mb: u64,
}

impl Default for DocumentParser {
    fn default() -> Self {
        Self {
            max_file_size_mb: 100,
        }
    }
}

impl DocumentParser {
    pub fn new(max_file_size_mb: u64) -> Self {
        Self { max_file_size_mb }
    }

    /// 解析文档
    pub fn parse(&self, file_path: &str) -> Result<ParsedDocument, ParseError> {
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(ParseError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "文件不存在",
            )));
        }

        let file_metadata = fs::metadata(path)?;
        let file_size = file_metadata.len();
        let file_size_mb = file_size / (1024 * 1024);
        
        if file_size_mb > self.max_file_size_mb {
            return Err(ParseError::FileTooLarge(file_size_mb, self.max_file_size_mb));
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "txt" => self.parse_txt(path, file_size),
            "docx" => self.parse_docx(path, file_size),
            "pdf" => self.parse_pdf(path, file_size),
            _ => Err(ParseError::UnsupportedFormat(extension)),
        }
    }

    fn parse_txt(&self, path: &Path, file_size: u64) -> Result<ParsedDocument, ParseError> {
        let content = fs::read_to_string(path)?;
        
        let metadata = self.create_metadata(path, file_size, "txt", None)?;
        let (paragraphs, sentences) = self.split_text(&content);
        
        let word_count = self.count_words(&content);
        let char_count = content.chars().count();

        Ok(ParsedDocument {
            metadata,
            full_text: content,
            paragraphs,
            sentences,
            images: vec![],
            word_count,
            char_count,
        })
    }

    fn parse_docx(&self, path: &Path, file_size: u64) -> Result<ParsedDocument, ParseError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| ParseError::ParseFailed(format!("无法打开DOCX文件: {}", e)))?;

        let mut full_text = String::new();
        let mut images: Vec<DocumentImage> = vec![];

        if let Ok(mut document_xml) = archive.by_name("word/document.xml") {
            let mut xml_content = String::new();
            document_xml.read_to_string(&mut xml_content)?;
            full_text = self.extract_text_from_docx_xml(&xml_content);
        }

        let mut image_index = 0;
        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let name = file.name().to_string();
                if name.starts_with("word/media/") {
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    
                    let format = Path::new(&name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let hash = self.compute_image_hash(&data);
                    let (width, height) = self.get_image_dimensions(&data);
                    
                    images.push(DocumentImage {
                        index: image_index,
                        data,
                        format,
                        width,
                        height,
                        page: None,
                        hash,
                    });
                    image_index += 1;
                }
            }
        }

        let metadata = self.create_metadata(path, file_size, "docx", None)?;
        let (paragraphs, sentences) = self.split_text(&full_text);
        let word_count = self.count_words(&full_text);
        let char_count = full_text.chars().count();

        Ok(ParsedDocument {
            metadata,
            full_text,
            paragraphs,
            sentences,
            images,
            word_count,
            char_count,
        })
    }

    fn extract_text_from_docx_xml(&self, xml: &str) -> String {
        let mut text = String::new();
        let mut in_text = false;
        let mut current_paragraph = String::new();
        
        let parser = xml::reader::EventReader::from_str(xml);
        
        for event in parser {
            match event {
                Ok(xml::reader::XmlEvent::StartElement { name, .. }) => {
                    if name.local_name == "t" {
                        in_text = true;
                    }
                }
                Ok(xml::reader::XmlEvent::EndElement { name }) => {
                    if name.local_name == "t" {
                        in_text = false;
                    } else if name.local_name == "p"
                        && !current_paragraph.is_empty() {
                            text.push_str(&current_paragraph);
                            text.push('\n');
                            current_paragraph.clear();
                    }
                }
                Ok(xml::reader::XmlEvent::Characters(chars)) => {
                    if in_text {
                        current_paragraph.push_str(&chars);
                    }
                }
                _ => {}
            }
        }
        
        if !current_paragraph.is_empty() {
            text.push_str(&current_paragraph);
        }
        
        text.trim().to_string()
    }

    fn parse_pdf(&self, path: &Path, file_size: u64) -> Result<ParsedDocument, ParseError> {
        let text = pdf_extract::extract_text(path)
            .map_err(|e| ParseError::ParseFailed(format!("PDF解析失败: {}", e)))?;

        if text.trim().is_empty() {
            return Err(ParseError::NoTextContent);
        }

        let doc = lopdf::Document::load(path)
            .map_err(|e| ParseError::ParseFailed(format!("无法加载PDF: {}", e)))?;
        
        let page_count = doc.get_pages().len() as u32;
        let images = self.extract_pdf_images(&doc);

        let metadata = self.create_metadata(path, file_size, "pdf", Some(page_count))?;
        let (paragraphs, sentences) = self.split_text(&text);
        let word_count = self.count_words(&text);
        let char_count = text.chars().count();

        Ok(ParsedDocument {
            metadata,
            full_text: text,
            paragraphs,
            sentences,
            images,
            word_count,
            char_count,
        })
    }

    fn extract_pdf_images(&self, _doc: &lopdf::Document) -> Vec<DocumentImage> {
        // PDF 图像提取较为复杂，暂时返回空列表
        // 后续可以通过更专业的库来实现
        Vec::new()
    }

    fn split_text(&self, text: &str) -> (Vec<Paragraph>, Vec<Sentence>) {
        let mut paragraphs = Vec::new();
        let mut sentences = Vec::new();
        let mut sentence_index = 0;

        for para_text in text.split('\n') {
            let trimmed = para_text.trim();
            if trimmed.is_empty() {
                continue;
            }

            paragraphs.push(Paragraph {
                index: paragraphs.len(),
                text: trimmed.to_string(),
                page: None,
                char_count: trimmed.chars().count(),
            });

            let para_sentences = self.split_sentences(trimmed);
            for sent_text in para_sentences {
                if !sent_text.is_empty() {
                    sentences.push(Sentence {
                        index: sentence_index,
                        text: sent_text,
                        paragraph_index: paragraphs.len() - 1,
                    });
                    sentence_index += 1;
                }
            }
        }

        (paragraphs, sentences)
    }

    fn split_sentences(&self, text: &str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();
        
        let sentence_endings = ['。', '！', '？', '.', '!', '?', '；', ';'];
        
        for ch in text.chars() {
            current.push(ch);
            if sentence_endings.contains(&ch) {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
        
        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            sentences.push(trimmed);
        }
        
        sentences
    }

    fn count_words(&self, text: &str) -> usize {
        use unicode_segmentation::UnicodeSegmentation;
        
        let mut count = 0;
        
        // 统计非中文词（英文、数字等）
        for word in text.unicode_words() {
            // 如果词不包含中文字符，则计数
            let has_cjk = word.chars().any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch));
            if !has_cjk {
                count += 1;
            }
        }
        
        // 单独统计中文字符（中文按字计数更合理）
        for ch in text.chars() {
            if ('\u{4e00}'..='\u{9fff}').contains(&ch) {
                count += 1;
            }
        }
        
        count
    }

    fn create_metadata(
        &self,
        path: &Path,
        file_size: u64,
        file_type: &str,
        page_count: Option<u32>,
    ) -> Result<DocumentMetadata, ParseError> {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let file_path = path.to_string_lossy().to_string();
        
        let metadata = fs::metadata(path)?;
        
        let created_time = metadata
            .created()
            .ok()
            .map(|t| {
                let datetime: DateTime<Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            });
        
        let modified_time = metadata
            .modified()
            .ok()
            .map(|t| {
                let datetime: DateTime<Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M:%S").to_string()
            });

        Ok(DocumentMetadata {
            file_name,
            file_path,
            file_size,
            file_type: file_type.to_string(),
            page_count,
            created_time,
            modified_time,
        })
    }

    fn compute_image_hash(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    fn get_image_dimensions(&self, data: &[u8]) -> (Option<u32>, Option<u32>) {
        if let Ok(img) = image::load_from_memory(data) {
            (Some(img.width()), Some(img.height()))
        } else {
            (None, None)
        }
    }
}

/// 检查文件类型是否支持
pub fn is_supported_format(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    
    matches!(extension.as_str(), "txt" | "docx" | "pdf")
}
