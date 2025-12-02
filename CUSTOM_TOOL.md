# 自定义工具开发指南

本指南将详细介绍如何为工具集框架开发自定义工具。

## 目录

1. [概述](#概述)
2. [准备工作](#准备工作)
3. [创建工具项目](#创建工具项目)
4. [实现工具逻辑](#实现工具逻辑)
5. [编译和部署](#编译和部署)
6. [测试工具](#测试工具)
7. [常见问题](#常见问题)
8. [最佳实践](#最佳实践)

## 概述

工具集框架采用插件化架构，每个工具都是一个独立的 Rust 动态库。工具通过实现 `ToolPlugin` trait 与框架交互。

### 工具架构

```
工具目录/
├── manifest.json          # 工具元数据
├── Cargo.toml            # Rust 项目配置
├── src/
│   └── lib.rs            # 工具实现
├── views/
│   └── ToolView.vue      # 前端界面（可选）
└── README.md             # 工具说明
```

### 核心概念

- **ToolPlugin Trait**: 所有工具必须实现的接口
- **共享 Crate**: `toolset-plugin` 提供统一的接口定义
- **动态加载**: 框架在运行时动态加载工具插件
- **前端组件**: Vue 3 组件用于工具的用户界面

## 准备工作

### 1. 环境要求

- Rust 1.70+ (推荐使用 rustup 安装)
- Cargo (Rust 包管理器)
- Node.js 和 pnpm (用于前端开发，可选)

### 2. 项目结构

确保你的工具目录位于 `tools/` 目录下：

```
toolsetV2/
├── toolset-plugin/        # 共享的插件接口 crate
├── tools/
│   ├── foldersync/       # 示例工具
│   └── your-tool/        # 你的工具
└── src-tauri/            # 框架代码
```

## 创建工具项目

### 步骤 1: 创建工具目录

```bash
mkdir -p tools/your-tool/{src,views}
cd tools/your-tool
```

### 步骤 2: 创建 Cargo.toml

创建 `Cargo.toml` 文件：

```toml
[package]
name = "your-tool"
version = "1.0.0"
edition = "2021"

[lib]
name = "tool_your_tool"        # 库名称，格式：tool_{工具id}
crate-type = ["cdylib"]         # 必须设置为动态库

[dependencies]
# 工具插件接口 - 使用共享的 toolset-plugin crate
toolset-plugin = { path = "../../toolset-plugin" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
# 根据需求添加其他依赖
```

**重要说明：**
- `name` 字段：库名称，建议格式为 `tool_{工具id}`
- `crate-type`：必须包含 `"cdylib"`，用于生成动态库
- `toolset-plugin`：必须依赖共享的插件接口 crate

### 步骤 3: 创建 manifest.json

创建 `manifest.json` 文件：

```json
{
  "id": "your-tool",
  "name": "你的工具名称",
  "version": "1.0.0",
  "description": "工具描述",
  "author": "你的名字",
  "library": "libtool_your_tool",
  "icon": "🔧",
  "requiredAuth": false,
  "hidden": false,
  "frontend": "views/ToolView.vue"
}
```

**字段说明：**
- `id`: 工具唯一标识符
- `name`: 工具显示名称
- `version`: 版本号
- `library`: Rust 库名称，必须与 Cargo.toml 中的 `name` 字段匹配（加上 `lib` 前缀）
- `icon`: 图标（emoji 或图标路径）
- `requiredAuth`: 是否需要授权
- `hidden`: 是否在工具列表中隐藏
- `frontend`: 前端组件路径（可选，默认为 `views/ToolView.vue`）

### 步骤 4: 创建 lib.rs

创建 `src/lib.rs` 文件，这是工具的核心实现：

```rust
use toolset_plugin::{ToolPlugin, ToolInfo, ProgressData, TaskStatus, LogEntry};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// 你的工具实现
pub struct YourTool {
    // 工具状态
    state: Arc<Mutex<ToolState>>,
}

struct ToolState {
    // 定义工具的内部状态
}

impl YourTool {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ToolState {
                // 初始化状态
            })),
        }
    }
}

impl ToolPlugin for YourTool {
    fn get_info(&self) -> ToolInfo {
        ToolInfo {
            id: "your-tool".to_string(),
            name: "你的工具名称".to_string(),
            version: "1.0.0".to_string(),
            description: "工具描述".to_string(),
            author: "你的名字".to_string(),
            icon: "🔧".to_string(),
        }
    }

    fn execute(
        &self,
        method: &str,
        args: Value,
        on_progress: Box<dyn Fn(ProgressData) + Send>,
    ) -> Result<Value, String> {
        match method {
            "yourMethod" => {
                // 实现你的方法
                Ok(serde_json::json!({
                    "success": true,
                    "message": "执行成功"
                }))
            }
            _ => Err(format!("未知方法: {}", method)),
        }
    }

    fn stop_task(&self, task_id: &str) -> Result<(), String> {
        // 实现停止任务的逻辑
        Ok(())
    }

    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String> {
        // 实现获取任务状态的逻辑
        Ok(TaskStatus {
            task_id: task_id.to_string(),
            status: "completed".to_string(),
            message: None,
            progress: None,
        })
    }

    fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String> {
        // 实现获取任务日志的逻辑
        Ok(vec![])
    }
}

/// 导出创建函数 - 必须导出此函数供框架动态加载
#[no_mangle]
pub extern "C" fn create_tool_plugin() -> *mut dyn ToolPlugin {
    Box::into_raw(Box::new(YourTool::new()))
}
```

## 实现工具逻辑

### ToolPlugin Trait 详解

#### 1. get_info()

返回工具的基本信息：

```rust
fn get_info(&self) -> ToolInfo {
    ToolInfo {
        id: "your-tool".to_string(),
        name: "你的工具".to_string(),
        version: "1.0.0".to_string(),
        description: "工具描述".to_string(),
        author: "作者".to_string(),
        icon: "🔧".to_string(),
    }
}
```

#### 2. execute()

执行工具方法，这是工具的核心功能：

```rust
fn execute(
    &self,
    method: &str,                    // 方法名
    args: Value,                     // JSON 格式的参数
    on_progress: Box<dyn Fn(ProgressData) + Send>,  // 进度回调
) -> Result<Value, String> {
    match method {
        "method1" => {
            // 解析参数
            let param1 = args.get("param1")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "param1 缺失".to_string())?;
            
            // 执行逻辑
            // ...
            
            // 发送进度更新
            on_progress(ProgressData {
                task_id: "task_1".to_string(),
                status: "running".to_string(),
                message: Some("处理中...".to_string()),
                progress: Some(50.0),
                processed_files: None,
                total_files: None,
                processed_size: None,
                total_size: None,
            });
            
            // 返回结果
            Ok(serde_json::json!({
                "success": true,
                "result": "处理完成"
            }))
        }
        _ => Err(format!("未知方法: {}", method)),
    }
}
```

#### 3. stop_task()

停止正在执行的任务：

```rust
fn stop_task(&self, task_id: &str) -> Result<(), String> {
    // 标记任务为停止状态
    // 取消正在执行的操作
    Ok(())
}
```

#### 4. get_task_status()

获取任务状态：

```rust
fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String> {
    Ok(TaskStatus {
        task_id: task_id.to_string(),
        status: "running".to_string(),  // "running", "completed", "failed", "stopped"
        message: Some("处理中...".to_string()),
        progress: Some(75.0),  // 0.0 - 100.0
    })
}
```

#### 5. get_task_logs()

获取任务日志：

```rust
fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String> {
    Ok(vec![
        LogEntry {
            time: "2024-01-01T00:00:00Z".to_string(),
            level: "info".to_string(),  // "info", "success", "error", "warn", "sync"
            message: "日志消息".to_string(),
        }
    ])
}
```

### 异步处理

如果工具需要异步操作，可以使用 `tokio`：

```rust
use tokio::fs;

fn execute(
    &self,
    method: &str,
    args: Value,
    on_progress: Box<dyn Fn(ProgressData) + Send>,
) -> Result<Value, String> {
    match method {
        "asyncMethod" => {
            // 创建运行时
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| format!("创建运行时失败: {}", e))?;
            
            // 执行异步操作
            rt.block_on(async {
                let content = fs::read_to_string("file.txt").await
                    .map_err(|e| format!("读取文件失败: {}", e))?;
                // ...
                Ok(serde_json::json!({"success": true}))
            })
        }
        _ => Err(format!("未知方法: {}", method)),
    }
}
```

### 错误处理

始终返回有意义的错误信息：

```rust
fn execute(...) -> Result<Value, String> {
    let param = args.get("param")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "参数 'param' 缺失".to_string())?;
    
    // 验证参数
    if param.is_empty() {
        return Err("参数 'param' 不能为空".to_string());
    }
    
    // 执行操作
    // ...
}
```

## 编译和部署

### 1. 编译工具

```bash
cd tools/your-tool
cargo build --release
```

编译后的动态库位于 `target/release/` 目录：
- macOS: `libtool_your_tool.dylib`
- Linux: `libtool_your_tool.so`
- Windows: `tool_your_tool.dll`

### 2. 复制动态库

将编译后的动态库复制到工具目录：

```bash
# macOS
cp target/release/libtool_your_tool.dylib .

# Linux
cp target/release/libtool_your_tool.so .

# Windows
cp target/release/tool_your_tool.dll .
```

### 3. 验证文件结构

确保工具目录包含以下文件：

```
your-tool/
├── manifest.json
├── libtool_your_tool.dylib  (或 .so/.dll)
├── views/
│   └── ToolView.vue         (可选)
└── README.md
```

### 4. 测试加载

启动框架应用，工具应该会自动加载。检查日志确认工具加载成功。

## 测试工具

### 单元测试

在 `src/lib.rs` 中添加测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_info() {
        let tool = YourTool::new();
        let info = tool.get_info();
        assert_eq!(info.id, "your-tool");
    }

    #[test]
    fn test_execute() {
        let tool = YourTool::new();
        let args = serde_json::json!({
            "param1": "value1"
        });
        let on_progress = Box::new(|_| {});
        let result = tool.execute("yourMethod", args, on_progress);
        assert!(result.is_ok());
    }
}
```

运行测试：

```bash
cargo test
```

### 集成测试

1. 编译工具
2. 启动框架应用
3. 在界面中测试工具功能
4. 检查日志输出

## 前端组件（可选）

如果需要自定义用户界面，创建 `views/ToolView.vue`：

```vue
<template>
  <div class="your-tool">
    <h2>你的工具</h2>
    <button @click="executeMethod">执行</button>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

async function executeMethod() {
  try {
    const result = await invoke('execute_tool', {
      toolId: 'your-tool',
      method: 'yourMethod',
      args: {
        param1: 'value1'
      }
    })
    console.log('结果:', result)
  } catch (error) {
    console.error('执行失败:', error)
  }
}

// 监听进度事件
const unlisten = await listen('tool:execution-progress', (event) => {
  console.log('进度更新:', event.payload)
})
</script>
```

## 常见问题

### 1. 动态库加载失败

**问题**: 框架无法加载工具插件

**解决方案**:
- 检查库文件是否存在
- 检查库文件格式是否正确（平台匹配）
- 检查 `create_tool_plugin` 函数是否导出
- 检查 `manifest.json` 中的 `library` 字段是否正确

### 2. Trait 不匹配

**问题**: 类型转换失败

**解决方案**:
- 确保使用共享的 `toolset-plugin` crate
- 检查依赖版本是否一致
- 重新编译工具和框架

### 3. 方法未找到

**问题**: 执行方法时返回 "未知方法"

**解决方案**:
- 检查 `execute` 方法中的 `match` 语句
- 确认方法名拼写正确
- 检查前端调用的方法名

### 4. 异步运行时错误

**问题**: tokio 运行时相关错误

**解决方案**:
- 确保在 `execute` 方法中正确创建运行时
- 或者使用框架提供的全局运行时（如果可用）
- 避免创建多个运行时

## 最佳实践

### 1. 代码组织

- 将复杂逻辑拆分为独立函数
- 使用结构体管理工具状态
- 合理使用 `Arc<Mutex<>>` 管理共享状态

### 2. 错误处理

- 始终返回有意义的错误信息
- 使用 `Result` 类型处理错误
- 记录详细的错误日志

### 3. 性能优化

- 使用异步操作处理 I/O
- 批量处理数据，减少系统调用
- 缓存计算结果

### 4. 安全性

- 验证所有输入参数
- 检查文件路径，防止路径遍历
- 限制资源使用（文件大小、数量等）

### 5. 文档

- 为工具编写清晰的 README
- 注释关键代码
- 提供使用示例

## 参考示例

查看 `tools/foldersync/` 目录下的示例工具，了解完整的实现：

- `Cargo.toml`: 项目配置
- `src/lib.rs`: 工具实现
- `manifest.json`: 工具元数据
- `views/ToolView.vue`: 前端组件

## 下一步

1. 阅读 `ARCHITECTURE.md` 了解整体架构
2. 参考 `foldersync` 工具的实现
3. 开始创建你的第一个工具！

## 获取帮助

如果遇到问题：

1. 查看日志输出
2. 检查示例工具的实现
3. 阅读 Rust 和 Tauri 文档
4. 在项目仓库中提交 Issue

祝你开发愉快！🚀

