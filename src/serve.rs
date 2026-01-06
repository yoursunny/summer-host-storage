use anyhow::Result;
use axum::{
    Router,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::get,
};
use axum_extra::body::AsyncReadBody;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};

use super::{BitCounts, download};

pub async fn serve(bind: &str) -> Result<()> {
    let app = app();
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn app() -> Router {
    Router::new().route("/{cnt0}/{cnt1}/{filename}", get(handler))
}

async fn handler(uri: Uri) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some((counts, _)) = BitCounts::from_url(&uri.to_string()) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let hdr0 = [
        ("Content-Disposition", "attachment"),
        ("Content-Type", "application/octet-stream"),
    ];
    let hdr1 = [("Content-Length", format!("{}", counts.total_bytes()))];

    let (receiver, mut sender) = io::simplex(8192);
    tokio::spawn(async move {
        download(&mut sender, &counts).await?;
        sender.shutdown().await?;
        Ok::<(), anyhow::Error>(())
    });
    let body = AsyncReadBody::new(receiver);
    Ok((hdr0, hdr1, body))
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
    async fn download() {
        let app = app();
        let req = Request::get("/12/1e/yoursunny.txt")
            .body(Body::empty())
            .unwrap();
        let rsp = app.oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::OK);

        let hdr = rsp.headers();
        assert_eq!(hdr.get("Content-Type").unwrap(), "application/octet-stream");
        assert_eq!(hdr.get("Content-Disposition").unwrap(), "attachment");
        assert_eq!(hdr.get("Content-Length").unwrap(), "6");

        let body = rsp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            Bytes::from_owner([0x00, 0x00, 0x3F, 0xFF, 0xFF, 0xFF])
        );
    }
}
