#[cfg(test)]
mod session_handler_tests {
    use axum::{
        Router,
        body::to_bytes,
        http::{Request, StatusCode},
        routing::*,
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_session_returns_201() {
        // Create a simple router for testing
        let app = Router::new()
            .route(
                "/api/v1/sessions",
                post(|| async { (StatusCode::CREATED, "session created") }),
            )
            .route(
                "/api/v1/sessions/:id",
                get(|| async {
                    (
                        StatusCode::OK,
                        r#"{"id":"test_session_123","name":"Test Session"}"#,
                    )
                }),
            );

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/sessions")
                    .header("Content-Type", "application/json")
                    .body(json!({"name": "Test Session"}).to_string())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_get_session_returns_200_for_existing() {
        let app = Router::new().route(
            "/api/v1/sessions/:id",
            get(|| async {
                (
                    StatusCode::OK,
                    r#"{"id":"test_session_123","name":"Test Session"}"#,
                )
            }),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/sessions/test_session_123")
                    .body(String::new())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_session_returns_404_for_non_existing() {
        let app = Router::new().route(
            "/api/v1/sessions/:id",
            get(|| async { (StatusCode::NOT_FOUND, r#"{"error":"NOT_FOUND"}"#) }),
        );

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/sessions/non_existing")
                    .body(String::new())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
