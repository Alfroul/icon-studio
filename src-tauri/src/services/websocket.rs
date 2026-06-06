use crate::commands::canvas::ProjectState;
use crate::model::IconProject;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;

// ---------------------------------------------------------------------------
// SyncMessage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SyncMessage {
    ProjectUpdate {
        version: u64,
        source: String,
        snapshot: IconProject,
    },
    ProjectSnapshot {
        version: u64,
        source: String,
        snapshot: IconProject,
    },
    ClientJoin {
        client_id: String,
        version: u64,
    },
    ClientLeave {
        client_id: String,
    },
}

// ---------------------------------------------------------------------------
// WebSocketServer
// ---------------------------------------------------------------------------

static CLIENT_COUNTER: AtomicU64 = AtomicU64::new(1);
static AUTH_TOKEN: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn generate_token() -> String {
    use std::fmt::Write;
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut state: u64 = seed as u64 ^ CLIENT_COUNTER.load(Ordering::Relaxed);
    let mut buf = [0u8; 16];
    for b in buf.iter_mut() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *b = ((state >> 33) & 0xFF) as u8;
    }
    let mut token = String::with_capacity(32);
    for b in &buf {
        let _ = write!(&mut token, "{b:02x}");
    }
    token
}

pub fn get_auth_token() -> &'static str {
    AUTH_TOKEN.get_or_init(generate_token)
}

type ClientMap = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Message>>>>;

pub struct WebSocketServer {
    clients: ClientMap,
    project: ProjectState,
    auth_token: String,
    app_handle: Option<AppHandle>,
}

impl WebSocketServer {
    pub fn new(project: ProjectState, app_handle: Option<AppHandle>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            project,
            auth_token: get_auth_token().to_string(),
            app_handle,
        }
    }

    /// Send a JSON-encoded SyncMessage to every connected client except
    /// the one identified by `exclude_client`.
    pub async fn broadcast(&self, message: &SyncMessage, exclude_client: Option<&str>) {
        let json = match serde_json::to_string(message) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("[WS] failed to serialize broadcast message: {e}");
                return;
            }
        };

        let clients = self.clients.read().await;
        for (id, tx) in clients.iter() {
            if exclude_client == Some(id.as_str()) {
                continue;
            }
            if tx.send(Message::Text(json.clone())).is_err() {
                // Receiver dropped; cleaned up when the connection task ends.
            }
        }
    }

    pub fn get_snapshot(&self) -> (u64, IconProject) {
        let project = self.project.lock().unwrap_or_else(|e| e.into_inner());
        (project.version, project.clone())
    }
}

// ---------------------------------------------------------------------------
// Connection handler
// ---------------------------------------------------------------------------

async fn handle_connection(
    server: Arc<WebSocketServer>,
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    client_id: String,
) {
    let (mut ws_sink, mut ws_source) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    {
        let mut clients = server.clients.write().await;
        clients.insert(client_id.clone(), tx);
    }

    // Send current project snapshot on connect.
    {
        let (version, snapshot) = server.get_snapshot();
        let welcome = SyncMessage::ProjectSnapshot {
            version,
            source: "server".to_string(),
            snapshot,
        };
        if let Ok(json) = serde_json::to_string(&welcome) {
            let _ = ws_sink.send(Message::Text(json)).await;
        }
    }

    // Broadcast ClientJoin to others.
    {
        let (version, _) = server.get_snapshot();
        let join_msg = SyncMessage::ClientJoin {
            client_id: client_id.clone(),
            version,
        };
        server.broadcast(&join_msg, Some(&client_id)).await;
    }

    let server_clone = server.clone();
    let client_id_clone = client_id.clone();

    let relay = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let read_loop = tokio::spawn(async move {
        while let Some(result) = ws_source.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    handle_incoming(&server_clone, &client_id_clone, &text).await;
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = relay => {},
        _ = read_loop => {},
    }

    // Cleanup on disconnect.
    {
        let mut clients = server.clients.write().await;
        clients.remove(&client_id);
    }
    let leave_msg = SyncMessage::ClientLeave {
        client_id: client_id.clone(),
    };
    server.broadcast(&leave_msg, None).await;
}

/// Process an incoming text message from a client.
async fn handle_incoming(server: &Arc<WebSocketServer>, client_id: &str, text: &str) {
    let msg: SyncMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[WS] invalid message from {client_id}: {e}");
            return;
        }
    };

    if let SyncMessage::ProjectUpdate {
        version,
        source,
        snapshot,
    } = msg
    {
        let applied_snapshot = {
            let mut project = server.project.lock().unwrap_or_else(|e| e.into_inner());
            if version > project.version {
                *project = snapshot;
                project.version = version;
                Some(project.clone())
            } else {
                None
            }
        };

        if let Some(snap) = applied_snapshot {
            if let Some(ref handle) = server.app_handle {
                let _ = handle.emit("project-changed", ());
            }
            let broadcast_msg = SyncMessage::ProjectUpdate {
                version,
                source,
                snapshot: snap,
            };
            server.broadcast(&broadcast_msg, Some(client_id)).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Server starter
// ---------------------------------------------------------------------------

#[allow(clippy::result_large_err)]
pub async fn start_server(project: ProjectState, port: u16, app_handle: Option<AppHandle>) {
    let addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[WS] failed to bind {addr}: {e}");
            return;
        }
    };

    println!("[WS] server listening on {addr}");

    let server = Arc::new(WebSocketServer::new(project, app_handle));

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let token = server.auth_token.clone();
                let ws_stream = match tokio_tungstenite::accept_hdr_async(
                    stream,
                    move |request: &Request, response: Response| {
                        let auth_ok = request.uri().query()
                            .map(|q| q.split('&').any(|p| p == format!("token={token}")))
                            .unwrap_or(false);
                        if auth_ok {
                            Ok(response)
                        } else {
                            #[allow(clippy::result_large_err)]
                            let err = Response::builder()
                                .status(401)
                                .body(Some("Unauthorized".to_string()))
                                .expect("building 401 response should not fail");
                            Err(err)
                        }
                    },
                ).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        eprintln!("[WS] handshake failed (auth?): {e}");
                        continue;
                    }
                };

                let n = CLIENT_COUNTER.fetch_add(1, Ordering::Relaxed);
                let client_id = format!("ws-{n}");
                let srv = server.clone();

                tokio::spawn(async move {
                    handle_connection(srv, ws_stream, client_id).await;
                });
            }
            Err(e) => {
                eprintln!("[WS] accept error: {e}");
            }
        }
    }
}

/// Convenience entry point: spawn a background thread with its own tokio runtime.
pub fn spawn(project: ProjectState, port: u16, app_handle: Option<AppHandle>) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .thread_name("ws-server")
            .enable_all()
            .build()
            .expect("[WS] failed to create tokio runtime");

        rt.block_on(start_server(project, port, app_handle));
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::IconProject;
    use std::sync::Mutex;

    fn make_server() -> Arc<WebSocketServer> {
        let project = Arc::new(Mutex::new(IconProject::default()));
        Arc::new(WebSocketServer::new(project, None))
    }

    #[test]
    fn test_sync_message_serialization() {
        let project = IconProject::default();

        let msg = SyncMessage::ProjectUpdate {
            version: 1,
            source: "ws-1".to_string(),
            snapshot: project.clone(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"project_update""#));
        let parsed: SyncMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SyncMessage::ProjectUpdate { version: 1, .. }));

        let msg = SyncMessage::ProjectSnapshot {
            version: 2,
            source: "server".to_string(),
            snapshot: project.clone(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"project_snapshot""#));
        let parsed: SyncMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SyncMessage::ProjectSnapshot { version: 2, .. }));

        let msg = SyncMessage::ClientJoin {
            client_id: "ws-1".to_string(),
            version: 3,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"client_join""#));
        let parsed: SyncMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SyncMessage::ClientJoin { version: 3, .. }));

        let msg = SyncMessage::ClientLeave {
            client_id: "ws-1".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"client_leave""#));
        let parsed: SyncMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, SyncMessage::ClientLeave { .. }));
    }

    #[tokio::test]
    async fn test_broadcast_excludes_sender() {
        let server = make_server();

        let (tx1, mut rx1) = mpsc::unbounded_channel::<Message>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<Message>();

        {
            let mut clients = server.clients.write().await;
            clients.insert("ws-1".to_string(), tx1);
            clients.insert("ws-2".to_string(), tx2);
        }

        let msg = SyncMessage::ClientLeave {
            client_id: "ws-1".to_string(),
        };
        server.broadcast(&msg, Some("ws-1")).await;

        // ws-1 excluded → should NOT receive
        assert!(rx1.try_recv().is_err());

        // ws-2 should receive the message
        let received = rx2.try_recv().unwrap();
        if let Message::Text(text) = received {
            let parsed: SyncMessage = serde_json::from_str(&text).unwrap();
            assert!(matches!(parsed, SyncMessage::ClientLeave { .. }));
        } else {
            panic!("Expected Text message");
        }
    }

    #[tokio::test]
    async fn test_version_conflict_resolution() {
        let server = make_server();

        // Given: project version is 5
        {
            let mut project = server.project.lock().unwrap();
            for _ in 0..5 {
                project.bump_version();
            }
        }

        // When: stale update with version 3
        let stale_snapshot = IconProject::default();
        handle_incoming(
            &server,
            "ws-1",
            &serde_json::to_string(&SyncMessage::ProjectUpdate {
                version: 3,
                source: "ws-1".to_string(),
                snapshot: stale_snapshot,
            }).unwrap(),
        )
        .await;

        // Then: version unchanged
        let (version, _) = server.get_snapshot();
        assert_eq!(version, 5);

        // When: newer update with version 10
        let mut new_snapshot = IconProject::default();
        new_snapshot.version = 10;
        new_snapshot.canvas.width = 1024;
        handle_incoming(
            &server,
            "ws-2",
            &serde_json::to_string(&SyncMessage::ProjectUpdate {
                version: 10,
                source: "ws-2".to_string(),
                snapshot: new_snapshot,
            }).unwrap(),
        )
        .await;

        // Then: version updated and canvas applied
        let (version, project) = server.get_snapshot();
        assert_eq!(version, 10);
        assert_eq!(project.canvas.width, 1024);
    }
}
