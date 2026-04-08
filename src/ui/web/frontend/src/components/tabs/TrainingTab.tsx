import { useState, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Mic, Square, Download, Trash2, CheckCircle2, Loader2 } from 'lucide-react';
import { Progress } from '@/components/ui/progress';

export function TrainingTab() {
  const [isRecording, setIsRecording] = useState(false);
  const [recordings, setRecordings] = useState<{id: string, text: string, duration: number, blob?: Blob}[]>([]);
  const [currentRecording, setCurrentRecording] = useState<{id: string, startTime: number} | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);

  // Frases sugeridas para treinar o modelo
  const trainingPhrases = [
    "Olá, meu nome é [seu nome] e estou testando o sistema de reconhecimento de voz.",
    "O rato roeu a roupa do rei de Roma.",
    "Três pratos de trigo para três tigres tristes.",
    "A aranha arranha a jarra, a jarra arranha a aranha.",
    "Hoje está um dia ensolarado e agradável.",
    "Preciso enviar um email importante para o cliente.",
    "Vou fazer uma reunião às três horas da tarde.",
    "O projeto está avançando conforme o planejado.",
    "Gostaria de agendar uma consulta para amanhã.",
    "Por favor, confirme o recebimento desta mensagem."
  ];

  const startRecording = async (phraseText: string) => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const mediaRecorder = new MediaRecorder(stream);
      mediaRecorderRef.current = mediaRecorder;
      chunksRef.current = [];

      const recordingId = Date.now().toString();
      setCurrentRecording({ id: recordingId, startTime: Date.now() });

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) {
          chunksRef.current.push(e.data);
        }
      };

      mediaRecorder.onstop = () => {
        const blob = new Blob(chunksRef.current, { type: 'audio/webm' });
        const duration = Math.floor((Date.now() - (currentRecording?.startTime || 0)) / 1000);

        setRecordings(prev => [...prev, {
          id: recordingId,
          text: phraseText,
          duration,
          blob
        }]);

        setCurrentRecording(null);
        stream.getTracks().forEach(track => track.stop());
      };

      mediaRecorder.start();
      setIsRecording(true);
    } catch (err) {
      console.error('Erro ao acessar microfone:', err);
      alert('Erro ao acessar o microfone. Verifique as permissões.');
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && isRecording) {
      mediaRecorderRef.current.stop();
      setIsRecording(false);
    }
  };

  const deleteRecording = (id: string) => {
    setRecordings(prev => prev.filter(r => r.id !== id));
  };

  const downloadRecording = (recording: {id: string, text: string, blob?: Blob}) => {
    if (!recording.blob) return;
    const url = URL.createObjectURL(recording.blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `training_${recording.id}.webm`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const sendToTraining = async () => {
    if (recordings.length === 0) {
      alert('Grave pelo menos uma frase antes de enviar para treinamento.');
      return;
    }

    setIsProcessing(true);

    try {
      const formData = new FormData();

      recordings.forEach((rec, idx) => {
        if (rec.blob) {
          formData.append(`audio_${idx}`, rec.blob, `training_${rec.id}.webm`);
          formData.append(`text_${idx}`, rec.text);
        }
      });

      const response = await fetch('/api/training/upload', {
        method: 'POST',
        body: formData
      });

      if (response.status === 404) {
        alert('⚠️ Endpoint /api/training/upload ainda não existe no backend atual. As gravações ficaram salvas localmente no navegador para exportação manual.');
        return;
      }

      if (response.ok) {
        alert('✅ Gravações enviadas com sucesso! O modelo será ajustado com suas amostras.');
        setRecordings([]);
      } else {
        const msg = await response.text().catch(() => 'Erro desconhecido');
        alert(`❌ Erro ao enviar gravações: ${msg}`);
      }
    } catch (err) {
      console.error('Erro ao enviar:', err);
      alert('❌ Erro de rede ao enviar gravações.');
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 pb-12">
      <div className="flex justify-between items-center mb-8">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Voice Training</h2>
          <p className="text-muted-foreground mt-1">Treine o modelo com sua voz para melhor reconhecimento</p>
        </div>
        {recordings.length > 0 && (
          <Button
            onClick={sendToTraining}
            disabled={isProcessing}
            className="bg-accent hover:bg-accent/90"
          >
            {isProcessing ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <CheckCircle2 className="w-4 h-4 mr-2" />}
            Enviar {recordings.length} gravação(ões)
          </Button>
        )}
      </div>

      {/* Instruções */}
      <Card className="bg-card/50 backdrop-blur border-accent/30">
        <CardHeader>
          <CardTitle className="text-accent">Como funciona o treinamento?</CardTitle>
          <CardDescription>
            Grave as frases abaixo com sua voz natural. Quanto mais amostras você fornecer, melhor será o reconhecimento.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-sm text-muted-foreground">
          <p>✅ Fale naturalmente, sem forçar a pronúncia</p>
          <p>✅ Grave em um ambiente silencioso</p>
          <p>✅ Use o mesmo microfone que usará no dia a dia</p>
          <p>✅ Grave pelo menos 5 frases diferentes</p>
          <p>⚠️ As gravações são processadas localmente e não são enviadas para a nuvem</p>
        </CardContent>
      </Card>

      {/* Frases para gravar */}
      <Card className="bg-card/50 backdrop-blur border-border">
        <CardHeader>
          <CardTitle>Frases de Treinamento</CardTitle>
          <CardDescription>Clique em "Gravar" e leia a frase em voz alta</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {trainingPhrases.map((phrase, idx) => {
            const isRecorded = recordings.some(r => r.text === phrase);

            return (
              <div
                key={idx}
                className={`p-4 rounded-lg border transition-all ${
                  isRecorded
                    ? 'bg-accent/10 border-accent/30'
                    : 'bg-secondary/20 border-border hover:border-accent/20'
                }`}
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-2">
                      <span className="text-xs font-mono text-muted-foreground">#{idx + 1}</span>
                      {isRecorded && <CheckCircle2 className="w-4 h-4 text-accent" />}
                    </div>
                    <p className="text-base">{phrase}</p>
                  </div>

                  <Button
                    size="sm"
                    onClick={() => isRecording ? stopRecording() : startRecording(phrase)}
                    disabled={isRecording && currentRecording?.id !== `${idx}`}
                    className={isRecording ? "bg-red-500 hover:bg-red-600" : ""}
                  >
                    {isRecording && currentRecording ? (
                      <>
                        <Square className="w-4 h-4 mr-2" />
                        Parar
                      </>
                    ) : (
                      <>
                        <Mic className="w-4 h-4 mr-2" />
                        Gravar
                      </>
                    )}
                  </Button>
                </div>
              </div>
            );
          })}
        </CardContent>
      </Card>

      {/* Gravações realizadas */}
      {recordings.length > 0 && (
        <Card className="bg-card/50 backdrop-blur border-border">
          <CardHeader>
            <CardTitle>Gravações Realizadas ({recordings.length})</CardTitle>
            <CardDescription>Suas amostras de voz para treinamento</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {recordings.map((rec) => (
              <div
                key={rec.id}
                className="flex items-center justify-between p-3 rounded-lg bg-secondary/30 border border-border"
              >
                <div className="flex-1">
                  <p className="text-sm font-medium truncate">{rec.text}</p>
                  <p className="text-xs text-muted-foreground">{rec.duration}s</p>
                </div>
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => downloadRecording(rec)}
                  >
                    <Download className="w-4 h-4" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => deleteRecording(rec.id)}
                    className="text-red-500 hover:text-red-600"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Progresso */}
      <Card className="bg-card/50 backdrop-blur border-border">
        <CardHeader>
          <CardTitle>Progresso do Treinamento</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Frases gravadas</span>
              <span className="font-medium">{recordings.length} / {trainingPhrases.length}</span>
            </div>
            <Progress value={(recordings.length / trainingPhrases.length) * 100} className="h-2" />
            <p className="text-xs text-muted-foreground mt-2">
              {recordings.length < 5
                ? `Grave pelo menos ${5 - recordings.length} frase(s) para começar o treinamento`
                : '✅ Você já tem amostras suficientes! Clique em "Enviar" para treinar o modelo.'}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
