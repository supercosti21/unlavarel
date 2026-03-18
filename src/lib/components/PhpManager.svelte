<script>
  import { invoke } from "@tauri-apps/api/core";

  let versions = $state([]);
  let extensions = $state([]);
  let loadingVersions = $state(true);
  let loadingExtensions = $state(true);
  let switching = $state(false);
  let message = $state(null);

  $effect(() => {
    loadAll();
  });

  async function loadAll() {
    loadVersions();
    loadExtensions();
  }

  async function loadVersions() {
    loadingVersions = true;
    try {
      versions = await invoke("get_php_versions");
    } catch (e) {
      versions = [];
    } finally {
      loadingVersions = false;
    }
  }

  async function loadExtensions() {
    loadingExtensions = true;
    try {
      extensions = await invoke("get_php_extensions");
    } catch (e) {
      extensions = [];
    } finally {
      loadingExtensions = false;
    }
  }

  async function switchVersion(version) {
    switching = true;
    message = null;
    try {
      const result = await invoke("switch_php_version", { version });
      message = { type: "success", text: result };
      await loadVersions();
      await loadExtensions();
    } catch (e) {
      message = { type: "error", text: String(e) };
    } finally {
      switching = false;
    }
  }

  async function toggleExtension(name, currentEnabled) {
    message = null;
    try {
      const result = await invoke("toggle_php_extension", {
        name,
        enable: !currentEnabled,
      });
      message = { type: "success", text: result };
      await loadExtensions();
    } catch (e) {
      message = { type: "error", text: String(e) };
    }
  }
</script>

<div class="php-mgr">
  <header class="php-mgr__header">
    <h2>PHP Manager</h2>
    {#if message}
      <span class="badge" class:badge--success={message.type === "success"} class:badge--danger={message.type === "error"}>
        {message.text}
      </span>
    {/if}
  </header>

  <section class="php-mgr__section card">
    <h3>Installed Versions</h3>
    {#if loadingVersions}
      <p class="php-mgr__muted">Detecting PHP versions...</p>
    {:else if versions.length === 0}
      <p class="php-mgr__muted">No PHP versions found.</p>
    {:else}
      <div class="php-mgr__versions">
        {#each versions as ver}
          <div class="php-mgr__version" class:php-mgr__version--active={ver.active}>
            <div class="php-mgr__version-info">
              <span class="php-mgr__version-num">PHP {ver.version}</span>
              {#if ver.active}
                <span class="badge badge--success">Active</span>
              {/if}
              <span class="php-mgr__version-path mono">{ver.path}</span>
            </div>
            {#if !ver.active}
              <button class="btn-primary" onclick={() => switchVersion(ver.version)} disabled={switching}>
                {switching ? "..." : "Activate"}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="php-mgr__section card">
    <h3>Extensions</h3>
    {#if loadingExtensions}
      <p class="php-mgr__muted">Loading extensions...</p>
    {:else}
      <div class="php-mgr__extensions">
        {#each extensions as ext}
          <label class="php-mgr__ext">
            <input
              type="checkbox"
              checked={ext.enabled}
              onchange={() => toggleExtension(ext.name, ext.enabled)}
            />
            <span class="php-mgr__ext-name">{ext.name}</span>
            <span class="badge" class:badge--success={ext.enabled} class:badge--neutral={!ext.enabled}>
              {ext.enabled ? "On" : "Off"}
            </span>
          </label>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .php-mgr {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .php-mgr__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .php-mgr__header h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .php-mgr__section h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-3);
  }

  .php-mgr__muted {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .php-mgr__versions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .php-mgr__version {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-subtle);
  }

  .php-mgr__version--active {
    border-color: var(--color-success);
    background: var(--color-success-subtle);
  }

  .php-mgr__version-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .php-mgr__version-num {
    font-weight: var(--font-semibold);
  }

  .php-mgr__version-path {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .php-mgr__extensions {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: var(--space-2);
  }

  .php-mgr__ext {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    transition: background var(--transition-fast);
  }

  .php-mgr__ext:hover {
    background: var(--color-bg-hover);
  }

  .php-mgr__ext input {
    accent-color: var(--color-accent);
  }

  .php-mgr__ext-name {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
</style>
