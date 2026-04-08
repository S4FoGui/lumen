#!/usr/bin/env bash
# ============================================================
# Glaido — Script de Instalação para Linux (Arch & Debian)
# ============================================================
set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}"
echo "   ██████╗ ██╗      █████╗ ██╗██████╗  ██████╗ "
echo "  ██╔════╝ ██║     ██╔══██╗██║██╔══██╗██╔═══██╗"
echo "  ██║  ███╗██║     ███████║██║██║  ██║██║   ██║"
echo "  ██║   ██║██║     ██╔══██║██║██║  ██║██║   ██║"
echo "  ╚██████╔╝███████╗██║  ██║██║██████╔╝╚██████╔╝"
echo "   ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝╚═════╝  ╚═════╝"
echo -e "${NC}"
echo "Instalador do Glaido — Voice Productivity for Linux"
echo "===================================================="
echo ""

# Detectar distro
detect_distro() {
    if command -v pacman &>/dev/null; then
        echo "arch"
    elif command -v apt &>/dev/null; then
        echo "debian"
    else
        echo "unknown"
    fi
}

DISTRO=$(detect_distro)
echo -e "${CYAN}[INFO]${NC} Distro detectada: ${DISTRO}"

# Instalar dependências do sistema
install_deps() {
    echo -e "${YELLOW}[DEPS]${NC} Instalando dependências do sistema..."

    if [ "$DISTRO" = "arch" ]; then
        sudo pacman -S --needed --noconfirm \
            base-devel cmake \
            gtk4 \
            alsa-lib \
            libnotify \
            xdotool \
            wtype \
            xclip \
            wl-clipboard
    elif [ "$DISTRO" = "debian" ]; then
        sudo apt update
        sudo apt install -y \
            build-essential cmake pkg-config \
            libgtk-4-dev \
            libasound2-dev \
            libnotify-bin \
            xdotool \
            xclip \
            wl-clipboard
    else
        echo -e "${RED}[ERRO]${NC} Distro não suportada. Instale manualmente: cmake, gtk4, alsa-lib"
        exit 1
    fi

    echo -e "${GREEN}[OK]${NC} Dependências instaladas"
}

# Verificar Rust
check_rust() {
    if ! command -v cargo &>/dev/null; then
        echo -e "${YELLOW}[RUST]${NC} Rust não encontrado. Instalando via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    echo -e "${GREEN}[OK]${NC} Rust: $(rustc --version)"
}

# Compilar
build_glaido() {
    echo -e "${CYAN}[BUILD]${NC} Compilando Glaido (release)..."
    cargo build --release
    echo -e "${GREEN}[OK]${NC} Compilação concluída"
}

# Instalar binário e recursos
install_glaido() {
    echo -e "${CYAN}[INSTALL]${NC} Instalando..."

    # Binário
    sudo cp target/release/glaido /usr/local/bin/glaido
    sudo chmod +x /usr/local/bin/glaido

    # Configuração padrão
    mkdir -p "$HOME/.config/glaido"
    if [ ! -f "$HOME/.config/glaido/config.yaml" ]; then
        cp config/default.yaml "$HOME/.config/glaido/config.yaml"
        echo -e "${GREEN}[OK]${NC} Config copiada para ~/.config/glaido/config.yaml"
    fi

    # Diretório para modelos
    mkdir -p "$HOME/.local/share/glaido/models"

    # Arquivos estáticos (dashboard web)
    sudo mkdir -p /usr/local/share/glaido/static/css
    sudo mkdir -p /usr/local/share/glaido/static/js
    sudo cp src/ui/web/static/index.html /usr/local/share/glaido/static/
    sudo cp src/ui/web/static/css/style.css /usr/local/share/glaido/static/css/
    sudo cp src/ui/web/static/js/app.js /usr/local/share/glaido/static/js/

    # Desktop entry
    sudo tee /usr/share/applications/glaido.desktop > /dev/null << 'EOF'
[Desktop Entry]
Name=Glaido
Comment=Voice Productivity Ecosystem for Linux
Exec=/usr/local/bin/glaido
Icon=audio-input-microphone
Terminal=false
Type=Application
Categories=Utility;Accessibility;
Keywords=voice;dictation;speech;transcription;
EOF

    echo -e "${GREEN}[OK]${NC} Glaido instalado em /usr/local/bin/glaido"
}

# Download do modelo Whisper
download_model() {
    local model_path="$HOME/.local/share/glaido/models/ggml-base.bin"
    if [ -f "$model_path" ]; then
        echo -e "${GREEN}[OK]${NC} Modelo Whisper já existe"
        return
    fi

    echo -e "${YELLOW}[MODEL]${NC} Deseja baixar o modelo Whisper base (~142MB)? [Y/n]"
    read -r response
    if [[ "$response" =~ ^[Nn]$ ]]; then
        echo -e "${YELLOW}[SKIP]${NC} Modelo não baixado. Execute mais tarde:"
        echo "  curl -L -o $model_path https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
        return
    fi

    echo -e "${CYAN}[DOWNLOAD]${NC} Baixando modelo Whisper base..."
    curl -L -o "$model_path" \
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
    echo -e "${GREEN}[OK]${NC} Modelo baixado"
}

# --- Main ---
echo ""
install_deps
check_rust
build_glaido
install_glaido
download_model

echo ""
echo -e "${GREEN}✅ Glaido instalado com sucesso!${NC}"
echo ""
echo "Uso:"
echo "  glaido              — Iniciar"
echo "  glaido config       — Ver caminhos de configuração"
echo "  glaido dashboard    — Abrir dashboard web"
echo "  glaido devices      — Listar dispositivos de áudio"
echo ""
echo "Atalhos padrão:"
echo "  Ctrl+Shift+Space    — Gravar / Parar"
echo "  Ctrl+Shift+L        — Modo Lightning"
echo "  Ctrl+Shift+D        — Abrir Dashboard"
echo ""
