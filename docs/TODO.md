# Hippos 实施计划

> 详细的任务清单和里程碑规划

## 阶段概览

| 阶段 | 名称 | 优先级 | 预估工时 |
|------|------|--------|----------|
| Phase 1 | 项目基础 | P0 | 1 周 |
| Phase 2 | 存储层 | P0 | 1 周 |
| Phase 3 | 索引与检索引擎 | P0 | 1.5 周 |
| Phase 4 | API 层 | P0 | 1 周 |
| Phase 5 | 安全层 | P1 | 1 周 |
| Phase 6 | 可观测性 | P1 | 0.5 周 |
| Phase 7 | 测试与验证 | P1 | 1 周 |

---

## Phase 1: 项目基础

### 目标
建立项目骨架，定义核心数据结构，配置开发环境。

### 任务清单

#### 1.1 项目初始化
- [ ] 创建 Cargo 工作区 (`cargo new --lib hippos`)
- [ ] 配置 `Cargo.toml` 依赖：
  - `axum` - Web 框架
  - `surrealdb` - 数据库客户端
  - `lancedb` - 向量索引
  - `tokio` - 异步运行时
  - `serde` / `serde_json` - 序列化
  - `anyhow` - 错误处理
- [ ] 配置 `.cargo/config.toml`（若有）
- [ ] 设置代码格式化 (`rustfmt.toml`)
- [ ] 配置 Clippy linting

#### 1.2 配置管理
- [ ] 创建 `config.yaml` 模板
- [ ] 实现配置加载模块 (`src/config/mod.rs`)
- [ ] 支持环境变量覆盖
- [ ] 定义配置结构体：
  - `DatabaseConfig` - SurrealDB 配置
  - `VectorConfig` - LanceDB 配置
  - `ServerConfig` - 服务配置
  - `SecurityConfig` - 安全配置
  - `LoggingConfig` - 日志配置

#### 1.3 核心数据模型
- [ ] 定义 `Session` 结构体 (`src/models/session.rs`)
- [ ] 定义 `Turn` 结构体 (`src/models/turn.rs`)
- [ ] 定义 `IndexRecord` 结构体 (`src/models/index_record.rs`)
- [ ] 定义 `Metadata` 结构体
- [ ] 定义 `Dehydrated` 结构体
- [ ] 实现 `Serialize` / `Deserialize`
- [ ] 实现 `Validate` trait

#### 1.4 错误处理
- [ ] 定义 `Error` 枚举 (`src/error.rs`)
- [ ] 实现 `std::error::Error` trait
- [ ] 实现 `From` traits for 常见错误
- [ ] 定义 `Result` 类型别名
- [ ] 实现错误转换宏

#### 1.5 项目结构
```
src/
├── main.rs
├── lib.rs
├── config/
│   ├── mod.rs
│   └── config.rs
├── models/
│   ├── mod.rs
│   ├── session.rs
│   ├── turn.rs
│   └── index_record.rs
├── error.rs
└── lib.rs
```

### 交付物
- 可编译的项目骨架
- 配置管理系统
- 核心数据结构定义
- 错误处理框架

### 验收标准
- `cargo build` 成功
- `cargo test` 通过
- `cargo clippy` 无警告

---

## Phase 2: 存储层

### 目标
实现 SurrealDB 仓储层，提供可靠的数据持久化。

### 任务清单

#### 2.1 SurrealDB 集成
- [ ] 创建 SurrealDB 客户端模块 (`src/storage/mod.rs`)
- [ ] 实现连接管理
- [ ] 配置连接池
- [ ] 实现 Namespace 切换
- [ ] 实现 Database 切换

#### 2.2 仓储模式
- [ ] 定义 `Repository` trait (`src/storage/repository.rs`)
- [ ] 实现 `SessionRepository`
- [ ] 实现 `TurnRepository`
- [ ] 实现 `IndexRecordRepository`
- [ ] 实现批量操作

#### 2.3 数据访问模式
- [ ] 实现 CRUD 操作
- [ ] 实现查询构建器
- [ ] 实现分页查询
- [ ] 实现事务支持

#### 2.4 数据迁移
- [ ] 创建迁移目录 (`migrations/`)
- [ ] 定义迁移脚本格式
- [ ] 实现会话表迁移
- [ ] 实现轮次表迁移
- [ ] 实现索引表迁移

### 交付物
- SurrealDB 仓储层
- 连接池管理
- 完整的 CRUD API
- 数据迁移脚本

### 验收标准
- 单元测试覆盖率 ≥ 80%
- 集成测试通过

---

## Phase 3: 索引与检索引擎

### 目标
构建高性能的索引和检索系统，支持向量搜索和全文搜索。

### 任务清单

#### 3.1 LanceDB 集成
- [ ] 创建 LanceDB 客户端模块 (`src/index/mod.rs`)
- [ ] 实现连接管理
- [ ] 创建向量表
- [ ] 配置 IVF-PQ 参数

#### 3.2 Embedding 服务
- [ ] 定义 `EmbeddingModel` trait (`src/index/embedding.rs`)
- [ ] 实现 all-MiniLM-L6-v2 集成
- [ ] 实现本地模型加载
- [ ] 实现批量编码
- [ ] 实现缓存机制

#### 3.3 索引管理
- [ ] 实现向量索引创建
- [ ] 实现索引更新
- [ ] 实现索引删除
- [ ] 实现索引重建

#### 3.4 检索管道
- [ ] 实现向量检索 (`src/index/vector_search.rs`)
- [ ] 实现全文检索 (`src/index/full_text_search.rs`)
- [ ] 实现混合检索 (`src/index/hybrid_search.rs`)
- [ ] 实现 RRF 融合算法
- [ ] 实现时间加权排序

#### 3.5 脱水处理
- [ ] 实现摘要生成 (`src/services/dehydration.rs`)
- [ ] 实现主题提取
- [ ] 实现标签生成
- [ ] 实现向量化

### 交付物
- LanceDB 集成
- Embedding 服务
- 完整的检索管道
- 脱水处理服务

### 验收标准
- 检索延迟 P99 < 10ms
- 向量检索 QPS ≥ 5,000

---

## Phase 4: API 层

### 目标
构建完整的 API 层，支持 MCP 协议和 RESTful API。

### 任务清单

#### 4.1 Axum 服务器
- [ ] 创建 HTTP 服务器 (`src/api/mod.rs`)
- [ ] 配置路由
- [ ] 配置中间件
- [ ] 配置 CORS
- [ ] 配置超时

#### 4.2 MCP 协议适配
- [ ] 实现 MCP 协议解析 (`src/api/mcp/mod.rs`)
- [ ] 实现 `tools/list` 端点
- [ ] 实现 `tools/call` 端点
- [ ] 实现工具调用路由
- [ ] 实现响应格式化

#### 4.3 REST API 端点
- [ ] 实现会话管理 API (`src/api/sessions.rs`)
  - POST /api/v1/sessions
  - GET /api/v1/sessions/{id}
  - DELETE /api/v1/sessions/{id}
- [ ] 实现轮次管理 API (`src/api/turns.rs`)
  - POST /api/v1/sessions/{id}/turns
  - GET /api/v1/sessions/{id}/turns
  - GET /api/v1/sessions/{id}/turns/{tid}
- [ ] 实现索引搜索 API (`src/api/search.rs`)
  - GET /api/v1/sessions/{id}/indices
  - GET /api/v1/sessions/{id}/search

#### 4.4 认证与授权
- [ ] 实现 API Key 认证
- [ ] 实现会话验证
- [ ] 实现权限检查中间件

#### 4.5 错误处理
- [ ] 实现错误响应格式化
- [ ] 实现统一错误码
- [ ] 实现详细错误信息

### 交付物
- Axum HTTP 服务器
- MCP 协议适配器
- 完整的 REST API
- 认证授权中间件

### 验收标准
- API 文档完整
- 端到端测试通过

---

## Phase 5: 安全层

### 目标
实现企业级的安全特性，保护用户数据。

### 任务清单

#### 5.1 三级数据隔离
- [ ] 实现 Namespace 隔离
- [ ] 实现表级隔离
- [ ] 实现字段级隔离
- [ ] 实现访问控制列表

#### 5.2 认证授权
- [ ] 实现 API Key 生成
- [ ] 实现 Token 验证
- [ ] 实现权限验证

#### 5.3 速率限制
- [ ] 实现 Redis 滑动窗口 (`src/security/rate_limit.rs`)
- [ ] 实现全局限流
- [ ] 实现会话级限流

#### 5.4 数据加密
- [ ] 实现 AES-256-GCM 加密
- [ ] 实现密钥管理
- [ ] 实现敏感数据加密

#### 5.5 审计日志
- [ ] 实现访问日志
- [ ] 实现操作日志
- [ ] 实现日志存储

### 交付物
- 完整的安全框架
- 速率限制中间件
- 审计日志系统

### 验收标准
- 安全扫描通过
- 无高危漏洞

---

## Phase 6: 可观测性

### 目标
建立完整的监控和日志系统，支持问题诊断和性能分析。

### 任务清单

#### 6.1 Prometheus 指标
- [ ] 定义业务指标 (`src/metrics/mod.rs`)
- [ ] 实现 HTTP 请求指标
- [ ] 实现数据库操作指标
- [ ] 实现自定义指标

#### 6.2 结构化日志
- [ ] 实现日志配置
- [ ] 实现日志格式化
- [ ] 实现日志级别控制
- [ ] 实现上下文日志

#### 6.3 健康检查
- [ ] 实现存活检查端点 (`/health/live`)
- [ ] 实现就绪检查端点 (`/health/ready`)
- [ ] 实现依赖检查

#### 6.4 告警配置
- [ ] 定义告警规则
- [ ] 配置 AlertManager
- [ ] 实现告警通知

### 交付物
- Prometheus 指标端点
- 结构化日志系统
- 健康检查端点
- 告警配置

### 验收标准
- 指标采集正常
- 日志可查询

---

## Phase 7: 测试与验证

### 目标
确保系统质量，验证性能目标。

### 任务清单

#### 7.1 单元测试
- [ ] 为核心模块编写单元测试
- [ ] 实现 Mock 测试
- [ ] 实现 Property Testing

#### 7.2 集成测试
- [ ] 实现 API 集成测试
- [ ] 实现存储层集成测试
- [ ] 实现检索集成测试

#### 7.3 性能测试
- [ ] 编写基准测试
- [ ] 执行吞吐量测试
- [ ] 执行延迟测试
- [ ] 验证性能目标

#### 7.4 CI/CD 流水线
- [ ] 配置 GitHub Actions
- [ ] 实现自动化测试
- [ ] 实现自动化构建
- [ ] 实现自动化部署

### 交付物
- 完整的测试套件
- 性能测试报告
- CI/CD 流水线

### 验收标准
- 测试覆盖率 ≥ 80%
- 所有性能目标达成
- CI/CD 正常运行

---

## 里程碑

| 里程碑 | 内容 | 目标日期 | 状态 |
|--------|------|----------|------|
| M0 | 项目初始化完成 | 2026-01-10 | ✅ |
| M1 | 项目骨架搭建完成 | T+1 周 | ⏳ |
| M2 | 核心功能可运行 | T+2 周 | ⏳ |
| M3 | 性能测试达标 | T+3 周 | ⏳ |
| M4 | 正式发布 | T+4 周 | ⏳ |

---

## 依赖关系

```
Phase 1 (项目基础)
    │
    ▼
Phase 2 (存储层) ──────────────┐
    │                          │
    ▼                          │
Phase 3 (索引与检索引擎)        │
    │                          │
    ▼                          │
Phase 4 (API 层) ──────────────┤
    │                          │
    ▼                          │
Phase 5 (安全层)               │
    │                          │
    ▼                          ▼
Phase 6 (可观测性) ◄────────────┘
    │
    ▼
Phase 7 (测试与验证)
```
