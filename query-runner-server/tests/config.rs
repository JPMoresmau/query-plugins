use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use pretty_assertions::assert_eq;
use query_runner_server::app;
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn connections() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/connections")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!([
            {
                "name": "memory",
                "db_type": "sqlite",
            },
            {
                "name": "postgres1",
                "db_type": "postgres",
            }
        ])
    );
    Ok(())
}

#[tokio::test]
async fn plugins() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/plugins")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        body,
        json!([
            {
                "name": "test_collect",
                "description": "test plugin collecting results",
            },
            {
                "name": "test_collect2",
                "description": "test plugin collecting results",
            }
        ])
    );
    Ok(())
}

#[tokio::test]
async fn plugin_metadata_not_found() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/plugins/missing")
                .body(Body::empty())
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
async fn plugin_metadata() -> Result<()> {
    let app = app()?;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/plugins/test_collect")
                .body(Body::empty())
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
                "name": "test_collect",
                "description": "test plugin collecting results",
                "parameters": [
                    {
                        "name": "customer_id",
                        "type": "integer",
                    }
                ]
            }
        )
    );
    Ok(())
}
