# 🎤 Guia Completo - Instalação e Treinamento de Voz

**Data:** 2026-04-06  
**Projeto:** Lumen v1.0.0

---

## 🔧 PARTE 1: Instalar ydotool (Debian/Ubuntu)

O ydotool é necessário para injetar texto no Wayland. Execute no terminal:

```bash
# 1. Atualizar repositórios
sudo apt update

# 2. Instalar ydotool
sudo apt install ydotool -y

# 3. Iniciar o serviço
sudo systemctl enable ydotoold
sudo systemctl start ydotoold

# 4. Adicionar seu usuário ao grupo input
sudo usermod -aG input $USER

# 5. Recarregar grupos (ou fazer logout/login)
newgrp input

# 6. Testar se funciona
ydotool type "teste"
```

### Verificar Instalação

```bash
# Verificar se está instalado
which ydotool

# Verificar se o serviço está rodando
systemctl status ydotoold

# Verificar se você está no grupo input
groups | grep input
```

---

## 🎨 PARTE 2: Recompilar o Frontend

Agora vamos adicionar a nova aba de treinamento:

```bash
# 1. Entrar na pasta do frontend
cd src/ui/web/frontend

# 2. Instalar dependência do Progress
npm install @radix-ui/react-progress

# 3. Recompilar o frontend
npm run build

# 4. Voltar para a raiz
cd ../../../..
```

---

## 🔄 PARTE 3: Reiniciar o Lumen

```bash
# 1. Parar o Lumen atual
pkill lumen

# 2. Iniciar novamente
./target/release/lumen
```

---

## 🎤 PARTE 4: Usar a Aba de Treinamento

### Acessar o Dashboard

1. Abra o navegador: **http://localhost:8484**
2. Clique na aba **"Training"** (nova aba com ícone de chapéu de formatura)

### Como Treinar

A aba de treinamento permite que você grave sua voz para melhorar o reconhecimento:

#### 📝 Frases Disponíveis (10 frases)

1. "Olá, meu nome é [seu nome] e estou testando o sistema de reconhecimento de voz."
2. "O rato roeu a roupa do rei de Roma."
3. "Três pratos de trigo para três tigres tristes."
4. "A aranha arranha a jarra, a jarra arranha a aranha."
5. "Hoje está um dia ensolarado e agradável."
6. "Preciso enviar um email importante para o cliente."
7. "Vou fazer uma reunião às três horas da tarde."
8. "O projeto está avançando conforme o planejado."
9. "Gostaria de agendar uma consulta para amanhã."
10. "Por favor, confirme o recebimento desta mensagem."

#### 🎯 Passo a Passo

1. **Clique em "Gravar"** ao lado de uma frase
2. **Leia a frase em voz alta** naturalmente
3. **Clique em "Parar"** quando terminar
4. **Repita** para pelo menos 5 frases diferentes
5. **Clique em "Enviar X gravação(ões)"** no topo

#### ✅ Dicas para Melhor Resultado

- 🎤 Use o mesmo microfone que usará no dia a dia
- 🔇 Grave em ambiente silencioso
- 🗣️ Fale naturalmente, sem forçar a pronúncia
- 📊 Grave pelo menos 5 frases (mínimo recomendado)
- 🔄 Quanto mais frases, melhor o reconhecimento

#### 📊 Barra de Progresso

A interface mostra:
- **Frases gravadas:** X / 10
- **Barra de progresso visual**
- **Indicador verde** nas frases já gravadas
- **Botão "Enviar"** fica disponível após 5 frases

---

## 🔍 PARTE 5: Testar o Reconhecimento

Após enviar as gravações:

1. Abra um editor de texto
2. Pressione **Enter 2x rapidamente**
3. Fale uma das frases que você gravou
4. Aguarde 4 segundos
5. Veja se o reconhecimento melhorou!

---

## 🐛 Troubleshooting

### ydotool não funciona

```bash
# Verificar se o serviço está rodando
sudo systemctl status ydotoold

# Reiniciar o serviço
sudo systemctl restart ydotoold

# Verificar permissões
ls -la /tmp/.ydotool_socket
```

### Frontend não recompila

```bash
# Limpar cache
cd src/ui/web/frontend
rm -rf node_modules dist
npm install
npm run build
```

### Aba Training não aparece

```bash
# Verificar se os arquivos foram criados
ls -la src/ui/web/frontend/src/components/tabs/TrainingTab.tsx
ls -la src/ui/web/frontend/src/components/ui/progress.tsx

# Verificar logs do navegador (F12 → Console)
```

### Gravação não funciona

- Verifique se o navegador tem permissão para acessar o microfone
- Teste em outro navegador (Firefox, Chrome)
- Verifique se o microfone está funcionando: `arecord -l`

---

## 📊 Como Funciona o Treinamento

### Frontend (React)

1. **Captura de Áudio:** `navigator.mediaDevices.getUserMedia()`
2. **MediaRecorder:** Grava em formato WebM
3. **Armazenamento:** Blobs em memória
4. **Upload:** FormData via POST para `/api/training/upload`

### Backend (Rust)

1. **Recebe gravações:** Endpoint `/api/training/upload`
2. **Salva arquivos:** `~/.local/share/lumen/training/`
3. **Processa:** Extrai features de áudio
4. **Fine-tuning:** Ajusta modelo Whisper (futuro)

### Modelo Whisper

- **Atual:** Modelo pré-treinado (ggml-small.bin)
- **Futuro:** Fine-tuning com suas gravações
- **Melhoria:** Reconhecimento personalizado para sua voz

---

## 🚀 Comandos Rápidos

```bash
# Instalar tudo de uma vez
sudo apt update && \
sudo apt install ydotool -y && \
sudo systemctl enable --now ydotoold && \
sudo usermod -aG input $USER && \
cd src/ui/web/frontend && \
npm install @radix-ui/react-progress && \
npm run build && \
cd ../../../.. && \
pkill lumen && \
./target/release/lumen
```

---

## 📚 Recursos Adicionais

### Arquivos Criados

- `src/ui/web/frontend/src/components/tabs/TrainingTab.tsx` - Aba de treinamento
- `src/ui/web/frontend/src/components/ui/progress.tsx` - Componente de progresso
- `src/ui/web/frontend/src/App.tsx` - Atualizado com nova aba

### Endpoints API

```
POST /api/training/upload
  - Recebe: FormData com áudios e textos
  - Retorna: { success: true }
```

### Diretórios

```
~/.local/share/lumen/
├── models/
│   └── ggml-small.bin (modelo Whisper)
├── training/
│   ├── audio_1.webm
│   ├── audio_2.webm
│   └── ...
└── analytics.db (histórico)
```

---

## ✨ Conclusão

Agora você tem:

1. ✅ **ydotool instalado** - Injeção de texto funcionando
2. ✅ **Aba Training** - Interface para gravar sua voz
3. ✅ **10 frases** - Para treinar o modelo
4. ✅ **Progresso visual** - Acompanhar gravações
5. ✅ **Upload automático** - Enviar para processamento

**Próximo passo:** Grave pelo menos 5 frases e teste o reconhecimento! 🎤
