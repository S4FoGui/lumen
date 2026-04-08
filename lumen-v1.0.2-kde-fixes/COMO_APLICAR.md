# Como aplicar os patches do Lumen v1.0.2

## Estrutura dos arquivos

```
lumen-fixes/
├── fix_kde.sh                          ← Script de diagnóstico/instalação
├── config/
│   └── default.yaml                    ← Config otimizada para KDE
├── src/
│   ├── audio/vad.rs                    ← VAD mais rápido + min speech duration
│   ├── transcription/engine.rs         ← BeamSearch, filtro de alucinações
│   ├── text/injector.rs                ← Clipboard para KDE/Wayland
│   └── ui/web/frontend/src/
│       └── components/tabs/
│           └── HistoryTab.tsx          ← Botão "Limpar tudo" funcional
└── COMO_APLICAR.md
```

## Passos

### 1. Executar o script de diagnóstico
```bash
chmod +x fix_kde.sh
./fix_kde.sh
```

### 2. Copiar os arquivos Rust para o projeto
```bash
cp config/default.yaml /caminho/para/lumen/config/default.yaml
cp src/audio/vad.rs /caminho/para/lumen/src/audio/vad.rs
cp src/transcription/engine.rs /caminho/para/lumen/src/transcription/engine.rs
cp src/text/injector.rs /caminho/para/lumen/src/text/injector.rs
```

### 3. Copiar o componente React
```bash
cp src/ui/web/frontend/src/components/tabs/HistoryTab.tsx \
   /caminho/para/lumen/src/ui/web/frontend/src/components/tabs/HistoryTab.tsx
```

### 4. Recompilar frontend
```bash
cd /caminho/para/lumen/src/ui/web/frontend
npm run build
```

### 5. Recompilar o Rust
```bash
cd /caminho/para/lumen
cargo build --release
```

### 6. Aplicar configuração para KDE (se não foi feito pelo script)
Edite `~/.config/lumen/config.yaml`:
```yaml
transcription:
  silence_threshold_ms: 1200   # Era 4000 — reduz tempo de espera
text_injection:
  method: "clipboard"          # Mais confiável no KDE/Wayland
  delay_ms: 150
```

## Melhorias da versão

| Problema | Causa | Fix |
|---|---|---|
| Barra fica muito tempo | `silence_threshold_ms: 4000` | Reduzido para `1200ms` |
| "pizza" → "pixssa" | Greedy com best_of=1 | BeamSearch(5) = mais preciso |
| Não escreve na caixa | xdotool não funciona no KDE Wayland | Clipboard + Ctrl+V |
| Timeline sem limpar | Botão sem handler | Implementado com confirmação |

## Dica: modelo medium para melhor PT-BR

O modelo `small` tem acurácia limitada para português. O `medium` é MUITO melhor:

```bash
curl -L -o ~/.local/share/lumen/models/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

Depois configure em `~/.config/lumen/config.yaml`:
```yaml
transcription:
  model_path: "~/.local/share/lumen/models/ggml-medium.bin"
```
