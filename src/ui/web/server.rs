use anyhow::Result;
use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::config::{DictionaryEntryData, LumenConfig};
use crate::state::{LumenEvent, LumenState};
use crate::ui::web::ws::ws_handler;

// ═══════════════════════════════════════════════════════════════
// API Response/Request types
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct StatusResponse {
    status: String,
    is_recording: bool,
    version: String,
    uptime_seconds: i64,
    total_transcriptions: u64,
    total_words: u64,
}

#[derive(Deserialize)]
struct SnippetRequest {
    trigger: String,
    text: String,
}

#[derive(Serialize)]
struct SnippetEntry {
    trigger: String,
    text: String,
}

#[derive(Deserialize)]
struct DictionaryRequest {
    key: String,
    value: String,
    context: Option<String>,
    icon_type: Option<String>,
}

#[derive(Serialize)]
struct DictionaryResponseEntry {
    key: String,
    value: String,
    context: Option<String>,
    icon_type: Option<String>,
}

#[derive(Deserialize)]
struct PaginationQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

// ═══════════════════════════════════════════════════════════════
// Server startup
// ═══════════════════════════════════════════════════════════════

/// Inicia o servidor web axum usando o LumenState centralizado.
pub async fn start_server(
    state: Arc<LumenState>,
    port: u16,
) -> Result<()> {
    // Diretório de arquivos estáticos
    let static_dir = get_static_dir();

    // Rotas da API
    let app = Router::new()
        // Página principal
        .route("/", get({
            let static_dir = static_dir.clone();
            move || serve_index_from_disk(static_dir.clone())
        }))
        // Servir arquivos estáticos (CSS, JS) gerados pelo Vite
        .nest_service("/assets/", ServeDir::new(format!("{}/assets", static_dir.clone())))
        // Health check
        .route("/api/health", get(api_health))
        // Status & Stats
        .route("/api/status", get(api_status))
        .route("/api/stats", get(api_stats))
        // Config
        .route("/api/config", get(api_get_config))
        .route("/api/config", put(api_update_config))
        // Snippets (opera nos componentes VIVOS)
        .route("/api/snippets", get(api_list_snippets))
        .route("/api/snippets", post(api_add_snippet))
        .route("/api/snippets/{trigger}", delete(api_delete_snippet))
        // Dictionary (opera nos componentes VIVOS)
        .route("/api/dictionary", get(api_list_dictionary))
        .route("/api/dictionary", post(api_add_dictionary_entry))
        .route("/api/dictionary/{key}", delete(api_delete_dictionary_entry))
        // Histórico
        .route("/api/transcriptions", get(api_list_transcriptions))
        .route("/api/transcriptions/{id}", delete(api_delete_transcription))
        .route("/api/history/clear", post(api_clear_history))
        // Devices
        .route("/api/devices", get(api_list_devices))
        // Training
        .route("/api/training/upload", post(api_training_upload))
        // WebSocket
        .route("/ws", get(ws_handler))
        // Servir demais arquivos estáticos do diretório dist
        .fallback_service(ServeDir::new(static_dir))
        // CORS
        .layer(CorsLayer::permissive())
        // Estado compartilhado (LumenState centralizado)
        .with_state(state);

    // Iniciar servidor
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("🌐 Dashboard web disponível em: http://localhost:{}", port);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&addr).await
            .expect("Falha ao fazer bind do servidor web");
        axum::serve(listener, app).await
            .expect("Falha ao iniciar servidor web");
    });

    Ok(())
}

/// Retorna o diretório de arquivos estáticos
fn get_static_dir() -> String {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

    let candidates = [
        exe_dir.join("frontend/dist"),
        exe_dir.join("../share/lumen/frontend/dist"),
        exe_dir.join("../../src/ui/web/frontend/dist"), // From target/debug/ dir
        std::path::PathBuf::from("src/ui/web/frontend/dist"),
        std::path::PathBuf::from("/usr/share/lumen/frontend/dist"),
        std::path::PathBuf::from("/usr/local/share/lumen/frontend/dist"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
    }

    "src/ui/web/frontend/dist".to_string()
}

// ═══════════════════════════════════════════════════════════════
// Route Handlers
// ═══════════════════════════════════════════════════════════════

async fn serve_index_from_disk(static_dir: String) -> impl IntoResponse {
    let index_path = std::path::Path::new(&static_dir).join("index.html");
    match tokio::fs::read_to_string(&index_path).await {
        Ok(html) => Html(html),
        Err(e) => {
            tracing::error!("Falha ao ler index.html de '{}': {}", index_path.display(), e);
            Html("<html><body><h1>Dashboard indisponível</h1><p>Falha ao carregar frontend.</p></body></html>".to_string())
        }
    }
}

async fn serve_index() -> impl IntoResponse {
    // Mantido apenas para compatibilidade; não utilizado no roteamento atual
    let html = include_str!("frontend/dist/index.html");
    Html(html)
}

async fn api_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn api_status(
    State(state): State<Arc<LumenState>>,
) -> Json<StatusResponse> {
    let is_recording = *state.is_recording.read().await;
    let session = state.session.read().await;

    Json(StatusResponse {
        status: if is_recording { "recording".into() } else { "idle".into() },
        is_recording,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: session.uptime_seconds(),
        total_transcriptions: session.total_transcriptions,
        total_words: session.total_words,
    })
}

async fn api_get_config(
    State(state): State<Arc<LumenState>>,
) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(serde_json::to_value(&*config).unwrap_or_default())
}

async fn api_update_config(
    State(state): State<Arc<LumenState>>,
    Json(new_config): Json<LumenConfig>,
) -> impl IntoResponse {
    // Validar antes de aceitar
    if let Err(e) = new_config.validate() {
        return (StatusCode::BAD_REQUEST, format!("Config inválida: {}", e)).into_response();
    }

    {
        let mut config = state.config.write().await;
        *config = new_config.clone();
    }

    // Salvar no disco
    if let Err(e) = new_config.save() {
        tracing::error!("Falha ao salvar config: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao salvar configuração").into_response();
    }

    state.emit(LumenEvent::ConfigChanged);
    (StatusCode::OK, "Configuração atualizada").into_response()
}

// ── Snippets (operando nos componentes VIVOS) ──

async fn api_list_snippets(
    State(state): State<Arc<LumenState>>,
) -> Json<Vec<SnippetEntry>> {
    let snippets = state.snippets.read().await;
    let entries: Vec<SnippetEntry> = snippets.list()
        .iter()
        .map(|(k, v)| SnippetEntry { trigger: k.clone(), text: v.clone() })
        .collect();
    Json(entries)
}

async fn api_add_snippet(
    State(state): State<Arc<LumenState>>,
    Json(req): Json<SnippetRequest>,
) -> impl IntoResponse {
    // Atualizar componente VIVO
    {
        let mut snippets = state.snippets.write().await;
        snippets.add(req.trigger.clone(), req.text.clone());
    }

    // Sincronizar config para persistência
    {
        let mut config = state.config.write().await;
        config.snippets.entries.insert(req.trigger.clone(), req.text.clone());
    }

    // Auto-save no disco
    state.save_config().await;
    state.emit(LumenEvent::SnippetsUpdated);

    tracing::info!("Snippet adicionado via API: '{}'", req.trigger);
    (StatusCode::CREATED, "Snippet adicionado").into_response()
}

async fn api_delete_snippet(
    State(state): State<Arc<LumenState>>,
    axum::extract::Path(trigger): axum::extract::Path<String>,
) -> impl IntoResponse {
    let removed = {
        let mut snippets = state.snippets.write().await;
        snippets.remove(&trigger)
    };

    if removed {
        // Sincronizar config
        {
            let mut config = state.config.write().await;
            config.snippets.entries.remove(&trigger);
        }
        state.save_config().await;
        state.emit(LumenEvent::SnippetsUpdated);
        (StatusCode::OK, "Snippet removido").into_response()
    } else {
        (StatusCode::NOT_FOUND, "Snippet não encontrado").into_response()
    }
}

// ── Dictionary (operando nos componentes VIVOS) ──

async fn api_list_dictionary(
    State(state): State<Arc<LumenState>>,
) -> Json<Vec<DictionaryResponseEntry>> {
    // Lê do config (tem context/icon), não do CustomDictionary (que normaliza)
    let config = state.config.read().await;
    let entries: Vec<DictionaryResponseEntry> = config.dictionary.entries
        .iter()
        .map(|(k, v)| DictionaryResponseEntry {
            key: k.clone(),
            value: v.value.clone(),
            context: v.context.clone(),
            icon_type: v.icon_type.clone(),
        })
        .collect();
    Json(entries)
}

async fn api_add_dictionary_entry(
    State(state): State<Arc<LumenState>>,
    Json(req): Json<DictionaryRequest>,
) -> impl IntoResponse {
    // Atualizar componente VIVO
    {
        let mut dict = state.dictionary.write().await;
        dict.add(req.key.clone(), req.value.clone());
    }

    // Sincronizar config
    {
        let mut config = state.config.write().await;
        config.dictionary.entries.insert(req.key.clone(), DictionaryEntryData {
            value: req.value.clone(),
            context: req.context.clone(),
            icon_type: req.icon_type.clone(),
        });
    }

    state.save_config().await;
    state.emit(LumenEvent::DictionaryUpdated);

    tracing::info!("Dicionário: '{}' → '{}'", req.key, req.value);
    (StatusCode::CREATED, "Entrada adicionada").into_response()
}

async fn api_delete_dictionary_entry(
    State(state): State<Arc<LumenState>>,
    axum::extract::Path(key): axum::extract::Path<String>,
) -> impl IntoResponse {
    let removed = {
        let mut dict = state.dictionary.write().await;
        dict.remove(&key)
    };

    if removed {
        {
            let mut config = state.config.write().await;
            config.dictionary.entries.remove(&key);
        }
        state.save_config().await;
        state.emit(LumenEvent::DictionaryUpdated);
        (StatusCode::OK, "Entrada removida").into_response()
    } else {
        (StatusCode::NOT_FOUND, "Entrada não encontrada").into_response()
    }
}

async fn api_stats(
    State(state): State<Arc<LumenState>>,
) -> Json<serde_json::Value> {
    let session = state.session.read().await;
    let uptime_secs = session.uptime_seconds();

    Json(serde_json::json!({
        "uptime_seconds": uptime_secs,
        "uptime_formatted": format_duration(uptime_secs),
        "total_transcriptions": session.total_transcriptions,
        "total_words": session.total_words,
        "avg_words_per_transcription": if session.total_transcriptions > 0 {
            session.total_words as f64 / session.total_transcriptions as f64
        } else {
            0.0
        },
    }))
}

async fn api_list_devices() -> Json<serde_json::Value> {
    let devices = crate::audio::capture::AudioCapture::list_devices()
        .unwrap_or_default();

    let json_list: Vec<serde_json::Value> = devices.into_iter().map(|(id, label)| {
        serde_json::json!({
            "id": id,
            "label": label
        })
    }).collect();

    Json(serde_json::json!({ "devices": json_list }))
}

async fn api_training_upload(body: Bytes) -> impl IntoResponse {
    let training_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("lumen")
        .join("training");

    if let Err(e) = tokio::fs::create_dir_all(&training_dir).await {
        tracing::error!("Falha ao criar diretório de treinamento: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao preparar diretório de treino").into_response();
    }

    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, "Body vazio no upload de treino").into_response();
    }

    let out_path = training_dir.join(format!("raw_upload_{}.bin", uuid::Uuid::new_v4()));
    if let Err(e) = tokio::fs::write(&out_path, &body).await {
        tracing::error!("Falha ao salvar upload bruto de treino '{}': {}", out_path.display(), e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao salvar upload de treino").into_response();
    }

    tracing::info!(
        "Treinamento: upload recebido ({} bytes) salvo em {}",
        body.len(),
        out_path.display()
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "bytes": body.len(),
            "saved_path": out_path.to_string_lossy().to_string(),
            "note": "Upload bruto salvo; parsing multipart ainda pendente no backend"
        })),
    )
        .into_response()
}

/// Formata duração em formato legível
fn format_duration(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// ── Histórico (Analytics DB) ──

async fn api_list_transcriptions(
    State(state): State<Arc<LumenState>>,
    axum::extract::Query(query): axum::extract::Query<PaginationQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0);

    match state.db.get_recent_transcriptions(limit, offset) {
        Ok(records) => Json(records).into_response(),
        Err(e) => {
            tracing::error!("Falha ao buscar histórico: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao buscar histórico").into_response()
        }
    }
}

async fn api_delete_transcription(
    State(state): State<Arc<LumenState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.db.delete_transcription(&id) {
        Ok(true) => (StatusCode::OK, "Registro removido").into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Registro não encontrado").into_response(),
        Err(e) => {
            tracing::error!("Falha ao remover registro do histórico: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao remover registro").into_response()
        }
    }
}

async fn api_clear_history(
    State(state): State<Arc<LumenState>>,
) -> impl IntoResponse {
    match state.db.clear_history() {
        Ok(_) => (StatusCode::OK, "Histórico limpo").into_response(),
        Err(e) => {
            tracing::error!("Falha ao limpar histórico: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Falha ao limpar histórico").into_response()
        }
    }
}

