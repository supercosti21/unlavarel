# Unlavarel Wiki

Welcome to the Unlavarel wiki. Unlavarel is a cross-platform local development environment manager — open source, free, and built for performance.

## Quick Links

- [Getting Started](Getting-Started.md)
- [Architecture Overview](Architecture.md)
- [Service Management](Services.md)
- [Project Management](Projects.md)
- [Package Manager Abstraction](Package-Managers.md)
- [DNS & SSL Configuration](DNS-and-SSL.md)
- [Mail Testing](Mail-Testing.md)
- [Design System](Design-System.md)
- [Contributing](Contributing.md)
- [CI/CD Pipeline](CI-CD.md)
- [Roadmap](Roadmap.md)

## What is Unlavarel?

Unlavarel is a lightweight GUI application that manages your local development stack. It installs and controls services like PHP, MySQL, Nginx, Redis, and more through your system's native package manager.

### Key Principles

1. **No bundled binaries** — Unlavarel is a control panel, not a distribution. Services are installed via Homebrew (macOS), apt/pacman (Linux), or winget (Windows).
2. **Zero root for daily use** — On macOS, Homebrew and dnsmasq operate without sudo. On Linux, systemctl user services are used where possible.
3. **Native performance** — Rust backend with Tauri v2, ~10MB RAM footprint, sub-second startup.
4. **Cross-platform** — Same features on macOS, Linux, and Windows.

### Comparison with Alternatives

| Feature | Unlavarel | Laragon | MAMP | Herd | Valet |
|---------|--------|---------|------|------|-------|
| macOS | Yes | No | Yes | Yes | Yes |
| Linux | Yes | No | No | No | Partial |
| Windows | Yes | Yes | Yes | Yes | No |
| Open Source | MIT | Freeware | Freemium | Freemium | MIT |
| GUI | Yes | Yes | Yes | Yes | No |
| Multi-PHP | Yes | Yes | Yes | Yes | Yes |
| Mail testing | Mailpit | Mailpit | No | Yes | No |
| DNS (no hosts) | dnsmasq | hosts file | No | Yes | dnsmasq |
| SSL (trusted) | mkcert | Self-signed | No | Yes | Yes |
| RAM usage | ~10MB | ~4-10MB | ~50MB+ | ~30MB | ~1MB |
