use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_sortascending::{health, router, telemetry};
use tower::ServiceExt;

fn app() -> axum::Router {
    router(telemetry::metrics_handle_for_tests())
}

async fn status_of(uri: &str) -> StatusCode {
    app()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

/// POST `{ "values": <values> }` to `/` and return (status, parsed JSON).
async fn eval(values: Value) -> (StatusCode, Value) {
    let res = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "values": values }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

// --- Standard srvcs service surface ---

#[tokio::test]
async fn index_ok() {
    assert_eq!(status_of("/").await, StatusCode::OK);
}

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

// --- Sort cases ---

#[tokio::test]
async fn sorts_three_two_one() {
    let (status, body) = eval(json!([3, 1, 2])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!([1, 2, 3]));
    assert_eq!(body["values"], json!([3, 1, 2]));
}

#[tokio::test]
async fn singleton_is_unchanged() {
    let (status, body) = eval(json!([7])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!([7]));
}

#[tokio::test]
async fn empty_list_is_empty() {
    let (status, body) = eval(json!([])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!([]));
}

#[tokio::test]
async fn already_sorted_is_unchanged() {
    let (status, body) = eval(json!([-2, 0, 7])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!([-2, 0, 7]));
}

#[tokio::test]
async fn negatives_and_duplicates_are_ordered() {
    let (status, body) = eval(json!([0, -5, 3, -5])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["result"], json!([-5, -5, 0, 3]));
}

// --- Error / edge cases ---

#[tokio::test]
async fn non_integer_element_is_422() {
    let (status, body) = eval(json!([1, "nope", 3])).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "values must be integers");
}

#[tokio::test]
async fn float_element_is_422() {
    let (status, body) = eval(json!([1, 1.5])).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "values must be integers");
}

#[tokio::test]
async fn missing_values_field_is_422() {
    // A body without the `values` field is a client error, not a 500.
    let res = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "notvalues": [1] }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let res = app()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}
