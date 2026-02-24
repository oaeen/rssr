use reqwest::header::{IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FetchedFeed {
    pub body: Vec<u8>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FetchStatus {
    Updated(FetchedFeed),
    NotModified,
}

#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("unexpected status code: {0}")]
    HttpStatus(u16),
}

pub async fn fetch_feed(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<FetchStatus, FetchError> {
    let mut request = client.get(url);
    if let Some(value) = etag {
        request = request.header(IF_NONE_MATCH, value);
    }
    if let Some(value) = last_modified {
        request = request.header(IF_MODIFIED_SINCE, value);
    }

    let response = request.send().await?;
    let status = response.status();
    if status.as_u16() == 304 {
        return Ok(FetchStatus::NotModified);
    }
    if !status.is_success() {
        return Err(FetchError::HttpStatus(status.as_u16()));
    }

    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let last_modified = response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    let body = response.bytes().await?.to_vec();

    Ok(FetchStatus::Updated(FetchedFeed {
        body,
        content_type,
        etag,
        last_modified,
    }))
}

pub async fn fetch_feed_with_retry(
    client: &reqwest::Client,
    url: &str,
    etag: Option<&str>,
    last_modified: Option<&str>,
    max_retries: usize,
) -> Result<FetchStatus, FetchError> {
    let mut attempt = 0_usize;
    loop {
        match fetch_feed(client, url, etag, last_modified).await {
            Ok(result) => return Ok(result),
            Err(err) => {
                let should_retry = matches!(err, FetchError::Request(_))
                    || matches!(err, FetchError::HttpStatus(code) if code >= 500);
                if !should_retry || attempt >= max_retries {
                    return Err(err);
                }
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(40 * attempt as u64)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode};
    use axum::response::Response;
    use axum::routing::get;
    use axum::Router;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[derive(Clone)]
    struct AppState {
        request_count: Arc<AtomicUsize>,
    }

    async fn feed_handler(State(state): State<AppState>, headers: HeaderMap) -> Response {
        let counter = state.request_count.fetch_add(1, Ordering::SeqCst);
        let etag = "\"rssr-feed-v1\"";
        let last_modified = "Tue, 24 Feb 2026 10:00:00 GMT";

        if counter == 0 {
            let mut response =
                Response::new(axum::body::Body::from("temporary failure".to_string()));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response.headers_mut().insert(
                reqwest::header::CONTENT_TYPE,
                "text/plain".parse().expect("header must parse"),
            );
            return response;
        }

        if headers
            .get(IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
            == Some(etag)
        {
            let mut response = Response::new(axum::body::Body::empty());
            *response.status_mut() = StatusCode::NOT_MODIFIED;
            response
                .headers_mut()
                .insert(reqwest::header::ETAG, etag.parse().expect("header must parse"));
            response.headers_mut().insert(
                LAST_MODIFIED,
                last_modified.parse().expect("header must parse"),
            );
            return response;
        }

        let mut response = Response::new(axum::body::Body::from(
            include_str!("../../../../fixtures/import-samples/sample.rss.xml").to_string(),
        ));
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(
            reqwest::header::CONTENT_TYPE,
            "application/rss+xml"
                .parse()
                .expect("header must parse"),
        );
        response
            .headers_mut()
            .insert(reqwest::header::ETAG, etag.parse().expect("header must parse"));
        response.headers_mut().insert(
            LAST_MODIFIED,
            last_modified.parse().expect("header must parse"),
        );
        response
    }

    async fn spawn_test_server() -> (String, tokio::task::JoinHandle<()>) {
        let state = AppState {
            request_count: Arc::new(AtomicUsize::new(0)),
        };
        let app = Router::new()
            .route("/feed.xml", get(feed_handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let address = listener.local_addr().expect("local addr should exist");
        let join_handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("server should run");
        });
        (format!("http://{address}/feed.xml"), join_handle)
    }

    #[tokio::test]
    async fn fetch_feed_supports_retry_and_conditional_headers() {
        let (url, server_task) = spawn_test_server().await;
        let client = reqwest::Client::new();

        let first = fetch_feed_with_retry(&client, &url, None, None, 2)
            .await
            .expect("first fetch should succeed with retry");
        let updated = match first {
            FetchStatus::Updated(payload) => payload,
            FetchStatus::NotModified => panic!("first fetch should be updated"),
        };
        assert!(updated.body.starts_with(b"<?xml"));
        assert_eq!(updated.content_type.as_deref(), Some("application/rss+xml"));
        assert_eq!(updated.etag.as_deref(), Some("\"rssr-feed-v1\""));

        let second = fetch_feed_with_retry(
            &client,
            &url,
            updated.etag.as_deref(),
            updated.last_modified.as_deref(),
            0,
        )
        .await
        .expect("second fetch should succeed");
        assert!(matches!(second, FetchStatus::NotModified));

        server_task.abort();
    }
}
