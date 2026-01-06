mod download;
mod serve;
mod upload;

pub use download::download;
pub use serve::serve;
pub use upload::upload;

const SERVER_BASE: &str = "https://summer-host-storage.yoursunny.dev";

pub struct BitCounts {
    pub cnt0: usize,
    pub cnt1: usize,
}

impl BitCounts {
    pub fn total_bytes(&self) -> usize {
        return (self.cnt0 + self.cnt1) / 8;
    }
}
