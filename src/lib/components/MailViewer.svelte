<script>
  let { mailpitUrl = "http://localhost:8025" } = $props();
  let available = $state(false);
  let checking = $state(true);

  $effect(() => {
    checkMailpit();
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
        <span class="badge badge--neutral">Checking...</span>
      {:else if available}
        <span class="badge badge--success">Mailpit Running</span>
      {:else}
        <span class="badge badge--danger">Mailpit Offline</span>
      {/if}
      <button class="btn-ghost" onclick={checkMailpit}>Refresh</button>
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
      <p>Mailpit is not running.</p>
      <p class="mail-viewer__hint">Start the Mailpit service from the Dashboard to view captured emails.</p>
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
    font-size: var(--text-base);
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
    gap: var(--space-2);
  }

  .mail-viewer__hint {
    font-size: var(--text-xs);
  }
</style>
