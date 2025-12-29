use super::BitCounts;
use std::io;
use std::io::prelude::*;
use std::sync::OnceLock;

fn get_ones_table() -> &'static [u8; 256] {
    static ONES_TABLE: OnceLock<[u8; 256]> = OnceLock::new();
    ONES_TABLE.get_or_init(|| {
        let mut ones = [0; 256];
        for b in 0..256 {
            let mut cnt = 0;
            for s in 0..8 {
                if (b >> s) % 2 == 1 {
                    cnt += 1;
                }
            }
            ones[b] = cnt;
        }
        ones
    })
}

pub fn upload<T: io::Read>(r: T) -> Result<BitCounts, io::Error> {
    let ones_table = get_ones_table();

    let mut size = 0usize;
    let mut cnt1 = 0usize;

    let r = io::BufReader::new(r);
    for b in r.bytes() {
        size += 8;
        cnt1 += ones_table[b? as usize] as usize;
    }

    Ok(BitCounts {
        cnt0: size - cnt1,
        cnt1: cnt1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ones_table() {
        let ones_table = get_ones_table();
        assert_eq!(ones_table[0x00], 0);
        assert_eq!(ones_table[0x08], 1);
        assert_eq!(ones_table[0x11], 2);
        assert_eq!(ones_table[0x70], 3);
        assert_eq!(ones_table[0xC3], 4);
        assert_eq!(ones_table[0xFF], 8);
    }

    #[test]
    fn empty() {
        let input = [0u8; 0];
        let counts = upload(io::Cursor::new(&input)).unwrap();
        assert_eq!(counts.cnt0, 0);
        assert_eq!(counts.cnt1, 0);
    }

    #[test]
    fn same_octets() {
        let ones_table = get_ones_table();
        for b in 0..256 {
            let input = [b as u8; 100_000];
            let counts = upload(io::Cursor::new(&input)).unwrap();
            assert_eq!(counts.cnt1, (ones_table[b] as usize) * input.len());
            assert_eq!(counts.cnt0, 8 * input.len() - counts.cnt1);
        }
    }
}
