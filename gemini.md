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
