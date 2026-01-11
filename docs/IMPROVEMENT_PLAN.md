# Hippos 改进设计方案

## 1. 问题定义

### 1.1 问题清单

基于代码探索和分析，发现以下问题需要解决：

#### 1.1.1 严重缺陷（P0）

| 问题 ID | 问题描述 | 影响范围 | 严重程度 |
|---------|----------|----------|----------|
| BUG-001 | `turn_number` 硬编码为 1 | 所有对话轮次创建 | 严重 |
| BUG-002 | Repository 层 `expect()` 调用导致 panic | 数据持久化 | 严重 |
| BUG-003 | Handler 层业务逻辑完全缺失 | API 功能 | 严重 |

#### 1.1.2 架构问题（P1）

| 问题 ID | 问题描述 | 影响范围 | 优先级 |
|---------|----------|----------|--------|
| ARCH-001 | 缺少 SessionService 和 TurnService | 服务层 | 高 |
| ARCH-002 | 连接池设计缺陷（单连接复用） | 存储层 | 高 |
| ARCH-003 | 全表加载问题（list/count 方法） | 数据库 | 高 |
| ARCH-004 | 缺少领域事件和事务支持 | 业务逻辑 | 中 |

#### 1.1.3 功能缺失（P1）

| 问题 ID | 问题描述 | 影响范围 | 优先级 |
|---------|----------|----------|--------|
| FEAT-001 | 缺少批量添加 Turn API | API | 高 |
| FEAT-002 | 缺少对话轮次自动识别 | 业务逻辑 | 高 |
| FEAT-003 | 缺少渐进式披露优化 | 用户体验 | 中 |
| FEAT-004 | 缺少会话归档功能 | 生命周期管理 | 中 |

#### 1.1.4 性能问题（P2）

| 问题 ID | 问题描述 | 影响范围 | 优先级 |
|---------|----------|----------|--------|
| PERF-001 | 内存索引未持久化 | 向量搜索 | 高 |
| PERF-002 | 向量搜索缺少分页 | 检索服务 | 中 |
| PERF-003 | 缺少并行批处理 | 内容处理 | 低 |
| PERF-004 | 嵌入模型预热延迟 | 搜索延迟 | 低 |

### 1.2 优先级排序

```
紧急修复 (P0)          核心架构 (P1)          功能增强 (P2)          性能优化 (P3)
─────────────────     ─────────────────     ─────────────────     ─────────────────
• turn_number 修复     • SessionService      • 批量添加 API         • 索引持久化
• expect() 移除        • TurnService         • 轮次识别             • 搜索分页
• Handler 业务逻辑     • 连接池重构          • 归档功能             • 并行处理
```

### 1.3 改进目标

#### 1.3.1 功能目标

- **完整性**: 实现完整的 CRUD 操作和业务逻辑
- **可靠性**: 消除 panic 风险，确保系统稳定运行
- **可扩展性**: 支持批量操作和高并发场景
- **可维护性**: 清晰的架构分层和职责划分

#### 1.3.2 性能目标

- **响应时间**: API 响应时间 < 100ms (P95)
- **吞吐量**: 支持 1000+ QPS
- **存储效率**: 内存索引支持持久化
- **检索质量**: 向量搜索准确率 > 90%

#### 1.3.3 质量目标

- **测试覆盖率**: > 80%
- **文档完整性**: API 文档 100% 覆盖
- **错误处理**: 所有异常情况都有明确处理

## 2. 解决方案

### 2.1 架构改进

#### 2.1.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Hippos Service                                 │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  ┌─────────┐ │
│  │   REST API  │  │   MCP API   │  │   Security Layer    │  │ Observ- │ │
│  │   (Axum)    │  │   (MCP)     │  │   (Auth/RateLimit)  │  │ ability │ │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────────┘  └─────────┘ │
│         │                │                                               │
│         └────────────────┼───────────────────────────────────────────────┘
│                          ▼                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     Application Layer                                 │ │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────────┐ │ │
│  │  │ Session   │ │ Turn      │ │Retrieval │ │ Dehydration Service   │ │ │
│  │  │ Service   │ │ Service   │ │ Service  │ │ (Context Compression) │ │ │
│  │  └───────────┘ └───────────┘ └───────────┘ └───────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                          │                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     Domain Layer                                      │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐  │ │
│  │  │ Session        │  │ Turn           │  │ Index Record           │  │ │
│  │  │ Aggregate      │  │ Aggregate      │  │ Entity                 │  │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                          │                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                     Infrastructure Layer                              │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐  │ │
│  │  │ SurrealDB   │  │ Vector      │  │ Cache Layer (Redis)         │  │ │
│  │  │ Pool        │  │ Index       │  │                             │  │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 2.1.2 服务层架构

```rust
// src/services/mod.rs

pub mod session;
pub mod turn;
pub mod retrieval;
pub mod dehydration;

pub use session::{SessionService, SessionServiceImpl, create_session_service};
pub use turn::{TurnService, TurnServiceImpl, create_turn_service};
pub use retrieval::{RetrievalService, RetrievalServiceImpl, create_retrieval_service};
pub use dehydration::{DehydrationService, create_dehydration_service};
```

#### 2.1.3 连接池设计

```rust
// src/storage/surrealdb.rs (改进版)

use std::sync::Arc;
use surrealdb::{Surreal, engine::any::Any, engine::any::connect, opt::auth::Root};
use tokio::sync::Mutex;
use deadpool::managed::{Pool, Object, Manager};

/// SurrealDB 连接池管理器
pub struct SurrealManager {
    config: DatabaseConfig,
}

#[async_trait::async_trait]
impl Manager for SurrealManager {
    type Type = Surreal<Any>;
    type Error = surrealdb::Error;

    async fn create(&self) -> Result<Surreal<Any>, Self::Error> {
        let db: Surreal<Any> = connect(&self.config.url).await?;
        
        db.signin(Root {
            username: &self.config.username,
            password: &self.config.password,
        })
        .await?;

        db.use_ns(&self.config.namespace)
            .use_db(&self.database)
            .await?;

        Ok(db)
    }

    async fn recycle(&self, conn: &mut Surreal<Any>) -> Result<(), Self::Error> {
        // 验证连接有效性
        Ok(())
    }
}

/// 连接池类型
pub type SurrealPool = Pool<SurrealManager>;

/// SurrealDB 连接池
#[derive(Clone)]
pub struct SurrealDbPool {
    pool: SurrealPool,
}

impl SurrealDbPool {
    pub async fn new(config: DatabaseConfig) -> Result<Self, surrealdb::Error> {
        let manager = SurrealManager { config: config.clone() };
        let pool = Pool::builder(manager)
            .max_size(config.max_connections)
            .min_size(config.min_connections)
            .timeout(std::time::Duration::from_secs(config.connection_timeout))
            .build()
            .map_err(|e| surrealdb::Error::Api(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn get(&self) -> Result<PooledConnection, surrealdb::Error> {
        self.pool.get().await.map_err(|e| surrealdb::Error::Api(e.to_string()))
    }

    pub async fn close(&self) {
        self.pool.close();
    }
}

/// 连接包装器
pub struct PooledConnection {
    conn: Object<SurrealManager>,
}

impl PooledConnection {
    pub async fn db(&self) -> &Surreal<Any> {
        &self.conn
    }
}
```

### 2.2 数据模型改进

#### 2.2.1 Session 模型增强

```rust
// src/models/session.rs (改进版)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Active,
    Paused,
    Archived,
    Deleted,
}

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionConfig {
    pub summary_limit: usize,
    pub index_refresh_interval: u64,
    pub semantic_search_enabled: bool,
    pub auto_summarize: bool,
    pub max_turns: usize,
    /// 保留策略：删除后保留天数
    pub retention_days: u32,
}

/// 会话统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionStats {
    pub total_turns: u64,
    pub total_tokens: u64,
    pub storage_size: u64,
    pub last_indexed_at: Option<DateTime<Utc>>,
    pub last_summarized_at: Option<DateTime<Utc>>,
}

/// 会话聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SessionHelper", into = "SessionHelper")]
pub struct Session {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub status: SessionStatus,
    pub config: SessionConfig,
    pub stats: SessionStats,
    pub metadata: HashMap<String, String>,
    /// 版本号（用于乐观锁）
    pub version: u64,
}

impl Session {
    pub fn new(tenant_id: &str, name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: format!("session_{}", Uuid::new_v4()),
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: now,
            last_active_at: now,
            status: SessionStatus::Active,
            config: SessionConfig::default(),
            stats: SessionStats::default(),
            metadata: HashMap::new(),
            version: 1,
        }
    }

    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    pub fn increment_turns(&mut self, tokens: u64) {
        self.stats.total_turns += 1;
        self.stats.total_tokens += tokens;
        self.touch();
    }

    pub fn can_add_turn(&self) -> bool {
        self.status == SessionStatus::Active &&
        (self.config.max_turns == 0 || self.stats.total_turns < self.config.max_turns as u64)
    }

    /// 归档会话
    pub fn archive(&mut self) {
        self.status = SessionStatus::Archived;
        self.touch();
    }

    /// 检查是否可以进行乐观锁更新
    pub fn can_update(&self, expected_version: u64) -> bool {
        self.version == expected_version
    }

    /// 增加版本号
    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}

/// 会话序列化辅助
#[derive(Serialize, Deserialize)]
struct SessionHelper {
    id: String,
    tenant_id: String,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
    status: SessionStatus,
    config: SessionConfig,
    stats: SessionStats,
    metadata: HashMap<String, String>,
    version: u64,
}

// ... 实现 From 转换
```

#### 2.2.2 Turn 模型增强

```rust
// src/models/turn.rs (改进版)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
}

/// 内容状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentStatus {
    Pending,
    Indexed,
    Archived,
    Processing,
    Failed,
}

/// Turn 元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TurnMetadata {
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub message_type: MessageType,
    pub role: Option<String>,
    pub model: Option<String>,
    pub token_count: Option<u64>,
    pub custom: HashMap<String, String>,
}

/// 脱水后的摘要信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DehydratedData {
    pub gist: String,
    pub topics: Vec<String>,
    pub tags: Vec<String>,
    pub embedding: Option<Vec<f32>>,
    pub generated_at: DateTime<Utc>,
    pub generator: Option<String>,
}

/// 对话轮次实体（聚合根）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "TurnHelper", into = "TurnHelper")]
pub struct Turn {
    pub id: String,
    pub session_id: String,
    pub turn_number: u64,
    pub raw_content: String,
    pub metadata: TurnMetadata,
    pub dehydrated: Option<DehydratedData>,
    pub status: ContentStatus,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    /// 版本号（用于乐观锁）
    pub version: u64,
}

impl Turn {
    pub fn new(session_id: &str, turn_number: u64, content: &str) -> Self {
        let now = Utc::now();
        Self {
            id: format!("turn_{}_{}", session_id, Uuid::new_v4()),
            session_id: session_id.to_string(),
            turn_number,
            raw_content: content.to_string(),
            metadata: TurnMetadata {
                timestamp: now,
                user_id: None,
                message_type: MessageType::User,
                role: None,
                model: None,
                token_count: None,
                custom: HashMap::new(),
            },
            dehydrated: None,
            status: ContentStatus::Pending,
            parent_id: None,
            children_ids: Vec::new(),
            version: 1,
        }
    }

    /// 创建助手回复轮次
    pub fn new_assistant(session_id: &str, turn_number: u64, content: &str, model: &str) -> Self {
        let mut turn = Self::new(session_id, turn_number, content);
        turn.metadata.message_type = MessageType::Assistant;
        turn.metadata.model = Some(model.to_string());
        turn
    }

    pub fn mark_indexed(&mut self) {
        self.status = ContentStatus::Indexed;
    }

    pub fn mark_processing(&mut self) {
        self.status = ContentStatus::Processing;
    }

    pub fn mark_failed(&mut self, error: &str) {
        self.status = ContentStatus::Failed;
        self.metadata.custom.insert("error".to_string(), error.to_string());
    }

    pub fn content_length(&self) -> usize {
        self.raw_content.len()
    }

    pub fn estimated_tokens(&self) -> u64 {
        // 使用更精确的估算方法
        (self.raw_content.len() / 4) as u64
    }

    /// 设置父轮次
    pub fn set_parent(&mut self, parent_id: &str) {
        self.parent_id = Some(parent_id.to_string());
    }

    /// 添加子轮次
    pub fn add_child(&mut self, child_id: &str) {
        self.children_ids.push(child_id.to_string());
    }
}

/// 轮次序列化辅助
#[derive(Serialize, Deserialize)]
struct TurnHelper {
    id: String,
    session_id: String,
    turn_number: u64,
    raw_content: String,
    metadata: TurnMetadata,
    dehydrated: Option<DehydratedData>,
    status: ContentStatus,
    parent_id: Option<String>,
    children_ids: Vec<String>,
    version: u64,
}

// ... 实现 From 转换
```

### 2.3 API 设计改进

#### 2.3.1 批量操作 API

```rust
// src/api/dto/turn_dto.rs (新增)

use serde::{Deserialize, Serialize};

/// 批量创建 Turn 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateTurnsRequest {
    pub turns: Vec<CreateTurnRequest>,
    /// 是否自动生成轮次号
    pub auto_turn_number: bool,
}

/// 批量创建 Turn 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateTurnsResponse {
    pub turns: Vec<CreateTurnResponse>,
    pub total_created: usize,
    pub total_failed: usize,
    pub errors: Vec<TurnCreationError>,
}

/// 单个轮次创建错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnCreationError {
    pub index: usize,
    pub message: String,
}

/// 批量更新请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateTurnsRequest {
    pub updates: Vec<UpdateTurnRequest>,
}

/// 批量删除响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDeleteTurnsResponse {
    pub deleted_ids: Vec<String>,
    pub total_deleted: usize,
    pub total_not_found: usize,
}
```

#### 2.3.2 会话归档 API

```rust
// src/api/dto/session_dto.rs (新增)

use serde::{Deserialize, Serialize};

/// 归档会话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSessionRequest {
    /// 归档原因
    pub reason: Option<String>,
    /// 是否同时归档关联的 Turn
    pub archive_turns: bool,
}

/// 归档会话响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSessionResponse {
    pub id: String,
    pub status: String,
    pub archived_at: chrono::DateTime<chrono::Utc>,
    pub message: String,
}

/// 恢复会话请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreSessionRequest {
    /// 恢复后的名称
    pub new_name: Option<String>,
}
```

### 2.4 性能优化

#### 2.4.1 索引持久化

```rust
// src/index/vector.rs (改进版)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::error::Result;
use surrealdb::{Surreal, engine::any::Any};

/// 向量索引 trait（增强版）
#[async_trait]
pub trait VectorIndex: Send + Sync {
    async fn add(&self, id: &str, vector: &[f32], metadata: VectorMetadata) -> Result<()>;
    async fn add_batch(&self, items: Vec<(String, Vec<f32>, VectorMetadata)>) -> Result<usize>;
    async fn search(&self, query: &[f32], session_id: &str, options: SearchOptions) -> Result<Vec<VectorSearchResult>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn delete_by_session(&self, session_id: &str) -> Result<u64>;
    async fn count(&self, session_id: &str) -> Result<u64>;
    async fn persist(&self, path: &Path) -> Result<()>;
    async fn load(&self, path: &Path) -> Result<()>;
}

/// 搜索选项
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub threshold: Option<f32>,
    pub filter: Option<HashMap<String, String>>,
}

/// 持久化向量索引
pub struct PersistentVectorIndex {
    memory_index: MemoryVectorIndex,
    db: Option<Surreal<Any>>,
    persist_path: Option<std::path::PathBuf>,
}

impl PersistentVectorIndex {
    pub fn new(dimension: usize, persist_path: Option<Path>) -> Self {
        Self {
            memory_index: MemoryVectorIndex::new(dimension),
            db: None,
            persist_path: persist_path.map(|p| p.to_path_buf()),
        }
    }

    pub fn with_db(mut self, db: Surreal<Any>) -> Self {
        self.db = Some(db);
        self
    }

    /// 定期持久化任务
    pub async fn start_persist_task(&self, interval: std::time::Duration) {
        let path = self.persist_path.clone();
        let index = self.memory_index.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                if let Some(p) = &path {
                    if let Err(e) = index.persist(p).await {
                        tracing::error!("Failed to persist vector index: {}", e);
                    }
                }
            }
        });
    }
}

#[async_trait]
impl VectorIndex for PersistentVectorIndex {
    async fn add(&self, id: &str, vector: &[f32], metadata: VectorMetadata) -> Result<()> {
        self.memory_index.add(id, vector, metadata).await?;
        
        // 如果有数据库，同时持久化到 DB
        if let Some(db) = &self.db {
            // 异步保存到 SurrealDB
        }
        
        Ok(())
    }

    async fn add_batch(&self, items: Vec<(String, Vec<f32>, VectorMetadata)>) -> Result<usize> {
        let mut count = 0;
        for (id, vector, metadata) in items {
            if self.add(&id, &vector, metadata).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn search(&self, query: &[f32], session_id: &str, options: SearchOptions) -> Result<Vec<VectorSearchResult>> {
        let results = self.memory_index.search(query, session_id, options.limit).await?;
        
        // 应用偏移量和阈值
        let mut results: Vec<_> = results.into_iter().skip(options.offset).collect();
        
        if let Some(threshold) = options.threshold {
            results.retain(|r| r.score >= threshold);
        }
        
        Ok(results)
    }

    // ... 其他方法实现

    async fn persist(&self, path: &Path) -> Result<()> {
        // 将内存索引序列化到文件
        let data = self.memory_index.export().await?;
        tokio::fs::write(path, serde_json::to_vec(&data)?).await?;
        Ok(())
    }

    async fn load(&self, path: &Path) -> Result<()> {
        if path.exists() {
            let data = tokio::fs::read(path).await?;
            self.memory_index.import(&serde_json::from_slice(&data)?).await?;
        }
        Ok(())
    }
}
```

#### 2.4.2 分页和流式查询

```rust
// src/storage/repository.rs (改进版)

use async_trait::async_trait;
use surrealdb::{Surreal, engine::any::Any, sql::Val};

use crate::error::{Result, AppError};
use crate::models::index_record::IndexRecord;
use crate::models::session::Session;
use crate::models::turn::Turn;

/// 分页参数
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: usize,
    pub page_size: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl Pagination {
    pub fn new(page: usize, page_size: usize) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        }
    }

    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }
}

/// Repository trait（增强版）
#[async_trait]
pub trait Repository<T: Clone + Send + Sync> {
    async fn create(&self, entity: &T) -> Result<T>;
    async fn get_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn update(&self, id: &str, entity: &T) -> Result<Option<T>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    
    /// 分页列出实体
    async fn list(&self, pagination: Pagination) -> Result<Vec<T>>;
    
    /// 统计数量
    async fn count(&self) -> Result<u64>;
    
    /// 根据条件统计
    async fn count_by(&self, field: &str, value: &str) -> Result<u64>;
    
    /// 批量获取
    async fn get_by_ids(&self, ids: &[String]) -> Result<Vec<T>>;
}

/// 会话仓储实现（改进版）
pub struct SessionRepository {
    db: Surreal<Any>,
}

impl SessionRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Repository<Session> for SessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let created: Option<Session> = self
            .db
            .create(("session", &session.id))
            .content(session)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        created.ok_or_else(|| AppError::Database("Failed to create session".to_string()))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        self.db
            .select(("session", id))
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(&self, id: &str, session: &Session) -> Result<Option<Session>> {
        self.db
            .update(("session", id))
            .content(session)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let result: Option<Session> = self
            .db
            .delete(("session", id))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.is_some())
    }

    async fn list(&self, pagination: Pagination) -> Result<Vec<Session>> {
        // 使用 SurrealDB 的 LIMIT 和 START 子句进行分页
        let mut statement = surrealdb::sql::Statement::from_string(
            format!("SELECT * FROM session LIMIT {} START {}", 
                pagination.page_size, pagination.offset())
        );
        
        let results: Vec<Session> = self.db.query(statement)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(results)
    }

    async fn count(&self) -> Result<u64> {
        // 使用 COUNT 聚合函数
        let result: Vec<serde_json::Value> = self.db
            .query("SELECT count() as count FROM session")
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0))
    }

    async fn count_by(&self, field: &str, value: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() as count FROM session WHERE {} = '{}'",
            field, value
        );
        
        let result: Vec<serde_json::Value> = self.db
            .query(query)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0))
    }

    async fn get_by_ids(&self, ids: &[String]) -> Result<Vec<Session>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str: Vec<String> = ids.iter()
            .map(|id| format!("'{}'", id))
            .collect();
        
        let query = format!(
            "SELECT * FROM session WHERE id IN [{}]",
            ids_str.join(", ")
        );
        
        self.db
            .query(query)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .take(0)
            .map_err(|e| AppError::Database(e.to_string()))
    }
}
```

## 3. 实施计划

### Phase 1: 核心修复（P0）

**目标**: 修复严重缺陷，确保系统稳定运行

#### 任务 1.1: 修复 turn_number 硬编码问题

**任务描述**: 实现动态 turn_number 生成逻辑

**验收标准**:
- [ ] 创建 Turn 时自动获取下一个轮次号
- [ ] 支持批量创建时正确分配轮次号
- [ ] 单元测试覆盖轮次号生成逻辑

**实现方案**:
```rust
// 在 TurnService 中实现
impl TurnServiceImpl {
    pub async fn get_next_turn_number(&self, session_id: &str) -> Result<u64> {
        // 查询会话中最大的 turn_number
        let max_turn = self.turn_repository
            .get_max_turn_number(session_id)
            .await?;
        
        Ok(max_turn + 1)
    }
}
```

#### 任务 1.2: 移除 Repository 层 expect() 调用

**任务描述**: 使用 Result 处理代替 expect()，避免 panic

**验收标准**:
- [ ] 所有 Repository 方法返回 Result<T, AppError>
- [ ] Handler 层正确处理错误情况
- [ ] 错误信息包含足够的上下文

**实现方案**:
```rust
// 改进 Repository create 方法
async fn create(&self, session: &Session) -> Result<Session> {
    let created: Option<Session> = self
        .db
        .create(("session", &session.id))
        .content(session)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    created.ok_or_else(|| AppError::Database(
        format!("Failed to create session: {}", session.id)
    ))
}
```

#### 任务 1.3: 实现 Handler 层业务逻辑

**任务描述**: 完成所有 Handler 的业务逻辑实现

**验收标准**:
- [ ] Session Handler: 完整 CRUD + 归档功能
- [ ] Turn Handler: 完整 CRUD + 批量操作
- [ ] Search Handler: 混合搜索功能
- [ ] 集成测试覆盖所有 API 端点

**实现方案**:
```rust
// src/api/handlers/turn_handler.rs (改进版)

pub async fn create_turn(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<CreateTurnRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating turn for session: {}", session_id);

    // 验证会话存在
    let session = state.session_service
        .get_by_id(&session_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    // 检查是否可以添加轮次
    if !session.can_add_turn() {
        return Err(AppError::Validation(
            "Session has reached maximum turn limit".to_string()
        ));
    }

    // 验证输入
    if request.content.is_empty() {
        return Err(AppError::Validation("Content cannot be empty".to_string()));
    }

    // 获取下一个轮次号
    let turn_number = state.turn_service
        .get_next_turn_number(&session_id)
        .await?;

    // 创建 Turn
    let turn = state.turn_service
        .create(&session_id, turn_number, &request.content, request.metadata)
        .await?;

    let response = CreateTurnResponse {
        id: turn.id,
        turn_number: turn.turn_number,
        created_at: turn.metadata.timestamp,
        message_count: 1,
        token_count: turn.metadata.token_count.unwrap_or(0),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
```

### Phase 2: 架构完善（P1）

**目标**: 建立完整的服务层架构

#### 任务 2.1: 实现 SessionService

**任务描述**: 创建 Session 业务逻辑服务

**验收标准**:
- [ ] 完整的会话生命周期管理
- [ ] 支持乐观锁并发控制
- [ ] 会话统计信息自动更新

**实现方案**:
```rust
// src/services/session/mod.rs

use async_trait::async_trait;
use crate::error::{Result, AppError};
use crate::models::session::{Session, SessionStatus};
use crate::storage::repository::{Repository, Pagination};

/// Session Service Trait
#[async_trait]
pub trait SessionService: Send + Sync {
    async fn create(&self, tenant_id: &str, name: &str) -> Result<Session>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Session>>;
    async fn update(&self, session: &Session) -> Result<Session>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn list(&self, tenant_id: &str, pagination: Pagination) -> Result<Vec<Session>>;
    async fn count(&self, tenant_id: &str) -> Result<u64>;
    async fn archive(&self, id: &str, reason: Option<String>) -> Result<Session>;
    async fn restore(&self, id: &str, new_name: Option<String>) -> Result<Session>;
}

/// Session Service 实现
pub struct SessionServiceImpl {
    session_repository: Arc<dyn Repository<Session>>,
    // 其他依赖...
}

#[async_trait]
impl SessionService for SessionServiceImpl {
    async fn create(&self, tenant_id: &str, name: &str) -> Result<Session> {
        let session = Session::new(tenant_id, name);
        self.session_repository.create(&session).await
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        self.session_repository.get_by_id(id).await
    }

    async fn update(&self, session: &Session) -> Result<Session> {
        // 乐观锁检查
        let existing = self.get_by_id(&session.id).await?
            .ok_or_else(|| AppError::NotFound(session.id.clone()))?;

        if !session.can_update(existing.version) {
            return Err(AppError::Concurrency(
                "Session was modified by another request".to_string()
            ));
        }

        let mut session = session.clone();
        session.increment_version();

        self.session_repository.update(&session.id, &session)
            .await?
            .ok_or_else(|| AppError::Database("Update failed".to_string()))
    }

    async fn archive(&self, id: &str, reason: Option<String>) -> Result<Session> {
        let mut session = self.get_by_id(id)?
            .ok_or_else(|| AppError::NotFound(id.to_string()))?;

        if session.status == SessionStatus::Archived {
            return Err(AppError::Validation("Session already archived".to_string()));
        }

        session.archive();
        session.metadata.insert("archive_reason".to_string(), reason.unwrap_or_default());

        self.update(&session).await
    }

    // ... 其他方法
}
```

#### 任务 2.2: 实现 TurnService

**任务描述**: 创建 Turn 业务逻辑服务

**验收标准**:
- [ ] 完整的轮次生命周期管理
- [ ] 自动关联会话和轮次关系
- [ ] 支持批量操作和事务

**实现方案**:
```rust
// src/services/turn/mod.rs

use async_trait::async_trait;
use crate::error::{Result, AppError};
use crate::models::turn::{Turn, TurnMetadata, ContentStatus};
use crate::storage::repository::{Repository, Pagination};

/// Turn Service Trait
#[async_trait]
pub trait TurnService: Send + Sync {
    async fn create(&self, session_id: &str, turn_number: u64, content: &str, metadata: Option<TurnMetadata>) -> Result<Turn>;
    async fn create_batch(&self, session_id: &str, turns: Vec<(&str, u64, &str)>) -> Result<Vec<Turn>>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>>;
    async fn list_by_session(&self, session_id: &str, pagination: Pagination) -> Result<Vec<Turn>>;
    async fn count_by_session(&self, session_id: &str) -> Result<u64>;
    async fn get_next_turn_number(&self, session_id: &str) -> Result<u64>;
    async fn update(&self, turn: &Turn) -> Result<Turn>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn delete_by_session(&self, session_id: &str) -> Result<u64>;
}

/// Turn Service 实现
pub struct TurnServiceImpl {
    turn_repository: Arc<dyn Repository<Turn>>,
    session_repository: Arc<dyn Repository<Session>>,
    // 其他依赖...
}

#[async_trait]
impl TurnService for TurnServiceImpl {
    async fn create(&self, session_id: &str, turn_number: u64, content: &str, metadata: Option<TurnMetadata>) -> Result<Turn> {
        // 验证会话存在
        let session = self.session_repository.get_by_id(session_id).await?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

        if !session.can_add_turn() {
            return Err(AppError::Validation("Session has reached maximum turn limit".to_string()));
        }

        let mut turn = Turn::new(session_id, turn_number, content);
        if let Some(metadata) = metadata {
            turn.metadata = metadata;
        }

        let turn = self.turn_repository.create(&turn).await?;

        // 更新会话统计
        let mut session = session.clone();
        session.increment_turns(turn.estimated_tokens());
        self.session_repository.update(&session.id, &session).await?;

        Ok(turn)
    }

    async fn create_batch(&self, session_id: &str, turns: Vec<(&str, u64, &str)>) -> Result<Vec<Turn>> {
        let mut results = Vec::new();
        let mut failed = Vec::new();

        for (index, (content, turn_number, _)) in turns.iter().enumerate() {
            match self.create(session_id, *turn_number, content, None).await {
                Ok(turn) => results.push(turn),
                Err(e) => failed.push((index, e.to_string())),
            }
        }

        if !failed.is_empty() {
            // 返回部分成功的结果和错误信息
        }

        Ok(results)
    }

    async fn get_next_turn_number(&self, session_id: &str) -> Result<u64> {
        // 查询当前会话中最大的 turn_number
        let max_turn = self.turn_repository
            .get_max_turn_number(session_id)
            .await?;

        Ok(max_turn + 1)
    }

    // ... 其他方法
}
```

#### 任务 2.3: 重构连接池

**任务描述**: 使用连接池代替单连接复用

**验收标准**:
- [ ] 支持多连接并发访问
- [ ] 连接超时和回收机制
- [ ] 连接池配置可调整

#### 任务 2.4: 修复全表加载问题

**任务描述**: 实现真正的分页查询

**验收标准**:
- [ ] list() 方法使用 LIMIT/OFFSET
- [ ] count() 方法使用聚合查询
- [ ] 支持条件过滤

### Phase 3: 功能增强（P2）

**目标**: 添加缺失的功能特性

#### 任务 3.1: 批量添加 API

**任务描述**: 实现批量 Turn 创建和查询

**验收标准**:
- [ ] 支持批量创建 Turn
- [ ] 支持部分成功和错误报告
- [ ] 批量操作事务支持

#### 任务 3.2: 对话轮次识别

**任务描述**: 实现智能轮次识别

**验收标准**:
- [ ] 基于消息角色自动识别轮次
- [ ] 支持多轮对话链
- [ ] 自动建立父子关系

#### 任务 3.3: 会话归档功能

**任务描述**: 实现会话归档和恢复

**验收标准**:
- [ ] 支持归档已完成的会话
- [ ] 归档后不计入活跃统计
- [ ] 支持恢复归档的会话

### Phase 4: 性能优化（P3）

**目标**: 优化系统性能

#### 任务 4.1: 索引持久化

**任务描述**: 实现向量索引持久化

**验收标准**:
- [ ] 定期保存索引到磁盘
- [ ] 启动时加载已有索引
- [ ] 支持索引重建

#### 任务 4.2: 搜索分页优化

**任务描述**: 实现高效的搜索分页

**验收标准**:
- [ ] 支持游标分页
- [ ] 分页性能优化
- [ ] 支持深度分页

#### 任务 4.3: 并行批处理

**任务描述**: 实现并行内容处理

**验收标准**:
- [ ] 批量嵌入计算并行化
- [ ] 摘要生成并行化
- [ ] 资源使用率优化

## 4. 技术规格

### 4.1 接口定义

#### 4.1.1 Session Service 接口

```rust
/// Session Service 接口
#[async_trait]
pub trait SessionService: Send + Sync {
    /// 创建会话
    async fn create(&self, tenant_id: &str, name: &str, description: Option<&str>) -> Result<Session>;
    
    /// 获取会话
    async fn get_by_id(&self, id: &str) -> Result<Option<Session>>;
    
    /// 更新会话
    async fn update(&self, session: &Session) -> Result<Session>;
    
    /// 删除会话
    async fn delete(&self, id: &str) -> Result<bool>;
    
    /// 列出会话（分页）
    async fn list(&self, tenant_id: &str, pagination: Pagination) -> Result<Vec<Session>>;
    
    /// 统计会话数量
    async fn count(&self, tenant_id: &str) -> Result<u64>;
    
    /// 归档会话
    async fn archive(&self, id: &str, reason: Option<String>) -> Result<Session>;
    
    /// 恢复会话
    async fn restore(&self, id: &str, new_name: Option<String>) -> Result<Session>;
}
```

#### 4.1.2 Turn Service 接口

```rust
/// Turn Service 接口
#[async_trait]
pub trait TurnService: Send + Sync {
    /// 创建轮次
    async fn create(&self, session_id: &str, content: &str, metadata: Option<TurnMetadata>) -> Result<Turn>;
    
    /// 批量创建轮次
    async fn create_batch(&self, session_id: &str, contents: Vec<&str>) -> Result<BatchCreateResult>;
    
    /// 获取轮次
    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>>;
    
    /// 列出会话轮次（分页）
    async fn list_by_session(&self, session_id: &str, pagination: Pagination) -> Result<Vec<Turn>>;
    
    /// 获取下一个轮次号
    async fn get_next_turn_number(&self, session_id: &str) -> Result<u64>;
    
    /// 更新轮次
    async fn update(&self, turn: &Turn) -> Result<Turn>;
    
    /// 删除轮次
    async fn delete(&self, id: &str) -> Result<bool>;
}

/// 批量创建结果
#[derive(Debug)]
pub struct BatchCreateResult {
    pub turns: Vec<Turn>,
    pub total_created: usize,
    pub total_failed: usize,
    pub errors: Vec<(usize, String)>,
}
```

### 4.2 数据结构

#### 4.2.1 分页结构

```rust
/// 分页参数
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    /// 页码（从 1 开始）
    pub page: usize,
    /// 每页数量
    pub page_size: usize,
}

impl Pagination {
    pub fn new(page: usize, page_size: usize) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.clamp(1, 100),
        }
    }

    /// 获取偏移量
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    /// 检查是否有下一页
    pub fn has_next(&self, total: u64) -> bool {
        self.page * self.page_size < total as usize
    }
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, pagination: &Pagination) -> Self {
        let total_pages = (total as usize + pagination.page_size - 1) / pagination.page_size;
        
        Self {
            items,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages,
        }
    }
}
```

#### 4.2.2 错误结构

```rust
/// 错误代码
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    // 客户端错误 (4xx)
    ValidationFailed,
    NotFound,
    Unauthorized,
    Forbidden,
    RateLimited,
    
    // 服务端错误 (5xx)
    InternalError,
    DatabaseError,
    VectorIndexError,
    EmbeddingError,
    Timeout,
}

/// API 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
```

### 4.3 错误处理

#### 4.3.1 错误类型定义

```rust
/// 应用程序错误类型
#[derive(Error, Debug, thiserror::Error)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(String),
    
    #[error("连接错误: {0}")]
    Connection(String),
    
    #[error("认证失败: {0}")]
    Authentication(String),
    
    #[error("未授权访问: {0}")]
    Authorization(String),
    
    #[error("资源不存在: {0}")]
    NotFound(String),
    
    #[error("参数验证失败: {0}")]
    Validation(String),
    
    #[error("并发冲突: {0}")]
    Concurrency(String),
    
    #[error("请求过于频繁，请稍后再试")]
    RateLimited,
    
    #[error("操作超时: {0}")]
    Timeout(String),
    
    #[error("配置错误: {0}")]
    Config(String),
    
    #[error("序列化错误: {0}")]
    Serialization(String),
    
    #[error("向量索引错误: {0}")]
    VectorIndex(String),
    
    #[error("嵌入模型错误: {0}")]
    Embedding(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
    
    #[error("IO 错误: {0}")]
    Io(String),
}

/// 错误转换
impl From<surrealdb::Error> for AppError {
    fn from(e: surrealdb::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}
```

#### 4.3.2 错误处理中间件

```rust
/// 错误处理中间件
pub async fn error_handler_layer(
    response: Response,
    request_id: &str,
) -> Response {
    if let Some(error) = response.extensions().get::<AppError>() {
        let (status, code) = error.to_status_and_code();
        
        let error_response = ApiError::new(code, error.to_string())
            .with_details(serde_json::json!({
                "request_id": request_id
            }));
        
        return (status, Json(error_response)).into_response();
    }
    
    response
}

trait ToStatusAndCode {
    fn to_status_and_code(&self) -> (StatusCode, ErrorCode);
}

impl ToStatusAndCode for AppError {
    fn to_status_and_code(&self) -> (StatusCode, ErrorCode) {
        match self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, ErrorCode::NotFound),
            AppError::Authentication(_) => (StatusCode::UNAUTHORIZED, ErrorCode::Unauthorized),
            AppError::Authorization(_) => (StatusCode::FORBIDDEN, ErrorCode::Forbidden),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, ErrorCode::ValidationFailed),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, ErrorCode::RateLimited),
            AppError::Timeout(_) => (StatusCode::REQUEST_TIMEOUT, ErrorCode::Timeout),
            AppError::Database(_) | AppError::VectorIndex(_) | AppError::Embedding(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError)
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, ErrorCode::InternalError),
        }
    }
}
```

### 4.4 测试策略

#### 4.4.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::repository::Repository;
    use crate::storage::surrealdb::SurrealPool;

    /// SessionService 单元测试
    mod session_service_tests {
        use super::*;

        #[tokio::test]
        async fn test_create_session() {
            // Arrange
            let mock_repo = MockRepository::new();
            let service = SessionServiceImpl::new(Arc::new(mock_repo));
            
            // Act
            let session = service.create("tenant_1", "Test Session", None).await.unwrap();
            
            // Assert
            assert_eq!(session.tenant_id, "tenant_1");
            assert_eq!(session.name, "Test Session");
            assert!(session.id.starts_with("session_"));
        }

        #[tokio::test]
        async fn test_update_session_with_optimistic_lock() {
            // Arrange
            let mock_repo = MockRepository::new();
            let service = SessionServiceImpl::new(Arc::new(mock_repo));
            
            let session = service.create("tenant_1", "Test", None).await.unwrap();
            let mut updated_session = session.clone();
            updated_session.name = "Updated Name";
            
            // 模拟其他请求更新了会话
            mock_repo.set_version(&session.id, 2);
            
            // Act & Assert
            assert!(service.update(&updated_session).await.is_err());
        }
    }

    /// TurnService 单元测试
    mod turn_service_tests {
        use super::*;

        #[tokio::test]
        async fn test_get_next_turn_number() {
            // Arrange
            let mock_turn_repo = MockRepository::new();
            let mock_session_repo = MockRepository::new();
            
            let service = TurnServiceImpl::new(
                Arc::new(mock_turn_repo),
                Arc::new(mock_session_repo),
            );
            
            // 模拟已有轮次
            mock_turn_repo.set_max_turn_number("session_1", 5);
            
            // Act
            let next_number = service.get_next_turn_number("session_1").await.unwrap();
            
            // Assert
            assert_eq!(next_number, 6);
        }

        #[tokio::test]
        async fn test_create_batch_turns() {
            // Arrange
            let service = TurnServiceImpl::new(
                Arc::new(MockRepository::new()),
                Arc::new(MockRepository::new()),
            );
            
            // Act
            let result = service.create_batch(
                "session_1",
                vec!["Content 1", "Content 2", "Content 3"]
            ).await.unwrap();
            
            // Assert
            assert_eq!(result.total_created, 3);
            assert_eq!(result.turns.len(), 3);
        }
    }
}
```

#### 4.4.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use surrealdb::engine::any::connect;
    use surrealdb::opt::auth::Root;

    /// 完整 API 测试
    mod api_tests {
        use super::*;
        use axum_test::TestServer;

        #[tokio::test]
        async fn test_full_turn_lifecycle() {
            // Arrange - 启动测试服务器
            let pool = create_test_pool().await;
            let app = create_app(pool).await;
            let client = TestServer::new(app).unwrap();
            
            // Act 1 - 创建会话
            let session_response = client
                .post("/api/v1/sessions")
                .json(&serde_json::json!({
                    "name": "Test Session",
                    "description": "Integration Test"
                }))
                .await;
            
            assert_eq!(session_response.status_code, StatusCode::CREATED);
            let session: SessionResponse = session_response.json();
            
            // Act 2 - 添加轮次
            let turn_response = client
                .post(&format!("/api/v1/sessions/{}/turns", session.id))
                .json(&serde_json::json!({
                    "role": "user",
                    "content": "Hello, this is a test message"
                }))
                .await;
            
            assert_eq!(turn_response.status_code, StatusCode::CREATED);
            let turn: TurnResponse = turn_response.json();
            
            // Assert - 验证轮次号正确
            assert_eq!(turn.turn_number, 1);
            
            // Act 3 - 再次添加轮次
            let turn2_response = client
                .post(&format!("/api/v1/sessions/{}/turns", session.id))
                .json(&serde_json::json!({
                    "role": "assistant",
                    "content": "Hello! How can I help you?"
                }))
                .await;
            
            let turn2: TurnResponse = turn2_response.json();
            assert_eq!(turn2.turn_number, 2);
            
            // Act 4 - 查询会话轮次
            let list_response = client
                .get(&format!("/api/v1/sessions/{}/turns", session.id))
                .query(&[("page", "1"), ("page_size", "10")])
                .await;
            
            let turns: PaginatedTurnsResponse = list_response.json();
            assert_eq!(turns.items.len(), 2);
        }
    }
}
```

#### 4.4.3 性能测试

```rust
/// 性能测试配置
#[cfg(test)]
mod performance_tests {
    use criterion::{black_box, Criterion};
    use tokio::time::Instant;

    fn bench_turn_creation(c: &mut Criterion) {
        c.bench_function("create_turn", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let service = create_test_service();
                    black_box(
                        service.create(
                            "session_1",
                            "Test content",
                            None,
                        ).await
                    )
                })
        });
    }

    fn bench_vector_search(c: &mut Criterion) {
        c.bench_function("vector_search_1000_items", |b| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    let index = create_test_index(1000).await;
                    black_box(
                        index.search(&[0.5; 384], "session_1", 10).await
                    )
                })
        });
    }
}
```

## 5. 验收标准

### 5.1 功能验收标准

| 功能模块 | 验收标准 | 测试方法 |
|---------|----------|----------|
| Session CRUD | 所有 API 端点返回正确响应 | 集成测试 |
| Turn CRUD | 轮次号自动递增 | 单元测试 + 集成测试 |
| 批量操作 | 批量创建正确处理 | 集成测试 |
| 归档功能 | 归档后状态正确更新 | 集成测试 |
| 搜索功能 | 返回准确结果 | 集成测试 |

### 5.2 性能验收标准

| 指标 | 目标值 | 测试方法 |
|------|--------|----------|
| API 响应时间 (P95) | < 100ms | 性能测试 |
| QPS | > 1000 | 负载测试 |
| 搜索延迟 | < 50ms | 性能测试 |
| 并发连接数 | > 100 | 压力测试 |

### 5.3 质量验收标准

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| 测试覆盖率 | > 80% | cargo tarpaulin |
| 文档覆盖 | 100% API | 代码审查 |
| 错误处理 | 全部覆盖 | 代码审查 |
| 内存泄漏 | 0 | 长期运行测试 |

### 5.4 安全验收标准

| 检查项 | 验收标准 |
|--------|----------|
| 认证 | API Key 和 JWT 认证正常工作 |
| 授权 | RBAC 权限控制生效 |
| 限流 | RateLimiter 正确限制请求 |
| 输入验证 | 所有输入经过验证 |
| SQL 注入防护 | 使用参数化查询 |

## 6. 实施路线图

### 6.1 时间规划

```
Week 1-2: Phase 1 核心修复
├── Day 1-3: 修复 turn_number 问题
├── Day 4-7: 移除 expect() 调用
└── Day 8-10: 实现 Handler 业务逻辑

Week 3-4: Phase 2 架构完善
├── Day 1-4: 实现 SessionService
├── Day 5-8: 实现 TurnService
├── Day 9-14: 重构连接池和分页

Week 5-6: Phase 3 功能增强
├── Day 1-3: 批量添加 API
├── Day 4-6: 轮次识别
└── Day 7-10: 归档功能

Week 7-8: Phase 4 性能优化
├── Day 1-4: 索引持久化
├── Day 5-6: 搜索分页
└── Day 7-8: 测试和优化
```

### 6.2 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 迁移风险 | 高 | 渐进式迁移，保持向后兼容 |
| 性能回归 | 中 | 性能测试覆盖，基准测试 |
| 数据一致性问题 | 高 | 事务支持，乐观锁 |
| 测试覆盖不足 | 中 | 增加自动化测试 |

## 7. 附录

### 7.1 术语表

| 术语 | 定义 |
|------|------|
| Session | 对话会话，承载一组相关对话的上下文 |
| Turn | 对话轮次，单次用户-助手交互 |
| Dehydration | 脱水，生成对话摘要的过程 |
| Vector Index | 向量索引，用于语义搜索的数据结构 |
| RRF | Reciprocal Rank Fusion，混合搜索融合策略 |

### 7.2 参考资料

- [SurrealDB 文档](https://surrealdb.com/docs)
- [Axum 框架文档](https://docs.rs/axum/latest/axum/)
- [Rust 最佳实践](https://rust-lang.github.io/api-guidelines/)
- [Hippos 项目 README](/README.md)

---

**文档版本**: 1.0  
**创建日期**: 2024-01  
**维护者**: Hippos Team
