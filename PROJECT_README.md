# ZToolSet V2 - 工具集系统

基于 Tauri 2 + Vue 3 的桌面工具集应用，从 Electron 项目迁移而来。

## 功能特性

### ✅ 已实现功能

1. **许可证认证系统**
   - 基于 RSA 加密的工具级授权管理
   - 本地缓存验证
   - 服务器端验证和设备绑定

2. **动态工具加载系统**
   - 从 `tools` 目录自动加载工具
   - 支持工具热更新
   - 工具元数据管理（manifest.json）

3. **依赖管理系统**
   - 自动检测工具依赖
   - 一键安装 npm 依赖
   - 依赖状态实时显示

4. **前端界面**
   - 工具列表页面（搜索、筛选）
   - 工具详情页面（认证、依赖管理）
   - 响应式设计

5. **本地数据持久化**
   - License 信息存储
   - 应用配置管理

### 🚧 待实现功能

1. **工具更新系统** - 检查和更新工具版本
2. **工具执行引擎** - Node.js/Python 脚本执行
3. **自定义工具视图** - 每个工具的独立 UI 界面

## 技术栈

### 后端 (Rust)
- **Tauri 2.x** - 应用框架
- **tokio** - 异步运行时
- **rsa/sha2** - 加密库
- **reqwest** - HTTP 客户端
- **serde/serde_json** - 序列化
- **directories** - 系统目录管理

### 前端 (Vue 3)
- **Vue 3** - 前端框架（Composition API）
- **Vue Router** - 路由管理
- **Pinia** - 状态管理
- **TypeScript** - 类型安全
- **Tailwind CSS** - 样式框架

## 项目结构

```
toolsetV2/
├── src/                      # 前端源码
│   ├── stores/              # Pinia 状态管理
│   │   ├── auth.ts         # 认证状态
│   │   └── tools.ts        # 工具状态
│   ├── router/             # 路由配置
│   ├── views/              # 页面组件
│   │   ├── Home.vue        # 主页
│   │   └── ToolDetail.vue  # 工具详情
│   ├── components/         # 通用组件
│   └── main.ts            # 入口文件
│
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── services/      # 业务服务
│   │   │   ├── auth.rs   # 认证服务
│   │   │   ├── tools.rs  # 工具管理
│   │   │   ├── dependency.rs  # 依赖管理
│   │   │   └── storage.rs     # 存储服务
│   │   ├── commands/      # Tauri 命令
│   │   │   ├── auth.rs   # 认证相关命令
│   │   │   ├── tools.rs  # 工具相关命令
│   │   │   └── filesystem.rs  # 文件系统命令
│   │   ├── error.rs      # 错误处理
│   │   ├── types.rs      # 类型定义
│   │   └── lib.rs        # 主入口
│   └── Cargo.toml        # Rust 依赖配置
│
└── tools/                  # 工具目录
    ├── example-tool/      # 示例工具
    ├── qrcode-generator/  # 二维码生成器
    └── markdown2pdf/      # Markdown转PDF
```

## 快速开始

### 前置要求

- Node.js >= 20.0.0
- pnpm
- Rust >= 1.70
- 操作系统：Windows、macOS 或 Linux

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# Rust 依赖会在构建时自动安装
```

### 开发

```bash
# 启动开发服务器
pnpm tauri dev
```

### 构建

```bash
# 构建生产版本
pnpm tauri build
```

## 如何创建新工具

### 1. 创建工具目录

在 `tools/` 目录下创建新文件夹，例如 `my-tool/`

### 2. 创建 manifest.json

```json
{
  "id": "my-tool",
  "name": "我的工具",
  "version": "1.0.0",
  "description": "工具描述",
  "author": "作者名",
  "main": "index.js",
  "icon": "🔧",
  "requiredAuth": false,
  "hidden": false,
  "dependencies": {}
}
```

### 3. 创建工具文件

创建 `index.js` 或其他入口文件。

### 4. （可选）创建自定义视图

在 `views/` 目录下创建 `ToolView.vue` 文件。

## License 系统使用说明

### 生成 License

需要配合服务器端的 license 生成工具使用（参考原 Electron 项目的 server 目录）。

### 注册 License

1. 打开需要认证的工具详情页
2. 粘贴 License 代码
3. 点击"注册许可证"按钮

### License 验证流程

1. **本地验证**：使用公钥解密 License
2. **服务器验证**：绑定设备指纹
3. **过期检查**：验证有效期

## 环境变量

创建 `.env` 文件配置：

```env
LICENSE_SERVER_URL=http://localhost:3000
```

## 开发注意事项

1. **工具加载**：工具在应用启动时自动加载
2. **热重载**：可以调用 `reload_tools` 命令重新加载工具
3. **依赖安装**：首次使用有依赖的工具时会自动提示安装
4. **数据存储**：应用数据存储在系统标准位置（使用 `directories` crate）

## 从 Electron 迁移的改动

1. **IPC 通信**：从 `ipcRenderer.invoke` 改为 Tauri 的 `invoke`
2. **文件对话框**：使用 Tauri 的 dialog 插件
3. **存储位置**：使用 Rust 的 `directories` crate 管理
4. **性能提升**：Rust 后端性能显著优于 Node.js
5. **包体积**：应用体积更小，启动更快

## 故障排除

### 工具无法加载
- 检查 `tools` 目录是否存在
- 验证 `manifest.json` 格式是否正确
- 查看控制台日志

### 依赖安装失败
- 确保已安装 npm
- 检查网络连接
- 尝试手动安装依赖

### License 认证失败
- 验证 License 格式是否正确
- 检查服务器连接
- 确认公钥已正确加载

## 参考资料

- [Tauri 文档](https://tauri.app)
- [Vue 3 文档](https://vuejs.org)
- [Pinia 文档](https://pinia.vuejs.org)

## 许可证

MIT

