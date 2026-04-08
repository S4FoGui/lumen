# 🎯 Plano de Ação (B.L.A.S.T.)

## Fase 1: Blueprint
- [x] Obter respostas do Discovery
- [x] Confirmar schema do Dicionário
- [x] Definir layout do Dashboard Web e GTK4 Overlay

## Fase 2: Link
- [x] Verificar comunicação entre Rust (axum) e CSS/JS

## Fase 3: Architect
- [x] Alterar `config.rs` se o dicionário passar a receber `Context`
- [x] Codificar components de UI modularizados

## Fase 4: Stylize
- [x] Aplicar estilo Lime/Dark ao HTML
- [x] Implementar a Pill UI no `overlay.rs` (Totalmente reconstruída com GTK4/Layer-Shell)
- [x] Refinar animações e responsividade

## Fase 6: MCP Integration
- [x] Configurar MCP Server `shadcn/ui`
- [x] Configurar MCP Server `convex`
- [x] Verificar conexão `/mcp`
- [x] Inicializar componentes UI via shadcn
- [x] Integrar MCP Server `brave-search` (v1.0)
- [x] Atualizar MCP Server `brave-browser` (v2.0 - CDP Support)

## Fase 7: Token Saver MCP Integration [Aguardando Blueprint]
- [ ] Blueprint: Definir schema e responder Discovery
- [ ] Link: Criar pasta e instalar dependências
- [ ] Architect: Implementar `src/index.js` e `package.json`
- [ ] Stylize: Validar ferramentas no `/mcp`
## Fase 8: v1.0.2 KDE Fixes (KDE/Wayland Stability)
- [/] Blueprint: Mapear arquivos afetados (VAD, Engine, Injector, HistoryTab)
- [ ] Link: Executar `fix_kde.sh` e validar dependências
- [ ] Architect: Aplicar patches Rust e Config
- [ ] Stylize: Atualizar HistoryTab.tsx (React)
- [ ] Trigger: Recompilar Frontend e Backend
