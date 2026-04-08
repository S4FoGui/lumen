use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::state::{LumenEvent, LumenState};

/// Handler de upgrade WebSocket.
///
/// Rota: GET /ws
///
/// O cliente conecta via WebSocket e recebe todos os `LumenEvent`
/// em tempo real como JSON. Não envia mensagens de volta (unidirecional).
///
/// Usado pelo dashboard React para:
/// - Mostrar status de gravação em tempo real
/// - Exibir waveform/nível de áudio
/// - Receber transcrições assim que completam
/// - Notificar mudanças de config/dicionário
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<LumenState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

/// Gerencia uma conexão WebSocket individual.
///
/// Subscreve ao broadcast channel e encaminha todos os eventos
/// para o cliente até que a conexão seja encerrada.
async fn handle_ws_connection(mut socket: WebSocket, state: Arc<LumenState>) {
    tracing::info!("🔌 WebSocket: cliente conectado");

    // Subscrever ao bus de eventos
    let mut rx: broadcast::Receiver<LumenEvent> = state.event_tx.subscribe();

    // Enviar evento inicial de "connected" com status atual
    let initial_status = {
        let is_recording = *state.is_recording.read().await;
        let session = state.session.read().await;
        serde_json::json!({
            "type": "connected",
            "data": {
                "version": env!("CARGO_PKG_VERSION"),
                "is_recording": is_recording,
                "uptime_seconds": session.uptime_seconds(),
                "total_transcriptions": session.total_transcriptions,
                "total_words": session.total_words,
            }
        })
    };

    if let Ok(json) = serde_json::to_string(&initial_status) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            tracing::debug!("WebSocket: cliente desconectou antes do status inicial");
            return;
        }
    }

    // Loop de encaminhamento de eventos
    loop {
        tokio::select! {
            // Receber evento do broadcast bus
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        match serde_json::to_string(&event) {
                            Ok(json) => {
                                if socket.send(Message::Text(json.into())).await.is_err() {
                                    // Cliente desconectou
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("WebSocket: falha ao serializar evento: {}", e);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        // Cliente ficou para trás — informar e continuar
                        tracing::warn!("WebSocket: cliente perdeu {} eventos (lag)", n);
                        let lag_msg = serde_json::json!({
                            "type": "warning",
                            "data": {
                                "message": format!("Perdeu {} eventos (lag)", n),
                            }
                        });
                        if let Ok(json) = serde_json::to_string(&lag_msg) {
                            if socket.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        // Bus encerrado — servidor está parando
                        break;
                    }
                }
            }

            // Receber mensagem do cliente (para manter a conexão viva)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {
                        // Ignorar outras mensagens do cliente
                    }
                }
            }
        }
    }

    tracing::info!("🔌 WebSocket: cliente desconectado");
}
