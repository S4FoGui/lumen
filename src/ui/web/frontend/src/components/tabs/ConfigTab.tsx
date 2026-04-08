import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Save, Loader2, CheckCircle2 } from 'lucide-react';

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
    setConfig((prev: any) => ({
      ...prev,
      [section]: {
        ...prev[section],
        [key]: value
      }
    }));
    setSavedStatus(false);
  };

  const updateAiProviderConfig = (provider: string, key: string, value: any) => {
    setConfig((prev: any) => ({
      ...prev,
      ai: {
        ...prev.ai,
        [provider]: {
          ...prev.ai[provider],
          [key]: value
        }
      }
    }));
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

  const aiProvider = config.ai?.provider || 'disabled';

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
              
              <div className="flex items-center justify-between p-4 rounded-lg border border-border bg-secondary/20 hover:bg-secondary/30 transition-colors">
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold cursor-pointer" htmlFor="auto-send">Auto-Send (Enter Automático)</Label>
                    <p className="text-sm text-muted-foreground">Envia a mensagem automaticamente logo após transcrever.</p>
                </div>
                <Switch 
                  id="auto-send"
                  checked={config.transcription.auto_send} 
                  onCheckedChange={(val) => updateConfig('transcription', 'auto_send', val)} 
                />
              </div>

              <div className="flex items-center justify-between p-4 rounded-lg border border-border bg-secondary/20 hover:bg-secondary/30 transition-colors">
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="voice-commands">Voice Commands</Label>
                    <p className="text-sm text-muted-foreground">Habilita detecção de comandos ("apague", "nova linha", "envie").</p>
                </div>
                <Switch 
                  id="voice-commands"
                  checked={config.transcription.voice_commands_enabled} 
                  onCheckedChange={(val) => updateConfig('transcription', 'voice_commands_enabled', val)} 
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

              <div className="flex items-center justify-between p-4 rounded-lg border border-border bg-secondary/20 hover:bg-secondary/30 transition-colors">
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="always-listening">Sempre Escutando</Label>
                    <p className="text-sm text-muted-foreground">Mantém o Lumen ouvindo continuamente e só processa quando detectar a palavra de ativação.</p>
                </div>
                <Switch
                  id="always-listening"
                  checked={config.transcription.always_listening || false}
                  onCheckedChange={(val) => updateConfig('transcription', 'always_listening', val)}
                />
              </div>

              {(config.transcription.always_listening || false) && (
                <div className="space-y-2 p-4 rounded-lg border border-accent/30 bg-accent/5">
                  <Label>Palavra de ativação (wake word)</Label>
                  <Input
                    value={config.transcription.wake_word || 'lumen'}
                    onChange={(e) => updateConfig('transcription', 'wake_word', e.target.value)}
                    placeholder="lumen"
                    className="bg-background/50 border-border"
                  />
                  <p className="text-xs text-muted-foreground">Exemplo: diga "Lumen, escreva ..." para ativar.</p>
                </div>
              )}
          </CardContent>
        </Card>

        {/* AI Formatter Config */}
        <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
          <CardHeader>
              <CardTitle>AI Formatter</CardTitle>
              <CardDescription>IA responsável por formatar e processar comandos complexos do texto.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
              <div className="flex items-center justify-between p-4 rounded-lg border border-accent/30 bg-accent/5">
                <div className="space-y-0.5">
                    <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="auto-formatting">Auto-Improve (Correção por IA)</Label>
                    <p className="text-sm text-muted-foreground">Melhora gramática e remove vícios de linguagem ("humm", "tipo") automaticamente de TUDO que for transcrito usando o provedor abaixo.</p>
                </div>
                <Switch 
                  id="auto-formatting"
                  checked={config.ai.auto_formatting} 
                  onCheckedChange={(val) => updateConfig('ai', 'auto_formatting', val)} 
                />
              </div>

              <div className="space-y-2">
                <Label>Provider Central</Label>
                <Select 
                  value={aiProvider} 
                  onValueChange={(val) => updateConfig('ai', 'provider', val)}
                >
                  <SelectTrigger className="w-full bg-secondary/50 border-border">
                    <SelectValue placeholder="Selecione um Provedor" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="disabled">Desabilitado</SelectItem>
                    <SelectItem value="omniroute">OmniRoute (Multi-Model Gateway)</SelectItem>
                    <SelectItem value="ollama">Ollama (Local / Offline)</SelectItem>
                    <SelectItem value="openai">OpenAI (Cloud)</SelectItem>
                    <SelectItem value="gemini">Google Gemini (Cloud)</SelectItem>
                    <SelectItem value="groq">Groq (High-Speed Llama)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {aiProvider === 'omniroute' && (
                <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
                  <div className="space-y-2">
                    <Label>URL Base do Gateway</Label>
                    <Input 
                      value={config.ai.omniroute?.url || ''} 
                      onChange={(e) => updateAiProviderConfig('omniroute', 'url', e.target.value)}
                      placeholder="http://cloud.omniroute.online/v1" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Bearer Token (API Key)</Label>
                    <Input 
                      type="password" 
                      value={config.ai.omniroute?.api_key || ''} 
                      onChange={(e) => updateAiProviderConfig('omniroute', 'api_key', e.target.value)}
                      placeholder="seu-api-key-aqui" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Modelo Alvo</Label>
                    <Input 
                      value={config.ai.omniroute?.model || ''} 
                      onChange={(e) => updateAiProviderConfig('omniroute', 'model', e.target.value)}
                      placeholder="openai/text-embedding-3-small..." 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <p className="text-xs text-muted-foreground mt-2">API 100% compatível com OpenAI. Encaminha para 60+ provedores.</p>
                </div>
              )}

              {aiProvider === 'ollama' && (
                <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
                  <div className="space-y-2">
                    <Label>Endpoint URL</Label>
                    <Input 
                      value={config.ai.ollama?.url || ''} 
                      onChange={(e) => updateAiProviderConfig('ollama', 'url', e.target.value)}
                      placeholder="http://127.0.0.1:11434" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Opcional: API Key (Bearer Token)</Label>
                    <Input 
                      type="password" 
                      value={config.ai.ollama?.api_key || ''} 
                      onChange={(e) => updateAiProviderConfig('ollama', 'api_key', e.target.value)}
                      placeholder="Caso use o Ollama via NGINX/Proxy" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Modelo</Label>
                    <Input 
                      value={config.ai.ollama?.model || ''} 
                      onChange={(e) => updateAiProviderConfig('ollama', 'model', e.target.value)}
                      placeholder="llama3, gemma2..." 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                </div>
              )}

              {aiProvider === 'openai' && (
                <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
                  <div className="space-y-2">
                    <Label>API Key</Label>
                    <Input 
                      type="password" 
                      value={config.ai.openai?.api_key || ''} 
                      onChange={(e) => updateAiProviderConfig('openai', 'api_key', e.target.value)}
                      placeholder="sk-..." 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Modelo</Label>
                    <Input 
                      value={config.ai.openai?.model || ''} 
                      onChange={(e) => updateAiProviderConfig('openai', 'model', e.target.value)}
                      placeholder="gpt-4o-mini" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                </div>
              )}

              {aiProvider === 'groq' && (
                <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
                  <div className="space-y-2">
                    <Label>Groq API Key</Label>
                    <Input 
                      type="password" 
                      value={config.ai.groq?.api_key || ''} 
                      onChange={(e) => updateAiProviderConfig('groq', 'api_key', e.target.value)}
                      placeholder="gsk_..." 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Modelo</Label>
                    <Input 
                      value={config.ai.groq?.model || ''} 
                      onChange={(e) => updateAiProviderConfig('groq', 'model', e.target.value)}
                      placeholder="llama-3.3-70b-versatile" 
                      className="bg-background/50 border-border" 
                    />
                  </div>
                </div>
              )}
          </CardContent>
        </Card>
        
    </div>
  );
}
