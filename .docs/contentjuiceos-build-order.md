

**ContentJuiceOS**

Build Order

152 Tasks  |  7 Phases  |  38 Sections

Version 2.0  |  March 2026

**CONFIDENTIAL**

# **Phase 1: Foundation & Core Infrastructure**

*19 tasks  |  Estimated 4–6 weeks*

## **Section 1A: Project Scaffolding**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **1.1** | **Tauri app scaffold** | Create the Tauri project with Rust backend (src-tauri/) and React/TypeScript frontend (src/). Standard Tauri project structure. Verify it builds and opens a window. | — |
| **1.2** | **Build pipeline** | Configure dev, build, and release scripts. TypeScript strict mode, ESLint, Prettier, Rust clippy/fmt. CI via GitHub Actions for lint \+ build on push. | 1.1 |
| **1.3** | **Shared types** | Create a src/types/ directory with TypeScript types/interfaces used across the project: platform enums, event types, config shapes, design schemas, video project schemas. | 1.1 |

## **Section 1B: Database & Configuration**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **1.4** | **SQLite integration** | Set up SQLite in the Rust backend via rusqlite or sqlx. Create the database file in the Tauri app data directory. Implement a migration system for schema versioning. | 1.1 |
| **1.5** | **Core database schema** | Define and run initial migrations: settings, platform\_connections, assets, designs, projects, voice\_profiles, cache tables. Include created\_at/updated\_at timestamps. | 1.4 |
| **1.6** | **Config system** | Build a settings manager in Rust that reads/writes user preferences to SQLite. Expose to the frontend via Tauri commands. Cover: general settings, keybindings, appearance, API keys. | 1.5 |
| **1.7** | **Database backup system** | Implement automatic rolling backups of the SQLite database file. Configurable interval (default: daily). Keep last N backups. Expose manual backup/restore via Tauri commands. | 1.5 |

## **Section 1C: Local Server & Communication**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **1.8** | **Embedded HTTP server** | Spin up a local HTTP server (Actix-web or Axum) inside the Tauri Rust backend. Serves on localhost with a configurable port. This will serve browser source HTML pages to OBS. | 1.1 |
| **1.9** | **Socket.IO server** | Integrate a Socket.IO server (Rust or sidecar Node process) for real-time internal communication. Namespaces for: /overlays, /control (future mobile). Test with a basic ping/pong from the frontend. | 1.8 |
| **1.10** | **Browser source base template** | Create a base HTML page served by the local HTTP server that connects to the Socket.IO server, listens for events, and renders content. Foundation for all overlays and alerts. | 1.9 |

## **Section 1D: Platform Authentication**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **1.11** | **Secure token storage** | Implement encrypted storage for OAuth tokens and API keys using the OS keychain (via Tauri’s tauri-plugin-keychain or similar). Fallback to encrypted SQLite if keychain is unavailable. | 1.5 |
| **1.12** | **Twitch OAuth2 flow** | Implement the full Twitch OAuth2 authorization code flow. Open browser for auth, handle redirect callback, exchange code for tokens, store encrypted. Include token refresh logic. | 1.11 |
| **1.13** | **YouTube OAuth2 flow** | Same as 1.12 but for YouTube/Google OAuth2. Handle Google’s consent screen, scopes for YouTube Data API and Live Streaming API. | 1.11 |
| **1.14** | **Kick authentication** | Implement Kick authentication. Kick’s API access may require different approaches depending on current API status. Build an abstraction that can adapt. | 1.11 |
| **1.15** | **Platform connection manager** | Unified UI page for managing platform connections. Show connected platforms with status, connect/disconnect buttons, token health indicators. Ability to revoke individual connections. | 1.12–1.14 |

## **Section 1E: API & Media Infrastructure**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **1.16** | **Rate limiter** | Build a per-platform rate limiter in Rust. Track request counts per time window. Queue non-urgent requests. Prioritise real-time operations over background tasks. | 1.1 |
| **1.17** | **Cache layer** | Implement a cache layer in SQLite for platform data (channel info, emotes, badges, categories). Configurable TTLs per data type. Event-driven invalidation hooks. | 1.5, 1.16 |
| **1.18** | **Retry & fallback system** | Build exponential backoff with jitter for transient API failures. Connection health tracking per platform. Queue outbound actions during outages for retry on reconnection. | 1.16 |
| **1.19** | **FFmpeg integration** | Bundle FFmpeg binary with the Tauri app for all target platforms. Build a Rust wrapper module that constructs and executes FFmpeg commands, manages processing queues, and reports progress via Socket.IO. | 1.1, 1.9 |

# **Phase 2: Alert System & Visual Editor**

*37 tasks  |  Estimated 6–8 weeks*

## **Section 2A: Asset Management**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.1** | **Asset storage structure** | Create the managed asset directory within the Tauri app data folder. Subdirectories for images, audio, video, fonts, animations, voice\_profiles, captions. Build Rust functions to import/copy files with unique IDs while preserving original names. | 1.5 |
| **2.2** | **Asset database schema** | Migration for the assets table: id, original\_filename, type, format, file\_size, dimensions, duration, tags, file\_path, created\_at. | 1.5 |
| **2.3** | **Asset import service** | Rust service to handle file imports: validate format and size limits, extract metadata (dimensions, duration), copy to managed directory, insert database record. | 2.1, 2.2 |
| **2.4** | **Asset library UI** | Frontend page showing all imported assets in a grid/list view. Filter by type, search by name/tag. Preview images inline, play audio on click. Import button with drag-and-drop support. | 2.3 |
| **2.5** | **Asset deletion & cleanup** | Delete assets from library: remove database record and file from disk. Handle assets in use by designs or video projects (warn user, or soft-delete with reference counting). | 2.4 |

## **Section 2B: Design Schema & Storage**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.6** | **Design data model** | Define the core design schema in shared types: a JSON-serialisable tree of design elements (text, image, shape, animation, sound). Each element has position, size, rotation, opacity, animation properties, and layer order. | 1.3 |
| **2.7** | **Design database schema** | Migration for the designs table: id, name, type (alert/overlay/scene), config (JSON blob of the design tree), thumbnail, created\_at, updated\_at. | 1.5 |
| **2.8** | **Design CRUD service** | Rust service for creating, reading, updating, deleting, and duplicating designs. JSON serialisation/deserialisation of the design tree. | 2.6, 2.7 |

## **Section 2C: Visual Editor — Core**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.9** | **Editor canvas** | Build the visual editor canvas using HTML5 Canvas or a library like Konva.js/Fabric.js. Render design elements from the design tree. Support zoom, pan, and grid snapping. | 2.6 |
| **2.10** | **Element selection & manipulation** | Click to select elements on the canvas. Drag to move, resize handles, rotation handle. Multi-select with shift-click or marquee. | 2.9 |
| **2.11** | **Properties panel** | Side panel showing properties of the selected element: position, size, rotation, opacity, colour, font, border. Changes update the design tree and re-render in real time. | 2.10 |
| **2.12** | **Layer panel** | Panel showing all elements in layer order. Drag to reorder layers. Toggle visibility and lock per element. Rename elements. | 2.10 |
| **2.13** | **Text element** | Add and edit text elements on the canvas. Properties: font family, size, weight, colour, stroke, shadow, alignment, line height. Inline editing on double-click. | 2.11 |
| **2.14** | **Image element** | Add image elements from the asset library. Properties: source, fit mode (cover/contain/stretch), border radius, border, shadow, opacity. | 2.11, 2.4 |
| **2.15** | **Shape element** | Add basic shapes: rectangle, circle, ellipse, rounded rectangle, line. Properties: fill colour, stroke colour/width, border radius, shadow, opacity. | 2.11 |
| **2.16** | **Undo/redo system** | Implement undo/redo for all editor actions using a command history stack. Keyboard shortcuts: Ctrl+Z / Ctrl+Shift+Z. | 2.10 |
| **2.17** | **Copy/paste & duplicate** | Copy/paste elements within and across designs. Duplicate shortcut. Paste with slight offset to avoid stacking. | 2.10 |
| **2.18** | **Snap & alignment guides** | Smart guides that appear when moving elements near other elements or the canvas centre/edges. Snap to grid, snap to element edges/centres. | 2.10 |

## **Section 2D: Animation System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.19** | **Animation data model** | Extend the design element schema with animation properties: entrance animation (type, duration, delay, easing), exit animation, looping animations. Define available animation types (fade, slide, scale, bounce, rotate, shake, etc.). | 2.6 |
| **2.20** | **Animation engine** | Build the animation renderer that applies CSS/JS animations to elements in the browser source based on the animation data model. Support chaining entrance → hold → exit sequences. | 2.19, 1.10 |
| **2.21** | **Animation editor UI** | Add animation controls to the properties panel. Select entrance/exit animation type from a visual picker, set duration, delay, and easing curve. Preview button. | 2.19, 2.11 |

## **Section 2E: Sound System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.22** | **Sound trigger system** | Attach audio files (from asset library) to design events: play on alert trigger, play on element entrance, loop during scene. Volume control per sound. | 2.4, 2.6 |
| **2.23** | **Sound playback in browser source** | Implement audio playback in the browser source overlay page. Receive sound trigger events via Socket.IO, play audio served by the local HTTP server. | 2.22, 1.10 |

## **Section 2F: Alert System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.24** | **Alert type definitions** | Define all supported alert types in shared types: follow, subscription (all tiers), gifted sub, raid, bits/cheers (Twitch), Super Chat/Super Sticker (YouTube), Kick equivalents. Each type has dynamic variables. | 1.3 |
| **2.25** | **Alert design templates** | Create alert-specific design templates that use dynamic variables. Alert text elements can reference variables like {username}, {amount}, {message} which are replaced at render time. | 2.24, 2.13 |
| **2.26** | **Alert configuration UI** | UI page to manage alerts: list all alert types, assign a design to each type, configure per-type settings (minimum amount for bits, enable/disable per platform, custom variations per sub tier). | 2.25, 2.8 |
| **2.27** | **Platform event listeners** | Connect to Twitch EventSub (WebSocket), YouTube Live Chat/Pub-Sub, and Kick WebSocket. Parse incoming events and normalise into the shared alert type format. Route to the alert queue. | 1.12–1.14, 2.24 |
| **2.28** | **Alert queue manager** | Receive normalised alert events. Queue with configurable behaviour: sequential (FIFO), priority-based (raids \> subs \> follows), or stack. Configurable delay between alerts. Max queue length. | 2.27 |
| **2.29** | **Alert rendering pipeline** | Pop alerts from the queue, resolve the assigned design template, replace dynamic variables, send to browser source overlay via Socket.IO. Render with entrance → hold → exit animation. | 2.28, 2.20, 1.10 |
| **2.30** | **Alert browser source page** | Dedicated browser source URL (localhost:PORT/alerts) that connects to Socket.IO, receives alert render commands, and displays them with full animation and sound. Transparent background for OBS. | 2.29, 2.23 |

## **Section 2G: Overlay & Scene System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.31** | **Overlay design type** | Overlays are persistent designs (always visible during stream). Create the overlay design type in the editor with a fixed canvas size matching common stream resolutions (1920x1080, 2560x1440). | 2.8, 2.9 |
| **2.32** | **Overlay browser source page** | Dedicated browser source URL (localhost:PORT/overlay/{id}) that renders a specific overlay design. Connects to Socket.IO for live updates. Transparent background. | 2.31, 1.10 |
| **2.33** | **Scene design type** | Scenes are full-screen designs for starting soon, BRB, ending screens. Support countdown timers as a special element type. | 2.8, 2.9 |
| **2.34** | **Scene browser source page** | Dedicated browser source URL (localhost:PORT/scene/{id}) for full-screen scenes. Renders with animations and countdown timers. | 2.33, 1.10 |
| **2.35** | **Dynamic data bindings** | Allow overlay elements to bind to live data: viewer count, follower count, latest follower name, stream uptime, current game/category. Data pushed via Socket.IO. | 2.32, 2.27 |

## **Section 2H: Preview & Testing**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **2.36** | **Platform event simulator** | Built-in tool to generate fake platform events with configurable parameters. Feeds into the same event pipeline as real events. Accessible as a dev tool and user-facing alert preview. | 2.27 |
| **2.37** | **Alert preview sandbox** | UI panel where users can trigger simulated alerts and see them render in an embedded preview (iframe of the alert browser source). Test individual alerts or rapid-fire sequences. | 2.36, 2.30 |

# **Phase 3: Unified Chat & Chat Bot**

*22 tasks  |  Estimated 4–6 weeks*

## **Section 3A: Chat Connections**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **3.1** | **Chat message data model** | Define normalised chat message type in shared types: id, platform, username, display\_name, message\_text, badges, emotes, is\_mod, is\_sub, is\_vip, timestamp, colour, reply\_to, platform\_specific\_data. | 1.3 |
| **3.2** | **Twitch IRC connection** | Connect to Twitch IRC (via WebSocket to irc-ws.chat.twitch.tv). Parse IRC messages into the normalised chat format. Handle PRIVMSG, USERNOTICE, CLEARCHAT, CLEARMSG. Reconnect on disconnect. | 1.12, 3.1 |
| **3.3** | **YouTube Live Chat connection** | Connect to YouTube Live Chat API. Poll for new messages. Parse into normalised format. Handle Super Chats and membership messages. | 1.13, 3.1 |
| **3.4** | **Kick chat connection** | Connect to Kick’s chat WebSocket. Parse messages into normalised format. Handle Kick-specific events. | 1.14, 3.1 |
| **3.5** | **Chat connection manager** | Manage chat connections for all platforms. Start/stop per platform. Health monitoring and auto-reconnect. Expose connection status to the frontend. | 3.2–3.4 |

## **Section 3B: Unified Chat UI**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **3.6** | **Chat message list** | Scrollable chat message list. Messages arrive via Socket.IO. Display username (coloured), badges, message text with emote rendering, platform indicator. Auto-scroll with scroll-pause on user scroll-up. | 3.5, 1.9 |
| **3.7** | **Emote rendering** | Parse emote codes and replace with emote images. Support Twitch, YouTube, Kick emotes. Cache locally. Support 7TV/BTTV/FFZ third-party emotes. | 3.6, 1.17 |
| **3.8** | **Badge rendering** | Display platform badges (mod, sub, VIP, broadcaster) next to usernames. Fetch badge definitions from each platform API, cache locally. | 3.6, 1.17 |
| **3.9** | **Chat input & sending** | Text input to send messages. Platform selector (send to one or all). Character limit display. Emote picker popup. | 3.6 |
| **3.10** | **Chat filters** | Filter chat by platform, user role (mods, subs), or keyword search. Filters apply in real time. | 3.6 |
| **3.11** | **User highlights** | Configure custom highlight colours for specific user roles or individual usernames. Highlighted messages get coloured background or border. | 3.6 |
| **3.12** | **Chat history storage** | Store chat messages in SQLite for searchable history. Configurable retention period (default: 30 days). Search by username, keyword, date range. | 3.6, 1.5 |

## **Section 3C: Chat Moderation**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **3.13** | **Moderation action API** | Rust service wrapping moderation actions for each platform: timeout, ban, unban, delete message, slow mode, sub-only mode, emote-only mode. Normalised interface. | 1.12–1.14 |
| **3.14** | **Moderation UI** | Right-click context menu on chat messages: timeout (30s, 1m, 10m, custom), ban, delete. Mod actions panel for channel-wide settings. Actions dispatch to the correct platform. | 3.13, 3.6 |
| **3.15** | **Mod log** | Log all moderation actions taken (who, what, when, which platform). Display in a dedicated mod log panel. | 3.14 |

## **Section 3D: Chat Bot**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **3.16** | **Chat bot engine** | Core bot engine that listens to incoming chat messages and matches against configured triggers. Runs as a service in the Rust backend. Responds via the chat send API. | 3.5, 3.9 |
| **3.17** | **Custom commands** | User-defined commands (e.g. \!discord, \!socials). Configure: trigger word, response text (with variables like {user}, {uptime}, {game}), cooldown, permission level. | 3.16 |
| **3.18** | **Timed messages** | Messages that fire automatically on a configurable interval. Configurable minimum chat activity threshold (don’t fire in a dead chat). | 3.16 |
| **3.19** | **Auto-moderation rules** | Configurable auto-mod: link filtering, excessive caps detection, spam detection, banned word list. Actions: delete, timeout, warn. Per-platform enable/disable. | 3.16, 3.13 |
| **3.20** | **Bot configuration UI** | Frontend page to manage all bot settings: command list, timed messages, auto-mod rule toggles. Live preview of command responses. | 3.17–3.19 |

## **Section 3E: Chat Overlay**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **3.21** | **Chat overlay design** | Special overlay type for displaying chat on stream. Customisable: font, size, colours, background opacity, message fade time, max visible messages, show/hide badges and platform icons. | 3.6, 2.9 |
| **3.22** | **Chat overlay browser source** | Browser source URL (localhost:PORT/chat-overlay) that renders the chat overlay. Receives messages via Socket.IO, applies design config, renders with entrance animations and fade-out. | 3.21, 1.10 |

# **Phase 4: Stream Management & Platform Sync**

*16 tasks  |  Estimated 3–4 weeks*

## **Section 4A: Multi-Platform Stream Control**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **4.1** | **Stream metadata API** | Rust service wrapping stream metadata operations for each platform: get/set title, category/game, tags. Normalised interface. Category/game mapping across platforms. | 1.12–1.14, 1.17 |
| **4.2** | **Multi-platform sync UI** | Frontend page for updating stream info across all platforms at once. Title input, category/game search with autocomplete, tag editor. "Apply to all" with per-platform overrides. | 4.1 |
| **4.3** | **Stream status dashboard** | Real-time dashboard showing live status per platform: live/offline indicator, viewer count, chat activity rate, uptime. | 4.1, 2.27 |

## **Section 4B: OBS WebSocket Integration**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **4.4** | **OBS WebSocket client** | Implement OBS WebSocket v5 client in Rust. Handle connection, authentication, and reconnection. Parse OBS events and expose to frontend via Socket.IO. | 1.9 |
| **4.5** | **OBS read operations** | Get current scene, scene list, source list, stream status (live, bitrate, dropped frames, encoding load), recording status, audio levels/mute states. Push updates in real time. | 4.4 |
| **4.6** | **OBS write operations** | Switch scene, toggle source visibility, start/stop recording, trigger studio mode transition, set source transform. Expose as Tauri commands. | 4.4 |
| **4.7** | **OBS connection UI** | Settings page for OBS WebSocket connection: host, port, password. Connection status indicator. Auto-connect on launch with saved credentials. | 4.4 |
| **4.8** | **Stream health monitor** | Dedicated panel showing real-time OBS stats: bitrate graph, CPU/encoding load, dropped frames counter, connection quality. Warning thresholds with visual alerts. | 4.5 |

## **Section 4C: Scene Control**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **4.9** | **Scene switcher UI** | Panel listing all OBS scenes with one-click switching. Current active scene highlighted. Optional preview thumbnails. Keyboard shortcut assignment per scene. | 4.5, 4.6 |
| **4.10** | **Source control panel** | Panel listing sources in the current OBS scene. Toggle visibility, adjust volume for audio sources, solo/mute controls. | 4.5, 4.6 |

## **Section 4D: Scheduling & Clips**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **4.11** | **Stream scheduler** | Schedule future streams with title, category, start time, and notes. Store in SQLite. Display upcoming schedule on the dashboard. Reminder notifications. | 1.5, 4.2 |
| **4.12** | **Clip capture trigger** | One-button clip creation that triggers the clip/highlight API on all active platforms simultaneously. Display confirmation with links to created clips. | 1.12–1.14 |

## **Section 4E: Analytics**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **4.13** | **Stream session tracking** | Track each stream session: start time, end time, peak viewers, average viewers, new followers, new subs, chat message count, per platform. Store in SQLite. | 4.3, 3.12 |
| **4.14** | **Analytics dashboard UI** | Visual dashboard with stream session history. Charts for viewer count over time, follower growth, sub trends. Compare across platforms. Filter by date range. | 4.13 |
| **4.15** | **Post-stream summary** | Auto-generate a summary after each stream ends: duration, peak/average viewers, top events, chat stats. Display as a popup or dedicated page. | 4.13 |
| **4.16** | **Platform connection status bar** | Persistent status bar showing connection status for each platform, OBS, and Socket.IO. Green/yellow/red indicators. Click to reconnect or view details. | 4.3, 4.7 |

# **Phase 5: Themes, Polish & Live Suite Release**

*18 tasks  |  Estimated 4–6 weeks*

## **Section 5A: Theme System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.1** | **Theme data model** | Define theme package format: JSON manifest (name, author, version, colour palette, font selections) \+ bundled designs (alerts, overlays, scenes) \+ optional bundled assets. | 2.6 |
| **5.2** | **Theme export** | Export current configuration as a theme package (.cjtheme zip file). Include all referenced designs and optionally bundle assets. Generate a thumbnail preview image. | 5.1, 2.8 |
| **5.3** | **Theme import** | Import a .cjtheme package: validate format, extract designs and assets, apply colour palette and fonts. Handle asset conflicts. Preview before applying. | 5.1, 2.8 |
| **5.4** | **Default themes** | Bundle 3–5 starter themes with the app (e.g. "Neon", "Minimal", "Retro", "Gradient"). Give new users a starting point. | 5.2 |

## **Section 5B: Transition Stingers**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.5** | **Stinger design type** | New design type for transition animations between scenes. Full-screen, short duration (0.5–3s). Uses the visual editor with timeline-based animation. | 2.9, 2.20 |
| **5.6** | **Stinger browser source** | Browser source URL for stinger playback. Triggered via Socket.IO command, plays once, then goes transparent. Used as an OBS stinger transition source. | 5.5, 1.10 |
| **5.7** | **Stinger trigger integration** | Connect stinger playback to OBS scene switches. Configurable per-scene-pair transitions. | 5.6, 4.6 |

## **Section 5C: Panel Designer**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.8** | **Panel design type** | Design type for Twitch/YouTube channel panels. Fixed width per platform specs. Uses the visual editor. Export as images (PNG/JPG). | 2.9 |
| **5.9** | **Panel export** | Export panel designs as image files. Batch export all panels. Correct dimensions per platform. | 5.8 |

## **Section 5D: Keyboard Shortcuts**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.10** | **Global hotkey system** | Register global hotkeys via Tauri’s global shortcut API. Predefined actions: switch OBS scene, trigger alert, toggle source, mute/unmute. | 1.6, 4.6 |
| **5.11** | **Hotkey configuration UI** | Settings page for assigning hotkeys. List of available actions, click-to-record shortcut assignment, conflict detection. | 5.10 |

## **Section 5E: Onboarding & UX**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.12** | **First-run onboarding wizard** | Multi-step wizard on first launch: welcome, connect platforms, connect OBS, create or import first overlay, set stream defaults. Skip option for experienced users. | 1.15, 4.7 |
| **5.13** | **OBS setup guide** | In-app guide for adding ContentJuiceOS browser sources to OBS. Step-by-step with screenshots/diagrams. Copy browser source URLs to clipboard. | 1.8 |
| **5.14** | **App navigation & layout** | Main app layout: top-level Live/Studio navigation split. Sidebar navigation (Dashboard, Alerts, Chat, Overlays, Scenes, Stream Mgmt, Settings). Responsive sizing. Dark theme using the ContentJuiceOS colour scheme. | All prior |
| **5.15** | **Notification system** | In-app notifications for: new follower/sub events (optional), connection status changes, update available, processing complete (Creator Studio), errors. Toast notifications \+ history panel. | 2.27, 4.16 |

## **Section 5F: Release Preparation**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **5.16** | **Auto-update system** | Configure Tauri’s built-in updater. Update manifest hosting (S3/GitHub Releases). Code-sign releases. Update check on launch with install/defer prompt. | 1.2 |
| **5.17** | **Error handling & crash recovery** | Global error boundaries in frontend. Rust panic handlers. Graceful crash recovery (restore last state, don’t lose unsaved designs or video projects). Error reporting dialog. | All prior |
| **5.18** | **Creator Studio placeholder** | Add the Creator Studio navigation entry with a "Coming Soon" landing page showing the planned features. This ensures the navigation structure is in place for Phases 6–7. | 5.14 |

# **Phase 6: Creator Studio — Transcription, Captions & Voice**

*22 tasks  |  Estimated 5–7 weeks*

## **Section 6A: Transcription Engine**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **6.1** | **Transcription service interface** | Define a Rust trait/interface for transcription backends. Input: audio/video file path. Output: timestamped transcript (word-level and segment-level). Supports swappable backends. | 1.3 |
| **6.2** | **Whisper API backend** | Implement the transcription interface using the OpenAI Whisper API. Send audio to the API, parse the response into the internal transcript format. Handle chunking for long files. | 6.1, 1.11 |
| **6.3** | **Local Whisper backend** | Implement the transcription interface using whisper.cpp compiled into the Rust backend (or as a sidecar binary). Run inference locally. Support model size selection (tiny/base/small/medium). | 6.1 |
| **6.4** | **Audio extraction** | Extract audio from video files using FFmpeg for transcription. Support all video formats in the asset library. Temporary audio file cleanup after transcription. | 1.19, 6.1 |
| **6.5** | **Transcription job manager** | Queue and manage transcription jobs. Show progress (percentage, estimated time remaining). Allow cancellation. Store completed transcripts in SQLite. | 6.2, 6.3, 6.4 |
| **6.6** | **Transcription UI** | Creator Studio page for transcription: file picker (import or select from asset library), backend selection (cloud/local), model size (for local), language selection, start button, progress display. | 6.5, 2.4 |

## **Section 6B: Caption System**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **6.7** | **Caption data model** | Define caption data structure in shared types: list of segments with start\_time, end\_time, text, word-level timestamps (optional), style properties. Support SRT, VTT, and JSON serialisation. | 1.3 |
| **6.8** | **Caption file generator** | Convert transcript output to caption files. Export as SRT, VTT, or JSON. Handle line breaking rules (max characters per line, max lines per segment). | 6.7, 6.5 |
| **6.9** | **Caption editor** | Full caption editing interface: list of segments with editable text and timestamps. Waveform visualisation of the source audio. Click on waveform to jump to timestamp. Drag segment boundaries. | 6.7, 6.8 |
| **6.10** | **Caption style editor** | Customise caption appearance: font family, size, colour, outline/stroke, shadow, background colour/opacity, position (top/middle/bottom), alignment. Live preview on a video frame. | 6.9 |
| **6.11** | **Caption style presets** | Save and load caption style presets. Bundle 3–5 default presets (e.g. "YouTube Standard", "TikTok Bold", "Minimal", "Karaoke Highlight"). User-created presets stored in SQLite. | 6.10 |

## **Section 6C: Voice Cloning & TTS**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **6.12** | **Voice service interface** | Define a Rust trait/interface for voice synthesis backends. Input: text \+ voice profile ID. Output: audio file. Supports swappable backends (API and local). | 1.3 |
| **6.13** | **ElevenLabs API backend** | Implement voice synthesis interface using the ElevenLabs API. Handle: list available voices, clone voice from samples, generate speech, stream audio. API key stored in secure credential store. | 6.12, 1.11 |
| **6.14** | **Voice profile manager** | Create and manage voice profiles. Upload voice samples (minimum requirements per provider), initiate cloning via the API, store profile metadata locally. List, preview, and delete profiles. | 6.13 |
| **6.15** | **Voice clone UI** | Creator Studio page for voice cloning: upload samples, select provider, initiate cloning, manage existing profiles. Preview each profile with sample text. | 6.14 |
| **6.16** | **Text-to-speech engine** | Generate speech from text input using a selected voice profile. Preview audio before saving. Export as audio file to the asset library. | 6.12, 6.13 |
| **6.17** | **TTS generation UI** | Creator Studio page for TTS: text input area, voice profile selector, speed/pitch controls (where supported), generate button, audio preview player, save to asset library button. | 6.16, 2.4 |
| **6.18** | **Local TTS fallback** | Implement text-to-speech using system voices (Windows SAPI, macOS AVSpeechSynthesizer, Linux espeak). No API key required. Basic voice selection. | 6.12 |
| **6.19** | **TTS alert integration** | Connect TTS to the alert system. Option to read alert messages (donation text, sub messages) aloud using a selected voice profile. Configurable per alert type. | 6.16, 2.29 |

## **Section 6D: Creator Studio Dashboard**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **6.20** | **Creator Studio landing page** | Replace the placeholder from 5.18 with a full dashboard: recent transcription jobs, voice profiles, caption projects, quick-start buttons for each workflow. | 6.6, 6.15, 6.17 |
| **6.21** | **API key management UI** | Dedicated settings page for managing external API keys (OpenAI, ElevenLabs). Show connection status, usage quota where available, and clear instructions for obtaining keys. | 1.11 |
| **6.22** | **Offline mode indicators** | Show which Creator Studio features are available offline (local transcription, caption editing, local TTS) vs online-only (cloud transcription, voice cloning, cloud TTS). Grey out unavailable features when offline. | 6.20 |

# **Phase 7: Creator Studio — Video Editor**

*18 tasks  |  Estimated 5–7 weeks*

## **Section 7A: Video Project Management**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.1** | **Video project data model** | Define project schema: source video reference, trim points, audio tracks (with volume/fade), caption track reference, aspect ratio, export settings. JSON-serialisable. Stored in SQLite. | 1.3 |
| **7.2** | **Video project database schema** | Migration for the projects table: id, name, source\_video\_asset\_id, config (JSON blob), thumbnail, status (draft/exported), created\_at, updated\_at. | 1.5 |
| **7.3** | **Video project CRUD** | Rust service for creating, loading, saving, deleting, and duplicating video projects. | 7.1, 7.2 |

## **Section 7B: Video Editor — Core**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.4** | **Video preview player** | Video preview window with play/pause, scrub bar, frame-by-frame stepping (arrow keys), playback speed control, and full-screen mode. Renders the source video file. | 7.1 |
| **7.5** | **Timeline component** | Horizontal timeline with a video track. Scrubber/playhead synced to the preview player. Zoom in/out on the timeline. Time markers. | 7.4 |
| **7.6** | **Trim & split tools** | Set in/out points on the timeline to trim the video. Split at playhead. Select and delete sections. Ripple delete (close the gap) or leave gap. | 7.5 |
| **7.7** | **Timeline audio track** | Display the original audio waveform on the timeline. Separate audio track for background music. Visual distinction between tracks. | 7.5 |

## **Section 7C: Aspect Ratio & Crop**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.8** | **Aspect ratio presets** | One-click preset buttons: 16:9 (landscape), 9:16 (vertical/TikTok/Reels), 1:1 (square/Instagram), 4:5 (Instagram portrait). Preview updates immediately. | 7.4 |
| **7.9** | **Smart crop with focus point** | When changing aspect ratio, show a crop frame on the preview. User can drag to adjust the focus point. Option for auto-centring on detected faces (stretch goal) or manual positioning. | 7.8 |

## **Section 7D: Audio Mixing**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.10** | **Background music import** | Add background music from the asset library or import a new audio file. Place on the music track in the timeline. | 7.7, 2.4 |
| **7.11** | **Audio volume & fading** | Volume slider per audio track (original audio, music). Fade in/fade out controls with configurable duration. Preview audio mix in real time. | 7.10 |
| **7.12** | **Audio ducking (optional)** | Auto-lower music volume when speech is detected in the original audio. Configurable threshold and reduction amount. | 7.11 |

## **Section 7E: Caption Overlay**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.13** | **Caption track on timeline** | Display captions as blocks on a dedicated caption track in the timeline. Each block shows the caption text. Drag to adjust timing. | 7.5, 6.7 |
| **7.14** | **Caption import** | Import captions from a transcription job (Phase 6\) or load an external SRT/VTT file. Apply a caption style preset. | 7.13, 6.8, 6.11 |
| **7.15** | **Caption preview on video** | Render styled captions on the video preview in real time as the playhead moves. Position and style based on the caption style system from Phase 6\. | 7.14, 6.10 |

## **Section 7F: Export Engine**

| \# | Task | Description | Depends |
| :---- | :---- | :---- | :---- |
| **7.16** | **Export pipeline** | Build FFmpeg command from the video project config: apply trim points, aspect ratio/crop, mix audio tracks, burn captions. Export as MP4 (H.264/H.265). Progress reporting via Socket.IO. | 7.1, 1.19 |
| **7.17** | **Export presets** | Pre-configured export profiles: YouTube (1080p/4K, 16:9), TikTok (1080p, 9:16), Instagram Reels (1080p, 9:16), Instagram Post (1080p, 1:1 or 4:5), Twitter/X (1080p, 16:9). Custom resolution/bitrate option. | 7.16 |
| **7.18** | **Batch export** | Export the same project in multiple presets in one go. Queue all exports, process sequentially, report progress for each. Save all exported files to a chosen directory. | 7.17 |

# **Task Count Summary**

| Phase | Section | Tasks | Total | Phase Total |
| :---- | :---- | :---: | :---: | :---: |
| **1** | 1A: Project Scaffolding | 3 | 3 |  |
| **1** | 1B: Database & Configuration | 4 | 7 |  |
| **1** | 1C: Local Server & Communication | 3 | 10 |  |
| **1** | 1D: Platform Authentication | 5 | 15 |  |
| **1** | 1E: API & Media Infrastructure | 4 | 19 | **19** |
| **2** | 2A: Asset Management | 5 | 24 |  |
| **2** | 2B: Design Schema & Storage | 3 | 27 |  |
| **2** | 2C: Visual Editor — Core | 10 | 37 |  |
| **2** | 2D: Animation System | 3 | 40 |  |
| **2** | 2E: Sound System | 2 | 42 |  |
| **2** | 2F: Alert System | 7 | 49 |  |
| **2** | 2G: Overlay & Scene System | 5 | 54 |  |
| **2** | 2H: Preview & Testing | 2 | 56 | **37** |
| **3** | 3A: Chat Connections | 5 | 61 |  |
| **3** | 3B: Unified Chat UI | 7 | 68 |  |
| **3** | 3C: Chat Moderation | 3 | 71 |  |
| **3** | 3D: Chat Bot | 5 | 76 |  |
| **3** | 3E: Chat Overlay | 2 | 78 | **22** |
| **4** | 4A: Multi-Platform Stream Control | 3 | 81 |  |
| **4** | 4B: OBS WebSocket Integration | 5 | 86 |  |
| **4** | 4C: Scene Control | 2 | 88 |  |
| **4** | 4D: Scheduling & Clips | 2 | 90 |  |
| **4** | 4E: Analytics | 4 | 94 | **16** |
| **5** | 5A: Theme System | 4 | 98 |  |
| **5** | 5B: Transition Stingers | 3 | 101 |  |
| **5** | 5C: Panel Designer | 2 | 103 |  |
| **5** | 5D: Keyboard Shortcuts | 2 | 105 |  |
| **5** | 5E: Onboarding & UX | 4 | 109 |  |
| **5** | 5F: Release Preparation | 3 | 112 | **18** |
| **6** | 6A: Transcription Engine | 6 | 118 |  |
| **6** | 6B: Caption System | 5 | 123 |  |
| **6** | 6C: Voice Cloning & TTS | 8 | 131 |  |
| **6** | 6D: Creator Studio Dashboard | 3 | 134 | **22** |
| **7** | 7A: Video Project Management | 3 | 137 |  |
| **7** | 7B: Video Editor — Core | 4 | 141 |  |
| **7** | 7C: Aspect Ratio & Crop | 2 | 143 |  |
| **7** | 7D: Audio Mixing | 3 | 146 |  |
| **7** | 7E: Caption Overlay | 3 | 149 |  |
| **7** | 7F: Export Engine | 3 | 152 | **18** |

**Total: 152 tasks across 7 phases and 38 sections.**