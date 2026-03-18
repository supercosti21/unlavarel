<script>
  let { service, onToggle, onRestart } = $props();

  let isRunning = $derived(service.status === "Running");
  let isInstalled = $derived(service.status === "Installed");
  let statusClass = $derived(
    service.status === "Running"
      ? "running"
      : service.status === "Stopped" || service.status === "Installed"
        ? "stopped"
        : "error"
  );
  let toggling = $state(false);
  let serviceKey = $derived(service.id || service.name);

  async function handleToggle() {
    toggling = true;
    await onToggle(serviceKey, isRunning);
    toggling = false;
  }

  async function handleRestart() {
    toggling = true;
    await onRestart(serviceKey);
    toggling = false;
  }
</script>

<div class="service-card card">
  <div class="service-card__header">
    <div class="service-card__info">
      <span class="status-dot status-dot--{statusClass}"></span>
      <h3 class="service-card__name">{service.name}</h3>
    </div>
    <span class="badge" class:badge--success={isRunning} class:badge--danger={!isRunning && !isInstalled} class:badge--neutral={isInstalled}>
      {service.status}
    </span>
  </div>

  <div class="service-card__meta">
    {#if service.version}
      <span class="service-card__version mono">{service.version}</span>
    {/if}
  </div>

  {#if service.has_service}
  <div class="service-card__actions">
    <button
      class="btn-primary"
      class:btn-danger={isRunning}
      onclick={handleToggle}
      disabled={toggling}
    >
      {toggling ? "..." : isRunning ? "Stop" : "Start"}
    </button>
    {#if isRunning}
      <button class="btn-ghost" onclick={handleRestart} disabled={toggling}>
        Restart
      </button>
    {/if}
  </div>
  {/if}
</div>

<style>
  .service-card {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .service-card__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .service-card__info {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .service-card__name {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .service-card__meta {
    min-height: 1.25rem;
  }

  .service-card__version {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .service-card__actions {
    display: flex;
    gap: var(--space-2);
  }

  .service-card__actions .btn-danger {
    background: var(--color-danger-subtle);
    color: var(--color-danger);
  }

  .service-card__actions .btn-danger:hover:not(:disabled) {
    background: var(--color-danger);
    color: var(--color-text-on-accent);
  }
</style>
