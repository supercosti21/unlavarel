# Roadmap

## v0.1.0 — MVP (Done)

- [x] Project scaffolding (Tauri v2 + Svelte 5 + Rust)
- [x] PackageManager trait (Homebrew, apt, pacman, winget)
- [x] ServiceManager trait (brew services, systemd, Windows stub)
- [x] Package registry (packages.toml) cross-platform name mapping
- [x] Service discovery — scans system, shows only what's installed
- [x] Dashboard with ServiceCards (start/stop/restart, version detection)
- [x] Start All / Stop All services
- [x] Setup wizard with system pre-scan (shows already installed)
- [x] Single-script pkexec (one password prompt for all installs)
- [x] Session password caching (authenticate once, reuse for session)
- [x] Password dialog (macOS-style, blurred backdrop)
- [x] Nginx vhost generation (per-OS templates, PHP-FPM socket detection)
- [x] dnsmasq DNS configuration (macOS resolver + Linux systemd-resolved)
- [x] mkcert SSL certificates (local CA + per-domain wildcard certs)
- [x] Project management (add/remove with vhost + SSL + DB pipeline)
- [x] Quick app creation (Laravel, Laravel+Filament, Symfony, WordPress, Blank)
- [x] Built-in database manager (create/drop DB, browse tables, run SQL queries)
- [x] Multi-version PHP switching (Homebrew link, apt update-alternatives)
- [x] PHP extension toggle (enable/disable Xdebug, OPcache, etc.)
- [x] Mailpit integration (PHP sendmail_path auto-config)
- [x] Mail viewer (embedded Mailpit UI)
- [x] Real-time log streaming (Tauri events, tail -f / journalctl -f)
- [x] Settings page (theme, PHP default, project root, editor, browser)
- [x] Health check (verify all deps, DNS, SSL CA, Nginx sites)
- [x] Uninstall packages from Settings
- [x] System tray (show/hide, start/stop all, quit)
- [x] Custom titlebar (no native decorations)
- [x] Sharing via ngrok/cloudflared
- [x] Project snapshots (tar.gz + DB dump, restore, delete)
- [x] CSS design system (dark/light, Linear-inspired, zero hardcoded colors)
- [x] GitHub Actions CI/CD (macOS/Ubuntu/Windows)
- [x] Wiki documentation (10+ pages)
- [x] MariaDB auto-init on first start (Arch)
- [x] Valkey/Redis auto-detection
- [x] Database choices: MySQL, MariaDB, PostgreSQL, None

## v0.2.0 — UI Polish & UX

- [ ] Full UI polish pass (macOS/iOS style consistency)
- [ ] Better error handling UX (toast notifications, retry buttons)
- [ ] Sidebar icons (SVG icon set)
- [ ] Animated transitions between pages
- [ ] Open project in editor/browser buttons on dashboard
- [ ] Drag-and-drop project folder selection
- [ ] Import existing projects (scan directory)
- [ ] Project search/filter
- [ ] php.ini inline editor
- [ ] Nginx config editor with syntax highlighting
- [ ] Auto-start services on app launch (uses cached password)
- [ ] Keyboard shortcuts (Ctrl+1-6 for pages, Ctrl+R refresh)
- [ ] Real MacEnv logo/icon (replace placeholder purple)

## v0.3.0 — Advanced Features

- [ ] Project profiles (save/restore stack per project)
- [ ] Auto-backup scheduling (daily/weekly)
- [ ] Dump collector (Laravel dd() viewer)
- [ ] Advanced log viewer with search/filter/tail
- [ ] Multi-version database switching
- [ ] System resource monitoring (CPU, RAM per service)
- [ ] Reverse proxy for Docker/Node apps

## v0.4.0 — Extended Services

- [ ] MongoDB support
- [ ] Meilisearch / Typesense
- [ ] MinIO (S3-compatible storage)
- [ ] Caddy web server (HTTP/3)
- [ ] Laravel Reverb (WebSockets)
- [ ] Bun/Deno runtime support

## v0.5.0 — Enterprise

- [ ] Multilingual interface
- [ ] Auto-update (Tauri updater)
- [ ] Plugin system for custom services
- [ ] Team config sharing (macenv.yml)

## v1.0.0 — Stable

- [ ] macOS code signing + notarization
- [ ] Windows code signing
- [ ] Flatpak / AUR packaging
- [ ] Comprehensive test suite
- [ ] Video tutorials
