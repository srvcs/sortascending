use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

/// This service's identity. `srvcs-sortascending` is a leaf: it depends on no
/// other service. It sorts a list of integers into ascending order entirely
/// with local logic.
pub const SERVICE: &str = "srvcs-sortascending";
pub const CONCERN: &str = "comparison: sort a list ascending";
pub const DEPENDS_ON: &[&str] = &[];

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    /// The list of integers to sort. Every element must be a JSON integer.
    #[schema(value_type = Object)]
    pub values: Vec<Value>,
}

#[derive(Serialize, ToSchema)]
pub struct SortAscendingResponse {
    #[schema(value_type = Object)]
    pub values: Vec<Value>,
    pub result: Vec<i64>,
}

/// The single concern: sort `values` into ascending order.
///
/// Returns `None` if any element is not a JSON integer; otherwise `Some` of the
/// elements read as `i64` and sorted ascending.
pub fn sort_ascending(values: &[Value]) -> Option<Vec<i64>> {
    let mut nums = Vec::with_capacity(values.len());
    for v in values {
        match v.as_i64() {
            Some(n) => nums.push(n),
            None => return None,
        }
    }
    nums.sort();
    Some(nums)
}

/// `POST /` — sort the list `values` into ascending order.
///
/// Reads each element as a JSON integer. If any element is not an integer the
/// request is rejected with `422`. Otherwise the integers are sorted ascending
/// and returned as `result`.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = SortAscendingResponse),
        (status = 422, description = "an element is not a valid integer")
    )
)]
pub async fn evaluate(Json(req): Json<EvalRequest>) -> Response {
    match sort_ascending(&req.values) {
        Some(result) => (
            StatusCode::OK,
            Json(json!({ "values": req.values, "result": result })),
        )
            .into_response(),
        None => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "values must be integers" })),
        )
            .into_response(),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, SortAscendingResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some(), "GET / documented");
        assert!(root.post.is_some(), "POST / documented");
    }

    #[test]
    fn index_reports_identity() {
        // Identity constants are the public contract of this leaf service.
        assert_eq!(SERVICE, "srvcs-sortascending");
        assert_eq!(CONCERN, "comparison: sort a list ascending");
        assert!(DEPENDS_ON.is_empty());
    }

    #[test]
    fn sorts_into_ascending_order() {
        assert_eq!(
            sort_ascending(&[json!(3), json!(1), json!(2)]),
            Some(vec![1, 2, 3])
        );
        assert_eq!(sort_ascending(&[json!(1)]), Some(vec![1]));
        assert_eq!(sort_ascending(&[]), Some(vec![]));
    }

    #[test]
    fn handles_negatives_and_duplicates() {
        assert_eq!(
            sort_ascending(&[json!(0), json!(-5), json!(3), json!(-5)]),
            Some(vec![-5, -5, 0, 3])
        );
    }

    #[test]
    fn already_sorted_is_unchanged() {
        assert_eq!(
            sort_ascending(&[json!(-2), json!(0), json!(7)]),
            Some(vec![-2, 0, 7])
        );
    }

    #[test]
    fn non_integer_element_is_rejected() {
        for bad in [
            json!("1"),
            json!(1.5),
            json!(true),
            json!(null),
            json!([1]),
            json!({ "v": 1 }),
        ] {
            assert_eq!(
                sort_ascending(&[json!(1), bad.clone()]),
                None,
                "{bad} should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn evaluate_returns_200_with_result() {
        let resp = evaluate(Json(EvalRequest {
            values: vec![json!(3), json!(1), json!(2)],
        }))
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn evaluate_returns_422_for_non_integer() {
        let resp = evaluate(Json(EvalRequest {
            values: vec![json!(1), json!(1.5)],
        }))
        .await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn index_reports_identity_over_http() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-sortascending");
        assert!(info.depends_on.is_empty());
    }
}
