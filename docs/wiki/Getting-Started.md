# Getting Started

## System Requirements

### All Platforms
- 64-bit operating system
- 512MB free RAM (MacEnv uses ~10MB, services use more)
- Internet connection for initial package installation

### macOS
- macOS 11 Big Sur or later
- Xcode Command Line Tools: `xcode-select --install`
- Homebrew is installed automatically by MacEnv if not present

### Linux
- **Ubuntu/Debian** 22.04+
  - `sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev`
- **Arch Linux**
  - `sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3`
- **Fedora** 38+
  - `sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel`

### Windows
- Windows 10 version 1803 or later
- WebView2 Runtime (included in Windows 11, installable on Windows 10)

## Installation

### From Release (Recommended)

1. Download the latest release for your platform from [GitHub Releases](../../releases)
2. Install:
   - **macOS**: Open the `.dmg`, drag MacEnv to Applications. First run: `xattr -rd com.apple.quarantine /Applications/MacEnv.app`
   - **Linux**: Use the `.AppImage` (make executable with `chmod +x`) or install the `.deb`
   - **Windows**: Run the `.msi` installer

### From Source

```bash
# Prerequisites: Rust (stable), Node.js 22+
git clone https://github.com/your-org/macenv.git
cd macenv
npm install
npm run tauri dev    # Development mode
npm run tauri build  # Production build
```

## First Run — Setup Wizard

On first launch, MacEnv runs a setup wizard:

### Step 1: Package Manager Check
MacEnv checks for your platform's package manager:
- **macOS**: Homebrew — if not installed, MacEnv installs it automatically
- **Linux**: Detects apt, pacman, or dnf
- **Windows**: Checks for winget

### Step 2: Stack Selection
Choose your development stack:

| Component | Options |
|-----------|---------|
| PHP | 8.1, 8.2, 8.3, 8.4 |
| Database | MySQL 8.0, MySQL 8.4, MariaDB, PostgreSQL 15/16/17 |
| Web Server | Nginx |
| Cache | Redis, Memcached |
| Mail | Mailpit |
| DNS | dnsmasq (*.test domains) |
| SSL | mkcert (trusted local certs) |
| Tools | Composer, Node.js 18/20/22 |

### Step 3: Installation
MacEnv installs selected packages via your native package manager. A progress bar shows installation status.

### Step 4: DNS Configuration
MacEnv configures dnsmasq so that all `*.test` domains resolve to `127.0.0.1`. On macOS, it also creates a resolver file at `/etc/resolver/test`.

## Daily Usage

### Starting Services
1. Open MacEnv
2. The Dashboard shows all services with status indicators
3. Click **Start** on any service to start it
4. Click **Start All** to start everything

### Creating a Project
1. Go to **Projects** tab
2. Click **Add Site**
3. Enter project name and select folder
4. MacEnv automatically:
   - Creates an Nginx virtual host
   - Generates an SSL certificate via mkcert
   - Configures DNS (`projectname.test`)
   - Optionally creates a database matching the project name

### Accessing Your Site
Open `https://projectname.test` in your browser. The SSL certificate is trusted by your system.

### Checking Mail
1. Go to **Mail** tab
2. The embedded Mailpit UI shows all captured emails
3. PHP's `mail()` function is automatically configured to route to Mailpit
