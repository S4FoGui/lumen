#!/usr/bin/env bash
# Glaido — Uninstall Script
set -euo pipefail

echo "Desinstalando Glaido..."

sudo rm -f /usr/local/bin/glaido
sudo rm -rf /usr/local/share/glaido
sudo rm -f /usr/share/applications/glaido.desktop

echo ""
echo "Deseja remover a configuração e modelos do usuário? [y/N]"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    rm -rf "$HOME/.config/glaido"
    rm -rf "$HOME/.local/share/glaido"
    echo "Dados do usuário removidos."
fi

echo "✅ Glaido desinstalado."
