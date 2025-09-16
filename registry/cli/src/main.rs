use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(about = "Seed OS registry management tooling (stub)", version)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a content-addressed registry layout
    Init {
        /// Directory to initialize
        #[arg(long, default_value = "registry/local")]
        path: String,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init { path } => {
            std::fs::create_dir_all(&path)
                .with_context(|| format!("creating registry directory {}", path))?;
            println!("initialized registry at {}", path);
        }
    }
    Ok(())
}
