<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let configFiles = $state([]);
  let loading = $state(true);
  let selectedFile = $state(null);
  let fileContent = $state("");
  let saving = $state(false);
  let loadingContent = $state(false);

  $effect(() => {
    loadConfigFiles();
  });

  async function loadConfigFiles() {
    loading = true;
    try {
      configFiles = await invoke("list_config_files");
    } catch (e) {
      toastStore.error("Failed to load config files: " + e);
    } finally {
      loading = false;
    }
  }

  async function selectFile(file) {
    selectedFile = file;
    loadingContent = true;
    try {
      fileContent = await invoke("read_config_file", { path: file.path });
    } catch (e) {
      toastStore.error("Failed to read file: " + e);
      fileContent = "";
    } finally {
      loadingContent = false;
    }
  }

  async function saveFile() {
    if (!selectedFile) return;
    saving = true;
    try {
      await invoke("write_config_file", { path: selectedFile.path, content: fileContent });
      toastStore.success("Config saved");
    } catch (e) {
      toastStore.error("Failed to save: " + e);
    } finally {
      saving = false;
    }
  }

  // Group files by category
  let grouped = $derived(() => {
    const groups = {};
    for (const file of configFiles) {
      if (!groups[file.category]) groups[file.category] = [];
      groups[file.category].push(file);
    }
    return groups;
  });
</script>

<div class="config-editor">
  <header class="config-editor__header">
    <h2>Configuration Files</h2>
    <p class="config-editor__desc">Edit php.ini, nginx.conf, MySQL, and DNS configuration files directly.</p>
  </header>

  {#if loading}
    <div class="config-editor__loading">
      <span class="spinner"></span>
      <span>Scanning for config files...</span>
    </div>
  {:else}
    <div class="config-editor__layout">
      <div class="config-editor__sidebar">
        {#each Object.entries(grouped()) as [category, files]}
          <div class="config-editor__group">
            <h4 class="config-editor__group-label">{category}</h4>
            {#each files as file}
              <button
                class="config-editor__file"
                class:config-editor__file--active={selectedFile?.path === file.path}
                onclick={() => selectFile(file)}
              >
                <Icon name="code" size={14} />
                <span>{file.name}</span>
              </button>
            {/each}
          </div>
        {/each}

        {#if configFiles.length === 0}
          <p class="config-editor__empty">No config files found. Install services first.</p>
        {/if}
      </div>

      <div class="config-editor__main">
        {#if selectedFile}
          <div class="config-editor__toolbar">
            <span class="config-editor__path mono">{selectedFile.path}</span>
            <button class="btn-primary" onclick={saveFile} disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </button>
          </div>
          {#if loadingContent}
            <div class="config-editor__loading">
              <span class="spinner spinner--sm"></span>
              <span>Loading...</span>
            </div>
          {:else}
            <textarea
              class="config-editor__textarea"
              bind:value={fileContent}
              spellcheck="false"
            ></textarea>
          {/if}
        {:else}
          <div class="config-editor__placeholder">
            <Icon name="code" size={32} />
            <p>Select a config file to edit</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .config-editor {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    height: 100%;
  }

  .config-editor__header h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .config-editor__desc {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .config-editor__loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-8);
    color: var(--color-text-muted);
  }

  .config-editor__layout {
    display: flex;
    gap: var(--space-4);
    flex: 1;
    min-height: 0;
  }

  .config-editor__sidebar {
    width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    overflow-y: auto;
  }

  .config-editor__group {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .config-editor__group-label {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 0 var(--space-2);
  }

  .config-editor__file {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-2);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    text-align: left;
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
    width: 100%;
  }

  .config-editor__file:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .config-editor__file--active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .config-editor__main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .config-editor__toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
  }

  .config-editor__path {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .config-editor__textarea {
    flex: 1;
    resize: none;
    border: none;
    padding: var(--space-3);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.6;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    tab-size: 4;
  }

  .config-editor__textarea:focus {
    outline: none;
  }

  .config-editor__placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .config-editor__empty {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    text-align: center;
    padding: var(--space-4);
  }
</style>
