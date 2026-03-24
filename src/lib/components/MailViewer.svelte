<script>
  import Icon from "./Icon.svelte";

  let { mailpitUrl = "http://localhost:8025" } = $props();
  let available = $state(false);
  let checking = $state(true);
  let retryTimer = $state(null);

  $effect(() => {
    checkMailpit();
    // Auto-retry every 10s when offline
    retryTimer = setInterval(() => {
      if (!available) checkMailpit();
    }, 10000);
    return () => {
      if (retryTimer) clearInterval(retryTimer);
    };
  });

  async function checkMailpit() {
    checking = true;
    try {
      const response = await fetch(mailpitUrl, { mode: "no-cors" });
      available = true;
    } catch {
      available = false;
    } finally {
      checking = false;
    }
  }
</script>

<div class="mail-viewer">
  <div class="mail-viewer__header">
    <h2 class="mail-viewer__title">Mail Inbox</h2>
    <div class="mail-viewer__status">
      {#if checking}
        <span class="badge badge--neutral">
          <span class="spinner spinner--sm"></span>
          Checking...
        </span>
      {:else if available}
        <span class="badge badge--success">
          <Icon name="check" size={10} />
          Running
        </span>
      {:else}
        <span class="badge badge--danger">
          <Icon name="x" size={10} />
          Offline
        </span>
      {/if}
      <button class="btn-icon" onclick={checkMailpit} aria-label="Refresh Mailpit status">
        <Icon name="refresh" size={16} />
      </button>
    </div>
  </div>

  {#if available}
    <iframe
      class="mail-viewer__frame"
      src={mailpitUrl}
      title="Mailpit Inbox"
    ></iframe>
  {:else if !checking}
    <div class="mail-viewer__offline">
      <Icon name="mail" size={48} />
      <h3>Mailpit is not running</h3>
      <p>Start the Mailpit service from the Dashboard to view captured emails.</p>
      <p class="mail-viewer__hint">
        Install with your package manager or via the Setup Wizard.
        <br />
        Auto-retrying every 10 seconds...
      </p>
    </div>
  {/if}
</div>

<style>
  .mail-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: var(--space-4);
  }

  .mail-viewer__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .mail-viewer__title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .mail-viewer__status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .mail-viewer__frame {
    flex: 1;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-bg-secondary);
    min-height: 400px;
  }

  .mail-viewer__offline {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    gap: var(--space-3);
    text-align: center;
  }

  .mail-viewer__offline h3 {
    font-size: var(--text-base);
    color: var(--color-text-secondary);
  }

  .mail-viewer__offline p {
    font-size: var(--text-sm);
    max-width: 360px;
    line-height: var(--leading-normal);
  }

  .mail-viewer__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }
</style>
