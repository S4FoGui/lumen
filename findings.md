# 🔍 Findings & Discoveries

## [2026-04-09] Audio & UI Optimization
- **RNNoise (nnnoiseless)**: A amostragem de 48kHz para supressão de ruído é mandatória para evitar artefatos metálicos. O buffer de 480 samples provou ser o ponto ideal de latência/qualidade.
- **Whisper CPU**: O modelo `medium` é inviável para CPUs padrão Linux em tempo real (~80s de atraso). O modelo `base` com `lightning_mode` reduz o tempo para <3s sem perda significativa em PT-BR.
- **GTK Opacity**: Alterar a opacidade via ticker callback requer casting de `f32` para `f64` e verificação de delta para evitar flickering em monitores de alta taxa de atualização.
- **Overlay State**: O auto-dismiss deve ser bloqueado por uma flag atômica de gravação, caso contrário, o visor some enquanto o usuário ainda está falando em silêncio (pausas dramáticas).
