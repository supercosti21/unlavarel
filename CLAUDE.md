# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MacEnv is a cross-platform local development environment manager (like Laragon) built with Tauri v2 + Svelte 5 + Rust. It targets Laravel + Filament developers on macOS, Linux, and Windows. The app does NOT bundle binaries — it installs services via native OS package managers.

## Commands

```bash
npm install                      # Install frontend deps
npm run tauri dev                # Full Tauri app with hot reload (frontend on port 1420)
npm run tauri build              # Production build
npm run dev                      # Frontend-only dev server (no Rust backend)
cd src-tauri && cargo clippy     # Lint Rust code
cd src-tauri && cargo build      # Build Rust backend only
```

### Platform-specific deps

- **Arch Linux**: `webkit2gtk-4.1 libappindicator-gtk3`
- **Ubuntu/Debian**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- **macOS**: Xcode Command Line Tools

## Architecture

- **Backend**: Rust (Tauri v2) in `src-tauri/`
- **Frontend**: Svelte 5 (runes) in `src/` — no TypeScript, no Tailwind
- **IPC**: Frontend calls Rust via `@tauri-apps/api` `invoke()`, receives events via `listen()`

### Key Abstractions (Rust)

Two trait-based abstractions are the core of the backend:

1. **`PackageManager` trait** (`src-tauri/src/package_manager/mod.rs`) — abstracts Homebrew/apt/pacman/winget. Implementations in sibling files (`homebrew.rs`, `apt.rs`, `pacman.rs`, `winget.rs`). Factory function auto-detects the correct one at runtime.

2. **`ServiceManager` trait** (`src-tauri/src/service_manager/mod.rs`) — abstracts brew services/systemd/Windows Services. Same pattern with sibling implementations.

3. **`packages.toml`** (`src-tauri/resources/packages.toml`) — maps canonical service names (e.g. `php`, `mysql`) to platform-specific package names, versions, taps/PPAs, service names, and binary paths. This is the single source of truth for package metadata.

### Important Rust Modules

- `lib.rs` — Tauri builder and all command registrations (invoke_handler)
- `error.rs` — `MacEnvError` enum using `thiserror`
- `discovery.rs` — scans for installed binaries on startup, caches results
- `elevated.rs` — session password caching for sudo/pkexec elevation
- `platform/detect.rs` — runtime OS/distro/architecture detection
- `registry/mod.rs` — loads and queries `packages.toml`

### Frontend Structure

- `src/App.svelte` — main layout, page routing via `$state`, elevation handling, log streaming
- `src/lib/stores/*.svelte.js` — reactive stores using `$state` runes (must use `.svelte.js` extension)
- `src/lib/components/` — UI components (ServiceCard, SetupWizard, PhpManager, DbViewer, etc.)
- `src/app.css` — full design system with CSS custom properties (dark/light theme, colors, spacing)

### Data Flow

Frontend components → Tauri `invoke()` → Rust commands (registered in `lib.rs`) → trait implementations → OS commands via `tokio::process::Command`. Real-time logs stream back via Tauri `emit()` events.

## Code Style

- **Rust**: rustfmt, `thiserror` for errors, `async-trait` for async traits, `tokio` for async ops
- **Svelte 5**: runes only (`$state`, `$derived`, `$effect`, `$props()`), NOT legacy stores/reactivity
- **CSS**: use variables from `app.css`, scoped styles in components, BEM naming (`.component__element--modifier`), no inline styles, no Tailwind
- **UI**: Linear.app inspired — minimal, dark theme, purple/green accents

## Adding a New Service

1. Add entry to `src-tauri/resources/packages.toml` with manager-specific mappings
2. Add to service lists in relevant service manager files (`brew_services.rs`, `systemd.rs`, etc.)
3. Service auto-appears in UI via `get_services` discovery

## Adding a New Package Manager

1. Create `src-tauri/src/package_manager/<name>.rs` implementing `PackageManager` trait
2. Register in `package_manager/mod.rs` and update factory detection logic
3. Add mappings for all packages in `packages.toml`

## CI/CD

GitHub Actions matrix build: macOS (universal), Ubuntu (AppImage+deb), Windows (exe+msi). Uses `tauri-apps/tauri-action@v0`. See `.github/workflows/build.yml`.
