---
name: rag-vectordb
description: |
  RAG 向量数据库开发规范。

  触发场景：
  - 实现知识检索
  - 文本分块处理
  - 向量相似性搜索
  - LanceDB 数据库操作

  触发词：RAG、向量、检索、知识库、embedding、lancedb、chunk、相似性搜索
---

# RAG 向量数据库开发规范

## 技术架构

```
services/ragService.ts       # RAG 服务（前端）
electron/lancedb/            # LanceDB 服务（主进程）
├── index.ts                  # 服务封装
└── ...

ragService.ts 功能：
├── splitTextIntoChunks()     # 文本分块
├── VectorStore class         # 向量存储类
│   ├── indexFile()           # 索引文件
│   ├── search()              # 相似性搜索
│   └── deleteByFile()        # 删除文件
└── cosineSimilarity()        # 余弦相似度计算
```

## 核心规范

### 文本分块策略

```typescript
// 默认配置
const CHUNK_SIZE = 800;      // 分块大小
const CHUNK_OVERLAP = 100;   // 重叠大小
const MAX_CHUNKS = 15;       // 最大检索块数
const MIN_SIMILARITY = 0.3;  // 最小相似度阈值

// 分块逻辑
export const splitTextIntoChunks = (file: MarkdownFile): Chunk[] => {
  const text = file.content;
  const chunks: Chunk[] = [];
  
  // 按标题分割，保留文档结构
  const sections = text.split(/(?=^#{1,3}\s)/m);
  
  sections.forEach(section => {
    if (section.length <= CHUNK_SIZE) {
      if (section.trim()) {
        chunks.push({
          id: `${file.id}-${chunks.length}`,
          fileId: file.id,
          text: section.trim(),
          metadata: { start: 0, end: 0, fileName: file.name }
        });
      }
    } else {
      // 大块需要进一步分割
      for (let i = 0; i < section.length; i += CHUNK_SIZE - CHUNK_OVERLAP) {
        // ... 智能分割逻辑
      }
    }
  });
  
  return chunks;
};
```

### 余弦相似度计算

```typescript
const cosineSimilarity = (vecA: number[], vecB: number[]) => {
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  
  for (let i = 0; i < vecA.length; i++) {
    dotProduct += vecA[i] * vecB[i];
    normA += vecA[i] * vecA[i];
    normB += vecB[i] * vecB[i];
  }
  
  return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
};
```

### 向量存储类

```typescript
class VectorStore {
  private chunks: Chunk[] = [];
  private embeddings: Map<string, number[]> = new Map();
  
  async indexFile(file: MarkdownFile, config: AIConfig) {
    // 1. 分块
    const chunks = splitTextIntoChunks(file);
    
    // 2. 生成向量
    const embeddings = await this.generateEmbeddings(chunks, config);
    
    // 3. 存储
    this.chunks.push(...chunks);
    embeddings.forEach((vec, idx) => {
      this.embeddings.set(chunks[idx].id, vec);
    });
    
    // 4. 如果在 Electron 环境，保存到 LanceDB
    if (window.electronAPI?.lancedb) {
      await window.electronAPI.lancedb.add(vectorChunks);
    }
  }
  
  async search(query: string, config: AIConfig, limit: number = 5) {
    // 1. 生成查询向量
    const queryVector = await this.generateEmbedding(query, config);
    
    // 2. 计算相似度
    const results = this.chunks.map(chunk => ({
      chunk,
      score: cosineSimilarity(queryVector, this.embeddings.get(chunk.id) || [])
    }));
    
    // 3. 排序过滤
    return results
      .filter(r => r.score >= MIN_SIMILARITY)
      .sort((a, b) => b.score - a.score)
      .slice(0, limit);
  }
  
  async deleteByFile(fileId: string) {
    this.chunks = this.chunks.filter(c => c.fileId !== fileId);
    // 同时删除 LanceDB 中的记录
    if (window.electronAPI?.lancedb) {
      await window.electronAPI.lancedb.deleteByFile(fileId);
    }
  }
}
```

### LanceDB API

```typescript
// 初始化
await window.electronAPI.lancedb.init();

// 添加向量
await window.electronAPI.lancedb.add(chunks: VectorChunk[]);

// 搜索
const results = await window.electronAPI.lancedb.search(queryVector, limit);

// 统计
const stats = await window.electronAPI.lancedb.getStats();
// { totalChunks: number, indexedFiles: number, totalFiles: number }
```

## 禁止事项

- ❌ 禁止在没有向量的情况下进行相似性搜索
- ❌ 禁止使用过大的分块大小（超过 1000 字符）
- ❌ 禁止忽略重叠区域导致信息丢失
- ❌ 禁止在主线程进行大量向量计算

## 参考代码

- `services/ragService.ts` - RAG 服务实现
- `electron/lancedb/index.ts` - LanceDB 服务封装
- `types.ts` - Chunk, SearchResult 类型定义

## Memory 持久化服务

### 文件结构

记忆以 Markdown 文件存储在 `%APPDATA%/tashanstone/.memories/`:

```
.memories/
├── _memories_index.json      # 索引文件
├── memory_2025-12-29_主题_abc123.md
└── memory_2025-12-29_开发_def456.md
```

### Memory 文档格式

```markdown
---
id: test-memory-abc123
created: 2025-12-29T06:50:56.904Z
updated: 2025-12-29T12:37:23.935Z
topics: ["开发","工作流","指南"]
importance: medium
source_sessions: []
---

# 记忆标题

记忆正文内容...
```

### IPC 处理器

```typescript
// 注册 Memory IPC 处理器
ipcMain.handle('memory:search', async (_, query, limit) => {
  const memoryService = getMainProcessMemoryService();
  return memoryService.searchMemories(query, limit);
});

ipcMain.handle('memory:update', async (_, data) => {
  // 更新记忆内容
  await memoryService.saveMemory(memoryDoc);
  return { success: true };
});

ipcMain.handle('memory:star', async (_, memoryId, isStarred) => {
  // 更新标星状态
  return { success: true };
});
```

### 自动注入机制

发送消息时自动搜索并注入相关记忆：

```typescript
// 基于用户问题搜索相关记忆
const autoResults = await window.electronAPI.memory.search(userQuery, 5);

// 合并手动注入和自动检索的记忆
const allMemories = [
  ...injectedMemories,                    // 手动注入（优先级高）
  ...autoResults.filter(notDuplicate)     // 自动检索
];

// 格式化并注入到消息
messageContent = formatMemoriesIntoPrompt(allMemories) + userQuery;
```

## 检查清单

- [ ] 是否正确配置分块参数
- [ ] 是否处理了空向量情况
- [ ] 是否设置了相似度阈值
- [ ] 是否正确处理了文件删除
- [ ] 是否在 Electron 环境正确使用 IPC
- [ ] Memory 文件是否包含正确的 YAML frontmatter
- [ ] Memory 索引是否同步更新
