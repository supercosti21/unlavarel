# Architecture Overview

## High-Level Architecture

```
┌──────────────────────────────────────────────────┐
│                   MacEnv GUI                      │
│              (Svelte 5 Frontend)                  │
│                                                   │
│  ┌───────────┐ ┌──────────┐ ┌───────────────┐   │
│  │ServiceCard│ │ SiteList │ │   Terminal    │   │
│  │  .svelte  │ │ .svelte  │ │   .svelte     │   │
│  └─────┬─────┘ └────┬─────┘ └──────┬────────┘   │
│        │             │              │             │
│  ┌─────┴─────────────┴──────────────┴──────┐     │
│  │         Tauri IPC (invoke)               │     │
│  └─────────────────┬────────────────────────┘     │
└────────────────────┼─────────────────────────────┘
                     │
┌────────────────────┼─────────────────────────────┐
│                    │     Rust Backend              │
│                    ▼                               │
│  ┌─────────────────────────────────────────┐      │
│  │           Tauri Commands                 │      │
│  │    (services.rs, projects.rs)            │      │
│  └──────┬──────────────┬───────────────────┘      │
│         │              │                           │
│  ┌──────▼──────┐ ┌─────▼──────────┐               │
│  │  Service    │ │   Package      │               │
│  │  Manager    │ │   Manager      │               │
│  │  (trait)    │ │   (trait)      │               │
│  └──────┬──────┘ └──────┬─────────┘               │
│         │               │                          │
│  ┌──────┼───────┐  ┌────┼────────┐                │
│  │ brew │system │  │brew│apt│pac │                │
│  │ svc  │  d    │  │   │   │man │                │
│  └──────┴───────┘  └────┴────┴───┘                │
│         │               │                          │
└─────────┼───────────────┼──────────────────────────┘
          │               │
          ▼               ▼
   ┌──────────┐   ┌──────────────┐
   │ Services │   │   Packages   │
   │ PHP,MySQL│   │ via Homebrew │
   │ Nginx,...│   │  apt, pacman │
   └──────────┘   └──────────────┘
```

## Directory Structure

```
macenv/
├── src/                          # Svelte 5 frontend
│   ├── app.css                   # Design system (CSS variables)
│   ├── main.js                   # Svelte mount()
│   ├── App.svelte                # Main layout
│   └── lib/
│       ├── components/           # UI components
│       │   ├── ServiceCard.svelte
│       │   ├── SiteList.svelte
│       │   ├── Terminal.svelte
│       │   ├── MailViewer.svelte
│       │   ├── Sidebar.svelte
│       │   └── StatusBar.svelte
│       └── stores/               # Reactive state
│           ├── services.svelte.js
│           └── projects.svelte.js
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── resources/
│   │   └── packages.toml        # Package name registry
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri builder + commands
│       ├── error.rs              # Error types
│       ├── platform/             # OS detection
│       ├── package_manager/      # Install/uninstall packages
│       ├── service_manager/      # Start/stop services
│       ├── registry/             # Package name mapping
│       ├── services.rs           # Service Tauri commands
│       ├── projects.rs           # Project Tauri commands
│       ├── vhosts.rs             # Nginx config generation
│       ├── dns.rs                # dnsmasq management
│       ├── ssl.rs                # mkcert integration
│       └── mail.rs               # Mailpit integration
├── .github/workflows/
│   └── build.yml                 # CI/CD pipeline
├── docs/wiki/                    # Documentation
├── CLAUDE.md                     # AI assistant guidelines
├── package.json                  # Node.js deps
└── vite.config.js                # Vite configuration
```

## Trait-Based Abstraction

### PackageManager Trait

The `PackageManager` trait defines a uniform interface for installing software across all platforms:

```rust
#[async_trait]
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    async fn is_available(&self) -> bool;
    async fn bootstrap(&self) -> Result<()>;
    fn resolve_native_name(&self, id: &PackageId) -> Result<String>;
    async fn install(&self, id: &PackageId) -> Result<InstalledPackage>;
    async fn uninstall(&self, id: &PackageId) -> Result<()>;
    async fn list_installed(&self) -> Result<Vec<InstalledPackage>>;
    fn install_privilege(&self) -> Privilege;
    // ... more methods
}
```

Implementations:
- `Homebrew` — macOS (`brew install`, `brew services`)
- `Apt` — Debian/Ubuntu (`apt-get install`, `systemctl`)
- `Pacman` — Arch Linux (`pacman -S`, `systemctl`)
- `Winget` — Windows (`winget install`)

### ServiceManager Trait

Separate from PackageManager — manages running services:

```rust
#[async_trait]
pub trait ServiceManager: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self, service: &str) -> Result<()>;
    async fn stop(&self, service: &str) -> Result<()>;
    async fn restart(&self, service: &str) -> Result<()>;
    async fn status(&self, service: &str) -> Result<ServiceInfo>;
    async fn list_managed(&self) -> Result<Vec<ServiceInfo>>;
    async fn logs(&self, service: &str, lines: usize) -> Result<String>;
}
```

Implementations:
- `BrewServices` — macOS (`brew services start/stop`)
- `Systemd` — Linux (`systemctl start/stop`)
- `WindowsService` — Windows (stub, planned)

### Package Registry

The `packages.toml` file maps canonical package names to native names:

```toml
[php.managers.homebrew]
pattern = "php@{version}"     # brew install php@8.3

[php.managers.apt]
pattern = "php{version}-fpm"  # apt install php8.3-fpm

[php.managers.pacman]
pattern = "php"               # pacman -S php
```

This TOML-based approach is extensible without recompiling.

## Frontend Architecture

### Svelte 5 Runes

MacEnv uses Svelte 5's rune system instead of legacy stores:

- `$state()` — reactive state variables
- `$derived()` — computed values
- `$effect()` — side effects
- `$props()` — component props

Store files use `.svelte.js` extension (required for runes to work outside `.svelte` files).

### Tauri IPC Communication

Frontend communicates with Rust backend via `invoke()`:

```javascript
import { invoke } from "@tauri-apps/api/core";

const services = await invoke("get_services");
await invoke("start_service", { name: "php" });
```

## Data Flow

1. User clicks "Start" on a ServiceCard
2. Svelte store calls `invoke("start_service", { name })` via Tauri IPC
3. Rust `start_service` command creates a `ServiceManager` for the current OS
4. ServiceManager executes the appropriate command (`brew services start` / `systemctl start`)
5. Rust returns updated `ServiceInfo` to the frontend
6. Svelte store updates reactive state, UI reflects the change
