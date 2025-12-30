use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::io;
use std::net::{Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;

use super::{BitCounts, download};

#[tokio::main]
pub async fn serve(port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from((Ipv6Addr::LOCALHOST, port));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

type AppBody = BoxBody<Bytes, io::Error>;

async fn handler(req: Request<hyper::body::Incoming>) -> Result<Response<AppBody>, hyper::Error> {
    if req.method() != &Method::GET {
        return Ok(not_found());
    }

    let Some((counts, _)) = BitCounts::from_url(&req.uri().to_string()) else {
        return Ok(not_found());
    };

    let mut buf = Vec::new();
    download(&mut buf, &counts).unwrap();

    let mut rsp = Response::new(Full::from(buf).map_err(|never| match never {}).boxed());
    rsp.headers_mut().insert(
        "Content-Disposition",
        HeaderValue::from_static("attachment"),
    );
    Ok(rsp)
}

fn not_found() -> Response<AppBody> {
    let body = Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed();
    let mut rsp = Response::new(body);
    *rsp.status_mut() = StatusCode::NOT_FOUND;
    rsp
}

#[cfg(test)]
mod tests {
    use super::*;
}
