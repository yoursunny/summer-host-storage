use super::{BitCounts, download, upload};
use anyhow::Result;
use axum::{
    Router,
    extract::{Path, Request},
    http::{StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::body::AsyncReadBody;
use futures_util::TryStreamExt;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};
use tokio_util::io::StreamReader;

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
    req: Request,
) -> Result<impl IntoResponse, impl IntoResponse> {
    use std::io;
    let body_stream = req
        .into_body()
        .into_data_stream()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    let mut reader = StreamReader::new(body_stream);

    let Ok(counts) = upload(&mut reader).await else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let url = counts.to_url(&filename);
    Ok((StatusCode::CREATED, [("Location", url)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use bytes::Bytes;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn download() {
        let req = Request::get("/22/2e/yoursunny.txt")
            .body(Body::empty())
            .unwrap();
        let rsp = app().oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::OK);

        let hdr = rsp.headers();
        assert_eq!(hdr.get("Content-Type").unwrap(), "application/octet-stream");
        assert_eq!(hdr.get("Content-Disposition").unwrap(), "attachment");
        assert_eq!(hdr.get("Content-Length").unwrap(), "10");

        let body = rsp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            body,
            Bytes::from_owner([0x00, 0x00, 0x00, 0x00, 0x3F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
        );
    }

    #[tokio::test]
    async fn upload_small() {
        let req = Request::post("/upload/yoursunny.txt")
            .body(String::from("@yoursunny"))
            .unwrap();
        let rsp = app().oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::CREATED);

        let hdr = rsp.headers();
        let location = hdr.get("Location").unwrap().to_str().unwrap();
        assert!(location.ends_with("/22/2e/yoursunny.txt"))
    }

    #[tokio::test]
    async fn upload_large() {
        let req_body = vec![0x70u8; 7_000_000];
        let req = Request::post("/upload/1.bin")
            .body(Body::from(req_body))
            .unwrap();
        let rsp = app().oneshot(req).await.unwrap();

        assert_eq!(rsp.status(), StatusCode::CREATED);

        let hdr = rsp.headers();
        let location = hdr.get("Location").unwrap().to_str().unwrap();
        assert!(location.ends_with("/2160ec0/1406f40/1.bin"))
    }
}
