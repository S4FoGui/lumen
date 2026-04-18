import { useState } from 'react';
import {
  Activity, Settings, BarChart3, HelpCircle, Sparkles
} from 'lucide-react';

import { useLumenSocket } from './hooks/useLumenSocket';
import { DashboardTab } from './components/tabs/DashboardTab';
import { ConfigTab } from './components/tabs/ConfigTab';
import { HistoryTab } from './components/tabs/HistoryTab';
import { FaqTab } from './components/tabs/FaqTab';
import { SnippetsTab } from './components/tabs/SnippetsTab';
import { WaveformVisualizer } from './components/Waveform';

export default function App() {
  const [activeTab, setActiveTab] = useState('status');
  
  // Conecta ao WebSocket do backend Rust!
  const { status, rms, isConnected, lastTranscription, lastError } = useLumenSocket();

  return (
    <div className="flex h-screen bg-background text-foreground overflow-hidden">
      {/* Sidebar Luxuosa */}
      <aside className="w-64 border-r border-border bg-card shadow-2xl flex flex-col z-10">
        <div className="p-6 flex items-center gap-3 border-b border-border">
          <div className="relative flex items-center justify-center w-10 h-10 rounded-full bg-accent/20 border border-accent/50 shadow-[0_0_15px_rgba(163,230,53,0.3)]">
            <img src="/favicon.png" alt="Lumen" className={`w-full h-full rounded-full object-cover ${status.is_recording ? 'animate-pulse opacity-50' : ''}`} />
            {status.is_recording && (
               <div className="absolute top-0 right-0 w-2.5 h-2.5 bg-red-500 rounded-full animate-ping" />
            )}
            {!status.is_recording && isConnected && (
               <div className="absolute top-0 right-0 w-2.5 h-2.5 bg-accent rounded-full" />
            )}
          </div>
          <div>
            <h1 className="text-xl font-bold tracking-tighter">LUMEN</h1>
            <p className="text-[10px] text-muted-foreground uppercase tracking-widest">{isConnected ? 'WS Connected' : 'Disconnected'}</p>
          </div>
        </div>
        
        <nav className="flex-1 p-4 space-y-2">
          {[
            { id: 'status', label: 'Ecosystem', icon: Activity },
            { id: 'config', label: 'Params', icon: Settings },
            { id: 'history', label: 'Timeline', icon: BarChart3 },
            { id: 'snippets', label: 'Prompt Studio', icon: Sparkles },
            { id: 'faq', label: 'Guide', icon: HelpCircle }
          ].map((item) => (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm transition-all duration-200 ${
                activeTab === item.id 
                  ? 'bg-accent/10 border border-accent/30 text-accent font-medium shadow-[0_4px_20px_rgba(163,230,53,0.05)]' 
                  : 'text-muted-foreground hover:bg-secondary hover:text-foreground'
              }`}
            >
              <item.icon className="w-4 h-4" />
              {item.label}
            </button>
          ))}
        </nav>
        
        <div className="p-4 border-t border-border">
          <div className="text-xs text-center text-muted-foreground">Version {status.version}</div>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 p-8 overflow-y-auto bg-gradient-to-br from-background to-secondary/30 relative">
        {!isConnected && (
            <div className="absolute top-0 left-0 w-full h-1 bg-red-500/50 animate-pulse" />
        )}

        {/* Overlay global de escuta — visível em qualquer aba enquanto gravando */}
        {status.is_recording && (
          <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2 px-6 py-4 rounded-2xl bg-card/90 backdrop-blur border border-accent/40 shadow-[0_0_30px_rgba(163,230,53,0.15)] animate-in fade-in slide-in-from-bottom-4 duration-300">
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-red-500 animate-ping" />
              <span className="text-sm font-semibold text-accent uppercase tracking-widest">Ouvindo...</span>
            </div>
            <div className="w-48">
              <WaveformVisualizer rms={rms} isRecording={status.is_recording} />
            </div>
          </div>
        )}

        {/* Notificação de aviso (API key não configurada, etc.) */}
        {lastError && (
          <div className="fixed top-4 right-4 z-50 max-w-sm px-4 py-3 rounded-lg bg-yellow-500/10 border border-yellow-500/40 text-yellow-300 text-sm shadow-lg animate-in fade-in slide-in-from-top-2 duration-300">
            <span className="font-semibold">Aviso: </span>{lastError}
          </div>
        )}

        {/* Tab Routing */}
        {activeTab === 'status' && (
          <DashboardTab status={status} rms={rms} lastTranscription={lastTranscription} />
        )}

        {activeTab === 'config' && <ConfigTab />}



        {activeTab === 'history' && <HistoryTab />}

        {activeTab === 'faq' && <FaqTab />}

        {activeTab === 'snippets' && <SnippetsTab />}
      </main>
    </div>
  );
}
