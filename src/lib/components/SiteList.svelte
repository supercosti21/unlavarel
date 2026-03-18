<script>
  let { projects = [], onAdd, onRemove, onOpen } = $props();
</script>

<div class="site-list">
  <div class="site-list__header">
    <h2 class="site-list__title">Projects</h2>
    <button class="btn-primary" onclick={onAdd}>Add Site</button>
  </div>

  {#if projects.length === 0}
    <div class="site-list__empty">
      <p>No projects configured yet.</p>
      <p class="site-list__hint">Add a project folder to get started.</p>
    </div>
  {:else}
    <div class="site-list__items">
      {#each projects as project}
        <div class="site-list__item">
          <div class="site-list__item-info">
            <span class="site-list__item-name">{project.name}</span>
            <span class="site-list__item-domain mono">{project.domain}</span>
            <span class="site-list__item-path">{project.path}</span>
          </div>
          <div class="site-list__item-actions">
            {#if project.ssl}
              <span class="badge badge--success">SSL</span>
            {/if}
            <button class="btn-ghost" onclick={() => onOpen(project.name)}>Open</button>
            <button class="btn-ghost" onclick={() => onRemove(project.name)}>Remove</button>
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

  .site-list__title {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
  }

  .site-list__empty {
    padding: var(--space-8) var(--space-4);
    text-align: center;
    color: var(--color-text-muted);
  }

  .site-list__hint {
    font-size: var(--text-xs);
    margin-top: var(--space-2);
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
    align-items: center;
    gap: var(--space-3);
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
  }

  .site-list__item-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
</style>
