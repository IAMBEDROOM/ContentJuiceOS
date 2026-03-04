# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ContentJuiceOS is an all-in-one content creator operating system — a Tauri desktop app with two major modules:
- **Live Suite**: Streaming command center (alerts, overlays, multi-platform chat, OBS integration, analytics)
- **Creator Studio**: Content production toolkit (voice cloning, transcription, captioning, video editor)

License: GPL-3.0. Targets Windows, macOS, Linux.

## Tech Stack

- **Backend**: Rust (Tauri core, embedded HTTP server via Actix-web/Axum, FFmpeg orchestration)
- **Frontend**: React + TypeScript (strict mode)
- **Database**: SQLite (via rusqlite or sqlx, with migration system)
- **Real-time**: Socket.IO server for overlay/alert communication and future mobile support
- **Media**: FFmpeg bundled with the app for all video/audio processing
- **External APIs**: Twitch EventSub/Helix, YouTube Data/Live API, Kick WebSocket, ElevenLabs (voice cloning), OpenAI Whisper (transcription)

## Build & Development Commands

Once scaffolded (Phase 1, Task 1.1-1.2):
```bash
# Frontend
npm install
npm run dev          # Tauri dev mode with hot reload
npm run build        # Production build
npm run lint         # ESLint + Prettier

# Rust backend
cargo clippy         # Rust linting
cargo fmt            # Rust formatting
cargo test           # Rust unit tests

# Tauri
npx tauri dev        # Full app dev mode
npx tauri build      # Production desktop build
```

CI runs lint + build on push via GitHub Actions.

## Architecture

### Communication Flow
```
React Frontend <--Tauri Commands--> Rust Backend
                                      |
                              Local HTTP Server (localhost)
                                      |
                              Socket.IO Server
                                      |
                              OBS Browser Sources
```

The Rust backend runs a local HTTP server that serves overlay/alert HTML pages as browser sources for OBS. Socket.IO handles real-time event delivery to those browser sources. The frontend communicates with the backend via Tauri commands (IPC).

### Module Structure
Both Live Suite and Creator Studio share a common foundation: asset library, database layer, settings system, and Socket.IO communication. Media imported in one module is available in the other.

### Key Design Decisions
- **OBS-native**: All visual outputs are standard browser source URLs — no OBS plugins required
- **Platform-agnostic**: Twitch, YouTube, Kick are first-class citizens with unified abstractions
- **Local-first**: All data stored locally in SQLite. No telemetry without explicit opt-in. Tokens encrypted via OS keychain (SQLite fallback)
- **Mobile-ready architecture**: Socket.IO namespaces (`/overlays`, `/control`) designed for future companion mobile app
- **API keys proxied through Rust backend** to prevent frontend exposure

### Data Layer
- SQLite database in Tauri app data directory with migration-based schema versioning
- Media files stored in a managed directory (configurable location) with DB references
- Automatic rolling database backups (configurable interval, default: daily)

## Design Language

Dark-first palette for long creative sessions:
| Token | Hex | Role |
|---|---|---|
| Deep Space Black | #0A0D14 | Primary background |
| Charcoal Navy | #151A26 | Surface/card background |
| Crisp Off-White | #E6EDF3 | Primary text |
| Electric Cyan | #00E5FF | Primary accent (buttons, active links) |
| Hyper Magenta | #FF007F | Secondary accent (notifications, errors) |
| Voltage Yellow | #FFD600 | Highlights, tertiary |

## Development Phases

The project follows a 7-phase, 152-task build order documented in `.docs/contentjuiceos-build-order.md`. Phases execute sequentially with tasks having explicit dependency chains:

1. **Foundation & Core Infrastructure** (4-6 wks) — Tauri scaffold, SQLite, local server, platform auth, FFmpeg
2. **Alert System & Visual Editor** (6-8 wks) — Asset library, visual design editor, animation engine, alert pipeline
3. **Unified Chat & Chat Bot** (4-6 wks) — Multi-platform chat, moderation, built-in bot
4. **Stream Management & Platform Sync** (3-4 wks) — OBS WebSocket integration, analytics
5. **Themes, Polish & Live Suite Release** (4-6 wks) — Theme system, onboarding, auto-updates
6. **Creator Studio — Transcription, Captions, Voice** (5-7 wks) — Speech-to-text, caption editor, voice cloning
7. **Creator Studio — Video Editor** (5-7 wks) — Timeline, trimming, export engine

## Important Files

- `.docs/contentjuiceos-project-plan.md` — Full project plan with architecture, security, and API strategy (CONFIDENTIAL)
- `.docs/contentjuiceos-build-order.md` — 152 tasks across 7 phases with dependencies (CONFIDENTIAL)
- `README.md` — Public project overview
