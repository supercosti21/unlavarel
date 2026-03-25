<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let settings = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let healthResult = $state(null);
  let healthLoading = $state(false);
  let phpVersions = $state([]);
  let installedServices = $state([]);
  let uninstalling = $state(null);
  let updateInfo = $state(null);
  let updateChecking = $state(false);
  let currentVersion = $state("");
  let preScan = $state(null);
  let preScanLoading = $state(false);
  let installingPkg = $state(null);
  let updating = $state(false);
  let updateProgress = $state(null);
  let updateDone = $state(false);

  function friendlySettingsError(raw) {
    const msg = String(raw).toLowerCase();
    if (msg.includes("permission") || msg.includes("denied")) return "Permission denied. Authenticate first.";
    if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec")) return "Admin password required. Authenticate from the Dashboard.";
    if (msg.includes("not found") || msg.includes("no such")) return "Package not found. It may have been removed already.";
    if (msg.includes("in use") || msg.includes("running")) return "Stop the service first before uninstalling.";
    return String(raw);
  }

  $effect(() => {
    loadSettings();
    loadPhpVersions();
    loadInstalledServices();
    loadVersion();
    runPreScan(); // Auto-detect installed packages on load
  });

  async function loadSettings() {
    loading = true;
    try {
      settings = await invoke("get_settings");
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      loading = false;
    }
  }

  async function loadPhpVersions() {
    try {
      phpVersions = await invoke("get_php_versions");
    } catch {
      phpVersions = [];
    }
  }

  async function loadInstalledServices() {
    try {
      installedServices = await invoke("get_cached_services");
      if (installedServices.length === 0) {
        installedServices = await invoke("discover_services");
      }
    } catch {
      installedServices = [];
    }
  }

  async function uninstallPkg(id) {
    uninstalling = id;
    try {
      const result = await invoke("uninstall_package", { packageId: id });
      toastStore.success(result);
      await loadInstalledServices();
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      uninstalling = null;
    }
  }

  async function loadVersion() {
    try {
      currentVersion = await invoke("get_current_version");
    } catch {
      currentVersion = "unknown";
    }
  }

  async function checkUpdates() {
    updateChecking = true;
    updateDone = false;
    try {
      updateInfo = await invoke("check_for_updates");
      if (updateInfo.update_available) {
        toastStore.info(`Update available: v${updateInfo.latest_version}`);
      } else {
        toastStore.success("You're on the latest version");
      }
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      updateChecking = false;
    }
  }

  async function applyUpdate() {
    if (!updateInfo?.download_url) return;
    updating = true;
    updateProgress = null;
    updateDone = false;

    // Listen for progress events
    const unlisten = await listen("update-progress", (event) => {
      updateProgress = event.payload;
    });

    try {
      const msg = await invoke("download_and_install_update", {
        downloadUrl: updateInfo.download_url,
      });
      updateDone = true;
      toastStore.success(msg);
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      updating = false;
      unlisten();
    }
  }

  async function restartApp() {
    try {
      await invoke("restart_app");
    } catch (e) {
      toastStore.error("Failed to restart: " + e);
    }
  }

  function formatBytes(bytes) {
    if (!bytes) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return (bytes / Math.pow(k, i)).toFixed(1) + " " + sizes[i];
  }

  async function runPreScan() {
    preScanLoading = true;
    try {
      preScan = await invoke("pre_scan_system");
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      preScanLoading = false;
    }
  }

  async function installPkg(packageId) {
    installingPkg = packageId;
    try {
      const result = await invoke("install_single_package", { packageId });
      toastStore.success(result);
      // Refresh both scans
      await runPreScan();
      await loadInstalledServices();
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      installingPkg = null;
    }
  }

  const availablePackages = [
    { id: "php", name: "PHP", category: "Language" },
    { id: "composer", name: "Composer", category: "Tool" },
    { id: "nginx", name: "Nginx", category: "Web Server" },
    { id: "mysql", name: "MySQL", category: "Database" },
    { id: "mariadb", name: "MariaDB", category: "Database" },
    { id: "postgresql", name: "PostgreSQL", category: "Database" },
    { id: "redis", name: "Redis", category: "Cache" },
    { id: "memcached", name: "Memcached", category: "Cache" },
    { id: "node", name: "Node.js", category: "Tool" },
    { id: "dnsmasq", name: "dnsmasq", category: "DNS" },
    { id: "mkcert", name: "mkcert", category: "SSL" },
    { id: "mailpit", name: "Mailpit", category: "Mail" },
  ];

  let installedIds = $derived(preScan ? preScan.installed.map((i) => i.id) : []);
  let missingPackages = $derived(
    availablePackages.filter((p) => !installedIds.includes(p.id))
  );

  async function runHealthCheck() {
    healthLoading = true;
    try {
      healthResult = await invoke("health_check");
      if (healthResult.all_ok) {
        toastStore.success("All health checks passed");
      } else {
        toastStore.warning("Some dependencies are missing");
      }
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      healthLoading = false;
    }
  }

  async function save() {
    saving = true;
    try {
      await invoke("save_settings", { settings });
      toastStore.success("Settings saved");
      document.documentElement.setAttribute("data-theme", settings.theme);
    } catch (e) {
      toastStore.error(friendlySettingsError(e));
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings">
  <header class="settings__header">
    <h2>Settings</h2>
  </header>

  {#if loading}
    <div class="settings__loading">
      <span class="spinner"></span>
      <span>Loading settings...</span>
    </div>
  {:else if settings}
    <div class="settings__sections">
      <section class="settings__section">
        <h3>Appearance</h3>
        <p class="settings__desc">Customize the look and feel of Unlavarel.</p>
        <div class="settings__field">
          <label for="theme">Theme</label>
          <select id="theme" bind:value={settings.theme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
          </select>
        </div>
      </section>

      <section class="settings__section">
        <h3>Development</h3>
        <p class="settings__desc">Configure your development environment defaults.</p>
        <div class="settings__field">
          <label for="php">Default PHP Version</label>
          <select id="php" bind:value={settings.default_php_version}>
            {#if phpVersions.length > 0}
              {#each phpVersions as ver}
                <option value={ver.version}>PHP {ver.version}{ver.active ? " (active)" : ""}</option>
              {/each}
            {:else}
              <option value={settings.default_php_version}>PHP {settings.default_php_version}</option>
            {/if}
          </select>
        </div>
        <div class="settings__field">
          <label for="root">Project Root Directory</label>
          <input id="root" type="text" bind:value={settings.project_root} />
        </div>
      </section>

      <section class="settings__section">
        <h3>Tools</h3>
        <p class="settings__desc">External tools used for opening projects.</p>
        <div class="settings__field">
          <label for="editor">Editor Command</label>
          <input id="editor" type="text" bind:value={settings.editor_command} placeholder="code" />
          <span class="settings__hint">e.g. code, phpstorm, subl, vim</span>
        </div>
        <div class="settings__field">
          <label for="browser">Browser Command</label>
          <input id="browser" type="text" bind:value={settings.browser_command} placeholder="open" />
          <span class="settings__hint">e.g. open (macOS), xdg-open (Linux), start (Windows)</span>
        </div>
      </section>

      <section class="settings__section">
        <h3>Behavior</h3>
        <label class="settings__switch">
          <input type="checkbox" bind:checked={settings.auto_start_services} />
          <span class="settings__switch-track"></span>
          <span>Auto-start services on app launch</span>
        </label>
        <label class="settings__switch">
          <input type="checkbox" bind:checked={settings.start_minimized} />
          <span class="settings__switch-track"></span>
          <span>Start minimized to tray</span>
        </label>
      </section>

      <div class="settings__actions">
        <button class="btn-primary" onclick={save} disabled={saving}>
          {#if saving}
            <span class="spinner spinner--sm"></span>
          {/if}
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>

      <section class="settings__section">
        <div class="settings__health-header">
          <h3>Updates</h3>
          <button class="btn-ghost" onclick={checkUpdates} disabled={updateChecking}>
            {#if updateChecking}
              <span class="spinner spinner--sm"></span>
            {:else}
              <Icon name="refresh" size={14} />
            {/if}
            {updateChecking ? "Checking..." : "Check for Updates"}
          </button>
        </div>
        <p class="settings__desc">Current version: {currentVersion || "..."}</p>
        {#if updateInfo}
          {#if updateInfo.update_available}
            <div class="settings__update-available">
              <div class="settings__update-info">
                <span class="badge badge--warning">v{updateInfo.latest_version} available</span>
                {#if updateInfo.published_at}
                  <span class="settings__update-date">{new Date(updateInfo.published_at).toLocaleDateString()}</span>
                {/if}
              </div>
              {#if updateInfo.release_notes}
                <p class="settings__update-notes">{updateInfo.release_notes.slice(0, 200)}{updateInfo.release_notes.length > 200 ? '...' : ''}</p>
              {/if}

              {#if updateDone}
                <div class="settings__update-done">
                  <Icon name="check" size={16} />
                  <span>Update installed!</span>
                  <button class="btn-primary" onclick={restartApp}>
                    <Icon name="refresh" size={14} />
                    Restart Now
                  </button>
                </div>
              {:else if updating}
                <div class="settings__update-progress">
                  <span class="spinner spinner--sm"></span>
                  <span>
                    {#if updateProgress?.phase === "downloading"}
                      Downloading... {formatBytes(updateProgress.downloaded)}{updateProgress.total ? ` / ${formatBytes(updateProgress.total)}` : ''}
                    {:else if updateProgress?.phase === "installing"}
                      Installing update...
                    {:else}
                      Preparing...
                    {/if}
                  </span>
                  {#if updateProgress?.total}
                    <div class="settings__progress-bar">
                      <div
                        class="settings__progress-fill"
                        style="width: {Math.round((updateProgress.downloaded / updateProgress.total) * 100)}%"
                      ></div>
                    </div>
                  {/if}
                </div>
              {:else}
                <button class="btn-primary settings__update-btn" onclick={applyUpdate}>
                  <Icon name="download" size={14} />
                  Update & Restart
                </button>
              {/if}
            </div>
          {:else}
            <div class="settings__health-summary">
              <span class="badge badge--success">
                <Icon name="check" size={12} />
                Up to date
              </span>
            </div>
          {/if}
        {/if}
      </section>

      <section class="settings__section">
        <div class="settings__health-header">
          <h3>Package Manager</h3>
          <button class="btn-ghost" onclick={runPreScan} disabled={preScanLoading}>
            {#if preScanLoading}
              <span class="spinner spinner--sm"></span>
            {:else}
              <Icon name="refresh" size={14} />
            {/if}
            {preScanLoading ? "Scanning..." : "Scan System"}
          </button>
        </div>
        <p class="settings__desc">Check what's installed on your system and install missing packages.</p>

        {#if preScan}
          {#if preScan.installed.length > 0}
            <div class="settings__pkg-group">
              <h4 class="settings__pkg-label">Installed</h4>
              <div class="settings__packages">
                {#each preScan.installed as item}
                  <div class="settings__package-row settings__package-row--installed">
                    <div class="settings__package-info">
                      <span class="status-dot status-dot--running"></span>
                      <span class="settings__package-name">{item.name}</span>
                      {#if item.version_number}
                        <span class="badge badge--neutral">{item.version_number}</span>
                      {/if}
                    </div>
                    <span class="settings__package-version">{item.version}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          {#if missingPackages.length > 0}
            <div class="settings__pkg-group">
              <h4 class="settings__pkg-label">Available to Install</h4>
              <div class="settings__packages">
                {#each missingPackages as pkg}
                  <div class="settings__package-row">
                    <div class="settings__package-info">
                      <span class="status-dot status-dot--stopped"></span>
                      <span class="settings__package-name">{pkg.name}</span>
                      <span class="badge badge--neutral">{pkg.category}</span>
                    </div>
                    <button
                      class="btn-primary btn-sm"
                      onclick={() => installPkg(pkg.id)}
                      disabled={installingPkg === pkg.id}
                    >
                      {#if installingPkg === pkg.id}
                        <span class="spinner spinner--sm"></span>
                      {:else}
                        <Icon name="download" size={12} />
                      {/if}
                      {installingPkg === pkg.id ? "Installing..." : "Install"}
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {:else}
            <div class="settings__health-summary">
              <span class="badge badge--success">
                <Icon name="check" size={12} />
                All packages installed
              </span>
            </div>
          {/if}
        {:else}
          <p class="settings__muted">Click "Scan System" to detect installed packages.</p>
        {/if}
      </section>

      <section class="settings__section">
        <div class="settings__health-header">
          <h3>Health Check</h3>
          <button class="btn-ghost" onclick={runHealthCheck} disabled={healthLoading}>
            {#if healthLoading}
              <span class="spinner spinner--sm"></span>
            {:else}
              <Icon name="shield" size={14} />
            {/if}
            {healthLoading ? "Checking..." : "Run Check"}
          </button>
        </div>
        <p class="settings__desc">Verify all dependencies, services, and configuration.</p>
        {#if healthResult}
          <div class="settings__health">
            {#each healthResult.checks as check}
              <div class="settings__health-row">
                <span class="status-dot status-dot--{check.status === 'ok' ? 'running' : check.status === 'missing' ? 'stopped' : 'error'}"></span>
                <span class="settings__health-name">{check.name}</span>
                <span class="settings__health-detail" title={check.detail}>{check.detail}</span>
              </div>
            {/each}
            <div class="settings__health-summary">
              {#if healthResult.all_ok}
                <span class="badge badge--success">
                  <Icon name="check" size={12} />
                  All checks passed
                </span>
              {:else}
                <span class="badge badge--warning">
                  <Icon name="alert-circle" size={12} />
                  Some dependencies missing
                </span>
              {/if}
            </div>
          </div>
        {/if}
      </section>

      <section class="settings__section">
        <h3>Installed Packages</h3>
        <p class="settings__desc">Manage packages installed by Unlavarel. Uninstalling will stop the service and remove the package.</p>
        {#if installedServices.length > 0}
          <div class="settings__packages">
            {#each installedServices as svc}
              <div class="settings__package-row">
                <div class="settings__package-info">
                  <span class="settings__package-name">{svc.display_name}</span>
                  <span class="badge badge--neutral">{svc.category}</span>
                </div>
                <button
                  class="btn-danger btn-sm"
                  onclick={() => uninstallPkg(svc.id)}
                  disabled={uninstalling === svc.id}
                >
                  {#if uninstalling === svc.id}
                    <span class="spinner spinner--sm"></span>
                  {:else}
                    <Icon name="trash" size={12} />
                  {/if}
                  {uninstalling === svc.id ? "Removing..." : "Uninstall"}
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <p class="settings__muted">No packages discovered yet. Run a Health Check first.</p>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .settings {
    max-width: 600px;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .settings__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .settings__header h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .settings__loading {
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--space-8);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
  }

  .settings__sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .settings__section {
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .settings__section h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .settings__desc {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: calc(-1 * var(--space-2));
  }

  .settings__field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .settings__field label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .settings__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  /* CSS-only toggle switch */
  .settings__switch {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    cursor: pointer;
    font-size: var(--text-sm);
  }

  .settings__switch input {
    position: absolute;
    opacity: 0;
    width: 0;
    height: 0;
  }

  .settings__switch-track {
    width: 36px;
    height: 20px;
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-full);
    position: relative;
    transition: background var(--transition-fast);
    border: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .settings__switch-track::after {
    content: '';
    position: absolute;
    left: 2px;
    top: 2px;
    width: 14px;
    height: 14px;
    background: var(--color-text-muted);
    border-radius: 50%;
    transition: transform var(--transition-spring), background var(--transition-fast);
  }

  .settings__switch input:checked + .settings__switch-track {
    background: var(--color-accent-subtle);
    border-color: var(--color-accent);
  }

  .settings__switch input:checked + .settings__switch-track::after {
    transform: translateX(16px);
    background: var(--color-accent);
  }

  .settings__actions {
    display: flex;
    justify-content: flex-end;
  }

  .settings__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .settings__health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .settings__health-header button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .settings__health {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .settings__health-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--color-border-subtle);
    font-size: var(--text-sm);
  }

  .settings__health-name {
    font-weight: var(--font-medium);
    min-width: 100px;
    flex-shrink: 0;
  }

  .settings__health-detail {
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    word-break: break-all;
  }

  .settings__health-summary {
    padding-top: var(--space-3);
    display: flex;
    justify-content: flex-end;
  }

  .settings__muted {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--space-4);
  }

  .settings__packages {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .settings__package-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-subtle);
  }

  .settings__package-row:hover {
    background: var(--color-bg-hover);
  }

  .settings__package-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .settings__package-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .settings__package-row button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  /* Update section */
  .settings__update-available {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: var(--space-3);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-sm);
    background: var(--color-accent-subtle);
  }

  .settings__update-info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .settings__update-date {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .settings__update-notes {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .settings__update-btn {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    align-self: flex-start;
    text-decoration: none;
  }

  .settings__update-progress {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .settings__update-progress > span {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .settings__progress-bar {
    width: 100%;
    height: 6px;
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-full);
    overflow: hidden;
  }

  .settings__progress-fill {
    height: 100%;
    background: var(--color-accent);
    border-radius: var(--radius-full);
    transition: width 200ms ease;
  }

  .settings__update-done {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-success, #34d399);
  }

  .settings__update-done button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    margin-left: auto;
  }

  /* Package manager */
  .settings__pkg-group {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .settings__pkg-label {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .settings__package-row--installed {
    background: transparent;
  }

  .settings__package-version {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
