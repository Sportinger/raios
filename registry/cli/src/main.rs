use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use registry_core::module_grant::ComputeCapabilityGrantRequest;
use registry_core::{EvidenceFile, ListFilter, PublishRequest, Registry};

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
        /// Shadow-VM test report JSON to bind as evidence
        #[arg(long = "vm-report")]
        vm_reports: Vec<PathBuf>,
        /// Local attestation JSON to bind as evidence
        #[arg(long = "local-attestation")]
        local_attestations: Vec<PathBuf>,
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
    /// Compute a non-authorizing module capability grant diagnostic
    GrantDiagnostic {
        /// raiOS module_manifest.v0 JSON
        #[arg(long)]
        manifest: PathBuf,
        /// Candidate artifact bytes bound by the manifest
        #[arg(long)]
        artifact: PathBuf,
        /// Shadow VM test report JSON
        #[arg(long = "vm-report")]
        vm_report: PathBuf,
        /// Local attestation JSON
        #[arg(long = "local-attestation")]
        local_attestation: PathBuf,
        /// Exact local approval phrase for the evidence tuple
        #[arg(long)]
        approval: String,
        /// Capability being evaluated
        #[arg(long, default_value = "cap.module.load_ephemeral")]
        requested_capability: String,
        /// Requested load mode
        #[arg(long, default_value = "ram_only")]
        load_mode: String,
        /// Subject for the diagnostic request
        #[arg(long, default_value = "agent.session.serial")]
        subject: String,
        /// Resource being evaluated
        #[arg(long, default_value = "live_service_graph")]
        resource: String,
        /// Scope for the diagnostic request
        #[arg(long, default_value = "current_boot")]
        scope: String,
        /// Expected local attestation SHA-256, with or without sha256: prefix
        #[arg(long)]
        expected_local_attestation_sha256: Option<String>,
        /// Print rejected diagnostics instead of exiting with an error
        #[arg(long)]
        allow_invalid: bool,
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
            vm_reports,
            local_attestations,
        } => {
            let registry = Registry::new(registry);
            registry.init()?;
            let evidence_files = vm_reports
                .into_iter()
                .map(EvidenceFile::vm_test_report)
                .chain(
                    local_attestations
                        .into_iter()
                        .map(EvidenceFile::local_attestation),
                )
                .collect();
            let result = registry.publish(PublishRequest {
                blob,
                manifest,
                root_pub,
                namespace,
                name,
                version,
                evidence_files,
            })?;
            println!(
                "stored {} bytes as {} (namespace={} name={} tag={} evidence={})",
                result.record.payload_len,
                result.record.payload_hash,
                result.namespace,
                result.record.logical_name,
                result.tag,
                result.record.evidence.len(),
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
                        "[{ns}] {name} {version} -> {hash} ({len} bytes, evidence={evidence})",
                        ns = entry.namespace,
                        name = entry.record.logical_name,
                        version = version,
                        hash = entry.record.payload_hash,
                        len = entry.record.payload_len,
                        evidence = entry.record.evidence.len(),
                    );
                }
            }
        }
        Command::GrantDiagnostic {
            manifest,
            artifact,
            vm_report,
            local_attestation,
            approval,
            requested_capability,
            load_mode,
            subject,
            resource,
            scope,
            expected_local_attestation_sha256,
            allow_invalid,
        } => {
            let mut request = ComputeCapabilityGrantRequest::new(
                manifest,
                artifact,
                vm_report,
                local_attestation,
                approval,
            );
            request.requested_capability = requested_capability;
            request.load_mode = load_mode;
            request.subject = subject;
            request.resource = resource;
            request.scope = scope;
            request.expected_local_attestation_sha256 = expected_local_attestation_sha256;
            let diagnostic = registry_core::module_grant::compute_capability_grant(&request)?;
            println!("{}", serde_json::to_string_pretty(&diagnostic)?);
            if !diagnostic.valid_evidence && !allow_invalid {
                anyhow::bail!(
                    "computed grant diagnostic rejected evidence: {}",
                    diagnostic.denial_reasons.join(", ")
                );
            }
        }
    }
    Ok(())
}
