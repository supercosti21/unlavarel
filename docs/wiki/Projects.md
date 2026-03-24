# Project Management

## Overview

Unlavarel manages local development projects. Each project gets:
- An Nginx virtual host configuration
- A `.test` domain via dnsmasq
- An SSL certificate via mkcert
- Optionally, a database matching the project name

## Creating a Project

### Via GUI
1. Navigate to the **Projects** tab
2. Click **Add Site**
3. Enter the project name (e.g., `myapp`)
4. Select the project folder (e.g., `/Users/you/Code/myapp`)
5. Unlavarel automatically configures everything

### What Happens Behind the Scenes

When you add a project named `myapp` at `/Users/you/Code/myapp`:

1. **Nginx vhost** is generated:
   ```nginx
   server {
       listen 80;
       listen 443 ssl;
       server_name myapp.test;
       root /Users/you/Code/myapp/public;
       index index.php index.html;

       ssl_certificate /path/to/certs/myapp.test.pem;
       ssl_certificate_key /path/to/certs/myapp.test-key.pem;

       location / {
           try_files $uri $uri/ /index.php?$query_string;
       }

       location ~ \.php$ {
           fastcgi_pass unix:/opt/homebrew/var/run/php-fpm.sock;
           fastcgi_param SCRIPT_FILENAME $realpath_root$fastcgi_script_name;
           include fastcgi_params;
       }
   }
   ```

2. **SSL certificate** is generated via mkcert:
   ```bash
   mkcert myapp.test
   ```

3. **DNS** is already handled by dnsmasq (`*.test → 127.0.0.1`)

4. **Database** (optional): creates a MySQL/PostgreSQL database named `myapp`

5. **Nginx is reloaded** to apply the new config

## Quick App Creation

Unlavarel can scaffold new projects using popular frameworks:

### Laravel
```bash
composer create-project laravel/laravel myapp
```
Unlavarel runs this, then automatically configures the vhost, SSL, DNS, and database.

### WordPress
Downloads and extracts the latest WordPress, configures `wp-config.php` with the local database credentials.

### Symfony
```bash
composer create-project symfony/skeleton myapp
```

## Project Data Storage

Projects are stored in a JSON file at:
- macOS: `~/Library/Application Support/macenv/projects.json`
- Linux: `~/.local/share/macenv/projects.json`
- Windows: `%APPDATA%/macenv/projects.json`

### Data Format
```json
[
  {
    "name": "myapp",
    "path": "/Users/you/Code/myapp",
    "domain": "myapp.test",
    "ssl": true
  }
]
```

## Removing a Project

Removing a project:
1. Deletes the Nginx vhost config
2. Removes the SSL certificate
3. Removes the project from Unlavarel's list

It does **NOT** delete the project files or database. Those must be removed manually.
