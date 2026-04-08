import { Clock, FileText, LayoutDashboard } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { WaveformVisualizer } from '../Waveform';
import type { SystemStatus, TranscriptionRecord } from '../../hooks/useLumenSocket';

type DashboardTabProps = {
  status: SystemStatus;
  rms: number;
  lastTranscription: TranscriptionRecord | null;
};

const formatUptime = (secs: number) => {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  return `${h > 0 ? h + 'h ' : ''}${m > 0 ? m + 'm ' : ''}${s}s`;
};

export function DashboardTab({ status, rms, lastTranscription }: DashboardTabProps) {
  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500">
      <div className="flex justify-between items-center mb-8">
        <h2 className="text-3xl font-bold tracking-tight">Ecosystem Status</h2>
        <div className="flex items-center gap-2 px-4 py-1.5 rounded-full bg-secondary border border-border">
          <span className={`w-2 h-2 rounded-full ${status.is_recording ? 'bg-red-500 shadow-[0_0_10px_rgba(239,68,68,0.8)]' : 'bg-accent shadow-[0_0_10px_rgba(163,230,53,0.8)]'}`} />
          <span className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
            {status.is_recording ? 'Listening' : 'Ready'}
          </span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-8">
        <div className="lg:col-span-2">
            <Card className="bg-card/50 backdrop-blur border-border overflow-hidden h-full">
            <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">Live Telemetry</CardTitle>
            </CardHeader>
            <CardContent className="flex flex-col items-center justify-center pt-4">
                <WaveformVisualizer rms={rms} isRecording={status.is_recording} />
                <p className="mt-4 text-sm text-muted-foreground">
                {status.is_recording ? 'Capturando áudio e detectando silêncio via VAD...' : 'Pressione Enter 2x rapidamente (double-tap) para iniciar'}
                </p>
            </CardContent>
            </Card>
        </div>
        
        <div className="space-y-6">
            <Card className="bg-card/50 backdrop-blur border-border overflow-hidden group">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
                <CardTitle className="text-sm font-medium text-muted-foreground">Engine Uptime</CardTitle>
                <Clock className="w-4 h-4 text-muted-foreground group-hover:text-accent transition-colors" />
            </CardHeader>
            <CardContent>
                <p className="text-2xl font-bold">{formatUptime(status.uptime_seconds)}</p>
            </CardContent>
            </Card>
            
            <Card className="bg-card/50 backdrop-blur border-border overflow-hidden group">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
                <CardTitle className="text-sm font-medium text-muted-foreground">Transcriptions</CardTitle>
                <FileText className="w-4 h-4 text-muted-foreground group-hover:text-accent transition-colors" />
            </CardHeader>
            <CardContent>
                <p className="text-2xl font-bold">{status.total_transcriptions}</p>
            </CardContent>
            </Card>

            <Card className="bg-card/50 backdrop-blur border-border overflow-hidden group">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
                <CardTitle className="text-sm font-medium text-muted-foreground">Words Flowing</CardTitle>
                <LayoutDashboard className="w-4 h-4 text-muted-foreground group-hover:text-accent transition-colors" />
            </CardHeader>
            <CardContent>
                <p className="text-2xl font-bold text-accent">{status.total_words}</p>
            </CardContent>
            </Card>
        </div>
      </div>

      {lastTranscription && (
        <Card className="bg-card/50 backdrop-blur border-border overflow-hidden animate-in fade-in slide-in-from-bottom-2">
            <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground flex justify-between">
                    <span>Última Transcrição</span>
                    <span>{lastTranscription.processing_time_ms}ms</span>
                </CardTitle>
            </CardHeader>
            <CardContent>
                <p className="text-base text-foreground/80 mb-2 italic">"{lastTranscription.raw_text}"</p>
                <div className="p-4 rounded-md bg-secondary/30 border border-border/50">
                    <p className="text-lg font-medium">{lastTranscription.processed_text}</p>
                </div>
                <div className="mt-4 flex gap-2">
                    {lastTranscription.ai_used && (
                        <span className="px-2 py-1 text-xs rounded-full bg-accent/20 text-accent border border-accent/30">AI Formatted</span>
                    )}
                    {lastTranscription.auto_sent && (
                        <span className="px-2 py-1 text-xs rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">Auto Sent</span>
                    )}
                </div>
            </CardContent>
        </Card>
      )}

    </div>
  );
}
