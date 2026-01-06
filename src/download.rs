use super::{BitCounts, SERVER_BASE};
use tokio::io::{self, AsyncWrite, AsyncWriteExt, BufWriter};
use url::Url;

const PAGESZ: usize = 1024;
const BUF0: [u8; PAGESZ] = [0x00; PAGESZ];
const BUF1: [u8; PAGESZ] = [0xFF; PAGESZ];

pub async fn download<T: AsyncWrite + Unpin>(w: T, counts: &BitCounts) -> Result<(), io::Error> {
    let bytes0 = counts.cnt0 / 8;
    let bytes1 = counts.cnt1 / 8;
    let middle0 = counts.cnt0 % 8;
    let middle1 = counts.cnt1 % 8;
    let pages0 = bytes0 / PAGESZ;
    let pages1 = bytes1 / PAGESZ;
    let bytes0 = bytes0 - pages0 * PAGESZ;
    let bytes1 = bytes1 - pages1 * PAGESZ;

    let mut w = BufWriter::new(w);
    write_pages(&mut w, &BUF0, pages0).await?;
    write_pages(&mut w, &BUF0[..bytes0], 1).await?;
    if middle0 + middle1 > 0 {
        let b = [(0xFFu8 >> middle0)];
        write_pages(&mut w, &b, 1).await?;
    }
    write_pages(&mut w, &BUF1[..bytes1], 1).await?;
    write_pages(&mut w, &BUF1, pages1).await?;
    w.flush().await?;
    Ok(())
}

async fn write_pages<T: AsyncWrite + Unpin>(
    w: &mut T,
    page: &[u8],
    n: usize,
) -> Result<(), io::Error> {
    for _ in 0..n {
        w.write_all(page).await?;
    }
    Ok(())
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
