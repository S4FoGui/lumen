use std::collections::HashMap;

use crate::config::DictionaryEntryData;

/// Dicionário customizado para correção de termos específicos.
/// Permite que profissionais de qualquer área ensinem ao Lumen
/// termos técnicos e nomes próprios com grafias corretas.
pub struct CustomDictionary {
    /// Mapa de palavra_transcrita (lowercase) → correção
    entries: HashMap<String, String>,
}

impl CustomDictionary {
    /// Cria um novo dicionário com as entradas do config
    pub fn new(entries: HashMap<String, DictionaryEntryData>) -> Self {
        tracing::info!("Carregadas {} entradas no dicionário customizado", entries.len());
        // Normalizar chaves para lowercase
        let normalized: HashMap<String, String> = entries
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v.value))
            .collect();
        Self { entries: normalized }
    }

    /// Recarrega o dicionário a partir de novas entradas da configuração.
    pub fn reload(&mut self, entries: HashMap<String, DictionaryEntryData>) {
        tracing::info!("🔄 Recarregando dicionário customizado ({} entradas)", entries.len());
        self.entries = entries
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v.value))
            .collect();
    }

    /// Aplica as correções do dicionário ao texto.
    /// Substitui ocorrências case-insensitive das chaves pelos valores.
    pub fn apply(&self, text: &str) -> String {
        if self.entries.is_empty() || text.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();

        for (wrong, correct) in &self.entries {
            // Substituição case-insensitive usando regex com word boundaries
            // Escape caracteres especiais regex e cria pattern case-insensitive
            let escaped = regex::escape(wrong);
            let pattern_str = format!(r"(?i)\b{}\b", escaped);

            if let Ok(regex) = regex::Regex::new(&pattern_str) {
                result = regex.replace_all(&result, correct.as_str()).to_string();
            }
        }

        result
    }

    /// Adiciona uma entrada ao dicionário
    pub fn add(&mut self, key: String, value: String) {
        tracing::info!("Dicionário: '{}' → '{}'", key, value);
        self.entries.insert(key.to_lowercase(), value);
    }

    /// Remove uma entrada do dicionário
    pub fn remove(&mut self, key: &str) -> bool {
        self.entries.remove(&key.to_lowercase()).is_some()
    }

    /// Lista todas as entradas
    pub fn list(&self) -> &HashMap<String, String> {
        &self.entries
    }

    /// Retorna entries owned (para serialização)
    pub fn entries_owned(&self) -> HashMap<String, String> {
        self.entries.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_corrections() {
        let mut entries = HashMap::new();
        entries.insert("javascript".into(), crate::config::DictionaryEntryData { value: "JavaScript".into(), context: None, icon_type: None });
        entries.insert("react".into(), crate::config::DictionaryEntryData { value: "React".into(), context: None, icon_type: None });

        let dict = CustomDictionary::new(entries);
        assert_eq!(
            dict.apply("eu uso javascript e react"),
            "eu uso JavaScript e React"
        );
    }

    #[test]
    fn test_empty_text() {
        let dict = CustomDictionary::new(HashMap::new());
        assert_eq!(dict.apply(""), "");
    }
}
