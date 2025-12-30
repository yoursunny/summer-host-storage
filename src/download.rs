use super::BitCounts;
use std::io;
use url::Url;

const PAGESZ: usize = 1024;
const BUF0: [u8; PAGESZ] = [0x00; PAGESZ];
const BUF1: [u8; PAGESZ] = [0xFF; PAGESZ];

pub fn download<T: io::Write>(mut w: T, counts: &BitCounts) -> Result<(), io::Error> {
    let bytes0 = counts.cnt0 / 8;
    let bytes1 = counts.cnt1 / 8;
    let middle0 = counts.cnt0 % 8;
    let middle1 = counts.cnt1 % 8;
    let pages0 = bytes0 / PAGESZ;
    let pages1 = bytes1 / PAGESZ;
    let bytes0 = bytes0 - pages0 * PAGESZ;
    let bytes1 = bytes1 - pages1 * PAGESZ;

    write_pages(&mut w, &BUF0, pages0)?;
    write_pages(&mut w, &BUF0[..bytes0], 1)?;
    if middle0 + middle1 > 0 {
        let b = [(0xFFu8 >> middle0)];
        write_pages(&mut w, &b, 1)?;
    }
    write_pages(&mut w, &BUF1[..bytes1], 1)?;
    write_pages(&mut w, &BUF1, pages1)?;
    Ok(())
}

fn write_pages<T: io::Write>(w: &mut T, page: &[u8], n: usize) -> Result<(), io::Error> {
    for _ in 0..n {
        w.write_all(page)?;
    }
    Ok(())
}

impl BitCounts {
    pub fn from_url(url: &str) -> Option<(BitCounts, String)> {
        let url = Url::parse(url).ok()?;
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

    #[test]
    fn whole_pages() {
        let mut buf = Vec::new();
        let counts = BitCounts {
            cnt0: 3 * PAGESZ * 8,
            cnt1: 2 * PAGESZ * 8,
        };
        download(&mut buf, &counts).unwrap();

        assert_eq!(buf.len(), 3 * PAGESZ + 2 * PAGESZ);
        let middle_index = 3 * PAGESZ;
        assert!(buf[..middle_index].iter().all(|v| *v == 0x00));
        assert!(buf[middle_index..].iter().all(|v| *v == 0xFF));
    }

    #[test]
    fn middle_bytes() {
        let mut buf = Vec::new();
        let counts = BitCounts {
            cnt0: (3 * PAGESZ + 5) * 8 + 6,
            cnt1: 2 + (7 + 2 * PAGESZ) * 8,
        };
        download(&mut buf, &counts).unwrap();

        assert_eq!(buf.len(), 3 * PAGESZ + 5 + 1 + 7 + 2 * PAGESZ);
        let middle_index = 3 * PAGESZ + 5;
        assert!(buf[..middle_index].iter().all(|v| *v == 0x00));
        assert_eq!(buf[middle_index], 0b00000011);
        assert!(buf[middle_index + 1..].iter().all(|v| *v == 0xFF));
    }

    #[test]
    fn from_url_success() {
        let url = "https://summer-host-storage.yoursunny.dev/3e9/2327/yoursunny.txt";
        let (counts, filename) = BitCounts::from_url(url).unwrap();
        assert_eq!(counts.cnt0, 1001);
        assert_eq!(counts.cnt1, 8999);
        assert_eq!(filename, "yoursunny.txt");
    }

    #[test]
    fn from_url_failure() {
        let urls = [
            "https://summer-host-storage.yoursunny.dev/3e9/2327",
            "https://summer-host-storage.yoursunny.dev/3e9/zzzz/yoursunny.txt",
            "https://summer-host-storage.yoursunny.dev/3e9/2327/extra/yoursunny.txt",
        ];
        for url in urls {
            assert!(BitCounts::from_url(url).is_none());
        }
    }
}
