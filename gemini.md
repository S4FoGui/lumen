# 📜 Project Constitution (gemini.md)

## 1. Identidade do Produto
O **Lumen** é uma ferramenta avançada de ditado por voz baseada em IA para Linux. Ele funciona como uma camada invisível sobre qualquer aplicativo, permitindo que o usuário "escreva" na velocidade da fala com texto polido e profissional.

**Fluxo Core:** Hotkey → Captura Áudio → Whisper → Filtro Fillers → Dicionário → IA Formatter → Injeção de Texto → (Auto-Send)

## 2. Regras Comportamentais
- **Data-First:** O Schema JSON de Input/Output decide como se programa a Lógica.
- **Separação:** UI (Stylize) é separada do Logic (Architect). Se uma API interna falhar, conserta-se o core, não apenas o JS cliente.
- **Confiabilidade:** O binário Lumen é uma ferramenta do sistema; nenhuma alteração deve causar vazamento de memória GTK4.
- **Privacy-First:** Processamento local prioritário (Whisper + Ollama). Dados nunca usados para treinar modelos.

## 3. Invariantes Arquiteturais (A.N.T)
- **Layer 1 (Architecture):** Atualização guiada por `.md`.
- **Layer 2 (Navigation):** O Axum serve a infra + WebSocket para tempo real.
- **Layer 3 (Tools):** Funções seguras (Rust) e renderização leve.

## 4. Data Schemas

### 4.1 Audio Input
```json
{
  "format": "f32 PCM",
  "sample_rate": 16000,
  "channels": 1,
  "encoding": "mono float32"
}
```

### 4.2 Transcription Record (Output Core)
```json
{
  "id": "uuid-v4",
  "timestamp": "2026-04-05T19:30:00Z",
  "raw_text": "humm eu uso javascript para programar",
  "processed_text": "Eu uso JavaScript para programar.",
  "word_count": 5,
  "processing_time_ms": 1234,
  "ai_used": true,
  "auto_sent": false,
  "voice_command_detected": null,
  "pipeline_stages": ["whisper", "filler_filter", "dictionary", "snippets", "ai"]
}
```

### 4.3 Voice Command Schema
```json
{
  "type": "transform" | "action" | "control",
  "command": "torne mais profissional",
  "target_text": "oi cara, valeu pelo trampo",
  "result": "Prezado, agradeço pelo excelente trabalho realizado."
}
```

### 4.4 WebSocket Event Stream
```json
{ "type": "recording_started", "data": {} }
{ "type": "recording_stopped", "data": {} }
{ "type": "audio_level", "data": { "rms": 0.42 } }
{ "type": "transcription_complete", "data": { /* TranscriptionRecord */ } }
{ "type": "voice_command_detected", "data": { /* VoiceCommand */ } }
{ "type": "auto_send_triggered", "data": { "text": "..." } }
{ "type": "error", "data": { "message": "..." } }
```

### 4.5 Dictionary Entry
```json
{
  "active_substitutions": [
    {
       "key": "rust",
       "value": "Rust (programming language)",
       "context": "Software Development, Systems Programming",
       "icon": "code"
    }
  ]
}
```

### 4.6 Authorized MCP Schema
```json
{
  "mcpServers": {
    "shadcn": { "command": "npx", "args": ["-y", "shadcn", "mcp"] },
    "convex": { "command": "npx", "args": ["-y", "convex@latest", "mcp", "start"] },
    "brave-browser": { "command": "python3", "args": ["/home/gui/.local/share/mcp-servers/brave-browser/server.py"] },
    "token-saver": { "command": "node", "args": ["/home/gui/token-saver-mcp/src/index.js"] }
  }
}

### 4.7 Token Saver Tool Payloads
```json
{
  "summarize_file": {
    "input": { "file_path": "string", "max_lines": "number (default: 80)" },
    "output": { "content": [{ "type": "text", "text": "string" }] }
  },
  "list_project_structure": {
    "input": { "root_path": "string", "max_depth": "number", "show_size": "boolean" },
    "output": { "content": [{ "type": "text", "text": "string" }] }
  },
  "extract_relevant_code": {
    "input": { "file_path": "string", "symbol": "string", "context_lines": "number" },
    "output": { "content": [{ "type": "text", "text": "string" }] }
  }
}
```
```

## 5. Decisões de Arquitetura (Aprovadas)
- **WebSocket** para streaming de eventos em tempo real
- **SQLite (rusqlite bundled)** para persistência de histórico e analytics
- **Execução em fases** incrementais (State Bus → API v2 → Pipeline → Analytics)
- **VAD (Voice Activity Detection)** para detectar fim de fala
- **Auto-Send (Enter)** quando o usuário termina de falar
