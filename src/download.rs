use super::{BitCounts, SERVER_BASE};
use bytes::Bytes;
use futures_util::{Stream, StreamExt, stream};
use std::{cmp::min, convert::Infallible};
use tokio::io::{self, AsyncWrite, AsyncWriteExt};
use url::Url;

const PAGESZ: usize = 1024;
const BUF0: [u8; PAGESZ] = [0x00; PAGESZ];
const BUF1: [u8; PAGESZ] = [0xFF; PAGESZ];

pub async fn download<T: AsyncWrite + Unpin>(
    mut w: T,
    counts: &BitCounts,
) -> Result<(), io::Error> {
    let mut stream = download_stream(counts);
    while let Some(Ok(chunk)) = stream.next().await {
        w.write_all(&chunk).await?;
    }
    w.flush().await?;
    Ok(())
}

pub fn download_stream(
    counts: &BitCounts,
) -> impl Stream<Item = Result<Bytes, Infallible>> + Unpin + 'static {
    let stream = stream::unfold(*counts, |counts| async move {
        let bytes0 = counts.cnt0 / 8;
        let bytes1 = counts.cnt1 / 8;
        let middle0 = counts.cnt0 % 8;
        let middle1 = counts.cnt1 % 8;

        if bytes0 > 0 {
            let n = min(PAGESZ, bytes0);
            return Some((
                Ok(Bytes::from_static(&BUF0[..n])),
                BitCounts {
                    cnt0: counts.cnt0 - 8 * n,
                    cnt1: counts.cnt1,
                },
            ));
        }
        if middle0 + middle1 > 0 {
            let b = [(0xFFu8 >> middle0)];
            return Some((
                Ok(Bytes::copy_from_slice(&b)),
                BitCounts {
                    cnt0: 0,
                    cnt1: counts.cnt1 - middle1,
                },
            ));
        }
        if bytes1 > 0 {
            let n = min(PAGESZ, bytes1);
            return Some((
                Ok(Bytes::from_static(&BUF1[..n])),
                BitCounts {
                    cnt0: 0,
                    cnt1: counts.cnt1 - 8 * n,
                },
            ));
        }
        return None;
    });
    Box::pin(stream)
}

impl BitCounts {
    pub fn from_url(url: &str) -> Option<(BitCounts, String)> {
        let base = Url::parse(SERVER_BASE).unwrap();
        let url = base.join(url).ok()?;
        let segments: Vec<&str> = url.path_segments()?.collect();
        if segments.len() != 3 {
            return None;
        }
        let cnt0 = usize::from_str_radix(segments[0], 16).ok()?;
        let cnt1 = usize::from_str_radix(segments[1], 16).ok()?;
        let filename = segments[2];
        Some((BitCounts { cnt0, cnt1 }, filename.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn whole_pages() {
        let mut buf = Vec::new();
        let counts = BitCounts {
            cnt0: 3 * PAGESZ * 8,
            cnt1: 2 * PAGESZ * 8,
        };
        download(&mut buf, &counts).await.unwrap();

        assert_eq!(buf.len(), 3 * PAGESZ + 2 * PAGESZ);
        let middle_index = 3 * PAGESZ;
        assert!(buf[..middle_index].iter().all(|v| *v == 0x00));
        assert!(buf[middle_index..].iter().all(|v| *v == 0xFF));
    }

    #[tokio::test]
    async fn middle_bytes() {
        let mut buf = Vec::new();
        let counts = BitCounts {
            cnt0: (3 * PAGESZ + 5) * 8 + 6,
            cnt1: 2 + (7 + 2 * PAGESZ) * 8,
        };
        download(&mut buf, &counts).await.unwrap();

        assert_eq!(buf.len(), 3 * PAGESZ + 5 + 1 + 7 + 2 * PAGESZ);
        let middle_index = 3 * PAGESZ + 5;
        assert!(buf[..middle_index].iter().all(|v| *v == 0x00));
        assert_eq!(buf[middle_index], 0b00000011);
        assert!(buf[middle_index + 1..].iter().all(|v| *v == 0xFF));
    }

    #[test]
    fn from_url_success() {
        let urls = [
            "/3e9/2327/1.bin",
            "https://summer-host-storage.yoursunny.dev/3e9/2327/1.bin",
            "http://[::1]:3000/3e9/2327/1.bin",
        ];
        for url in urls {
            let (counts, filename) = BitCounts::from_url(url).unwrap();
            assert_eq!(counts.cnt0, 1001);
            assert_eq!(counts.cnt1, 8999);
            assert_eq!(filename, "1.bin");
        }
    }

    #[test]
    fn from_url_failure() {
        let urls = [
            "https://summer-host-storage.yoursunny.dev/3e9/2327",
            "https://summer-host-storage.yoursunny.dev/3e9/zzzz/1.bin",
            "https://summer-host-storage.yoursunny.dev/3e9/2327/extra/1.bin",
        ];
        for url in urls {
            assert!(BitCounts::from_url(url).is_none());
        }
    }
}
