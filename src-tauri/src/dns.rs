// dnsmasq DNS management for .test TLD resolution
//
// Planned operations:
// - setup_dnsmasq() -> install and configure dnsmasq for *.test -> 127.0.0.1
// - is_configured() -> check if dnsmasq resolver is set up
// - restart_dnsmasq() -> restart the dnsmasq service
//
// macOS setup (no root required with Homebrew):
//   1. brew install dnsmasq
//   2. echo "address=/.test/127.0.0.1" >> $(brew --prefix)/etc/dnsmasq.conf
//   3. sudo mkdir -p /etc/resolver
//   4. sudo bash -c 'echo "nameserver 127.0.0.1" > /etc/resolver/test'
//   5. brew services start dnsmasq
//
// Linux setup:
//   1. Install dnsmasq via package manager
//   2. Add "address=/.test/127.0.0.1" to /etc/dnsmasq.conf
//   3. Configure NetworkManager or systemd-resolved to use dnsmasq
//   4. systemctl restart dnsmasq

use std::path::PathBuf;

/// Get the dnsmasq config file path for the current platform.
pub fn dnsmasq_config_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        PathBuf::from(prefix).join("etc/dnsmasq.conf")
    } else {
        PathBuf::from("/etc/dnsmasq.conf")
    }
}
