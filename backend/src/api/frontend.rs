use std::borrow::Cow;

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Method, StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

use super::error::ApiError;

// debug-embed makes debug binaries independent of the build machine's filesystem too.
#[derive(Embed)]
#[folder = "$OUT_DIR/frontend/"]
struct Frontend;

pub(super) async fn serve(method: Method, uri: Uri, headers: HeaderMap) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path == "api" || path.starts_with("api/") {
        return error(StatusCode::NOT_FOUND, "Endpoint not found.");
    }
    if method != Method::GET && method != Method::HEAD {
        let mut response = error(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method not allowed for this endpoint.",
        );
        response
            .headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
        return response;
    }
    // The app uses hash URLs, so only / needs an index; missing assets stay 404.
    let path = if path.is_empty() { "index.html" } else { path };
    let Some(file) = Frontend::get(path) else {
        return error(StatusCode::NOT_FOUND, "File not found.");
    };
    let hash: String = file
        .metadata
        .sha256_hash()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let etag = format!("\"{hash}\"");
    let unchanged = headers
        .get_all(header::IF_NONE_MATCH)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .any(|value| value.trim() == "*" || value.trim().trim_start_matches("W/") == etag);
    let length = file.data.len();
    let mime = HeaderValue::from_str(file.metadata.mimetype())
        .expect("embedded MIME types are valid headers");
    let mut response = if unchanged {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        let body = if method == Method::HEAD {
            Body::empty()
        } else {
            match file.data {
                Cow::Borrowed(data) => Body::from(data),
                Cow::Owned(data) => Body::from(data),
            }
        };
        let mut response = body.into_response();
        response.headers_mut().insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );
        response
    };
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, mime);
    headers.insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if path.starts_with("assets/") {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        }),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

fn error(status: StatusCode, message: &'static str) -> Response {
    let mut response = ApiError::new(status, message).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::{ApiConfig, AppState, router},
        session::SessionManager,
    };
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn get(path: &str, method: Method, etag: Option<&str>) -> Response {
        let state = AppState::new(SessionManager::new(), ApiConfig::default());
        let mut request = Request::builder().uri(path).method(method);
        if let Some(etag) = etag {
            request = request.header(header::IF_NONE_MATCH, etag);
        }
        router(state)
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn root_serves_the_built_app_and_embeds_its_referenced_assets() {
        assert!(matches!(
            Frontend::get("index.html").unwrap().data,
            Cow::Borrowed(_)
        ));
        let response = get("/", Method::GET, None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/html");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
        let html = response.into_body().collect().await.unwrap().to_bytes();
        let html = std::str::from_utf8(&html).unwrap();
        assert!(html.contains("Stadt Land Streetview"));
        assert!(!html.contains("/src/main.ts"));
        let script = html
            .split("src=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(script.starts_with("/assets/") && script.ends_with(".js"));
        let asset = get(script, Method::GET, None).await;
        assert_eq!(asset.status(), StatusCode::OK);
        assert!(
            asset.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .contains("javascript")
        );
        assert_eq!(
            asset.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        for extension in ["css", "woff2"] {
            let name = Frontend::iter()
                .find(|name| name.ends_with(&format!(".{extension}")))
                .unwrap();
            let response = get(&format!("/{name}"), Method::GET, None).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_ne!(response.headers()[header::CONTENT_TYPE], "text/html");
        }
        let favicon = get("/favicon.svg?version=1", Method::GET, None).await;
        assert_eq!(favicon.status(), StatusCode::OK);
        assert_eq!(favicon.headers()[header::CONTENT_TYPE], "image/svg+xml");
        assert_eq!(favicon.headers()[header::CACHE_CONTROL], "no-cache");
    }

    #[tokio::test]
    async fn head_and_conditional_requests_preserve_asset_metadata() {
        let response = get("/", Method::GET, None).await;
        let etag = response.headers()[header::ETAG]
            .to_str()
            .unwrap()
            .to_owned();
        let length = response.headers()[header::CONTENT_LENGTH].clone();
        let head = get("/index.html", Method::HEAD, None).await;
        assert_eq!(head.status(), StatusCode::OK);
        assert_eq!(head.headers()[header::CONTENT_LENGTH], length);
        assert!(
            head.into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
        let response = get("/", Method::GET, Some(&format!("\"other\", W/{etag}"))).await;
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert_eq!(response.headers()[header::ETAG], etag);
        assert!(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn missing_assets_api_paths_and_traversal_do_not_fall_back_to_html() {
        for path in [
            "/assets/missing.js",
            "/api",
            "/api/unknown",
            "/%2e%2e/Cargo.toml",
            "/src/main.ts",
            "/unknown",
        ] {
            let response = get(path, Method::GET, None).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        }
        let response = get("/", Method::POST, None).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers()[header::ALLOW], "GET, HEAD");
    }
}
