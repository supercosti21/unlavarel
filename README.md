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

Like Laragon, but for macOS, Linux, and Windows.

## Features

- **Service management** — PHP-FPM, MySQL/MariaDB/PostgreSQL, Nginx, Redis, Memcached
- **Multi-version support** — Switch between PHP 8.1–8.4, multiple DB versions, Node.js
- **Project management** — Auto virtual hosts, SSL certificates, database creation
- **Mail testing** — Mailpit integration with embedded inbox
- **DNS resolution** — dnsmasq for *.test domains (no /etc/hosts editing)
- **SSL** — Local trusted certificates via mkcert
- **Cross-platform** — macOS (Homebrew), Linux (apt/pacman), Windows (winget)

## Tech Stack

- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte 5 + custom CSS
- **Package management**: Native OS package managers (Homebrew, apt, pacman, winget)

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (22+)
- Platform dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Linux (Debian/Ubuntu)**: `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **Linux (Arch)**: `sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3`

### Setup

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## License

MIT
