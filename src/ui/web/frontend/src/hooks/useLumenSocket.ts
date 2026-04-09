import { useState, useEffect, useCallback, useRef } from 'react';

// === Tipagens baseadas no LumenEvent do Rust ===

export type TranscriptionRecord = {
  id: string;
  timestamp: string;
  raw_text: string;
  processed_text: string;
  word_count: number;
  processing_time_ms: number;
  ai_used: boolean;
  auto_sent: boolean;
};

export type VoiceCommand = {
  command_type: string;
  command: string;
};

export type SystemStatus = {
  version: string;
  is_recording: boolean;
  uptime_seconds: number;
  total_transcriptions: number;
  total_words: number;
};

export type LumenEvent =
  | { type: 'connected'; data: SystemStatus }
  | { type: 'RecordingStarted' }
  | { type: 'RecordingStopped' }
  | { type: 'AudioLevel'; data: { rms: number } }
  | { type: 'TranscriptionComplete'; data: TranscriptionRecord }
  | { type: 'VoiceCommandDetected'; data: VoiceCommand }
  | { type: 'DictionaryUpdated' }
  | { type: 'SnippetsUpdated' }
  | { type: 'ConfigChanged' }
  | { type: 'Error'; data: { message: string } }
  | { type: 'warning'; data: { message: string } };

// === Hook de Integração WS ===

export function useLumenSocket() {
  const [status, setStatus] = useState<SystemStatus>({
    version: '...',
    is_recording: false,
    uptime_seconds: 0,
    total_transcriptions: 0,
    total_words: 0,
  });
  
  const [rms, setRms] = useState(0);
  const [isConnected, setIsConnected] = useState(false);
  const [lastTranscription, setLastTranscription] = useState<TranscriptionRecord | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);
  const socketRef = useRef<WebSocket | null>(null);

  const connect = useCallback(() => {
    // Definir a porta (usamos URL absoluta em dev e relativa em prod via API proxy)
    const host = window.location.hostname === 'localhost' && window.location.port === '5173' 
        ? 'localhost:8484' // ambiente de dev Vite
        : window.location.host; // ambiente de prod Axum
        
    const ws = new WebSocket(`ws://${host}/ws`);

    ws.onopen = () => {
      console.log('✅ Conectado ao Lumen Event Bus');
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      try {
        const payload: LumenEvent = JSON.parse(event.data);
        
        switch (payload.type) {
          case 'connected':
            setStatus(payload.data);
            break;
            
          case 'RecordingStarted':
            setStatus(prev => ({ ...prev, is_recording: true }));
            break;
            
          case 'RecordingStopped':
            setStatus(prev => ({ ...prev, is_recording: false }));
            setRms(0); // reset visualizer
            break;
            
          case 'AudioLevel':
            setRms(payload.data.rms);
            break;
            
          case 'TranscriptionComplete':
            setLastTranscription(payload.data);
            // Increment counters optimistically
            setStatus(prev => ({
              ...prev,
              total_transcriptions: prev.total_transcriptions + 1,
              total_words: prev.total_words + (payload.data.word_count || 0)
            }));
            break;

          case 'Error':
            console.error('Lumen Error:', payload.data.message);
            setLastError(payload.data.message);
            setTimeout(() => setLastError(null), 6000);
            break;
            
          case 'warning':
            console.warn('Lumen Warning:', payload.data.message);
            break;
        }
      } catch (err) {
        console.error('Falha ao processar evento WS:', err);
      }
    };

    ws.onclose = () => {
      console.log('❌ Desconectado do Lumen Event Bus. Tentando reconectar...');
      setIsConnected(false);
      setStatus(prev => ({ ...prev, is_recording: false }));
      
      // Auto-reconnect após 3s
      setTimeout(connect, 3000);
    };

    ws.onerror = (err) => {
      console.error('WebSocket error:', err);
      ws.close();
    };

    socketRef.current = ws;
  }, []);

  // Ref para rastrear se o uptime veio do servidor recentemente
  const lastServerUptimeRef = useRef<number>(0);
  const lastUptimeUpdateRef = useRef<number>(Date.now());

  useEffect(() => {
    connect();

    // Atualizar o uptime a cada segundo APENAS se não recebemos do servidor recentemente
    const uptimeInterval = setInterval(() => {
        const now = Date.now();
        const timeSinceLastServerUpdate = now - lastUptimeUpdateRef.current;

        // Só incrementa localmente se não recebemos do servidor nos últimos 2 segundos
        if (timeSinceLastServerUpdate > 2000) {
            setStatus(prev => ({
                ...prev,
                uptime_seconds: prev.uptime_seconds > 0 ? prev.uptime_seconds + 1 : 0
            }));
        }
    }, 1000);

    return () => {
      if (socketRef.current) {
        // Previne loop de reconnect no unmount
        socketRef.current.onclose = null;
        socketRef.current.close();
      }
      clearInterval(uptimeInterval);
    };
  }, [connect]);

  // Atualizar referência quando receber uptime do servidor
  useEffect(() => {
    lastServerUptimeRef.current = status.uptime_seconds;
    lastUptimeUpdateRef.current = Date.now();
  }, [status.uptime_seconds]);

  return {
    status,
    rms,
    isConnected,
    lastTranscription,
    lastError,
  };
}
