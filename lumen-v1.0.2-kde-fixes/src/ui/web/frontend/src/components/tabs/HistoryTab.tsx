import { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { FileText, Trash2, RefreshCw, AlertTriangle } from 'lucide-react';
import type { TranscriptionRecord } from '../../hooks/useLumenSocket';

export function HistoryTab() {
  const [history, setHistory] = useState<TranscriptionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [clearing, setClearing] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);

  const loadHistory = useCallback(async () => {
    setLoading(true);
    try {
      const res = await fetch('/api/transcriptions?limit=50');
      if (res.ok) {
        const data = await res.json();
        setHistory(Array.isArray(data) ? data : []);
      }
    } catch (err) {
      console.error('Falha ao carregar histórico', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const deleteRecord = async (id: string) => {
    try {
      const res = await fetch(`/api/transcriptions/${id}`, { method: 'DELETE' });
      if (res.ok) {
        setHistory(prev => prev.filter(r => r.id !== id));
      }
    } catch (err) {
      console.error('Falha ao deletar registro', err);
    }
  };

  const clearHistory = async () => {
    if (!confirmClear) {
      setConfirmClear(true);
      // Auto-cancel confirmation after 3s
      setTimeout(() => setConfirmClear(false), 3000);
      return;
    }

    setClearing(true);
    setConfirmClear(false);
    try {
      const res = await fetch('/api/history/clear', { method: 'POST' });
      if (res.ok) {
        setHistory([]);
      } else {
        console.error('Falha ao limpar histórico:', await res.text());
      }
    } catch (err) {
      console.error('Erro ao limpar histórico', err);
    } finally {
      setClearing(false);
    }
  };

  const formatDate = (isoStr: string) => {
    try {
      const d = new Date(isoStr);
      return new Intl.DateTimeFormat('pt-BR', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        day: '2-digit',
        month: '2-digit',
      }).format(d);
    } catch {
      return isoStr;
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex justify-between items-center mb-8">
        <h2 className="text-3xl font-bold tracking-tight">Transcription Timeline</h2>
        <div className="flex gap-2">
          {/* Botão Refresh */}
          <Button
            variant="outline"
            size="sm"
            onClick={loadHistory}
            disabled={loading}
            className="border-border"
          >
            <RefreshCw className={`w-4 h-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Atualizar
          </Button>

          {/* Botão Limpar com confirmação de 2 cliques */}
          {history.length > 0 && (
            <Button
              variant={confirmClear ? 'destructive' : 'outline'}
              size="sm"
              onClick={clearHistory}
              disabled={clearing}
              className={confirmClear
                ? 'bg-red-600 hover:bg-red-700 text-white animate-pulse'
                : 'border-red-500/30 text-red-400 hover:bg-red-500/10'
              }
            >
              <AlertTriangle className="w-4 h-4 mr-2" />
              {clearing
                ? 'Limpando...'
                : confirmClear
                  ? '⚠️ Confirmar? (clique de novo)'
                  : 'Limpar tudo'
              }
            </Button>
          )}
        </div>
      </div>

      <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-muted-foreground flex justify-between items-center">
            <span>Histórico Local SQLite</span>
            <span className="text-xs px-2 py-1 rounded bg-secondary border border-border">
              {history.length} registro{history.length !== 1 ? 's' : ''}
            </span>
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex justify-center p-8">
              <RefreshCw className="w-6 h-6 animate-spin opacity-50" />
            </div>
          ) : history.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-12 text-muted-foreground">
              <FileText className="w-12 h-12 mb-4 opacity-50" />
              <p>Nenhuma transcrição salva ainda.</p>
              <p className="text-xs mt-2 opacity-70">Use o Lumen para gravar e as transcrições aparecem aqui.</p>
            </div>
          ) : (
            <div className="space-y-3">
              {history.map((record) => (
                <div
                  key={record.id}
                  className="p-4 rounded-lg bg-secondary/30 border border-border/50 hover:bg-secondary/50 transition-colors group"
                >
                  <div className="flex justify-between items-start mb-2">
                    <span className="text-xs text-muted-foreground bg-background/50 px-2 py-1 rounded border border-border/50 font-mono">
                      {formatDate(record.timestamp)}
                    </span>
                    <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                      <span className="text-xs text-muted-foreground">
                        {record.processing_time_ms}ms · {record.word_count} words
                      </span>
                      {record.ai_used && (
                        <span className="text-xs text-accent px-2 py-0.5 rounded bg-accent/10 border border-accent/20">
                          AI
                        </span>
                      )}
                      {record.auto_sent && (
                        <span className="text-xs text-blue-400 px-2 py-0.5 rounded bg-blue-500/10 border border-blue-500/20">
                          Sent
                        </span>
                      )}
                      {/* Botão deletar por item */}
                      <button
                        onClick={() => deleteRecord(record.id)}
                        className="text-muted-foreground hover:text-red-400 transition-colors p-1 rounded hover:bg-red-500/10"
                        title="Deletar esta transcrição"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                  <div className="space-y-1.5">
                    {record.raw_text !== record.processed_text && (
                      <p className="text-sm text-muted-foreground italic line-clamp-1 opacity-60">
                        "{record.raw_text}"
                      </p>
                    )}
                    <p className="text-base text-foreground font-medium leading-snug">
                      {record.processed_text || record.raw_text}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
