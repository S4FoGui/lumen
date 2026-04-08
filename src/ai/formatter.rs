use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Provider de IA suportado
#[derive(Debug, Clone, PartialEq)]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Gemini,
    Groq,
    OmniRoute,
    Disabled,
}

/// Formatador de texto via IA.
/// Envia o texto transcrito para um LLM para formatação inteligente.
pub struct AiFormatter {
    client: reqwest::Client,
    provider: AiProvider,
    // Ollama
    ollama_url: String,
    ollama_key: String,
    ollama_model: String,
    // OpenAI
    openai_key: String,
    openai_model: String,
    // Gemini
    gemini_key: String,
    gemini_model: String,
    // Groq
    groq_key: String,
    groq_model: String,
    // OmniRoute
    omniroute_url: String,
    omniroute_key: String,
    omniroute_model: String,
    // Instrução padrão
    default_instruction: String,
}

// --- Request/Response structs ---

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

impl AiFormatter {
    /// Cria um novo formatador de IA.
    pub fn new(
        provider_str: &str,
        ollama_url: &str,
        ollama_key: &str,
        ollama_model: &str,
        openai_key: &str,
        openai_model: &str,
        gemini_key: &str,
        gemini_model: &str,
        groq_key: &str,
        groq_model: &str,
        omniroute_url: &str,
        omniroute_key: &str,
        omniroute_model: &str,
        default_instruction: &str,
    ) -> Self {
        let provider = match provider_str.to_lowercase().as_str() {
            "ollama" => AiProvider::Ollama,
            "openai" => AiProvider::OpenAi,
            "gemini" => AiProvider::Gemini,
            "groq" => AiProvider::Groq,
            "omniroute" => AiProvider::OmniRoute,
            _ => AiProvider::Disabled,
        };

        if provider != AiProvider::Disabled {
            tracing::info!("IA configurada: {:?}", provider);
        }

        Self {
            client: reqwest::Client::new(),
            provider,
            ollama_url: ollama_url.to_string(),
            ollama_key: ollama_key.to_string(),
            ollama_model: ollama_model.to_string(),
            openai_key: openai_key.to_string(),
            openai_model: openai_model.to_string(),
            gemini_key: gemini_key.to_string(),
            gemini_model: gemini_model.to_string(),
            groq_key: groq_key.to_string(),
            groq_model: groq_model.to_string(),
            omniroute_url: omniroute_url.to_string(),
            omniroute_key: omniroute_key.to_string(),
            omniroute_model: omniroute_model.to_string(),
            default_instruction: default_instruction.to_string(),
        }
    }

    /// Formata o texto usando o LLM configurado.
    /// `raw`: texto bruto transcrito.
    /// `instruction`: instrução específica (None = usar instrução padrão).
    /// Retorna o texto formatado pelo LLM.
    pub async fn format_text(&self, raw: &str, instruction: Option<&str>) -> Result<String> {
        if self.provider == AiProvider::Disabled {
            return Ok(raw.to_string());
        }

        let instruction = instruction.unwrap_or(&self.default_instruction);
        let prompt = format!(
            "Instrução: {}\n\nTexto para processar:\n{}",
            instruction, raw
        );

        tracing::debug!("Enviando texto para {:?} ({} chars)", self.provider, raw.len());

        let result = match self.provider {
            AiProvider::Ollama => self.call_ollama(&prompt).await,
            AiProvider::OpenAi => self.call_openai(&prompt, instruction).await,
            AiProvider::Gemini => self.call_gemini(&prompt).await,
            AiProvider::Groq => self.call_groq(&prompt, instruction).await,
            AiProvider::OmniRoute => self.call_omniroute(&prompt, instruction).await,
            AiProvider::Disabled => unreachable!(),
        };

        match result {
            Ok(formatted) => {
                tracing::debug!("Resposta IA: \"{}\"", formatted);
                Ok(formatted)
            }
            Err(e) => {
                tracing::warn!("Falha na IA, retornando texto original: {}", e);
                Ok(raw.to_string())
            }
        }
    }

    /// Retorna se a IA está ativa
    pub fn is_enabled(&self) -> bool {
        self.provider != AiProvider::Disabled
    }

    // --- Provider implementations ---

    async fn call_groq(&self, _prompt: &str, instruction: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: self.groq_model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".into(),
                    content: instruction.to_string(),
                },
                OpenAiMessage {
                    role: "user".into(),
                    content: _prompt.to_string(),
                },
            ],
            temperature: 0.3,
        };

        tracing::debug!(payload = %serde_json::to_string(&request).unwrap_or_default(), "Payload Groq");
        let response = self.client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.groq_key))
            .json(&request)
            .send()
            .await
            .context("Falha ao conectar com Groq")?;

        let text = response.text().await.context("Falha ao ler resposta Groq")?;
        tracing::debug!(response = %text, "Resposta bruta do Groq");
        
        let body: OpenAiResponse = serde_json::from_str(&text)
            .map_err(|e| {
                tracing::error!("Erro de parsing no Groq. Resposta recebida: {}", text);
                anyhow::anyhow!("Falha ao parsear resposta Groq: {}. Corpo: {}", e, text)
            })?;

        body.choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .context("Resposta Groq vazia")
    }

    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.ollama_model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let mut req = self.client.post(format!("{}/api/generate", self.ollama_url.trim_end_matches('/')));
        
        if !self.ollama_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.ollama_key));
        }

        let response = req
            .json(&request)
            .send()
            .await
            .context("Falha ao conectar com Ollama")?;

        let body: OllamaResponse = response.json().await
            .context("Falha ao parsear resposta Ollama")?;

        Ok(body.response.trim().to_string())
    }

    async fn call_openai(&self, _prompt: &str, instruction: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: self.openai_model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".into(),
                    content: instruction.to_string(),
                },
                OpenAiMessage {
                    role: "user".into(),
                    content: _prompt.to_string(),
                },
            ],
            temperature: 0.3,
        };

        let url = if self.openai_key.starts_with("gsk_") {
            "https://api.groq.com/openai/v1/chat/completions"
        } else {
            "https://api.openai.com/v1/chat/completions"
        };

        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.openai_key))
            .json(&request)
            .send()
            .await
            .context("Falha ao conectar com Provedor (OpenAI/Groq)")?;

        let body: OpenAiResponse = response.json().await
            .context("Falha ao parsear resposta OpenAI")?;

        body.choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .context("Resposta OpenAI vazia")
    }

    async fn call_gemini(&self, prompt: &str) -> Result<String> {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.gemini_model, self.gemini_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Falha ao conectar com Gemini")?;

        let body: GeminiResponse = response.json().await
            .context("Falha ao parsear resposta Gemini")?;

        body.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.trim().to_string())
            .context("Resposta Gemini vazia")
    }

    async fn call_omniroute(&self, _prompt: &str, instruction: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: self.omniroute_model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".into(),
                    content: instruction.to_string(),
                },
                OpenAiMessage {
                    role: "user".into(),
                    content: _prompt.to_string(),
                },
            ],
            temperature: 0.3,
        };

        // Usa a URL base provida. O OmniRoute é 100% compatível com a interface do OpenAI.
        let url = format!("{}/chat/completions", self.omniroute_url.trim_end_matches('/'));

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.omniroute_key))
            .json(&request)
            .send()
            .await
            .context("Falha ao conectar com OmniRoute")?;

        let body: OpenAiResponse = response.json().await
            .context("Falha ao parsear resposta OmniRoute")?;

        body.choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .context("Resposta OmniRoute vazia")
    }
}
