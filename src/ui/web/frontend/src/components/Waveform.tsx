import { useEffect, useRef } from 'react';

type WaveformProps = {
  rms: number;
  isRecording: boolean;
};

export function WaveformVisualizer({ rms, isRecording }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const phaseRef = useRef(0);
  const smoothedRmsRef = useRef(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;

    const render = () => {
      // Limpar canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Suavizar a entrada de RMS para evitar saltos bruscos na animação
      smoothedRmsRef.current += (rms - smoothedRmsRef.current) * 0.2;
      
      const width = canvas.width;
      const height = canvas.height;
      const centerY = height / 2;
      
      // Ganho dinâmico: mais sensível para pequenos volumes
      const activityLevel = isRecording ? Math.min(smoothedRmsRef.current * 12, 1.2) : 0;
      
      // Incrementar fase para movimento contínuo
      phaseRef.current += 0.05 + activityLevel * 0.1;

      // Desenhar 3 camadas de ondas
      const drawWave = (
        opacity: number, 
        amplitudeMult: number, 
        freqMult: number, 
        phaseOffset: number,
        lineWidth: number
      ) => {
        ctx.beginPath();
        ctx.strokeStyle = `rgba(163, 230, 53, ${opacity})`;
        ctx.lineWidth = lineWidth;
        ctx.lineJoin = 'round';
        ctx.lineCap = 'round';

        // Efeito de Glow sutil
        if (isRecording) {
            ctx.shadowBlur = 15 * activityLevel;
            ctx.shadowColor = 'rgba(163, 230, 53, 0.4)';
        }

        for (let x = 0; x <= width; x += 2) {
          // Curva de sino para manter as pontas finas e o meio volumoso
          const normalizePosition = x / width;
          const bellCurve = Math.sin(normalizePosition * Math.PI);
          
          // Equação da onda: sen(x * freq + fase)
          // Adicionamos variação baseada no activityLevel
          const wave = Math.sin(normalizePosition * Math.PI * 2 * freqMult + phaseRef.current + phaseOffset);
          
          // Amplitude depende do volume e da posição (bell curve)
          const amplitude = (height * 0.4) * activityLevel * amplitudeMult * bellCurve;
          
          const y = centerY + wave * amplitude;

          if (x === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();
        ctx.shadowBlur = 0; // Reset shadow
      };

      if (isRecording) {
        // Camada 3 (Fundo, lenta, larga)
        drawWave(0.15, 0.6, 0.8, phaseRef.current * 0.5, 2);
        // Camada 2 (Meio, média)
        drawWave(0.3, 0.8, 1.2, -phaseRef.current * 0.3, 3);
        // Camada 1 (Frente, rápida, detalhada)
        drawWave(0.9, 1.0, 1.5, phaseRef.current, 4);
      } else {
        // Linha base estática pulsante sutil
        ctx.beginPath();
        ctx.strokeStyle = 'rgba(163, 230, 53, 0.2)';
        ctx.lineWidth = 2;
        ctx.moveTo(0, centerY);
        ctx.lineTo(width, centerY);
        ctx.stroke();
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
        width={400} 
        height={64} 
        className="w-full h-full max-w-sm"
      />
    </div>
  );
}
