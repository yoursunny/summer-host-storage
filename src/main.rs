use std::{fs::File, io, path::Path};
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
    #[command(arg_required_else_help = true)]
    Upload {
        #[arg(help = "Filename to upload")]
        filename: String,
        #[arg(long, help = "Read content from stdin")]
        stdin: bool,
    },
    #[command(about = "Download a file in offline mode")]
    #[command(arg_required_else_help = true)]
    Download { cnt0: usize, cnt1: usize },
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Upload { filename, stdin } => {
            let counts = (if stdin {
                upload(io::stdin())
            } else {
                upload(File::open(&filename).unwrap())
            })
            .unwrap();
            let basename = Path::new(&filename).file_name().unwrap().to_str().unwrap();
            let url = counts.to_url(basename);
            println!("{}", url);
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
