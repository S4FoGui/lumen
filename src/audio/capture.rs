use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Captura de áudio do microfone usando cpal.
/// Grava em 16kHz mono f32 (formato requerido pelo Whisper).
pub struct AudioCapture {
    /// Buffer de amostras acumuladas durante a gravação
    samples: Arc<Mutex<Vec<f32>>>,
    /// Stream de áudio ativo (Some quando gravando)
    stream: Option<cpal::Stream>,
    /// Configuração do dispositivo selecionado
    device_name: Option<String>,
    /// Taxa de amostragem alvo
    target_sample_rate: u32,
    /// Se a supressão de ruído (RNNoise) está ativa
    noise_suppression: bool,
}

impl AudioCapture {
    /// Cria uma nova instância de captura de áudio.
    /// `device_name`: None para o dispositivo padrão do sistema.
    /// `sample_rate`: Taxa de amostragem alvo (16000 para Whisper).
    pub fn new(device_name: Option<String>, sample_rate: u32, noise_suppression: bool) -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            device_name,
            target_sample_rate: sample_rate,
            noise_suppression,
        }
    }

    /// Atualiza o dispositivo de áudio (efeito no próximo start())
    pub fn set_device(&mut self, device_name: Option<String>) {
        self.device_name = device_name;
    }

    /// Inicia a gravação do microfone.
    /// Retorna erro se o dispositivo não for encontrado ou não suportar o formato.
    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();

        // Selecionar dispositivo
        let device = match &self.device_name {
            Some(name) => {
                let devices = host.input_devices()
                    .context("Falha ao listar dispositivos de entrada")?;
                #[allow(deprecated)]
                devices
                    .into_iter()
                    .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                    .with_context(|| format!("Dispositivo '{}' não encontrado", name))?
            }
            None => host
                .default_input_device()
                .context("Nenhum dispositivo de entrada padrão encontrado")?,
        };

        #[allow(deprecated)]
        let device_name = device.name().unwrap_or_else(|_| "desconhecido".into());
        tracing::info!("Usando dispositivo de áudio: {}", device_name);

        // Usar a config nativa do dispositivo (evita falhas ALSA com sample rates não suportados)
        let default_config = device.default_input_config()
            .context("Falha ao obter configuração padrão do dispositivo")?;
        
        let native_sample_rate = default_config.sample_rate();
        let native_channels = default_config.channels();
        tracing::info!("📊 Config nativa do dispositivo: {}Hz, {} canais", native_sample_rate, native_channels);
        
        let config = cpal::StreamConfig {
            channels: native_channels,
            sample_rate: native_sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        // Limpar buffer
        {
            if let Ok(mut samples) = self.samples.lock() {
                samples.clear();
            }
        }

        let samples_clone = Arc::clone(&self.samples);
        let target_rate = self.target_sample_rate;
        let src_channels = native_channels as usize;
        let src_rate = native_sample_rate;

        let mut denoiser = if self.noise_suppression {
            tracing::info!("✨ Supressão de ruído IA ativa (RNNoise)");
            Some(nnnoiseless::DenoiseState::new())
        } else {
            None
        };
        
        // Buffer para acumular amostras para o RNNoise (precisa de 480 amostras @ 48kHz)
        let mut rnnoise_accumulation = Vec::new();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // 1. Downmix para mono se necessário
                    let mono: Vec<f32> = if src_channels > 1 {
                        data.chunks(src_channels)
                            .map(|frame| frame.iter().sum::<f32>() / src_channels as f32)
                            .collect()
                    } else {
                        data.to_vec()
                    };
                    
                    if let Some(ref mut ds) = denoiser {
                        // MODALIDADE 1: Com Supressão de Ruído (RNNoise)
                        // A) Resample nativo -> 48kHz (RNNoise fixo)
                        let mono_48k = if src_rate != 48000 {
                            resample(&mono, src_rate, 48000)
                        } else {
                            mono
                        };

                        rnnoise_accumulation.extend_from_slice(&mono_48k);

                        // B) Processar frames de 480 amostras
                        while rnnoise_accumulation.len() >= 480 {
                            let mut input_frame = [0.0f32; 480];
                            input_frame.copy_from_slice(&rnnoise_accumulation[..480]);
                            rnnoise_accumulation.drain(..480);

                            let mut output_frame = [0.0f32; 480];
                            ds.process_frame(&mut output_frame, &input_frame);

                            // C) Resample 48kHz -> 16kHz (Target Whisper)
                            let denoised_16k = resample(&output_frame, 48000, target_rate);
                            
                            if let Ok(mut samples) = samples_clone.lock() {
                                samples.extend_from_slice(&denoised_16k);
                            }
                        }
                    } else {
                        // MODALIDADE 2: Sem Supressão (Direto)
                        let resampled = if src_rate != target_rate {
                            resample(&mono, src_rate, target_rate)
                        } else {
                            mono
                        };
                        
                        if let Ok(mut samples) = samples_clone.lock() {
                            samples.extend_from_slice(&resampled);
                        }
                    }
                },
                move |err| {
                    tracing::error!("Erro na captura de áudio: {}", err);
                },
                None,
            )
            .context("Falha ao criar stream de áudio")?;

        stream.play().context("Falha ao iniciar stream de áudio")?;
        tracing::info!("🎙️ Gravação iniciada");

        self.stream = Some(stream);
        Ok(())
    }

    /// Para a gravação e retorna as amostras capturadas.
    /// As amostras são em formato f32, 16kHz, mono.
    pub fn stop(&mut self) -> Result<Vec<f32>> {
        // Drop the stream to stop recording
        self.stream.take();
        tracing::info!("🛑 Gravação parada");

        let samples = {
            if let Ok(mut buffer) = self.samples.lock() {
                std::mem::take(&mut *buffer)
            } else {
                Vec::new()
            }
        };

        tracing::info!("Capturadas {} amostras ({:.1}s de áudio)",
            samples.len(),
            samples.len() as f64 / self.target_sample_rate as f64
        );

        Ok(samples)
    }

// is_recording removido (unused)

    /// Lista todos os dispositivos de entrada disponíveis com nomes bonitos
    pub fn list_devices() -> Result<Vec<(String, String)>> {
        let host = cpal::default_host();
        let devices = host.input_devices()
            .context("Falha ao listar dispositivos de entrada")?;

        // Mapear Nomes bonitos do ALSA no Linux via /proc/asound/cards
        let mut alsa_names = std::collections::HashMap::new();
        if let Ok(cards) = std::fs::read_to_string("/proc/asound/cards") {
            for line in cards.lines() {
                if line.contains('[') && line.contains(']') && line.contains(':') {
                    let parts: Vec<&str> = line.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        let left = parts[0];
                        let right = parts[1];
                        if let Some(start) = left.find('[') {
                            if let Some(end) = left.find(']') {
                                let card_code = left[start+1..end].trim().to_string();
                                let mut card_name = right.trim().to_string();
                                if let Some(dash) = card_name.find(" - ") {
                                    card_name = card_name[dash+3..].to_string();
                                }
                                alsa_names.insert(card_code, card_name);
                            }
                        }
                    }
                }
            }
        }

        let mut categorized_alsa: std::collections::HashMap<String, (String, String, i32)> = std::collections::HashMap::new();
        let mut final_list: Vec<(String, String)> = Vec::new();

        #[allow(deprecated)]
        for d in devices.filter_map(|d| d.name().ok()) {
            let id = d.clone();
            
            if id.starts_with("dmix") || id.starts_with("dsnoop") || id == "sysdefault" {
                continue;
            }

            let mut is_alsa = false;
            let mut current_card_code = String::new();

            if id.starts_with("hw:CARD=") || id.starts_with("plughw:CARD=") || id.starts_with("default:CARD=") || id.starts_with("sysdefault:CARD=") || id.starts_with("front:CARD=") || id.starts_with("dsnoop:CARD=") {
                is_alsa = true;
                let parts: Vec<&str> = id.split(',').collect();
                if let Some(card_part) = parts.first() {
                    current_card_code = card_part.split('=').nth(1).unwrap_or("").to_string();
                }
            }

            if is_alsa {
                let score = if id.starts_with("default:CARD=") { 4 }
                           else if id.starts_with("plughw:CARD=") { 3 }
                           else if id.starts_with("sysdefault:CARD=") { 2 }
                           else if id.starts_with("hw:CARD=") { 1 }
                           else { 0 };

                let mut label = id.clone();
                if let Some(pretty) = alsa_names.get(&current_card_code) {
                    let prefix = id.split(':').next().unwrap_or("");
                    label = format!("{} ({})", pretty, prefix);
                }

                if let Some((_, _, best_score)) = categorized_alsa.get(&current_card_code) {
                    if score > *best_score {
                        categorized_alsa.insert(current_card_code.clone(), (id.clone(), label, score));
                    }
                } else {
                    categorized_alsa.insert(current_card_code.clone(), (id.clone(), label, score));
                }
            } else {
                final_list.push((id.clone(), id.clone()));
            }
        }

        // Add the best ALSA devices to the final list
        for (_, (id, label, _)) in categorized_alsa {
            final_list.push((id, label));
        }

        // Sort for deterministic UI
        final_list.sort_by(|a, b| a.1.cmp(&b.1));

        Ok(final_list)
    }

    /// Retorna uma referência ao buffer de samples compartilhado.
    /// Usado pelo VAD para monitorar áudio em tempo real durante gravação.
    pub fn samples_ref(&self) -> Arc<Mutex<Vec<f32>>> {
        Arc::clone(&self.samples)
    }
}

/// Helper local para resampling linear
fn resample(input: &[f32], src_rate: u32, target_rate: u32) -> Vec<f32> {
    if src_rate == target_rate {
        return input.to_vec();
    }
    let ratio = target_rate as f64 / src_rate as f64;
    let out_len = (input.len() as f64 * ratio) as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;
        let sample = if idx + 1 < input.len() {
            input[idx] * (1.0 - frac as f32) + input[idx + 1] * frac as f32
        } else if idx < input.len() {
            input[idx]
        } else {
            0.0
        };
        out.push(sample);
    }
    out
}

