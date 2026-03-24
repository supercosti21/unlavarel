# Mail Testing

## Overview

Unlavarel integrates **Mailpit** for email testing. Mailpit captures all outgoing email from your PHP applications and provides a web UI to inspect them — no emails ever leave your machine.

## How It Works

```
PHP Application → mail() → Mailpit SMTP (port 1025) → Mailpit Web UI (port 8025)
```

1. PHP's `sendmail_path` is configured to route through Mailpit
2. Mailpit captures all emails on SMTP port 1025
3. The Mailpit web UI displays captured emails on port 8025
4. Unlavarel embeds the Mailpit UI in the **Mail** tab

## Installation

Mailpit is installed via your native package manager:

```bash
# macOS
brew install mailpit

# Arch Linux (AUR)
yay -S mailpit

# Ubuntu/Debian
# Download from GitHub releases or use Homebrew on Linux
```

## PHP Configuration

Unlavarel automatically configures PHP to use Mailpit. In `php.ini`:

```ini
; macOS (Homebrew)
sendmail_path = /opt/homebrew/bin/mailpit sendmail

; Linux
sendmail_path = /usr/bin/mailpit sendmail
```

This means any call to `mail()`, `Laravel's Mail facade`, or any PHP mailer sending via sendmail will be captured by Mailpit.

## Mailpit Web UI

The Mailpit web UI is accessible at:
- **URL**: `http://localhost:8025`
- **Embedded**: Available in the Unlavarel **Mail** tab

Features of the Mailpit UI:
- View all captured emails
- HTML and text rendering
- View email headers and raw source
- Attachment viewing
- Search and filter
- Mark as read/unread
- Delete emails
- API for automation

## Ports

| Port | Protocol | Purpose |
|------|----------|---------|
| 1025 | SMTP | Email capture |
| 8025 | HTTP | Web UI |

## Laravel Configuration

For Laravel applications, you can also configure SMTP directly in `.env`:

```env
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=null
MAIL_PASSWORD=null
MAIL_ENCRYPTION=null
```

This approach works with any PHP framework, not just when using `mail()`.

## Troubleshooting

### Emails not appearing
1. Check if Mailpit is running (Dashboard → Mailpit service status)
2. Verify PHP's `sendmail_path` is set correctly: `php -i | grep sendmail_path`
3. Test with a simple PHP script: `<?php mail('test@test.com', 'Test', 'Body');`

### Port conflicts
If port 1025 or 8025 is already in use, Mailpit will fail to start. Check for conflicts:
```bash
lsof -i :1025
lsof -i :8025
```
