<script>
  let { services = [] } = $props();

  let runningCount = $derived(
    services.filter((s) => s.status === "Running").length
  );
  let totalCount = $derived(services.length);
</script>

<footer class="statusbar">
  <div class="statusbar__left">
    <span class="statusbar__item">
      <span class="status-dot status-dot--{runningCount > 0 ? 'running' : 'stopped'}"></span>
      {runningCount}/{totalCount} services
    </span>
  </div>
  <div class="statusbar__right">
    <span class="statusbar__item mono">MacEnv v0.1.0</span>
  </div>
</footer>

<style>
  .statusbar {
    height: var(--statusbar-height);
    background: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 var(--space-4);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .statusbar__left,
  .statusbar__right {
    display: flex;
    align-items: center;
    gap: var(--space-4);
  }

  .statusbar__item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
</style>
