---
name: ai-integration
description: |
  AI 服务集成开发规范。

  触发场景：
  - 调用 AI API（Gemini, Ollama, OpenAI）
  - 实现流式响应
  - AI 对话管理
  - MCP 工具调用

  触发词：AI、大模型、Gemini、Ollama、OpenAI、API、对话、生成、流式、token
---

# AI 服务集成开发规范

## 支持的 AI 提供商

| 提供商 | 模型 | 说明 |
|--------|------|------|
| Gemini | gemini-2.5-flash | 默认 |
| Ollama | 本地模型 | 离线运行 |
| OpenAI 兼容 | GPT-4/DeepSeek | API 兼容 |
| Anthropic 兼容 | Claude | API 兼容 |

## 核心架构

```
services/aiService.ts        # AI 服务封装
├── callGemini()             # Gemini API
├── callOllama()             # Ollama 本地
├── callOpenAICompatible()   # OpenAI 兼容 API
└── VirtualMCPClient         # MCP 客户端

src/services/ai/platformFetch.ts  # 跨域调用
src/services/mcpService.ts        # MCP 服务
```

## 核心规范

### AI 调用模式

```typescript
// services/aiService.ts
import { GoogleGenAI } from "@google/genai";

const DEFAULT_MODEL = 'gemini-2.5-flash';

export async function callGemini(config: AIConfig, prompt: string): Promise<string> {
  const client = new GoogleGenAI({ apiKey: config.apiKey });
  
  const result = await client.models.generateContent({
    model: config.model || DEFAULT_MODEL,
    contents: [{ parts: [{ text: prompt }] }]
  });
  
  return result.text || '';
}
```

### 平台感知调用

```typescript
// 使用 platformFetch 避免 CORS 问题
import { platformFetch, platformStreamFetch } from '../src/services/ai/platformFetch';

// 在 Electron 中
const response = await window.electronAPI.ai.fetch({
  provider: 'gemini',
  endpoint: 'https://generativelanguage.googleapis.com/...',
  body: { contents: [...] },
  apiKey: config.apiKey
});
```

### 流式响应

```typescript
async function* streamAIResponse(config: AIConfig, prompt: string) {
  const response = await platformStreamFetch({
    url: config.baseUrl,
    body: {
      model: config.model,
      messages: [{ role: 'user', content: prompt }],
      stream: true
    },
    apiKey: config.apiKey
  });
  
  for await (const chunk of response) {
    yield chunk;
  }
}

// 使用
for await (const chunk of streamAIResponse(config, prompt)) {
  appendToEditor(chunk);
}
```

### 对话管理

```typescript
interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
}

class AIChatManager {
  private messages: ChatMessage[] = [];
  
  addMessage(role: ChatMessage['role'], content: string) {
    this.messages.push({
      id: generateId(),
      role,
      content,
      timestamp: new Date().toISOString()
    });
  }
  
  getContext(maxTokens: number = 8000): ChatMessage[] {
    // 截取最近的上下文
    // ... 实现
  }
  
  clear() {
    this.messages = [];
  }
}
```

## 错误处理

```typescript
try {
  const result = await callAI(config, prompt);
  return result;
} catch (error) {
  if (error.message.includes('401')) {
    throw new Error('API Key 无效');
  }
  if (error.message.includes('429')) {
    throw new Error('请求频率超限，请稍后重试');
  }
  throw new Error(`AI 调用失败: ${error.message}`);
}
```

## 禁止事项

- ❌ 禁止在前端直接调用 AI API（导致 CORS 问题）
- ❌ 禁止在代码中硬编码 API Key
- ❌ 禁止忽略流式响应的错误处理
- ❌ 禁止发送超过上下文限制的内容

## 参考代码

- `services/aiService.ts` - AI 服务实现
- `services/geminiService.ts` - Gemini 专用服务
- `src/services/ai/platformFetch.ts` - 跨域调用封装

## 检查清单

- [ ] 是否使用平台感知调用（避免 CORS）
- [ ] API Key 是否从配置读取
- [ ] 是否处理了流式响应
- [ ] 是否设置了超时和重试
- [ ] 是否处理了常见的 API 错误
