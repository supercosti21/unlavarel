/**
 * Format bytes into a human-readable size string.
 */
export function formatSize(bytes) {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
  return `${(bytes / 1073741824).toFixed(1)} GB`;
}

/**
 * Validate a database name (alphanumeric + underscore only).
 */
export function isValidDbName(name) {
  return /^[a-zA-Z0-9_]+$/.test(name);
}

/**
 * Generate a .test domain from a project name.
 */
export function projectDomain(name) {
  return `${name}.test`;
}

/**
 * Classify a service status string into a display category.
 */
export function statusCategory(status) {
  if (!status) return "unknown";
  const s = status.toLowerCase();
  if (s === "running") return "running";
  if (s === "stopped") return "stopped";
  if (s === "installed") return "installed";
  if (s.startsWith("error")) return "error";
  return "unknown";
}

/**
 * Sanitize a project name for use as a database name.
 */
export function sanitizeDbName(name) {
  return name.replace(/[^a-zA-Z0-9_]/g, "_");
}
