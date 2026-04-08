# SOP: Token Saver MCP Management

## Overview
The Token Saver MCP server is a technical utility designed to optimize interaction with LLMs by reducing the redundant parts of files (like long function bodies) during the discovery and navigation phase.

## Architecture (A.N.T Layer 3)
- **Engine:** Node.js (ESM).
- **Transport:** Stdio.
- **Protocol:** Model Context Protocol (v1.0.0).

## Maintenance
### Installation Directory
`/home/gui/token-saver-mcp`

### Dependency Management
- `@modelcontextprotocol/sdk`: Core protocol implementation.
- `zod`: Schema validation for tool inputs.

### Tool Logic
1. **`summarize_file`**: Uses regex-based filtering to extract structural elements (imports, signatures). Should be used for large files (>500 lines).
2. **`list_project_structure`**: Recursive directory walking with explicit ignores for build/node artifacts.
3. **`extract_relevant_code`**: Brace-depth aware extraction. If depth calculation fails, it defaults to a 100-line hard cap per block.

## Troubleshooting
- **Error: "Module not found"**: Ensure `npm install` was run in the installation directory.
- **High latency**: Check if `list_project_structure` is being called on a high-latency filesystem (network drives).
