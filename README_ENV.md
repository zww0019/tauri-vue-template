# 环境变量配置说明

## 设置授权服务器URL

有三种方式可以设置授权服务器URL，优先级从高到低：

### 方式1：环境变量（推荐用于开发环境）

在启动应用前设置环境变量：

**macOS/Linux:**
```bash
export LICENSE_SERVER_URL=https://your-api-server.com
pnpm tauri dev
```

**Windows (PowerShell):**
```powershell
$env:LICENSE_SERVER_URL="https://your-api-server.com"
pnpm tauri dev
```

**Windows (CMD):**
```cmd
set LICENSE_SERVER_URL=https://your-api-server.com
pnpm tauri dev
```

### 方式2：配置文件（推荐用于生产环境）

编辑 `src-tauri/config.json` 文件：

```json
{
  "license_server_url": "https://your-api-server.com"
}
```

配置文件会在构建时打包到应用中。

### 方式3：默认值

如果以上两种方式都未设置，将使用默认值：`https://api.example.com`

## 开发环境快速设置

在 `package.json` 中已经添加了一个便捷脚本：

```bash
pnpm tauri:dev
```

你可以在 `package.json` 中修改 `LICENSE_SERVER_URL` 的值。

