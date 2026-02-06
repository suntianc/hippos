# Hippos 代码审查修复计划

**创建日期**: 2026-02-03
**基于**: CODE_REVIEW_REPORT.md
**目标**: 修复所有识别的代码问题

---

## 执行摘要

| 阶段 | 问题数 | 预计时间 |
|------|--------|----------|
| Phase 1: Critical 修复 | 2 | 30 分钟 |
| Phase 2: High 修复 | 5 | 2 小时 |
| Phase 3: Medium 修复 | 7 | 3 小时 |
| Phase 4: Low 改进 | 3 | 1 小时 |
| **总计** | **17** | **~6.5 小时** |

---

## Phase 1: Critical 修复

### CRIT-001: WebSocket JSON 序列化错误处理

**文件**: `src/websocket/mod.rs`
**行号**: 113, 268, 292, 334
**预计时间**: 15 分钟

#### 任务清单

- [ ] 1.1 创建错误处理辅助函数 `serialize_and_send()`
- [ ] 1.2 重构第 113 行: `init_event` 发送
- [ ] 1.3 重构第 268 行: `confirmation` 发送
- [ ] 1.4 重构第 292 行: `error_confirmation` 发送
- [ ] 1.5 重构第 334 行: `message` 发送

#### 验证命令

```bash
cargo check --lib 2>&1 | grep -E "error|warning"
cargo test --lib websocket 2>&1 | tail -5
```

#### 风险等级: 中
- 修改 WebSocket 核心逻辑，需要完整测试

---

### CRIT-002: 浮点数排序的 NaN 处理

**文件**: `src/services/memory_recall.rs`, `src/services/entity_manager.rs`
**行号**: 421, 612, 740, 429
**预计时间**: 15 分钟

#### 任务清单

- [ ] 2.1 创建排序辅助函数 `safe_sort_by_score()`
- [ ] 2.2 重构 `memory_recall.rs:421` - hybrid_search 排序
- [ ] 2.3 重构 `memory_recall.rs:612` - temporal_search 排序
- [ ] 2.4 重构 `memory_recall.rs:740` - semantic_search 排序
- [ ] 2.5 重构 `entity_manager.rs:429` - entity_similarity 排序

#### 验证命令

```bash
cargo test --lib memory_recall 2>&1 | tail -10
cargo test --lib entity_manager 2>&1 | tail -10
```

#### 风险等级: 低
- 修改排序逻辑，不影响业务功能

---

## Phase 2: High 优先级修复

### HIGH-001: 输入长度验证

**文件**: 
- `src/api/handlers/entity_handler.rs`
- `src/api/handlers/memory_handler.rs`
- `src/api/handlers/pattern_handler.rs`
- `src/api/handlers/profile_handler.rs`
**预计时间**: 45 分钟

#### 任务清单

- [ ] 3.1 在 `src/api/dto/` 中定义常量 `MAX_NAME_LENGTH`, `MAX_CONTENT_LENGTH`
- [ ] 3.2 更新 `entity_handler.rs` - CreateEntityRequest 验证
- [ ] 3.3 更新 `entity_handler.rs` - CreateRelationshipRequest 验证
- [ ] 3.4 更新 `memory_handler.rs` - CreateMemoryRequest 验证
- [ ] 3.5 更新 `memory_handler.rs` - UpdateMemoryRequest 验证
- [ ] 3.6 更新 `pattern_handler.rs` - CreatePatternRequest 验证
- [ ] 3.7 更新 `profile_handler.rs` - CreateProfileRequest 验证

#### 验证命令

```bash
# 测试超长输入被拒绝
curl -X POST http://localhost:8080/api/v1/entities \
  -H "Content-Type: application/json" \
  -d '{"name":"'$(python3 -c 'print("x"*1000)')'", "entity_type":"person"}'
# 应返回 400 Validation Error
```

#### 风险等级: 中
- 修改 API 行为，需要更新文档

---

### HIGH-002: 正则表达式缓存

**文件**: `src/services/entity_manager.rs`
**行号**: 631, 656
**预计时间**: 30 分钟

#### 任务清单

- [ ] 4.1 添加依赖: `once_cell = "1.20"`
- [ ] 4.2 创建静态正则: `ENTITY_NAME_RE`
- [ ] 4.3 创建静态正则: `RELATIONSHIP_RE_PATTERNS`
- [ ] 4.4 重构 `extract_entity_names()` 使用缓存
- [ ] 4.5 重构 `extract_relationships()` 使用缓存

#### 验证命令

```bash
cargo test --lib entity_manager::tests::test_extract_entity_names 2>&1
cargo bench  # 验证性能提升
```

#### 风险等级: 低
- 性能优化，不影响功能

---

### HIGH-003: 连接池改进

**文件**: `src/storage/surrealdb.rs`
**行号**: 15
**预计时间**: 1 小时

#### 任务清单

- [ ] 5.1 重构 `SurrealPool` 结构体
- [ ] 5.2 实现连接池管理逻辑
- [ ] 5.3 更新 `get()` 方法
- [ ] 5.4 更新 `execute()` 方法
- [ ] 5.5 更新所有调用点

#### 验证命令

```bash
cargo test --lib storage 2>&1 | tail -10
cargo build --release 2>&1 | tail -5
```

#### 风险等级: 高
- 核心存储层修改，需要全面测试

---

### HIGH-004: 缓存容量限制

**文件**: `src/services/performance.rs`
**行号**: 109
**预计时间**: 30 分钟

#### 任务清单

- [ ] 6.1 添加依赖: `lru = "0.12"` 或 `moka = "0.12"`
- [ ] 6.2 重构 `MemoryCache` 使用 LruCache
- [ ] 6.3 添加 `max_capacity` 配置
- [ ] 6.4 更新 `config.yaml` 配置

#### 验证命令

```bash
cargo test --lib performance 2>&1 | tail -10
```

#### 风险等级: 中
- 修改缓存行为，需要测试命中率

---

### HIGH-005: 时间戳处理

**文件**: `src/security/auth.rs`
**行号**: 225, 252
**预计时间**: 15 分钟

#### 任务清单

- [ ] 7.1 创建常量 `API_KEY_EXPIRY_YEARS = 50`
- [ ] 7.2 使用 `chrono::Duration` 计算过期时间
- [ ] 7.3 重构第 225 行 API key 过期时间
- [ ] 7.4 重构第 252 行 JWT 过期时间

#### 验证命令

```bash
cargo test --lib auth 2>&1 | tail -10
```

#### 风险等级: 低
- 未来兼容性修复

---

## Phase 3: Medium 优先级修复

### MED-001: 可配置请求体大小限制

**文件**: `src/security/middleware.rs`
**行号**: 439
**预计时间**: 20 分钟

#### 任务清单

- [ ] 8.1 在 `config.rs` 中添加 `max_request_size` 配置
- [ ] 8.2 更新 `middleware.rs` 使用配置值
- [ ] 8.3 更新 `config.yaml` 示例

---

### MED-002: 测试代码改进

**文件**: `src/services/pattern_manager.rs`
**预计时间**: 30 分钟

#### 任务清单

- [ ] 9.1 将 `unwrap()` 替换为 `assert!()` / `assert_eq!()`
- [ ] 9.2 添加有意义的断言消息

---

### MED-003: 日志级别配置

**文件**: `src/config/`, `src/main.rs`
**预计时间**: 20 分钟

#### 任务清单

- [ ] 10.1 确保 `RUST_LOG` 环境变量被正确处理
- [ ] 10.2 添加开发/生产环境日志配置

---

### MED-004: 断路器模式

**文件**: `src/storage/surrealdb.rs`
**预计时间**: 1.5 小时

#### 任务清单

- [ ] 11.1 添加依赖: `circuitbreaker = "2.0"`
- [ ] 11.2 实现 SurrealDB 断路器
- [ ] 11.3 配置断路器参数（失败阈值、超时）
- [ ] 11.4 添加断路器指标到 observability

---

### MED-005: 请求超时设置

**文件**: `src/api/handlers/`, `src/config/`
**预计时间**: 30 分钟

#### 任务清单

- [ ] 12.1 在 `config.yaml` 添加默认超时配置
- [ ] 12.2 创建超时中间件
- [ ] 12.3 应用到所有 API 路由

---

### MED-006: 链路追踪

**文件**: `src/observability/`
**预计时间**: 1 小时

#### 任务清单

- [ ] 13.1 添加依赖: `opentelemetry`, `tracing`
- [ ] 13.2 创建追踪初始化函数
- [ ] 13.3 添加关键操作的追踪 span
- [ ] 13.4 配置追踪导出器（OTLP/Jaeger）

---

### MED-007: 配置验证

**文件**: `src/config/`
**预计时间**: 20 分钟

#### 任务清单

- [ ] 14.1 创建配置验证函数
- [ ] 14.2 验证数值范围（端口、超时等）
- [ ] 14.3 验证必需字段
- [ ] 14.4 启动时打印配置摘要

---

## Phase 4: Low 优先级改进

### LOW-001: 命名一致性

**文件**: `src/services/performance.rs`
**预计时间**: 10 分钟

#### 任务清单

- [ ] 15.1 重命名为 `performance_manager.rs`
- [ ] 15.2 更新 `mod.rs` 中的模块引用

---

### LOW-002: 模块文档

**文件**: 多个模块
**预计时间**: 30 分钟

#### 任务清单

- [ ] 16.1 为缺少文档的模块添加 `//!` 文档注释
- [ ] 16.2 遵循项目文档风格

---

### LOW-003: 错误消息改进

**文件**: 多个文件
**预计时间**: 20 分钟

#### 任务清单

- [ ] 17.1 审查所有 `AppError` 定义
- [ ] 17.2 添加上下文信息到错误消息
- [ ] 17.3 确保错误消息对用户友好

---

## 依赖关系图

```
Phase 1 (Critical)
├── CRIT-001 → 无依赖
└── CRIT-002 → 无依赖

Phase 2 (High)
├── HIGH-001 → CRIT-001, CRIT-002
├── HIGH-002 → Phase 1 完成
├── HIGH-003 → HIGH-002 (连接池影响存储测试)
├── HIGH-04  → Phase 1 完成
└── HIGH-005 → Phase 1 完成

Phase 3 (Medium)
├── MED-001  → HIGH-001 (配置系统)
├── MED-002  → Phase 2 完成
├── MED-003  → 无依赖
├── MED-004  → HIGH-003 (断路器基于连接池)
├── MED-005  → MED-001 (超时配置)
├── MED-006  → MED-003 (日志配置)
└── MED-007  → MED-001 (配置验证)

Phase 4 (Low)
├── LOW-001  → Phase 3 完成
├── LOW-002  → 无依赖
└── LOW-003  → Phase 1 完成
```

---

## 验证清单

### 每次提交后

- [ ] `cargo check --lib` 通过
- [ ] `cargo test --lib` 全部通过
- [ ] `cargo clippy --lib` 无新警告

### Phase 完成后

- [ ] API 测试通过 (`./test_api.sh`)
- [ ] MCP 测试通过 (`./test_mcp_server.sh`)
- [ ] 性能基准无明显退化

### 最终验证

- [ ] 所有 18 个问题状态为 "fixed"
- [ ] 代码审查报告更新为 "completed"
- [ ] 提交审查修复 commit

---

## 风险缓解

### 高风险任务 (HIGH-003, MED-004)

1. **备份当前实现**
2. **创建功能开关**
3. **灰度发布**
4. **快速回滚机制**

### 测试策略

1. **单元测试**: 覆盖所有修复
2. **集成测试**: 验证 API 行为
3. **负载测试**: 验证性能改进
4. **混沌测试**: 验证断路器

---

## 时间估算

| 阶段 | 乐观 | 悲观 | 实际 |
|------|------|------|------|
| Phase 1 | 30 分钟 | 1 小时 | - |
| Phase 2 | 2 小时 | 4 小时 | - |
| Phase 3 | 3 小时 | 6 小时 | - |
| Phase 4 | 1 小时 | 2 小时 | - |
| **总计** | **6.5 小时** | **13 小时** | - |

---

## 下一步

1. **确认计划** - 用户确认后开始执行
2. **Phase 1** - 立即修复 Critical 问题
3. **迭代修复** - 按优先级逐步修复

---

**计划状态**: 待用户确认
**预计总时间**: 6.5 - 13 小时
