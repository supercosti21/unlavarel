use std::path::PathBuf;
use tokio::process::Command;

use crate::error::{MacEnvError, Result};
use crate::ssl::certs_dir;

/// Get the Nginx sites directory for the current platform.
pub fn nginx_sites_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        PathBuf::from(prefix).join("etc/nginx/servers")
    } else if cfg!(target_os = "linux") {
        // Arch uses /etc/nginx/sites-enabled (need to create it)
        // Debian/Ubuntu has it by default
        PathBuf::from("/etc/nginx/sites-enabled")
    } else {
        PathBuf::from("C:\\nginx\\conf\\sites-enabled")
    }
}

/// Get the PHP-FPM socket path for the current platform.
fn php_fpm_socket() -> String {
    if cfg!(target_os = "macos") {
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        format!("unix:{}/var/run/php-fpm.sock", prefix)
    } else if cfg!(target_os = "windows") {
        "127.0.0.1:9000".to_string()
    } else {
        // Linux: check common socket paths
        let paths = [
            "/run/php-fpm/php-fpm.sock",
            "/run/php/php-fpm.sock",
            "/var/run/php-fpm/php-fpm.sock",
        ];
        for path in &paths {
            if PathBuf::from(path).exists() {
                return format!("unix:{}", path);
            }
        }
        "unix:/run/php-fpm/php-fpm.sock".to_string()
    }
}

/// Generate an Nginx server block for a Laravel/PHP site.
fn generate_nginx_config(domain: &str, root_path: &str, ssl: bool) -> String {
    let fpm_socket = php_fpm_socket();
    let cert_dir = certs_dir();
    let cert_path = cert_dir.join(format!("{}.pem", domain));
    let key_path = cert_dir.join(format!("{}-key.pem", domain));

    let mut config = format!(
        r#"server {{
    listen 80;
    server_name {domain};
    root {root_path}/public;
    index index.php index.html index.htm;

    charset utf-8;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location = /favicon.ico {{ access_log off; log_not_found off; }}
    location = /robots.txt  {{ access_log off; log_not_found off; }}

    error_page 404 /index.php;

    location ~ \.php$ {{
        fastcgi_pass {fpm_socket};
        fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    location ~ /\.(?!well-known).* {{
        deny all;
    }}
}}
"#
    );

    if ssl {
        config.push_str(&format!(
            r#"
server {{
    listen 443 ssl;
    server_name {domain};
    root {root_path}/public;
    index index.php index.html index.htm;

    ssl_certificate {cert};
    ssl_certificate_key {key};

    charset utf-8;

    location / {{
        try_files $uri $uri/ /index.php?$query_string;
    }}

    location = /favicon.ico {{ access_log off; log_not_found off; }}
    location = /robots.txt  {{ access_log off; log_not_found off; }}

    error_page 404 /index.php;

    location ~ \.php$ {{
        fastcgi_pass {fpm_socket};
        fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
        include fastcgi_params;
    }}

    location ~ /\.(?!well-known).* {{
        deny all;
    }}
}}
"#,
            cert = cert_path.display(),
            key = key_path.display(),
        ));
    }

    config
}

/// Create an Nginx vhost for a domain.
pub async fn add_vhost(domain: &str, root_path: &str, ssl: bool) -> Result<PathBuf> {
    let sites_dir = nginx_sites_dir();
    tokio::fs::create_dir_all(&sites_dir).await?;

    let config_path = sites_dir.join(format!("{}.conf", domain));
    let config = generate_nginx_config(domain, root_path, ssl);

    tokio::fs::write(&config_path, config).await?;
    reload_nginx().await?;

    Ok(config_path)
}

/// Remove an Nginx vhost.
pub async fn remove_vhost(domain: &str) -> Result<()> {
    let config_path = nginx_sites_dir().join(format!("{}.conf", domain));
    if config_path.exists() {
        tokio::fs::remove_file(&config_path).await?;
        reload_nginx().await?;
    }
    Ok(())
}

/// List all configured vhosts.
pub async fn list_vhosts() -> Result<Vec<String>> {
    let sites_dir = nginx_sites_dir();
    if !sites_dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = tokio::fs::read_dir(&sites_dir).await?;
    let mut vhosts = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".conf") {
            vhosts.push(name.trim_end_matches(".conf").to_string());
        }
    }

    Ok(vhosts)
}

/// Reload Nginx to apply configuration changes.
pub async fn reload_nginx() -> Result<()> {
    let nginx_bin = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "/opt/homebrew/bin/nginx"
        } else {
            "/usr/local/bin/nginx"
        }
    } else if cfg!(target_os = "windows") {
        "C:\\nginx\\nginx.exe"
    } else {
        "nginx"
    };

    let output = Command::new(nginx_bin)
        .args(["-s", "reload"])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(MacEnvError::ServiceOperationFailed {
            service: "nginx".into(),
            op: "reload".into(),
            reason: String::from_utf8_lossy(&output.stderr).into(),
        })
    }
}

/// Test Nginx configuration is valid.
pub async fn test_nginx_config() -> Result<bool> {
    let output = Command::new("nginx").args(["-t"]).output().await?;

    Ok(output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_nginx_config_http_only() {
        let config = generate_nginx_config("myapp.test", "/home/user/projects/myapp", false);
        assert!(config.contains("server_name myapp.test"));
        assert!(config.contains("root /home/user/projects/myapp/public"));
        assert!(config.contains("listen 80"));
        assert!(config.contains("index.php"));
        assert!(config.contains("fastcgi_pass"));
        assert!(!config.contains("listen 443"));
        assert!(!config.contains("ssl_certificate"));
    }

    #[test]
    fn test_generate_nginx_config_with_ssl() {
        let config = generate_nginx_config("myapp.test", "/home/user/projects/myapp", true);
        assert!(config.contains("listen 80"));
        assert!(config.contains("listen 443 ssl"));
        assert!(config.contains("ssl_certificate"));
        assert!(config.contains("ssl_certificate_key"));
        assert!(config.contains("myapp.test.pem"));
        assert!(config.contains("myapp.test-key.pem"));
    }

    #[test]
    fn test_generate_nginx_config_has_security_rules() {
        let config = generate_nginx_config("app.test", "/srv/app", false);
        // Blocks hidden files (except .well-known)
        assert!(config.contains("deny all"));
        assert!(config.contains(".well-known"));
    }

    #[test]
    fn test_nginx_sites_dir_not_empty() {
        let dir = nginx_sites_dir();
        assert!(!dir.to_string_lossy().is_empty());
    }
}
