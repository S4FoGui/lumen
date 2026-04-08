#!/usr/bin/env python3
import os
import struct
import fcntl
import time
import sys

# Constantes do kernel Linux (uinput.h / input-event-codes.h)
EV_KEY = 0x01
EV_SYN = 0x00
SYN_REPORT = 0x00

UI_SET_EVBIT = 0x40045564
UI_SET_KEYBIT = 0x40045565
UI_DEV_CREATE = 0x5501
UI_DEV_DESTROY = 0x5502

# Keycodes
KEY_LEFTCTRL = 29
KEY_V = 47
KEY_ENTER = 28

# Estrutura uinput_user_dev (simplificada para o kernel moderno)
UINPUT_MAX_NAME_SIZE = 80
class uinput_user_dev:
    def __init__(self, name):
        self.name = name.encode('utf-8')[:UINPUT_MAX_NAME_SIZE].ljust(UINPUT_MAX_NAME_SIZE, b'\0')
        self.id_bustype = 0x03 # BUS_USB
        self.id_vendor = 0x1234
        self.id_product = 0x5678
        self.id_version = 0x1
        self.ff_effects_max = 0
        self.absmax = [0] * 64
        self.absmin = [0] * 64
        self.absfuzz = [0] * 64
        self.absflat = [0] * 64

    def pack(self):
        # Struct format: 80s H H H H I 64i 64i 64i 64i
        return struct.pack('80sHHHH I 64i 64i 64i 64i',
                          self.name,
                          self.id_bustype,
                          self.id_vendor,
                          self.id_product,
                          self.id_version,
                          self.ff_effects_max,
                          *self.absmax, *self.absmin, *self.absfuzz, *self.absflat)

def send_event(fd, type, code, value):
    # struct input_event: long, long, unsigned short, unsigned short, unsigned int
    # No Linux 64-bit moderno, os tempos (long, long) são secundários aqui.
    # Usaremos struct.pack('llHHI', 0, 0, type, code, value)
    now = time.time()
    sec = int(now)
    usec = int((now - sec) * 1000000)
    event = struct.pack('llHHi', sec, usec, type, code, value)
    os.write(fd, event)

def main():
    if len(sys.argv) < 2:
        print("Uso: injector.py [paste|enter]")
        sys.exit(1)

    cmd = sys.argv[1]

    try:
        # 1. Abrir uinput
        fd = os.open('/dev/uinput', os.O_WRONLY | os.O_NONBLOCK)

        # 2. Configurar o dispositivo como teclado
        fcntl.ioctl(fd, UI_SET_EVBIT, EV_KEY)
        fcntl.ioctl(fd, UI_SET_KEYBIT, KEY_LEFTCTRL)
        fcntl.ioctl(fd, UI_SET_KEYBIT, KEY_V)
        fcntl.ioctl(fd, UI_SET_KEYBIT, KEY_ENTER)

        # 3. Criar o dispositivo
        dev = uinput_user_dev("Lumen Virtual Keyboard")
        os.write(fd, dev.pack())
        fcntl.ioctl(fd, UI_DEV_CREATE)

        # Aguardar o kernel registrar o dispositivo (muito importante!)
        time.sleep(0.5)

        if cmd == "paste":
            # Ctrl DOWN
            send_event(fd, EV_KEY, KEY_LEFTCTRL, 1)
            send_event(fd, EV_SYN, SYN_REPORT, 0)
            time.sleep(0.05)
            
            # V DOWN
            send_event(fd, EV_KEY, KEY_V, 1)
            send_event(fd, EV_SYN, SYN_REPORT, 0)
            time.sleep(0.05)

            # V UP
            send_event(fd, EV_KEY, KEY_V, 0)
            send_event(fd, EV_SYN, SYN_REPORT, 0)
            time.sleep(0.05)

            # Ctrl UP
            send_event(fd, EV_KEY, KEY_LEFTCTRL, 0)
            send_event(fd, EV_SYN, SYN_REPORT, 0)

        elif cmd == "enter":
            send_event(fd, EV_KEY, KEY_ENTER, 1)
            send_event(fd, EV_SYN, SYN_REPORT, 0)
            time.sleep(0.05)
            send_event(fd, EV_KEY, KEY_ENTER, 0)
            send_event(fd, EV_SYN, SYN_REPORT, 0)

        # Aguardar os eventos serem processados antes de destruir
        time.sleep(0.2)
        fcntl.ioctl(fd, UI_DEV_DESTROY)
        os.close(fd)
        
        print(f"Sucesso: comando '{cmd}' enviado via uinput.")

    except Exception as e:
        print(f"Erro: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
