# 文档相似度检测 - 调试指南

## 问题：前端一直显示"正在分析文档"

### 已修复的问题

1. **参数名不匹配**
   - ✅ 前端使用 `file_path` 而不是 `filePath`
   - ✅ 后端命令参数使用 snake_case

2. **异步执行优化**
   - ✅ `parse_document` 使用 `spawn_blocking` 避免阻塞主线程

### 调试步骤

#### 1. 启动开发环境
```bash
pnpm tauri:dev
```

#### 2. 打开浏览器开发者工具
- 按 `Cmd+Option+I` (Mac) 或 `F12` (Windows/Linux)
- 切换到 Console 标签页

#### 3. 查看日志输出
应该看到类似以下的日志：
```
开始解析文档: /path/to/file.txt
文档解析成功: { metadata: {...}, full_text: "...", ... }
```

#### 4. 如果看到错误
检查错误信息：
- `验证文件格式失败` - 文件格式不支持
- `解析文档失败` - 文件解析出错
- 网络错误 - Tauri 命令调用失败

#### 5. 测试文件
项目根目录下有两个测试文件：
- `test_doc.txt` - 测试文档1
- `test_doc2.txt` - 测试文档2

可以用这两个文件测试相似度检测功能。

### 常见问题

#### Q: 文件一直处于"正在解析..."状态
**A:** 检查：
1. 浏览器控制台是否有错误
2. 文件路径是否正确
3. 文件是否可读
4. 文件格式是否支持（txt/docx/pdf）

#### Q: 选择文件后没有反应
**A:** 检查：
1. Tauri 命令是否正确注册（查看 `src-tauri/src/lib.rs`）
2. 后端是否编译成功（运行 `cargo check`）
3. 前端参数名是否正确（应该是 `file_path`）

#### Q: PDF 文件解析失败
**A:** 
- 确保 PDF 不是扫描版（需要包含文本层）
- PDF 图像提取功能暂时简化，可能不会提取图片

### 手动测试命令

在浏览器控制台中手动测试：

```javascript
// 测试验证文件格式
await window.__TAURI__.core.invoke('validate_file_format', { 
  file_path: '/path/to/test_doc.txt' 
})

// 测试解析文档
await window.__TAURI__.core.invoke('parse_document', { 
  file_path: '/path/to/test_doc.txt' 
})

// 测试对比文档
await window.__TAURI__.core.invoke('compare_documents', { 
  file1: '/path/to/test_doc.txt',
  file2: '/path/to/test_doc2.txt',
  config: null
})
```

### 性能优化

如果文档很大导致解析缓慢：
1. 检查文件大小（默认限制 100MB）
2. 考虑增加超时时间
3. 添加进度反馈

### 下一步

如果问题仍然存在：
1. 检查 Rust 后端日志（终端输出）
2. 使用 `console.log` 追踪前端执行流程
3. 确认 Tauri 版本兼容性
