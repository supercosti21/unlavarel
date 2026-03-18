<script>
  import { invoke } from "@tauri-apps/api/core";

  let settings = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let message = $state(null);
  let healthResult = $state(null);
  let healthLoading = $state(false);
  let phpVersions = $state([]);
  let installedServices = $state([]);
  let uninstalling = $state(null);

  $effect(() => {
    loadSettings();
    loadPhpVersions();
    loadInstalledServices();
  });

  async function loadSettings() {
    loading = true;
    try {
      settings = await invoke("get_settings");
    } catch (e) {
      message = { type: "error", text: String(e) };
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
      message = { type: "success", text: result };
      await loadInstalledServices();
    } catch (e) {
      message = { type: "error", text: String(e) };
    } finally {
      uninstalling = null;
    }
  }

  async function runHealthCheck() {
    healthLoading = true;
    try {
      healthResult = await invoke("health_check");
    } catch (e) {
      message = { type: "error", text: String(e) };
    } finally {
      healthLoading = false;
    }
  }

  async function save() {
    saving = true;
    message = null;
    try {
      await invoke("save_settings", { settings });
      message = { type: "success", text: "Settings saved" };
      // Apply theme immediately
      document.documentElement.setAttribute("data-theme", settings.theme);
    } catch (e) {
      message = { type: "error", text: String(e) };
    } finally {
      saving = false;
    }
  }
</script>

<div class="settings">
  <header class="settings__header">
    <h2>Settings</h2>
    {#if message}
      <span class="badge" class:badge--success={message.type === "success"} class:badge--danger={message.type === "error"}>
        {message.text}
      </span>
    {/if}
  </header>

  {#if loading}
    <p class="settings__loading">Loading settings...</p>
  {:else if settings}
    <div class="settings__sections">
      <section class="settings__section">
        <h3>Appearance</h3>
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
        <label class="settings__toggle">
          <input type="checkbox" bind:checked={settings.auto_start_services} />
          Auto-start services on app launch
        </label>
      </section>

      <div class="settings__actions">
        <button class="btn-primary" onclick={save} disabled={saving}>
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </div>

      <section class="settings__section">
        <div class="settings__health-header">
          <h3>Health Check</h3>
          <button class="btn-ghost" onclick={runHealthCheck} disabled={healthLoading}>
            {healthLoading ? "Checking..." : "Run Check"}
          </button>
        </div>
        {#if healthResult}
          <div class="settings__health">
            {#each healthResult.checks as check}
              <div class="settings__health-row">
                <span class="status-dot status-dot--{check.status === 'ok' ? 'running' : check.status === 'missing' ? 'stopped' : 'error'}"></span>
                <span class="settings__health-name">{check.name}</span>
                <span class="settings__health-detail">{check.detail}</span>
              </div>
            {/each}
            <div class="settings__health-summary">
              {#if healthResult.all_ok}
                <span class="badge badge--success">All checks passed</span>
              {:else}
                <span class="badge badge--warning">Some dependencies missing</span>
              {/if}
            </div>
          </div>
        {/if}
      </section>

      <section class="settings__section">
        <h3>Installed Packages</h3>
        <p class="settings__hint">Manage packages installed by MacEnv. Uninstalling will stop the service and remove the package.</p>
        {#if installedServices.length > 0}
          <div class="settings__packages">
            {#each installedServices as svc}
              <div class="settings__package-row">
                <div class="settings__package-info">
                  <span class="settings__package-name">{svc.display_name}</span>
                  <span class="settings__package-cat badge badge--neutral">{svc.category}</span>
                </div>
                <button
                  class="btn-danger"
                  onclick={() => uninstallPkg(svc.id)}
                  disabled={uninstalling === svc.id}
                >
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

  .settings__toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .settings__toggle input {
    accent-color: var(--color-accent);
  }

  .settings__actions {
    display: flex;
    justify-content: flex-end;
  }

  .settings__health-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
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
    min-width: 120px;
  }

  .settings__health-detail {
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .settings__health-summary {
    padding-top: var(--space-3);
    display: flex;
    justify-content: flex-end;
  }

  .settings__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
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
</style>
