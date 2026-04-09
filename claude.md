# Project Constitution: Lumen - Audio Waveform Synchronization

## Data Schemas

### Audio Level Payload
```json
{
  "type": "audio_level",
  "data": {
    "peak": 0.5,
    "rms": 0.2
  }
}
```

## Behavioral Rules
- **Offline First**: All audio processing must happen locally.
- **Privacy**: No audio data should leave the local system unless explicitly requested.
- **Latency**: Audio-to-visual latency must be minimized for a responsive feel.

## Architectural Invariants
- Backend (Rust) handles raw audio capture and calculation of levels.
- Frontend (React/Vite) receives levels via WebSocket and animates the Waveform component.
- The 3-layer architecture (Architecture, Navigation, Tools) must be respected for automation tasks.
