use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use ota_tools::{load_public_key_hex, SignedBlob};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod module_grant;

pub const VM_TEST_REPORT_SCHEMA: &str = "raios.vm_test_report.v0";
pub const LOCAL_ATTESTATION_SCHEMA: &str = "raios.local_attestation.v0";

#[derive(Clone, Debug)]
pub struct PublishRequest {
    pub blob: PathBuf,
    pub manifest: PathBuf,
    pub root_pub: PathBuf,
    pub namespace: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub evidence_files: Vec<EvidenceFile>,
}

#[derive(Clone, Debug)]
pub struct PublishResult {
    pub namespace: String,
    pub tag: String,
    pub record: IndexRecord,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexRecord {
    pub payload_hash: String,
    pub payload_len: u64,
    pub manifest: String,
    pub signer_key_id: String,
    pub logical_name: String,
    pub logical_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct EvidenceFile {
    pub kind: EvidenceKind,
    pub path: PathBuf,
}

impl EvidenceFile {
    pub fn vm_test_report(path: PathBuf) -> Self {
        Self {
            kind: EvidenceKind::VmTestReport,
            path,
        }
    }

    pub fn local_attestation(path: PathBuf) -> Self {
        Self {
            kind: EvidenceKind::LocalAttestation,
            path,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    VmTestReport,
    LocalAttestation,
}

impl EvidenceKind {
    fn expected_schema(&self) -> &'static str {
        match self {
            Self::VmTestReport => VM_TEST_REPORT_SCHEMA,
            Self::LocalAttestation => LOCAL_ATTESTATION_SCHEMA,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub kind: EvidenceKind,
    pub schema: String,
    pub sha256: String,
    pub sha256_source: String,
    pub registry_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RegistryEntry {
    pub namespace: String,
    pub name: String,
    pub tag: String,
    pub record: IndexRecord,
}

#[derive(Clone, Debug, Default)]
pub struct ListFilter<'a> {
    pub namespace: Option<&'a str>,
    pub name: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct Registry {
    root: PathBuf,
}

impl Registry {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn init(&self) -> Result<()> {
        for sub in ["blobs", "manifests", "evidence", "index"] {
            let path = self.root.join(sub);
            if !path.exists() {
                fs::create_dir_all(&path)
                    .with_context(|| format!("creating registry directory {}", path.display()))?;
            }
        }
        Ok(())
    }

    pub fn publish(&self, request: PublishRequest) -> Result<PublishResult> {
        let manifest_str = fs::read_to_string(&request.manifest)
            .with_context(|| format!("reading manifest {}", request.manifest.display()))?;
        let manifest: SignedBlob = serde_json::from_str(&manifest_str)
            .with_context(|| format!("parsing manifest {}", request.manifest.display()))?;
        let root_public = load_public_key_hex(&request.root_pub)?;
        manifest.verify(&request.blob, &root_public)?;

        let hash = manifest.payload_hash().to_string();
        let blob_dest = self.root.join("blobs").join(&hash);
        if !blob_dest.exists() {
            fs::copy(&request.blob, &blob_dest)
                .with_context(|| format!("copying blob to {}", blob_dest.display()))?;
        }

        let manifest_dest = self.root.join("manifests").join(format!("{}.json", hash));
        if !manifest_dest.exists() {
            fs::write(&manifest_dest, manifest_str)
                .with_context(|| format!("writing manifest {}", manifest_dest.display()))?;
        }

        let logical_name = request
            .name
            .or_else(|| metadata_string(manifest.metadata.as_ref(), "module_id"))
            .unwrap_or_else(|| hash.clone());
        let logical_version = request
            .version
            .or_else(|| metadata_string(manifest.metadata.as_ref(), "module_version"));

        let namespace_component = sanitize_component(&request.namespace);
        let name_component = sanitize_component(&logical_name);
        let tag_component = logical_version
            .as_ref()
            .map(|v| sanitize_component(v))
            .unwrap_or_else(|| hash.clone());

        let index_dir = self
            .root
            .join("index")
            .join(&namespace_component)
            .join(&name_component);
        if !index_dir.exists() {
            fs::create_dir_all(&index_dir)
                .with_context(|| format!("creating index directory {}", index_dir.display()))?;
        }

        let evidence = self.prepare_evidence_records(&request.evidence_files)?;

        let index_path = index_dir.join(format!("{}.json", tag_component));
        let record = IndexRecord {
            payload_hash: hash.clone(),
            payload_len: manifest.payload_len(),
            manifest: format!("manifests/{}.json", hash),
            signer_key_id: manifest.signer_key_id.clone(),
            logical_name: logical_name.clone(),
            logical_version: logical_version.clone(),
            evidence,
            metadata: manifest.metadata.clone(),
        };
        fs::write(&index_path, serde_json::to_string_pretty(&record)?)
            .with_context(|| format!("writing index record {}", index_path.display()))?;

        Ok(PublishResult {
            namespace: namespace_component,
            tag: tag_component,
            record,
        })
    }

    fn prepare_evidence_records(
        &self,
        evidence_files: &[EvidenceFile],
    ) -> Result<Vec<EvidenceRecord>> {
        evidence_files
            .iter()
            .map(|file| self.prepare_evidence_record(file))
            .collect()
    }

    fn prepare_evidence_record(&self, evidence_file: &EvidenceFile) -> Result<EvidenceRecord> {
        let content = fs::read_to_string(&evidence_file.path)
            .with_context(|| format!("reading evidence {}", evidence_file.path.display()))?;
        let json: Value = serde_json::from_str(&content)
            .with_context(|| format!("parsing evidence {}", evidence_file.path.display()))?;
        let schema = json
            .get("schema")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("evidence {} has no schema", evidence_file.path.display()))?;
        let expected = evidence_file.kind.expected_schema();
        if schema != expected {
            return Err(anyhow!(
                "evidence {} schema {} does not match expected {}",
                evidence_file.path.display(),
                schema,
                expected
            ));
        }
        if evidence_file.kind == EvidenceKind::LocalAttestation {
            let grants_load_now = json
                .pointer("/limits/grants_load_now")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if grants_load_now {
                return Err(anyhow!(
                    "local attestation {} must not grant load permission",
                    evidence_file.path.display()
                ));
            }
        }

        let sha256 = read_sha256_sidecar(&evidence_file.path)?;
        let registry_path = format!("evidence/{}.json", sha256);
        let evidence_dest = self.root.join(&registry_path);
        if !evidence_dest.exists() {
            fs::write(&evidence_dest, content)
                .with_context(|| format!("writing evidence {}", evidence_dest.display()))?;
        }
        let result = json
            .get("result")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);

        Ok(EvidenceRecord {
            kind: evidence_file.kind.clone(),
            schema: schema.to_string(),
            sha256,
            sha256_source: "sidecar".to_string(),
            registry_path,
            result,
        })
    }

    pub fn list(&self, filter: ListFilter<'_>) -> Result<Vec<RegistryEntry>> {
        let index_root = self.root.join("index");
        if !index_root.exists() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for ns_entry in fs::read_dir(&index_root)? {
            let ns_entry = ns_entry?;
            if !ns_entry.file_type()?.is_dir() {
                continue;
            }
            let ns_name = ns_entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| "invalid".to_string());
            if let Some(filter_ns) = filter.namespace {
                if sanitize_component(filter_ns) != ns_name {
                    continue;
                }
            }
            for name_entry in fs::read_dir(ns_entry.path())? {
                let name_entry = name_entry?;
                if !name_entry.file_type()?.is_dir() {
                    continue;
                }
                let logical_name = name_entry
                    .file_name()
                    .into_string()
                    .unwrap_or_else(|_| "invalid".to_string());
                if let Some(filter_name) = filter.name {
                    if sanitize_component(filter_name) != logical_name {
                        continue;
                    }
                }
                for file_entry in fs::read_dir(name_entry.path())? {
                    let file_entry = file_entry?;
                    if !file_entry.file_type()?.is_file() {
                        continue;
                    }
                    let path = file_entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("json") {
                        continue;
                    }
                    let tag = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("invalid")
                        .to_string();
                    let record: IndexRecord = serde_json::from_str(&fs::read_to_string(&path)?)?;
                    entries.push(RegistryEntry {
                        namespace: ns_name.clone(),
                        name: logical_name.clone(),
                        tag,
                        record,
                    });
                }
            }
        }
        entries.sort_by(|a, b| {
            a.namespace
                .cmp(&b.namespace)
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.tag.cmp(&b.tag))
        });
        Ok(entries)
    }
}

pub fn sanitize_component(input: &str) -> String {
    let mut sanitized: String = input
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect();
    if sanitized.is_empty() {
        sanitized.push_str("unnamed");
    }
    sanitized
}

fn metadata_string(meta: Option<&Value>, key: &str) -> Option<String> {
    match meta {
        Some(Value::Object(map)) => map.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()),
        _ => None,
    }
}

fn read_sha256_sidecar(path: &PathBuf) -> Result<String> {
    let mut sidecar_name = path.clone().into_os_string();
    sidecar_name.push(".sha256");
    let sidecar = PathBuf::from(sidecar_name);
    let content = fs::read_to_string(&sidecar)
        .with_context(|| format!("reading evidence hash sidecar {}", sidecar.display()))?;
    let hash = content
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("evidence hash sidecar {} is empty", sidecar.display()))?;
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!(
            "evidence hash sidecar {} does not start with a sha256 hex hash",
            sidecar.display()
        ));
    }
    Ok(hash.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ota_tools::{public_key_to_hex, KeyMaterial, SignerCertificate};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn publish_and_list_roundtrip() -> Result<()> {
        let temp = tempdir()?;
        let registry_path = temp.path().join("registry");
        let registry = Registry::new(registry_path.clone());
        registry.init()?;

        let root = KeyMaterial::from_seed("root", "registry-test")?;
        let online = KeyMaterial::from_seed("online", "registry-test")?;
        let cert = SignerCertificate::new(&online, &root, 1_700_000_000_000, 60_000)?;

        let blob_path = temp.path().join("module.wasm");
        fs::write(&blob_path, b"fake wasm bytes")?;
        let manifest = SignedBlob::sign(
            &blob_path,
            &online,
            &cert,
            Some(serde_json::json!({
                "module_id": "hello-ui",
                "module_version": "0.1.0",
            })),
        )?;
        let manifest_path = temp.path().join("module.manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        let root_pub_path = temp.path().join("root.pub");
        fs::write(
            &root_pub_path,
            format!("{}\n", public_key_to_hex(&root.public_key()?)),
        )?;

        let result = registry.publish(PublishRequest {
            blob: blob_path.clone(),
            manifest: manifest_path.clone(),
            root_pub: root_pub_path.clone(),
            namespace: "modules".into(),
            name: None,
            version: None,
            evidence_files: Vec::new(),
        })?;
        assert_eq!(result.record.logical_name, "hello-ui");
        assert_eq!(result.record.logical_version.as_deref(), Some("0.1.0"));

        let entries = registry.list(ListFilter {
            namespace: Some("modules"),
            name: Some("hello-ui"),
        })?;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].record.payload_hash, manifest.payload_hash());
        Ok(())
    }

    #[test]
    fn publish_records_vm_report_and_local_attestation_references() -> Result<()> {
        let temp = tempdir()?;
        let registry_path = temp.path().join("registry");
        let registry = Registry::new(registry_path.clone());
        registry.init()?;

        let root = KeyMaterial::from_seed("root", "registry-evidence-test")?;
        let online = KeyMaterial::from_seed("online", "registry-evidence-test")?;
        let cert = SignerCertificate::new(&online, &root, 1_700_000_000_000, 60_000)?;

        let blob_path = temp.path().join("module.wasm");
        fs::write(&blob_path, b"candidate module bytes")?;
        let manifest = SignedBlob::sign(
            &blob_path,
            &online,
            &cert,
            Some(serde_json::json!({
                "module_id": "evidence-ui",
                "module_version": "0.2.0",
            })),
        )?;
        let manifest_path = temp.path().join("module.manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        let root_pub_path = temp.path().join("root.pub");
        fs::write(
            &root_pub_path,
            format!("{}\n", public_key_to_hex(&root.public_key()?)),
        )?;

        let report_path = temp.path().join("vm-report.json");
        fs::write(
            &report_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": VM_TEST_REPORT_SCHEMA,
                "result": "passed",
                "evidence_binding": {
                    "candidate_artifact_sha256": "abc"
                }
            }))?,
        )?;
        fs::write(
            temp.path().join("vm-report.json.sha256"),
            "1111111111111111111111111111111111111111111111111111111111111111  vm-report.json\n",
        )?;
        let attestation_path = temp.path().join("local-attestation.json");
        fs::write(
            &attestation_path,
            serde_json::to_string_pretty(&serde_json::json!({
                "schema": LOCAL_ATTESTATION_SCHEMA,
                "result": "evidence_recorded_load_still_denied_in_stage0",
                "limits": {
                    "grants_load_now": false
                }
            }))?,
        )?;
        fs::write(
            temp.path().join("local-attestation.json.sha256"),
            "2222222222222222222222222222222222222222222222222222222222222222  local-attestation.json\n",
        )?;

        let result = registry.publish(PublishRequest {
            blob: blob_path,
            manifest: manifest_path,
            root_pub: root_pub_path,
            namespace: "modules".into(),
            name: None,
            version: None,
            evidence_files: vec![
                EvidenceFile::vm_test_report(report_path),
                EvidenceFile::local_attestation(attestation_path),
            ],
        })?;

        assert_eq!(result.record.evidence.len(), 2);
        assert_eq!(result.record.evidence[0].kind, EvidenceKind::VmTestReport);
        assert_eq!(result.record.evidence[0].schema, VM_TEST_REPORT_SCHEMA);
        assert_eq!(result.record.evidence[0].result.as_deref(), Some("passed"));
        assert_eq!(
            result.record.evidence[1].kind,
            EvidenceKind::LocalAttestation
        );
        assert_eq!(
            result.record.evidence[1].result.as_deref(),
            Some("evidence_recorded_load_still_denied_in_stage0")
        );
        for evidence in &result.record.evidence {
            assert!(registry_path.join(&evidence.registry_path).exists());
            assert!(!evidence
                .registry_path
                .contains(temp.path().to_string_lossy().as_ref()));
        }
        Ok(())
    }
}
