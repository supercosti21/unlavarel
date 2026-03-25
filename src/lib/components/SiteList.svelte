<script>
  import Icon from "./Icon.svelte";

  let { projects = [], onAdd, onImport, onRemove, onOpen } = $props();
</script>

<div class="site-list">
  <div class="site-list__header">
    <div class="site-list__title-row">
      <h2 class="site-list__title">Projects</h2>
      {#if projects.length > 0}
        <span class="badge badge--neutral">{projects.length}</span>
      {/if}
    </div>
    <div class="site-list__actions">
      <button class="btn-ghost" onclick={onImport}>
        <Icon name="upload" size={14} />
        Import Existing
      </button>
      <button class="btn-primary" onclick={onAdd}>
        <Icon name="plus" size={14} />
        New Site
      </button>
    </div>
  </div>

  {#if projects.length === 0}
    <div class="site-list__empty">
      <Icon name="folder" size={32} />
      <p>No projects configured yet.</p>
      <p class="site-list__hint">Add a project folder to get started.</p>
    </div>
  {:else}
    <div class="site-list__items">
      {#each projects as project}
        <div class="site-list__item">
          <div class="site-list__item-info">
            <div class="site-list__item-top">
              <span class="site-list__item-name">{project.name}</span>
              <span class="site-list__item-domain mono">{project.domain}</span>
              {#if project.ssl}
                <span class="badge badge--success">
                  <Icon name="lock" size={10} />
                  SSL
                </span>
              {/if}
            </div>
            <span class="site-list__item-path">{project.path}</span>
          </div>
          <div class="site-list__item-actions">
            <button class="btn-icon" onclick={() => onOpen(project.name)} aria-label="Open in browser">
              <Icon name="external-link" size={16} />
            </button>
            <button class="btn-icon" onclick={() => onRemove(project.name)} aria-label="Remove project">
              <Icon name="trash" size={16} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .site-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .site-list__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .site-list__actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .site-list__actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .site-list__title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .site-list__title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .site-list__empty {
    padding: var(--space-8) var(--space-4);
    text-align: center;
    color: var(--color-text-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
  }

  .site-list__hint {
    font-size: var(--text-xs);
  }

  .site-list__items {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .site-list__item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border-subtle);
    transition: background var(--transition-fast);
  }

  .site-list__item:hover {
    background: var(--color-bg-hover);
  }

  .site-list__item-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .site-list__item-top {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .site-list__item-name {
    font-weight: var(--font-medium);
    color: var(--color-text-primary);
  }

  .site-list__item-domain {
    font-size: var(--text-xs);
    color: var(--color-accent);
  }

  .site-list__item-path {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .site-list__item-actions {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    flex-shrink: 0;
  }
</style>
