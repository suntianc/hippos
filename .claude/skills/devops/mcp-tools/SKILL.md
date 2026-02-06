---
name: mcp-tools
description: |
  MCP (Model Context Protocol) 工具协议开发规范。

  触发场景：
  - 配置 MCP 服务器
  - 使用 MCP 工具
  - 开发自定义 MCP 客户端

  触发词：MCP、工具、protocol、server、浏览器、文件系统、GitHub
---

# MCP 工具协议开发规范

## 技术架构

```
electron/mcp/                  # MCP 管理器
├── index.ts                   # 管理器入口
├── handlers.ts                # IPC 处理器
├── MCPClient.ts               # 单个服务器客户端
└── types.ts                   # 类型定义

src/services/mcpService.ts     # 渲染进程服务
```

## MCP 通信流程

```
渲染进程 (React)
    ↓
src/services/mcpService.ts (封装 IPC 调用)
    ↓ IPC
electron/mcp/handlers.ts (IPC 处理器)
    ↓
electron/mcp/index.ts (MCP 管理器)
    ↓
electron/mcp/MCPClient.ts (单个服务器客户端)
    ↓ JSON-RPC over stdio
MCP 服务器进程 (外部进程)
```

## 配置格式

```json
{
  "mcpServers": {
    "server-name": {
      "command": "启动命令",
      "args": ["参数1", "参数2"],
      "env": {
        "环境变量": "值"
      },
      "cwd": "工作目录（可选）"
    }
  }
}
```

## 官方 MCP 服务器

| 服务器 | 命令 | 用途 |
|--------|------|------|
| chrome-devtools | `npx -y @modelcontextprotocol/server-chrome-devtools` | 浏览器控制 |
| filesystem | `npx -y @modelcontextprotocol/server-filesystem /path` | 文件管理 |
| github | `npx -y @modelcontextprotocol/server-github` | GitHub API |
| git | `npx -y @modelcontextprotocol/server-git` | Git 操作 |

## 使用示例

### 配置 MCP 服务器

```typescript
// 在 AI 配置中设置 MCP Tools
const mcpConfig = {
  mcpServers: {
    "chrome-devtools": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-chrome-devtools"]
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "C:\\Users\\Documents"]
    }
  }
};
```

### 调用 MCP 工具

```typescript
// 渲染进程通过 mcpService 调用
import { mcpService } from '../src/services/mcpService';

// 获取所有可用工具
const tools = await mcpService.getTools();

// 执行工具
const result = await mcpService.callTool('navigate_page', {
  url: 'https://example.com'
});
```

## MCP 客户端实现

```typescript
// electron/mcp/MCPClient.ts
import { spawn } from 'child_process';

export class MCPClient {
  private process: ChildProcess | null = null;
  private requestId = 0;
  private callbacks = new Map<number, (result: any) => void>();
  
  constructor(private config: MCPServerConfig) {}
  
  async connect(): Promise<void> {
    this.process = spawn(this.config.command, this.config.args, {
      env: { ...process.env, ...this.config.env },
      cwd: this.config.cwd
    });
    
    this.process.stdout?.on('data', (data) => {
      this.handleMessage(JSON.parse(data.toString()));
    });
  }
  
  async callTool(name: string, args: any): Promise<any> {
    const id = ++this.requestId;
    
    return new Promise((resolve, reject) => {
      this.callbacks.set(id, (result) => {
        if (result.error) {
          reject(new Error(result.error));
        } else {
          resolve(result);
        }
      });
      
      this.send({
        jsonrpc: '2.0',
        id,
        method: 'tools/call',
        params: { name, arguments: args }
      });
      
      // 超时处理
      setTimeout(() => {
        this.callbacks.delete(id);
        reject(new Error('工具调用超时'));
      }, 30000);
    });
  }
  
  private send(message: any) {
    this.process?.stdin?.write(JSON.stringify(message) + '\n');
  }
  
  private handleMessage(message: any) {
    if (message.id && this.callbacks.has(message.id)) {
      this.callbacks.get(message.id)?.(message.result);
      this.callbacks.delete(message.id);
    }
  }
}
```

## JSON-RPC 2.0 协议

**请求示例**：
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "navigate_page",
    "arguments": {
      "url": "https://example.com"
    }
  }
}
```

**响应示例**：
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {"type": "text", "text": "Navigated successfully"}
    ]
  }
}
```

## 禁止事项

- ❌ 禁止使用未验证的 MCP 服务器
- ❌ 禁止在配置中硬编码敏感信息
- ❌ 禁止忽略工具调用超时
- ❌ 禁止不处理进程异常退出

## 参考代码

- `electron/mcp/index.ts` - MCP 管理器
- `electron/mcp/MCPClient.ts` - 客户端实现
- `src/services/mcpService.ts` - 渲染进程服务

## 检查清单

- [ ] MCP 服务器配置是否正确
- [ ] 是否处理了进程启动超时
- [ ] 是否处理了工具调用超时
- [ ] 是否处理了进程异常退出
- [ ] 是否正确解析 JSON-RPC 响应
