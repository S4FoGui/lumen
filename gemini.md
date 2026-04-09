# Project Definition: Lumen Waveform Synchronization

## Data Schema
### WebSocket Audio Payload
```json
{
  "type": "AudioLevel",
  "data": {
    "rms": 0.123
  }
}
```

## Protocol Constants
- **Sampling Frequency**: 30ms (33Hz) for UI updates.
- **Normalization Factor**: 15x multiplier for RMS values in UI.

## Behavioral Constraints
- Waveform must respond in real-time with <50ms end-to-end latency.
- Styling: Neon Lime (#a3e635) with glow effects.
- Pattern: Multi-layered sine waves (Siri/Claude style).

## Maintenance Log - 09/04/2026
- **Manual Mode Implementation**: Forçado `always_listening = false` para operação via Hotkey (2x Enter) apenas.
- **UI Modal Behavior**: Overlay configurado para fechar completamente (opacidade 0 + `visible: false`) após processamento ou idle.
- **Audio Buffer Stability**: Fixado em 480 samples para compatibilidade ótima com RNNoise.
- **Wayland Overlay**: Ativado via feature `wayland-overlay` usando `gtk4-layer-shell`.
