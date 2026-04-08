# 🔍 Findings & Research

## [NEW] Token Saver MCP Technical Specs
- **Name:** token-saver
- **Platform:** Node.js (MCP SDK)
- **Tools:** `summarize_file`, `list_project_structure`, `extract_relevant_code`, `diff_summary`, `count_tokens_estimate`.
- **Objective:** Reduce token usage by sending abstracts/subsets of code instead of full files.

### Implementation Logic
- **`summarize_file`**: Filters for imports, exports, and function/class signatures.
- **`list_project_structure`**: Recursive tree view with ignored directories (node_modules, .git, dist).
- **`extract_relevant_code`**: Context-aware extraction based on line matching and brace depth.
- **`diff_summary`**: Aggregates added/removed lines per file from a git diff.
- **`count_tokens_estimate`**: Heuristic estimation (~0.75 tokens/word and ~4 chars/token).

---

## Identidade do Lumen (Definição Oficial do Criador)

O Lumen é uma **ferramenta avançada de ditado por voz baseada em IA** que funciona como uma camada invisível sobre qualquer aplicativo do sistema. Seu fluxo é:

1. **Hotkey Global** → Usuário pressiona atalho para ativar escuta
2. **Captura de Áudio** → Grava a voz do microfone
3. **Transcrição Instantânea** → Whisper converte áudio em texto
4. **Processamento IA** → Remove fillers, corrige gramática, adiciona pontuação
5. **Inserção Direta** → Texto aparece onde o cursor estiver (qualquer app)
6. **Auto-Send** → Detecta fim de fala e pode pressionar Enter automaticamente

### Diferenciais vs Ditado Padrão (Windows/macOS)
- Remove "humm", "tipo", "né" automaticamente
- Texto sai com **pontuação perfeita** e gramática corrigida
- Aceita **comandos de voz** para transformação ("torne mais profissional")
- **Agent Mode (Beta)** — IA realiza ações contextuais complexas

### Features Descritas pelo Criador
| Feature | Status no Código | Prioridade |
|---------|-----------------|------------|
| Hotkey global → captura → transcrição | ✅ Implementado | — |
| Injeção de texto (xdotool/wtype/clipboard) | ✅ Implementado | — |
| Remoção de fillers | ✅ Implementado | — |
| Dicionário customizado | ✅ Implementado | — |
| Snippets de voz | ✅ Implementado | — |
| Formatação IA (Ollama/OpenAI/Gemini) | ✅ Implementado | — |
| Multi-provider IA | ✅ Implementado | — |
| **Auto-Send (Enter)** | ❌ NÃO existe | 🔴 Alta |
| **Detecção de fim de fala (VAD)** | ⚠️ v1.0.2 Fix Applied | 🔴 Alta |
| **Comandos de voz ("torne profissional")** | ❌ NÃO existe | 🔴 Alta |
| **Agent Mode** | ❌ NÃO existe | 🟡 Média |
| **Suporte 100+ idiomas** | ⚠️ Parcial (1 idioma por config) | 🟡 Média |
| **Freemium (2000 palavras/semana)** | ❌ NÃO existe | 🟡 Futura |
| Histórico de transcrições | ✅ v1.0.2 Fix Applied | 🟡 Média |
| WebSocket tempo real | ✅ Implementado | 🟡 Média |
| State Bus centralizado | ✅ Implementado | 🔴 Alta |

## Decisões Aprovadas pelo Usuário
- **WebSocket** para streaming de eventos (não SSE)
- **SQLite (rusqlite bundled)** para persistência de histórico
- **Execução em fases** (incremental)

## MCP Servers Integrated
- **shadcn/ui:** `npx -y shadcn mcp`
- **Convex:** `npx -y convex@latest mcp start`
- **Brave Browser (v2.0 - CDP Support Discovery):** Script `brave-mcp-setup.sh` updated with CDP support. 
    - **New Tools:** `brave_list_tabs`, `brave_read_page`, `brave_screenshot`, `brave_execute_js`.
    - **Capabilities:** Chrome DevTools Protocol (CDP) on port 9222.
    - **Target:** Flatpak Brave Browser (`com.brave.Browser`).
    - **Location (Proposed):** `/home/gui/.local/share/mcp-servers/brave-browser/server.py`
- **Global Config Path:** `/home/gui/.gemini/antigravity/mcp_config.json`

## Project Context
- **Root:** `/home/gui/Downloads/projetos/Projeto_guilherme/lumen`
- **Stack:** Rust (binário único ~20MB) + React/Vite frontend
- **Server:** Axum em localhost:8484

## [2026-04-08] KDE Wayland & Performance Findings
- **Wayland Virtual Keyboard Protocol:** O KDE Plasma 6 rejeita o protocolo `zwp_virtual_keyboard_v1` usado pelo `wtype`. O método mais confiável para injeção de texto nativa sem permissões de root é **Clipboard + Paste (wl-copy + Ctrl+V)**.
- **XWayland & Focus:** Ferramentas baseadas em X11 (xdotool) têm dificuldade com foco em janelas nativas Wayland. Delays de pelo menos 250ms são necessários após a cópia para assegurar o foco.
- **Whisper Medium Latency:** Modelos de IA maiores (1.5GB+) travam o loop principal do sistema se processados de forma síncrona. O uso de `tokio::spawn` para isolar a transcrição é mandatório para manter a responsividade das hotkeys.
