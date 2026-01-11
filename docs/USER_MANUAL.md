# Hippos 用户手册

**Hippos - 高性能上下文管理服务 for AI Agents**

---

## 目录

1. [简介与概述](#1-简介与概述)
2. [快速开始指南](#2-快速开始指南)
3. [核心概念](#3-核心概念)
4. [功能详解](#4-功能详解)
5. [MCP 服务器集成](#5-mcp-服务器集成)
6. [API 参考](#6-api-参考)
7. [最佳实践](#7-最佳实践)
8. [常见问题 (FAQ)](#8-常见问题-faq)
9. [术语表](#9-术语表)

---

## 1. 简介与概述

### 1.1 项目定位

Hippos 是一款基于 Rust 开发的高性能上下文管理服务，专为大语言模型（LLM）和 AI Agent 设计。它的核心使命是解决长对话场景中面临的上下文窗口限制问题，让 AI Agent 能够持久化地管理对话记忆。

### 1.2 核心价值

Hippos 的设计灵感源自人类认知系统的记忆机制。在实际的 AI Agent 应用中，随着对话轮次的不断增加，模型往往会出现"失忆"现象——遗忘早期讨论的重要细节，或者因上下文超出限制而产生"幻觉"，编造出不准确的信息。Hippos 通过构建类似人类认知系统的"工作记忆"与"长期记忆"切换机制，为 AI Agent 提供持久化的对话记忆能力。

**核心价值主张体现在三个维度：**

- **无限会话感知**：通过全量存储机制确保每一轮对话的原始内容都被完整保留，突破模型上下文窗口的物理限制。
- **Token 成本优化**：采用"渐进式披露"策略，仅在 Agent 明确需要时才加载详细历史内容，显著降低平均 Token 消耗。
- **会话隔离安全**：严格的多租户架构确保不同会话之间的数据完全独立，满足企业级应用的安全合规要求。

### 1.3 技术架构

Hippos 采用经典的分层架构设计：

```
┌─────────────────────────────────────────────────────────────┐
│                        接入层 (API Layer)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  MCP Server │  │  REST API   │  │  gRPC Interface     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        服务层 (Service Layer)                │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │  会话管理服务     │  │  检索引擎服务     │  │  索引服务    │  │
│  │  SessionManager │  │  RetrievalSvc   │  │  IndexSvc   │  │
│  └─────────────────┘  └─────────────────┘  └─────────────┘  │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │  脱水处理服务     │  │  权限校验服务     │                   │
│  │  DehydrationSvc │  │  AuthSvc        │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        索引层 (Index Layer)                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              向量索引引擎 (Vector Index)              │    │
│  │    [Session-A Space]  [Session-B Space]  ...        │    │
│  └─────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              全文索引引擎 (Full-Text Index)           │    │
│  │    [Session-A FTS]  [Session-B FTS]  ...            │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        存储层 (Storage Layer)                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              主存储引擎 (SurrealDB/LanceDB)           │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 1.4 使用场景

Hippos 适用于多种 AI Agent 应用场景：

| 场景 | 描述 | 优势 |
|------|------|------|
| **多轮对话系统** | Chatbot、客服系统、虚拟助手 | 持久化对话历史，跨越会话保持上下文 |
| **代码审查助手** | 持续跟踪代码变更和讨论 | 自动摘要和语义搜索，快速定位历史决策 |
| **项目管理协作** | 团队协作讨论和决策记录 | 会话隔离，多租户支持，完整的审计追踪 |
| **研究助手** | 长期研究项目的知识积累 | 向量检索，发现相关历史讨论 |
| **教育辅导** | 个性化学习路径追踪 | 学生级别的会话隔离和学习进度追踪 |

---

## 2. 快速开始指南

### 2.1 系统要求

在开始安装 Hippos 之前，请确保您的系统满足以下要求：

| 组件 | 最低要求 | 推荐配置 |
|------|----------|----------|
| **Rust** | 1.70.0 或更高版本 | 1.75.0+ |
| **操作系统** | Linux / macOS / Windows | Linux (Ubuntu 22.04+) |
| **内存** | 4 GB | 16 GB+ |
| **存储** | 1 GB 可用空间 | 10 GB+ SSD |
| **SurrealDB** | 1.5.6+ (可选，内存模式可用) | 1.5.6+ |
| **Redis** | 可选 (用于限流) | 7.0+ |

### 2.2 安装步骤

#### 2.2.1 从源码编译

```bash
# 1. 克隆仓库
git clone https://github.com/hippos/hippos.git
cd hippos

# 2. 安装 Rust (如果尚未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 编译项目
cargo build --release

# 4. 运行测试
cargo test --lib

# 5. 启动服务
cargo run
```

#### 2.2.2 验证安装

服务启动后，您可以执行以下命令来验证安装是否成功：

```bash
# 1. 检查健康状态
curl http://localhost:8080/health

# 2. 检查版本信息
curl http://localhost:8080/version

# 3. 查看 Prometheus 指标
curl http://localhost:8080/metrics
```

**成功的健康检查响应：**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T12:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": [
    {
      "name": "database",
      "status": "healthy",
      "message": "Connected",
      "latency_ms": 5
    }
  ]
}
```

### 2.3 快速示例

以下是使用 Hippos 的基本流程示例：

```bash
# 1. 创建新会话
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"name": "my-first-session", "description": "Test session"}'

# 响应：
# {"id": "session_abc123", "created_at": "2024-01-15T10:30:00Z"}

# 2. 添加对话轮次
curl -X POST http://localhost:8080/api/v1/sessions/session_abc123/turns \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"role": "user", "content": "Hello, I need help with Rust programming"}'

# 响应：
# {"id": "turn_xyz789", "session_id": "session_abc123", "turn_number": 1, "created_at": "2024-01-15T11:00:00Z"}

# 3. 搜索对话内容
curl "http://localhost:8080/api/v1/sessions/session_abc123/search?q=rust+programming" \
  -H "Authorization: ApiKey dev-api-key"

# 4. 列出所有会话
curl http://localhost:8080/api/v1/sessions \
  -H "Authorization: ApiKey dev-api-key"
```

### 2.4 配置管理

Hippos 支持通过配置文件和环境变量进行配置。默认配置文件为 `config.yaml`：

```yaml
# 应用配置
app:
  name: "hippos"
  environment: "development"

# 数据库配置
database:
  url: "ws://localhost:8000"
  namespace: "hippos"
  database: "sessions"
  username: "root"
  password: "root"
  min_connections: 5
  max_connections: 50

# 向量数据库配置
vector:
  data_dir: "./data/lancedb"
  dimension: 384
  distance_type: "cosine"

# 服务器配置
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

# 安全配置
security:
  api_key: "dev-api-key-change-in-production"
  rate_limit_enabled: false

# 日志配置
logging:
  level: "debug"
  structured: true

# 嵌入模型配置
embedding:
  model_name: "all-MiniLM-L6-v2"
  batch_size: 32
```

**环境变量覆盖：**

```bash
export EXOCORTEX_SERVER_PORT=8080
export EXOCORTEX_DATABASE_URL="ws://localhost:8000"
export EXOCORTEX_API_KEY="your-secret-key"
export EXOCORTEX_LOG_LEVEL="info"
```

---

## 3. 核心概念

### 3.1 Session（会话）

**定义**：会话是 Hippos 中对话管理的基本单元，每个会话代表一个独立的对话上下文。

**会话生命周期：**

```
创建 (Created) → 活跃 (Active) → 归档 (Archived) → 删除 (Deleted)
```

**会话属性：**

| 属性 | 类型 | 描述 |
|------|------|------|
| `id` | String | 会话唯一标识符，格式为 `session_{uuid}` |
| `tenant_id` | String | 租户标识符，用于多租户隔离 |
| `name` | String | 会话名称 |
| `description` | String | 会话描述（可选） |
| `created_at` | DateTime | 创建时间 |
| `last_active_at` | DateTime | 最后活跃时间 |
| `status` | String | 状态：`active`、`archived` |
| `config` | Object | 会话配置 |
| `stats` | Object | 统计信息 |

**会话配置：**

```yaml
config:
  summary_limit: 10      # 保留摘要数量
  max_turns: 100         # 最大轮次数
  semantic_search_enabled: true  # 启用语义搜索
  auto_summarize: false  # 自动生成摘要
```

### 3.2 Turn（对话轮次）

**定义**：轮次是对话中的单个交互单元，包含用户消息或 AI 响应。

**轮次结构：**

```
Turn
├── id: String              # 轮次唯一标识
├── session_id: String      # 所属会话
├── turn_number: u64        # 轮次序号
├── raw_content: String     # 原始内容 (Markdown)
├── metadata: Object        # 元数据
│   ├── timestamp: DateTime
│   ├── user_id: String?
│   ├── message_type: String
│   ├── role: String?
│   ├── model: String?
│   └── token_count: u64?
├── dehydrated: Object?     # 脱水数据
│   ├── gist: String        # 极简概括
│   ├── topics: Vec<String> # 主题列表
│   ├── tags: Vec<String>   # 关键词标签
│   └── generated_at: DateTime
└── status: String          # 状态: pending/indexed/archived
```

**角色类型（Role）：**

| 角色 | 描述 |
|------|------|
| `user` | 用户消息 |
| `assistant` | AI 响应 |
| `system` | 系统消息/提示词 |

### 3.3 Index（索引）

**定义**：索引是脱水处理后的轻量级数据结构，专门为高速检索优化。

**索引记录结构：**

```
IndexRecord
├── turn_id: String      # 关联的对话轮次ID
├── session_id: String   # 会话ID（用于快速过滤）
├── gist: String         # 摘要文本
├── topics: Vec<String>  # 主题列表
├── tags: Vec<String>    # 标签列表
├── timestamp: DateTime  # 时间戳
├── vector_id: String    # 向量标识
└── relevance_score: f32 # 预计算相关性评分（可选）
```

### 3.4 Embedding（向量嵌入）

**定义**：Embedding 是将文本内容转换为高维向量表示的技术，用于语义相似度计算。

**Hippos 默认配置：**

| 参数 | 值 | 描述 |
|------|-----|------|
| 模型 | `all-MiniLM-L6-v2` | HuggingFace 模型标识 |
| 维度 | 384 | 向量维度 |
| 距离度量 | `cosine` | 余弦相似度 |

**Embedding 处理流程：**

```
原始文本 → Tokenization → 模型推理 → 384维向量 → 存储/索引
```

### 3.5 Dehydration（脱水处理）

**定义**：脱水处理是将长对话内容压缩为轻量级摘要的过程。

**脱水输出：**

| 字段 | 长度 | 描述 |
|------|------|------|
| `gist` | 50-100 字 | 极简概括 |
| `topics` | 3-5 个 | 核心讨论主题 |
| `tags` | 5-10 个 | 关键词标签 |

### 3.6 渐进式披露（Progressive Disclosure）

**设计理念**：Hippos 采用"全量存储、渐进索引、按需挂载"的设计哲学。

**披露层级：**

| 层级 | 内容 | 适用场景 |
|------|------|----------|
| **感知阶段** | 索引列表（摘要、主题、时间戳） | 快速浏览、选择 |
| **挂载阶段** | 完整原始内容 | 详细阅读、引用 |

---

## 4. 功能详解

### 4.1 上下文管理

#### 4.1.1 Session CRUD 操作

Hippos 提供完整的会话生命周期管理功能：

**创建会话：**

```bash
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{
    "name": "project-discussion",
    "description": "项目需求讨论",
    "max_turns": 1000,
    "summary_limit": 50,
    "semantic_search_enabled": true,
    "auto_summarize": false
  }'
```

**查询会话列表：**

```bash
# 分页查询
curl "http://localhost:8080/api/v1/sessions?page=1&page_size=20&status=active" \
  -H "Authorization: ApiKey dev-api-key"
```

**更新会话：**

```bash
curl -X PUT http://localhost:8080/api/v1/sessions/session_abc123 \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"name": "updated-name", "status": "active"}'
```

**删除会话：**

```bash
curl -X DELETE http://localhost:8080/api/v1/sessions/session_abc123 \
  -H "Authorization: ApiKey dev-api-key"
```

#### 4.1.2 Turn 管理

**添加轮次：**

```bash
curl -X POST http://localhost:8080/api/v1/sessions/session_abc123/turns \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{
    "role": "user",
    "content": "我们需要讨论微服务架构的设计方案",
    "metadata": {
      "user_id": "user_001",
      "message_type": "user"
    }
  }'
```

**批量添加轮次：**

```bash
# 多次调用 POST /api/v1/sessions/{id}/turns
# 或使用脚本自动化
for i in {1..10}; do
  curl -X POST "http://localhost:8080/api/v1/sessions/session_abc123/turns" \
    -H "Content-Type: application/json" \
    -H "Authorization: ApiKey dev-api-key" \
    -d "{\"role\": \"user\", \"content\": \"对话内容 $i\"}"
done
```

**查询轮次历史：**

```bash
curl "http://localhost:8080/api/v1/sessions/session_abc123/turns?page=1&page_size=50" \
  -H "Authorization: ApiKey dev-api-key"
```

### 4.2 混合搜索引擎

Hippos 提供三种检索模式，满足不同场景需求：

#### 4.2.1 语义搜索（Semantic Search）

基于向量相似度的语义检索，理解查询意图：

```bash
curl -X POST http://localhost:8080/api/v1/sessions/session_abc123/search/semantic \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{
    "query": "微服务架构的最佳实践",
    "limit": 10,
    "threshold": 0.7
  }'
```

**响应：**

```json
{
  "query": "微服务架构的最佳实践",
  "search_type": "semantic",
  "results": [
    {
      "turn_id": "turn_xyz789",
      "gist": "讨论了使用 Rust 改写中间件的方案...",
      "score": 0.89,
      "result_type": "semantic",
      "turn_number": 5,
      "timestamp": "2024-01-15T12:00:00Z",
      "sources": ["vector"]
    }
  ],
  "total_results": 1,
  "took_ms": 25
}
```

#### 4.2.2 关键词搜索（Full-Text Search）

基于关键词的精确匹配检索：

```bash
# 使用混合搜索接口，指定策略为 fulltext
curl "http://localhost:8080/api/v1/sessions/session_abc123/search?q=API设计&strategy=fulltext" \
  -H "Authorization: ApiKey dev-api-key"
```

#### 4.2.3 混合搜索（Hybrid Search）

结合语义和关键词搜索，使用 RRF（Reciprocal Rank Fusion）算法融合结果：

```bash
curl "http://localhost:8080/api/v1/sessions/session_abc123/search?q=微服务设计模式&limit=10" \
  -H "Authorization: ApiKey dev-api-key"
```

**RRF 融合算法：**

```
RRF_Score(d) = Σ (1 / (k + rank_i(d)))
```

其中 `k` 为常数（通常取 60），`rank_i(d)` 为在不同检索结果中的排名。

### 4.3 内容处理

#### 4.3.1 脱水处理

Hippos 自动对对话内容进行脱水处理，生成轻量级摘要：

```rust
// 脱水处理示例
let dehydration_service = create_dehydration_service(
    summary_length: 100,      // 摘要长度
    max_topics: 5,            // 最大主题数
    max_tags: 10              // 最大标签数
);

let dehydrated = dehydration_service.dehydrate("原始对话内容...").await;
// 输出：gist, topics, tags
```

#### 4.3.2 上下文压缩

对于超长对话，Hippos 支持上下文压缩以优化 Token 消耗：

```bash
# 获取压缩后的上下文
curl "http://localhost:8080/api/v1/sessions/session_abc123/context/recent?limit=5" \
  -H "Authorization: ApiKey dev-api-key"
```

### 4.4 安全性

#### 4.4.1 认证机制

Hippos 支持两种认证方式：

**API Key 认证：**

```bash
curl -H "Authorization: ApiKey YOUR_API_KEY" http://localhost:8080/api/v1/sessions
```

**JWT Bearer Token 认证：**

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" http://localhost:8080/api/v1/sessions
```

**JWT Claims 结构：**

```json
{
  "sub": "user_id",
  "tenant_id": "tenant_1",
  "role": "admin",
  "exp": 1705315200,
  "nbf": 1705311600,
  "iss": "hippos",
  "aud": "hippos-api",
  "jti": "uuid"
}
```

#### 4.4.2 角色权限控制（RBAC）

| 角色 | 权限 |
|------|------|
| `admin` | 完全系统管理权限 |
| `tenant_admin` | 租户级别完全访问 |
| `user` | CRUD 自身会话和轮次 |
| `readonly` | 只读访问 |

**权限矩阵：**

| 操作 | Admin | TenantAdmin | User | ReadOnly |
|------|-------|-------------|------|----------|
| 创建会话 | ✓ | ✓ | ✓ | ✗ |
| 读取会话 | ✓ | ✓ (租户内) | ✓ (自身) | ✓ (租户内) |
| 更新会话 | ✓ | ✓ (租户内) | ✓ (自身) | ✗ |
| 删除会话 | ✓ | ✓ (租户内) | ✓ (自身) | ✗ |
| 搜索 | ✓ | ✓ | ✓ | ✓ |

#### 4.4.3 限流保护

Hippos 实现 Token Bucket 算法进行请求限流：

```yaml
security:
  rate_limit_enabled: true
  global_rate_limit: 1000      # 每分钟全局请求数
  per_session_rate_limit: 100  # 每分钟每会话请求数
  redis_url: "redis://localhost:6379"
```

**限流响应头：**

| Header | 描述 |
|--------|------|
| `X-RateLimit-Limit` | 允许的请求数 |
| `X-RateLimit-Remaining` | 剩余请求数 |
| `X-RateLimit-Reset` | 重置时间戳 |
| `Retry-After` | 建议等待时间（秒） |

#### 4.4.4 安全响应头

Hippos 自动为所有 HTTP 响应添加安全头：

```http
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
Content-Security-Policy: default-src 'self'
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

### 4.5 可观测性

#### 4.5.1 健康检查端点

| 端点 | 用途 | 场景 |
|------|------|------|
| `/health` | 完整健康状态 | 综合监控 |
| `/health/live` | 存活检查 | Kubernetes liveness probe |
| `/health/ready` | 就绪检查 | Kubernetes readiness probe |

**健康检查响应示例：**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T12:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": [
    {
      "name": "database",
      "status": "healthy",
      "message": "Connected",
      "latency_ms": 5
    },
    {
      "name": "vector_index",
      "status": "healthy",
      "message": "Ready",
      "latency_ms": 1
    }
  ]
}
```

#### 4.5.2 Prometheus 指标

Hippos 暴露以下 Prometheus 指标：

```
# HTTP 指标
http_requests_total              # 总 HTTP 请求数
http_request_duration_seconds    # 请求延迟分布

# 连接指标
active_connections               # 当前活跃连接数

# 会话指标
sessions_active                  # 活跃会话数
sessions_archived                # 归档会话数

# 轮次指标
turns_total                      # 总轮次数

# 搜索指标
search_requests_total            # 搜索请求总数
search_latency_seconds           # 搜索延迟分布

# 错误指标
errors_total                     # 总错误数
```

**获取指标：**

```bash
curl http://localhost:8080/metrics
```

#### 4.5.3 结构化日志

Hippos 使用 `tracing` 框架实现结构化日志：

```json
{
  "timestamp": "2024-01-15T12:00:00Z",
  "level": "INFO",
  "target": "hippos::api",
  "message": "Creating new session",
  "session_id": "session_abc123",
  "tenant_id": "tenant_1"
}
```

**日志级别：**

| 级别 | 环境默认值 | 使用场景 |
|------|------------|----------|
| `debug` | 开发环境 | 调试信息 |
| `info` | 生产环境 | 一般信息 |
| `warn` | 所有环境 | 警告 |
| `error` | 所有环境 | 错误 |

---

## 5. MCP 服务器集成

### 5.1 什么是 MCP

MCP（Model Context Protocol）是一个标准化协议，允许 AI Agent 与外部工具和服务进行交互。Hippos可以作为 MCP 服务器运行，为 Claude Desktop、Cursor、Cline 等 AI 工具提供上下文管理能力。

### 5.2 启动 MCP 模式

#### 5.2.1 环境变量方式

```bash
# 设置 MCP 模式
export HIPPOS_MCP_MODE=1

# 启动服务
cargo run
```

#### 5.2.2 编译后运行

```bash
# 编译项目
cargo build --release

# MCP 模式运行
HIPPOS_MCP_MODE=1 ./target/release/hippos
```

### 5.3 可用 MCP 工具

#### 5.3.1 hippos_search

执行混合搜索（语义 + 关键词）：

**参数：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `session_id` | string | 是 | - | 会话唯一标识符 |
| `query` | string | 是 | - | 搜索查询文本 |
| `limit` | integer | 否 | 10 | 最大返回结果数 |

**示例：**

```json
{
  "session_id": "session_abc123",
  "query": "Rust 所有权模型的使用技巧",
  "limit": 5
}
```

**响应：**

```json
{
  "results": [
    {
      "turn_id": "turn_xyz789",
      "gist": "讨论了 Rust 的所有权和借用检查器...",
      "score": 0.89,
      "result_type": "hybrid",
      "turn_number": 5,
      "timestamp": "2024-01-15T12:00:00Z",
      "sources": ["vector", "fulltext"]
    }
  ],
  "total_results": 1,
  "took_ms": 25
}
```

#### 5.3.2 hippos_semantic_search

执行纯语义（向量）搜索：

**参数：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `session_id` | string | 是 | - | 会话唯一标识符 |
| `query` | string | 是 | - | 语义搜索查询 |
| `limit` | integer | 否 | 10 | 最大返回结果数 |

### 5.4 客户端集成

#### 5.4.1 Claude Desktop

修改 `claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

#### 5.4.2 Cursor IDE

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

#### 5.4.3 Claude Code CLI

```json
{
  "mcpServers": {
    "hippos": {
      "command": "/full/path/to/hippos",
      "args": [],
      "env": {
        "HIPPOS_MCP_MODE": "1"
      }
    }
  }
}
```

### 5.5 测试 MCP 服务器

使用官方 MCP Inspector 测试：

```bash
# 安装 Inspector
npx @modelcontextprotocol/inspector

# 测试连接
npx @modelcontextprotocol/inspector ./target/release/hippos
```

---

## 6. API 参考

### 6.1 基础信息

| 项目 | 值 |
|------|-----|
| 基础 URL | `http://localhost:8080` |
| API 版本 | `v1` |
| 认证方式 | `ApiKey` 或 `Bearer Token` |
| 默认 API Key | `dev-api-key` |

### 6.2 Sessions API

#### 6.2.1 创建会话

**端点：** `POST /api/v1/sessions`

**请求头：**

```
Authorization: ApiKey dev-api-key
Content-Type: application/json
```

**请求体：**

```json
{
  "name": "session-name",
  "description": "Optional session description",
  "max_turns": 100,
  "summary_limit": 10,
  "semantic_search_enabled": true,
  "auto_summarize": false
}
```

**响应（201 Created）：**

```json
{
  "id": "session_abc123",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### 6.2.2 列出会话

**端点：** `GET /api/v1/sessions`

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `page` | integer | 1 | 页码（从 1 开始） |
| `page_size` | integer | 20 | 每页数量（最大 100） |
| `status` | string | "all" | 状态筛选：`active`、`archived`、`all` |

**响应（200 OK）：**

```json
{
  "sessions": [
    {
      "id": "session_abc123",
      "tenant_id": "tenant_1",
      "name": "my-session",
      "description": "Session description",
      "created_at": "2024-01-15T10:30:00Z",
      "last_active_at": "2024-01-15T11:00:00Z",
      "status": "active",
      "config": {
        "summary_limit": 10,
        "max_turns": 100,
        "semantic_search_enabled": true,
        "auto_summarize": false
      },
      "stats": {
        "total_turns": 5,
        "total_tokens": 1500,
        "storage_size": 8192,
        "last_indexed_at": "2024-01-15T11:00:00Z"
      }
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

#### 6.2.3 获取会话详情

**端点：** `GET /api/v1/sessions/{id}`

**路径参数：**

| 参数 | 类型 | 描述 |
|------|------|------|
| `id` | string | 会话唯一标识符 |

#### 6.2.4 更新会话

**端点：** `PUT /api/v1/sessions/{id}`

**请求体：**

```json
{
  "name": "updated-name",
  "description": "Updated description",
  "max_turns": 200,
  "status": "active"
}
```

#### 6.2.5 删除会话

**端点：** `DELETE /api/v1/sessions/{id}`

### 6.3 Turns API

#### 6.3.1 添加轮次

**端点：** `POST /api/v1/sessions/{session_id}/turns`

**请求体：**

```json
{
  "role": "user",
  "content": "User message content",
  "metadata": {
    "user_id": "user_001",
    "message_type": "user"
  }
}
```

**响应（201 Created）：**

```json
{
  "id": "turn_xyz789",
  "session_id": "session_abc123",
  "turn_number": 1,
  "created_at": "2024-01-15T11:00:00Z"
}
```

#### 6.3.2 列出轮次

**端点：** `GET /api/v1/sessions/{session_id}/turns`

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `page` | integer | 1 | 页码 |
| `page_size` | integer | 50 | 每页数量 |
| `message_type` | string | - | 消息类型筛选 |

#### 6.3.3 获取轮次详情

**端点：** `GET /api/v1/sessions/{session_id}/turns/{turn_id}`

#### 6.3.4 删除轮次

**端点：** `DELETE /api/v1/sessions/{session_id}/turns/{turn_id}`

### 6.4 Search API

#### 6.4.1 混合搜索

**端点：** `GET /api/v1/sessions/{session_id}/search`

**查询参数：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `q` | string | 是 | - | 搜索查询 |
| `limit` | integer | 否 | 10 | 最大结果数 |
| `strategy` | string | 否 | "hybrid" | 搜索策略：`semantic`、`fulltext`、`hybrid` |

#### 6.4.2 语义搜索

**端点：** `POST /api/v1/sessions/{session_id}/search/semantic`

**请求体：**

```json
{
  "query": "What was discussed about Rust programming?",
  "limit": 10,
  "threshold": 0.7
}
```

#### 6.4.3 最近上下文

**端点：** `GET /api/v1/sessions/{session_id}/context/recent`

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `limit` | integer | 10 | 返回最近轮次数 |

### 6.5 Health & Metrics API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/health` | GET | 完整健康检查 |
| `/health/live` | GET | 存活检查 |
| `/health/ready` | GET | 就绪检查 |
| `/metrics` | GET | Prometheus 指标 |
| `/version` | GET | 版本信息 |

### 6.6 错误响应

所有错误返回统一格式：

```json
{
  "code": "ERROR_CODE",
  "message": "Human readable message",
  "details": "string?",
  "request_id": "string?"
}
```

**HTTP 状态码映射：**

| 状态码 | 错误码 | 描述 |
|--------|--------|------|
| 400 | BAD_REQUEST | 参数验证错误 |
| 401 | UNAUTHORIZED | 认证失败 |
| 403 | FORBIDDEN | 权限不足 |
| 404 | NOT_FOUND | 资源不存在 |
| 429 | RATE_LIMITED | 请求过于频繁 |
| 500 | INTERNAL_ERROR | 服务器内部错误 |
| 503 | SERVICE_UNAVAILABLE | 服务不可用 |

---

## 7. 最佳实践

### 7.1 性能优化

#### 7.1.1 连接池配置

根据实际负载调整数据库连接池：

```yaml
database:
  min_connections: 10
  max_connections: 100
  connection_timeout: 30
  idle_timeout: 300
```

#### 7.1.2 批量操作

对于大量数据导入，使用批量操作：

```bash
# 批量创建会话
for i in {1..100}; do
  curl -X POST "http://localhost:8080/api/v1/sessions" \
    -H "Content-Type: application/json" \
    -H "Authorization: ApiKey dev-api-key" \
    -d "{\"name\": \"batch-session-$i\"}" &
done
wait
```

#### 7.1.3 搜索优化

- 使用 `limit` 参数限制返回结果数量
- 启用 `threshold` 过滤低相关性结果
- 定期归档不活跃会话

### 7.2 安全部署

#### 7.2.1 生产环境配置

```yaml
# config.yaml
app:
  environment: "production"

security:
  api_key: "your-secure-api-key-min-32-chars"
  rate_limit_enabled: true
  tls_enabled: true
  tls_cert_path: "/etc/ssl/certs/hippos.crt"
  tls_key_path: "/etc/ssl/private/hippos.key"

logging:
  level: "info"
  structured: true
  log_dir: "/var/log/hippos"
```

#### 7.2.2 API Key 管理

- 使用长随机字符串（32+ 字符）
- 定期轮换 API Key
- 为不同用途使用不同的 Key

#### 7.2.3 JWT 配置

```yaml
security:
  jwt_secret: "your-32-character-secret-key-minimum"
  jwt_issuer: "hippos"
  jwt_audience: "hippos-api"
  jwt_expiry_seconds: 3600  # 1小时
```

### 7.3 监控与告警

#### 7.3.1 关键指标监控

| 指标 | 告警阈值 | 描述 |
|------|----------|------|
| `http_request_duration_seconds` | P99 > 1s | 请求延迟过高 |
| `errors_total` | rate > 0.01 | 错误率异常 |
| `active_connections` | > 1000 | 连接数过高 |
| `search_latency_seconds` | P99 > 500ms | 搜索延迟过高 |

#### 7.3.2 健康检查监控

监控 `/health` 响应中的 `checks` 数组，确保所有依赖服务正常。

### 7.4 故障排查

#### 7.4.1 常见问题

**问题：连接 SurrealDB 失败**

```bash
# 检查 SurrealDB 是否运行
curl http://localhost:8000/status

# 查看错误日志
tail -f /var/log/hippos/error.log
```

**问题：搜索无结果**

```bash
# 检查会话是否存在
curl http://localhost:8080/api/v1/sessions/{session_id} \
  -H "Authorization: ApiKey dev-api-key"

# 检查是否有轮次
curl "http://localhost:8080/api/v1/sessions/{session_id}/turns?page_size=1" \
  -H "Authorization: ApiKey dev-api-key"
```

**问题：认证失败**

```bash
# 验证 API Key
curl -v -H "Authorization: ApiKey dev-api-key" \
  http://localhost:8080/api/v1/sessions

# 检查配置
cat config.yaml | grep api_key
```

#### 7.4.2 日志分析

```bash
# 查看最近错误日志
grep "ERROR" /var/log/hippos/app.log | tail -50

# 搜索特定请求
grep "session_abc123" /var/log/hippos/app.log

# 查看搜索相关日志
grep "search" /var/log/hippos/app.log
```

---

## 8. 常见问题 (FAQ)

### Q1: Hippos 与直接使用数据库存储有什么区别？

**答：** Hippos 提供专门的对话管理功能，包括：
- 语义搜索和混合检索
- 自动摘要和脱水处理
- 会话隔离和权限控制
- MCP 协议集成
- 性能优化的向量索引

### Q2: 支持多租户吗？

**答：** 是的。Hippos 通过三层隔离机制支持多租户：
1. Namespace 隔离（SurrealDB 级别）
2. 表级隔离（表名包含会话标识）
3. 字段级隔离（记录级别的所有者标识）

### Q3: 如何处理超出上下文限制的长对话？

**答：** Hippos 采用"渐进式披露"策略：
1. 仅存储摘要和索引（轻量级）
2. 按需加载完整内容
3. 自动压缩历史上下文

### Q4: 可以离线使用吗？

**答：** 可以。Hippos 支持：
- SurrealDB 内存模式（无需外部数据库）
- 本地嵌入模型（all-MiniLM-L6-v2）
- 完全离线部署

### Q5: 如何扩展 Hippos？

**答：** Hippos 提供多种扩展方式：
1. 添加新的 API 路由
2. 实现自定义的检索服务
3. 集成其他存储后端
4. 添加自定义健康检查

### Q6: 生产环境需要哪些配置？

**答：** 生产环境建议：
- 启用 TLS/HTTPS
- 启用请求限流
- 配置日志持久化
- 设置监控告警
- 使用 Redis 进行分布式限流

### Q7: 如何备份和恢复数据？

**答：** 备份 SurrealDB 数据：
```bash
# SurrealDB 导出
surreal export --conn ws://localhost:8000 --user root --pass root --ns hippos --db sessions backup.surql

# 恢复
surreal import --conn ws://localhost:8000 --user root --pass root --ns hippos --db sessions backup.surql
```

### Q8: Hippos 与 LangChain/LlamaIndex 集成？

**答：** 可以通过 REST API 集成：
```python
import requests

class HipposMemory:
    def __init__(self, base_url="http://localhost:8080"):
        self.base_url = base_url
        self.api_key = "dev-api-key"
    
    def add_message(self, session_id, role, content):
        # 调用 Turn API
        pass
    
    def search(self, session_id, query):
        # 调用 Search API
        pass
```

---

## 9. 术语表

| 术语 | 英文 | 定义 |
|------|------|------|
| 会话 | Session | 对话管理的基本单元，包含多轮对话 |
| 轮次 | Turn | 对话中的单个交互单元 |
| 脱水 | Dehydration | 将长文本压缩为轻量级摘要的过程 |
| 嵌入 | Embedding | 将文本转换为向量表示 |
| 语义搜索 | Semantic Search | 基于向量相似度的检索方式 |
| 混合搜索 | Hybrid Search | 结合语义和关键词的检索方式 |
| RRF | Reciprocal Rank Fusion | 排名融合算法 |
| 多租户 | Multi-tenant | 单一实例服务多个客户/组织 |
| RBAC | Role-Based Access Control | 基于角色的访问控制 |
| Token Bucket | Token Bucket | 限流算法 |
| MCP | Model Context Protocol | 模型上下文协议 |
| Axum | - | Rust Web 框架 |
| SurrealDB | - | 多模态数据库 |
| LanceDB | - | 向量数据库 |

---

## 附录

### A. 配置参数完整参考

| 配置节 | 参数 | 类型 | 默认值 | 描述 |
|--------|------|------|--------|------|
| app | name | String | "hippos" | 应用名称 |
| app | environment | String | "development" | 环境模式 |
| database | url | String | "ws://localhost:8000" | SurrealDB 连接 URL |
| database | namespace | String | "hippos" | 命名空间 |
| database | database | String | "sessions" | 数据库名 |
| database | username | String | "root" | 用户名 |
| database | password | String | "root" | 密码 |
| database | min_connections | usize | 5 | 最小连接数 |
| database | max_connections | usize | 50 | 最大连接数 |
| vector | data_dir | String | "./data/lancedb" | 向量存储目录 |
| vector | dimension | usize | 384 | 向量维度 |
| vector | distance_type | String | "cosine" | 距离度量 |
| server | host | String | "0.0.0.0" | 绑定地址 |
| server | port | u16 | 8080 | 服务端口 |
| server | workers | usize | 4 | 工作线程数 |
| security | api_key | String | - | API Key |
| security | rate_limit_enabled | bool | false | 启用限流 |
| security | global_rate_limit | u64 | 1000 | 全局限流阈值 |
| embedding | model_name | String | "all-MiniLM-L6-v2" | 嵌入模型 |

### B. 环境变量参考

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `EXOCORTEX_APP_NAME` | "hippos" | 应用名称 |
| `EXOCORTEX_ENVIRONMENT` | "development" | 环境模式 |
| `EXOCORTEX_DATABASE_URL` | "ws://localhost:8000" | 数据库 URL |
| `EXOCORTEX_SERVER_HOST` | "0.0.0.0" | 绑定地址 |
| `EXOCORTEX_SERVER_PORT` | 8080 | 服务端口 |
| `EXOCORTEX_API_KEY` | "dev-api-key" | API Key |
| `EXOCORTEX_LOG_LEVEL` | "info" | 日志级别 |
| `HIPPOS_MCP_MODE` | "0" | MCP 模式开关 |

### C. 性能基准

| 场景 | 目标 QPS | P99 延迟 |
|------|----------|----------|
| 索引列表查询 | ≥ 50,000 | ≤ 10ms |
| 语义检索 | ≥ 5,000 | ≤ 50ms |
| 全文检索 | ≥ 10,000 | ≤ 20ms |
| 混合检索 | ≥ 3,000 | ≤ 100ms |
| 内容获取 | ≥ 20,000 | ≤ 5ms |
| 新对话写入 | ≥ 10,000 | ≤ 10ms |

---

**文档版本：** 0.1.0  
**最后更新：** 2026年1月11日  
**维护团队：** Hippos Team

---

*本手册由 Hippos 团队编写，如有问题请提交 GitHub Issue。*