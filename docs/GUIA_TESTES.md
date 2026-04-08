# 🧪 Guia de Teste - Lumen v1.0.0

**Data:** 2026-04-06  
**Status:** Pronto para testes

---

## ⚡ Como Usar o Lumen

**ATALHO DE GRAVAÇÃO:** Pressione **Enter DUAS VEZES rapidamente** (double-tap)

- Janela de tempo: 400ms entre os dois toques
- Não precisa segurar nenhuma tecla (Ctrl/Shift/Alt)
- Funciona como double-click do mouse

---

## ✅ Pré-requisitos

### 1. Dependências do Sistema

#### Arch Linux
```bash
sudo pacman -S gtk4 alsa-lib libnotify xdotool wtype
```

#### Debian/Ubuntu
```bash
sudo apt install libgtk-4-1 libasound2 libnotify-bin xdotool
```

### 2. Modelo Whisper
```bash
# Criar diretório
mkdir -p ~/.local/share/lumen/models

# Baixar modelo (escolha um)
# Modelo small (recomendado) - ~500MB
curl -L -o ~/.local/share/lumen/models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# Modelo base (mais rápido) - ~150MB
curl -L -o ~/.local/share/lumen/models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin

# Modelo tiny (ultra rápido) - ~75MB
curl -L -o ~/.local/share/lumen/models/ggml-tiny.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
```

### 3. Configuração Inicial
```bash
# Criar diretório de config
mkdir -p ~/.config/lumen

# Copiar config padrão (se não existir)
cp config/default.yaml ~/.config/lumen/config.yaml

# Editar configuração
nano ~/.config/lumen/config.yaml
```

---

## 🚀 Executar o Lumen

### Opção 1: Modo Debug (desenvolvimento)
```bash
cargo run
```

### Opção 2: Release (produção)
```bash
# Compilar
cargo build --release

# Executar
./target/release/lumen
```

### Opção 3: Instalar no sistema
```bash
sudo cp target/release/lumen /usr/local/bin/
lumen
```

---

## 🧪 Testes Funcionais

### Teste 1: Verificar Dispositivos de Áudio
```bash
lumen devices
```

**Resultado esperado:**
```
🎤 Dispositivos de áudio disponíveis:
  [0] Default (default)
  [1] Built-in Audio (hw:0,0)
  [2] USB Microphone (hw:1,0)
```

### Teste 2: Verificar Configuração
```bash
lumen config
```

**Resultado esperado:**
```
📁 Arquivo de configuração: /home/user/.config/lumen/config.yaml
📂 Diretório de dados: /home/user/.local/share/lumen
```

### Teste 3: Abrir Dashboard
```bash
lumen dashboard
```

**Resultado esperado:**
- Navegador abre em `http://localhost:8484`
- Dashboard carrega com interface web

---

## 🎤 Testes de Transcrição

### Teste 4: Gravação Básica

1. **Iniciar Lumen:**
   ```bash
   lumen
   ```

2. **Pressionar Enter duas vezes rapidamente (double-tap):**
   - Pressione Enter 2x em menos de 400ms para iniciar gravação

3. **Falar algo:**
   - "Olá, este é um teste do Lumen"

4. **Aguardar silêncio:**
   - VAD detecta automaticamente após 4 segundos de silêncio
   - Ou pressione Enter 2x novamente para parar manualmente

5. **Verificar:**
   - Overlay mostra "Gravando..." durante captura
   - Texto aparece no aplicativo ativo após transcrição

**Resultado esperado:**
```
🎙️ Gravação iniciada (VAD ativo)
🎤 VAD: Fim de fala detectado — processando...
Raw transcription: "Olá, este é um teste do Lumen"
⏎ Enter pressionado automaticamente (se auto_send: true)
```

### Teste 5: VAD Automático

1. **Configurar VAD:**
   ```yaml
   # ~/.config/lumen/config.yaml
   transcription:
     silence_threshold_ms: 1500  # 1.5 segundos de silêncio
   ```

2. **Iniciar gravação:**
   - Pressione Enter 2x rapidamente (double-tap)

3. **Falar e parar:**
   - Fale algo e fique em silêncio por 1.5 segundos
   - VAD deve detectar automaticamente e parar

**Resultado esperado:**
- Gravação para automaticamente após silêncio
- Não precisa pressionar hotkey novamente

### Teste 6: Remoção de Fillers

1. **Falar com vícios:**
   - "Humm, eu acho que, ééé, isso funciona, né?"

**Resultado esperado:**
```
Raw: "Humm, eu acho que, ééé, isso funciona, né?"
Processed: "eu acho que isso funciona"
```

### Teste 7: Comandos de Voz

#### 7.1 Comando "Envie"
1. **Falar:**
   - "Olá mundo, envie"

**Resultado esperado:**
- Texto "Olá mundo" é injetado
- Enter é pressionado automaticamente

#### 7.2 Comando "Apague"
1. **Falar:**
   - "apague"

**Resultado esperado:**
- Nenhum texto é injetado
- Comando é detectado e ignorado

#### 7.3 Comando "Torne Profissional"
1. **Configurar IA:**
   ```yaml
   ai:
     provider: "ollama"
     auto_formatting: false
     ollama:
       url: "http://localhost:11434"
       model: "llama3.2"
   ```

2. **Falar:**
   - "torne mais profissional"

**Resultado esperado:**
- IA reformata o último texto
- Texto refinado é injetado

### Teste 8: Dicionário Customizado

1. **Configurar dicionário:**
   ```yaml
   dictionary:
     entries:
       kubernetes:
         value: "Kubernetes"
         context: "tecnologia"
       js:
         value: "JavaScript"
         context: "programação"
   ```

2. **Falar:**
   - "eu uso kubernetes e js"

**Resultado esperado:**
```
Raw: "eu uso kubernetes e js"
Processed: "eu uso Kubernetes e JavaScript"
```

### Teste 9: Snippets de Voz

1. **Configurar snippets:**
   ```yaml
   snippets:
     entries:
       "/ola": "Olá! Tudo bem?"
       "/email": "Atenciosamente,\nGuilherme"
       "/sig": "---\nGuilherme Silva\nDesenvolvedor"
   ```

2. **Falar:**
   - "/ola"

**Resultado esperado:**
```
Raw: "/ola"
Processed: "Olá! Tudo bem?"
```

### Teste 10: Formatação IA

1. **Configurar auto-formatting:**
   ```yaml
   ai:
     provider: "ollama"
     auto_formatting: true
   ```

2. **Falar:**
   - "oi tudo bem com voce"

**Resultado esperado:**
```
Raw: "oi tudo bem com voce"
AI Processing...
Processed: "Olá! Tudo bem com você?"
```

---

## 🌐 Testes do Dashboard

### Teste 11: WebSocket Tempo Real

1. **Abrir dashboard:**
   ```bash
   lumen dashboard
   ```

2. **Fazer uma gravação**

3. **Verificar no dashboard:**
   - Status muda para "Gravando..."
   - Nível de áudio (RMS) atualiza em tempo real
   - Transcrição aparece quando completa

### Teste 12: Gerenciar Snippets

1. **No dashboard:**
   - Ir para seção "Snippets"
   - Adicionar novo snippet: `/teste` → "Texto de teste"
   - Salvar

2. **Testar:**
   - Falar "/teste"
   - Verificar se expande para "Texto de teste"

### Teste 13: Histórico de Transcrições

1. **Fazer várias transcrições**

2. **No dashboard:**
   - Ir para seção "Histórico"
   - Verificar lista de transcrições
   - Verificar estatísticas (total de palavras, tempo de processamento)

---

## 🐛 Testes de Erro

### Teste 14: Sem Modelo Whisper

1. **Remover modelo:**
   ```bash
   mv ~/.local/share/lumen/models/ggml-small.bin /tmp/
   ```

2. **Iniciar Lumen:**
   ```bash
   lumen
   ```

**Resultado esperado:**
```
⚠️ Modelo Whisper não encontrado em: ~/.local/share/lumen/models/ggml-small.bin
   O Lumen iniciará sem transcrição.
   Baixe o modelo com:
   curl -L -o ~/.local/share/lumen/models/ggml-small.bin ...
```

### Teste 15: Sem Dispositivo de Áudio

1. **Configurar dispositivo inválido:**
   ```yaml
   audio:
     device: "dispositivo_inexistente"
   ```

2. **Tentar gravar**

**Resultado esperado:**
```
Falha ao iniciar gravação: Device not found
```

### Teste 16: IA Indisponível

1. **Configurar IA sem servidor:**
   ```yaml
   ai:
     provider: "ollama"
     auto_formatting: true
     ollama:
       url: "http://localhost:99999"  # Porta inválida
   ```

2. **Fazer transcrição**

**Resultado esperado:**
```
AI falhou internamente (connection refused), injetando texto bruto como fallback
```

---

## 📊 Verificar Logs

### Logs em Tempo Real
```bash
# Nível debug
RUST_LOG=debug lumen

# Nível trace (muito verboso)
RUST_LOG=trace lumen

# Apenas módulo específico
RUST_LOG=lumen::transcription=debug lumen
```

### Logs Salvos
```bash
# Configurar log em arquivo
# ~/.config/lumen/config.yaml
logging:
  level: "debug"
  file: "/tmp/lumen.log"

# Ver logs
tail -f /tmp/lumen.log
```

---

## ✅ Checklist de Testes

- [ ] Dispositivos de áudio listados corretamente
- [ ] Configuração carregada sem erros
- [ ] Dashboard abre no navegador
- [ ] Gravação básica funciona
- [ ] VAD detecta fim de fala automaticamente
- [ ] Fillers são removidos
- [ ] Comando "envie" funciona
- [ ] Comando "apague" funciona
- [ ] Comando "torne profissional" funciona (com IA)
- [ ] Dicionário customizado aplica substituições
- [ ] Snippets expandem corretamente
- [ ] Formatação IA funciona (se configurada)
- [ ] WebSocket atualiza dashboard em tempo real
- [ ] Snippets podem ser gerenciados via dashboard
- [ ] Histórico salva transcrições no SQLite
- [ ] Erros são tratados graciosamente
- [ ] Logs são gerados corretamente

---

## 🎯 Resultado Esperado

Após todos os testes, o Lumen deve:

✅ Capturar áudio via hotkey  
✅ Detectar fim de fala automaticamente (VAD)  
✅ Transcrever com Whisper  
✅ Remover fillers  
✅ Aplicar dicionário e snippets  
✅ Formatar com IA (opcional)  
✅ Injetar texto em qualquer aplicativo  
✅ Pressionar Enter automaticamente (opcional)  
✅ Responder a comandos de voz  
✅ Atualizar dashboard em tempo real  
✅ Salvar histórico no SQLite  

**Status:** 🎉 Todos os testes devem passar!
