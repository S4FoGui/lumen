# 🎙️ Plano de Atualização: Lumen UI Redesign (B.L.A.S.T. / A.N.T.)

## 🎯 1. Visão Geral (North Star)
**Objetivo Singular:** Implementar o novo design premium e "dark mode" focado em produtividade para o Lumen, baseando-se nas referências visuais propostas. Isso abrange o redesign do **Dictionary Manager** no painel web, e a atualização da **Floating Window (Overlay)** de captação de voz.

## 🏗️ 2. Blueprint do Projeto

### Fase A: Dictionary Manager (Web Dashboard)
O painel de Dicionário em `src/ui/web/static/` (HTML/CSS/JS) sofrerá um redesign completo.
- **Nova Paleta e Tipografia:** Destaque para a cor `Lime Green` e background escuro estruturado em "Glassmorphism" ou "Surface solid", adotando a tipografia Inter em pesos dinâmicos.
- **Layout Assíncrono:** Dividiremos a view em duas colunas principais. A coluna da esquerda (maior) para inserção de dados e listagem de atalhos ativos. A coluna da direita exclusiva para a métrica (ex: _Transcription Accuracy 99.4%_).
- **Cards do Dicionário (Substitutions):** Expansão do modelo de dado, agora incluindo formato horizontal agrupado, campo descritivo `Context: ...`, e ícones dinâmicos nas laterais indicando a área (Tech, AI, GPU).

### Fase B: Overlay de Gravação (Floating UI)
A janela de ação principal (`overlay.rs`) deverá corresponder às referências da pill-shape UI flutuante.
- **Visual:** Capsule shape de bordas ultra-arredondadas em fundo deep dark. Efeito de glow/sombra projetada sutil.
- **Logo Area:** O novo símbolo de Lumen ("L" central) em Lime Green ativo.
- **Waveform:** Inclusão de visualização de "Onda Sonora" ativada por microfone, transmitindo feedback imediato à percepção de áudio.
- **Labeling Ativo:** Texto instrucional sutil "READY TO LISTEN" ou "CONFIGURAÇÕES" com linhas guias modernas apontando para os elementos principais.

## 📦 3. Data-First Rationale (Esboço do Schema)
A mudança na UI impactará nos modulos em Rust, transformando entradas chave-valor atuais em estruturas elaboradas, pois o design agora prevê "Contexto".

```rust
// Modificação em src/config.rs (Necessita Confirmação)
pub struct DictionaryEntry {
    pub key: String,
    pub replacement: String,
    pub context: Option<String>,
    pub category_icon: Option<String>,
}
```

## ⚙️ 4. A.N.T. 3-Layer Execution
- **Layer 1 (Architecture):** Atualizações nos Manuais (este documento e `.mds` criados).
- **Layer 2 (Navitgation/State):** O backend *axum* precisará serializar/desserializar o Dicionário em novo formato (JSON) a partir do `config.yaml`.
- **Layer 3 (Tools):** Edição isolada e determinística do arquivo `style.css`, DOM em `index.html`, e por fim reescrever a render GTK4 no `overlay.rs`.

---

**⚠️ Ação Bloqueada:** Conforme Regra de Inicialização do Piloto Autônomo, solicito que responda às **5 Perguntas de Descoberta** (ver Chat) para que este Blueprint e os Schemas ganhem a aprovação formal (Payload final).