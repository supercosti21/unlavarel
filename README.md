<p align="center">
  <img src="docs/logo.svg" alt="Unlavarel" width="220">
</p>

<p align="center">
  A cross-platform local development environment manager.<br>
  Open source, free, and built for performance.
</p>

<p align="center">
  <a href="https://github.com/supercosti21/unlavarel/releases"><img src="https://img.shields.io/github/v/release/supercosti21/unlavarel?style=flat-square&color=7c5bf0" alt="Release"></a>
  <a href="https://github.com/supercosti21/unlavarel/blob/main/LICENSE"><img src="https://img.shields.io/github/license/supercosti21/unlavarel?style=flat-square" alt="License"></a>
  <a href="https://supercosti21.github.io/unlavarel/"><img src="https://img.shields.io/badge/website-live-34d399?style=flat-square" alt="Website"></a>
</p>

---

Like Laragon, but open source and cross-platform. Built with Rust for speed.

## Features

- **Dashboard** — Start, stop, restart services with one click. Real-time status, PID, and port monitoring
- **Service discovery** — Auto-detects installed services at startup, shows only what you have
- **Multi-version PHP** — Switch between PHP 8.1–8.4 instantly, toggle extensions by category
- **Database manager** — Create/drop databases, browse tables, inspect schemas, run SQL queries
- **Project scaffolding** — Create Laravel, Filament, Symfony, WordPress projects with one click
- **Import existing projects** — Add existing project folders with auto-scan directory detection
- **Virtual hosts** — Auto-generated Nginx configs per project with PHP-FPM socket detection
- **SSL certificates** — Local HTTPS via mkcert, auto-generated per domain
- **DNS resolution** — dnsmasq for `*.test` domains (no `/etc/hosts` editing, no root)
- **Mail testing** — Built-in mail viewer with Mailpit (message list, read/unread, HTML preview)
- **Site sharing** — Public URLs via Cloudflare Tunnels or ngrok
- **Project snapshots** — One-click backup (files + database), timestamped, restorable
- **In-app update checker** — Check for new versions and download updates directly from settings
- **Package manager** — Scan system for installed packages, install missing ones from settings
- **Health check** — Verify all dependencies, services, DNS, SSL, and configuration
- **Session password caching** — Authenticate once, reuse for the session (Linux/macOS)
- **System tray** — Show/hide, start/stop all, quit from tray icon
- **Dark/Light theme** — GitHub-inspired design system, Linear-style UI
- **Keyboard shortcuts** — Ctrl+1-7 navigate, Ctrl+R refresh, Ctrl+N new project

## Supported Services

| Category | Services |
|----------|----------|
| Web Server | Nginx |
| Language | PHP 8.1, 8.2, 8.3, 8.4 |
| Database | MySQL, MariaDB, PostgreSQL 15-17 |
| Cache | Redis, Memcached |
| DNS | dnsmasq |
| Mail | Mailpit |
| Tools | Composer, Node.js, mkcert |
| Sharing | ngrok, Cloudflared |

## Tech Stack

- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte 5 (runes) + custom CSS
- **Package management**: Native OS package managers (Homebrew, apt, pacman, winget)
- **No Docker, no Electron, no YAML config**

## Installation

Download the latest release for your platform:

| Platform | Format |
|----------|--------|
| macOS | `.dmg` (universal) |
| Linux (Debian/Ubuntu) | `.deb`, `.AppImage` |
| Linux (Arch) | `.AppImage` |
| Windows | `.exe`, `.msi` |

[Download Latest Release](https://github.com/supercosti21/unlavarel/releases/latest)

> **⚠️ macOS: required step after installation**
>
> The app is not yet notarized with Apple. macOS will block it from opening ("Unlavarel non può essere aperta" / "Apple cannot verify...").
>
> **Open Terminal and run:**
> ```bash
> xattr -rd com.apple.quarantine /Applications/Unlavarel.app
> ```
> Then open the app normally. You only need to do this once.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (22+)
- Platform dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Arch Linux**: `sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3`
  - **Debian/Ubuntu**: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Run

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

### Lint

```bash
cd src-tauri && cargo clippy
```

## Architecture

```
src-tauri/src/           Rust backend (Tauri v2)
  lib.rs                 Tauri builder, 40+ command registrations
  discovery.rs           Parallel service detection at startup
  services.rs            Start/stop/restart, status + PID + port
  setup.rs               First-run wizard, stack installer, health check
  settings.rs            App settings persistence
  database.rs            DB manager (MySQL/MariaDB/PostgreSQL)
  vhosts.rs              Nginx config generation, PHP-FPM socket detection
  dns.rs                 dnsmasq configuration
  ssl.rs                 mkcert certificate generation
  projects.rs            Project CRUD + import + vhost/SSL/DB pipeline
  quickapp.rs            Project templates (Laravel, Filament, Symfony, WP)
  updater.rs             In-app update checker via GitHub releases API
  php.rs                 PHP version switching, extension management
  sharing.rs             ngrok/Cloudflare tunnel management
  snapshots.rs           File + DB backup/restore
  elevated.rs            Session password caching + elevated execution
  logs.rs                Real-time log streaming via Tauri events
  tray.rs                System tray icon + menu
  mail.rs                Mailpit PHP sendmail_path configuration
  package_manager/       PackageManager trait (Homebrew, apt, pacman, winget)
  service_manager/       ServiceManager trait (brew services, systemd, Windows)
  platform/              OS detection, architecture, distribution
  registry/              packages.toml name mapping

src/                     Svelte 5 frontend
  App.svelte             Root component, routing, keyboard shortcuts
  app.css                Design system (CSS variables, dark/light)
  lib/components/        UI components (18 components)
  lib/stores/            Reactive stores (.svelte.js)
```

## License

MIT
