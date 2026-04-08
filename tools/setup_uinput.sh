#!/bin/bash
# Configuração do uinput para permitir injeção de teclas sem sudo
# Deve ser executado uma única vez.

if [ "$EUID" -ne 0 ]; then
  echo "Por favor, execute como root (sudo)"
  exit
fi

# Adiciona o usuário atual ao grupo input
usermod -aG input $SUDO_USER
echo "Usuário $SUDO_USER adicionado ao grupo 'input'."

# Cria regra udev para o /dev/uinput
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' > /etc/udev/rules.d/99-uinput.rules
echo "Regra udev criada para /dev/uinput."

# Recarrega regras
udevadm control --reload-rules
udevadm trigger

echo "Configuração concluída. Por favor, faça logout e login novamente para aplicar a mudança de grupo."
