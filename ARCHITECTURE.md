# 工具集架构设计文档

## 概述

工具集采用模块化架构，分为两部分：

1. **框架部分（Framework）**：提供核心功能
   - 工具级授权码认证
   - 工具管理（加载、注册、发现）
   - 工具动态加载（支持 Node.js 和 Rust 插件）
   - 工具任务管理（执行、监控、日志）

2. **工具部分（Tools）**：独立实现的功能模块
   - 每个工具独立开发、测试、打包
   - 前端：Vue 3 组件
   - 后端：Rust 插件（推荐）或 Node.js（向后兼容）
   - 支持单独更新和下载

## 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    框架部分 (Framework)                   │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  授权服务    │  │  工具管理    │  │  任务管理    │   │
│  │ AuthService │  │ ToolsManager │  │ TaskManager  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 工具加载器   │  │ 工具执行器    │  │ 工具更新服务  │ │
│  │ ToolLoader   │  │ ToolExecutor  │  │ ToolUpdate    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          │ 动态加载
                          ▼
┌─────────────────────────────────────────────────────────┐
│                    工具部分 (Tools)                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │           工具 A (foldersync)                      │ │
│  │  ┌──────────────┐         ┌──────────────┐       │ │
│  │  │ 前端 Vue组件 │         │ Rust 插件    │       │ │
│  │  │ ToolView.vue │         │ libtool_*.so │       │ │
│  │  └──────────────┘         └──────────────┘       │ │
│  │         │                        │                │ │
│  │         └────────┬───────────────┘                │ │
│  │                  │                                │ │
│  │            manifest.json                          │ │
│  └────────────────────────────────────────────────────┘ │
│                                                          │
│  ┌────────────────────────────────────────────────────┐ │
│  │           工具 B (markdown2pdf)                    │ │
│  │  ┌──────────────┐         ┌──────────────┐       │ │
│  │  │ 前端 Vue组件 │         │ Rust 插件    │       │ │
│  │  │ ToolView.vue │         │ libtool_*.so │       │ │
│  │  └──────────────┘         └──────────────┘       │ │
│  └────────────────────────────────────────────────────┘ │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 工具类型

### Rust 插件工具

**优势：**
- 性能优异
- 类型安全
- 无需外部运行时
- 更好的内存管理
- 可以编译为单个动态库

**结构：**
```
tool-name/
├── manifest.json          # 工具元数据
├── src/
│   └── lib.rs            # Rust 插件实现
├── Cargo.toml            # Rust 依赖配置
├── views/
│   └── ToolView.vue      # 前端 Vue 组件
└── README.md
```

**实现要求：**
- 实现 `ToolPlugin` trait
- 导出 `create_tool_plugin` 函数
- 编译为动态库（cdylib）

## 工具清单 (manifest.json)

### 工具清单示例

```json
{
  "id": "foldersync",
  "name": "文件夹实时同步",
  "version": "1.0.1",
  "description": "单向文件夹实时同步工具",
  "author": "zww",
  "library": "libtool_foldersync",
  "icon": "🔄",
  "requiredAuth": true,
  "hidden": false,
  "frontend": "views/ToolView.vue"
}
```

**字段说明：**
- `id`: 工具唯一标识
- `name`: 工具显示名称
- `version`: 版本号
- `description`: 工具描述（可选）
- `author`: 作者（可选）
- `library`: Rust 插件库名称，默认为 `libtool_{id}`
- `icon`: 图标（emoji 或图标路径）
- `requiredAuth`: 是否需要授权
- `hidden`: 是否在工具列表中隐藏
- `frontend`: 前端组件路径，默认为 `views/ToolView.vue`

## 工具插件接口

所有 Rust 工具必须实现 `ToolPlugin` trait：

```rust
pub trait ToolPlugin: Send + Sync {
    fn get_info(&self) -> ToolInfo;
    fn execute(&self, method: &str, args: Value, on_progress: Box<dyn Fn(ProgressData) + Send>) -> Result<Value, String>;
    fn stop_task(&self, task_id: &str) -> Result<(), String>;
    fn get_task_status(&self, task_id: &str) -> Result<TaskStatus, String>;
    fn get_task_logs(&self, task_id: &str, limit: usize) -> Result<Vec<LogEntry>, String>;
}
```

## 前端组件规范

### 组件位置
- 默认路径：`views/ToolView.vue`
- 可在 manifest.json 中通过 `frontend` 字段自定义

### 组件接口
- 组件通过 Tauri 命令与后端通信
- 使用 `invoke('execute_tool', ...)` 执行工具方法
- 监听 `tool:execution-progress` 事件获取进度更新

### 示例

```vue
<template>
  <div class="tool-view">
    <!-- 工具UI -->
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 执行工具方法
const result = await invoke('execute_tool', {
  toolId: 'foldersync',
  method: 'addSyncTask',
  args: { sourceDir: '/path/to/source' }
})

// 监听进度事件
const unlisten = await listen('tool:execution-progress', (event) => {
  console.log('进度更新:', event.payload)
})
</script>
```

## 工具开发流程

### 创建 Rust 工具

1. **创建工具目录结构**
   ```bash
   mkdir -p tools/my-tool/{src,views}
   ```

2. **创建 manifest.json**
   ```json
   {
     "id": "my-tool",
     "type": "rust",
     ...
   }
   ```

3. **创建 Cargo.toml**
   ```toml
   [package]
   name = "my-tool"
   version = "1.0.0"
   
   [lib]
   crate-type = ["cdylib"]
   
   [dependencies]
   toolset-plugin = { path = "../../src-tauri" }
   ```

4. **实现 ToolPlugin trait**
   ```rust
   use toolset_plugin::{ToolPlugin, ToolInfo, ...};
   
   pub struct MyTool;
   
   impl ToolPlugin for MyTool {
       fn get_info(&self) -> ToolInfo { ... }
       fn execute(...) -> Result<Value, String> { ... }
       // ...
   }
   
   #[no_mangle]
   pub extern "C" fn create_tool_plugin() -> *mut dyn ToolPlugin {
       Box::into_raw(Box::new(MyTool))
   }
   ```

5. **创建前端组件**
   ```vue
   <!-- views/ToolView.vue -->
   ```

6. **编译工具**
   ```bash
   cd tools/my-tool
   cargo build --release
   ```


## 工具更新机制

1. **版本检查**：框架定期检查工具版本
2. **下载更新**：从远程服务器下载新版本
3. **验证校验和**：确保文件完整性
4. **热更新**：支持运行时更新（需重启应用）

## 工具目录结构

```
tools/
├── foldersync/           # 工具A
│   ├── manifest.json
│   ├── src/
│   │   └── lib.rs
│   ├── Cargo.toml
│   ├── views/
│   │   └── ToolView.vue
│   └── target/
│       └── release/
│           └── libtool_foldersync.dylib
│
└── markdown2pdf/         # 工具B
    ├── manifest.json
    ├── src/
    ├── Cargo.toml
    └── views/
```

## 运行时加载流程

1. **扫描工具目录**：查找所有工具的 manifest.json
2. **验证工具**：检查必要文件是否存在
3. **加载插件**：动态加载 Rust 插件库（.so/.dylib/.dll）
4. **注册工具**：将工具添加到工具管理器
5. **加载前端组件**：动态导入 Vue 组件

## 优势

1. **模块化**：框架和工具完全解耦
2. **独立开发**：每个工具可以独立开发、测试、发布
3. **热更新**：工具可以单独更新，无需更新整个应用
4. **类型安全**：Rust 工具提供编译时类型检查
5. **性能优异**：Rust 工具性能接近原生代码
6. **简化架构**：无需 Node.js 运行时，减少依赖和复杂度

