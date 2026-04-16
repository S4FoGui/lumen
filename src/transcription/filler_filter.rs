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

    /// Test 1: Transcription Pipeline Processing
    /// Simulates the full filter chain: raw text → filler removal → clean output
    /// Validates that TranscriptionRecord-like output has expected fields.
    #[test]
    fn test_pipeline_filler_filter_produces_clean_record() {
        // Simulate configurable filler words (same as default.yaml)
        let fillers: Vec<String> = vec![
            "humm".into(), "ééé".into(), "ãhh".into(), "uhh".into(),
            "hmm".into(), "uhm".into(), "eh".into(), "ah".into(),
            "tipo assim".into(), "né".into(), "então".into(), "bom".into(),
            "um".into(), "uh".into(),
        ];
        let filter = FillerFilter::new(&fillers);

        // Case 1: Portuguese text with multiple fillers
        let raw = "Eu humm quero tipo assim dizer que ééé isso é um teste né";
        let processed = filter.filter(raw);
        assert!(!processed.contains("humm"), "Filler 'humm' should be removed");
        assert!(!processed.contains("tipo assim"), "Filler 'tipo assim' should be removed");
        assert!(!processed.contains("ééé"), "Filler 'ééé' should be removed");
        assert!(processed.contains("quero"), "Real words must be preserved");
        assert!(processed.contains("dizer"), "Real words must be preserved");
        assert!(processed.contains("teste"), "Real words must be preserved");

        // Case 2: All fillers → result should be empty or whitespace-only
        let raw_all_fillers = "humm ééé uhh hmm";
        let processed_empty = filter.filter(raw_all_fillers);
        assert!(processed_empty.trim().is_empty(), "All-filler text should produce empty output");

        // Case 3: No fillers → text should pass through unchanged
        let raw_clean = "Teste de transcrição limpa sem fillers";
        let processed_clean = filter.filter(raw_clean);
        assert_eq!(processed_clean, raw_clean, "Clean text should pass through unchanged");

        // Case 4: Punctuation cleanup
        let raw_punct = "Sim uhh , claro uhh . Muito bom";
        let processed_punct = filter.filter(raw_punct);
        assert!(processed_punct.contains("Sim,"), "Orphan comma should be cleaned");
        assert!(processed_punct.contains("claro."), "Orphan period should be cleaned");

        // Case 5: Validate simulated TranscriptionRecord fields
        let word_count = processed.split_whitespace().count();
        assert!(word_count > 0, "Word count must be > 0 for non-empty transcription");
        assert!(word_count < raw.split_whitespace().count(), "Processed text should have fewer words than raw (fillers removed)");
    }
}

