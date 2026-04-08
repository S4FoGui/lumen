import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Terminal, Mic, Wand2, KeyRound } from 'lucide-react';

export function FaqTab() {
  return (
    <div className="space-y-6 max-w-4xl mx-auto animate-in fade-in slide-in-from-bottom-4 duration-500 pb-12">
      <div className="flex items-center gap-4 mb-8">
        <div className="p-3 bg-accent/20 rounded-full border border-accent/30 text-accent">
          <Terminal className="w-8 h-8" />
        </div>
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Guia & Tutorial</h2>
          <p className="text-muted-foreground">Como dominar o Lumen e aumentar sua velocidade.</p>
        </div>
      </div>

      <div className="space-y-6">
        {/* Passo 1 */}
        <Card className="bg-card/50 backdrop-blur border-border/50 hover:border-accent/30 transition-colors">
          <CardHeader className="flex flex-row items-start gap-4">
            <Mic className="w-6 h-6 text-accent mt-1" />
            <div>
              <CardTitle>1. Como gravar a voz (Captura de Foco)</CardTitle>
              <CardDescription>O núcleo offline do Lumen</CardDescription>
            </div>
          </CardHeader>
          <CardContent className="pl-[3.25rem] space-y-4 text-sm text-foreground/80">
            <p>O Lumen roda de forma global no seu PC. Isso significa que ele vai injetar o texto onde o seu cursor estiver (seja no Chrome, no VSCode ou no Telegram).</p>
            <ul className="list-disc list-inside space-y-1">
              <li>Pressione <kbd className="bg-muted px-1.5 py-0.5 rounded-md text-xs font-mono ml-1 mr-1 border border-border">Enter 2x rapidamente</kbd> para ativar o microfone a qualquer momento.</li>
              <li>A barra (Pill) translúcida vai surgir no topo da sua tela escrito "READY TO LISTEN".</li>
              <li>Fale naturalmente! Quando terminar de falar, nosso mecanismo inteligente <strong>Voice Activity Detector (VAD)</strong> vai perceber o silêncio automaticamente e parar a gravação sem você ter que clicar em nada.</li>
            </ul>
          </CardContent>
        </Card>

        {/* Passo 2 */}
        <Card className="bg-card/50 backdrop-blur border-border/50 hover:border-accent/30 transition-colors">
          <CardHeader className="flex flex-row items-start gap-4">
            <Wand2 className="w-6 h-6 text-accent mt-1" />
            <div>
              <CardTitle>2. Comandos Mágicos</CardTitle>
              <CardDescription>O que falar primeiro modifica o final.</CardDescription>
            </div>
          </CardHeader>
          <CardContent className="pl-[3.25rem] space-y-4 text-sm text-foreground/80">
            <p>Se as opções de "Voice Commands" estiverem ativas, o sistema reage dinamicamente às primeiras palavras que você diz.</p>
            <div className="bg-secondary/30 border border-border/50 rounded-lg p-3 grid gap-2">
              <div className="flex items-center gap-2">
                <span className="font-bold text-accent min-w-24">"Lumen, apague"</span>
                <span>Cancela a transcrição atual sem deletar ou gerar texto.</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-accent min-w-24">"Lumen, envie"</span>
                <span>Transcreve tudo após essa palavra e dá ENTER no final.</span>
              </div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-accent min-w-24">"Torne profissional"</span>
                <span>Envia para a Inteligência Artificial o que você falou depois e reformula totalmente o parágrafo de forma séria (AI Formatter necessário).</span>
              </div>
            </div>
            <p>Não quer usar os comandos ativadores? Fale direto! O seu texto transcrito cru vai cair na tela num piscar de olhos.</p>
          </CardContent>
        </Card>

        {/* Passo 3 */}
        <Card className="bg-card/50 backdrop-blur border-border/50 hover:border-accent/30 transition-colors">
          <CardHeader className="flex flex-row items-start gap-4">
            <KeyRound className="w-6 h-6 text-accent mt-1" />
            <div>
              <CardTitle>3. Adicionando Inteligência (OmniRoute / OpenAI)</CardTitle>
              <CardDescription>Como fazer o motor IA funcionar</CardDescription>
            </div>
          </CardHeader>
          <CardContent className="pl-[3.25rem] space-y-4 text-sm text-foreground/80">
            <p>Para aplicar "reformatações inteligentes" (ex: "torne isso profissional"), o Lumen precisa de um serviço externo que rode o modelo.</p>
            <ol className="list-decimal list-inside space-y-2">
              <li>Vá até a guia <strong>Params</strong> (segundo menu).</li>
              <li>Desça até <strong>AI Formatter</strong>.</li>
              <li>Recomendação Gratuita: Escolha <strong>OmniRoute</strong> e insira o Endpoint do gateway aberto e sua API Key, com os modelos super potentes da Anthropic ou Meta.</li>
              <li>Recomendação 100% Offline e Privada: Escolha <strong>Ollama</strong> e conecte um modelo Llama que esteja rodando direto da placa de vídeo do seu computador (http://localhost:11434).</li>
            </ol>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
