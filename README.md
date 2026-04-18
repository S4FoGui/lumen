# 🎙️ Lumen

**Ecossistema de Produtividade por Voz para Linux**

Transforme sua voz em texto com transcrição Whisper local, limpeza automática de vícios de fala, formatação inteligente via IA, e injeção direta em qualquer aplicativo.

> Zero Python. Zero pip. **Um binário. Pronto.**

---

## ✨ Recursos

| Nível | Recurso | Descrição |
|-------|---------|-----------|
| 1 | **Substituição de Digitação** | Dite e o texto aparece em qualquer app |
| 1 | **Limpeza de Áudio** | Remove "humm", "ééé", fillers automáticos |
| 2 | **Dicionário Customizado** | Ensine termos técnicos ao Lumen |
| 2 | **Snippets de Voz** | `/ola` → bloco de texto completo |
| 3 | **Modo Lightning** | Transcrição bruta em milissegundos |
| 2 | **Comandos de Voz** | Detecção de comandos como "envie", "apague", "nova linha" |
| 3 | **Modo Sempre Escutando** | O Lumen escuta continuamente e processa apenas com a palavra de ativação |
| 3 | **Formatação IA** | Ollama, OpenAI ou Gemini formatam seu texto |

## 🏗️ Stack

- **Rust** — Binário único estático (~20MB)
- **whisper-rs** — Transcrição local via whisper.cpp
- **cpal** — Captura de áudio (ALSA/PulseAudio/PipeWire)
- **axum** — Dashboard web em localhost:8484
- **global-hotkey** — Atalhos globais X11 + Wayland

## 📦 Instalação

### Arch Linux (AUR)
```bash
yay -S lumen
```

### Debian/Ubuntu
```bash
sudo dpkg -i lumen-1.0.0-amd64.deb
```

### Do código fonte
```bash
git clone https://github.com/guilherme/lumen.git
cd lumen
chmod +x scripts/install.sh
./scripts/install.sh
```

### Download do modelo Whisper
O Lumen requer um modelo Whisper em formato GGML. Recomendamos o modelo `small` para um bom equilíbrio entre velocidade e precisão.

```bash
mkdir -p ~/.local/share/lumen/models
# Modelo Small (Recomendado)
curl -L -o ~/.local/share/lumen/models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
```

## 🚀 Uso

```bash
lumen              # Iniciar o Lumen
lumen config       # Ver caminho da configuração
lumen dashboard    # Abrir dashboard no navegador
lumen devices      # Listar dispositivos de áudio
```

### Atalhos Padrão

| Atalho | Ação |
|--------|------|
| `Enter Enter` (double-tap) | Gravar / Parar (pressione 2x rapidamente) |
| `Ctrl+Shift+L` | Modo Lightning |
| `Ctrl+Shift+D` | Abrir Dashboard |

## ⚙️ Configuração

O arquivo de configuração fica em `~/.config/lumen/config.yaml`:

```yaml
transcription:
  language: "pt"
  lightning_mode: false

ai:
  provider: "ollama"  # ollama, openai, gemini, disabled
  ollama:
    url: "http://localhost:11434"
    model: "llama3.2"

snippets:
  entries:
    "/ola": "Olá! Tudo bem?"
    "/email": "Atenciosamente,\nGuilherme"
```

## 🌐 Dashboard

Acesse `http://localhost:8484` para o painel de controle web com:
- Status em tempo real
- Gerenciamento de snippets
- Dicionário customizado
- Estatísticas de uso
- Configuração visual

## 📋 Dependências do Sistema

Para rodar o Lumen compilado, seu sistema precisará de ferramentas de áudio (ALSA), interface (GTK4) e área de transferência.

### Arch Linux
**Para Executar:**
```bash
sudo pacman -S gtk4 alsa-lib xclip wl-clipboard
```
**Para Compilar (Build):**
```bash
sudo pacman -S base-devel pkgconf
```

### Debian/Ubuntu
**Para Executar:**
```bash
sudo apt install libgtk-4-1 libasound2 xclip wl-clipboard
```
**Para Compilar (Build):**
```bash
sudo apt install build-essential pkg-config libgtk-4-dev libasound2-dev
```

### ⚠️ Permissão Crucial (Para Injeção de Texto Automática)
O Lumen usa o moderno injetor kernel-level (`uinput`) para garantir que o texto seja colado fluidamente debaixo dos panos, furando bloqueios do Wayland nativamente.

Para que seu teclado fantasma funcione sem precisar usar `sudo`, adicione seu usuário ao grupo `input` e recarregue as regras:

```bash
sudo usermod -aG input $USER
# (Obrigatório reiniciar o computador ou fazer logout para a permissão do grupo funcionar)
```

MIT License — veja [LICENSE](LICENSE).

## 🔄 Changelog

### v1.0.1
- **Always Listening Mode**: Novo modo que mantém o Lumen escutando continuamente e processa apenas quando detectar a palavra de ativação
- **Wake Word Detection**: Implementação de detecção de palavra de ativação no modo sempre escutando
- **Fix recursão infinita**: Corrigido problema de função recursiva assíncrona que causava erros de compilação
- **Melhorias no Dashboard**: Correções para garantir que os arquivos CSS e JS sejam carregados corretamente
- **Atualização de Hotkeys**: Referências atualizadas para usar "Enter 2x" em vez de combinações antigas
- **Comandos de Voz**: Adicionada detecção de comandos de voz como "envie", "apague", "nova linha"
