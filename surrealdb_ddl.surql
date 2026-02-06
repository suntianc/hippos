-- 个人 Agent 记忆系统 DDL (SurrealDB)
-- 约定：tenant_id/user_id/agent_id/session_id 为内部标准字段，个人版由服务端默认注入。

DEFINE NAMESPACE IF NOT EXISTS mem_ns;
DEFINE DATABASE IF NOT EXISTS mem_db;

USE NS mem_ns DB mem_db;

-- ========================
-- 1) event_log
-- ========================
DEFINE TABLE IF NOT EXISTS event_log SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE event_log TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE event_log TYPE string;
DEFINE FIELD IF NOT EXISTS agent_id ON TABLE event_log TYPE string;
DEFINE FIELD IF NOT EXISTS session_id ON TABLE event_log TYPE string;
DEFINE FIELD IF NOT EXISTS content ON TABLE event_log TYPE string;
DEFINE FIELD IF NOT EXISTS event_type ON TABLE event_log TYPE string ASSERT $value INSIDE ['utterance','action','feedback','import'];
DEFINE FIELD IF NOT EXISTS source ON TABLE event_log TYPE string DEFAULT 'api';
DEFINE FIELD IF NOT EXISTS ts ON TABLE event_log TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS meta ON TABLE event_log FLEXIBLE TYPE object;

DEFINE INDEX IF NOT EXISTS idx_event_tenant_user_ts ON TABLE event_log COLUMNS tenant_id, user_id, ts;
DEFINE INDEX IF NOT EXISTS idx_event_session_ts ON TABLE event_log COLUMNS session_id, ts;

-- ========================
-- 2) recall_chunk (vector)
-- ========================
DEFINE TABLE IF NOT EXISTS recall_chunk SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE recall_chunk TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE recall_chunk TYPE string;
DEFINE FIELD IF NOT EXISTS text ON TABLE recall_chunk TYPE string;
DEFINE FIELD IF NOT EXISTS embedding ON TABLE recall_chunk TYPE array<float>;
DEFINE FIELD IF NOT EXISTS event_ref ON TABLE recall_chunk TYPE record<event_log>;
DEFINE FIELD IF NOT EXISTS decay_score ON TABLE recall_chunk TYPE float DEFAULT 1.0;
DEFINE FIELD IF NOT EXISTS expire_at ON TABLE recall_chunk TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE recall_chunk TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_recall_tenant_user_created ON TABLE recall_chunk COLUMNS tenant_id, user_id, created_at;

-- 向量索引（不同 SurrealDB 版本语法可能有差异，按运行版本调整参数）
DEFINE INDEX IF NOT EXISTS idx_recall_embedding
ON TABLE recall_chunk
FIELDS embedding
HNSW DIMENSION 1536 DIST COSINE;

-- ========================
-- 3) fact_candidate
-- ========================
DEFINE TABLE IF NOT EXISTS fact_candidate SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE fact_candidate TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE fact_candidate TYPE string;
DEFINE FIELD IF NOT EXISTS fact_key ON TABLE fact_candidate TYPE string;
DEFINE FIELD IF NOT EXISTS fact_value ON TABLE fact_candidate TYPE any;
DEFINE FIELD IF NOT EXISTS scope ON TABLE fact_candidate TYPE option<string>;
DEFINE FIELD IF NOT EXISTS confidence ON TABLE fact_candidate TYPE float DEFAULT 0.0;
DEFINE FIELD IF NOT EXISTS ttl_seconds ON TABLE fact_candidate TYPE int DEFAULT 604800;
DEFINE FIELD IF NOT EXISTS expire_at ON TABLE fact_candidate TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS status ON TABLE fact_candidate TYPE string ASSERT $value INSIDE ['active','promoted','expired','rejected'] DEFAULT 'active';
DEFINE FIELD IF NOT EXISTS evidence_refs ON TABLE fact_candidate TYPE array<record<event_log>>;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE fact_candidate TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_candidate_tenant_user_key ON TABLE fact_candidate COLUMNS tenant_id, user_id, fact_key;
DEFINE INDEX IF NOT EXISTS idx_candidate_status_expire ON TABLE fact_candidate COLUMNS status, expire_at;

-- ========================
-- 4) fact_truth
-- ========================
DEFINE TABLE IF NOT EXISTS fact_truth SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE fact_truth TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE fact_truth TYPE string;
DEFINE FIELD IF NOT EXISTS fact_key ON TABLE fact_truth TYPE string;
DEFINE FIELD IF NOT EXISTS fact_value ON TABLE fact_truth TYPE any;
DEFINE FIELD IF NOT EXISTS scope ON TABLE fact_truth TYPE option<string>;
DEFINE FIELD IF NOT EXISTS version ON TABLE fact_truth TYPE int DEFAULT 1;
DEFINE FIELD IF NOT EXISTS valid_from ON TABLE fact_truth TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS valid_to ON TABLE fact_truth TYPE option<datetime>;
DEFINE FIELD IF NOT EXISTS status ON TABLE fact_truth TYPE string ASSERT $value INSIDE ['active','overridden','revoked'] DEFAULT 'active';
DEFINE FIELD IF NOT EXISTS source_trace ON TABLE fact_truth TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE fact_truth TYPE datetime DEFAULT time::now();
DEFINE FIELD IF NOT EXISTS updated_at ON TABLE fact_truth TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_truth_tenant_user_key_status ON TABLE fact_truth COLUMNS tenant_id, user_id, fact_key, status;
DEFINE INDEX IF NOT EXISTS idx_truth_valid_window ON TABLE fact_truth COLUMNS valid_from, valid_to;

-- 唯一约束建议：同租户+用户+fact_key 在 active 状态仅允许一条。
-- SurrealDB 对“条件唯一”支持与版本相关，V1 可在应用层事务保障。

-- ========================
-- 5) graph: entity + relation
-- ========================
DEFINE TABLE IF NOT EXISTS entity SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE entity TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE entity TYPE string;
DEFINE FIELD IF NOT EXISTS entity_type ON TABLE entity TYPE string ASSERT $value INSIDE ['goal','preference','context','concept'];
DEFINE FIELD IF NOT EXISTS name ON TABLE entity TYPE string;
DEFINE FIELD IF NOT EXISTS attrs ON TABLE entity FLEXIBLE TYPE object;

DEFINE INDEX IF NOT EXISTS idx_entity_tenant_user_type_name ON TABLE entity COLUMNS tenant_id, user_id, entity_type, name;

DEFINE TABLE IF NOT EXISTS relation TYPE RELATION IN entity OUT entity SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE relation TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE relation TYPE string;
DEFINE FIELD IF NOT EXISTS rel_type ON TABLE relation TYPE string ASSERT $value INSIDE ['depends_on','conflicts_with','applies_in','contains'];
DEFINE FIELD IF NOT EXISTS weight ON TABLE relation TYPE float DEFAULT 1.0;
DEFINE FIELD IF NOT EXISTS scope ON TABLE relation TYPE option<string>;
DEFINE FIELD IF NOT EXISTS source_trace ON TABLE relation TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE relation TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_relation_tenant_user_type ON TABLE relation COLUMNS tenant_id, user_id, rel_type;

-- ========================
-- 6) audit_log
-- ========================
DEFINE TABLE IF NOT EXISTS audit_log SCHEMAFULL;

DEFINE FIELD IF NOT EXISTS trace_id ON TABLE audit_log TYPE string;
DEFINE FIELD IF NOT EXISTS tenant_id ON TABLE audit_log TYPE string;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE audit_log TYPE string;
DEFINE FIELD IF NOT EXISTS action ON TABLE audit_log TYPE string ASSERT $value INSIDE ['propose','promote','reject','override','revoke','delete'];
DEFINE FIELD IF NOT EXISTS target_type ON TABLE audit_log TYPE string;
DEFINE FIELD IF NOT EXISTS target_id ON TABLE audit_log TYPE string;
DEFINE FIELD IF NOT EXISTS reason ON TABLE audit_log TYPE option<string>;
DEFINE FIELD IF NOT EXISTS operator ON TABLE audit_log TYPE string ASSERT $value INSIDE ['system','agent','user'];
DEFINE FIELD IF NOT EXISTS ts ON TABLE audit_log TYPE datetime DEFAULT time::now();

DEFINE INDEX IF NOT EXISTS idx_audit_trace_ts ON TABLE audit_log COLUMNS trace_id, ts;
DEFINE INDEX IF NOT EXISTS idx_audit_tenant_user_ts ON TABLE audit_log COLUMNS tenant_id, user_id, ts;

-- ========================
-- 7) 推荐默认值（个人版）
-- ========================
-- 应用层建议在写入时注入：
-- tenant_id = 'tenant:default'
-- user_id   = 'user:me'
-- agent_id  = 'agent:default'
-- session_id（若缺失）= 新建 conversation_id
