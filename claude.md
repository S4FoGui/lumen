# 📜 Project Constitution (claude.md)

## 1. Identity
- **System Pilot:** AI assistant for building deterministic, self-healing automation.
- **Protocol:** B.L.A.S.T. (Blueprint, Link, Architect, Stylize, Trigger).
- **Architecture:** A.N.T. 3-layer system.

## 2. Core Constraints
- **Data-First:** JSON Data Schema (Input/Output) defines logic.
- **Reliability > Speed:** Never guess business logic.
- **Self-Annealing:** Repair loops for failed tools.

## 3. Project Specific Rules
- Use `npx -y` for new project creation.
- Vanilla CSS/Javascript for web components.
- GTK4 / Layer-Shell for the Rust backend (Lumen).

## 4. MCP Server Management
- **Shadcn/UI MCP:** `npx -y shadcn mcp`.
- **Target:** Environment-specific configuration.

## 5. File Structure Reference
```
/lumen/
├── architecture/      # Layer 1: Technical SOPs and Architecture Docs
├── assets/            # Project assets (icons, images)
├── docs/              # Layer 1: General documentation and user guides
├── models/            # AI Models (Whisper, etc.)
├── scripts/           # Installation and maintenance scripts
├── src/               # Layer 2 & 3: Rust source code (A.N.T. layers 2 and 3)
├── tests/             # Integration tests and debug tools
│   └── scripts/       # Standalone test/debug scripts
├── tools/             # Layer 3: Utility tools and shell scripts
├── claude.md          # Project Map & State Tracking
├── gemini.md          # Project Constitution & Data Schemas
├── findings.md        # Research and Discoveries
├── progress.md        # Task Progress Tracking
└── task_plan.md       # Development Roadmap
```
