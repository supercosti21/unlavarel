<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { servicesStore } from "../stores/services.svelte.js";

  let { services = [] } = $props();

  let runningCount = $derived(
    services.filter((s) => s.status === "Running").length
  );
  let totalCount = $derived(services.length);
  let phpVersion = $state(null);
  let refreshing = $state(false);

  $effect(() => {
    invoke("get_settings").then(s => {
      phpVersion = s.default_php_version;
    }).catch(() => {});
  });

  async function refreshServices() {
    refreshing = true;
    try {
      await servicesStore.loadServices();
    } finally {
      refreshing = false;
    }
  }
</script>

<footer class="statusbar">
  <div class="statusbar__left">
    <span class="statusbar__item">
      <span class="status-dot status-dot--{runningCount > 0 ? 'running' : 'stopped'}"></span>
      {runningCount}/{totalCount} services
    </span>
    {#if phpVersion}
      <span class="statusbar__item statusbar__divider">PHP {phpVersion}</span>
    {/if}
  </div>
  <div class="statusbar__right">
    <button class="btn-icon statusbar__refresh" onclick={refreshServices} aria-label="Refresh services" disabled={refreshing}>
      <Icon name="refresh" size={12} />
    </button>
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
    gap: var(--space-3);
  }

  .statusbar__item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .statusbar__divider {
    padding-left: var(--space-3);
    border-left: 1px solid var(--color-border);
  }

  .statusbar__refresh {
    padding: 2px;
  }

  .statusbar__refresh:disabled {
    animation: spin 0.7s linear infinite;
  }
</style>
