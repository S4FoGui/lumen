import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Save, Loader2, CheckCircle2, Plus, Trash2, RotateCcw, Sparkles, Zap, Maximize2, X } from 'lucide-react';

const DEFAULT_PROMPT = "You are an advanced voice dictation pre-processor. The input is raw speech-to-text. Your goal is to completely remove language fillers (e.g., um, ah, you know, tipo, né, então), clean up accidental repetitions, and fix stutters. Apply perfect grammar and punctuation while preserving 100% of the original meaning and natural tone. CRITICAL RULE: You MUST output the response in the EXACT same language as the input (e.g., if the user speaks in English, output in English; if Portuguese, output in Portuguese). Do NOT translate the text.";

export function SnippetsTab() {
  const [config, setConfig] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [, setSaving] = useState(false);
  const [savedStatus, setSavedStatus] = useState(false);
  const [newPromptName, setNewPromptName] = useState('');
  const [newPromptContent, setNewPromptContent] = useState('');
  const [editingPrompt, setEditingPrompt] = useState<string | null>(null);
  const [editContent, setEditContent] = useState('');
  // Fullscreen editor state: { type: 'default' | 'custom', name?: string }
  const [fullscreen, setFullscreen] = useState<{ type: string; name?: string } | null>(null);
  const [fullscreenContent, setFullscreenContent] = useState('');

  useEffect(() => {
    fetch('/api/config')
      .then(r => r.json())
      .then(data => { setConfig(data); setLoading(false); })
      .catch(err => { console.error(err); setLoading(false); });
  }, []);

  const saveConfig = async (newConfig: any) => {
    setSaving(true);
    try {
      const res = await fetch('/api/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(newConfig)
      });
      if (res.ok) {
        setSavedStatus(true);
        setTimeout(() => setSavedStatus(false), 2000);
      }
    } catch (err) {
      console.error('Erro ao salvar:', err);
    }
    setSaving(false);
  };

  const updateConfig = (section: string, key: string, value: any) => {
    const newConfig = { ...config, [section]: { ...config[section], [key]: value } };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  const updateAiProviderConfig = (provider: string, key: string, value: string) => {
    const newConfig = {
      ...config,
      ai: { ...config.ai, [provider]: { ...config.ai[provider], [key]: value } }
    };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  // ── Prompt Management ──
  const customPrompts: Record<string, string> = config?.ai?.custom_prompts || {};
  const activePromptName: string = config?.ai?.active_prompt_name || '';

  const getActivePromptContent = (): string => {
    if (activePromptName && customPrompts[activePromptName]) {
      return customPrompts[activePromptName];
    }
    return config?.ai?.default_instruction || DEFAULT_PROMPT;
  };

  const addPrompt = () => {
    const name = newPromptName.trim();
    if (!name || customPrompts[name]) return;
    const content = newPromptContent.trim() || DEFAULT_PROMPT;
    const updated = { ...customPrompts, [name]: content };
    const newConfig = {
      ...config,
      ai: { ...config.ai, custom_prompts: updated, active_prompt_name: name }
    };
    setConfig(newConfig);
    saveConfig(newConfig);
    setNewPromptName('');
    setNewPromptContent('');
  };

  const deletePrompt = (name: string) => {
    const updated = { ...customPrompts };
    delete updated[name];
    const newActive = activePromptName === name ? '' : activePromptName;
    const newConfig = {
      ...config,
      ai: { ...config.ai, custom_prompts: updated, active_prompt_name: newActive }
    };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  const selectPrompt = (name: string) => {
    // "" means use default_instruction
    const newConfig = {
      ...config,
      ai: { ...config.ai, active_prompt_name: name }
    };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  const saveEditedPrompt = (name: string) => {
    const updated = { ...customPrompts, [name]: editContent };
    const newConfig = { ...config, ai: { ...config.ai, custom_prompts: updated } };
    setConfig(newConfig);
    saveConfig(newConfig);
    setEditingPrompt(null);
  };

  const resetDefault = () => {
    const newConfig = {
      ...config,
      ai: { ...config.ai, default_instruction: DEFAULT_PROMPT }
    };
    setConfig(newConfig);
    saveConfig(newConfig);
  };

  const saveDefaultInstruction = (value: string) => {
    const newConfig = {
      ...config,
      ai: { ...config.ai, default_instruction: value }
    };
    setConfig(newConfig);
    // Don't auto-save the default instruction on every keystroke; we'll debounce
  };

  const commitDefaultInstruction = () => {
    saveConfig(config);
  };

  // ── Fullscreen Editor ──
  const openFullscreen = (type: string, name?: string) => {
    if (type === 'default') {
      setFullscreenContent(config.ai.default_instruction);
    } else if (type === 'custom' && name) {
      setFullscreenContent(customPrompts[name] || '');
    }
    setFullscreen({ type, name });
  };

  const saveFullscreen = () => {
    if (!fullscreen) return;
    if (fullscreen.type === 'default') {
      const newConfig = { ...config, ai: { ...config.ai, default_instruction: fullscreenContent } };
      setConfig(newConfig);
      saveConfig(newConfig);
    } else if (fullscreen.type === 'custom' && fullscreen.name) {
      const updated = { ...customPrompts, [fullscreen.name]: fullscreenContent };
      const newConfig = { ...config, ai: { ...config.ai, custom_prompts: updated } };
      setConfig(newConfig);
      saveConfig(newConfig);
    }
    setFullscreen(null);
  };

  if (loading || !config) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-accent" />
      </div>
    );
  }

  const aiProvider = config.ai.provider;
  const promptNames = Object.keys(customPrompts);

  return (
    <>
    {/* ═══ FULLSCREEN EDITOR OVERLAY ═══ */}
    {fullscreen && (
      <div className="fixed inset-0 z-50 bg-background/95 backdrop-blur-md flex flex-col animate-in fade-in zoom-in-95 duration-200">
        <div className="flex items-center justify-between px-6 py-4 border-b border-border bg-card/80">
          <div className="flex items-center gap-3">
            <Sparkles className="w-5 h-5 text-accent" />
            <div>
              <h2 className="text-lg font-bold">
                {fullscreen.type === 'default' ? 'Prompt Padrão do Sistema' : fullscreen.name}
              </h2>
              <p className="text-xs text-muted-foreground">Editor em tela cheia — edite e salve seu prompt.</p>
            </div>
          </div>
          <div className="flex gap-2">
            <Button onClick={saveFullscreen} className="bg-accent hover:bg-accent/90">
              <Save className="w-4 h-4 mr-2" />
              Salvar
            </Button>
            <Button variant="ghost" onClick={() => setFullscreen(null)}>
              <X className="w-4 h-4" />
            </Button>
          </div>
        </div>
        <div className="flex-1 p-6 overflow-hidden">
          <textarea
            value={fullscreenContent}
            onChange={(e) => setFullscreenContent(e.target.value)}
            className="w-full h-full bg-secondary/10 border border-border rounded-lg p-6 font-mono text-sm leading-relaxed resize-none focus:outline-none focus:ring-2 focus:ring-accent/50 text-foreground placeholder:text-muted-foreground"
            placeholder="Escreva seu prompt aqui..."
            autoFocus
          />
        </div>
      </div>
    )}

    <div className="space-y-6 max-w-5xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 pb-12">
      <div className="flex justify-between items-center mb-4">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Prompt Studio</h2>
          <p className="text-muted-foreground mt-1">Gerencie prompts de IA e configure o processamento de texto.</p>
        </div>
        {savedStatus && (
          <div className="flex items-center gap-2 text-accent animate-in fade-in zoom-in-95">
            <CheckCircle2 className="w-4 h-4" />
            <span className="text-sm font-medium">Salvo!</span>
          </div>
        )}
      </div>

      {/* ═══ PROMPT ATIVO ═══ */}
      <Card className="bg-card/50 backdrop-blur border-accent/30 overflow-hidden">
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <Sparkles className="w-5 h-5 text-accent" />
            <CardTitle>Prompt Ativo</CardTitle>
          </div>
          <CardDescription>
            Este é o prompt que a IA usará para processar cada transcrição.
            {activePromptName ? (
              <span className="ml-1 text-accent font-medium">Usando: "{activePromptName}"</span>
            ) : (
              <span className="ml-1 text-muted-foreground">Usando: Prompt Padrão</span>
            )}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="relative group">
            <div className="p-4 rounded-lg bg-accent/5 border border-accent/20 text-sm leading-relaxed text-foreground/80 max-h-40 overflow-y-auto font-mono">
              {getActivePromptContent()}
            </div>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => openFullscreen(activePromptName ? 'custom' : 'default', activePromptName || undefined)}
              className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Maximize2 className="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* ═══ PROMPT PADRÃO (editável, com reset) ═══ */}
      <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Zap className="w-5 h-5 text-yellow-400" />
              <CardTitle className="text-base">Prompt Padrão do Sistema</CardTitle>
            </div>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant="ghost"
                onClick={resetDefault}
                className="text-xs text-muted-foreground hover:text-accent"
              >
                <RotateCcw className="w-3 h-3 mr-1" />
                Restaurar Original
              </Button>
              <Button
                size="sm"
                variant={activePromptName === '' ? 'default' : 'outline'}
                onClick={() => selectPrompt('')}
                className={activePromptName === '' ? 'bg-accent text-accent-foreground hover:bg-accent/90' : ''}
              >
                {activePromptName === '' ? <CheckCircle2 className="w-3 h-3 mr-1" /> : null}
                {activePromptName === '' ? 'Ativo' : 'Usar Este'}
              </Button>
            </div>
          </div>
          <CardDescription>O prompt original. Pode ser editado e restaurado a qualquer momento.</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="relative group">
            <Textarea
              value={config.ai.default_instruction}
              onChange={(e) => saveDefaultInstruction(e.target.value)}
              onBlur={commitDefaultInstruction}
              className="min-h-[120px] bg-background/50 border-border font-mono text-xs leading-relaxed resize-y"
              placeholder="Instruções para a IA processar o texto transcrito..."
            />
            <Button
              size="sm"
              variant="ghost"
              onClick={() => openFullscreen('default')}
              className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity"
              title="Editar em tela cheia"
            >
              <Maximize2 className="w-4 h-4" />
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* ═══ CRIAR NOVO PROMPT ═══ */}
      <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <Plus className="w-5 h-5 text-accent" />
            <CardTitle className="text-base">Criar Novo Prompt</CardTitle>
          </div>
          <CardDescription>Crie variações do prompt para diferentes necessidades (formal, casual, técnico etc).</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Input
            value={newPromptName}
            onChange={(e) => setNewPromptName(e.target.value)}
            placeholder="Nome do prompt (ex: Formal, Resumido, Técnico...)"
            className="bg-background/50 border-border"
          />
          <Textarea
            value={newPromptContent}
            onChange={(e) => setNewPromptContent(e.target.value)}
            placeholder="Conteúdo do prompt... (deixe vazio para clonar o padrão)"
            className="min-h-[80px] bg-background/50 border-border font-mono text-xs resize-y"
          />
          <Button
            onClick={addPrompt}
            disabled={!newPromptName.trim()}
            className="w-full bg-accent hover:bg-accent/90"
          >
            <Plus className="w-4 h-4 mr-2" />
            Criar Prompt
          </Button>
        </CardContent>
      </Card>

      {/* ═══ GALERIA DE PROMPTS ═══ */}
      {promptNames.length > 0 && (
        <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Seus Prompts ({promptNames.length})</CardTitle>
            <CardDescription>Clique para ativar ou editar. O prompt ativo é usado em todas as transcrições.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {promptNames.map((name) => {
              const isActive = activePromptName === name;
              const isEditing = editingPrompt === name;
              return (
                <div
                  key={name}
                  className={`p-4 rounded-lg border transition-all ${
                    isActive
                      ? 'border-accent/50 bg-accent/10 shadow-[0_0_15px_rgba(163,230,53,0.1)]'
                      : 'border-border bg-secondary/20 hover:border-accent/20'
                  }`}
                >
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      {isActive && <CheckCircle2 className="w-4 h-4 text-accent" />}
                      <span className="font-medium text-sm">{name}</span>
                    </div>
                    <div className="flex gap-1">
                      {!isActive && (
                        <Button size="sm" variant="ghost" onClick={() => selectPrompt(name)} className="text-xs">
                          Ativar
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          if (isEditing) {
                            saveEditedPrompt(name);
                          } else {
                            setEditingPrompt(name);
                            setEditContent(customPrompts[name]);
                          }
                        }}
                      >
                        {isEditing ? <Save className="w-3 h-3" /> : <span className="text-xs">Editar</span>}
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => openFullscreen('custom', name)}
                        title="Tela cheia"
                      >
                        <Maximize2 className="w-3 h-3" />
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => deletePrompt(name)}
                        className="text-red-500 hover:text-red-400"
                      >
                        <Trash2 className="w-3 h-3" />
                      </Button>
                    </div>
                  </div>
                  {isEditing ? (
                    <Textarea
                      value={editContent}
                      onChange={(e) => setEditContent(e.target.value)}
                      className="min-h-[80px] bg-background/50 border-border font-mono text-xs resize-y"
                    />
                  ) : (
                    <p className="text-xs text-muted-foreground font-mono line-clamp-3 leading-relaxed">
                      {customPrompts[name]}
                    </p>
                  )}
                </div>
              );
            })}
          </CardContent>
        </Card>
      )}

      {/* ═══ AI FORMATTER CONFIG (movido do ConfigTab) ═══ */}
      <Card className="bg-card/50 backdrop-blur border-border overflow-hidden">
        <CardHeader>
          <CardTitle>AI Formatter</CardTitle>
          <CardDescription>IA responsável por formatar e processar comandos complexos do texto.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex items-center justify-between p-4 rounded-lg border border-accent/30 bg-accent/5">
            <div className="space-y-0.5">
              <Label className="text-base font-semibold text-accent cursor-pointer" htmlFor="auto-formatting">Auto-Improve (Correção por IA)</Label>
              <p className="text-sm text-muted-foreground">Melhora gramática e remove vícios de linguagem automaticamente de TUDO que for transcrito.</p>
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
                <Input value={config.ai.omniroute?.url || ''} onChange={(e) => updateAiProviderConfig('omniroute', 'url', e.target.value)} placeholder="http://cloud.omniroute.online/v1" className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Bearer Token (API Key)</Label>
                <Input type="password" value={config.ai.omniroute?.api_key || ''} onChange={(e) => updateAiProviderConfig('omniroute', 'api_key', e.target.value)} placeholder="seu-api-key-aqui" className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Modelo Alvo</Label>
                <Input value={config.ai.omniroute?.model || ''} onChange={(e) => updateAiProviderConfig('omniroute', 'model', e.target.value)} placeholder="openai/text-embedding-3-small..." className="bg-background/50 border-border" />
              </div>
              <p className="text-xs text-muted-foreground mt-2">API 100% compatível com OpenAI. Encaminha para 60+ provedores.</p>
            </div>
          )}

          {aiProvider === 'ollama' && (
            <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
              <div className="space-y-2">
                <Label>Endpoint URL</Label>
                <Input value={config.ai.ollama?.url || ''} onChange={(e) => updateAiProviderConfig('ollama', 'url', e.target.value)} placeholder="http://127.0.0.1:11434" className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Opcional: API Key (Bearer Token)</Label>
                <Input type="password" value={config.ai.ollama?.api_key || ''} onChange={(e) => updateAiProviderConfig('ollama', 'api_key', e.target.value)} placeholder="Caso use o Ollama via NGINX/Proxy" className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Modelo</Label>
                <Input value={config.ai.ollama?.model || ''} onChange={(e) => updateAiProviderConfig('ollama', 'model', e.target.value)} placeholder="llama3, gemma2..." className="bg-background/50 border-border" />
              </div>
            </div>
          )}

          {aiProvider === 'openai' && (
            <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
              <div className="space-y-2">
                <Label>API Key</Label>
                <Input type="password" value={config.ai.openai?.api_key || ''} onChange={(e) => updateAiProviderConfig('openai', 'api_key', e.target.value)} placeholder="sk-..." className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Modelo</Label>
                <Input value={config.ai.openai?.model || ''} onChange={(e) => updateAiProviderConfig('openai', 'model', e.target.value)} placeholder="gpt-4o-mini" className="bg-background/50 border-border" />
              </div>
            </div>
          )}

          {aiProvider === 'gemini' && (
            <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
              <div className="space-y-2">
                <Label>Google API Key</Label>
                <Input type="password" value={config.ai.gemini?.api_key || ''} onChange={(e) => updateAiProviderConfig('gemini', 'api_key', e.target.value)} placeholder="AIza..." className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Modelo</Label>
                <Input value={config.ai.gemini?.model || ''} onChange={(e) => updateAiProviderConfig('gemini', 'model', e.target.value)} placeholder="gemini-1.5-flash" className="bg-background/50 border-border" />
              </div>
              <p className="text-xs text-muted-foreground mt-2">Obtenha sua API key em makersuite.google.com/app/apikey</p>
            </div>
          )}

          {aiProvider === 'groq' && (
            <div className="animate-in fade-in slide-in-from-top-2 space-y-4 p-4 rounded-md border border-border bg-secondary/10">
              <div className="space-y-2">
                <Label>Groq API Key</Label>
                <Input type="password" value={config.ai.groq?.api_key || ''} onChange={(e) => updateAiProviderConfig('groq', 'api_key', e.target.value)} placeholder="gsk_..." className="bg-background/50 border-border" />
              </div>
              <div className="space-y-2">
                <Label>Modelo</Label>
                <Input value={config.ai.groq?.model || ''} onChange={(e) => updateAiProviderConfig('groq', 'model', e.target.value)} placeholder="llama-3.3-70b-versatile" className="bg-background/50 border-border" />
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
    </>
  );
}
