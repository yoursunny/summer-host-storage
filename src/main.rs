use clap::{Parser, Subcommand};
use std::{fs::File, io, path::Path};
use yoursunny_summer_host_storage::{BitCounts, download, serve, upload};

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
    Download {
        #[arg(help = "URL to download")]
        url: String,
        #[arg(long, help = "Write content to stdout")]
        stdout: bool,
    },
    #[command(about = "Serve the storage server")]
    Serve {
        #[arg(long, default_value = "[::1]:3000", help = "Port to serve on")]
        bind: String,
    },
}

#[tokio::main]
async fn main() {
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
        Commands::Download { url, stdout } => {
            use tokio::{fs::File, io};
            let (counts, filename) = BitCounts::from_url(&url).unwrap();
            if stdout {
                download(io::stdout(), &counts).await
            } else {
                let mut file = File::create_new(filename).await.unwrap();
                download(&mut file, &counts).await
            }
            .unwrap();
        }
        Commands::Serve { bind } => {
            serve(&bind).await;
        }
    }
}
