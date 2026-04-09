import { useEffect, useRef } from 'react';

type WaveformProps = {
  rms: number;
  isRecording: boolean;
};

export function WaveformVisualizer({ rms, isRecording }: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const targetRms = useRef(0);
  const currentRms = useRef(0);
  const phase = useRef(0);

  // Atualiza o alvo sempre que o prop mudar
  useEffect(() => {
    targetRms.current = rms;
  }, [rms]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;

    const render = () => {
      const width = canvas.width;
      const height = canvas.height;
      
      // Interpolação suave (LERP) para o volume (input)
      currentRms.current += (targetRms.current - currentRms.current) * 0.15;
      const intensity = isRecording ? Math.max(0.2, currentRms.current * 10) : 0.2;
      
      phase.current += 0.15 + (intensity * 0.5);

      ctx.clearRect(0, 0, width, height);

      // Gradiente moderno
      const gradient = ctx.createLinearGradient(0, 0, width, 0);
      gradient.addColorStop(0, 'rgba(163, 230, 53, 0.6)');
      gradient.addColorStop(0.5, 'rgba(163, 230, 53, 1.0)');
      gradient.addColorStop(1, 'rgba(163, 230, 53, 0.6)');

      ctx.beginPath();
      ctx.strokeStyle = gradient;
      ctx.lineWidth = 3;
      ctx.lineJoin = 'round';
      ctx.lineCap = 'round';

      for (let x = 0; x <= width; x++) {
        const progress = x / width;
        const sine = Math.sin(progress * Math.PI * 3 + phase.current);
        const y = (height / 2) + (sine * intensity * 20 * Math.sin(progress * Math.PI));
        
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      
      ctx.stroke();
      animationFrameId = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrameId);
  }, [isRecording]);

  return (
    <div className="w-full h-16 bg-black/5 rounded-lg overflow-hidden border border-white/10">
      <canvas ref={canvasRef} width={400} height={64} className="w-full h-full" />
    </div>
  );
}
