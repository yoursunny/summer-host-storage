use anyhow::{Result, anyhow};
use bpaf::Bpaf;
use std::path::Path;
use tokio::{fs::File, io};
use yoursunny_summer_host_storage::{BitCounts, download, serve, upload};

/// Deep Atlantic Storage app
#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, fallback_to_usage)]
enum Action {
    /// Upload a file in offline mode
    #[bpaf(command, fallback_to_usage)]
    Upload {
        /// Read content from stdin
        #[bpaf(switch)]
        stdin: bool,
        /// Filename to upload
        #[bpaf(positional("FILENAME"))]
        filename: String,
    },
    /// Download a file in offline mode
    #[bpaf(command, fallback_to_usage)]
    Download {
        /// Write content to stdout
        #[bpaf(switch)]
        stdout: bool,
        /// URL to download
        #[bpaf(positional("URL"))]
        url: String,
    },
    /// Serve the storage server
    #[bpaf(command)]
    Serve {
        /// Listen host:port
        #[bpaf(argument("HOSTPORT"), fallback(String::from("[::1]:3000")))]
        bind: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let act = action().run();

    match act {
        Action::Upload { filename, stdin } => {
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

        Action::Download { url, stdout } => {
            let (counts, filename) = BitCounts::from_url(&url).ok_or(anyhow!("invalid URL"))?;

            if stdout {
                download(io::stdout(), &counts).await
            } else {
                let basename = to_basename(&filename).ok_or(anyhow!("bad filename"))?;
                let mut file = File::create_new(basename).await?;
                download(&mut file, &counts).await
            }?;
        }

        Action::Serve { bind } => {
            serve(&bind).await?;
        }
    }

    Ok(())
}

fn to_basename<'a>(filename: &'a str) -> Option<&'a str> {
    Path::new(filename).file_name()?.to_str()
}
