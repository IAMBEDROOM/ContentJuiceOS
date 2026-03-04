# ContentJuiceOS
ContentJuiceOS is an all-in-one content creator operating system built as a Tauri desktop application. It combines two major toolsets into a single, unified platform:

* **Live Suite —** A complete streaming command centre replacing the fragmented ecosystem of tools like StreamElements, Streamlabs, separate chat clients, alert editors, and social media managers. The only external dependency for streaming is OBS Studio (base version) for encoding and broadcasting.

* **Creator Studio —** A content production toolkit featuring voice cloning, video transcription and captioning, and a simple video editor for trimming, adding music, applying captions, and adjusting aspect ratios.

Everything lives inside one app with one design language. Content creators design their entire broadcast identity, manage chat from all platforms, edit their clips, generate captions, and produce polished short-form content without switching between a dozen different tools.

A companion mobile app is planned for a future release once the desktop application is fully functional. Its architecture is accounted for in this plan to ensure the desktop app is built ready for mobile integration.

| Target Platform (v1) | Desktop (Windows, macOS, Linux via Tauri) |
| :---- | :---- |
| **Future Platform** | Mobile companion app (iOS, Android) — post-v1 release |
| **Tech Stack** | Tauri (Rust backend), React/TypeScript frontend, FFmpeg |
| **External Dependency** | OBS Studio (base) for encoding/broadcast only |
| **Out of Scope (v1)** | Companion mobile app, self-hosted donations/tipping |
