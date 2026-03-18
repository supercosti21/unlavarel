// Mailpit mail testing integration
//
// Mailpit captures all outgoing email from PHP's mail() function.
// It provides a web UI on port 8025 for viewing captured emails.
//
// Planned operations:
// - setup_mailpit() -> install Mailpit and configure PHP to use it
// - is_running() -> check if Mailpit service is active
// - get_mailpit_url() -> return the Mailpit web UI URL
//
// PHP configuration for Mailpit:
//   sendmail_path = /opt/homebrew/bin/mailpit sendmail
//   (or equivalent path per platform)
//
// Mailpit listens on:
//   SMTP: localhost:1025
//   Web UI: localhost:8025

/// Default Mailpit web UI URL.
pub const MAILPIT_URL: &str = "http://localhost:8025";

/// Default Mailpit SMTP port.
pub const MAILPIT_SMTP_PORT: u16 = 1025;
