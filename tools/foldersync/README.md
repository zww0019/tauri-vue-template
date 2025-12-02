# FolderSync 工具 - Rust 版本

单向文件夹实时同步工具，支持多个文件夹同时监控。

## 功能特性

- ✅ 单向同步：将源文件夹内容同步到目标文件夹
- ✅ 自动对齐：初始化阶段自动对齐两个文件夹
- ✅ 增量同步：后续实时监控并同步文件变化
- ✅ 多任务支持：可同时监控多个文件夹
- ✅ 实时日志：每个任务有独立的日志窗口

## 工具结构

```
foldersync/
├── manifest.json          # 工具元数据
├── Cargo.toml            # Rust 依赖配置
├── src/
│   └── lib.rs            # Rust 插件实现
├── views/
│   └── ToolView.vue      # 前端 Vue 组件
└── README.md
```

## 编译

```bash
cd tools/foldersync
cargo build --release
```

编译后的动态库将位于 `target/release/` 目录下：
- macOS: `libtool_foldersync.dylib`
- Linux: `libtool_foldersync.so`
- Windows: `tool_foldersync.dll`

## 使用方法

1. 编译工具插件
2. 将编译后的动态库复制到工具目录
3. 确保 `manifest.json` 配置正确
4. 启动应用，工具会自动加载

## API 方法

### addSyncTask

添加同步任务

**参数：**
- `sourceDir`: 源文件夹路径
- `targetDir`: 目标文件夹路径

**返回：**
```json
{
  "success": true,
  "taskId": "task_1_xxx",
  "message": "同步任务已启动"
}
```

### stopSyncTask

停止同步任务

**参数：**
- `taskId`: 任务ID

**返回：**
```json
{
  "success": true,
  "message": "同步任务已停止"
}
```

### getTaskLogs

获取任务日志

**参数：**
- `taskId`: 任务ID
- `limit`: 日志条数限制（可选，默认1000）

**返回：**
```json
{
  "success": true,
  "logs": [...],
  "total": 100
}
```

## 注意事项

1. 目标文件夹中的文件可能会被覆盖
2. 删除源文件夹中的文件，目标文件夹中对应文件也会被删除
3. 请确保有足够的磁盘空间
4. 同步过程中请勿关闭应用

## 技术实现

- **文件监听**：使用 `notify` crate 实现跨平台文件系统监听
- **异步处理**：使用 `tokio` 异步运行时
- **文件操作**：使用 `tokio::fs` 进行异步文件操作
- **路径处理**：使用标准库 `Path` 和 `PathBuf`

## 开发说明

### 实现 ToolPlugin Trait

工具必须实现 `ToolPlugin` trait，包括以下方法：

- `get_info()`: 返回工具信息
- `execute()`: 执行工具方法
- `stop_task()`: 停止任务
- `get_task_status()`: 获取任务状态
- `get_task_logs()`: 获取任务日志

### 导出函数

必须导出 `create_tool_plugin` 函数供框架动态加载：

```rust
#[no_mangle]
pub extern "C" fn create_tool_plugin() -> *mut dyn ToolPlugin {
    Box::into_raw(Box::new(FolderSyncTool::new()))
}
```

### 依赖管理

工具的依赖在 `Cargo.toml` 中定义，编译时会自动下载和编译。

## 迁移说明

从 Node.js 版本迁移到 Rust 版本的主要变化：

1. **性能提升**：Rust 版本性能更优，内存占用更少
2. **类型安全**：编译时类型检查，减少运行时错误
3. **无需运行时**：不需要 Node.js 运行时环境
4. **更好的并发**：利用 Rust 的异步特性，更好的并发处理
