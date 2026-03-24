# DNS & SSL Configuration

## DNS with dnsmasq

### How It Works

Instead of editing `/etc/hosts` for every new project, Unlavarel uses **dnsmasq** to resolve all `*.test` domains to `127.0.0.1`. This means:

- Any `something.test` domain automatically resolves to localhost
- No manual host file editing
- No need to restart DNS after adding a new project
- Works with wildcards (subdomains like `api.myapp.test` work too)

### macOS Setup

On macOS, dnsmasq runs as a user-level service via Homebrew (no root required for the service itself):

```bash
# Unlavarel runs these automatically:
brew install dnsmasq

# Configure dnsmasq to resolve *.test to 127.0.0.1
echo "address=/.test/127.0.0.1" >> $(brew --prefix)/etc/dnsmasq.conf

# Create a resolver so macOS uses dnsmasq for .test domains
# (This is the ONLY step that requires sudo)
sudo mkdir -p /etc/resolver
sudo bash -c 'echo "nameserver 127.0.0.1" > /etc/resolver/test'

# Start dnsmasq
brew services start dnsmasq
```

The `/etc/resolver/test` file tells macOS to use `127.0.0.1` (dnsmasq) for resolving any `.test` domain. All other DNS queries use your normal DNS servers.

### Linux Setup

On Linux, dnsmasq integrates with systemd-resolved or NetworkManager:

```bash
# Install dnsmasq
sudo pacman -S dnsmasq   # Arch
sudo apt install dnsmasq  # Ubuntu/Debian

# Configure
echo "address=/.test/127.0.0.1" | sudo tee -a /etc/dnsmasq.conf

# If using systemd-resolved, configure it to use dnsmasq
# Add to /etc/systemd/resolved.conf:
# [Resolve]
# DNS=127.0.0.1
# Domains=~test

sudo systemctl restart dnsmasq
```

### Troubleshooting DNS

```bash
# Test if .test resolution works
dig myapp.test @127.0.0.1

# Expected output should show:
# myapp.test.    0    IN    A    127.0.0.1

# If using ping:
ping myapp.test
# Should resolve to 127.0.0.1
```

## SSL with mkcert

### Why mkcert?

Self-signed certificates trigger browser warnings. **mkcert** creates certificates signed by a local CA that your system trusts — no warnings, real HTTPS.

### Setup

```bash
# Unlavarel runs these automatically:
brew install mkcert  # or equivalent

# Install the local CA into system trust stores
# (This needs to run once)
mkcert -install
```

The `mkcert -install` command adds a CA certificate to:
- macOS: System Keychain
- Linux: NSS shared security databases (for Firefox/Chrome)
- Windows: Local Machine certificate store

### Generating Certificates

When Unlavarel creates a project, it generates a trusted certificate:

```bash
cd ~/.local/share/macenv/certs/
mkcert myapp.test
# Creates: myapp.test.pem and myapp.test-key.pem
```

These certificates are referenced in the Nginx vhost configuration:

```nginx
ssl_certificate /path/to/myapp.test.pem;
ssl_certificate_key /path/to/myapp.test-key.pem;
```

### Certificate Storage

Certificates are stored at:
- macOS: `~/Library/Application Support/macenv/certs/`
- Linux: `~/.local/share/macenv/certs/`
- Windows: `%APPDATA%/macenv/certs/`

### Wildcard Certificates

For multi-tenant applications, Unlavarel can generate wildcard certs:

```bash
mkcert "*.myapp.test" myapp.test
```

This covers both `myapp.test` and any subdomain like `api.myapp.test` or `tenant1.myapp.test`.
