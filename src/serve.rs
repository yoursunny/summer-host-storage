use super::{BitCounts, download, upload};
use anyhow::Result;
use axum::{
    Router,
    body::Bytes,
    extract::Path,
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::body::AsyncReadBody;
use std::io::Cursor;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};

pub async fn serve(bind: &str) -> Result<()> {
    let app = app();
    let listener = TcpListener::bind(bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn app() -> Router {
    Router::new()
        .route("/{cnt0}/{cnt1}/{filename}", get(download_handler))
        .route("/upload/{filename}", post(upload_handler))
}

async fn download_handler(uri: Uri) -> Result<impl IntoResponse, impl IntoResponse> {
    let Some((counts, _)) = BitCounts::from_url(&uri.to_string()) else {
        return Err(StatusCode::NOT_FOUND);
    };

    let hdr0 = [
        ("Content-Disposition", "attachment"),
        ("Content-Type", "application/octet-stream"),
    ];
    let hdr1 = [("Content-Length", counts.total_bytes().to_string())];

    let (receiver, mut sender) = io::simplex(8192);
    tokio::spawn(async move {
        download(&mut sender, &counts).await?;
        sender.shutdown().await?;
        Ok::<(), anyhow::Error>(())
    });
    let body = AsyncReadBody::new(receiver);
    Ok((hdr0, hdr1, body))
}

async fn upload_handler(
    Path(filename): Path<String>,
    body: Bytes,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let Ok(counts) = upload(Cursor::new(body)).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let url = counts.to_url(&filename);
    Ok((StatusCode::CREATED, [("Location", url)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
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

    #[tokio::test]
    async fn upload() {
        let app = app();
        let req = Request::post("/upload/yoursunny.txt")
            .body(Body::from(Bytes::from_owner([
                0x0F, 0xFF, 0xFC, 0x00, 0xFF, 0x71,
            ])))
            .unwrap();
        let rsp = app.oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::CREATED);

        let hdr = rsp.headers();
        let location = hdr.get("Location").unwrap().to_str().unwrap();
        assert!(location.ends_with("/12/1e/yoursunny.txt"))
    }
}
