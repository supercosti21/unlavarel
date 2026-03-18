# MacEnv - Project Guidelines

## Project Overview

MacEnv is a cross-platform local development environment manager (like Laragon) built with Tauri v2 + Svelte 5 + Rust. It targets Laravel + Filament developers on macOS, Linux, and Windows.

## Architecture

- **Backend**: Rust (Tauri v2) in `src-tauri/`
- **Frontend**: Svelte 5 (runes) in `src/`
- **No Tailwind**: Pure CSS with custom properties in `src/app.css`
- **Package management**: Native OS package managers via trait abstraction

### Key Design Decisions

- `PackageManager` trait (`src-tauri/src/package_manager/mod.rs`) abstracts Homebrew/apt/pacman/winget
- `ServiceManager` trait (`src-tauri/src/service_manager/mod.rs`) abstracts brew services/systemd/Windows Services
- `packages.toml` (`src-tauri/resources/packages.toml`) maps canonical names to native package names
- Svelte stores use `.svelte.js` extension (required for `$state` runes)
- App does NOT bundle binaries — installs via native package manager

### Module Structure (Rust)

```
src-tauri/src/
  lib.rs              → Tauri builder, command registration
  error.rs            → MacEnvError enum (thiserror)
  platform/           → OS detection, permissions
  package_manager/    → PackageManager trait + implementations
  service_manager/    → ServiceManager trait + implementations
  registry/           → Package name mapping from packages.toml
  services.rs         → Tauri commands (get_services, start/stop)
  projects.rs         → Project CRUD + persistence
  vhosts.rs           → Nginx virtual hosts (stub)
  dns.rs              → dnsmasq DNS (stub)
  ssl.rs              → mkcert SSL (stub)
  mail.rs             → Mailpit integration (stub)
```

## Development

```bash
npm install          # Install frontend deps
npm run tauri dev    # Run in dev mode (hot reload)
npm run tauri build  # Production build
```

### Platform-specific deps

- **Arch Linux**: `webkit2gtk-4.1 libappindicator-gtk3`
- **Ubuntu/Debian**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- **macOS**: Xcode Command Line Tools

## Code Style

- Rust: standard rustfmt, use `thiserror` for error types, `async-trait` for async traits
- Svelte 5: use runes (`$state`, `$derived`, `$effect`, `$props()`), NOT legacy stores
- CSS: use variables from `app.css`, no inline styles, no Tailwind
- UI style: Linear.app inspired — minimal, dark theme, purple/green accents

## Commands

- `cargo clippy` in `src-tauri/` to lint Rust code
- `npm run dev` for frontend-only dev server (port 1420)
- `npm run tauri dev` for full Tauri app

## CI/CD

GitHub Actions matrix build: macOS (universal), Ubuntu (AppImage+deb), Windows (exe+msi).
Uses `tauri-apps/tauri-action@v0`. See `.github/workflows/build.yml`.
