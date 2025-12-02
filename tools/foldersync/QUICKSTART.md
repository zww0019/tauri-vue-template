# FolderSync 工具快速开始

这是一个完整的 Rust 工具插件示例，展示了如何实现一个文件夹同步工具。

## 文件结构

```
foldersync/
├── Cargo.toml              # Rust 项目配置
├── manifest.json           # 工具元数据（框架使用）
├── src/
│   └── lib.rs             # 工具实现
├── views/
│   └── ToolView.vue       # 前端界面（复用原有）
├── README.md              # 工具说明
├── DEVELOPMENT.md         # 开发指南
└── QUICKSTART.md         # 本文件
```

## 关键实现点

### 1. 实现 ToolPlugin Trait

工具必须实现 `ToolPlugin` trait 的所有方法：

```rust
impl ToolPlugin for FolderSyncTool {
    fn get_info(&self) -> ToolInfo { ... }
    fn execute(&self, method: &str, args: Value, on_progress: ...) -> Result<Value, String> { ... }
    fn stop_task(&self, task_id: &str) -> Result<(), String> { ... }
    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String> { ... }
    fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String> { ... }
}
```

### 2. 导出创建函数

必须导出 `create_tool_plugin` 函数供框架动态加载：

```rust
#[no_mangle]
pub extern "C" fn create_tool_plugin() -> *mut dyn ToolPlugin {
    Box::into_raw(Box::new(FolderSyncTool::new()))
}
```

### 3. 编译配置

`Cargo.toml` 中必须配置为动态库：

```toml
[lib]
name = "tool_foldersync"
crate-type = ["cdylib"]
```

### 4. Manifest 配置

`manifest.json` 中指定库名称：

```json
{
  "library": "libtool_foldersync"
}
```

## 编译步骤

1. **编译工具**
   ```bash
   cd tools/foldersync
   cargo build --release
   ```

2. **复制动态库**
   ```bash
   # 根据平台选择
   cp target/release/libtool_foldersync.dylib .  # macOS
   cp target/release/libtool_foldersync.so .     # Linux
   cp target/release/tool_foldersync.dll .       # Windows
   ```

3. **验证文件**
   ```bash
   ls -la
   # 应该看到：
   # - manifest.json
   # - libtool_foldersync.dylib (或 .so/.dll)
   # - views/ToolView.vue
   ```

## 注意事项

### Trait 共享问题

当前实现中，工具自己定义了 `ToolPlugin` trait。在实际项目中，建议：

1. **创建共享 crate**（推荐）
   - 将 `ToolPlugin` trait 定义在独立的 crate 中
   - 工具和框架都依赖这个 crate
   - 确保类型一致性

2. **使用 FFI 接口**
   - 通过 C FFI 传递数据
   - 更复杂但更灵活

3. **当前方式**
   - 确保 trait 定义完全一致
   - 适合快速原型开发

### 文件监听器管理

`notify::Watcher` 不是 `Clone` 的，需要特殊处理：

- 使用 `Arc<Mutex<Option<Watcher>>>` 包装
- 或者使用 `mpsc::channel` 传递事件
- 当前实现使用了 channel 方式

### 异步运行时

工具使用了 `tokio` 异步运行时：

- 在 `execute` 方法中创建运行时
- 或者使用全局运行时（如果框架提供）
- 注意避免创建多个运行时

## 测试

### 单元测试

```bash
cargo test
```

### 集成测试

1. 编译工具
2. 启动框架应用
3. 在界面中测试工具功能

## 调试技巧

1. **使用日志**
   ```rust
   log::info!("任务开始: {}", task_id);
   ```

2. **打印调试信息**
   ```rust
   eprintln!("调试: {:?}", data);
   ```

3. **使用调试器**
   ```bash
   cargo build --release
   # 在框架中设置断点
   ```

## 下一步

- 查看 `DEVELOPMENT.md` 了解详细开发指南
- 查看 `README.md` 了解工具功能
- 参考其他工具实现更多功能

