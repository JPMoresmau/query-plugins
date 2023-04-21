use anyhow::Result;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use pretty_assertions::assert_eq;
use query_runner_server::app;
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn plugin_execute() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/plugins/test_collect2/postgres1")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "user_name": "john"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!(
            {
                "names": [
                    "name",
                    "email"
                ,],
                "values": [
                    [
                        "John Doe",
                        "john.doe@example.com"
                    ]
                ]
            }
        )
    );
    Ok(())
}

#[tokio::test]
async fn plugin_execute_missing_plugin() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/plugins/missing/postgres1")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "user_name": "john"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!(
            {
                "error": "plugin `missing` not found",
            }
        )
    );
    Ok(())
}

#[tokio::test]
async fn plugin_execute_missing_connection() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/plugins/test_collect2/missing")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "user_name": "john"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!(
            {
                "error": "connection `missing` not found",
            }
        )
    );
    Ok(())
}

#[tokio::test]
async fn plugin_execute_missing_parameter() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/plugins/test_collect2/postgres1")
                .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "unknown": "john"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!(
            {
                "error": "plugin `test_collect2` failed on connection `postgres1` while parsing parameters: no value provided for parameter `user_name`",
            }
        )
    );
    Ok(())
}
