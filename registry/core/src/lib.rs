use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use ota_tools::{load_public_key_hex, SignedBlob};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct PublishRequest {
    pub blob: PathBuf,
    pub manifest: PathBuf,
    pub root_pub: PathBuf,
    pub namespace: String,
    pub name: Option<String>,
    pub version: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
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
        for sub in ["blobs", "manifests", "index"] {
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

        let manifest_dest = self
            .root
            .join("manifests")
            .join(format!("{}.json", hash));
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

        let index_path = index_dir.join(format!("{}.json", tag_component));
        let record = IndexRecord {
            payload_hash: hash.clone(),
            payload_len: manifest.payload_len(),
            manifest: format!("manifests/{}.json", hash),
            signer_key_id: manifest.signer_key_id.clone(),
            logical_name: logical_name.clone(),
            logical_version: logical_version.clone(),
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
        Some(Value::Object(map)) => map
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
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
        fs::write(&root_pub_path, format!("{}\n", public_key_to_hex(&root.public_key()?)))?;

        let result = registry.publish(PublishRequest {
            blob: blob_path.clone(),
            manifest: manifest_path.clone(),
            root_pub: root_pub_path.clone(),
            namespace: "modules".into(),
            name: None,
            version: None,
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
}
