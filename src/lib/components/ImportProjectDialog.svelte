<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";

  let { onImported, onClose } = $props();

  let projectPath = $state("");
  let projectName = $state("");
  let createDb = $state(true);
  let scanning = $state(false);
  let importing = $state(false);
  let error = $state(null);
  let scannedProjects = $state([]);
  let scanDir = $state("");
  let mode = $state("manual"); // "manual" or "scan"

  $effect(() => {
    loadDefaultDir();
  });

  async function loadDefaultDir() {
    try {
      const settings = await invoke("get_settings");
      scanDir = settings.project_root;
    } catch {
      scanDir = "";
    }
  }

  async function scanDirectory() {
    if (!scanDir.trim()) return;
    scanning = true;
    error = null;
    try {
      scannedProjects = await invoke("scan_projects", { directory: scanDir.trim() });
    } catch (e) {
      error = String(e);
    } finally {
      scanning = false;
    }
  }

  async function importSingle() {
    if (!projectName.trim() || !projectPath.trim()) {
      error = "Project name and path are required";
      return;
    }
    importing = true;
    error = null;
    try {
      await invoke("import_project", {
        name: projectName.trim(),
        path: projectPath.trim(),
        createDb,
      });
      onImported(projectName.trim(), projectPath.trim());
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }

  async function importScanned(project) {
    if (project.already_added) return;
    importing = project.name;
    error = null;
    try {
      await invoke("import_project", {
        name: project.name,
        path: project.path,
        createDb,
      });
      // Mark as added in local state
      scannedProjects = scannedProjects.map((p) =>
        p.name === project.name ? { ...p, already_added: true } : p
      );
      onImported(project.name, project.path);
    } catch (e) {
      error = String(e);
    } finally {
      importing = false;
    }
  }

  // Auto-derive name from path
  function onPathInput() {
    if (!projectName && projectPath) {
      const parts = projectPath.replace(/\/$/, "").split("/");
      projectName = parts[parts.length - 1] || "";
    }
  }
</script>

<div class="dialog-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header class="dialog__header">
      <h2>Import Existing Project</h2>
      <button class="btn-ghost" onclick={onClose}>Close</button>
    </header>

    <div class="dialog__tabs">
      <button
        class="dialog__tab"
        class:dialog__tab--active={mode === "manual"}
        onclick={() => (mode = "manual")}
      >Manual</button>
      <button
        class="dialog__tab"
        class:dialog__tab--active={mode === "scan"}
        onclick={() => (mode = "scan")}
      >Scan Directory</button>
    </div>

    <div class="dialog__body">
      {#if mode === "manual"}
        <div class="dialog__field">
          <label for="import-path">Project Path</label>
          <input
            id="import-path"
            type="text"
            bind:value={projectPath}
            oninput={onPathInput}
            placeholder="/Users/you/Code/my-project"
            autofocus
          />
        </div>

        <div class="dialog__field">
          <label for="import-name">Project Name</label>
          <input
            id="import-name"
            type="text"
            bind:value={projectName}
            placeholder="my-project"
          />
        </div>

        <label class="dialog__checkbox">
          <input type="checkbox" bind:checked={createDb} />
          <span>Create database</span>
        </label>

        {#if projectName && projectPath}
          <div class="dialog__preview">
            <span class="mono">{projectPath}</span>
            <span class="dialog__preview-domain">{projectName.toLowerCase()}.test</span>
          </div>
        {/if}

      {:else}
        <div class="dialog__field">
          <label for="scan-dir">Directory to scan</label>
          <div class="dialog__scan-row">
            <input
              id="scan-dir"
              type="text"
              bind:value={scanDir}
              placeholder="/Users/you/Code"
            />
            <button class="btn-primary" onclick={scanDirectory} disabled={scanning}>
              {scanning ? "Scanning..." : "Scan"}
            </button>
          </div>
        </div>

        <label class="dialog__checkbox">
          <input type="checkbox" bind:checked={createDb} />
          <span>Create database for imported projects</span>
        </label>

        {#if scannedProjects.length > 0}
          <div class="dialog__scan-results">
            {#each scannedProjects as project}
              <div class="dialog__scan-item">
                <div class="dialog__scan-info">
                  <span class="dialog__scan-name">{project.name}</span>
                  <span class="badge badge--neutral">{project.project_type}</span>
                  <span class="dialog__scan-path">{project.path}</span>
                </div>
                {#if project.already_added}
                  <span class="badge badge--success">Added</span>
                {:else}
                  <button
                    class="btn-primary btn-sm"
                    onclick={() => importScanned(project)}
                    disabled={importing === project.name}
                  >
                    {importing === project.name ? "Importing..." : "Import"}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {:else if !scanning}
          <p class="dialog__muted">Click "Scan" to find projects in the directory.</p>
        {/if}
      {/if}

      {#if error}
        <div class="dialog__error">{error}</div>
      {/if}
    </div>

    <footer class="dialog__footer">
      <button class="btn-ghost" onclick={onClose}>Cancel</button>
      {#if mode === "manual"}
        <button
          class="btn-primary"
          onclick={importSingle}
          disabled={importing || !projectName.trim() || !projectPath.trim()}
        >
          {importing ? "Importing..." : "Import Project"}
        </button>
      {/if}
    </footer>
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    width: 580px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: var(--shadow-elevated);
  }

  .dialog__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--color-border);
  }

  .dialog__header h2 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
  }

  .dialog__tabs {
    display: flex;
    border-bottom: 1px solid var(--color-border);
  }

  .dialog__tab {
    flex: 1;
    padding: var(--space-2) var(--space-4);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .dialog__tab:hover {
    color: var(--color-text-primary);
  }

  .dialog__tab--active {
    color: var(--color-accent);
    border-bottom-color: var(--color-accent);
  }

  .dialog__body {
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .dialog__field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .dialog__field label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .dialog__scan-row {
    display: flex;
    gap: var(--space-2);
  }

  .dialog__scan-row input {
    flex: 1;
  }

  .dialog__checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
  }

  .dialog__preview {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-3);
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .dialog__preview-domain {
    color: var(--color-accent);
    font-family: var(--font-mono);
  }

  .dialog__scan-results {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-height: 300px;
    overflow-y: auto;
  }

  .dialog__scan-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
  }

  .dialog__scan-item:hover {
    background: var(--color-bg-hover);
  }

  .dialog__scan-info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-wrap: wrap;
    min-width: 0;
  }

  .dialog__scan-name {
    font-weight: var(--font-medium);
    font-size: var(--text-sm);
  }

  .dialog__scan-path {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dialog__muted {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--space-4);
  }

  .dialog__error {
    padding: var(--space-3);
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
  }

  .dialog__footer {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-5);
    border-top: 1px solid var(--color-border);
  }
</style>
