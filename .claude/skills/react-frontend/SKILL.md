---
name: react-frontend
description: |
  React 前端和 WebAssembly 开发规范。

  触发场景：
  - 开发 React 组件
  - 使用 hooks 管理状态
  - TypeScript 类型定义
  - Tailwind CSS 样式
  - WebAssembly 模块开发

  触发词：React、前端、组件、hooks、typescript、jsx、tsx、UI、样式、Tailwind、wasm、webassembly
---

# React 前端开发规范

## 核心架构

```
src/
├── App.tsx              # 主组件（状态管理）
├── index.tsx            # 入口
├── index.css            # 全局样式 (Tailwind)
├── hooks/               # 自定义 hooks
│   └── usePlatform.ts
├── services/            # 平台抽象层
│   ├── platform/
│   ├── storage/
│   └── ai/
└── utils/               # 工具函数

components/
├── Editor.tsx           # Markdown 编辑器
├── Preview.tsx          # 预览面板
├── ChatPanel.tsx        # AI 对话
├── Sidebar.tsx          # 文件浏览器
├── Toolbar.tsx          # 工具栏
├── KnowledgeGraph.tsx   # 知识图谱 (D3)
├── QuizPanel.tsx        # 测验系统
├── MindMap.tsx          # 思维导图 (Mermaid)
└── ...
```

## 核心规范

### 状态管理

使用 React Hooks (useState, useEffect, useCallback)：

```typescript
// 状态声明
const [files, setFiles] = useState<MarkdownFile[]>([]);
const [activeFileId, setActiveFileId] = useState<string | null>(null);

// 副作用
useEffect(() => {
  loadFiles();
}, [folderId]);

// 回调
const handleSave = useCallback(async () => {
  await saveFile(activeFileId, content);
}, [activeFileId, content]);
```

### 类型定义

集中在 `types.ts`：

```typescript
interface MarkdownFile {
  id: string;
  name: string;
  content: string;
  path?: string;
  folderId?: string;
  createdAt: string;
  updatedAt: string;
}

interface AIConfig {
  provider: string;
  model: string;
  apiKey: string;
  baseUrl?: string;
  temperature?: number;
  maxTokens?: number;
}
```

### Tailwind CSS 样式

```tsx
<div className="flex w-full h-screen bg-paper-50 dark:bg-cyber-900 
    text-slate-800 dark:text-slate-200 overflow-hidden">
  <Sidebar />
  <div className="flex-1 flex flex-col">
    <Toolbar />
    <Editor />
  </div>
</div>
```

### 组件结构

```tsx
interface ComponentProps {
  data: DataType;
  onAction: (action: string) => void;
  theme?: 'light' | 'dark';
}

export function Component({ data, onAction, theme = 'dark' }: ComponentProps) {
  return (
    <div className="...">
      {/* 内容 */}
    </div>
  );
}
```

## 第三方库使用

| 库 | 用途 | 导入方式 |
|------|------|----------|
| react-markdown | Markdown 渲染 | `import ReactMarkdown from 'react-markdown'` |
| d3 | 知识图谱 | `import * as d3 from 'd3'` |
| mermaid | 思维导图 | `import mermaid from 'mermaid'` |
| pdfjs-dist | PDF 解析 | `import * as pdfjsLib from 'pdfjs-dist'` |

## 禁止事项

- ❌ 禁止在渲染进程使用 Node.js API
- ❌ 禁止直接 require 原生模块 (better-sqlite3 等)
- ❌ 禁止跳过 electronAPI 直接 IPC
- ❌ 避免使用 class 组件（统一使用函数组件 + hooks）

## 参考代码

- `App.tsx` - 主组件（状态管理范例）
- `types.ts` - 类型定义
- `components/Editor.tsx` - 编辑器组件
- `components/Sidebar.tsx` - 文件浏览器

## Memory 组件规范

### 状态同步原则

在编辑 Memory 后，**必须**同步更新本地状态：

```typescript
// ✅ 正确 - 成功后更新本地状态
const handleSaveMemory = async (memory: MemoryItem) => {
  const result = await window.electronAPI.memory.update(data);
  if (result?.success) {
    // 关键：更新预览状态
    setPreviewMemory(prev => prev ? {
      ...prev,
      content: memory.content,
      updatedAt: Date.now()
    } : null);
    showToast('保存成功');
  }
};

// ❌ 错误 - 只显示 toast 不更新状态
const handleSaveMemory = async (memory: MemoryItem) => {
  await window.electronAPI.memory.update(data);
  showToast('保存成功'); // 状态不同步！
};
```

### Memory IPC 调用

```typescript
// 搜索记忆
window.electronAPI.memory.search(query, limit)

// 更新记忆
window.electronAPI.memory.update({ id, content, updatedAt })

// 标星记忆
window.electronAPI.memory.star(memoryId, isStarred)

// 获取所有记忆
window.electronAPI.memory.getAll()
```

### Memory 组件结构

```tsx
// 状态定义
const [previewMemory, setPreviewMemory] = useState<MemoryItem | null>(null);
const [injectedMemories, setInjectedMemories] = useState<any[]>([]);

// 编辑后同步状态
if (previewMemory?.id === memory.id) {
  setPreviewMemory(prev => prev ? { ...prev, content, updatedAt } : null);
}
```

## WebAssembly 开发

当开发 WebAssembly 模块时，使用 rust-analyzer 进行 Rust 代码分析：

### Rust WebAssembly 开发流程

```rust
// src/lib.rs - Rust WebAssembly 模块
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_text(input: &str) -> String {
    // 高性能文本处理
    input.to_uppercase()
}

#[wasm_bindgen]
pub struct Processor {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Processor {
        Processor { data: Vec::new() }
    }

    pub fn process(&mut self, input: &[u8]) {
        // 高性能数据处理
    }
}
```

### React 中加载 WebAssembly

```typescript
// hooks/useWasmModule.ts
import { useState, useEffect, useCallback } from 'react';

interface WasmModule {
  processText: (input: string) => string;
  Processor: new () => WasmProcessor;
}

interface WasmProcessor {
  process: (input: Uint8Array) => void;
}

export function useWasmModule() {
  const [module, setModule] = useState<WasmModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadWasm = async () => {
      try {
        const wasm = await import('../../pkg/my_wasm_module');
        setModule(wasm);
      } catch (e) {
        setError(`Failed to load WASM: ${e}`);
      } finally {
        setLoading(false);
      }
    };
    loadWasm();
  }, []);

  const processText = useCallback((input: string) => {
    if (!module?.processText) throw new Error('WASM not loaded');
    return module.processText(input);
  }, [module]);

  return { module, loading, error, processText };
}
```

### WebAssembly 编译

```bash
# 安装 wasm-pack
cargo install wasm-pack

# 编译为 WebAssembly
wasm-pack build --target web

# 输出到 pkg/ 目录
# - my_wasm_module_bg.wasm
# - my_wasm_module_bg.js
# - my_wasm_module.d.ts
```

## rust-analyzer 集成

开发 Rust WebAssembly 时，使用 rust-analyzer 获取：
- 准确的类型推断
- borrow checker 指导
- 自动实现 trait
- 代码补全和导航

## 检查清单

- [ ] 是否使用函数组件 + hooks
- [ ] 是否正确使用 TypeScript 类型
- [ ] 是否使用 Tailwind CSS 样式
- [ ] 是否通过 electronAPI 调用主进程功能
- [ ] 是否处理了 loading/error 状态
- [ ] Memory 操作后是否同步更新本地状态
- [ ] WebAssembly 代码是否使用 rust-analyzer
