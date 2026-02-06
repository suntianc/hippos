---
name: electron-main
description: |
  Electron 主进程和系统开发规范。

  触发场景：
  - 开发 IPC 处理器
  - 数据库操作 (SQLite)
  - 向量数据库 (LanceDB)
  - 原生模块集成
  - Go/Rust 语言开发

  触发词：Electron、主进程、IPC、数据库、SQLite、LanceDB、MCP、原生模块、Go、golang、rust、rust-analyzer、cargo
---

# Electron 主进程开发规范

## 核心架构

```
electron/
├── main.ts              # 应用入口
├── preload.ts           # IPC 桥接
├── ipc/                 # IPC 处理器
│   ├── index.ts         # 注册器
│   ├── aiHandlers.ts
│   ├── dbHandlers.ts
│   ├── fileHandlers.ts
│   ├── lancedbHandlers.ts
│   └── ...
├── database/            # SQLite 数据库
│   ├── index.ts         # 初始化
│   ├── schema.sql       # 表结构
│   ├── migrations.ts
│   └── repositories/
├── lancedb/             # 向量数据库
│   └── index.ts
├── mcp/                 # MCP 管理器
│   ├── index.ts
│   ├── handlers.ts
│   └── MCPClient.ts
└── ocr/                 # OCR 服务
    └── index.ts
```

## 核心规范

### IPC 通信模式

渲染进程 → `window.electronAPI.*` → IPC Renderer → IPC Main → 处理函数

```typescript
// preload.ts 中暴露 API
contextBridge.exposeInMainWorld('electronAPI', {
  ai: {
    fetch: (options) => ipcRenderer.invoke('ai:fetch', options),
  },
  db: {
    files: {
      getAll: () => ipcRenderer.invoke('db:files:getAll'),
      create: (file) => ipcRenderer.invoke('db:files:create', file),
    },
  },
  // ... 更多 API
});
```

### 数据库操作

- 使用 `better-sqlite3` 原生模块
- 遵循 `schema.sql` 表结构
- 通过 `repositories/` 封装数据访问

```typescript
// database/index.ts
import Database from 'better-sqlite3';
import path from 'path';

export function initializeDatabase() {
  const dbPath = path.join(app.getPath('userData'), 'zhangnote.db');
  const db = new Database(dbPath);
  db.pragma('foreign_keys = ON');
  db.pragma('journal_mode = WAL');
  return db;
}
```

### ESM 模块规范

Electron 主进程使用 ESM 模块：

```typescript
// electron/main.ts
import { app, BrowserWindow } from 'electron';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
```

### 原生模块处理

打包时需要特殊处理原生模块（better-sqlite3, canvas, onnxruntime）：

```typescript
// 配置 module.paths 用于打包环境
if (app.isPackaged) {
  const resourcesPath = process.resourcesPath;
  const unpackedModules = path.join(resourcesPath, 'app.asar.unpacked', 'node_modules');
  if (!module.paths.includes(unpackedModules)) {
    module.paths.unshift(unpackedModules);
  }
}
```

## 禁止事项

- ❌ 禁止在前端代码直接使用 Node.js API
- ❌ 禁止在渲染进程执行同步 IPC
- ❌ 禁止在渲染进程 require 原生模块
- ❌ 禁止跳过 preload 直接通信

## 参考代码

- `electron/main.ts` - 应用入口
- `electron/ipc/index.ts` - IPC 注册器
- `electron/database/index.ts` - 数据库初始化
- `electron/preload.ts` - IPC 桥接

## Go 语言开发

当开发 Go 代码时，使用 gopls (Go Language Server Protocol):

### gopls 功能

| 功能 | 描述 |
|------|------|
| 代码补全 | 智能导入包和符号补全 |
| 跳转到定义 | 跨包的类型和函数导航 |
| 查找引用 | 查找符号的所有使用位置 |
| 悬停信息 | 显示类型签名和文档 |
| 诊断 | 实时错误和警告检测 |
| 代码重构 | 重命名符号、提取函数 |

### 使用场景

```go
// 开发 TashanStone 相关的 Go 工具
// - CLI 工具
// - 后端服务
// - MCP 服务器实现

func main() {
    // 使用 gopls 获取准确的类型信息
    // 自动导入包
    // 遵循 Go 最佳实践
}
```

## Rust 语言开发

当开发 Rust 代码时，使用 rust-analyzer:

### rust-analyzer 功能

| 功能 | 描述 |
|------|------|
| 代码补全 | 上下文感知补全（含生命周期推断） |
| 类型推断 | 显示推断的类型和生命周期 |
| 跳转到定义/实现 | 导航到定义和 trait 实现 |
| 查找引用 | 查找符号的所有使用位置 |
| 悬停文档 | 显示文档和类型信息 |
| 快速修复 | 自动修复错误、实现 trait |
| Cargo 集成 | 构建和测试集成 |

### 使用场景

```rust
// 开发 TashanStone 扩展
// - CLI 工具
// - WebAssembly 模块
// - VS Code 插件
// - 其他 Electron 替代方案

fn main() {
    // 使用 rust-analyzer 获取准确类型信息
    // 遵循 Rust 最佳实践
    // 利用 borrow checker 指导
}
```

## 检查清单

- [ ] 是否使用 IPC 通信
- [ ] 是否遵循 ESM 规范
- [ ] 是否正确处理异步操作
- [ ] 是否在 preload 中暴露必要 API
- [ ] 是否处理了打包环境路径
- [ ] Go/Rust 代码是否使用对应 LSP
