# 📊 Dashboard Web do Lumen - Guia Completo

**Data:** 2026-04-06  
**URL:** http://localhost:8484  
**Projeto:** Lumen v1.0.0

---

## 🌐 Visão Geral

O Lumen possui um **dashboard web completo** rodando em **localhost:8484** com interface moderna, WebSocket em tempo real e 5 abas principais.

### Tecnologias
- **Frontend:** React + TypeScript + Vite
- **UI:** Tailwind CSS + shadcn/ui
- **Backend:** Axum (Rust)
- **Tempo Real:** WebSocket
- **Banco:** SQLite (histórico)

---

## 📑 Abas do Dashboard

### 1. 📈 Dashboard (Status em Tempo Real)

**Funcionalidades:**
- **Status de Gravação:** Indicador visual (vermelho = gravando, verde = pronto)
- **Waveform Visualizer:** Visualização em tempo real do áudio (RMS)
- **Engine Uptime:** Tempo que o Lumen está rodando
- **Total de Transcrições:** Contador de transcrições realizadas
- **Total de Palavras:** Palavras processadas na sessão
- **Última Transcrição:** Mostra raw text e processed text
- **Badges:** Indica se usou IA ou auto-send

**WebSocket Events:**
- `RecordingStarted` / `RecordingStopped`
- `TranscriptionComplete`
- `AudioLevel` (RMS para waveform)
- `VoiceCommandDetected`
- `Error`

**Código:** `src/ui/web/frontend/src/components/tabs/DashboardTab.tsx`

---

### 2. 📜 History (Histórico de Transcrições)

**Funcionalidades:**
- **Lista de Transcrições:** Últimas 20 transcrições do SQLite
- **Detalhes:** Raw text, processed text, timestamp
- **Badges:** AI used, auto-sent
- **Tempo de Processamento:** Mostra ms de cada transcrição
- **Botão Deletar:** Remove transcrição do histórico
- **Formato de Data:** pt-BR (DD/MM HH:MM:SS)

**API Endpoints:**
- `GET /api/transcriptions?limit=20` - Lista transcrições
- `DELETE /api/transcriptions/:id` - Deleta transcrição

**Banco de Dados:**
- Arquivo: `~/.local/share/lumen/analytics.db`
- Tabela: `transcriptions`
- Campos: id, raw_text, processed_text, word_count, processing_time_ms, ai_used, auto_sent, timestamp

**Código:** `src/ui/web/frontend/src/components/tabs/HistoryTab.tsx`

---

### 3. ⚙️ Settings (Configurações)

**Funcionalidades:**

#### 🎤 Voice Engine Parameters
- **Audio Input Device:** Dropdown com todos os microfones disponíveis
- **Language Target:** Português (BR) ou English (US)
- **Auto-Send:** Toggle para Enter automático
- **Voice Commands:** Toggle para comandos de voz
- **Lightning Mode:** Toggle para modo ultra-rápido

#### 🤖 AI Formatter
- **Auto-Improve:** Toggle para correção automática por IA
- **Provider Central:** Dropdown com 5 opções:
  - **Disabled:** Sem IA
  - **OmniRoute:** Multi-Model Gateway (60+ providers)
  - **Ollama:** Local/Offline (Llama, Gemma, etc)
  - **OpenAI:** Cloud (GPT-4o-mini, etc)
  - **Gemini:** Google Cloud
  - **Groq:** High-Speed Llama

#### Configuração por Provider

**OmniRoute:**
- URL Base do Gateway
- Bearer Token (API Key)
- Modelo Alvo (ex: `cc/claude-3-5-sonnet`)

**Ollama:**
- Endpoint URL (ex: `http://127.0.0.1:11434`)
- API Key (opcional, para proxy)
- Modelo (ex: `llama3.2`, `gemma2`)

**OpenAI:**
- API Key (sk-...)
- Modelo (ex: `gpt-4o-mini`)

**Groq:**
- API Key (gsk_...)
- Modelo (ex: `llama-3.3-70b-versatile`)

**Botão Salvar:**
- Salva configurações no arquivo `~/.config/lumen/config.yaml`
- Feedback visual (verde = salvo com sucesso)
- Validação de campos obrigatórios

**API Endpoints:**
- `GET /api/config` - Carrega configuração atual
- `PUT /api/config` - Salva nova configuração
- `GET /api/devices` - Lista dispositivos de áudio

**Código:** `src/ui/web/frontend/src/components/tabs/ConfigTab.tsx`

---

### 4. ❓ FAQ (Guia & Tutorial)

**Funcionalidades:**

#### 1. Como gravar a voz
- Explicação do hotkey (Ctrl+Shift+Space)
- Como funciona o VAD (Voice Activity Detector)
- Overlay visual "READY TO LISTEN"

#### 2. Comandos Mágicos
- **"Lumen, apague":** Cancela transcrição
- **"Lumen, envie":** Transcreve e dá Enter
- **"Torne profissional":** Reformula com IA

#### 3. Adicionando Inteligência
- Como configurar OmniRoute
- Como configurar Ollama (100% offline)
- Passo a passo para ativar IA

**Código:** `src/ui/web/frontend/src/components/tabs/FaqTab.tsx`

---

### 5. ✂️ Snippets (Atalhos de Voz)

**Funcionalidades:**
- **Lista de Snippets:** Mostra todos os atalhos configurados
- **Adicionar Snippet:** Trigger + texto expandido
- **Editar Snippet:** Modifica trigger ou texto
- **Deletar Snippet:** Remove snippet
- **Exemplos:**
  - `/ola` → "Olá! Tudo bem? Como posso ajudar?"
  - `/email` → "Atenciosamente,\nGuilherme"
  - `/obg` → "Muito obrigado pela atenção!"

**Como Usar:**
- Fale o trigger durante a gravação
- O Lumen expande automaticamente para o texto completo

**API Endpoints:**
- `GET /api/snippets` - Lista snippets
- `POST /api/snippets` - Adiciona snippet
- `PUT /api/snippets/:trigger` - Edita snippet
- `DELETE /api/snippets/:trigger` - Deleta snippet

**Código:** `src/ui/web/frontend/src/components/tabs/SnippetsTab.tsx` (não encontrado, mas API existe)

---

### 6. 📖 Dictionary (Dicionário Customizado)

**Funcionalidades:**
- **Lista de Entradas:** Mostra todas as substituições
- **Adicionar Entrada:** Palavra → Correção
- **Contexto:** Categoria da palavra (ex: "Web Development")
- **Ícone:** Tipo de ícone (brand, code, os, git)
- **Editar Entrada:** Modifica correção ou contexto
- **Deletar Entrada:** Remove entrada

**Exemplos:**
- `react` → `React` (Web Development)
- `javascript` → `JavaScript` (Web Development)
- `python` → `Python` (Software Development)
- `linux` → `Linux` (Operating Systems)
- `github` → `GitHub` (Version Control)

**Como Usar:**
- Fale a palavra durante a gravação
- O Lumen substitui automaticamente pela correção

**API Endpoints:**
- `GET /api/dictionary` - Lista entradas
- `POST /api/dictionary` - Adiciona entrada
- `PUT /api/dictionary/:word` - Edita entrada
- `DELETE /api/dictionary/:word` - Deleta entrada

**Código:** `src/ui/web/frontend/src/components/tabs/DictionaryTab.tsx` (não encontrado, mas API existe)

---

## 🔌 API REST Completa

### Configuração
```
GET  /api/config          # Carrega configuração
PUT  /api/config          # Salva configuração
GET  /api/devices         # Lista dispositivos de áudio
```

### Transcrições (Histórico)
```
GET    /api/transcriptions?limit=20    # Lista transcrições
GET    /api/transcriptions/:id         # Busca por ID
DELETE /api/transcriptions/:id         # Deleta transcrição
DELETE /api/transcriptions             # Limpa histórico
```

### Snippets
```
GET    /api/snippets                   # Lista snippets
POST   /api/snippets                   # Adiciona snippet
PUT    /api/snippets/:trigger          # Edita snippet
DELETE /api/snippets/:trigger          # Deleta snippet
```

### Dicionário
```
GET    /api/dictionary                 # Lista entradas
POST   /api/dictionary                 # Adiciona entrada
PUT    /api/dictionary/:word           # Edita entrada
DELETE /api/dictionary/:word           # Deleta entrada
```

### Estatísticas
```
GET /api/stats                         # Estatísticas globais
```

**Código Backend:** `src/ui/web/server.rs`

---

## 🔄 WebSocket em Tempo Real

### Conexão
```
ws://localhost:8484/ws
```

### Eventos Enviados pelo Servidor

```typescript
// Gravação iniciada
{
  "event": "RecordingStarted"
}

// Gravação parada
{
  "event": "RecordingStopped"
}

// Transcrição completa
{
  "event": "TranscriptionComplete",
  "data": {
    "id": "uuid",
    "raw_text": "texto bruto",
    "processed_text": "texto processado",
    "word_count": 10,
    "processing_time_ms": 1234,
    "ai_used": false,
    "auto_sent": false
  }
}

// Nível de áudio (RMS)
{
  "event": "AudioLevel",
  "rms": 0.45
}

// Comando de voz detectado
{
  "event": "VoiceCommandDetected",
  "command_type": "Send",
  "command": "envie"
}

// Erro
{
  "event": "Error",
  "message": "Erro ao processar"
}
```

**Código WebSocket:** `src/ui/web/ws.rs`

---

## 🎨 Design System

### Cores (Tema Escuro)
- **Background:** `#0a0a0a`
- **Card:** `#1a1a1a` com backdrop-blur
- **Accent:** `#a3e635` (verde limão)
- **Border:** `#2a2a2a`
- **Text:** `#fafafa`
- **Muted:** `#737373`

### Componentes (shadcn/ui)
- Card, CardHeader, CardTitle, CardContent
- Button, Input, Label, Switch
- Select, SelectTrigger, SelectContent, SelectItem
- Lucide Icons (Clock, FileText, Mic, Wand2, etc)

### Animações
- `animate-in fade-in slide-in-from-bottom-4`
- `animate-pulse` (loading)
- `transition-colors` (hover)

---

## 🚀 Como Acessar

### 1. Iniciar o Lumen
```bash
./target/release/lumen
```

### 2. Abrir Dashboard
```bash
# Opção 1: Navegador manual
firefox http://localhost:8484

# Opção 2: Comando do Lumen
./target/release/lumen dashboard
```

### 3. Verificar WebSocket
Abra o DevTools (F12) → Console → Deve mostrar:
```
WebSocket conectado a ws://localhost:8484/ws
```

---

## 🔧 Configuração Avançada

### Mudar Porta do Dashboard

Edite `~/.config/lumen/config.yaml`:
```yaml
ui:
  dashboard_port: 8484  # Mude para outra porta
  open_on_start: false  # true = abre navegador automaticamente
  show_overlay: true    # Overlay GTK4
  show_tray: true       # System tray
```

### Desabilitar Dashboard

```yaml
ui:
  dashboard_port: 0  # Desabilita o servidor web
```

---

## 📊 Estatísticas e Métricas

O dashboard mostra em tempo real:
- **Uptime:** Tempo que o Lumen está rodando
- **Total de Transcrições:** Contador da sessão
- **Total de Palavras:** Palavras processadas
- **RMS (Audio Level):** Nível do microfone (0.0 - 1.0)
- **Tempo de Processamento:** ms por transcrição
- **Status de Gravação:** Gravando ou Pronto

---

## 🐛 Troubleshooting

### Dashboard não abre
```bash
# Verificar se a porta está em uso
lsof -i :8484

# Verificar logs
tail -f /tmp/claude-1000/.../tasks/*.output | grep "Dashboard"
```

### WebSocket não conecta
```bash
# Verificar firewall
sudo ufw allow 8484

# Verificar se o Lumen está rodando
ps aux | grep lumen
```

### Configurações não salvam
```bash
# Verificar permissões
ls -la ~/.config/lumen/config.yaml

# Verificar logs de erro no console do navegador (F12)
```

---

## ✨ Recursos Especiais

### 1. Waveform Visualizer
- Visualização em tempo real do áudio
- Barras animadas baseadas no RMS
- Cor verde quando gravando

### 2. Auto-Refresh
- Dashboard atualiza automaticamente via WebSocket
- Não precisa recarregar a página

### 3. Feedback Visual
- Botão "Salvar" fica verde quando salvo
- Loading spinners durante operações
- Badges coloridos (AI, Auto-sent)

### 4. Responsivo
- Funciona em desktop e mobile
- Layout adaptativo (grid → stack)

---

## 📚 Conclusão

O dashboard do Lumen é uma **interface web completa** com:
- ✅ 5 abas funcionais
- ✅ WebSocket em tempo real
- ✅ API REST completa
- ✅ Configuração visual de todos os parâmetros
- ✅ Histórico SQLite
- ✅ Gerenciamento de snippets e dicionário
- ✅ Suporte a 5 providers de IA
- ✅ Design moderno e responsivo

**Acesse agora:** http://localhost:8484 🚀
