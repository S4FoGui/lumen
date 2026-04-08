import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { FileText, Trash2 } from 'lucide-react';
import type { TranscriptionRecord } from '../../hooks/useLumenSocket';

export function HistoryTab() {
  const [history, setHistory] = useState<TranscriptionRecord[]>([]);
  const [loading, setLoading] = useState(true);

  const loadHistory = async () => {
    try {
      const res = await fetch('/api/transcriptions?limit=20');
      if (res.ok) {
        setHistory(await res.json());
      }
    } catch (err) {
      console.error('Falha ao carregar histórico', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  const formatDate = (isoStr: string) => {
    const d = new Date(isoStr);
    return new Intl.DateTimeFormat('pt-BR', {
      hour: '2-digit', minute: '2-digit', second: '2-digit',
      day: '2-digit', month: '2-digit'
    }).format(d);
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex justify-between items-center mb-8">
        <h2 className="text-3xl font-bold tracking-tight">Transcription History</h2>
      </div>

      <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
        <CardHeader>
          <CardTitle className="text-sm font-medium text-muted-foreground">Histórico Local SQLite</CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="flex justify-center p-8"><span className="animate-pulse">Carregando...</span></div>
          ) : history.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-12 text-muted-foreground">
              <FileText className="w-12 h-12 mb-4 opacity-50" />
              <p>Nenhuma transcrição salva localmente ainda.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {history.map((record) => (
                <div key={record.id} className="p-4 rounded-lg bg-secondary/30 border border-border/50 hover:bg-secondary/50 transition-colors">
                  <div className="flex justify-between items-start mb-2">
                    <span className="text-xs text-muted-foreground bg-background px-2 py-1 rounded border border-border">
                      {formatDate(record.timestamp)}
                    </span>
                    <div className="flex gap-2">
                        <span className="text-xs text-muted-foreground px-2 py-1">{record.processing_time_ms}ms</span>
                        {record.ai_used && <span className="text-xs text-accent px-2 py-1 rounded bg-accent/10 border border-accent/20">AI</span>}
                        <button className="text-muted-foreground hover:text-red-400 transition-colors">
                            <Trash2 className="w-4 h-4" />
                        </button>
                    </div>
                  </div>
                  <div className="space-y-2">
                     <p className="text-sm text-muted-foreground italic line-clamp-2">{record.raw_text}</p>
                     <p className="text-base text-foreground font-medium">{record.processed_text}</p>
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
