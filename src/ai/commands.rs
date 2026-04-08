/// Tipos de comandos de voz detectáveis no texto transcrito.
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceCommand {
    /// "envie", "mande", "envia" → Pressionar Enter para enviar
    Send,
    /// "apague", "delete", "limpe" → Apagar o texto digitado
    Delete,
    /// "torne mais profissional", "reescreva formal" → Transformação com IA
    Transform { instruction: String },
    /// "nova linha", "enter" → Inserir quebra de linha no texto
    NewLine,
    /// Nenhum comando detectado — texto normal
    None,
}

/// Detector de comandos de voz no texto transcrito.
///
/// Analisa o texto após transcrição para identificar intenções
/// do usuário que não são texto literal, mas comandos.
///
/// Exemplos:
/// - "Olá, como vai? Envie" → texto "Olá, como vai?" + comando Send
/// - "Torne mais profissional" → comando Transform
/// - "Texto normal sem comando" → VoiceCommand::None
pub struct CommandDetector {
    send_triggers: Vec<String>,
    delete_triggers: Vec<String>,
    transform_prefixes: Vec<String>,
    newline_triggers: Vec<String>,
}

impl CommandDetector {
    /// Cria um detector com os triggers padrão em português e inglês.
    pub fn new() -> Self {
        Self {
            send_triggers: vec![
                "envie".into(),
                "envia".into(),
                "mande".into(),
                "manda".into(),
                "pode enviar".into(),
                "send".into(),
                "enviar".into(),
            ],
            delete_triggers: vec![
                "apague".into(),
                "apaga".into(),
                "delete".into(),
                "limpe".into(),
                "limpa".into(),
                "apague tudo".into(),
            ],
            transform_prefixes: vec![
                "torne".into(),
                "reescreva".into(),
                "reformule".into(),
                "mude o tom".into(),
                "mais profissional".into(),
                "mais formal".into(),
                "mais informal".into(),
                "mais amigável".into(),
                "corrija".into(),
                "resuma".into(),
            ],
            newline_triggers: vec![
                "nova linha".into(),
                "próxima linha".into(),
                "próximo parágrafo".into(),
                "pula linha".into(),
                "new line".into(),
            ],
        }
    }

    /// Analisa o texto transcrito e identifica comandos de voz.
    ///
    /// Retorna `(texto_limpo, comando_detectado)`:
    /// - Se um comando de envio é detectado no final, remove o trigger e retorna Send
    /// - Se o texto inteiro é um comando de transformação, retorna Transform
    /// - Se nenhum comando é detectado, retorna o texto original com None
    pub fn detect(&self, text: &str) -> (String, VoiceCommand) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return (String::new(), VoiceCommand::None);
        }

        let lower = trimmed.to_lowercase();

        // 1. Verificar se o texto inteiro é um comando de delete
        for trigger in &self.delete_triggers {
            if lower == *trigger {
                return (String::new(), VoiceCommand::Delete);
            }
        }

        // 2. Verificar se o texto inteiro é um comando de transformação
        for prefix in &self.transform_prefixes {
            if lower.starts_with(prefix) {
                return (String::new(), VoiceCommand::Transform {
                    instruction: trimmed.to_string(),
                });
            }
        }

        // 3. Verificar se o texto TERMINA com um comando de envio
        //    Ex: "Olá mundo, envie" → texto "Olá mundo" + Send
        for trigger in &self.send_triggers {
            // Verificar final exato (com possível pontuação/espaço)
            let patterns = [
                format!(" {}", trigger),      // "texto envie"
                format!(", {}", trigger),      // "texto, envie"
                format!(". {}", trigger),      // "texto. envie"
            ];

            for pattern in &patterns {
                if lower.ends_with(pattern) {
                    let clean = trimmed[..trimmed.len() - pattern.len()]
                        .trim()
                        .trim_end_matches(|c: char| c == ',' || c == '.' || c == ';' || c == ':')
                        .trim()
                        .to_string();
                    if !clean.is_empty() {
                        return (clean, VoiceCommand::Send);
                    }
                }
            }

            // Verificar se o texto inteiro é só o trigger (sem texto antes)
            if lower == *trigger {
                return (String::new(), VoiceCommand::Send);
            }
        }

        // 4. Substituir triggers de nova linha inline
        //    Ex: "Primeiro parágrafo nova linha segundo parágrafo"
        let mut result = trimmed.to_string();
        let mut found_newline = false;
        for trigger in &self.newline_triggers {
            let lower_result = result.to_lowercase();
            if let Some(start) = lower_result.find(trigger.as_str()) {
                // Substituir trigger por \n (case-insensitive)
                let end = start + trigger.len();
                result = format!("{}\n{}", result[..start].trim(), result[end..].trim());
                found_newline = true;
            }
        }
        if found_newline {
            return (result, VoiceCommand::NewLine);
        }

        // 5. Nenhum comando detectado
        (trimmed.to_string(), VoiceCommand::None)
    }
}

impl Default for CommandDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_command() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("Olá, como vai você?");
        assert_eq!(text, "Olá, como vai você?");
        assert_eq!(cmd, VoiceCommand::None);
    }

    #[test]
    fn test_send_command_at_end() {
        let detector = CommandDetector::new();

        let (text, cmd) = detector.detect("Olá mundo, envie");
        assert_eq!(text, "Olá mundo");
        assert_eq!(cmd, VoiceCommand::Send);

        let (text, cmd) = detector.detect("Boa tarde. mande");
        assert_eq!(text, "Boa tarde");
        assert_eq!(cmd, VoiceCommand::Send);
    }

    #[test]
    fn test_send_only() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("envie");
        assert_eq!(text, "");
        assert_eq!(cmd, VoiceCommand::Send);
    }

    #[test]
    fn test_delete_command() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("apague");
        assert_eq!(text, "");
        assert_eq!(cmd, VoiceCommand::Delete);
    }

    #[test]
    fn test_transform_command() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("Torne mais profissional");
        assert_eq!(text, "");
        match cmd {
            VoiceCommand::Transform { instruction } => {
                assert_eq!(instruction, "Torne mais profissional");
            }
            _ => panic!("Deveria ser Transform"),
        }
    }

    #[test]
    fn test_newline_command() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("Primeiro parágrafo nova linha segundo parágrafo");
        assert_eq!(text, "Primeiro parágrafo\nsegundo parágrafo");
        assert_eq!(cmd, VoiceCommand::NewLine);
    }

    #[test]
    fn test_empty_text() {
        let detector = CommandDetector::new();
        let (text, cmd) = detector.detect("");
        assert_eq!(text, "");
        assert_eq!(cmd, VoiceCommand::None);
    }
}
