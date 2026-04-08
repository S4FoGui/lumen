# Lumen — Descrição Completa do Projeto

## Visão Geral

**Lumen** é um ecossistema de produtividade por voz para Linux, escrito inteiramente em Rust. Ele permite ditar texto com transcrição local via Whisper, limpar automaticamente vícios de fala (como "humm", "ééé"), formatar o texto via IA (Ollama, OpenAI, Gemini ou OmniRoute) e injetar o resultado diretamente em qualquer aplicativo — tudo com um único binário estático de ~20MB, sem dependências de Python ou pip.

---

## Arquitetura

O projeto segue uma arquitetura modular com os seguintes componentes principais:

### Módulos do Código-Fonte

| Módulo | Arquivo(s) | Responsabilidade |
|--------|-----------|------------------|
| **Config** | `src/config.rs` | Carregamento, validação e persistência de configuração YAML (`~/.config/lumen/config.yaml`). Suporta áudio, transcrição, hotkeys, injeção de texto, IA, snippets, dicionário, UI e logging. |
| **State Hub** | `src/state.rs` | Hub centralizado de estado compartilhado entre event loop, API web e overlay. Gerencia configuração, dicionário, snippets, AI formatter, analytics, status de gravação e um event bus via `broadcast::channel`. |
| **Error** | `src/error.rs` | Tipos de erro customizados (`LumenError`) para captura de áudio, motor Whisper, IA, configuração, analytics, hotkeys e erros internos. |
| **Analytics** | `src/analytics.rs` | Persistência do histórico de transcrições em SQLite (`~/.local/share/lumen/analytics.db`). Suporta salvar, deletar, limpar histórico, paginação e estatísticas globais. |
| **Audio** | `src/audio/capture.rs`, `src/audio/vad.rs` | Captura de áudio via cpal (ALSA/PulseAudio/PipeWire) e Voice Activity Detector (VAD) baseado em energia RMS com suavização EMA para detecção automática de fim de fala. |
| **Transcription** | `src/transcription/engine.rs`, `src/transcription/pipeline.rs`, `src/transcription/filler_filter.rs` | Motor de transcrição via whisper-rs/whisper.cpp com suporte a modo Lightning. Pipeline completo: Whisper → Filtro de Fillers → Detecção de Comandos de Voz → Dicionário → Snippets → IA → Injeção → Auto-Send. |
| **AI** | `src/ai/formatter.rs`, `src/ai/commands.rs` | Formatador de texto via IA com suporte a 4 providers: Ollama (local), OpenAI, Gemini e OmniRoute. Detector de comandos de voz ("envie", "apague", "torne profissional"). |
| **Text** | `src/text/injector.rs`, `src/text/snippets.rs`, `src/text/auto_send.rs` | Injetor de texto com detecção automática de sessão (X11/Wayland/Clipboard), gerenciador de snippets de voz e auto-send (Enter automático). |
| **Dictionary** | `src/dictionary/custom.rs` | Dicionário customizado para substituição de termos técnicos e palavras personalizadas. |
| **Hotkeys** | `src/hotkeys/manager.rs` | Gerenciador de atalhos globais compatível com X11 e Wayland via crate `global-hotkey`. |
| **UI** | `src/ui/web/`, `src/ui/overlay.rs`, `src/ui/tray.rs` | Dashboard web (axum em localhost:8484), overlay GTK4 para feedback visual e ícone de system tray. |

---

## Pipeline de Transcrição

Quando o usuário dita algo, o áudio passa pelo seguinte pipeline:

1. **Captura de Áudio** — cpal captura samples a 16kHz mono
2. **VAD (Voice Activity Detection)** — Detecta automaticamente quando o usuário parou de falar baseado em RMS com suavização EMA
3. **Whisper Transcription** — whisper.cpp transcreve o áudio para texto bruto
4. **Filler Filter** — Remove palavras de preenchimento ("humm", "né", "ééé", etc.)
5. **Voice Command Detection** — Detecta comandos como "envie", "apague", "torne profissional"
6. **Custom Dictionary** — Aplica substituições de termos técnicos configurados pelo usuário
7. **Snippet Expansion** — Expande atalhos de voz (ex: `/ola` → "Olá! Tudo bem?")
8. **AI Formatting** — Opcionalmente formata/corrige o texto via LLM configurado
9. **Text Injection** — Injeta o texto no aplicativo ativo via xdotool (X11), wtype (Wayland) ou clipboard
10. **Auto-Send** — Pressiona Enter automaticamente se configurado ou se comando de voz "envie" foi detectado

---

## Recursos Detalhados

### Nível 1 — Básico
- **Substituição de Digitação**: Dite e o texto aparece em qualquer aplicativo
- **Limpeza de Áudio**: Remove automaticamente fillers e vícios de fala
- **VAD Inteligente**: Detecção automática de fim de fala com RMS suavizado por EMA
- **Overlay Visual**: Feedback visual em tempo real do status de gravação

### Nível 2 — Intermediário
- **Dicionário Customizado**: Ensine termos técnicos e substituições personalizadas
- **Snippets de Voz**: Atalhos expansíveis (ex: `/email` → bloco de assinatura completo)
- **Comandos de Voz**: "Envie" (auto-send), "Apague" (deleta texto), "Torne profissional" (transformação via IA)

### Nível 3 — Avançado
- **Modo Lightning**: Transcrição bruta otimizada para velocidade máxima
- **Formatação IA**: Integração com Ollama, OpenAI, Gemini ou OmniRoute
- **Auto-Send**: Pressiona Enter automaticamente após transcrição
- **Analytics**: Histórico completo de transcrições com estatísticas via SQLite

---

## Stack Tecnológica

| Tecnologia | Crate | Finalidade |
|-----------|-------|-----------|
| **Rust 2021** | — | Linguagem principal, binário estático ~20MB |
| **Tokio** | `tokio` | Runtime async para event loop e concorrência |
| **Whisper** | `whisper-rs` | Transcrição local via whisper.cpp |
| **Captura de Áudio** | `cpal` | ALSA, PulseAudio e PipeWire |
| **Web Server** | `axum` + `tower-http` | Dashboard web em localhost:8484 com WebSocket |
| **HTTP Client** | `reqwest` | Comunicação com APIs de LLM |
| **Serialização** | `serde`, `serde_json`, `serde_yaml` | Config YAML e JSON |
| **Regex** | `regex` | Filtro de filler words |
| **Hotkeys Globais** | `global-hotkey` | Atalhos X11 + Wayland |
| **System Tray** | `tray-item` (ksni) | Ícone na bandeja do sistema |
| **Overlay** | `gtk4` | Interface visual sobreposta |
| **Wayland Overlay** | `gtk4-layer-shell` (opcional) | Suporte a layer-shell no Wayland |
| **Diretórios XDG** | `dirs` | Paths padrão do Linux |
| **CLI** | `clap` | Interface de linha de comando |
| **Logging** | `tracing`, `tracing-subscriber` | Logs estruturados com env-filter |
| **Banco de Dados** | `rusqlite` (bundled) | SQLite para histórico de transcrições |
| **Clipboard** | `arboard` | Acesso nativo ao clipboard |
| **Utilidades** | `chrono`, `anyhow`, `uuid`, `thiserror` | Data/hora, erros, UUIDs |

---

## Configuração

Arquivo: `~/.config/lumen/config.yaml`

### Seções de Configuração

- **audio**: Dispositivo de captura, sample rate (16000), canais (1)
- **transcription**: Caminho do modelo, idioma, modo lightning, filler words, auto-send, threshold de silêncio (VAD), comandos de voz
- **hotkeys**: Atalhos para gravar/parar, modo lightning e abrir dashboard
- **text_injection**: Método (auto/x11/wayland/clipboard) e delay
- **ai**: Provider (ollama/openai/gemini/omniroute/disabled), auto-formatting, credenciais e modelos de cada provider, instrução padrão
- **snippets**: Mapa de atalhos → textos expansíveis
- **dictionary**: Mapa de termos → substituições com contexto opcional
- **ui**: Porta do dashboard, abrir no start, mostrar overlay, mostrar tray
- **logging**: Nível de log e arquivo opcional

---

## Comandos CLI

```bash
lumen              # Iniciar o Lumen (modo normal)
lumen config       # Exibir caminhos de configuração e dados
lumen dashboard    # Abrir dashboard no navegador
lumen devices      # Listar dispositivos de áudio disponíveis
```

### Atalhos Padrão

| Atalho | Ação |
|--------|------|
| `Enter Enter` (double-tap) | Gravar / Parar gravação (pressione 2x rapidamente) |
| `Ctrl+Shift+L` | Modo Lightning |
| `Ctrl+Shift+D` | Abrir Dashboard |

---

## State Hub e Event Bus

O `LumenState` é o hub central que gerencia todo o estado compartilhado:

- **Config**: `RwLock<LumenConfig>` — configuração carregada do YAML
- **Dictionary**: `RwLock<CustomDictionary>` — dicionário customizado vivo
- **Snippets**: `RwLock<SnippetManager>` — gerenciador de snippets vivo
- **AI Formatter**: `Arc<AiFormatter>` — formatador de texto via IA
- **Analytics DB**: `Arc<Analytics>` — banco SQLite de histórico
- **Recording Status**: `RwLock<bool>` — estado de gravação
- **Session Stats**: `RwLock<SessionStats>` — métricas da sessão atual
- **Event Bus**: `broadcast::Sender<LumenEvent>` — eventos broadcast para WebSocket, overlay e log

### Eventos (`LumenEvent`)

- `RecordingStarted` / `RecordingStopped`
- `TranscriptionComplete` (com id, raw, processed, words, processing_time, auto_sent)
- `VoiceCommandDetected` (com command_type e command)
- `AudioLevel` (com rms)
- `DictionaryUpdated` / `SnippetsUpdated` / `ConfigChanged`
- `Error` (com message)

---

## Instalação

### Arch Linux (AUR)
```bash
yay -S lumen
```

### Debian/Ubuntu
```bash
sudo dpkg -i lumen-1.0.0-amd64.deb
```

### Do Código Fonte
```bash
git clone https://github.com/guilherme/lumen.git
cd lumen
chmod +x scripts/install.sh
./scripts/install.sh
```

### Download do Modelo Whisper
```bash
mkdir -p ~/.local/share/lumen/models
curl -L -o ~/.local/share/lumen/models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

### Dependências do Sistema

**Arch Linux:**
```bash
sudo pacman -S gtk4 alsa-lib libnotify xdotool
```

**Debian/Ubuntu:**
```bash
sudo apt install libgtk-4-1 libasound2 libnotify-bin xdotool
```

---

## Dashboard Web

Acesse `http://localhost:8484` para o painel de controle com:
- Status em tempo real via WebSocket
- Gerenciamento de snippets
- Dicionário customizado
- Estatísticas de uso
- Configuração visual
- Histórico de transcrições

---

## Perfis de Build

```toml
[profile.release]
opt-level = 3
lto = true
strip = true
codegen-units = 1
```

### Features Opcionais

- `wayland-overlay`: Habilita suporte a `gtk4-layer-shell` para overlays nativos no Wayland

---

## Estrutura de Diretórios

```
lumen/
├── src/
│   ├── main.rs              # Entry point, CLI, event loop principal
│   ├── config.rs            # Configuração YAML
│   ├── state.rs             # State Hub + Event Bus
│   ├── error.rs             # Tipos de erro
│   ├── analytics.rs         # SQLite para histórico
│   ├── audio/
│   │   ├── capture.rs       # Captura de áudio via cpal
│   │   ├── vad.rs           # Voice Activity Detector
│   │   └── mod.rs
│   ├── transcription/
│   │   ├── engine.rs        # Motor Whisper
│   │   ├── pipeline.rs      # Pipeline completo de processamento
│   │   ├── filler_filter.rs # Filtro de fillers
│   │   └── mod.rs
│   ├── ai/
│   │   ├── formatter.rs     # Formatador via IA (4 providers)
│   │   ├── commands.rs      # Detector de comandos de voz
│   │   └── mod.rs
│   ├── text/
│   │   ├── injector.rs      # Injetor de texto (X11/Wayland/Clipboard)
│   │   ├── snippets.rs      # Gerenciador de snippets
│   │   ├── auto_send.rs     # Auto-send (Enter automático)
│   │   └── mod.rs
│   ├── dictionary/
│   │   ├── custom.rs        # Dicionário customizado
│   │   └── mod.rs
│   ├── hotkeys/
│   │   ├── manager.rs       # Gerenciador de hotkeys globais
│   │   └── mod.rs
│   └── ui/
│       ├── overlay.rs       # Overlay GTK4
│       ├── tray.rs          # System tray
│       ├── web/             # Dashboard web (axum)
│       └── mod.rs
├── config/
│   └── default.yaml         # Configuração padrão embutida
├── packaging/               # Scripts de empacotamento
├── scripts/                 # Scripts de instalação e utilitários
├── assets/                  # Recursos visuais
├── Cargo.toml               # Manifesto Rust
├── build.rs                 # Build script (cmake)
└── README.md
```

---

## Licença

MIT License — veja [LICENSE](LICENSE).
