//! 可观测性模块
//!
//! 提供 Prometheus 指标、结构化日志和健康检查。

use axum::{Json, Router, response::IntoResponse, routing::get};

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::Mutex;

// ===== Simple Metrics (using atomics for zero-dep implementation) =====

/// 简单应用指标
#[derive(Clone, Default)]
pub struct AppMetrics {
    pub http_requests_total: Arc<AtomicU64>,
    pub http_request_duration_sum: Arc<AtomicU64>,
    pub active_connections: Arc<AtomicUsize>,
    pub sessions_active: Arc<AtomicUsize>,
    pub sessions_archived: Arc<AtomicUsize>,
    pub turns_total: Arc<AtomicU64>,
    pub search_requests_total: Arc<AtomicU64>,
    pub search_latency_sum: Arc<AtomicU64>,
    pub errors_total: Arc<AtomicU64>,
}

impl AppMetrics {
    /// 记录 HTTP 请求
    pub fn record_http_request(&self, duration_ms: u64) {
        self.http_requests_total.fetch_add(1, Ordering::SeqCst);
        self.http_request_duration_sum
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// 记录活跃连接
    pub fn record_connection(&self, delta: isize) {
        self.active_connections
            .fetch_add(delta as usize, Ordering::SeqCst);
    }

    /// 记录会话变化
    pub fn record_session(&self, status: &str, delta: isize) {
        let _ = match status {
            "active" => self
                .sessions_active
                .fetch_add(delta as usize, Ordering::SeqCst),
            "archived" => self
                .sessions_archived
                .fetch_add(delta as usize, Ordering::SeqCst),
            _ => return,
        };
    }

    /// 记录搜索请求
    pub fn record_search(&self, duration_ms: u64) {
        self.search_requests_total.fetch_add(1, Ordering::SeqCst);
        self.search_latency_sum
            .fetch_add(duration_ms, Ordering::SeqCst);
    }

    /// 记录错误
    pub fn record_error(&self) {
        self.errors_total.fetch_add(1, Ordering::SeqCst);
    }

    /// 生成 Prometheus 格式指标
    pub fn gather(&self) -> String {
        format!(
            r#"# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total {}
# HELP http_request_duration_seconds HTTP request duration in seconds
# TYPE http_request_duration_seconds histogram
http_request_duration_seconds_sum {}
http_request_duration_seconds_count {}
# HELP active_connections Active HTTP connections
# TYPE active_connections gauge
active_connections {}
# HELP sessions_active Active sessions
# TYPE sessions_active gauge
sessions_active {}
# HELP sessions_archived Archived sessions
# TYPE sessions_archived gauge
sessions_archived {}
# HELP turns_total Total turns
# TYPE turns_total counter
turns_total {}
# HELP search_requests_total Total search requests
# TYPE search_requests_total counter
search_requests_total {}
# HELP search_latency_seconds Search request latency in seconds
# TYPE search_latency_seconds histogram
search_latency_seconds_sum {}
search_latency_seconds_count {}
# HELP errors_total Total errors
# TYPE errors_total counter
errors_total {}
"#,
            self.http_requests_total.load(Ordering::SeqCst),
            self.http_request_duration_sum.load(Ordering::SeqCst) as f64 / 1000.0,
            self.http_requests_total.load(Ordering::SeqCst),
            self.active_connections.load(Ordering::SeqCst),
            self.sessions_active.load(Ordering::SeqCst),
            self.sessions_archived.load(Ordering::SeqCst),
            self.turns_total.load(Ordering::SeqCst),
            self.search_requests_total.load(Ordering::SeqCst),
            self.search_latency_sum.load(Ordering::SeqCst) as f64 / 1000.0,
            self.search_requests_total.load(Ordering::SeqCst),
            self.errors_total.load(Ordering::SeqCst),
        )
    }
}

// ===== Health Check =====

/// 健康检查状态
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime_seconds: f64,
    pub checks: Vec<HealthCheck>,
}

/// 单个健康检查项
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

/// 健康检查结果
#[derive(Clone)]
pub struct HealthCheckResult {
    pub name: String,
    pub healthy: bool,
    pub message: String,
    pub latency_ms: u64,
}

/// 应用状态（用于健康检查）
#[derive(Clone)]
pub struct ObservabilityState {
    pub metrics: Arc<AppMetrics>,
    pub health_checks: Arc<Mutex<Vec<HealthCheckResult>>>,
    pub start_time: DateTime<Utc>,
    pub version: String,
}

impl ObservabilityState {
    pub fn new(version: String) -> Self {
        let metrics = Arc::new(AppMetrics::default());

        Self {
            metrics,
            health_checks: Arc::new(Mutex::new(Vec::new())),
            start_time: Utc::now(),
            version,
        }
    }

    /// 添加健康检查结果
    pub async fn add_health_check(&self, result: HealthCheckResult) {
        let mut checks = self.health_checks.lock().await;
        checks.push(result);
        if checks.len() > 10 {
            checks.remove(0);
        }
    }

    /// 获取应用正常运行时间
    pub fn uptime_seconds(&self) -> f64 {
        (Utc::now() - self.start_time).num_seconds() as f64
    }
}

// ===== Health Check Handlers =====

/// 获取完整健康状态
pub async fn health_check(
    state: axum::extract::State<Arc<ObservabilityState>>,
) -> impl IntoResponse {
    let checks = state.health_checks.lock().await;
    let all_healthy = checks.iter().all(|c| c.healthy);

    let health_status = HealthStatus {
        status: if all_healthy {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        timestamp: Utc::now().to_rfc3339(),
        version: state.version.clone(),
        uptime_seconds: state.uptime_seconds(),
        checks: checks
            .iter()
            .map(|c| HealthCheck {
                name: c.name.clone(),
                status: if c.healthy {
                    "healthy".to_string()
                } else {
                    "unhealthy".to_string()
                },
                message: Some(c.message.clone()),
                latency_ms: Some(c.latency_ms),
            })
            .collect(),
    };

    let status_code = if all_healthy {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health_status))
}

/// 简单存活检查
pub async fn liveness() -> impl IntoResponse {
    "OK"
}

/// 就绪检查（检查依赖服务）
pub async fn readiness(state: axum::extract::State<Arc<ObservabilityState>>) -> impl IntoResponse {
    let checks = state.health_checks.lock().await;
    let all_healthy = checks.iter().all(|c| c.healthy);

    if all_healthy {
        (axum::http::StatusCode::OK, "Ready")
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "Not Ready")
    }
}

/// Prometheus 指标端点
pub async fn metrics(state: axum::extract::State<Arc<ObservabilityState>>) -> impl IntoResponse {
    let output = state.metrics.gather();
    (axum::http::StatusCode::OK, output)
}

/// 版本信息端点
pub async fn version(state: axum::extract::State<Arc<ObservabilityState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "version": state.version,
        "uptime_seconds": state.uptime_seconds(),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

/// 创建可观测性路由
pub fn create_observability_router(state: Arc<ObservabilityState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness))
        .route("/metrics", get(metrics))
        .route("/version", get(version))
        .with_state(state)
}

// ===== Structured Logging =====

/// 初始化结构化日志
pub fn init_tracing(service_name: &str) {
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| format!("info,{}", service_name));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}

// ===== Request Metrics Middleware =====

/// 记录请求指标的中间件
pub async fn metrics_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
    state: Arc<ObservabilityState>,
) -> Result<axum::response::Response, std::convert::Infallible> {
    let start = std::time::Instant::now();

    state.metrics.record_connection(1);

    let response = next.run(req).await;

    let duration_ms = start.elapsed().as_millis() as u64;
    state.metrics.record_http_request(duration_ms);
    state.metrics.record_connection(-1);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_gather() {
        let metrics = AppMetrics::default();
        metrics.record_http_request(100);
        metrics.record_connection(1);
        metrics.record_search(50);
        metrics.record_error();

        let output = metrics.gather();
        assert!(output.contains("http_requests_total 1"));
        assert!(output.contains("active_connections 1"));
        assert!(output.contains("search_requests_total 1"));
        assert!(output.contains("errors_total 1"));
    }

    #[test]
    fn test_health_status_structure() {
        let status = HealthStatus {
            status: "healthy".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600.0,
            checks: vec![],
        };

        assert_eq!(status.status, "healthy");
        assert_eq!(status.version, "1.0.0");
    }

    #[test]
    fn test_health_check_structure() {
        let check = HealthCheck {
            name: "database".to_string(),
            status: "healthy".to_string(),
            message: Some("Connected".to_string()),
            latency_ms: Some(10),
        };

        assert_eq!(check.name, "database");
        assert!(check.message.is_some());
    }
}
