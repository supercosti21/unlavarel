<script>
  import { invoke } from "@tauri-apps/api/core";

  let settings = $state(null);
  let loading = $state(true);
  let saving = $state(false);
  let message = $state(null);

  $effect(() => {
    loadSettings();
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
            <option value="8.1">PHP 8.1</option>
            <option value="8.2">PHP 8.2</option>
            <option value="8.3">PHP 8.3</option>
            <option value="8.4">PHP 8.4</option>
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
</style>
