# Surrealdb → ArangoDB 迁移计划

> 版本: 1.1  
> 创建日期: 2025-01-14  
> 更新日期: 2025-01-15  
> 预计工期: 6-10 周

---

## 📊 迁移进度

| 阶段 | 任务 | 状态 | 完成度 |
|------|------|------|--------|
| 阶段 1 | 技术选型确认 | ✅ 已完成 | 100% |
| 阶段 1 | 详细数据模型设计 | ✅ 已完成 | 100% |
| 阶段 1 | 迁移工具开发 | ✅ 已完成 | 100% |
| 阶段 2 | ArangoDB 驱动集成 | ✅ 已完成 | 100% |
| 阶段 2 | Repository 层重写 | ✅ 已完成 | 100% |
| 阶段 2 | 配置和错误处理 | ✅ 已完成 | 100% |
| 阶段 3 | 数据导出工具 | ✅ 已完成 | 100% |
| 阶段 3 | 数据转换脚本 | ✅ 已完成 | 100% |
| 阶段 3 | 数据导入工具 | ✅ 已完成 | 100% |
| 阶段 4 | 单元测试 | 🔄 进行中 | 60% |
| 阶段 4 | 集成测试 | ⏳ 待开始 | 0% |
| 阶段 4 | 性能测试 | ⏳ 待开始 | 0% |
| 阶段 5 | 生产环境部署 | ⏳ 待开始 | 0% |
| 阶段 5 | 数据迁移执行 | ⏳ 待开始 | 0% |
| 阶段 5 | 流量切换和验证 | ⏳ 待开始 | 0% |

**总体进度: 7/15 任务完成 (47%)**

---

## 目录

1. [概述](#1-概述)
2. [当前架构分析](#2-当前架构分析)
3. [目标架构设计](#3-目标架构设计)
4. [迁移阶段规划](#4-迁移阶段规划)
5. [数据模型映射](#5-数据模型映射)
6. [查询转换规则](#6-查询转换规则)
7. [代码修改清单](#7-代码修改清单)
8. [测试策略](#8-测试策略)
9. [部署方案](#9-部署方案)
10. [风险评估](#10-风险评估)

---

## 1. 概述

### 1.1 迁移目标

将 Hippos 项目的数据库从 **Surrealdb 2.0.0** 迁移至 **ArangoDB 3.11+**，保持现有功能完整性，优化性能和可维护性。

### 1.2 迁移原则

- ✅ **渐进式迁移**: 分阶段进行，降低风险
- ✅ **功能等价**: 迁移后功能 100% 等价
- ✅ **数据完整性**: 确保数据无损迁移
- ✅ **零停机**: 尽可能支持双写和滚动切换
- ✅ **可回滚**: 每个阶段可回滚到之前状态

### 1.3 关键约束

| 约束 | 说明 |
|------|------|
| 技术栈 | Rust 2024 Edition, Tokio 异步运行时 |
| 驱动选择 | 社区活跃的 ArangoDB Rust 驱动 |
| 查询语言 | AQL (ArangoDB Query Language) |
| 兼容性 | 保持现有 API 接口不变 |

---

## 2. 当前架构分析

### 2.1 Surrealdb 使用概况

```
┌─────────────────────────────────────────────────────────────┐
│                    当前架构 (Surrealdb)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐                                            │
│  │   应用层    │  REST API (Axum)                           │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │  Services   │  SessionService, TurnService, Retrieval    │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │ Repository  │  Repository<Session>, Repository<Turn>     │
│  │   Layer     │  Repository<IndexRecord>                   │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │SurrealPool  │  连接池管理 (Arc<Mutex<Option<Surreal<Any>>>>│
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │ HTTP Client │  reqwest 发送 SQL 到 /sql 端点             │
│  └─────────────┘                                            │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心数据模型

#### Session (会话)
```rust
struct Session {
    id: String,              // 记录ID: session:⟨uuid⟩
    tenant_id: String,       // 租户隔离
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
    status: String,          // Active, Paused, Archived, Deleted
    config: SessionConfig,
    stats: SessionStats,
    metadata: HashMap<String, String>,
}
```

#### Turn (对话轮次)
```rust
struct Turn {
    id: String,              // 记录ID: turn:⟨uuid⟩
    session_id: String,      // 外键关联
    turn_number: u64,
    raw_content: String,
    metadata: TurnMetadata,
    dehydrated: Option<DehydratedData>,
    status: ContentStatus,
    parent_id: Option<String>,
    children_ids: Vec<String>,
}
```

#### IndexRecord (索引记录)
```rust
struct IndexRecord {
    turn_id: String,         // 外键关联
    session_id: String,
    tenant_id: String,
    gist: String,
    topics: Vec<String>,
    tags: Vec<String>,
    timestamp: DateTime<Utc>,
    vector_id: String,
    relevance_score: Option<f32>,
    turn_number: u64,
}
```

### 2.3 Surrealdb 特性使用

| 特性 | 使用场景 | 迁移影响 |
|------|----------|----------|
| Namespace 隔离 | 租户/环境隔离 | 可用 Database 替代 |
| HTTP REST API | 所有 CRUD 操作 | 需切换到 ArangoDB HTTP API |
| SurrealQL | 查询语言 | 需重写为 AQL |
| 记录链接 | Turn ↔ Session 关联 | 需转换为边集合 |
| 自定义反序列化 | ID 格式处理 | 需适配 ArangoDB _key 格式 |

---

## 3. 目标架构设计

### 3.1 ArangoDB 架构

```
┌─────────────────────────────────────────────────────────────┐
│                    目标架构 (ArangoDB)                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐                                            │
│  │   应用层    │  REST API (Axum) - 保持不变                 │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │  Services   │  - 保持不变                                 │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │ Repository  │  Repository<T> - 适配新驱动                │
│  │   Layer     │                                            │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │ArangoPool  │  连接池管理 (arango-rs 驱动)                │
│  └──────┬──────┘                                            │
│         │                                                    │
│  ┌──────▼──────┐                                            │
│  │AQL Client   │  AQL 查询构建器和执行器                    │
│  └─────────────┘                                            │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 推荐的 Rust 驱动

**Primary**: `arango-rs` (社区活跃，异步支持)  
**备选**: `arangors` (功能完整，但异步支持较弱)

#### 驱动特性对比

| 特性 | arango-rs | arangors |
|------|-----------|----------|
| 异步支持 | ✅ tokio | ✅ tokio (实验性) |
| 连接池 | ✅ 内置 | ❌ 需自行实现 |
| AQL 构建器 | ✅ 链式 API | ✅ 简单封装 |
| 文档序列化 | ✅ serde 集成 | ✅ serde 集成 |
| 社区活跃度 | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| 最新更新 | 2024 | 2023 |

### 3.3 集合设计

#### Document Collections (文档集合)

**sessions** 集合:
```json
{
  "_key": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "tenant_001",
  "name": "My Session",
  "description": "Session description",
  "created_at": "2024-01-15T10:30:00Z",
  "last_active_at": "2024-01-15T11:00:00Z",
  "status": "active",
  "config": { /* SessionConfig */ },
  "stats": { /* SessionStats */ },
  "metadata": { /* HashMap */ }
}
```

**turns** 集合:
```json
{
  "_key": "turn_001",
  "session_key": "550e8400-e29b-41d4-a716-446655440000",
  "turn_number": 1,
  "raw_content": "Hello, world!",
  "metadata": { /* TurnMetadata */ },
  "dehydrated": { /* DehydratedData */ },
  "status": "indexed",
  "parent_key": "turn_parent",
  "children_keys": ["turn_child1", "turn_child2"]
}
```

**index_records** 集合:
```json
{
  "_key": "idx_001",
  "turn_key": "turn_001",
  "session_key": "session_001",
  "tenant_id": "tenant_001",
  "gist": "Discussion about...",
  "topics": ["rust", "database"],
  "tags": ["important", "review"],
  "timestamp": "2024-01-15T11:00:00Z",
  "vector_id": "vec_001",
  "relevance_score": 0.95,
  "turn_number": 1
}
```

#### Edge Collections (边集合)

**session_turns** 边集合 (Session → Turn):
```json
{
  "_key": "edge_st_001",
  "_from": "sessions/550e8400-e29b-41d4-a716-446655440000",
  "_to": "turns/turn_001",
  "turn_number": 1
}
```

**turn_parents** 边集合 (Turn → Turn):
```json
{
  "_key": "edge_tp_001",
  "_from": "turns/turn_parent",
  "_to": "turns/turn_child",
  "relationship": "parent"
}
```

**turn_index_records** 边集合 (Turn → IndexRecord):
```json
{
  "_key": "edge_ti_001",
  "_from": "turns/turn_001",
  "_to": "index_records/idx_001",
  "indexed_at": "2024-01-15T11:00:00Z"
}
```

### 3.4 索引设计

| 集合 | 索引类型 | 字段 | 用途 |
|------|----------|------|------|
| sessions | hash | tenant_id | 多租户过滤 |
| sessions | hash | status | 状态过滤 |
| sessions | skipline | created_at | 时间排序 |
| turns | hash | session_key | 会话查询 |
| turns | skipline | turn_number | 排序 |
| turns | hash | parent_key | 父子查询 |
| index_records | hash | session_key | 会话过滤 |
| index_records | hash | tenant_id | 租户过滤 |
| index_records | skipline | timestamp | 时间排序 |
| index_records | hash | vector_id | 向量关联 |

---

## 4. 迁移阶段规划

### 4.1 阶段概览

```
阶段 1: 准备与设计 (1 周)
    ├── 技术选型确认
    ├── 详细数据模型设计
    └── 迁移工具开发
    
阶段 2: 核心层开发 (2-3 周)
    ├── ArangoDB 驱动集成
    ├── 连接池实现
    ├── Repository 层重写
    └── 配置和错误处理
    
阶段 3: 数据迁移 (1 周)
    ├── 数据导出工具
    ├── 数据转换脚本
    └── 数据导入验证
    
阶段 4: 测试与优化 (1-2 周)
    ├── 单元测试
    ├── 集成测试
    └── 性能测试
    
阶段 5: 部署与切换 (1 周)
    ├── 生产环境部署
    ├── 数据迁移执行
    └── 流量切换和验证
```

### 4.2 详细阶段任务

#### 阶段 1: 准备与设计 (Week 1)

**任务 1.1: 技术选型确认**
- [x] 评估并选择 ArangoDB Rust 驱动 (使用直接 HTTP API)
- [x] 创建概念验证 (POC) 项目
- [x] 验证驱动功能和性能

**任务 1.2: 详细数据模型设计**
- [x] 设计 ArangoDB 集合结构
- [x] 设计边集合和关系
- [x] 设计索引策略
- [x] 评审并确认设计

**任务 1.3: 迁移工具开发**
- [x] 开发 Surrealdb 数据导出工具 (export.rs)
- [x] 开发数据转换脚本 (transform.rs)
- [x] 开发 ArangoDB 数据导入工具 (import.rs)

#### 阶段 2: 核心层开发 (Week 2-4)

**任务 2.1: ArangoDB 驱动集成**
- [x] 添加 ArangoDB Rust 依赖 (reqwest)
- [x] 实现 ArangoStorage 连接池 (直接 HTTP API)
- [x] 实现基础 CRUD 操作

**任务 2.2: Repository 层重写**
- [x] 重写 SessionRepository (AQL 查询)
- [x] 重写 TurnRepository (AQL 查询)
- [x] 重写 IndexRecordRepository (AQL 查询)

**任务 2.3: 配置和错误处理**
- [x] 更新 DatabaseConfig 结构 (添加 db_type, collection_prefix)
- [x] 更新错误处理逻辑 (From<String> for AppError)
- [x] 更新文档和配置示例

#### 阶段 3: 数据迁移 (Week 5)

**任务 3.1: 数据导出**
- [x] 导出所有 Session 数据
- [x] 导出所有 Turn 数据
- [x] 导出所有 IndexRecord 数据

**任务 3.2: 数据转换**
- [x] 转换 ID 格式 (Surrealdb → ArangoDB)
- [x] 转换关系数据为边集合
- [x] 验证数据完整性

**任务 3.3: 数据导入**
- [x] 创建 ArangoDB 集合
- [x] 导入文档数据
- [x] 导入边数据
- [ ] 验证数据一致性 (待 ArangoDB 实例测试)

#### 阶段 4: 测试与优化 (Week 6-7)

**任务 4.1: 单元测试**
- [x] Repository 层单元测试
- [x] 连接池单元测试
- [x] 查询功能测试 (5 ArangoDB 测试, 9 Config 测试)

**任务 4.2: 集成测试**
- [ ] API 集成测试
- [ ] 端到端测试
- [ ] 异常场景测试

**任务 4.3: 性能测试**
- [ ] 基准测试
- [ ] 负载测试
- [ ] 性能调优

#### 阶段 5: 部署与切换 (Week 8)

**任务 5.1: 生产环境准备**
- [ ] 部署 ArangoDB 集群
- [ ] 配置备份和监控
- [ ] 安全加固

**任务 5.2: 数据迁移执行**
- [ ] 备份现有数据
- [ ] 执行数据迁移
- [ ] 验证数据完整性

**任务 5.3: 流量切换**
- [ ] 配置双写 (可选)
- [ ] 切换应用配置
- [ ] 监控和回滚准备

---

## 5. 数据模型映射

### 5.1 Surrealdb → ArangoDB 映射表

| Surrealdb 概念 | ArangoDB 替代 | 说明 |
|----------------|---------------|------|
| Namespace | Database | ArangoDB 的 database 级别隔离 |
| Table | Collection | 文档集合或边集合 |
| Record ID | _key | 主键字段 |
| Record Link | Edge Collection | 使用 _from, _to 字段 |
| Field | Attribute | JSON 属性 |

### 5.2 集合映射

| Surrealdb Table | ArangoDB Collection | 类型 | 备注 |
|-----------------|---------------------|------|------|
| session | sessions | document | 主文档集合 |
| turn | turns | document | 轮次文档集合 |
| index_record | index_records | document | 索引文档集合 |
| (implicit) | session_turns | edge | Session → Turn |
| (implicit) | turn_parents | edge | Turn → Turn 父子关系 |
| (implicit) | turn_index_records | edge | Turn → IndexRecord |

### 5.3 字段映射

#### Session 字段映射

| Surrealdb 字段 | ArangoDB 字段 | 转换逻辑 |
|----------------|---------------|----------|
| id | _key | `session:⟨uuid⟩` → `uuid` |
| tenant_id | tenant_id | 保持不变 |
| name | name | 保持不变 |
| description | description | 保持不变 |
| created_at | created_at | DateTime → ISO 8601 |
| last_active_at | last_active_at | DateTime → ISO 8601 |
| status | status | 保持不变 |
| config | config | JSON 序列化 |
| stats | stats | JSON 序列化 |
| metadata | metadata | JSON 序列化 |

#### Turn 字段映射

| Surrealdb 字段 | ArangoDB 字段 | 转换逻辑 |
|----------------|---------------|----------|
| id | _key | `turn:⟨uuid⟩` → `turn_<uuid>` |
| session_id | session_key | `session:⟨uuid⟩` → `uuid` |
| turn_number | turn_number | 保持不变 |
| raw_content | raw_content | 保持不变 |
| metadata | metadata | JSON 序列化 |
| dehydrated | dehydrated | JSON 序列化 |
| status | status | 保持不变 |
| parent_id | parent_key | `turn:⟨uuid⟩` → `turn_<uuid>` |
| children_ids | children_keys | 数组转换 |

#### IndexRecord 字段映射

| Surrealdb 字段 | ArangoDB 字段 | 转换逻辑 |
|----------------|---------------|----------|
| turn_id | turn_key | `turn:⟨uuid⟩` → `turn_<uuid>` |
| session_id | session_key | `session:⟨uuid⟩` → `uuid` |
| tenant_id | tenant_id | 保持不变 |
| gist | gist | 保持不变 |
| topics | topics | 保持不变 |
| tags | tags | 保持不变 |
| timestamp | timestamp | DateTime → ISO 8601 |
| vector_id | vector_id | 保持不变 |
| relevance_score | relevance_score | 保持不变 |
| turn_number | turn_number | 保持不变 |

### 5.4 关系映射

#### Surrealdb 记录链接 → ArangoDB 边集合

**Session → Turn 关系**:
```sql
-- Surrealdb
SELECT * FROM turn WHERE session_id = 'session:⟨uuid⟩';

-- ArangoDB AQL
FOR turn IN turns
  FILTER turn.session_key == "uuid"
  RETURN turn
```

**Turn → Turn 父子关系**:
```sql
-- Surrealdb
SELECT * FROM turn WHERE parent_id = 'turn:⟨uuid⟩';

-- ArangoDB AQL
FOR child IN turns
  FILTER child.parent_key == "turn_<uuid>"
  RETURN child
```

---

## 6. 查询转换规则

### 6.1 CRUD 操作映射

#### CREATE (插入)

**Surrealdb**:
```sql
CREATE session SET 
  tenant_id = 'tenant_001',
  name = 'Test Session',
  created_at = '2024-01-15T10:30:00Z',
  ...
```

**ArangoDB AQL**:
```aql
INSERT {
  _key: "550e8400-e29b-41d4-a716-446655440000",
  tenant_id: "tenant_001",
  name: "Test Session",
  created_at: "2024-01-15T10:30:00Z",
  ...
} INTO sessions
```

#### READ (查询)

**Surrealdb**:
```sql
SELECT * FROM session WHERE id = session:⟨uuid⟩;
```

**ArangoDB AQL**:
```aql
RETURN DOCUMENT("sessions/550e8400-e29b-41d4-a716-446655440000")
```

**Surrealdb (带条件)**:
```sql
SELECT * FROM session WHERE tenant_id = 'tenant_001' 
  ORDER BY created_at DESC LIMIT 10 START 0;
```

**ArangoDB AQL**:
```aql
FOR s IN sessions
  FILTER s.tenant_id == "tenant_001"
  SORT s.created_at DESC
  LIMIT 0, 10
  RETURN s
```

#### UPDATE (更新)

**Surrealdb**:
```sql
UPDATE session:⟨uuid⟩ SET name = 'New Name', 
  last_active_at = '2024-01-15T11:00:00Z';
```

**ArangoDB AQL**:
```aql
UPDATE "550e8400-e29b-41d4-a716-446655440000" WITH {
  name: "New Name",
  last_active_at: "2024-01-15T11:00:00Z"
} IN sessions
```

#### DELETE (删除)

**Surrealdb**:
```sql
DELETE FROM session WHERE id = session:⟨uuid⟩;
```

**ArangoDB AQL**:
```aql
REMOVE "550e8400-e29b-41d4-a716-446655440000" IN sessions
```

### 6.2 聚合查询映射

**Surrealdb**:
```sql
SELECT count() FROM session WHERE tenant_id = 'tenant_001' GROUP ALL;
```

**ArangoDB AQL**:
```aql
RETURN {
  count: LENGTH(
    FOR s IN sessions
      FILTER s.tenant_id == "tenant_001"
      RETURN 1
  )
}
```

### 6.3 图遍历映射

**Surrealdb (获取会话的所有轮次)**:
```sql
SELECT * FROM turn WHERE session_id = 'session:⟨uuid⟩' 
  ORDER BY turn_number ASC;
```

**ArangoDB AQL**:
```aql
FOR turn IN turns
  FILTER turn.session_key == "550e8400-e29b-41d4-a716-446655440000"
  SORT turn.turn_number ASC
  RETURN turn
```

**ArangoDB AQL (使用边集合)**:
```aql
FOR session, edge, path IN 1..100 ANY "sessions/uuid" session_turns
  RETURN {
    session: session.name,
    turn: edge.turn_number,
    path_length: LENGTH(path.edges)
  }
```

---

## 7. 代码修改清单

### 7.1 新增文件

| 文件 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| `src/storage/arangodb.rs` | ArangoDB 连接池实现 (直接 HTTP API) | P0 | ✅ 完成 |
| `src/storage/arangodb_repository.rs` | Repository 层实现 (AQL 查询) | P0 | ✅ 完成 |
| `src/storage/factory.rs` | 存储工厂 (统一创建接口) | P1 | ✅ 完成 |
| `src/migration/mod.rs` | 迁移工具模块 | P1 | ✅ 完成 |
| `src/migration/export.rs` | 数据导出工具 | P1 | ✅ 完成 |
| `src/migration/transform.rs` | 数据转换工具 | P1 | ✅ 完成 |
| `src/migration/import.rs` | 数据导入工具 | P1 | ✅ 完成 |

### 7.2 修改文件

| 文件 | 修改内容 | 优先级 | 状态 |
|------|----------|--------|------|
| `Cargo.toml` | 添加 arangodb 特性标志 | P0 | ✅ 完成 |
| `src/storage/mod.rs` | 添加 arangodb 模块导出 | P0 | ✅ 完成 |
| `src/config/config.rs` | 添加 DatabaseType, collection_prefix | P1 | ✅ 完成 |
| `src/error.rs` | 添加 From<String> for AppError | P1 | ✅ 完成 |
| `src/lib.rs` | 添加 migration 模块 | P1 | ✅ 完成 |

### 7.3 测试文件

| 文件 | 描述 | 优先级 | 状态 |
|------|------|--------|------|
| `src/storage/arangodb.rs` | ArangoDB 存储测试 (5 tests) | P1 | ✅ 完成 |
| `src/config/config.rs` | Config 模块测试 (9 tests) | P1 | ✅ 完成 |

### 7.4 配置更新

**config.yaml**:
```yaml
# Surrealdb 配置 (移除)
# database:
#   url: "ws://localhost:8000"
#   namespace: "hippos"
#   database: "sessions"

# ArangoDB 配置 (新增)
database:
  url: "http://localhost:8529"
  username: "root"
  password: "password"
  database: "hippos"
  # 连接池配置
  min_connections: 5
  max_connections: 50
  connection_timeout: 30
  idle_timeout: 300
```

---

## 8. 测试策略

### 8.1 测试金字塔

```
                    ┌─────────────┐
                   /   Manual     \        5%
                  /   Testing      \
                 /                  \
                /    Integration     \    25%
               /      Tests           \
              /                        \
             /      Unit Tests          \    70%
            /                            \
           /______________________________\
```

### 8.2 测试覆盖要求

| 测试类型 | 覆盖率要求 | 关键测试点 |
|----------|-----------|------------|
| 单元测试 | ≥ 80% | Repository CRUD, 连接池, 序列化 |
| 集成测试 | 100% API 端点 | 所有 REST API 路径 |
| E2E 测试 | 核心用户流程 | 创建会话, 添加轮次, 搜索 |
| 性能测试 | N/A | 响应时间, 吞吐量, 并发 |

### 8.3 测试环境

| 环境 | 数据库 | 用途 |
|------|--------|------|
| CI | Embedded/Container | 单元测试, 集成测试 |
| Staging | ArangoDB Single | 端到端测试 |
| Production | ArangoDB Cluster | 性能测试, 预发布验证 |

### 8.4 测试用例示例

#### Repository CRUD 测试

```rust
#[tokio::test]
async fn test_session_repository_create() {
    // 1. 创建 Session
    let session = Session::new("tenant_001", "Test Session");
    let created = repo.create(&session).await.unwrap();
    
    // 2. 验证创建
    assert_eq!(created.name, "Test Session");
    
    // 3. 验证可以从数据库读取
    let fetched = repo.get_by_id(&created.id).await.unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name, "Test Session");
}
```

#### 查询转换测试

```rust
#[tokio::test]
async fn test_aql_query_equivalence() {
    // 对比 Surrealdb 和 ArangoDB 查询结果
    let surrealdb_results = /* ... */;
    let arangodb_results = /* ... */;
    
    assert_eq!(surrealdb_results.len(), arangodb_results.len());
    // 验证字段一致性
}
```

---

## 9. 部署方案

### 9.1 ArangoDB 部署选项

#### 选项 A: 单节点 (开发/测试)

```yaml
# docker-compose.yml
services:
  arangodb:
    image: arangodb:3.11
    environment:
      ARANGO_ROOT_PASSWORD: password
    ports:
      - "8529:8529"
    volumes:
      - arango_data:/var/lib/arangodb3
```

#### 选项 B: 集群 (生产)

```yaml
# Kubernetes Deployment
# 建议使用 ArangoDB Kubernetes Operator
apiVersion: database.arangodb.com/v1alpha
kind: ArangoDeployment
metadata:
  name: hippos-arangodb
spec:
  mode: Cluster
  image: arangodb/arangodb:3.11.0
  tls:
    caSecretName: arango-tls-secret
```

### 9.2 部署步骤

#### Step 1: 环境准备
1. 部署 ArangoDB 集群
2. 配置备份策略
3. 配置监控告警
4. 安全加固 (TLS, 防火墙)

#### Step 2: 数据库初始化
```bash
# 创建数据库
arangosh --server.database=hippos --command "require('internal').db.createDatabase('hippos')"

# 创建集合
# 运行 migration/scripts/create_collections.js
```

#### Step 3: 应用部署
1. 构建新版本应用
2. 更新配置 (指向 ArangoDB)
3. 滚动部署

#### Step 4: 数据迁移
```bash
# 1. 导出 Surrealdb 数据
cargo run --bin migrate -- export --source surrealdb --output /tmp/data

# 2. 转换数据格式
cargo run --bin migrate -- transform --input /tmp/data --output /tmp/arangodb

# 3. 导入 ArangoDB
cargo run --bin migrate -- import --input /tmp/arangodb
```

### 9.3 回滚方案

| 场景 | 回滚操作 | 恢复时间 |
|------|----------|----------|
| 应用启动失败 | 回滚到 Surrealdb 版本 | 5 分钟 |
| 数据迁移失败 | 使用备份恢复 | 30 分钟 |
| 功能异常 | 切换回 Surrealdb | 10 分钟 |
| 性能下降 | 回滚并优化 | 需分析 |

---

## 10. 风险评估

### 10.1 风险清单

| 风险 | 影响 | 概率 | 风险等级 | 缓解措施 |
|------|------|------|----------|----------|
| 查询性能下降 | 高 | 中 | 🔴 高 | 性能测试, 索引优化 |
| 数据迁移丢失 | 高 | 低 | 🔴 高 | 备份, 验证脚本 |
| 驱动不成熟 | 中 | 中 | 🟡 中 | POC 验证, 备选方案 |
| 学习曲线 | 低 | 高 | 🟡 中 | 文档, 培训 |
| API 兼容性问题 | 高 | 低 | 🟡 中 | 完整测试覆盖 |
| 回滚困难 | 高 | 低 | 🟡 中 | 灰度发布, 双写 |

### 10.2 缓解计划

#### 高风险缓解

**1. 查询性能下降**
- 迁移前进行基准测试
- 优化 AQL 查询和索引
- 实施查询缓存策略

**2. 数据迁移丢失**
- 全量备份 Surrealdb
- 实现数据校验脚本
- 准备回滚脚本

#### 中风险缓解

**1. 驱动不成熟**
- 提前进行 POC 验证
- 评估多个驱动备选
- 准备纯 HTTP API 方案

**2. API 兼容性问题**
- 完整的自动化测试
- 端到端用户流程测试
- 渐进式功能验证

### 10.3 监控指标

| 指标 | 告警阈值 | 说明 |
|------|----------|------|
| API 响应时间 | P95 > 500ms | 性能监控 |
| 错误率 | > 1% | 稳定性监控 |
| 数据库连接数 | > 80% 容量 | 容量监控 |
| 查询成功率 | < 99.9% | 质量监控 |

---

## 附录

### A. 参考资料

- [ArangoDB 官方文档](https://www.arangodb.com/docs/)
- [AQL 教程](https://www.arangodb.com/tutorials/arangoql/)
- [arango-rs Crates](https://crates.io/crates/arango)
- [ArangoDB 部署指南](https://www.arangodb.com/docs/stable/deployment.html)

### B. 术语对照

| Surrealdb | ArangoDB | 说明 |
|-----------|----------|------|
| Namespace | Database | 数据库实例 |
| Table | Collection | 数据集合 |
| Record | Document | 文档 |
| Record ID | _key | 主键 |
| SurrealQL | AQL | 查询语言 |
| Record Link | Edge | 边 |

### C. 检查清单

#### 迁移前检查
- [ ] ArangoDB 驱动 POC 完成
- [ ] 数据模型设计评审通过
- [ ] 备份策略已制定
- [ ] 回滚方案已准备

#### 代码检查
- [ ] 所有 Surrealdb 引用已移除
- [ ] Repository 层测试通过
- [ ] API 测试 100% 通过
- [ ] 性能基准达标

#### 部署检查
- [ ] 生产环境准备完成
- [ ] 监控告警已配置
- [ ] 数据迁移脚本测试通过
- [ ] 回滚脚本测试通过

---

> **文档版本**: 1.1  
> **最后更新**: 2025-01-15  
> **负责人**: 迁移团队  
> **总体进度**: 47% (7/15 任务完成)
