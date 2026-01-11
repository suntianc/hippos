# Hippos - 专为 AI Agent 设计的高性能上下文管理服务

Hippos 是一款基于 Rust 构建的高性能上下文管理服务，专为大语言模型（LLM）提供持久化的对话记忆能力。通过高效地管理、索引和检索对话上下文，它有效解决了长对话场景中面临的上下文窗口限制问题。

## 🚀 快速开始

### 环境要求

在开始之前，请确保您的开发环境满足以下要求：

| 依赖项 | 版本要求 | 说明 |
|--------|----------|------|
| Rust | 1.70.0 或更高版本 | 2024 Edition，推荐使用最新稳定版 |
| SurrealDB | 1.5.6 或更高版本 | 可选，支持内存模式运行 |
| Cargo | 最新稳定版 | Rust 包管理工具 |

### 安装步骤

按照以下步骤从源码构建和安装 Hippos：

```bash
# 克隆项目仓库
git clone https://github.com/hippos/hippos.git
cd hippos

# 构建项目（发布版本，优化性能）
cargo build --release

# 运行单元测试，验证构建正确性
cargo test --lib
```

### 启动服务

安装完成后，您可以通过以下方式启动服务：

```bash
# 使用默认配置运行（内存数据库模式，无需额外数据库服务）
cargo run

# 自定义服务器端口
EXOCORTEX_SERVER_PORT=8080 cargo run

# 使用自定义配置文件
EXOCORTEX_CONFIG=/path/to/config.yaml cargo run
```

### API 调用示例

以下是一些基本的 API 调用示例，帮助您快速上手：

```bash
# 创建新会话
curl -X POST http://localhost:8080/api/v1/sessions \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"name": "my-first-session", "description": "测试会话"}'

# 列出所有会话
curl http://localhost:8080/api/v1/sessions \
  -H "Authorization: ApiKey dev-api-key"

# 为会话添加对话轮次
curl -X POST http://localhost:8080/api/v1/sessions/{session_id}/turns \
  -H "Content-Type: application/json" \
  -H "Authorization: ApiKey dev-api-key" \
  -d '{"role": "user", "content": "您好，我需要帮助学习 Rust 编程"}'

# 在会话中搜索内容
curl "http://localhost:8080/api/v1/sessions/{session_id}/search?q=rust+编程" \
  -H "Authorization: ApiKey dev-api-key"

# 检查服务健康状态
curl http://localhost:8080/health
```

**默认开发凭据：**
- API 密钥：`dev-api-key`
- JWT 密钥：`dev-secret-change-in-production-min-32-chars`（生产环境必须修改）

## ✨ 功能概述

Hippos 提供了一套完整的上下文管理解决方案，专为 AI Agent 场景优化。以下是核心功能介绍：

### 1. 上下文管理

Hippos 的核心功能是为 AI Agent 提供持久化的对话记忆：

- **会话生命周期管理**：完整支持会话的创建、更新、归档和删除操作
- **对话轮次管理**：独立存储和检索每个对话轮次，包含完整的元数据
- **会话统计追踪**：实时跟踪令牌使用量、轮次数量和存储空间指标
- **多租户隔离**：支持多租户场景，提供完善的数据隔离机制
- **配置灵活性**：每个会话可独立配置摘要限制、最大轮次等参数

### 2. 混合搜索引擎

结合多种检索技术，提供精准的上下文检索能力：

- **语义搜索**：基于 Transformer 嵌入模型的向量相似度搜索
- **全文搜索**：支持关键词匹配和排序的全文检索
- **RRF 融合算法**：使用倒数排名融合（Reciprocal Rank Fusion）优化混合搜索结果
- **实时索引**：新内容自动索引，无需手动触发
- **可配置搜索策略**：支持纯语义、纯全文或混合搜索模式

### 3. 内容处理

为长对话场景提供智能内容压缩能力：

- **对话脱水**：为长对话生成简洁的摘要信息
- **关键词提取**：自动提取重要主题和关键词
- **上下文压缩**：优化上下文内容以适应 LLM 提示词限制
- **批量处理**：支持大规模数据集的高效批量操作

### 4. 安全机制

企业级的安全保障体系：

- **API 密钥认证**：简单高效的服务间通信认证方式
- **JWT 令牌验证**：支持 Bearer 令牌的 JSON Web Token 认证
- **RBAC 权限控制**：基于角色的细粒度权限管理
- **速率限制**：基于令牌桶算法的请求限流
- **请求验证**：完整的输入验证中间件

### 5. 可观测性

生产环境所需的全方位监控能力：

- **Prometheus 指标**：完整的指标端点，支持自定义指标
- **健康检查**：存活检查、就绪检查和完整健康状态
- **结构化日志**：JSON 格式的日志输出，支持追踪
- **请求追踪**：完整的请求延迟和错误追踪

### 6. AI 上下文管理用例

Hippos 特别适合以下应用场景：

| 用例场景 | 描述 |
|----------|------|
| **多轮对话 Agent** | 为持续运行的 AI Agent 提供跨会话的持久记忆 |
| **客服机器人** | 记住用户历史交互，提供个性化服务 |
| **代码助手** | 追踪项目上下文，理解代码库结构 |
| **知识问答系统** | 检索相关历史对话，提供连贯的回答 |
| **教育辅导 AI** | 追踪学习进度，提供个性化的学习建议 |

## 🏗️ 架构说明

### 系统架构概览

Hippos 采用分层架构设计，每层职责明确，模块化程度高：

```
┌─────────────────────────────────────────────────────────────────┐
│                        Hippos Service                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │   REST API  │  │  安全层     │  │     可观测性层          │ │
│  │   (Axum)    │  │  Layer      │  │   (Prometheus/Logs)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
│         │               │                     │                 │
│         └───────────────┼─────────────────────┘                 │
│                         ▼                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                     应用状态层                             │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │ │
│  │  │会话服务  │ │ 轮次服务 │ │ 检索服务 │ │ 脱水服务    │  │ │
│  │  │Service   │ │ Service  │ │ Service  │ │ Service     │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                         │                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    存储层                                   │ │
│  │  ┌──────────────────────────────────────────────────────┐  │ │
│  │  │              SurrealDB 连接池                         │  │ │
│  │  └──────────────────────────────────────────────────────┘  │ │
│  └───────────────────────────────────────────────────────────┘ │
│                         │                                       │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                   索引层                                    │ │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐   │ │
│  │  │向量索引      │ │全文索引      │ │嵌入模型服务     │   │ │
│  │  │(DashMap)     │ │Index         │ │(Candle)          │   │ │
│  │  └──────────────┘ └──────────────┘ └──────────────────┘   │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 组件职责

| 组件 | 职责描述 |
|------|----------|
| **REST API (Axum)** | HTTP 请求处理、路由、请求/响应序列化 |
| **安全层** | 认证、授权、速率限制、输入验证 |
| **可观测性层** | 指标收集、健康检查、日志记录、追踪 |
| **会话服务** | 会话 CRUD 操作、生命周期管理 |
| **轮次服务** | 轮次存储与检索、统计信息追踪 |
| **检索服务** | 向量和混合搜索操作 |
| **脱水服务** | 内容摘要和上下文压缩 |
| **存储层** | 数据库连接池、Repository 模式实现 |
| **索引层** | 向量嵌入、全文搜索、内存索引 |

### 数据流

Hippos 的请求处理流程遵循以下步骤：

1. **请求接入**：HTTP 请求首先经过安全中间件进行身份验证
2. **路由分发**：通过路由器将验证通过的请求分发到对应的处理器
3. **业务处理**：处理器调用业务逻辑服务层
4. **数据交互**：服务层与存储层和索引层进行数据交互
5. **响应返回**：格式化处理结果并返回给客户端
6. **指标记录**：整个请求生命周期中记录相关指标

### 项目目录结构

```
src/
├── lib.rs                 # 库入口点
├── main.rs                # 二进制可执行文件入口
├── api/                   # Phase 4 - REST API 层
│   ├── mod.rs
│   ├── app_state.rs       # 应用状态管理
│   ├── dto/               # 数据传输对象
│   ├── handlers/          # 请求处理器
│   └── routes/            # 路由定义
├── config/                # 配置管理
│   ├── mod.rs
│   ├── config.rs          # 配置结构定义
│   └── loader.rs          # 配置加载器
├── error.rs               # 错误处理定义
├── index/                 # Phase 3 - 搜索索引
│   ├── mod.rs
│   ├── embedding.rs       # 嵌入向量生成
│   ├── full_text.rs       # 全文索引
│   └── vector.rs          # 向量索引
├── models/                # 核心数据模型
│   ├── mod.rs
│   ├── session.rs         # 会话模型
│   ├── turn.rs            # 轮次模型
│   └── index_record.rs    # 索引记录模型
├── observability/         # Phase 6 - 指标与日志
│   └── mod.rs
├── security/              # Phase 5 - 安全
│   ├── auth.rs            # 认证实现
│   ├── config.rs          # 安全配置
│   ├── middleware.rs      # 中间件
│   ├── rate_limit.rs      # 速率限制
│   ├── rbac.rs            # 权限控制
│   ├── validation.rs      # 请求验证
│   └── security_tests.rs  # 安全测试
├── services/              # 业务逻辑
│   ├── mod.rs
│   ├── retrieval.rs       # 检索服务
│   └── dehydration.rs     # 内容脱水服务
└── storage/               # Phase 2 - 持久化
    ├── mod.rs
    ├── surrealdb.rs       # SurrealDB 连接
    └── repository.rs      # Repository 模式实现
```

## 📚 API 文档

### 基础 URL

所有 API 请求的基础 URL 如下：

```
http://localhost:8080
```

### 认证方式

所有 API 请求都需要通过以下任一方式进行身份验证：

```bash
# API 密钥认证
curl -H "Authorization: ApiKey YOUR_API_KEY" http://localhost:8080/api/v1/sessions

# JWT Bearer 令牌认证
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" http://localhost:8080/api/v1/sessions
```

### 会话 API

会话是 Hippos 的核心概念，用于组织和管理对话上下文。

#### 创建会话

创建一个新的对话会话。

```http
POST /api/v1/sessions
Content-Type: application/json
```

**请求体：**

```json
{
  "name": "会话名称",
  "description": "可选的会话描述",
  "max_turns": 100,
  "summary_limit": 10,
  "semantic_search_enabled": true,
  "auto_summarize": false
}
```

**参数说明：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `name` | String | 是 | - | 会话名称 |
| `description` | String | 否 | 空字符串 | 会话描述 |
| `max_turns` | Integer | 否 | 100 | 最大轮次数 |
| `summary_limit` | Integer | 否 | 10 | 生成摘要的轮次间隔 |
| `semantic_search_enabled` | Boolean | 否 | true | 是否启用语义搜索 |
| `auto_summarize` | Boolean | 否 | false | 是否自动生成摘要 |

**响应（201 Created）：**

```json
{
  "id": "session_abc123",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### 列出会话

获取会话列表，支持分页和状态筛选。

```http
GET /api/v1/sessions?page=1&page_size=20&status=active
```

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `page` | Integer | 1 | 页码，从 1 开始 |
| `page_size` | Integer | 20 | 每页数量，最大 100 |
| `status` | String | all | 状态筛选：active、archived、all |

**响应（200 OK）：**

```json
{
  "sessions": [
    {
      "id": "session_abc123",
      "tenant_id": "tenant_1",
      "name": "my-session",
      "description": "会话描述",
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

#### 获取会话

根据 ID 获取单个会话的详细信息。

```http
GET /api/v1/sessions/{id}
```

**路径参数：**

| 参数 | 类型 | 描述 |
|------|------|------|
| `id` | String | 会话唯一标识符 |

**响应（200 OK）：**

```json
{
  "id": "session_abc123",
  "tenant_id": "tenant_1",
  "name": "my-session",
  "description": "会话描述",
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
```

#### 更新会话

更新会话的名称、描述或配置。

```http
PUT /api/v1/sessions/{id}
Content-Type: application/json
```

**请求体：**

```json
{
  "name": "更新后的名称",
  "description": "更新后的描述",
  "max_turns": 200,
  "status": "active"
}
```

**响应（200 OK）：**

```json
{
  "id": "session_abc123",
  "message": "会话更新成功"
}
```

#### 删除会话

删除指定会话及其所有关联数据。

```http
DELETE /api/v1/sessions/{id}
```

**响应（200 OK）：**

```json
{
  "id": "session_abc123",
  "message": "会话删除成功"
}
```

### 轮次 API

轮次代表会话中的一次对话交互，包含用户或 AI 的消息。

#### 添加轮次

向会话中添加新的对话轮次。

```http
POST /api/v1/sessions/{id}/turns
Content-Type: application/json
```

**请求体：**

```json
{
  "role": "user",
  "content": "用户消息内容",
  "metadata": {
    "custom_key": "custom_value"
  }
}
```

**参数说明：**

| 参数 | 类型 | 必填 | 描述 |
|------|------|------|------|
| `role` | String | 是 | 角色：user 或 assistant |
| `content` | String | 是 | 消息内容 |
| `metadata` | Object | 否 | 自定义元数据 |

**响应（201 Created）：**

```json
{
  "id": "turn_xyz789",
  "session_id": "session_abc123",
  "turn_number": 1,
  "created_at": "2024-01-15T11:00:00Z",
  "message_count": 1,
  "token_count": 50
}
```

#### 列出轮次

获取会话中的所有轮次，支持分页。

```http
GET /api/v1/sessions/{id}/turns?page=1&page_size=50
```

**响应（200 OK）：**

```json
{
  "turns": [
    {
      "id": "turn_xyz789",
      "session_id": "session_abc123",
      "turn_number": 1,
      "created_at": "2024-01-15T11:00:00Z",
      "role": "user",
      "content": "用户消息内容",
      "metadata": {}
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 50
}
```

#### 获取轮次

获取单个轮次的详细信息。

```http
GET /api/v1/sessions/{id}/turns/{turn_id}
```

**响应（200 OK）：**

```json
{
  "id": "turn_xyz789",
  "session_id": "session_abc123",
  "turn_number": 1,
  "created_at": "2024-01-15T11:00:00Z",
  "role": "user",
  "content": "用户消息内容",
  "metadata": {},
  "token_count": 50
}
```

#### 删除轮次

删除指定的对话轮次。

```http
DELETE /api/v1/sessions/{id}/turns/{turn_id}
```

**响应（200 OK）：**

```json
{
  "id": "turn_xyz789",
  "message": "轮次删除成功"
}
```

### 搜索 API

提供多种搜索方式，用于检索会话中的相关内容。

#### 混合搜索

结合语义搜索和全文搜索的混合检索。

```http
GET /api/v1/sessions/{id}/search?q=搜索内容&limit=10&strategy=hybrid
```

**查询参数：**

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `q` | String | - | 搜索查询 |
| `limit` | Integer | 10 | 返回结果数量 |
| `strategy` | String | hybrid | 搜索策略：semantic、fulltext、hybrid |

**响应（200 OK）：**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.95,
      "type": "semantic",
      "content": "匹配的内容...",
      "metadata": {
        "turn_number": 1,
        "created_at": "2024-01-15T11:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 15
}
```

#### 纯语义搜索

仅使用向量相似度进行语义搜索。

```http
POST /api/v1/sessions/{id}/search/semantic
Content-Type: application/json
```

**请求体：**

```json
{
  "query": "关于 Rust 编程的讨论内容？",
  "limit": 10,
  "threshold": 0.7
}
```

**参数说明：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `query` | String | 是 | - | 语义搜索查询 |
| `limit` | Integer | 否 | 10 | 返回结果数量 |
| `threshold` | Double | 否 | 0.0 | 相似度阈值 |

**响应（200 OK）：**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.89,
      "content": "Rust 编程讨论内容...",
      "metadata": {
        "turn_number": 5,
        "created_at": "2024-01-15T12:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 25
}
```

### 健康与指标 API

用于服务监控和健康检查。

#### 完整健康检查

返回服务的完整健康状态和所有依赖检查结果。

```http
GET /health
```

**响应（200 OK）：**

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
      "message": "已连接",
      "latency_ms": 5
    }
  ]
}
```

#### 存活检查

简单的存活探测，用于负载均衡器检查服务是否运行。

```http
GET /health/live
```

**响应：** `OK`（200 OK）

#### 就绪检查

检查服务是否准备好接受请求，会验证所有依赖服务。

```http
GET /health/ready
```

**响应：** `Ready`（200 OK）或 `Not Ready`（503 Service Unavailable）

#### Prometheus 指标

返回 Prometheus 格式的指标数据。

```http
GET /metrics
```

**响应示例：**

```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total 1234
# HELP http_request_duration_seconds HTTP request duration in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_sum 123.456
http_request_duration_seconds_count 1234
# HELP active_connections Active HTTP connections
# TYPE active_connections gauge
active_connections 5
# HELP sessions_active Active sessions
# TYPE sessions_active gauge
sessions_active 10
# HELP turns_total Total turns stored
# TYPE turns_total counter
turns_total 150
# HELP search_requests_total Total search requests
# TYPE search_requests_total counter
search_requests_total 500
# HELP errors_total Total errors
# TYPE errors_total counter
errors_total 5
```

#### 版本信息

返回服务版本和运行时信息。

```http
GET /version
```

**响应（200 OK）：**

```json
{
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "timestamp": "2024-01-15T12:00:00Z"
}
```

### 错误响应

所有 API 错误返回统一的错误格式：

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "请求参数验证失败",
    "details": {
      "field": "name",
      "reason": "名称不能为空"
    }
  },
  "request_id": "req_abc123",
  "timestamp": "2024-01-15T12:00:00Z"
}
```

**常见错误码：**

| 错误码 | HTTP 状态码 | 描述 |
|--------|-------------|------|
| `UNAUTHORIZED` | 401 | 未提供或无效的认证令牌 |
| `FORBIDDEN` | 403 | 无权限访问该资源 |
| `NOT_FOUND` | 404 | 请求的资源不存在 |
| `VALIDATION_ERROR` | 400 | 请求参数验证失败 |
| `RATE_LIMITED` | 429 | 请求频率超出限制 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

## ⚙️ 配置说明

Hippos 通过 `config.yaml` 文件进行配置，支持环境变量覆盖。

### 配置文件示例

完整配置文件示例：

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
  connection_timeout: 30
  idle_timeout: 300

# 向量索引配置
vector:
  data_dir: "./data/lancedb"
  dimension: 384
  nlist: 1024
  nprobe: 32
  distance_type: "cosine"

# 服务器配置
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4
  request_timeout: 30
  max_request_size: 10485760

# 安全配置
security:
  api_key: "dev-api-key-change-in-production"
  rate_limit_enabled: false
  global_rate_limit: 1000
  per_session_rate_limit: 100
  redis_url: "redis://localhost:6379"
  jwt_secret: "dev-secret-change-in-production-min-32-chars"
  jwt_expiry_seconds: 3600

# 日志配置
logging:
  level: "debug"
  structured: true
  log_dir: "./logs"
  file_max_size: 104857600
  file_max_count: 10

# 嵌入模型配置
embedding:
  model_name: "all-MiniLM-L6-v2"
  model_path: null
  batch_size: 32
  use_gpu: false
```

### 环境变量

可以通过环境变量覆盖配置文件中的设置：

| 环境变量 | 默认值 | 描述 |
|----------|--------|------|
| `EXOCORTEX_APP_NAME` | `hippos` | 应用名称 |
| `EXOCORTEX_ENVIRONMENT` | `development` | 环境模式 |
| `EXOCORTEX_DATABASE_URL` | `ws://localhost:8000` | SurrealDB 连接 URL |
| `EXOCORTEX_DATABASE_NAMESPACE` | `hippos` | 数据库命名空间 |
| `EXOCORTEX_DATABASE_NAME` | `sessions` | 数据库名称 |
| `EXOCORTEX_SERVER_HOST` | `0.0.0.0` | 服务器绑定地址 |
| `EXOCORTEX_SERVER_PORT` | `8080` | 服务器端口 |
| `EXOCORTEX_SERVER_WORKERS` | `4` | 工作线程数 |
| `EXOCORTEX_API_KEY` | `dev-api-key` | 默认 API 密钥 |
| `EXOCORTEX_LOG_LEVEL` | `info` | 日志级别 |
| `EXOCORTEX_EMBEDDING_MODEL` | `all-MiniLM-L6-v2` | 嵌入模型名称 |

### 配置项详解

#### 数据库配置

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `url` | String | `ws://localhost:8000` | SurrealDB WebSocket URL |
| `namespace` | String | `hippos` | 数据库命名空间 |
| `database` | String | `sessions` | 数据库名称 |
| `username` | String | `root` | 认证用户名 |
| `password` | String | `root` | 认证密码 |
| `min_connections` | usize | `5` | 连接池最小连接数 |
| `max_connections` | usize | `50` | 连接池最大连接数 |
| `connection_timeout` | u64 | `30` | 连接超时时间（秒） |
| `idle_timeout` | u64 | `300` | 空闲连接超时时间（秒） |

**使用内存模式：**

```yaml
database:
  url: "mem://"
```

#### 向量索引配置

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `data_dir` | String | `./data/lancedb` | 向量数据库存储目录 |
| `dimension` | usize | `384` | 嵌入向量维度 |
| `nlist` | usize | `1024` | IVF 索引列表数 |
| `nprobe` | usize | `32` | 搜索探针数 |
| `distance_type` | String | `cosine` | 距离度量方式 |

#### 服务器配置

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `host` | String | `0.0.0.0` | 绑定的网络接口 |
| `port` | u16 | `8080` | 服务器端口 |
| `workers` | usize | `4` | Tokio 工作线程数 |
| `request_timeout` | u64 | `30` | 请求超时时间（秒） |
| `max_request_size` | usize | `10485760` | 最大请求体大小（10MB） |

#### 安全配置

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `api_key` | String | - | 默认 API 密钥 |
| `rate_limit_enabled` | bool | `false` | 是否启用速率限制 |
| `global_rate_limit` | u64 | `1000` | 全局限流（请求/分钟） |
| `per_session_rate_limit` | u64 | `100` | 每会话限流（请求/分钟） |
| `redis_url` | String | `redis://localhost:6379` | Redis URL（用于限流） |
| `jwt_secret` | String | - | JWT 密钥（至少 32 字符） |
| `jwt_expiry_seconds` | u64 | `3600` | JWT 令牌过期时间 |

#### 嵌入模型配置

| 参数 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `model_name` | String | `all-MiniLM-L6-v2` | HuggingFace 模型标识符 |
| `model_path` | Option<String> | `None` | 本地模型路径 |
| `batch_size` | usize | `32` | 批处理大小 |
| `use_gpu` | bool | `false` | 是否启用 GPU 加速 |

## 📊 指标与监控

### 可用指标

Hippos 提供以下 Prometheus 指标：

| 指标名称 | 类型 | 描述 |
|----------|------|------|
| `http_requests_total` | Counter | HTTP 请求总数 |
| `http_request_duration_seconds` | Histogram | 请求持续时间直方图 |
| `active_connections` | Gauge | 当前活跃连接数 |
| `sessions_active` | Gauge | 活跃会话数量 |
| `sessions_archived` | Gauge | 已归档会话数量 |
| `turns_total` | Counter | 存储的轮次总数 |
| `search_requests_total` | Counter | 搜索请求总数 |
| `search_latency_seconds` | Histogram | 搜索请求延迟直方图 |
| `errors_total` | Counter | 错误总数 |

### 健康检查端点

| 端点 | 描述 |
|------|------|
| `/health` | 完整健康状态，包含所有检查项 |
| `/health/live` | 简单存活检查（始终返回 OK） |
| `/health/ready` | 就绪检查（验证依赖项） |
| `/metrics` | Prometheus 指标端点 |
| `/version` | 版本和运行时信息 |

### 自定义健康检查

您可以通过实现 `HealthCheck` trait 来注册自定义健康检查：

```rust
use crate::observability::HealthCheckResult;

#[async_trait]
trait HealthCheck: Send + Sync {
    fn name(&self) -> String;
    async fn check(&self) -> HealthCheckResult;
}
```

### 日志配置

Hippos 使用 `tracing` 库进行结构化日志记录：

```yaml
logging:
  level: "debug"        # 日志级别：trace, debug, info, warn, error
  structured: true      # 使用结构化 JSON 日志
  log_dir: "./logs"     # 日志目录
  file_max_size: 104857600  # 单个日志文件最大大小（100MB）
  file_max_count: 10    # 保留的日志文件数量
```

**日志输出示例：**

```json
{
  "timestamp": "2024-01-15T12:00:00.000Z",
  "level": "INFO",
  "target": "hippos::api::handlers",
  "message": "Request completed",
  "request_id": "req_abc123",
  "method": "GET",
  "path": "/api/v1/sessions",
  "status": 200,
  "duration_ms": 15.5
}
```

## 🤖 MCP 服务器

Hippos 还可以作为 **Model Context Protocol (MCP)** 服务器运行，允许 AI Agent 和应用程序通过标准化协议访问其上下文管理功能。这使得与 Claude Desktop、Cursor 和其他 AI 工具等 MCP 兼容客户端的集成变得无缝。

### 作为 MCP 服务器运行

```bash
# 构建发布版本
cargo build --release

# 在 MCP 模式下启动（使用 stdio 进行通信）
./target/release/hippos
```

或者在运行前设置环境变量：

```bash
export HIPPOS_MCP_MODE=1
./target/release/hippos
```

在 MCP 模式下运行时，Hippos 暴露了两个可供客户端调用的工具：

在 MCP 模式下运行时，Hippos 暴露了两个可供客户端调用的工具：

### 可用工具

#### hippos_search

在会话内执行结合语义和关键词匹配的混合搜索。

**参数：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `session_id` | string | 是 | - | 要搜索的会话的唯一标识符 |
| `query` | string | 是 | - | 搜索查询文本 |
| `limit` | integer | 否 | 10 | 返回的最大结果数量 |

**示例请求：**

```json
{
  "session_id": "session_abc123",
  "query": "关于 Rust 编程的讨论内容？",
  "limit": 5
}
```

**响应：**

```json
{
  "results": [
    {
      "id": "turn_xyz789",
      "score": 0.89,
      "content": "我们讨论了 Rust 的所有权模型...",
      "metadata": {
        "turn_number": 5,
        "created_at": "2024-01-15T12:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 25
}
```

#### hippos_semantic_search

在会话内执行纯语义（基于向量）搜索。

**参数：**

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `session_id` | string | 是 | - | 要搜索的会话的唯一标识符 |
| `query` | string | 是 | - | 语义搜索查询 |
| `limit` | integer | 否 | 10 | 返回的最大结果数量 |

**示例请求：**

```json
{
  "session_id": "session_abc123",
  "query": "异步编程是如何工作的？",
  "limit": 5
}
```

**响应：**

```json
{
  "results": [
    {
      "id": "turn_abc456",
      "score": 0.92,
      "content": "Rust 中的异步编程使用 async/await 语法...",
      "metadata": {
        "turn_number": 10,
        "created_at": "2024-01-15T14:00:00Z"
      }
    }
  ],
  "total_results": 1,
  "search_time_ms": 20
}
```

### 使用 MCP Inspector 测试

您可以使用官方的 MCP Inspector 工具测试 MCP 服务器：

```bash
# 安装 MCP Inspector
npx @modelcontextprotocol/inspector

# 对您的 MCP 服务器运行 inspector
npx @modelcontextprotocol/inspector ./target/release/hippos
```

### 客户端集成

#### Claude Desktop

将 Hippos 添加到您的 `claude_desktop_config.json`：

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

#### Cursor IDE

添加到您的 Cursor MCP 配置：

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

#### Claude Code CLI

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

> **注意**：将 `/full/path/to/hippos` 替换为编译后二进制的实际路径。

### MCP 模式下的环境变量

| 变量 | 默认值 | 描述 |
|------|--------|------|
| `HIPPOS_MCP_MODE` | `0` | 设置为 `1` 以启用 MCP stdio 服务器模式 |
| `EXOCORTEX_DATABASE_URL` | `ws://localhost:8000` | SurrealDB 连接 URL |
| `EXOCORTEX_API_KEY` | `dev-api-key` | 认证 API 密钥 |

## 🔒 安全机制

### 认证方式

Hippos 支持两种主要的认证方式：

#### 1. API 密钥认证

适用于服务间通信的简单基于令牌的认证：

```bash
curl -H "Authorization: ApiKey YOUR_API_KEY" http://localhost:8080/api/v1/sessions
```

**配置方式：**

```yaml
security:
  api_key: "your-secret-api-key"
```

#### 2. JWT 认证

使用 JSON Web Tokens 的 Bearer 令牌认证：

```bash
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" http://localhost:8080/api/v1/sessions
```

**JWT 声明结构：**

```json
{
  "sub": "user_id",
  "tenant_id": "tenant_1",
  "role": "admin",
  "exp": 1705315200,
  "iss": "hippos",
  "aud": "hippos-api"
}
```

**配置方式：**

```yaml
security:
  jwt_secret: "your-32-character-secret-key"
  jwt_issuer: "hippos"
  jwt_audience: "hippos-api"
  jwt_expiry_seconds: 3600
```

**JWT 字段说明：**

| 字段 | 描述 |
|------|------|
| `sub` | 用户唯一标识 |
| `tenant_id` | 租户标识（多租户场景） |
| `role` | 用户角色：admin、user、readonly |
| `exp` | 令牌过期时间戳 |
| `iss` | 签发者 |
| `aud` | 目标受众 |

### 速率限制

Hippos 实现基于令牌桶算法的速率限制：

| 限制类型 | 描述 | 默认值 |
|----------|------|--------|
| 全局限流 | 全局每分钟请求数 | 1000/分钟 |
| 每会话限流 | 每个会话每分钟请求数 | 100/分钟 |
| 自定义端点限流 | 可为特定端点配置独立限制 | 可配置 |

**配置方式：**

```yaml
security:
  rate_limit_enabled: true
  global_rate_limit: 1000
  per_session_rate_limit: 100
  redis_url: "redis://localhost:6379"  # 分布式限流需要 Redis
```

**响应头信息：**

当请求被限流时，返回以下响应头：

| 响应头 | 描述 |
|--------|------|
| `X-RateLimit-Limit` | 允许的请求数 |
| `X-RateLimit-Remaining` | 剩余请求数 |
| `X-RateLimit-Reset` | 速率限制重置时间戳 |
| `Retry-After` | 建议的重试等待时间（秒） |

### 基于角色的访问控制（RBAC）

Hippos 提供细粒度的权限控制：

**预定义角色：**

| 角色 | 权限 |
|------|------|
| `admin` | 完全访问所有资源 |
| `user` | 访问自己的资源 |
| `readonly` | 只读访问权限 |

**权限模型：**

```rust
use hippos::security::rbac::{Role, Permission, Resource};

// 定义权限
let permissions = vec![
    Permission::new("sessions:read", Role::User),
    Permission::new("sessions:write", Role::User),
    Permission::new("sessions:delete", Role::Admin),
    Permission::new("turns:read", Role::User),
    Permission::new("turns:write", Role::User),
    Permission::new("search:execute", Role::User),
    Permission::new("admin:metrics", Role::Admin),
];
```

### 请求验证

所有进入的请求都会经过验证：

- **JSON Schema 验证**：请求体结构验证
- **类型验证**：字段类型和格式验证
- **大小限制**：请求体最大大小限制
- **Content-Type 验证**：POST/PUT 请求必需 Content-Type

**验证错误响应：**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "请求参数验证失败",
    "details": {
      "field": "content",
      "reason": "内容不能为空且长度不能超过 100000 字符"
    }
  },
  "request_id": "req_abc123",
  "timestamp": "2024-01-15T12:00:00Z"
}
```

## 🛠️ 开发指南

### 从源码构建

```bash
# Debug 构建
cargo build

# Release 构建（优化性能）
cargo build --release

# 构建指定特性
cargo build --release --features "metrics,security"
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test --lib index

# 显示测试输出
cargo test --lib -- --nocapture

# 运行集成测试
cargo test --test integration
```

**测试结果示例：**

```
✅ 构建: 0 错误, 2 警告（外部依赖）
✅ 测试: 20/20 通过（100%）

测试分布：
├── Index 模块: 10 个测试
├── API 模块: 3 个测试
├── 可观测性: 3 个测试
└── 服务: 4 个测试
```

### 测试覆盖率

```bash
# 安装 tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html
```

### 添加新功能

#### 1. 创建新模块

```rust
// src/new_feature/mod.rs
pub mod handler;
pub mod service;
pub mod model;
```

#### 2. 定义数据模型

```rust
// src/new_feature/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeature {
    pub id: String,
    pub name: String,
    // 添加更多字段
}
```

#### 3. 实现服务层

```rust
// src/new_feature/service.rs
use async_trait::async_trait;

#[async_trait]
pub trait NewFeatureService {
    async fn create(&self, input: Input) -> Result<Output>;
    async fn get(&self, id: &str) -> Result<Output>;
}
```

#### 4. 创建处理器

```rust
// src/new_feature/handler.rs
use axum::{Json, extract::State};
use crate::api::AppState;

pub async fn create_feature(
    State(state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 实现逻辑
}
```

#### 5. 添加路由

```rust
// src/api/routes/new_feature_routes.rs
pub fn create_new_feature_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_feature))
        .route("/:id", get(get_feature))
}
```

#### 6. 注册路由

```rust
// src/api/mod.rs
pub fn create_router(app_state: AppState) -> Router {
    let api = Router::new()
        .merge(routes::session_routes::create_session_router())
        .merge(routes::turn_routes::create_turn_router())
        .merge(routes::search_routes::create_search_router())
        .merge(routes::new_feature_routes::create_new_feature_router()); // 在这里添加

    Router::new().nest("/api/v1", api).with_state(app_state)
}
```

### 代码风格

```bash
# 格式化代码
cargo fmt

# 检查格式化
cargo fmt --check

# 代码检查
cargo clippy

# 修复 clippy 建议
cargo clippy --fix
```

### 数据库迁移

```bash
# 运行迁移
cargo run -- migrate

# 创建新迁移
cargo run -- migration create migration_name
```

### 性能基准测试

```bash
# 运行基准测试
cargo bench

# 运行特定基准测试
cargo bench search_latency
```

## 📦 依赖说明

### 核心依赖

| 依赖项 | 版本 | 用途 |
|--------|------|------|
| `axum` | 0.7 | Web 框架 |
| `surrealdb` | 1.0 | 数据库 |
| `tokio` | 1.35 | 异步运行时 |
| `tracing` | 0.1 | 结构化日志 |
| `serde` | 1.0 | 序列化/反序列化 |
| `thiserror` | 1.0 | 错误处理 |

### 可选依赖

| 依赖项 | 版本 | 用途 |
|--------|------|------|
| `candle-core` | 0.4 | 机器学习/嵌入推理 |
| `tokenizers` | 0.22 | 文本分词 |
| `redis` | 0.25 | 速率限制 |
| `jsonwebtoken` | 10.2 | JWT 认证 |
| `openssl` | 0.10 | 加密支持 |

## 🤝 贡献指南

### 入门步骤

1. Fork 本仓库
2. 创建功能分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 提交 Pull Request

### 开发工作流

1. **阅读**：阅读现有代码和文档
2. **理解**：理解架构和代码模式
3. **实现**：编写清晰、有测试的代码
4. **测试**：确保所有测试通过
5. **文档**：必要时更新文档
6. **评审**：处理评审反馈

### 代码标准

- 遵循 Rust 最佳实践（rustfmt、clippy）
- 编写全面的测试
- 为公共 API 添加文档注释
- 使用有意义的变量和函数名
- 保持函数小而专注
- 编写描述性的提交信息

### Pull Request 指南

- 提供清晰的更改描述
- 关联相关 Issue
- 包含测试覆盖率
- 更新文档
- 确保 CI 通过

## 📄 许可证

本项目基于 MIT 许可证开源，详情请参阅 [LICENSE](LICENSE) 文件。

## 🙏 致谢

感谢以下开源项目：

- [SurrealDB](https://surrealdb.com/) - 数据库
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Candle](https://github.com/huggingface/candle) - 机器学习框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [sentence-transformers](https://www.sbert.net/) - 嵌入模型

## 📞 支持

- **文档**：[docs.hippos.io](https://docs.hippos.io)
- **Issue**：[GitHub Issues](https://github.com/hippos/hippos/issues)
- **讨论**：[GitHub Discussions](https://github.com/hippos/hippos/discussions)

---

**Hippos** - 为 AI Agent 赋予持久记忆能力