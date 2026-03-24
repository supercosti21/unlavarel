# Package Manager Abstraction

## Overview

Unlavarel uses your system's native package manager to install and manage development tools. This means:

- No bloated bundles — packages are shared with the rest of your system
- Automatic updates via your package manager's update mechanism
- Native integration with system services

## Supported Package Managers

| Platform | Package Manager | Service Manager | Privilege |
|----------|----------------|-----------------|-----------|
| macOS | Homebrew | brew services | User (no sudo) |
| Ubuntu/Debian | apt | systemd | Elevated (sudo) |
| Arch Linux | pacman | systemd | Elevated (sudo) |
| Fedora | dnf (planned) | systemd | Elevated (sudo) |
| Windows | winget (planned) | Windows Services | Elevated |

## The PackageManager Trait

```rust
#[async_trait]
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &'static str;
    async fn is_available(&self) -> bool;
    async fn bootstrap(&self) -> Result<()>;
    fn resolve_native_name(&self, id: &PackageId) -> Result<String>;
    async fn install(&self, id: &PackageId) -> Result<InstalledPackage>;
    async fn uninstall(&self, id: &PackageId) -> Result<()>;
    async fn upgrade(&self, id: &PackageId) -> Result<InstalledPackage>;
    async fn list_installed(&self) -> Result<Vec<InstalledPackage>>;
    async fn is_installed(&self, id: &PackageId) -> Result<bool>;
    async fn available_versions(&self, canonical: &str) -> Result<Vec<String>>;
    fn prefix(&self) -> PathBuf;
    fn install_privilege(&self) -> Privilege;
    async fn update_index(&self) -> Result<()>;
}
```

## Package Name Registry

The same software has different package names across managers. Unlavarel resolves this via `packages.toml`:

```toml
[php.managers.homebrew]
pattern = "php@{version}"           # → php@8.3

[php.managers.apt]
pattern = "php{version}-fpm"        # → php8.3-fpm

[php.managers.pacman]
pattern = "php"                     # → php (single version)
```

### Pattern Variables

- `{version}` — The requested version (e.g., "8.3")
- `{prefix}` — The package manager's install prefix

### Additional Metadata

Each mapping can include:
- `tap` — Homebrew tap to add first
- `ppa` — apt PPA to add first
- `additional_packages` — Extra packages to install (e.g., PHP extensions)
- `binary` — Name of the main binary
- `binary_path_template` — Where to find the binary
- `config_path_template` — Where config files are stored

## Platform Details

### Homebrew (macOS)

- **Binary paths**: `/opt/homebrew/bin/` (Apple Silicon) or `/usr/local/bin/` (Intel)
- **Config paths**: `/opt/homebrew/etc/` or `/usr/local/etc/`
- **Prefix**: `/opt/homebrew` (Apple Silicon) or `/usr/local` (Intel)
- **No sudo required** for install, uninstall, or service management
- **Multi-version**: `php@8.1`, `php@8.3`, `mysql@8.0`, `node@20` etc.
- **Taps**: Some packages need a tap first (e.g., `shivammathur/php`)

### apt (Debian/Ubuntu)

- **Binary paths**: `/usr/bin/`, `/usr/sbin/`
- **Config paths**: `/etc/`
- **Requires sudo** for install/uninstall
- **PPAs**: Some packages need a PPA (e.g., `ondrej/php` for multiple PHP versions)
- **Additional packages**: PHP on apt requires installing extensions separately

### pacman (Arch Linux)

- **Binary paths**: `/usr/bin/`
- **Config paths**: `/etc/`
- **Requires sudo** for install/uninstall
- **Single version**: pacman repos typically only have the latest version
- **AUR**: Some packages (like Mailpit) are in the AUR
- **MariaDB default**: Arch uses MariaDB instead of MySQL

### winget (Windows)

- **Planned implementation**
- Uses `winget install` for package installation
- Windows Services for service management
- Package IDs use publisher format (e.g., `PHP.PHP`, `Oracle.MySQL`)

## Automatic Platform Detection

Unlavarel detects the platform at startup:

```rust
pub fn create_package_manager() -> Box<dyn PackageManager> {
    match current_os() {
        OsType::MacOS => Box::new(Homebrew::new()),
        OsType::Linux => {
            if Path::new("/usr/bin/pacman").exists() {
                Box::new(Pacman::new())
            } else {
                Box::new(Apt::new())
            }
        }
        OsType::Windows => Box::new(Winget::new()),
    }
}
```

## Adding New Package Managers

To add support for a new package manager (e.g., dnf):

1. Create `src-tauri/src/package_manager/dnf.rs`
2. Implement the `PackageManager` trait
3. Add the new module to `mod.rs`
4. Update `create_package_manager()` detection logic
5. Add mappings to `packages.toml`
