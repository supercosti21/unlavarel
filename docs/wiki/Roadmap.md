# Roadmap

## v0.1.0 — MVP (Current)

### Done
- [x] Project scaffolding (Tauri v2 + Svelte 5 + Rust)
- [x] PackageManager trait with Homebrew, apt, pacman, winget implementations
- [x] ServiceManager trait with brew services, systemd implementations
- [x] Package registry (packages.toml) with cross-platform name mapping
- [x] Dashboard UI with ServiceCards
- [x] Project management (add/remove with vhost + SSL + DB pipeline)
- [x] Terminal/log viewer component
- [x] Mail viewer component (Mailpit iframe)
- [x] CSS design system (dark/light, Linear-inspired)
- [x] GitHub Actions CI/CD (macOS/Linux/Windows)
- [x] Setup wizard (first-run experience)
- [x] Nginx vhost generation (per-OS templates, PHP-FPM socket detection)
- [x] dnsmasq configuration automation (macOS resolver + Linux systemd-resolved)
- [x] mkcert SSL certificate generation (local CA + per-domain certs)
- [x] Auto database creation for new projects (MySQL/MariaDB/PostgreSQL)
- [x] PHP sendmail_path configuration for Mailpit (multi-version, all distros)
- [x] Version detection for installed services (binary parsing)
- [x] Quick app creation (Laravel, Laravel+Filament, WordPress, Symfony, Blank)
- [x] Multi-version PHP switching (Homebrew unlink/link, apt update-alternatives)
- [x] PHP extension toggle (enable/disable in php.ini, Xdebug etc.)
- [x] System tray integration (show/hide, start/stop all, quit)
- [x] Start/Stop All services
- [x] Service log viewer (per-service, journalctl/brew logs)
- [x] Restart service command
- [x] Database viewer component (connection details + Adminer embed)

### Remaining for v0.1.0 Release
- [ ] End-to-end testing on macOS with real Homebrew
- [ ] End-to-end testing on Ubuntu with real apt
- [ ] Placeholder icon replaced with real MacEnv logo
- [ ] Error handling UX polish (user-friendly error messages)

## v0.2.0 — Polish & UX

- [ ] Real-time log streaming via Tauri events (push, not poll)
- [ ] php.ini quick editor (open in default editor or inline)
- [ ] Nginx config editor (syntax highlighting)
- [ ] Auto-start on system boot option
- [ ] Settings page (theme toggle, default PHP version, project root path)
- [ ] Keyboard shortcuts
- [ ] Open project in VS Code / terminal
- [ ] Import existing projects (scan directory)
- [ ] Project search/filter

## v0.3.0 — Advanced Features

- [ ] Project profiles (save/restore stack configurations per project)
- [ ] Sharing via ngrok/cloudflared (1-click public URL)
- [ ] Project snapshots (files + database backup)
- [ ] Auto-backup scheduling (daily/weekly)
- [ ] Dump collector (Laravel `dd()` viewer)
- [ ] Advanced log viewer with search/filter/tail
- [ ] Multi-version database switching

## v0.4.0 — Extended Services

- [ ] MongoDB support
- [ ] Meilisearch / Typesense integration
- [ ] MinIO (S3-compatible storage)
- [ ] Caddy web server option (HTTP/3)
- [ ] Laravel Reverb (WebSockets)
- [ ] Bun/Deno runtime support
- [ ] Docker container proxy

## v0.5.0 — Enterprise & Community

- [ ] Multilingual interface
- [ ] Auto-update mechanism (Tauri updater)
- [ ] System resource monitoring (CPU, RAM, disk per service)
- [ ] Self-diagnostic tools
- [ ] Plugin system for custom services
- [ ] Community service registry

## v1.0.0 — Stable Release

- [ ] Full Laragon feature parity + cross-platform
- [ ] Comprehensive test suite (unit + integration)
- [ ] macOS code signing + notarization
- [ ] Windows code signing
- [ ] Flatpak / AUR packaging
- [ ] Full documentation and video tutorials

## Future Ideas

- Team configuration sharing (shared `macenv.yml` per project)
- AI assistant integration (Ollama for local LLMs)
- Remote deployment integration (Forge, Ploi, Envoyer)
- Mobile testing (LAN device access configuration)
- VS Code extension for MacEnv controls
- Performance profiling tools (Xdebug profiler viewer)
