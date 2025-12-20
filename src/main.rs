use std::io;
use yoursunny_summer_host_storage::{BitCounts, download};

fn main() {
    let counts = BitCounts { cnt0: 17, cnt1: 15 };
    download(io::stdout(), &counts).unwrap();
}
