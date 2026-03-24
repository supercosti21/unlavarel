<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let versions = $state([]);
  let extensions = $state([]);
  let loadingVersions = $state(true);
  let loadingExtensions = $state(true);
  let switching = $state(false);

  function friendlyPhpError(raw) {
    const msg = String(raw).toLowerCase();
    if (msg.includes("not found") || msg.includes("no such")) return "PHP is not installed. Install it from the Setup Wizard.";
    if (msg.includes("permission") || msg.includes("denied")) return "Permission denied. Authenticate first.";
    if (msg.includes("password") || msg.includes("auth")) return "Admin password required. Authenticate from the Dashboard.";
    if (msg.includes("already")) return "This PHP version is already active.";
    return String(raw);
  }

  const dbExts = ["pdo_mysql", "pdo_pgsql", "pdo_sqlite", "mysqli", "pgsql", "sqlite3"];
  const cacheExts = ["redis", "memcached", "apcu", "opcache"];
  const debugExts = ["xdebug", "pcov"];

  let groupedExtensions = $derived.by(() => {
    const groups = { "Database": [], "Cache": [], "Debug": [], "Other": [] };
    for (const ext of extensions) {
      const name = ext.name.toLowerCase();
      if (dbExts.includes(name)) groups["Database"].push(ext);
      else if (cacheExts.includes(name)) groups["Cache"].push(ext);
      else if (debugExts.includes(name)) groups["Debug"].push(ext);
      else groups["Other"].push(ext);
    }
    return Object.entries(groups).filter(([, exts]) => exts.length > 0);
  });

  $effect(() => {
    loadVersions();
    loadExtensions();
  });

  async function loadVersions() {
    loadingVersions = true;
    try {
      versions = await invoke("get_php_versions");
    } catch {
      versions = [];
    } finally {
      loadingVersions = false;
    }
  }

  async function loadExtensions() {
    loadingExtensions = true;
    try {
      extensions = await invoke("get_php_extensions");
    } catch {
      extensions = [];
    } finally {
      loadingExtensions = false;
    }
  }

  async function switchVersion(version) {
    switching = true;
    try {
      const result = await invoke("switch_php_version", { version });
      toastStore.success(result);
      await loadVersions();
      await loadExtensions();
    } catch (e) {
      toastStore.error(friendlyPhpError(e));
    } finally {
      switching = false;
    }
  }

  async function toggleExtension(name, currentEnabled) {
    try {
      const result = await invoke("toggle_php_extension", {
        name,
        enable: !currentEnabled,
      });
      toastStore.success(result);
      await loadExtensions();
    } catch (e) {
      toastStore.error(friendlyPhpError(e));
    }
  }
</script>

<div class="php-mgr">
  <header class="php-mgr__header">
    <h2>PHP Manager</h2>
  </header>

  <section class="php-mgr__section card">
    <h3>
      <Icon name="code" size={14} />
      Installed Versions
    </h3>
    {#if loadingVersions}
      <div class="php-mgr__muted">
        <span class="spinner spinner--sm"></span>
        Detecting PHP versions...
      </div>
    {:else if versions.length === 0}
      <p class="php-mgr__muted">No PHP versions found.</p>
    {:else}
      <div class="php-mgr__versions">
        {#each versions as ver}
          <div class="php-mgr__version" class:php-mgr__version--active={ver.active}>
            <div class="php-mgr__version-info">
              <span class="php-mgr__version-num">PHP {ver.version}</span>
              {#if ver.active}
                <span class="badge badge--success">
                  <Icon name="check" size={10} />
                  Active
                </span>
              {/if}
              <span class="php-mgr__version-path mono">{ver.path}</span>
            </div>
            {#if !ver.active}
              <button class="btn-primary btn-sm" onclick={() => switchVersion(ver.version)} disabled={switching}>
                {#if switching}
                  <span class="spinner spinner--sm"></span>
                {:else}
                  Activate
                {/if}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <section class="php-mgr__section card">
    <h3>
      <Icon name="settings" size={14} />
      Extensions
    </h3>
    {#if loadingExtensions}
      <div class="php-mgr__muted">
        <span class="spinner spinner--sm"></span>
        Loading extensions...
      </div>
    {:else if groupedExtensions.length === 0}
      <p class="php-mgr__muted">No extensions found.</p>
    {:else}
      {#each groupedExtensions as [groupName, exts]}
        <div class="php-mgr__ext-group">
          <h4 class="php-mgr__ext-group-title">{groupName}</h4>
          <div class="php-mgr__extensions">
            {#each exts as ext}
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
        </div>
      {/each}
    {/if}
  </section>
</div>

<style>
  .php-mgr {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
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
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .php-mgr__muted {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
    border-color: color-mix(in srgb, var(--color-success) 40%, transparent);
    background: var(--color-success-subtle);
  }

  .php-mgr__version-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .php-mgr__version-num {
    font-weight: var(--font-semibold);
    flex-shrink: 0;
  }

  .php-mgr__version-path {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .php-mgr__ext-group {
    margin-bottom: var(--space-3);
  }

  .php-mgr__ext-group-title {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: var(--space-2);
    font-weight: var(--font-semibold);
  }

  .php-mgr__extensions {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
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
