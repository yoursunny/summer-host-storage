use axum::{
    Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};
use tokio::net::TcpListener;

use super::{BitCounts, download};

#[tokio::main]
pub async fn serve(bind: &str) {
    let app = app();
    let listener = TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn app() -> Router {
    Router::new().route("/{cnt0}/{cnt1}/{filename}", get(handler))
}

async fn handler(uri: Uri) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some((counts, _)) = BitCounts::from_url(&uri.to_string()) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let mut buf = Vec::new();
    download(&mut buf, &counts).unwrap();

    Ok(([("Content-Disposition", "attachment")], buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, Bytes},
        http::Request,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn download_empty() {
        let app = app();
        let req = Request::get("/12/1e/yoursunny.txt")
            .body(Body::empty())
            .unwrap();
        let rsp = app.oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::OK);
        assert_eq!(
            rsp.headers().get("Content-Type").unwrap(),
            "application/octet-stream"
        );
        assert_eq!(
            rsp.headers().get("Content-Disposition").unwrap(),
            "attachment"
        );
        let body = rsp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            Bytes::from_owner([0x00, 0x00, 0x3F, 0xFF, 0xFF, 0xFF])
        );
    }
}
