# FolderSync 工具开发指南

## 重要说明

当前实现中，工具插件需要实现 `ToolPlugin` trait。在实际项目中，有几种方式处理 trait 共享：

### 方案1：共享 Crate（推荐）

将 `ToolPlugin` trait 定义在独立的 crate 中，工具和框架都依赖这个 crate：

```
workspace/
├── toolset-framework/     # 框架
├── toolset-plugin/        # 共享的 trait 定义
│   └── src/
│       └── lib.rs         # ToolPlugin trait
└── tools/
    └── foldersync/        # 工具
```

**toolset-plugin/Cargo.toml:**
```toml
[package]
name = "toolset-plugin"
version = "1.0.0"
edition = "2021"

[dependencies]
serde_json = "1"
```

**foldersync/Cargo.toml:**
```toml
[dependencies]
toolset-plugin = { path = "../../toolset-plugin" }
# ... 其他依赖
```

### 方案2：通过 FFI 接口

如果不想共享代码，可以通过 C FFI 接口：

```rust
// 在框架中定义 C 接口
#[repr(C)]
pub struct CProgressData {
    task_id: *const c_char,
    status: *const c_char,
    // ...
}

extern "C" {
    fn tool_execute(
        method: *const c_char,
        args: *const c_char,
        on_progress: extern "C" fn(*const CProgressData),
    ) -> *mut c_char;
}
```

### 方案3：使用动态分发（当前实现）

当前实现中，工具自己定义了 trait，框架通过动态分发调用。这种方式需要确保 trait 定义完全一致。

## 编译和部署

### 1. 编译工具

```bash
cd tools/foldersync
cargo build --release
```

### 2. 复制动态库

编译后的动态库需要复制到工具目录：

```bash
# macOS
cp target/release/libtool_foldersync.dylib ../foldersync/

# Linux
cp target/release/libtool_foldersync.so ../foldersync/

# Windows
cp target/release/tool_foldersync.dll ../foldersync/
```

### 3. 更新 manifest.json

确保 `manifest.json` 中的 `library` 字段与实际的库名称匹配：

```json
{
  "library": "libtool_foldersync"
}
```

## 调试

### 使用日志

工具可以使用 `log` crate 输出日志：

```rust
use log::{info, error, warn};

info!("任务开始: {}", task_id);
error!("同步失败: {}", error_msg);
```

在框架中配置日志级别：

```rust
env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
```

### 测试

创建测试用例：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sync_task() {
        let tool = FolderSyncTool::new();
        // 测试代码
    }
}
```

运行测试：

```bash
cargo test
```

## 常见问题

### 1. 动态库加载失败

**问题**：框架无法加载工具插件

**解决**：
- 检查库文件是否存在
- 检查库文件格式是否正确（平台匹配）
- 检查 `create_tool_plugin` 函数是否导出

### 2. Trait 不匹配

**问题**：类型转换失败

**解决**：
- 确保 trait 定义完全一致
- 使用方案1（共享 crate）避免此问题

### 3. 内存泄漏

**问题**：工具卸载后内存未释放

**解决**：
- 确保所有资源正确释放
- 使用 `Arc` 和 `Mutex` 管理共享状态
- 实现 `Drop` trait 清理资源

## 性能优化

1. **异步处理**：使用 `tokio` 进行异步文件操作
2. **批量处理**：批量处理文件变化，减少系统调用
3. **缓存**：缓存文件元数据，减少文件系统访问
4. **并发控制**：限制并发任务数量，避免资源耗尽

## 安全考虑

1. **路径验证**：验证所有路径，防止路径遍历攻击
2. **权限检查**：检查文件读写权限
3. **资源限制**：限制文件大小和数量
4. **错误处理**：妥善处理所有错误，避免崩溃

