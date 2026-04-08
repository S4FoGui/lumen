use regex::Regex;

/// Filtro de palavras de preenchimento (fillers) e limpeza de texto.
/// Remove "ums", "ahs", "ééés" e outros vícios de fala do texto transcrito.
pub struct FillerFilter {
    /// Regex compilada para detectar fillers
    pattern: Regex,
    /// Regex para limpar espaços duplicados
    spaces_pattern: Regex,
}

impl FillerFilter {
    /// Cria um novo filtro com a lista de palavras de preenchimento.
    /// `filler_words`: lista de palavras/frases a serem removidas.
    pub fn new(filler_words: &[String]) -> Self {
        // Construir regex que match fillers como palavras inteiras
        // Usa word boundaries (\b) para não remover substrings
        let escaped: Vec<String> = filler_words
            .iter()
            .map(|w| regex::escape(w.trim()))
            .filter(|w| !w.is_empty())
            .collect();

        let pattern_str = if escaped.is_empty() {
            // Regex que nunca faz match
            r"[^\s\S]".to_string()
        } else {
            format!(r"(?i)\b(?:{})\b", escaped.join("|"))
        };

        let pattern = Regex::new(&pattern_str)
            .unwrap_or_else(|_| Regex::new(r"[^\s\S]").unwrap());

        let spaces_pattern = Regex::new(r"\s{2,}").unwrap();

        Self {
            pattern,
            spaces_pattern,
        }
    }

    /// Filtra as palavras de preenchimento do texto.
    /// Também limpa espaços duplicados e pontuação órfã.
    pub fn filter(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }

        // Remover fillers
        let cleaned = self.pattern.replace_all(text, "");

        // Limpar espaços duplicados
        let cleaned = self.spaces_pattern.replace_all(&cleaned, " ");

        // Limpar pontuação órfã (", ," → ",")
        let cleaned = cleaned
            .replace(" ,", ",")
            .replace(" .", ".")
            .replace(" !", "!")
            .replace(" ?", "?");

        cleaned.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filler_removal() {
        let fillers = vec![
            "humm".into(), "ééé".into(), "uhh".into(), "hmm".into(),
        ];
        let filter = FillerFilter::new(&fillers);

        assert_eq!(
            filter.filter("Eu humm quero dizer ééé que isso"),
            "Eu quero dizer que isso"
        );
    }

    #[test]
    fn test_empty_input() {
        let filter = FillerFilter::new(&["um".into()]);
        assert_eq!(filter.filter(""), "");
    }

    #[test]
    fn test_no_fillers() {
        let filter = FillerFilter::new(&[]);
        assert_eq!(filter.filter("Texto limpo"), "Texto limpo");
    }

    #[test]
    fn test_orphan_punctuation() {
        let filter = FillerFilter::new(&["uhh".into()]);
        assert_eq!(
            filter.filter("Sim uhh , claro"),
            "Sim, claro"
        );
    }
}
