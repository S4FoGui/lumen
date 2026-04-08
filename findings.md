# 🔍 Findings & Research

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
| **Detecção de fim de fala (VAD)** | ❌ NÃO existe | 🔴 Alta |
| **Comandos de voz ("torne profissional")** | ❌ NÃO existe | 🔴 Alta |
| **Agent Mode** | ❌ NÃO existe | 🟡 Média |
| **Suporte 100+ idiomas** | ⚠️ Parcial (1 idioma por config) | 🟡 Média |
| **Freemium (2000 palavras/semana)** | ❌ NÃO existe | 🟡 Futura |
| Histórico de transcrições | ❌ NÃO existe | 🟡 Média |
| WebSocket tempo real | ❌ NÃO existe | 🟡 Média |
| State Bus centralizado | ❌ NÃO existe | 🔴 Alta |

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
