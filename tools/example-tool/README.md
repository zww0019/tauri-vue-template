# 示例工具

这是一个示例工具，用于演示工具集系统的基本功能。

## 功能特性

- 不需要认证
- 无外部依赖
- 简单易用

## 如何创建新工具

1. 在 `tools` 目录下创建新文件夹
2. 创建 `manifest.json` 文件，配置工具信息
3. 创建 `index.js` 文件（如果需要后端逻辑）
4. 创建 `views/ToolView.vue` 文件（如果需要自定义UI）

## manifest.json 说明

```json
{
  "id": "tool-id",                 // 工具唯一标识
  "name": "工具名称",              // 显示名称
  "version": "1.0.0",              // 版本号
  "description": "工具描述",       // 描述
  "author": "作者名",              // 作者
  "main": "index.js",              // 入口文件
  "icon": "🔧",                    // 图标（emoji）
  "requiredAuth": true,            // 是否需要认证
  "hidden": false,                 // 是否隐藏
  "dependencies": {                // npm依赖
    "package-name": "version"
  }
}
```

## 下一步

根据您的需求修改此工具或创建新工具！

