# Log de Mudanças e Correções - 09/04/2026

## Sessão 2 (Tarde/Noite) - Bug Fixes e Modo Manual

### 4. Correção de Erros Lógicos Críticos
- **Arquivos:** `src/main.rs`, `src/audio/capture.rs`
- **Ações:**
    - Adicionado debounce compartilhado entre hotkey e tray (evita toggles duplos)
    - Verificação de estado de gravação no Always Listening (previene race condition)
    - Verificação de texto vazio no modo normal (consistência com Always Listening)
    - Correção do tipo de retorno `stop()` de `&self` para `&mut self`
    - Limpeza de buffer de amostras no `stop()` para evitar vazamento entre sessões
- **Resultado:** Maior estabilidade no estado de gravação, prevenção de gravações duplicadas.

### 5. Modo 100% Manual (Always Listening Desativado)
- **Arquivo:** `src/main.rs`
- **Ações:**
    - Forçado `always_listening = false` permanentemente
    - Removida inicialização automática no startup
    - Ignorada reativação via mudanças de config no dashboard
    - Removido reinício assíncrono após transcrição
- **Resultado:** Lumen só ativa via hotkey (2x Enter), comportamento 100% manual como solicitado.

### 6. Comportamento Modal (Overlay Abre/Fecha)
- **Arquivo:** `src/ui/overlay.rs`
- **Ações:**
    - Auto-dismiss após 3 segundos de idle → opacidade 0
    - `HideRecording`: fecha completamente (opacidade 0, `visible = false`)
    - Quando opacidade chega a 0, janela é escondida completamente
    - `ShowRecording`: reaparece do zero quando 2x Enter é pressionado
- **Resultado:** Overlay funciona como dialog modal: aparece, processa, **fecha completamente** (não minimiza, não fica cinza).

### 7. Overlay True Always-on-Top (Wayland)
- **Arquivo:** `src/ui/overlay.rs`
- **Ação:** Configurado `gtk4-layer-shell` com `Layer::Overlay` quando feature `wayland-overlay` está ativa
- **Instrução de uso:** Compilar com `cargo run --features wayland-overlay` para overlay que nunca minimiza
- **Resultado:** No Wayland, overlay fica sempre no topo independente do foco. No X11, comportamento depende do WM.

---

## Sessão 1 (Manhã) - Melhorias Iniciais

## 1. Melhorias na Visualização (Waveform)
- **Arquivo:** `src/ui/web/frontend/src/components/Waveform.tsx`
- **Ação:** Substituído o renderizador de ondas baseado em múltiplas camadas SVG/Canvas por uma implementação baseada em `requestAnimationFrame` com interpolação linear (LERP).
- **Resultado:** Eliminação do "flicker" (cintilação) da animação, maior fluidez no movimento e estética visual mais moderna com gradientes contínuos.

## 2. Otimização na Captura de Áudio (Backend)
- **Arquivo:** `src/audio/capture.rs`
- **Ação:** 
    - Substituído `cpal::BufferSize::Default` por `cpal::BufferSize::Fixed(480)`.
    - Adicionado log de diagnóstico para monitorar latência no callback de captura.
- **Resultado:** Estabilização do buffer de áudio para o processamento via `RNNoise`, reduzindo latência e evitando "pulsos" na captura sonora.

## 3. Correção de Gerenciamento de Janela (UI/UX)
- **Arquivo:** `src/ui/overlay.rs`
- **Ação:** 
    - Adicionado `window.present()` ao evento `ShowRecording`.
    - Alterado `ApplicationFlags::NON_UNIQUE` para `ApplicationFlags::FLAGS_NONE`.
    - Implementada proteção no `Drop` através do campo `is_owner` para evitar encerramento prematuro da thread GUI por instâncias secundárias (proxy).
- **Resultado:** A interface do overlay agora mantém uma instância única, não fecha inesperadamente e traz a janela para o plano de frente (focus) corretamente ao iniciar a gravação.
