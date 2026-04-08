import { useEffect, useRef } from 'react';

type WaveformProps = {
  rms: number;
  isRecording: boolean;
};

export function WaveformVisualizer({ rms, isRecording }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const barsRef = useRef<number[]>(Array(32).fill(0)); // 32 barras de espectro simulado

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;

    const render = () => {
      // Limpar canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      const targetGain = isRecording ? Math.min(rms * 15, 1.0) : 0;
      
      const width = canvas.width;
      const height = canvas.height;
      const barCount = barsRef.current.length;
      const barWidth = (width / barCount) * 0.6;
      const gap = (width / barCount) * 0.4;
      
      const centerY = height / 2;

      // Desenhar cada barra
      for (let i = 0; i < barCount; i++) {
        // Gerar variação "aleatória" baseada na posição para simular frequência
        // As barras do meio tendem a ser mais altas (curva de sino)
        const normalizePosition = (i / barCount) * Math.PI;
        const bellCurve = Math.sin(normalizePosition); 
        
        const noise = Math.random() * 0.3 + 0.7; // 0.7 a 1.0 variação
        
        // Suavização (easing) para o alvo
        const targetHeight = targetGain * bellCurve * noise * (height - 10);
        
        // Mover o valor atual suavemente em direção ao valor alvo (lerp)
        barsRef.current[i] += (targetHeight - barsRef.current[i]) * 0.2;
        
        // Se isRecording for false e targetGain for 0, as barras retornam pro meio lentamente.
        const barHeight = Math.max(barsRef.current[i], 2); // Altura min de 2px
        
        const x = i * (barWidth + gap);
        const y = centerY - barHeight / 2;

        // Cor Neon Lime (accent)
        ctx.fillStyle = isRecording ? 'rgba(163, 230, 53, 0.9)' : 'rgba(163, 230, 53, 0.2)';
        
        // Glow effect
        ctx.shadowBlur = isRecording ? 10 : 0;
        ctx.shadowColor = 'rgba(163, 230, 53, 0.5)';
        
        // Desenhar a barra com borda arredondada (simulado via dois arcos e um ret)
        ctx.beginPath();
        ctx.roundRect(x, y, barWidth, barHeight, 4);
        ctx.fill();
        
        // Reset shadow
        ctx.shadowBlur = 0;
      }

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      cancelAnimationFrame(animationFrameId);
    };
  }, [rms, isRecording]);

  return (
    <div className="relative w-full h-16 flex items-center justify-center overflow-hidden rounded-xl border border-border/50 bg-secondary/20">
      {/* Background pulsante sutil se gravando */}
      {isRecording && (
        <div className="absolute inset-0 bg-accent/5 animate-pulse" />
      )}
      <canvas 
        ref={canvasRef} 
        width={300} 
        height={64} 
        className="w-full h-full max-w-sm"
      />
    </div>
  );
}
