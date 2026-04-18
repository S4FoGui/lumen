import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Save, Loader2, CheckCircle2, Zap, Target, Mic } from 'lucide-react';

const MODEL_LABELS: Record<string, string> = { base: 'Base', small: 'Small', medium: 'Medium' };

export function ConfigTab() {
  const [config, setConfig] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [savedStatus, setSavedStatus] = useState(false);

  const [devices, setDevices] = useState<{id: string, label: string}[]>([]);

  // Carrega a configuração inicial
  useEffect(() => {
    fetch('/api/config')
      .then(res => res.json())
      .then(data => {
        setConfig(data);
        setLoading(false);
      })
      .catch(err => {
        console.error("Falha ao carregar configuração:", err);
        setLoading(false);
      });

    fetch('/api/devices')
      .then(res => res.json())
      .then(data => setDevices(data.devices || []))
      .catch(() => {});
  }, []);

  // Update locale state
  const updateConfig = (section: string, key: string, value: any) => {
    setConfig((prev: any) => {
      // ✅ FIX: Verificar se prev existe e prev[section] existe
      if (!prev || typeof prev !== 'object') {
        console.warn('Config não carregada ainda');
        return prev;
      }
      return {
        ...prev,
        [section]: {
          ...(prev[section] || {}),
          [key]: value
        }
      };
    });
    setSavedStatus(false);
  };



  // Salvar no Backend
  const saveConfig = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const res = await fetch('/api/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config)
      });
      if (res.ok) {
        setSavedStatus(true);
        setTimeout(() => setSavedStatus(false), 3000);
      } else {
        alert("Falha ao salvar. Verifique se há campos em branco.");
      }
    } catch (err) {
      console.error(err);
      alert("Erro de rede ao salvar configuração.");
    } finally {
      setSaving(false);
    }
  };

  if (loading || !config) {
    return <div className="flex h-64 items-center justify-center"><Loader2 className="w-8 h-8 animate-spin opacity-50" /></div>;
  }



  return (
    <div className="space-y-6 max-w-4xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 pb-24">
        
        <div className="flex justify-between items-center mb-8">
            <h2 className="text-3xl font-bold tracking-tight">Hardware & Configuration</h2>
            <Button 
                onClick={saveConfig} 
                disabled={saving}
                className={savedStatus ? "bg-green-600 hover:bg-green-700" : "bg-accent hover:bg-accent/90"}
            >
                {saving ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : 
                 savedStatus ? <CheckCircle2 className="w-4 h-4 mr-2" /> : 
                 <Save className="w-4 h-4 mr-2" />}
                {savedStatus ? "Salvo com sucesso" : "Salvar Configurações"}
            </Button>
        </div>
        
        {/* Lógica / Engine de Transcrição e Produtividade */}
        <Card className="bg-card/50 backdrop-blur">
          <CardHeader>
              <CardTitle>Voice Engine Parameters</CardTitle>
              <CardDescription>Configura o núcleo de transcrição Whisper.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
              <div className="space-y-2">
                <Label>Audio Input Device</Label>
                <Select 
                  value={config.audio?.device || "default"} 
                  onValueChange={(val) => updateConfig('audio', 'device', val === "default" ? null : val)}
                >
                  <SelectTrigger className="w-full bg-secondary/50 border-border truncate">
                    <SelectValue placeholder="Padrão do Sistema" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">Padrão do Sistema (Automático / Pulseaudio / Pipewire)</SelectItem>
                    {devices.filter(d => d.id !== "null").map((device) => (
                      <SelectItem key={device.id} value={device.id} className="truncate" title={device.label}>
                        {device.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-2">
                <Label>Language Target</Label>
                <Select 
                  value={config.transcription.language} 
                  onValueChange={(val) => updateConfig('transcription', 'language', val)}
                >
                  <SelectTrigger className="w-full bg-secondary/50 border-border">
                    <SelectValue placeholder="Selecione o Idioma" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="pt">Português (Brasil)</SelectItem>
                    <SelectItem value="en">English (US)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-4 p-4 rounded-lg border border-accent/20 bg-accent/5">
                <h3 className="text-sm font-medium text-accent">Roteamento Dinâmico de Transcrição</h3>
                <p className="text-xs text-muted-foreground">Escolha quais modelos carregar. Especifique quem assume áudios curtos e longos.</p>
                
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label className="text-xs">Modelo p/ Áudios Curtos</Label>
                    <Select 
                      value={config.transcription.model_short || "base"} 
                      onValueChange={(val) => updateConfig('transcription', 'model_short', val)}
                    >
                      <SelectTrigger className="w-full bg-background/50 border-border">
                        <span>{MODEL_LABELS[config.transcription.model_short] || 'Base'}</span>
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="base" textValue="Base">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Base</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />3v
                              <Target className="w-3 h-3 text-red-400 ml-1" />1p
                            </span>
                          </span>
                        </SelectItem>
                        <SelectItem value="small" textValue="Small">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Small</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />2v
                              <Target className="w-3 h-3 text-red-400 ml-1" />2p
                            </span>
                          </span>
                        </SelectItem>
                        <SelectItem value="medium" textValue="Medium">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Medium</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />1v
                              <Target className="w-3 h-3 text-red-400 ml-1" />3p
                            </span>
                          </span>
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  
                  <div className="space-y-2">
                    <Label className="text-xs">Modelo p/ Áudios Longos</Label>
                    <Select 
                      value={config.transcription.model_long || "medium"} 
                      onValueChange={(val) => updateConfig('transcription', 'model_long', val)}
                    >
                      <SelectTrigger className="w-full bg-background/50 border-border">
                        <span>{MODEL_LABELS[config.transcription.model_long] || 'Medium'}</span>
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="base" textValue="Base">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Base</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />3v
                              <Target className="w-3 h-3 text-red-400 ml-1" />1p
                            </span>
                          </span>
                        </SelectItem>
                        <SelectItem value="small" textValue="Small">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Small</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />2v
                              <Target className="w-3 h-3 text-red-400 ml-1" />2p
                            </span>
                          </span>
                        </SelectItem>
                        <SelectItem value="medium" textValue="Medium">
                          <span className="flex items-center justify-between w-full gap-3">
                            <span>Medium</span>
                            <span className="flex items-center gap-1.5 text-[10px] text-muted-foreground font-mono">
                              <Zap className="w-3 h-3 text-yellow-400" />1v
                              <Target className="w-3 h-3 text-red-400 ml-1" />3p
                            </span>
                          </span>
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div className="space-y-2 pt-2 border-t border-border/50">
                  <div className="flex justify-between items-center">
                    <Label className="text-xs">Corte de Duração (Segundos)</Label>
                    <span className="text-xs text-accent font-mono">{config.transcription.duration_threshold_sec || 5.0} s</span>
                  </div>
                  <Input 
                    type="number" 
                    step="0.5"
                    min="1.0"
                    value={config.transcription.duration_threshold_sec || 5.0} 
                    onChange={(e) => updateConfig('transcription', 'duration_threshold_sec', parseFloat(e.target.value) || 5.0)}
                    className="bg-background/50 border-border"
                  />
                  <p className="text-[10px] text-muted-foreground mt-1">Áudios maiores que este tempo vão para o Modelo Longo.</p>
                </div>
              </div>
              
              <div className={`flex items-center justify-between p-4 rounded-lg border transition-colors ${config.transcription.always_listening ? 'border-border bg-secondary/10 opacity-50 cursor-not-allowed' : 'border-border bg-secondary/20 hover:bg-secondary/30'}`}>
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold cursor-pointer" htmlFor="auto-send">Auto-Send (Enter Automático)</Label>
                    <p className="text-sm text-muted-foreground">
                      {config.transcription.always_listening 
                        ? 'Desativado — Always Listening usa Voice Commands para enviar.' 
                        : 'Envia a mensagem automaticamente logo após transcrever.'}
                    </p>
                </div>
                <Switch 
                  id="auto-send"
                  checked={config.transcription.auto_send} 
                  onCheckedChange={(val) => updateConfig('transcription', 'auto_send', val)}
                  disabled={config.transcription.always_listening}
                />
              </div>

              <div className={`flex items-center justify-between p-4 rounded-lg border transition-colors ${config.transcription.always_listening ? 'border-accent/30 bg-accent/5' : 'border-border bg-secondary/20 hover:bg-secondary/30'}`}>
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="voice-commands">Voice Commands</Label>
                    <p className="text-sm text-muted-foreground">
                      {config.transcription.always_listening
                        ? 'Ativado automaticamente pelo Always Listening.'
                        : 'Habilita detecção de comandos ("apague", "nova linha", "envie").'}
                    </p>
                </div>
                <Switch 
                  id="voice-commands"
                  checked={config.transcription.voice_commands_enabled} 
                  onCheckedChange={(val) => updateConfig('transcription', 'voice_commands_enabled', val)}
                  disabled={config.transcription.always_listening}
                />
              </div>

              <div className="flex items-center justify-between p-4 rounded-lg border border-border bg-secondary/20 hover:bg-secondary/30 transition-colors">
                <div className="space-y-0.5">
                    <Label className="text-base text-accent cursor-pointer" htmlFor="lightning-mode">Lightning Mode</Label>
                    <p className="text-sm text-muted-foreground">Modo ultra-rápido para Whisper, com leve perda de acurácia.</p>
                </div>
                <Switch
                  id="lightning-mode"
                  checked={config.transcription.lightning_mode}
                  onCheckedChange={(val) => updateConfig('transcription', 'lightning_mode', val)}
                />
              </div>

              <div className="flex items-center justify-between p-4 rounded-lg border border-accent/40 bg-accent/10 hover:bg-accent/15 transition-colors shadow-[0_0_15px_rgba(163,230,53,0.08)]">
                <div className="space-y-0.5">
                    <div className="flex items-center gap-2">
                      <Mic className="w-4 h-4 text-accent" />
                      <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="always-listening">Sempre Escutando</Label>
                    </div>
                    <p className="text-sm text-muted-foreground">Mantém o Lumen ouvindo continuamente e só processa quando detectar a palavra de ativação.</p>
                </div>
                <Switch
                  id="always-listening"
                  checked={config.transcription.always_listening || false}
                  onCheckedChange={(val) => {
                    // Inter-dependências automáticas
                    const updates: Record<string, any> = { always_listening: val };
                    if (val) {
                      updates.auto_send = false;
                      updates.voice_commands_enabled = true;
                    } else {
                      updates.voice_commands_enabled = false;
                    }
                    const newConfig = {
                      ...config,
                      transcription: { ...config.transcription, ...updates }
                    };
                    setConfig(newConfig);
                    fetch('/api/config', {
                      method: 'PUT',
                      headers: { 'Content-Type': 'application/json' },
                      body: JSON.stringify(newConfig)
                    });
                  }}
                />
              </div>

              {(config.transcription.always_listening || false) && (
                <div className="space-y-4 p-4 rounded-lg border border-accent/30 bg-accent/5 animate-in fade-in slide-in-from-top-2">
                  <div className="space-y-2">
                    <Label>Palavra de ativação (wake word)</Label>
                    <Input
                      value={config.transcription.wake_word || 'lumen'}
                      onChange={(e) => updateConfig('transcription', 'wake_word', e.target.value)}
                      placeholder="lumen"
                      className="bg-background/50 border-border"
                    />
                    <p className="text-xs text-muted-foreground">Exemplo: diga "Lumen, escreva ..." para ativar.</p>
                  </div>

                  <div className="border-t border-border/50 pt-3">
                    <h4 className="text-sm font-medium text-accent mb-3">Comandos Disponíveis</h4>
                    <div className="grid grid-cols-2 gap-2">
                      {[
                        { cmd: '"Escreva" / "Digite"', desc: 'Transcreve e digita exatamente o que você disser', icon: '📝' },
                        { cmd: '"Apague"', desc: 'Apaga todo o texto do campo', icon: '🗑️' },
                        { cmd: '"Selecionar tudo"', desc: 'Seleciona todo o texto (Ctrl+A)', icon: '📋' },
                        { cmd: '"Copiar"', desc: 'Copia o texto selecionado (Ctrl+C)', icon: '📄' },
                        { cmd: '"Melhorar"', desc: 'Seleciona tudo, envia para IA e cola versão melhorada', icon: '✨' },
                        { cmd: '"Envie"', desc: 'Pressiona Enter para enviar a mensagem', icon: '📤' },
                        { cmd: '"Nova linha"', desc: 'Insere uma quebra de linha', icon: '↵' },
                      ].map(({cmd, desc, icon}) => (
                        <div key={cmd} className="flex items-start gap-2 p-2 rounded-md bg-secondary/20 border border-border/50">
                          <span className="text-base mt-0.5">{icon}</span>
                          <div>
                            <p className="text-xs font-mono font-semibold text-accent">{cmd}</p>
                            <p className="text-[10px] text-muted-foreground leading-tight">{desc}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
          </CardContent>
        </Card>
        
    </div>
  );
}
