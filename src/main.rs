use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use std::path::Path;
use tokio::{fs::File, io};
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
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Upload { filename, stdin } => {
            let counts = (if stdin {
                upload(io::stdin()).await
            } else {
                let mut file = File::open(&filename).await?;
                upload(&mut file).await
            })?;

            let basename = to_basename(&filename).ok_or(anyhow!("bad filename"))?;
            let url = counts.to_url(basename);
            println!("{}", url);
        }

        Commands::Download { url, stdout } => {
            let (counts, filename) = BitCounts::from_url(&url).ok_or(anyhow!("invalid URL"))?;

            if stdout {
                download(io::stdout(), &counts).await
            } else {
                let basename = to_basename(&filename).ok_or(anyhow!("bad filename"))?;
                let mut file = File::create_new(basename).await?;
                download(&mut file, &counts).await
            }?;
        }

        Commands::Serve { bind } => {
            serve(&bind).await?;
        }
    }

    Ok(())
}

fn to_basename<'a>(filename: &'a str) -> Option<&'a str> {
    Path::new(filename).file_name()?.to_str()
}
