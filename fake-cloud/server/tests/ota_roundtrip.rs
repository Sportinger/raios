use std::net::TcpListener;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use fake_cloud_server::{run_server, ServerConfig};
use futures::{SinkExt, StreamExt};
use ota_tools::{public_key_to_hex, KeyMaterial, SignerCertificate, SignedBlob};
use registry_core::{ListFilter, Registry};
use serde_json::json;
use tempfile::tempdir;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn ota_roundtrip_publishes_to_registry() -> anyhow::Result<()> {
    let temp = tempdir()?;
    let registry_path = temp.path().join("registry");
    let root = KeyMaterial::from_seed("root", "fake-cloud-test")?;
    let online = KeyMaterial::from_seed("online", "fake-cloud-test")?;
    let cert = SignerCertificate::new(&online, &root, 1_700_000_000_000, 60_000)?;

    let blob_path = temp.path().join("module.wasm");
    std::fs::write(&blob_path, b"fake wasm bytes from test")?;
    let manifest = SignedBlob::sign(
        &blob_path,
        &online,
        &cert,
        Some(json!({
            "module_id": "hello-ui",
            "module_version": "0.1.0",
        })),
    )?;

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    let manifest_path = temp.path().join("module.manifest.json");
    std::fs::write(&manifest_path, &manifest_json)?;

    let root_pub_path = temp.path().join("root.pub");
    std::fs::write(&root_pub_path, format!("{}\n", public_key_to_hex(&root.public_key()?)))?;

    let bind = reserve_port();
    let config = ServerConfig {
        bind: bind.clone(),
        root_pub: root_pub_path.clone(),
        registry: Some(registry_path.clone()),
    };

    let server = spawn_server(config).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let url = format!("ws://{}", bind);
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await?;

    let envelope = json!({
        "v": 1,
        "t": "ota_begin",
        "id": "test-ota",
        "ts": 1_700_000_000_000u64,
        "body": {
            "manifest": manifest,
            "blob_b64": BASE64.encode(std::fs::read(&blob_path)?),
            "namespace": "modules",
            "name": "hello-ui",
            "version": "0.1.0"
        }
    });
    ws.send(Message::Text(envelope.to_string())).await?;

    let reply = ws.next().await.expect("response");
    let text = match reply? {
        Message::Text(text) => text,
        other => panic!("unexpected response: {:?}", other),
    };
    let ack: serde_json::Value = serde_json::from_str(&text)?;
    assert_eq!(ack["t"], "ota_ack");
    assert_eq!(ack["body"]["status"], "verified");
    assert_eq!(ack["body"]["payload_hash"], manifest.payload_hash());

    // ensure registry entry exists
    let registry = Registry::new(registry_path.clone());
    registry.init()?;
    let entries = registry.list(ListFilter {
        namespace: Some("modules"),
        name: Some("hello-ui"),
    })?;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].record.payload_hash, manifest.payload_hash());

    ws.close(None).await?;
    server.abort();
    Ok(())
}

fn reserve_port() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr.to_string()
}

async fn spawn_server(config: ServerConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        let _ = run_server(config).await;
    })
}
