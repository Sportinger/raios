use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::PublicKey;
use futures::{SinkExt, StreamExt};
use ota_tools::{load_public_key_hex, SignedBlob};
use registry_core::{PublishRequest, Registry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind: String,
    pub root_pub: PathBuf,
    pub registry: Option<PathBuf>,
}

#[derive(Clone)]
struct ServerState {
    root_pub_path: PathBuf,
    root_public: PublicKey,
    registry_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    v: u8,
    #[serde(rename = "t")]
    message_type: String,
    id: String,
    ts: u64,
    body: Value,
}

#[derive(Debug, Serialize)]
struct ResponseEnvelope {
    v: u8,
    #[serde(rename = "t")]
    message_type: String,
    id: String,
    ts: u64,
    body: Value,
}

#[derive(Debug, Deserialize)]
struct OtaBeginBody {
    manifest: SignedBlob,
    blob_b64: String,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Serialize)]
struct OtaAckBody {
    status: String,
    payload_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    registry_tag: Option<String>,
}

#[derive(Error, Debug)]
enum HandlerError {
    #[error("unsupported message type: {0}")]
    UnsupportedType(String),
    #[error("missing or invalid body: {0}")]
    InvalidBody(String),
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("registry error: {0}")]
    Registry(String),
}

pub async fn run_server(config: ServerConfig) -> Result<()> {
    info!("starting fake cloud on {}", config.bind);
    let root_public = load_public_key_hex(&config.root_pub)?;
    if let Some(ref registry_path) = config.registry {
        Registry::new(registry_path.clone()).init()?;
        info!("registry initialized at {}", registry_path.display());
    }

    let state = Arc::new(ServerState {
        root_pub_path: config.root_pub.clone(),
        root_public,
        registry_root: config.registry.clone(),
    });

    let listener = TcpListener::bind(&config.bind).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, addr, state).await {
                warn!("client error from {}: {}", addr, err);
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<ServerState>,
) -> Result<()> {
    info!("connection accepted from {}", addr);
    let ws_stream = accept_async(stream).await?;
    let (mut sink, mut stream) = ws_stream.split();

    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(err) => {
                warn!("websocket error from {}: {}", addr, err);
                break;
            }
        };
        match msg {
            Message::Text(text) => match serde_json::from_str::<Envelope>(&text) {
                Ok(env) => match handle_envelope(&env, state.clone()).await {
                    Ok(Some(resp)) => {
                        let json = serde_json::to_string(&resp)?;
                        sink.send(Message::Text(json)).await?;
                    }
                    Ok(None) => {}
                    Err(err) => {
                        error!("handler error for {}: {}", addr, err);
                        let body = json!({
                            "status": "error",
                            "message": err.to_string(),
                        });
                        let resp = ResponseEnvelope {
                            v: env.v,
                            message_type: "error".to_string(),
                            id: env.id.clone(),
                            ts: now_ms(),
                            body,
                        };
                        let json = serde_json::to_string(&resp)?;
                        sink.send(Message::Text(json)).await?;
                    }
                },
                Err(err) => {
                    warn!("invalid envelope from {}: {} -- {}", addr, err, text);
                    let body = json!({
                        "status": "error",
                        "message": format!("invalid envelope: {}", err),
                    });
                    let resp = ResponseEnvelope {
                        v: 1,
                        message_type: "error".to_string(),
                        id: "error".to_string(),
                        ts: now_ms(),
                        body,
                    };
                    sink.send(Message::Text(serde_json::to_string(&resp)?))
                        .await?;
                }
            },
            Message::Binary(_) => {
                warn!("binary payload not supported from {}", addr);
            }
            Message::Close(frame) => {
                info!("connection {} closed: {:?}", addr, frame);
                break;
            }
            Message::Ping(payload) => {
                sink.send(Message::Pong(payload)).await?;
            }
            Message::Pong(_) => {}
            Message::Frame(_) => {}
        }
    }

    Ok(())
}

async fn handle_envelope(
    env: &Envelope,
    state: Arc<ServerState>,
) -> Result<Option<ResponseEnvelope>> {
    let _ = env.ts;
    match env.message_type.as_str() {
        "hello" => {
            let body = json!({
                "status": "ok",
            });
            let response = ResponseEnvelope {
                v: env.v,
                message_type: "hello_ack".to_string(),
                id: env.id.clone(),
                ts: now_ms(),
                body,
            };
            Ok(Some(response))
        }
        "ota_begin" => {
            let ack = tokio::task::spawn_blocking({
                let state = state.clone();
                let body = env.body.clone();
                move || handle_ota_begin(body, &state)
            })
            .await??;
            let response = ResponseEnvelope {
                v: env.v,
                message_type: "ota_ack".to_string(),
                id: env.id.clone(),
                ts: now_ms(),
                body: serde_json::to_value(ack)?,
            };
            Ok(Some(response))
        }
        other => Err(HandlerError::UnsupportedType(other.to_string()).into()),
    }
}

fn handle_ota_begin(body: Value, state: &ServerState) -> Result<OtaAckBody> {
    let OtaBeginBody {
        manifest,
        blob_b64,
        namespace,
        name,
        version,
    } = serde_json::from_value(body).map_err(|err| HandlerError::InvalidBody(err.to_string()))?;

    let decoded = BASE64
        .decode(blob_b64.as_bytes())
        .map_err(|err| HandlerError::InvalidBody(err.to_string()))?;

    let mut blob_file = NamedTempFile::new()
        .map_err(|err| HandlerError::Manifest(format!("create blob tmp: {}", err)))?;
    blob_file
        .write_all(&decoded)
        .map_err(|err| HandlerError::Manifest(format!("write blob tmp: {}", err)))?;
    let blob_path = blob_file.path().to_path_buf();

    manifest
        .verify(&blob_path, &state.root_public)
        .map_err(|err| HandlerError::Manifest(err.to_string()))?;

    let manifest_json =
        serde_json::to_string(&manifest).map_err(|err| HandlerError::Manifest(err.to_string()))?;
    let mut manifest_file = NamedTempFile::new()
        .map_err(|err| HandlerError::Manifest(format!("create manifest tmp: {}", err)))?;
    manifest_file
        .write_all(manifest_json.as_bytes())
        .map_err(|err| HandlerError::Manifest(format!("write manifest tmp: {}", err)))?;
    let manifest_path = manifest_file.path().to_path_buf();

    let registry_result = if let Some(root) = &state.registry_root {
        let registry = Registry::new(root.clone());
        registry
            .init()
            .map_err(|err| HandlerError::Registry(err.to_string()))?;
        let publish = registry
            .publish(PublishRequest {
                blob: blob_path.clone(),
                manifest: manifest_path.clone(),
                root_pub: state.root_pub_path.clone(),
                namespace: namespace.unwrap_or_else(|| "ota".into()),
                name,
                version,
                evidence_files: Vec::new(),
            })
            .map_err(|err| HandlerError::Registry(err.to_string()))?;
        Some(publish.tag)
    } else {
        None
    };

    // Clean up by ensuring temp files stay until publish completes
    drop(manifest_file);
    drop(blob_file);

    Ok(OtaAckBody {
        status: "verified".to_string(),
        payload_hash: manifest.payload_hash().to_string(),
        registry_tag: registry_result,
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
