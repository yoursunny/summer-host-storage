use std::io;
use yoursunny_summer_host_storage::{BitCounts, download, upload};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "yoursunny_summer_host_storage")]
#[command(about = "Deep Atlantic Storage app")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Upload a file in offline mode")]
    Upload {},
    #[command(about = "Download a file in offline mode")]
    #[command(arg_required_else_help = true)]
    Download { cnt0: usize, cnt1: usize },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Upload {} => {
            let counts = upload(io::stdin()).unwrap();
            println!("{:} {:}", counts.cnt0, counts.cnt1)
        }
        Commands::Download { cnt0, cnt1 } => {
            let counts = BitCounts {
                cnt0: cnt0,
                cnt1: cnt1,
            };
            download(io::stdout(), &counts).unwrap();
        }
    }
}
