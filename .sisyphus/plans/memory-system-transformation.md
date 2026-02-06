# Hippos AI 记忆系统改造计划

## 一、项目概述

### 1.1 背景

Hippos 是一个高性能对话上下文管理服务，现需要改造为 OpenClaw Agent 的"无线记忆引擎"，支持：
- 多类型记忆存储（用户画像、会话摘要、Agent技能、知识库）
- 多种记忆来源（对话交互、研究结果、执行经验、用户配置）
- 混合检索能力（时序 + 语义 + 上下文推断）
- 本地 SurrealDB 存储（隐私优先）
- HTTP + WebSocket 双协议交互

### 1.2 核心设计原则

| 原则 | 说明 |
|------|------|
| **渐进改造** | 保留经过验证的组件，逐步替换需要重构的部分 |
| **隐私优先** | 本地 SurrealDB 存储，数据不离开用户设备 |
| **渐进式整合** | 定期摘要 + 重要性评分 + 关系构建 |
| **服务化架构** | OpenClaw 通过 API 调用记忆服务 |
| **可观测性** | 健康检查、指标监控、结构化日志 |

### 1.3 技术选型决策

| 组件 | 决策 | 理由 |
|------|------|------|
| **向量索引** | SurrealDB 原生向量搜索 | 简化架构，统一的存储层 |
| **协议层** | HTTP + WebSocket | 批量操作用 HTTP，实时推送用 WebSocket |
| **集成方式** | REST API 独立服务 | 保持服务独立性，支持未来多客户端 |
| **MCP 协议** | 暂不考虑 | 当前聚焦核心功能，后续可扩展 |

---

## 二、架构设计

### 2.1 目标架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         OpenClaw Agent                                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐ │
│  │ 对话记忆模块    │  │ 技能学习模块    │  │ 知识管理模块               │ │
│  └────────┬────────┘  └────────┬────────┘  └──────────────┬──────────────┘ │
└───────────┼────────────────────┼───────────────────────────┼─────────────────┘
            │                    │                           │
            └────────────────────┼───────────────────────────┘
                                 ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                      Hippos Memory Service                                 │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                         API Layer (Axum)                              │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐   │ │
│  │  │ REST API       │  │ WebSocket      │  │ Health & Metrics       │   │ │
│  │  │ /memories      │  │ /ws            │  │ /health                │   │ │
│  │  │ /profiles      │  │ /subscriptions │  │ /metrics               │   │ │
│  │  │ /patterns      │  │                │  │                        │   │ │
│  │  │ /entities      │  │                │  │                        │   │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────────┘   │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                       │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                     Security Layer                                    │ │
│  │  Auth Middleware │ RBAC │ Validation │ Security Headers               │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                       │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                      Service Layer                                    │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐    │ │
│  │  │ MemoryBuilder   │  │ MemoryRecall    │  │ ProfileManager      │    │ │
│  │  │ - 重要性评分      │  │ - 语义检索       │  │ - 用户画像管理        │    │ │
│  │  │ - 记忆摘要        │  │ - 时序检索       │  │ - 偏好追踪           │    │ │
│  │  │ - 关系构建       │  │ - 上下文推断      │  │ - 事实存储           │    │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘    │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐    │ │
│  │  │ PatternManager  │  │ Dehydration     │  │ EntityManager       │    │ │
│  │  │ - 问题模式库      │  │ - 摘要生成       │  │ - 实体/关系管理       │    │ │
│  │  │ - 技能模式       │  │ - 关键词提取      │  │ - 知识图谱           │     │ │
│  │  │ - 经验沉淀       │  │ - 主题分类        │  │                     │    │ │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────────┘    │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                    │                                       │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                     Storage Layer (SurrealDB)                         │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐   │ │
│  │  │ MemoryRepo     │  │ ProfileRepo    │  │ PatternRepo            │   │ │
│  │  │ EntityRepo     │  │ IndexRepo      │  │                        │   │ │
│  │  └────────────────┘  └────────────────┘  └────────────────────────┘   │ │
│  │                                                                       │ │
│  │  ┌───────────────────────────────────────────────────────────────┐    │ │
│  │  │          SurrealDB Vector Search + Full-Text Search           │    │ │
│  │  └───────────────────────────────────────────────────────────────┘    │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 数据模型

#### Memory（核心记忆模型）

```rust
struct Memory {
    id: String,              // 记忆唯一标识
    memory_type: MemoryType, // EPISODIC, SEMANTIC, PROCEDURAL, PROFILE
    tenant_id: String,       // 租户隔离
    user_id: String,         // 用户/Agent ID

    // 内容
    content: String,         // 原始内容
    gist: String,            // 摘要（脱水后）
    embedding: Vec<f32>,     // 向量表示

    // 元数据
    importance: f32,         // 重要性评分 (0.0-1.0)
    confidence: f32,         // 置信度
    source: MemorySource,    // 来源类型
    source_id: Option<String>, // 原始来源ID

    // 关系
    parent_id: Option<String>,   // 父记忆ID
    related_ids: Vec<String>,    // 相关记忆ID
    tags: Vec<String>,           // 标签
    topics: Vec<String>,         // 主题

    // 时间
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    accessed_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,

    // 状态
    status: MemoryStatus,    // ACTIVE, ARCHIVED, DELETED
    version: u32,            // 版本号（乐观锁）
}

enum MemoryType {
    EPISODIC,    // 事件/对话记忆
    SEMANTIC,    // 事实/知识记忆
    PROCEDURAL,  // 技能/流程记忆
    PROFILE,     // 用户画像记忆
}

enum MemorySource {
    CONVERSATION,  // 对话交互
    RESEARCH,      // 研究结果
    EXECUTION,     // 执行经验
    USER_CONFIG,   // 用户配置
}
```

#### Profile（用户画像）

```rust
struct Profile {
    id: String,
    tenant_id: String,
    user_id: String,

    // 基本信息
    name: Option<String>,
    role: Option<String>,
    organization: Option<String>,
    location: Option<String>,

    // 偏好
    preferences: HashMap<String, Value>,  // 结构化偏好
    communication_style: Option<String>,   // 沟通风格
    technical_level: Option<String>,       // 技术水平

    // 重要事实
    facts: Vec<ProfileFact>,               // 关键事实
    interests: Vec<String>,                // 兴趣领域

    // 工作模式
    working_hours: Option<WorkingHours>,
    common_tasks: Vec<String>,
    tools_used: Vec<String>,

    // 元数据
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    confidence: f32,                       // 画像置信度
    last_verified: Option<DateTime<Utc>>,
}

struct ProfileFact {
    fact: String,              // 事实描述
    category: String,          // 类别
    source_memory_id: Option<String>, // 来源记忆
    confidence: f32,           // 置信度
    verified: bool,            // 是否验证
}
```

#### Pattern（问题模式库）

```rust
struct Pattern {
    id: String,
    tenant_id: String,
    pattern_type: PatternType,

    // 模式定义
    name: String,
    description: String,
    trigger: String,           // 触发条件
    context: String,           // 适用场景

    // 模式内容
    problem: String,           // 问题描述
    solution: String,          // 解决方案
    examples: Vec<PatternExample>,

    // 效果追踪
    success_count: u32,
    failure_count: u32,
    avg_outcome: f32,

    // 元数据
    tags: Vec<String>,
    created_by: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    usage_count: u32,
}

enum PatternType {
    PROBLEM_SOLUTION,  // 问题-解决方案模式
    WORKFLOW,          // 工作流程模式
    BEST_PRACTICE,     // 最佳实践
    COMMON_ERROR,      // 常见错误模式
    SKILL,             // 技能模式
}
```

#### Entity & Relationship（实体关系）

```rust
struct Entity {
    id: String,
    tenant_id: String,

    // 实体定义
    name: String,
    entity_type: String,       // PERSON, ORGANIZATION, PROJECT, TOOL, CONCEPT...
    description: Option<String>,

    // 属性
    properties: HashMap<String, Value>,
    aliases: Vec<String>,      // 别名

    // 向量表示
    embedding: Vec<f32>,

    // 元数据
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    confidence: f32,
    source_memory_ids: Vec<String>,
}

struct Relationship {
    id: String,
    tenant_id: String,

    // 关系定义
    source_entity_id: String,
    target_entity_id: String,
    relationship_type: String, // KNOWS, WORKS_ON, PART_OF, USES...

    // 元数据
    strength: f32,             // 关系强度 (0-1)
    context: Option<String>,   // 关系上下文
    source_memory_id: String,  // 来源记忆

    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

---

## 三、改造范围

### 3.1 可复用组件（保留）

| 组件 | 状态 | 说明 |
|------|------|------|
| `src/storage/repository.rs` | ✅ 保留 | Repository 模式 + SurrealDB 集成 |
| `src/storage/surrealdb.rs` | ✅ 保留 | 连接池 + HTTP API |
| `src/security/middleware.rs` | ✅ 保留 | 认证、RBAC、限流中间件 |
| `src/security/auth.rs` | ✅ 保留 | API Key + JWT 认证 |
| `src/api/app_state.rs` | ✅ 保留 | 依赖注入容器 |
| `src/api/mod.rs` | ✅ 保留 | 路由组装模式 |
| `src/index/mod.rs` | ⚠️ 适配 | IndexService 抽象，改为 SurrealDB 原生向量 |
| `src/services/dehydration.rs` | ⚠️ 适配 | 摘要生成框架，扩展主题分类 |

### 3.2 需要重构的组件

| 组件 | 新增/修改 | 说明 |
|------|-----------|------|
| `src/models/memory.rs` | 新增 | Memory 记忆模型 |
| `src/models/profile.rs` | 新增 | Profile 用户画像模型 |
| `src/models/pattern.rs` | 新增 | Pattern 问题模式模型 |
| `src/models/entity.rs` | 新增 | Entity/Relationship 实体关系模型 |
| `src/services/memory_builder.rs` | 新增 | 记忆构建服务（重要性评分、摘要、关系构建） |
| `src/services/memory_recall.rs` | 新增 | 记忆检索服务（语义搜索、时序检索、上下文推断） |
| `src/services/profile_manager.rs` | 新增 | 用户画像管理服务 |
| `src/services/pattern_manager.rs` | 新增 | 问题模式管理服务 |
| `src/services/entity_manager.rs` | 新增 | 实体关系管理服务 |
| `src/storage/memory_repository.rs` | 新增 | Memory 仓储 |
| `src/storage/profile_repository.rs` | 新增 | Profile 仓储 |
| `src/storage/pattern_repository.rs` | 新增 | Pattern 仓储 |
| `src/storage/entity_repository.rs` | 新增 | Entity/Relationship 仓储 |
| `src/api/routes/memory_routes.rs` | 新增 | 记忆 API 路由 |
| `src/api/routes/profile_routes.rs` | 新增 | 画像 API 路由 |
| `src/api/routes/pattern_routes.rs` | 新增 | 模式 API 路由 |
| `src/api/routes/entity_routes.rs` | 新增 | 实体 API 路由 |
| `src/websocket/mod.rs` | 新增 | WebSocket handler |
| `src/websocket/subscription.rs` | 新增 | 订阅管理 |
| `src/api/handlers/session_handler.rs` | 🔄 废弃 | 原有 Session handler（保留但标记废弃） |
| `src/api/handlers/turn_handler.rs` | 🔄 废弃 | 原有 Turn handler（保留但标记废弃） |
| `src/api/handlers/search_handler.rs` | 🔄 适配 | 改造为通用搜索服务 |

### 3.3 改造优先级

| 优先级 | 任务 | 预计工作量 |
|--------|------|------------|
| P0 | 核心基础设施（模型 + 存储 + 基础服务） | 中 |
| P0 | 记忆 CRUD API + 检索服务 | 中 |
| P0 | 用户画像服务 | 中 |
| P1 | 问题模式服务 | 中 |
| P1 | 实体关系服务 | 中 |
| P1 | WebSocket 实时订阅 | 中 |
| P2 | 记忆整合（摘要、重要性重评） | 中 |
| P2 | MCP 协议支持（未来扩展） | 低 |

---

## 四、实施计划

### 阶段 1：基础设施搭建

#### 1.1 新增数据模型

- [x] 创建 `src/models/memory.rs`
- [x] 创建 `src/models/profile.rs`
- [x] 创建 `src/models/pattern.rs`
- [x] 创建 `src/models/entity.rs`
- [x] 创建 DTO 文件 `src/api/dto/memory_dto.rs`
- [x] 创建 DTO 文件 `src/api/dto/profile_dto.rs`

#### 1.2 新增存储层

- [x] 创建 `src/storage/memory_repository.rs`
- [x] 创建 `src/storage/profile_repository.rs`
- [x] 创建 `src/storage/pattern_repository.rs`
- [x] 创建 `src/storage/entity_repository.rs`
- [x] 在 SurrealDB 中创建新表结构

#### 1.3 新增服务层基础

- [x] 创建 `src/services/memory_builder.rs`
  - 重要性评分算法
  - 摘要生成集成
  - 关系自动构建
- [x] 创建 `src/services/memory_recall.rs`
  - 语义检索
  - 时序检索
  - 上下文推断

#### 1.4 新增 API 层

- [x] 创建 `src/api/routes/memory_routes.rs`
- [x] 创建 `src/api/handlers/memory_handler.rs`
- [x] 创建 DTO 和请求验证
- [x] 更新 `src/api/mod.rs` 注册新路由

**验收标准**：
- [x] `cargo check` 通过
- [x] API 文档生成
- [x] 单元测试覆盖 > 60%

---

### 阶段 2：用户画像服务

#### 2.1 Profile 服务

- [x] 创建 `src/services/profile_manager.rs`
  - 基本信息 CRUD
  - 偏好管理
  - 事实存储和验证
  - 工作模式追踪
- [x] 创建 `src/storage/profile_repository.rs`

#### 2.2 Profile API

- [x] 创建 `src/api/routes/profile_routes.rs`
- [x] 创建 `src/api/handlers/profile_handler.rs`

**验收标准**：
- [x] Profile CRUD 功能完整
- [x] 事实验证机制可用
- [x] 与 Memory 服务集成

---

### 阶段 3：问题模式服务

#### 3.1 Pattern 服务

- [x] 创建 `src/services/pattern_manager.rs`
  - Pattern CRUD
  - 效果追踪
  - 自动推荐
- [x] 创建 `src/storage/pattern_repository.rs`

#### 3.2 Pattern API

- [x] 创建 `src/api/routes/pattern_routes.rs`
- [x] 创建 `src/api/handlers/pattern_handler.rs`

**验收标准**：
- [x] Pattern 完整生命周期管理
- [x] 效果追踪和统计
- [x] 基于 Memory 自动生成 Pattern 的能力

---

### 阶段 4：实体关系服务

#### 4.1 Entity 服务

- [x] 创建 `src/services/entity_manager.rs`
  - Entity CRUD
  - Relationship 管理
  - 知识图谱查询
- [x] 创建 `src/storage/entity_repository.rs`

#### 4.2 Entity API

- [x] 创建 `src/api/routes/entity_routes.rs`
- [x] 创建 `src/api/handlers/entity_handler.rs`

**验收标准**：
- [x] Entity 和 Relationship 完整管理
- [x] 知识图谱遍历查询
- [x] 与 Memory 服务的双向关联

---

### 阶段 5：WebSocket 实时订阅

#### 5.1 WebSocket Handler

- [x] 创建 `src/websocket/mod.rs`
- [x] 创建 `src/websocket/subscription.rs`
- [x] 实现连接管理
- [x] 实现主题订阅

#### 5.2 WebSocket API

**验收标准**：
- [x] 连接认证和生命周期管理
- [x] 主题订阅和过滤
- [x] 记忆更新实时推送

---

### 阶段 6：记忆整合与优化

#### 6.1 记忆整合服务

- [x] 实现定期摘要任务
- [x] 实现重要性重评
- [x] 实现冗余合并
- [x] 实现关系更新

#### 6.2 优化

- [x] 性能优化（缓存、批量操作）
- [x] SurrealDB 向量搜索配置
- [x] 索引优化

---

## 五、技术决策记录

### 5.1 向量索引方案

**决策**：使用 SurrealDB 原生向量搜索

**理由**：
- 简化架构，统一的存储层
- 避免维护额外的向量数据库
- SurrealDB 的向量搜索已足够生产可用
- 数据一致性更好（无需同步多个数据源）

**配置**：
```yaml
surrealdb:
  vector:
    dimension: 384
    metric: cosine
```

### 5.2 API 协议选择

**决策**：REST API + WebSocket

**理由**：
- REST：批量操作、标准化、便于文档化
- WebSocket：实时推送、订阅机制
- 两者都是成熟且广泛支持的协议

### 5.3 存储策略

**决策**：单一 SurrealDB 数据库，本地存储

**理由**：
- 隐私优先，数据不离开用户设备
- SurrealDB 支持多种数据模型（文档、图、向量的融合）
- 简化部署和运维

---

## 六、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SurrealDB 向量搜索性能 | 中 | 监控性能，必要时迁移到专用向量库 |
| 记忆整合复杂度 | 高 | 渐进式实现，先基础后优化 |
| 数据迁移 | 中 | 提供迁移工具，支持新旧并存 |
| 向后兼容性 | 低 | 旧 API 保留，标记废弃 |

---

## 七、测试策略

### 7.1 测试覆盖目标

| 类型 | 覆盖率目标 |
|------|------------|
| 单元测试 | > 70% |
| 集成测试 | > 50% |
| API 测试 | 关键路径 100% |

### 7.2 测试策略

1. **单元测试**：每个服务层方法
2. **集成测试**：存储层 + SurrealDB
3. **API 测试**：使用 reqwest 测试所有端点
4. **WebSocket 测试**：连接、订阅、推送

---

## 八、文档计划

- [x] API 文档（OpenAPI 3.0）
- [x] 架构文档
- [x] 部署指南
- [x] 迁移指南（从旧版 Hippos）
- [x] 示例和教程

---

## 九、时间估算

| 阶段 | 任务数 | 预估时间 |
|------|--------|----------|
| 阶段 1：基础设施 | 14 | 3-4 天 |
| 阶段 2：用户画像 | 6 | 2 天 |
| 阶段 3：问题模式 | 6 | 2 天 |
| 阶段 4：实体关系 | 6 | 2 天 |
| 阶段 5：WebSocket | 5 | 2 天 |
| 阶段 6：整合优化 | 5 | 2 天 |
| **总计** | **42** | **15-16 天** |

---

## 十、后续扩展

### 10.1 短期扩展（P2）

- MCP 协议支持
- 跨设备同步
- 备份和恢复

### 10.2 长期扩展

- 多语言支持
- 分布式部署
- 高级分析功能
