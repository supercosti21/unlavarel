# Unlavarel - Project Guidelines

## Project Overview

Unlavarel is a cross-platform local development environment manager (like Laragon) built with Tauri v2 + Svelte 5 + Rust. It targets Laravel + Filament developers on macOS, Linux, and Windows. Open source, free, no Docker required.

## Architecture

- **Backend**: Rust (Tauri v2) in `src-tauri/`
- **Frontend**: Svelte 5 (runes) in `src/`
- **No Tailwind**: Pure CSS with custom properties in `src/app.css`
- **Package management**: Native OS package managers via trait abstraction

### Key Design Decisions

- `PackageManager` trait (`src-tauri/src/package_manager/mod.rs`) abstracts Homebrew/apt/pacman/winget
- `ServiceManager` trait (`src-tauri/src/service_manager/mod.rs`) abstracts brew services/systemd/Windows Services
- `packages.toml` (`src-tauri/resources/packages.toml`) maps canonical names to native package names per OS
- Service discovery (`discovery.rs`) runs all binary checks in parallel via `tokio::join!`
- PHP-FPM socket detection (`vhosts.rs`) searches 20+ candidate paths dynamically per OS
- brew services parsing handles both JSON (brew >= 4.x) and plain text output
- Setup installer runs packages one-by-one so single failures don't block others
- Session password caching (`elevated.rs`) — authenticate once, reuse via `sudo -S` for the session
- Svelte stores use `.svelte.js` extension (required for `$state` runes)
- App does NOT bundle binaries — installs via native package manager
- Custom titlebar with macOS-style traffic lights on all platforms (`decorations: false`)

### Module Structure (Rust)

```
src-tauri/src/
  lib.rs              → Tauri builder, 40+ command registrations
  error.rs            → UnlavarelError enum (thiserror)
  discovery.rs        → Parallel service detection, caching
  services.rs         → Service CRUD (start/stop/restart/status/PID/port)
  setup.rs            → First-run wizard, stack installer, health check, pre-scan
  settings.rs         → AppSettings persistence (theme, PHP, project root, editor)
  database.rs         → DB manager (connect, list DBs, tables, schema, run SQL)
  vhosts.rs           → Nginx config generation, dynamic PHP-FPM socket detection
  dns.rs              → dnsmasq configuration (macOS resolver + Linux systemd-resolved)
  ssl.rs              → mkcert certificate generation (CA + per-domain wildcards)
  projects.rs         → Project CRUD + import existing + vhost/SSL/DB creation pipeline
  quickapp.rs         → Project templates (Laravel, Filament, Symfony, WordPress, Blank)
  updater.rs          → In-app update checker via GitHub Releases API
  php.rs              → PHP version switching, extension management by category
  sharing.rs          → Site sharing via ngrok/Cloudflare tunnels
  snapshots.rs        → File + DB backup/restore with timestamps
  elevated.rs         → Session password caching + elevated script execution
  logs.rs             → Real-time log streaming via Tauri events
  tray.rs             → System tray icon + menu
  mail.rs             → Mailpit PHP sendmail_path configuration
  platform/           → OS detection, architecture, distribution
  package_manager/    → PackageManager trait + 4 implementations
  service_manager/    → ServiceManager trait + 3 implementations
  registry/           → Package name mapping from packages.toml
```

### Frontend Structure (Svelte 5)

```
src/
  App.svelte          → Root component, routing, keyboard shortcuts, elevation flow
  app.css             → Design system (CSS variables, dark/light themes)
  lib/components/
    Titlebar.svelte       → Custom titlebar with macOS traffic lights
    Sidebar.svelte        → Navigation with icons, keyboard nav
    ServiceCard.svelte    → Service status, version, PID, port, start/stop/restart
    SiteList.svelte       → Project list with domain, path, framework
    Terminal.svelte       → Log viewer with mono font
    MailViewer.svelte     → Custom mail client (message list, detail, HTML rendering)
    DbViewer.svelte       → Database manager (DB list, tables, schema, SQL runner)
    PhpManager.svelte     → PHP version switcher, extension toggle by category
    SetupWizard.svelte    → First-run setup (system check, stack selection, install)
    SettingsPage.svelte   → Settings, update checker, package manager, health check
    QuickAppDialog.svelte → Project scaffolding dialog
    ImportProjectDialog.svelte → Import existing project (manual + directory scan)
    SharingPanel.svelte   → Site sharing (ngrok/cloudflare)
    SnapshotsPanel.svelte → Backup/restore management
    PasswordDialog.svelte → Session password prompt
    StatusBar.svelte      → Bottom bar with service counts
    Toast.svelte          → Toast notifications
    Icon.svelte           → SVG icon system (30+ icons)
  lib/stores/
    services.svelte.js    → Service state management
    projects.svelte.js    → Project state management
    toast.svelte.js       → Toast notification store
```

## Development

```bash
npm install          # Install frontend deps
npm run tauri dev    # Run in dev mode (hot reload)
npm run tauri build  # Production build
npm run dev          # Frontend-only dev server (port 1420)
```

### Platform-specific deps

- **Arch Linux**: `webkit2gtk-4.1 libappindicator-gtk3`
- **Ubuntu/Debian**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
- **macOS**: Xcode Command Line Tools

## Code Style

- Rust: standard rustfmt, use `thiserror` for error types, `async-trait` for async traits
- Svelte 5: use runes (`$state`, `$derived`, `$effect`, `$props()`), NOT legacy stores
- CSS: use variables from `app.css`, no inline styles, no Tailwind
- UI style: GitHub/Vercel-inspired — minimal, dark theme, muted blue accents
- Error messages: user-friendly in UI, technical in console/logs

## Commands

- `cargo clippy` in `src-tauri/` to lint Rust code
- `npm run dev` for frontend-only dev server (port 1420)
- `npm run tauri dev` for full Tauri app

## CI/CD

GitHub Actions matrix build: macOS (universal), Ubuntu (AppImage+deb), Windows (exe+msi).
Uses `tauri-apps/tauri-action@v0`. See `.github/workflows/build.yml`.
Auto-publishes "latest" release on every push to main + versioned releases on tags.

## Cross-Platform Notes

- **macOS**: Homebrew for packages, `brew services` for service management, `/etc/resolver/test` for DNS
- **Linux (Arch)**: pacman for packages, systemd for services, session password via `sudo -S`
- **Linux (Debian/Ubuntu)**: apt for packages, systemd for services
- **Windows**: winget for packages, Windows Services (stub — not fully implemented yet)
- Config stored via `dirs_next::config_dir()` (macOS: `~/Library/Application Support/unlavarel/`, Linux: `~/.config/unlavarel/`)
