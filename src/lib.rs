mod download;
mod serve;
mod upload;

pub use download::download;
pub use serve::serve;
pub use upload::upload;

pub struct BitCounts {
    pub cnt0: usize,
    pub cnt1: usize,
}
