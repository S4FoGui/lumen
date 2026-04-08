# 🌍 Configuração de Idiomas - Lumen

**Data:** 2026-04-06  
**Projeto:** Lumen v1.0.0

---

## 🎯 Idiomas Suportados

O Lumen suporta múltiplos idiomas através do Whisper. Os principais são:

- **Português (BR):** `pt`
- **Inglês:** `en`
- **Detecção Automática:** `auto`

---

## ⚙️ Como Configurar

### Arquivo de Configuração

Edite: `~/.config/lumen/config.yaml`

```yaml
# --- Transcrição ---
transcription:
  model_path: "/home/gui/.local/share/lumen/models/ggml-small.bin"
  
  # Escolha o idioma:
  language: "pt"    # Português BR
  # language: "en"  # Inglês
  # language: "auto" # Detecção automática
```

---

## 🔄 Trocar de Idioma

### Opção 1: Editar Manualmente

```bash
# Abrir editor
nano ~/.config/lumen/config.yaml

# Mudar a linha:
language: "en"  # Para inglês
# ou
language: "pt"  # Para português
```

### Opção 2: Via Comando

```bash
# Português
sed -i 's/language: ".*"/language: "pt"/' ~/.config/lumen/config.yaml

# Inglês
sed -i 's/language: ".*"/language: "en"/' ~/.config/lumen/config.yaml

# Auto-detecção
sed -i 's/language: ".*"/language: "auto"/' ~/.config/lumen/config.yaml
```

### Opção 3: Via Dashboard Web

1. Acesse: http://localhost:8484
2. Vá em **Configurações**
3. Selecione o idioma desejado
4. Clique em **Salvar**

---

## 🎤 Comportamento por Idioma

### Português (pt)
```yaml
language: "pt"
```

**Características:**
- Transcreve apenas em português
- Remove fillers em português: "humm", "ééé", "né", "então"
- Melhor precisão para português BR
- Comandos de voz em português: "envie", "apague", "torne profissional"

**Exemplo:**
```
Entrada: "Humm, eu acho que, ééé, isso funciona, né?"
Saída: "eu acho que isso funciona"
```

---

### Inglês (en)
```yaml
language: "en"
```

**Características:**
- Transcreve apenas em inglês
- Remove fillers em inglês: "um", "uh", "you know", "like", "actually"
- Melhor precisão para inglês
- Comandos de voz em inglês: "send", "delete", "make it professional"

**Exemplo:**
```
Entrada: "Um, I think, you know, this works, like, really well"
Saída: "I think this works really well"
```

---

### Auto-detecção (auto)
```yaml
language: "auto"
```

**Características:**
- Whisper detecta automaticamente o idioma
- Funciona com qualquer idioma suportado
- Pode ser menos preciso que idioma fixo
- Remove fillers de ambos os idiomas

**Exemplo:**
```
Entrada (PT): "Olá, como vai?"
Saída: "Olá, como vai?"

Entrada (EN): "Hello, how are you?"
Saída: "Hello, how are you?"
```

---

## 🔧 Configuração Avançada

### Fillers Personalizados por Idioma

Edite `~/.config/lumen/config.yaml`:

```yaml
transcription:
  language: "pt"
  
  # Fillers para português
  filler_words:
    - "humm"
    - "ééé"
    - "ãhh"
    - "né"
    - "então"
    - "tipo assim"
    - "bom"
    
  # Para inglês, adicione:
  # filler_words:
  #   - "um"
  #   - "uh"
  #   - "you know"
  #   - "like"
  #   - "actually"
  #   - "basically"
  #   - "literally"
```

---

## 🌐 Outros Idiomas Suportados

O Whisper suporta 99+ idiomas. Alguns exemplos:

| Código | Idioma |
|--------|--------|
| `pt` | Português |
| `en` | Inglês |
| `es` | Espanhol |
| `fr` | Francês |
| `de` | Alemão |
| `it` | Italiano |
| `ja` | Japonês |
| `zh` | Chinês |
| `ru` | Russo |
| `ar` | Árabe |

Para usar outro idioma:
```yaml
language: "es"  # Espanhol
language: "fr"  # Francês
# etc...
```

---

## 🎯 Recomendações

### Para Melhor Precisão
✅ Use idioma fixo (`pt` ou `en`) se você fala apenas um idioma  
✅ Use `auto` se você alterna entre idiomas  
✅ Configure fillers específicos do seu idioma  

### Para Uso Bilíngue (PT + EN)
```yaml
language: "auto"  # Detecção automática
```

Ou crie dois perfis de configuração:
```bash
# Perfil português
cp ~/.config/lumen/config.yaml ~/.config/lumen/config-pt.yaml

# Perfil inglês
cp ~/.config/lumen/config.yaml ~/.config/lumen/config-en.yaml

# Edite cada um com o idioma correspondente
```

---

## 🧪 Testar Idiomas

### Teste em Português
```bash
./target/release/lumen
# Pressione Enter 2x
# Fale: "Olá, este é um teste em português"
# Resultado esperado: "Olá, este é um teste em português"
```

### Teste em Inglês
```bash
# Mude para inglês primeiro:
sed -i 's/language: "pt"/language: "en"/' ~/.config/lumen/config.yaml

./target/release/lumen
# Pressione Enter 2x
# Fale: "Hello, this is a test in English"
# Resultado esperado: "Hello, this is a test in English"
```

---

## 📊 Dashboard - Seletor de Idioma

O dashboard web permite trocar o idioma em tempo real:

1. Acesse: http://localhost:8484
2. Vá em **Configurações** → **Transcrição**
3. Selecione o idioma no dropdown:
   - 🇧🇷 Português (BR)
   - 🇺🇸 English
   - 🌍 Auto-detect
4. Clique em **Salvar**
5. O Lumen recarrega automaticamente

---

## 🚀 Atalho Rápido

Crie aliases para trocar rapidamente:

```bash
# Adicione ao ~/.bashrc ou ~/.zshrc

alias lumen-pt='sed -i "s/language: \".*\"/language: \"pt\"/" ~/.config/lumen/config.yaml && echo "✅ Idioma: Português"'
alias lumen-en='sed -i "s/language: \".*\"/language: \"en\"/" ~/.config/lumen/config.yaml && echo "✅ Idioma: English"'
alias lumen-auto='sed -i "s/language: \".*\"/language: \"auto\"/" ~/.config/lumen/config.yaml && echo "✅ Idioma: Auto-detect"'
```

Uso:
```bash
lumen-pt   # Muda para português
lumen-en   # Muda para inglês
lumen-auto # Muda para auto-detecção
```

---

## ✨ Conclusão

O Lumen suporta **português e inglês** nativamente, com possibilidade de usar qualquer um dos 99+ idiomas do Whisper. Configure o idioma em `~/.config/lumen/config.yaml` ou use o dashboard web para trocar em tempo real.

**Configuração atual:** Português (pt) ✅
