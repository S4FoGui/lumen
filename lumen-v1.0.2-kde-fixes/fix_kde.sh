#!/usr/bin/env bash
# ============================================================
# Lumen v1.0.2 — Diagnóstico e Fix para KDE/Plasma/Debian
# ============================================================
set -euo pipefail

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "  ██╗     ██╗   ██╗███╗   ███╗███████╗███╗   ██╗"
echo "  ██║     ██║   ██║████╗ ████║██╔════╝████╗  ██║"
echo "  ██║     ██║   ██║██╔████╔██║█████╗  ██╔██╗ ██║"
echo "  ██║     ██║   ██║██║╚██╔╝██║██╔══╝  ██║╚██╗██║"
echo "  ███████╗╚██████╔╝██║ ╚═╝ ██║███████╗██║ ╚████║"
echo "  ╚══════╝ ╚═════╝ ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝"
echo -e "${NC}"
echo "  Diagnóstico para KDE/Plasma/Debian — v1.0.2"
echo ""

# ── 1. Detectar ambiente ──────────────────────────────────────
echo -e "${CYAN}[INFO]${NC} Detectando ambiente..."
SESSION="${XDG_SESSION_TYPE:-desconhecido}"
DESKTOP="${XDG_CURRENT_DESKTOP:-desconhecido}"
echo "  Sessão: $SESSION"
echo "  Desktop: $DESKTOP"
echo ""

# ── 2. Verificar ferramentas de injeção ───────────────────────
echo -e "${CYAN}[TOOLS]${NC} Verificando ferramentas de injeção de texto..."

check_tool() {
  if command -v "$1" &>/dev/null; then
    echo -e "  ${GREEN}✅ $1${NC} — encontrado em $(which $1)"
  else
    echo -e "  ${RED}❌ $1${NC} — não encontrado"
    return 1
  fi
}

MISSING_TOOLS=()

check_tool "xdotool" || MISSING_TOOLS+=("xdotool")
check_tool "wl-copy"  || MISSING_TOOLS+=("wl-clipboard")
check_tool "wtype"    || true  # Opcional
check_tool "ydotool"  || true  # Opcional mas útil

echo ""

# ── 3. Instalar ferramentas faltantes ─────────────────────────
if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
  echo -e "${YELLOW}[INSTALL]${NC} Instalando ferramentas faltantes: ${MISSING_TOOLS[*]}"
  sudo apt update -qq
  for tool in "${MISSING_TOOLS[@]}"; do
    echo -e "  Instalando ${tool}..."
    sudo apt install -y "$tool" || echo -e "  ${RED}Falha ao instalar $tool${NC}"
  done
  echo ""
fi

# ── 4. Verificar modelo Whisper ───────────────────────────────
echo -e "${CYAN}[MODEL]${NC} Verificando modelo Whisper..."
MODEL_DIR="$HOME/.local/share/lumen/models"
mkdir -p "$MODEL_DIR"

if ls "$MODEL_DIR"/*.bin 2>/dev/null | head -1 | grep -q ".bin"; then
  echo -e "  ${GREEN}✅ Modelo encontrado:${NC}"
  ls -lh "$MODEL_DIR"/*.bin 2>/dev/null
else
  echo -e "  ${RED}❌ Nenhum modelo encontrado em $MODEL_DIR${NC}"
  echo ""
  echo -e "  ${YELLOW}[RECOMENDADO]${NC} Baixar modelo medium para melhor acurácia em PT-BR:"
  echo ""
  echo "  # Modelo small (500MB) — equilibrado:"
  echo "  curl -L -o $MODEL_DIR/ggml-small.bin \\"
  echo "    https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
  echo ""
  echo "  # Modelo medium (1.5GB) — MELHOR acurácia em português:"
  echo "  curl -L -o $MODEL_DIR/ggml-medium.bin \\"
  echo "    https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
  echo ""
  
  read -p "  Baixar modelo small agora? [s/N] " -r reply
  if [[ "$reply" =~ ^[Ss]$ ]]; then
    echo -e "  ${CYAN}Baixando...${NC}"
    curl -L --progress-bar \
      -o "$MODEL_DIR/ggml-small.bin" \
      "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
    echo -e "  ${GREEN}✅ Modelo baixado!${NC}"
  fi
fi

echo ""

# ── 5. Aplicar configuração otimizada para KDE ────────────────
echo -e "${CYAN}[CONFIG]${NC} Aplicando configuração otimizada para KDE/Debian..."
CONFIG_DIR="$HOME/.config/lumen"
mkdir -p "$CONFIG_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -f "$SCRIPT_DIR/default.yaml" ]; then
  cp "$SCRIPT_DIR/default.yaml" "$CONFIG_DIR/config.yaml"
  echo -e "  ${GREEN}✅ Configuração KDE aplicada${NC}"
else
  echo -e "  ${YELLOW}⚠️  default.yaml não encontrado ao lado do script${NC}"
  echo -e "  Ajustando config existente manualmente..."
  
  # Ajustar silence_threshold_ms se config existir
  if [ -f "$CONFIG_DIR/config.yaml" ]; then
    sed -i 's/silence_threshold_ms: [0-9]*/silence_threshold_ms: 1200/' "$CONFIG_DIR/config.yaml"
    sed -i 's/method: "auto"/method: "clipboard"/' "$CONFIG_DIR/config.yaml"
    sed -i 's/method: "x11"/method: "clipboard"/' "$CONFIG_DIR/config.yaml"
    sed -i 's/method: "wayland"/method: "clipboard"/' "$CONFIG_DIR/config.yaml"
    echo -e "  ${GREEN}✅ Config ajustado (silence=1200ms, method=clipboard)${NC}"
  fi
fi

echo ""

# ── 6. Teste rápido de clipboard ──────────────────────────────
echo -e "${CYAN}[TEST]${NC} Testando injeção via clipboard..."
TEST_TEXT="Lumen KDE test OK"

if command -v wl-copy &>/dev/null; then
  echo "$TEST_TEXT" | wl-copy
  echo -e "  ${GREEN}✅ wl-copy funcionando${NC}"
  # Limpar clipboard
  echo "" | wl-copy
elif command -v xclip &>/dev/null; then
  echo "$TEST_TEXT" | xclip -selection clipboard
  echo -e "  ${GREEN}✅ xclip funcionando${NC}"
else
  echo -e "  ${YELLOW}⚠️  Nenhum clipboard tool funcionando${NC}"
fi

echo ""

# ── 7. Resumo ─────────────────────────────────────────────────
echo -e "${GREEN}══════════════════════════════════════════${NC}"
echo -e "${GREEN}  Diagnóstico concluído!${NC}"
echo -e "${GREEN}══════════════════════════════════════════${NC}"
echo ""
echo "  Problemas corrigidos nesta versão (v1.0.2):"
echo ""
echo "  1. ⏱️  VAD mais rápido — silence_threshold: 1200ms (era 4000ms)"
echo "     → A barra de escuta some muito mais cedo"
echo ""
echo "  2. 🎯 Transcrição mais precisa — BeamSearch(5) ao invés de Greedy"
echo "     → 'pizza planet' não vira 'pixssa' mais"
echo ""
echo "  3. 📋 Injeção via clipboard — funciona em qualquer app KDE"
echo "     → Usa wl-copy + Ctrl+V automaticamente"
echo ""
echo "  4. 🧹 Timeline com botão 'Limpar tudo' funcional"
echo "     → Duplo clique para confirmar (anti-acidente)"
echo ""
echo "  Recompile o Lumen para aplicar todas as melhorias:"
echo ""
echo "    cargo build --release"
echo ""
echo -e "${YELLOW}  Dica extra para melhor acurácia em PT-BR:${NC}"
echo "  Use o modelo 'medium' (~1.5GB) — muito melhor que o 'small':"
echo "  curl -L -o ~/.local/share/lumen/models/ggml-medium.bin \\"
echo "    https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
echo ""
echo "  E configure em ~/.config/lumen/config.yaml:"
echo "    model_path: \"$HOME/.local/share/lumen/models/ggml-medium.bin\""
echo ""
