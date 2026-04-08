use std::collections::HashMap;

/// Gerenciador de snippets de voz.
/// Permite inserir blocos de texto pré-definidos através de comandos curtos.
/// Exemplo: ditar "/ola" → "Olá! Tudo bem? Como posso ajudar?"
pub struct SnippetManager {
    /// Mapa de trigger → texto expandido
    entries: HashMap<String, String>,
}

impl SnippetManager {
    /// Cria um novo gerenciador com os snippets do config
    pub fn new(entries: HashMap<String, String>) -> Self {
        tracing::info!("Carregados {} snippets", entries.len());
        Self { entries }
    }

    /// Recarrega todos os snippets a partir de um novo mapa.
    pub fn reload(&mut self, entries: HashMap<String, String>) {
        tracing::info!("🔄 Recarregando snippets ({} entradas)", entries.len());
        self.entries = entries;
    }

    /// Verifica se o texto é um trigger de snippet e retorna o texto expandido.
    /// Retorna None se não for um snippet.
    pub fn expand(&self, text: &str) -> Option<&String> {
        let trimmed = text.trim().to_lowercase();
        self.entries.get(&trimmed)
    }

    /// Processa o texto, expandindo qualquer snippet encontrado.
    /// Se o texto inteiro é um trigger, retorna o snippet.
    /// Caso contrário, retorna o texto original.
    pub fn process(&self, text: &str) -> String {
        let trimmed = text.trim();

        // Verificar se o texto inteiro é um trigger
        if let Some(expanded) = self.expand(trimmed) {
            return expanded.clone();
        }

        // Verificar cada palavra (para snippets inline)
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        let mut result = Vec::new();

        for word in words {
            let lower = word.to_lowercase();
            if let Some(expanded) = self.entries.get(&lower) {
                result.push(expanded.as_str());
            } else {
                result.push(word);
            }
        }

        result.join(" ")
    }

    /// Adiciona um novo snippet
    pub fn add(&mut self, trigger: String, text: String) {
        tracing::info!("Snippet adicionado: '{}' → '{}'", trigger, text);
        self.entries.insert(trigger, text);
    }

    /// Remove um snippet
    pub fn remove(&mut self, trigger: &str) -> bool {
        if self.entries.remove(trigger).is_some() {
            tracing::info!("Snippet removido: '{}'", trigger);
            true
        } else {
            false
        }
    }

    /// Lista todos os snippets
    pub fn list(&self) -> &HashMap<String, String> {
        &self.entries
    }

    /// Retorna os entries como owned HashMap (para serialização)
    pub fn entries_owned(&self) -> HashMap<String, String> {
        self.entries.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_expand() {
        let mut entries = HashMap::new();
        entries.insert("/ola".into(), "Olá! Tudo bem?".into());

        let manager = SnippetManager::new(entries);
        assert_eq!(manager.expand("/ola"), Some(&"Olá! Tudo bem?".into()));
        assert_eq!(manager.expand("hello"), None);
    }

    #[test]
    fn test_snippet_process() {
        let mut entries = HashMap::new();
        entries.insert("/obg".into(), "Muito obrigado!".into());

        let manager = SnippetManager::new(entries);
        assert_eq!(manager.process("/obg"), "Muito obrigado!");
        assert_eq!(manager.process("texto normal"), "texto normal");
    }
}
