# Changelog

All notable changes to the Lumen project will be documented in this file.

## [Unreleased]

## [1.0.1] - 2026-04-06

### Added
- Always Listening Mode: New mode that keeps Lumen listening continuously and only processes when the wake word is detected
- Wake Word Detection: Implementation of wake word detection in always listening mode
- Voice Commands: Added detection of voice commands like "envie", "apague", "nova linha"

### Changed
- Updated dashboard to properly serve CSS and JavaScript assets
- Fixed recursive async function issue that was causing build failures
- Updated hotkey references to use "Enter 2x" instead of old key combinations
- Improved static file serving in the web server for better asset loading

### Fixed
- Resolved compilation error caused by infinite recursion in async functions
- Fixed missing CSS styling in the dashboard frontend
- Corrected path resolution for static assets when running from different directories

## [1.0.0] - Initial Release

### Added
- Local Whisper-based voice transcription
- Audio cleaning to remove fillers like "humm", "ééé"
- Custom dictionary for technical terms
- Voice snippets functionality
- Lightning mode for fast transcription
- AI formatting with Ollama, OpenAI, or Gemini
- Web dashboard interface
- Global hotkeys support for X11 and Wayland