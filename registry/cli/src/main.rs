use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use registry_core::{ListFilter, PublishRequest, Registry};

#[derive(Parser, Debug)]
#[command(about = "Seed OS registry management tooling", version)]
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
        path: PathBuf,
    },
    /// Publish a signed blob + manifest into the registry
    Publish {
        /// Registry root directory
        #[arg(long, default_value = "registry/local")]
        registry: PathBuf,
        /// Blob content to store
        #[arg(long)]
        blob: PathBuf,
        /// Signed manifest JSON (output of ota-sign/mod-sign)
        #[arg(long)]
        manifest: PathBuf,
        /// Offline root public key (hex file)
        #[arg(long)]
        root_pub: PathBuf,
        /// Logical namespace (e.g. modules, ota)
        #[arg(long, default_value = "modules")]
        namespace: String,
        /// Logical name (defaults to metadata.module_id or blob hash)
        #[arg(long)]
        name: Option<String>,
        /// Version or tag (defaults to metadata.module_version or blob hash)
        #[arg(long)]
        version: Option<String>,
    },
    /// List registry index entries
    List {
        /// Registry root directory
        #[arg(long, default_value = "registry/local")]
        registry: PathBuf,
        /// Filter to namespace
        #[arg(long)]
        namespace: Option<String>,
        /// Filter to logical name
        #[arg(long)]
        name: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init { path } => {
            Registry::new(path).init()?;
        }
        Command::Publish {
            registry,
            blob,
            manifest,
            root_pub,
            namespace,
            name,
            version,
        } => {
            let registry = Registry::new(registry);
            registry.init()?;
            let result = registry.publish(PublishRequest {
                blob,
                manifest,
                root_pub,
                namespace,
                name,
                version,
            })?;
            println!(
                "stored {} bytes as {} (namespace={} name={} tag={})",
                result.record.payload_len,
                result.record.payload_hash,
                result.namespace,
                result.record.logical_name,
                result.tag,
            );
        }
        Command::List {
            registry,
            namespace,
            name,
        } => {
            let registry = Registry::new(registry);
            let entries = registry.list(ListFilter {
                namespace: namespace.as_deref(),
                name: name.as_deref(),
            })?;
            if entries.is_empty() {
                println!("(empty)");
            } else {
                for entry in entries {
                    let version = entry.record.logical_version.as_deref().unwrap_or("<hash>");
                    println!(
                        "[{ns}] {name} {version} -> {hash} ({len} bytes)",
                        ns = entry.namespace,
                        name = entry.record.logical_name,
                        version = version,
                        hash = entry.record.payload_hash,
                        len = entry.record.payload_len,
                    );
                }
            }
        }
    }
    Ok(())
}
