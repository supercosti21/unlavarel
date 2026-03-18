# Roadmap

## v0.1.0 — MVP (Current)

### Done
- [x] Project scaffolding (Tauri v2 + Svelte 5 + Rust)
- [x] PackageManager trait with Homebrew, apt, pacman, winget implementations
- [x] ServiceManager trait with brew services, systemd implementations
- [x] Package registry (packages.toml) with cross-platform name mapping
- [x] Dashboard UI with ServiceCards
- [x] Project management (add/remove)
- [x] Terminal/log viewer component
- [x] Mail viewer component (Mailpit iframe)
- [x] CSS design system (dark/light, Linear-inspired)
- [x] GitHub Actions CI/CD (macOS/Linux/Windows)

### Remaining
- [ ] Setup wizard (first-run experience)
- [ ] Actual service start/stop integration (test on macOS)
- [ ] Nginx vhost generation
- [ ] dnsmasq configuration automation
- [ ] mkcert SSL certificate generation
- [ ] Auto database creation for new projects
- [ ] PHP sendmail_path configuration for Mailpit
- [ ] Version detection for installed services
- [ ] Quick app creation (Laravel, WordPress)

## v0.2.0 — Core Features

- [ ] Multi-version PHP switching (UI + backend)
- [ ] Multi-version database switching
- [ ] Database viewer (embedded Adminer or custom)
- [ ] PHP extension toggle (enable/disable Xdebug, etc.)
- [ ] php.ini quick editor
- [ ] Nginx config editor
- [ ] Real-time log streaming via Tauri events
- [ ] System tray integration
- [ ] Start/stop all services button
- [ ] Auto-start on system boot option

## v0.3.0 — Advanced Features

- [ ] Project profiles (save/restore stack configurations)
- [ ] Sharing via ngrok/cloudflared (1-click public URL)
- [ ] Project snapshots (files + database backup)
- [ ] Auto-backup scheduling
- [ ] Import existing projects (scan directory)
- [ ] Dump collector (Laravel `dd()` viewer)
- [ ] Advanced log viewer with search/filter

## v0.4.0 — Extended Services

- [ ] MongoDB support
- [ ] Meilisearch / Typesense integration
- [ ] MinIO (S3-compatible storage)
- [ ] Caddy web server option (HTTP/3)
- [ ] Laravel Reverb (WebSockets)
- [ ] Bun/Deno runtime support
- [ ] Docker container proxy

## v0.5.0 — Polish

- [ ] Settings UI (theme toggle, paths, preferences)
- [ ] Keyboard shortcuts
- [ ] Multilingual interface
- [ ] Auto-update mechanism (Tauri updater)
- [ ] System resource monitoring
- [ ] Performance profiling tools
- [ ] Self-diagnostic tools

## v1.0.0 — Stable Release

- [ ] Feature parity with Laragon
- [ ] Comprehensive test suite
- [ ] macOS code signing + notarization
- [ ] Windows code signing
- [ ] Flatpak / AUR packaging
- [ ] Full documentation and tutorials
- [ ] Community feedback integration

## Future Ideas

- Team configuration sharing (like Herd's `herd.yml`)
- Plugin system for custom services
- AI assistant integration (Ollama for local LLMs)
- Remote deployment integration
- Mobile testing (LAN access configuration)
- VS Code extension
