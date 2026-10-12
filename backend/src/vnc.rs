use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{error, info, warn};

use crate::state::AppState;

pub async fn vnc_ws_handler(
    Path(vm_id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.protocols(["binary"])
      .on_upgrade(move |socket| handle_vnc_socket(socket, vm_id, state))
}

async fn handle_vnc_socket(socket: WebSocket, vm_id: String, state: Arc<AppState>) {
    info!("VNC WebSocket connection initiated for VM: {}", vm_id);

    // Get live VNC port directly from hypervisor driver
    let vnc_port = {
        let local_driver = state.get_driver("local-kvm").await;
        if let Some(driver) = local_driver {
            if let Ok(info) = driver.get_vnc_info(&vm_id).await {
                info.port
            } else {
                let store = state.store.read().await;
                store.vms.get(&vm_id).and_then(|v| v.vnc_port).unwrap_or(5900)
            }
        } else {
            let store = state.store.read().await;
            store.vms.get(&vm_id).and_then(|v| v.vnc_port).unwrap_or(5900)
        }
    };

    let target_addr = format!("127.0.0.1:{}", vnc_port);
    info!("Connecting VNC proxy to TCP {}", target_addr);

    let tcp_stream = match TcpStream::connect(&target_addr).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to connect to VNC target {}: {}. Attempting fallback port 5900...", target_addr, e);
            match TcpStream::connect("127.0.0.1:5900").await {
                Ok(s) => s,
                Err(e2) => {
                    error!("VNC TCP connection completely failed: {}", e2);
                    return;
                }
            }
        }
    };

    let (mut tcp_read, mut tcp_write) = tcp_stream.into_split();
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Forward WebSocket -> TCP
    let ws_to_tcp = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Binary(bin) => {
                    if let Err(e) = tcp_write.write_all(&bin).await {
                        warn!("Error writing to TCP VNC stream: {}", e);
                        break;
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
        let _ = tcp_write.shutdown().await;
    });

    // Forward TCP -> WebSocket
    let tcp_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => break, // EOF
                Ok(n) => {
                    if let Err(e) = ws_sender.send(Message::Binary(buf[..n].to_vec())).await {
                        warn!("Error sending to WS client: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    warn!("Error reading from TCP VNC stream: {}", e);
                    break;
                }
            }
        }
        let _ = ws_sender.close().await;
    });

    let _ = tokio::select! {
        _ = ws_to_tcp => {},
        _ = tcp_to_ws => {},
    };

    info!("VNC WebSocket session ended for VM: {}", vm_id);
}
