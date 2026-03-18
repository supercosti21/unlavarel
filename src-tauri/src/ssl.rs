// Local SSL certificate management via mkcert
//
// Planned operations:
// - setup_mkcert() -> install mkcert and set up local CA
// - generate_cert(domain) -> generate trusted cert for a domain
// - cert_exists(domain) -> check if cert already exists
// - cert_path(domain) -> return path to cert files
//
// Flow:
//   1. brew install mkcert (or equivalent)
//   2. mkcert -install (installs local CA in system trust store)
//   3. mkcert {domain} (generates {domain}.pem and {domain}-key.pem)
//
// Certs are stored in ~/.local/share/macenv/certs/

use std::path::PathBuf;

/// Get the directory where MacEnv stores SSL certificates.
pub fn certs_dir() -> PathBuf {
    let data_dir = dirs_next::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("macenv")
        .join("certs");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir
}
