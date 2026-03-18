<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onCreated, onClose } = $props();

  let templates = $state([]);
  let selectedTemplate = $state("laravel");
  let projectName = $state("");
  let parentDir = $state("");
  let loading = $state(true);
  let creating = $state(false);
  let error = $state(null);

  $effect(() => {
    loadTemplates();
    loadDefaultDir();
  });

  async function loadTemplates() {
    try {
      templates = await invoke("get_templates");
    } catch (e) {
      templates = [];
    } finally {
      loading = false;
    }
  }

  async function loadDefaultDir() {
    try {
      const settings = await invoke("get_settings");
      parentDir = settings.project_root;
    } catch {
      parentDir = "";
    }
  }

  async function create() {
    if (!projectName.trim()) {
      error = "Project name is required";
      return;
    }
    creating = true;
    error = null;
    try {
      const path = await invoke("create_app", {
        templateId: selectedTemplate,
        name: projectName.trim(),
        parentDir,
      });
      // Auto-register as project
      await invoke("add_project", {
        name: projectName.trim(),
        path,
        createDb: true,
      });
      onCreated(projectName.trim(), path);
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }

  let selectedInfo = $derived(templates.find((t) => t.id === selectedTemplate));
</script>

<div class="dialog-overlay" role="presentation" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <header class="dialog__header">
      <h2>Create New Project</h2>
      <button class="btn-ghost" onclick={onClose}>Close</button>
    </header>

    <div class="dialog__body">
      {#if loading}
        <p class="dialog__loading">Loading templates...</p>
      {:else}
        <div class="dialog__field">
          <label for="template">Template</label>
          <div class="dialog__templates">
            {#each templates as tmpl}
              <button
                class="dialog__template"
                class:dialog__template--active={selectedTemplate === tmpl.id}
                onclick={() => (selectedTemplate = tmpl.id)}
              >
                <span class="dialog__template-name">{tmpl.name}</span>
                <span class="dialog__template-desc">{tmpl.description}</span>
              </button>
            {/each}
          </div>
        </div>

        <div class="dialog__field">
          <label for="name">Project Name</label>
          <input
            id="name"
            type="text"
            bind:value={projectName}
            placeholder="my-app"
            autofocus
          />
        </div>

        <div class="dialog__field">
          <label for="dir">Parent Directory</label>
          <input id="dir" type="text" bind:value={parentDir} />
        </div>

        {#if selectedInfo}
          <div class="dialog__preview">
            <span class="mono">{parentDir}/{projectName || "my-app"}</span>
            <span class="dialog__preview-domain">{(projectName || "my-app").toLowerCase()}.test</span>
          </div>
        {/if}

        {#if error}
          <div class="dialog__error">{error}</div>
        {/if}
      {/if}
    </div>

    <footer class="dialog__footer">
      <button class="btn-ghost" onclick={onClose}>Cancel</button>
      <button class="btn-primary" onclick={create} disabled={creating || !projectName.trim()}>
        {creating ? "Creating..." : "Create Project"}
      </button>
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
    width: 520px;
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

  .dialog__body {
    padding: var(--space-5);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .dialog__loading {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-6);
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

  .dialog__templates {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .dialog__template {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-primary);
    text-align: left;
    border: 1px solid var(--color-border-subtle);
    transition: all var(--transition-fast);
  }

  .dialog__template:hover {
    background: var(--color-bg-hover);
  }

  .dialog__template--active {
    border-color: var(--color-accent);
    background: var(--color-accent-subtle);
  }

  .dialog__template-name {
    font-weight: var(--font-semibold);
    font-size: var(--text-sm);
  }

  .dialog__template-desc {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
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
