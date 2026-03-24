# Service Management

## Supported Services

| Service | Category | Multi-Version | Default Port |
|---------|----------|---------------|-------------|
| PHP-FPM | Language | Yes (8.1-8.4) | 9000 |
| MySQL | Database | Yes (8.0, 8.4) | 3306 |
| MariaDB | Database | No | 3306 |
| PostgreSQL | Database | Yes (15-17) | 5432 |
| Nginx | Web Server | No | 80/443 |
| Redis | Cache | No | 6379 |
| Memcached | Cache | No | 11211 |
| Mailpit | Mail | No | 1025 (SMTP), 8025 (Web) |
| dnsmasq | DNS | No | 53 |

## Service Lifecycle

### States

Each service can be in one of four states:

- **Running** (green dot) — Service is active and accepting connections
- **Stopped** (red dot) — Service is installed but not running
- **Error** (yellow dot) — Service encountered an error
- **Unknown** (gray dot) — Status could not be determined

### Operations

| Operation | macOS | Linux | Windows |
|-----------|-------|-------|---------|
| Start | `brew services start {name}` | `systemctl start {unit}` | Planned |
| Stop | `brew services stop {name}` | `systemctl stop {unit}` | Planned |
| Restart | `brew services restart {name}` | `systemctl restart {unit}` | Planned |
| Status | `brew services info {name}` | `systemctl is-active {unit}` | Planned |
| Logs | Homebrew log files | `journalctl -u {unit}` | Planned |

## Multi-Version PHP

Unlavarel supports running multiple PHP versions simultaneously:

### macOS (Homebrew)
```bash
brew install shivammathur/php/php@8.1
brew install shivammathur/php/php@8.2
brew install shivammathur/php/php@8.3
brew install shivammathur/php/php@8.4
```

Each version runs its own FPM process on a different socket:
- php@8.1: `/opt/homebrew/var/run/php81-fpm.sock`
- php@8.3: `/opt/homebrew/var/run/php83-fpm.sock`

### Linux (apt with Ondrej PPA)
```bash
sudo add-apt-repository ppa:ondrej/php
sudo apt install php8.1-fpm php8.2-fpm php8.3-fpm
```

### Switching Active PHP Version

The "active" PHP version (used by CLI) is switched by modifying PATH priority. Per-project PHP version is set in the Nginx vhost config by pointing `fastcgi_pass` to the correct FPM socket.

## Database Services

### MySQL
- Installed via `mysql` or `mysql@{version}` on Homebrew
- On Arch Linux, MariaDB is the default (drop-in MySQL replacement)
- Default root password: empty (local development)
- Data directory: Homebrew prefix `/var/mysql` or `/var/lib/mysql` on Linux

### PostgreSQL
- Multiple versions supported (15, 16, 17)
- Each version uses its own data directory
- Default user: `postgres` (local peer authentication)

## Configuration

### Nginx
Config files are stored per-platform:
- macOS: `$(brew --prefix)/etc/nginx/servers/`
- Linux: `/etc/nginx/sites-enabled/`
- Windows: `C:\nginx\conf\sites-enabled\`

### PHP
PHP configuration (php.ini) paths:
- macOS: `$(brew --prefix)/etc/php/{version}/php.ini`
- Linux (apt): `/etc/php/{version}/fpm/php.ini`
- Linux (pacman): `/etc/php/php.ini`

## Log Viewing

The Terminal component in Unlavarel shows real-time logs from services. Logs are sourced from:

- macOS: Homebrew log files in `$(brew --prefix)/var/log/`
- Linux: `journalctl` for systemd services
- Per-service log files (e.g., `/var/log/nginx/error.log`)
