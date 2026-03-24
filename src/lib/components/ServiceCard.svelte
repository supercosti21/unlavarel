<script>
  import Icon from "./Icon.svelte";

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

  const iconMap = {
    nginx: "globe",
    php: "code",
    mysql: "database",
    mariadb: "database",
    postgresql: "database",
    redis: "zap",
    memcached: "zap",
    dnsmasq: "globe",
    mailpit: "mail",
  };

  let serviceIcon = $derived(iconMap[serviceKey] || iconMap[service.name?.toLowerCase()] || "settings");

  async function handleToggle() {
    toggling = true;
    try {
      await onToggle(serviceKey, isRunning);
    } finally {
      toggling = false;
    }
  }

  async function handleRestart() {
    toggling = true;
    try {
      await onRestart(serviceKey);
    } finally {
      toggling = false;
    }
  }
</script>

<div class="service-card card" class:service-card--running={isRunning} role="article" aria-label="{service.name} service">
  <div class="service-card__header">
    <div class="service-card__info">
      <span class="service-card__icon">
        <Icon name={serviceIcon} size={16} />
      </span>
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
      {#if toggling}
        <span class="spinner spinner--sm"></span>
      {:else}
        <Icon name={isRunning ? "stop" : "play"} size={14} />
        {isRunning ? "Stop" : "Start"}
      {/if}
    </button>
    {#if isRunning}
      <button class="btn-ghost" onclick={handleRestart} disabled={toggling}>
        {#if toggling}
          <span class="spinner spinner--sm"></span>
        {:else}
          <Icon name="refresh" size={14} />
          Restart
        {/if}
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
    transition: border-color var(--transition-normal), box-shadow var(--transition-normal);
  }

  .service-card--running {
    border-color: color-mix(in srgb, var(--color-success) 30%, transparent);
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

  .service-card__icon {
    color: var(--color-text-muted);
    display: flex;
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
    display: block;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .service-card__actions {
    display: flex;
    gap: var(--space-2);
  }

  .service-card__actions button {
    min-width: 72px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
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
