// Virtual hosts management (Nginx server blocks)
//
// Planned operations:
// - list_vhosts() -> list all configured virtual hosts
// - add_vhost(domain, root_path) -> generate Nginx server block config
// - remove_vhost(domain) -> remove Nginx server block
// - reload_nginx() -> reload Nginx to apply changes
//
// Nginx config template for a Laravel site:
//   server {
//       listen 80;
//       listen 443 ssl;
//       server_name {domain};
//       root {root_path}/public;
//       index index.php;
//
//       ssl_certificate {cert_path};
//       ssl_certificate_key {key_path};
//
//       location / {
//           try_files $uri $uri/ /index.php?$query_string;
//       }
//
//       location ~ \.php$ {
//           fastcgi_pass unix:/var/run/php/php-fpm.sock;
//           fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
//           include fastcgi_params;
//       }
//   }

use std::path::PathBuf;

/// Get the Nginx sites directory for the current platform.
pub fn nginx_sites_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        // Homebrew Nginx config
        let prefix = if cfg!(target_arch = "aarch64") {
            "/opt/homebrew"
        } else {
            "/usr/local"
        };
        PathBuf::from(prefix).join("etc/nginx/servers")
    } else if cfg!(target_os = "linux") {
        PathBuf::from("/etc/nginx/sites-enabled")
    } else {
        PathBuf::from("C:\\nginx\\conf\\sites-enabled")
    }
}
