<script>
  let { adminerUrl = "http://localhost:8080" } = $props();
  let available = $state(false);
  let checking = $state(true);

  $effect(() => {
    checkAdminer();
  });

  async function checkAdminer() {
    checking = true;
    try {
      const response = await fetch(adminerUrl, { mode: "no-cors" });
      available = true;
    } catch {
      available = false;
    } finally {
      checking = false;
    }
  }
</script>

<div class="db-viewer">
  <div class="db-viewer__header">
    <h2 class="db-viewer__title">Database</h2>
    <div class="db-viewer__status">
      {#if checking}
        <span class="badge badge--neutral">Checking...</span>
      {:else if available}
        <span class="badge badge--success">Adminer Running</span>
      {:else}
        <span class="badge badge--danger">Adminer Offline</span>
      {/if}
      <button class="btn-ghost" onclick={checkAdminer}>Refresh</button>
    </div>
  </div>

  {#if available}
    <iframe
      class="db-viewer__frame"
      src={adminerUrl}
      title="Database Viewer"
    ></iframe>
  {:else if !checking}
    <div class="db-viewer__offline">
      <p>Database viewer is not available.</p>
      <p class="db-viewer__hint">
        Install Adminer or use the database CLI tools directly.
        You can also connect via your preferred GUI tool
        (TablePlus, DBeaver, HeidiSQL).
      </p>
      <div class="db-viewer__connections">
        <h3>Connection Details</h3>
        <div class="db-viewer__conn-info">
          <div class="db-viewer__conn-row">
            <span>MySQL/MariaDB</span>
            <code>mysql -u root</code>
          </div>
          <div class="db-viewer__conn-row">
            <span>PostgreSQL</span>
            <code>psql -U postgres</code>
          </div>
          <div class="db-viewer__conn-row">
            <span>Host</span>
            <code>127.0.0.1</code>
          </div>
          <div class="db-viewer__conn-row">
            <span>MySQL Port</span>
            <code>3306</code>
          </div>
          <div class="db-viewer__conn-row">
            <span>PostgreSQL Port</span>
            <code>5432</code>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .db-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: var(--space-4);
  }

  .db-viewer__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .db-viewer__title {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
  }

  .db-viewer__status {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .db-viewer__frame {
    flex: 1;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-bg-secondary);
    min-height: 400px;
  }

  .db-viewer__offline {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    gap: var(--space-4);
    text-align: center;
  }

  .db-viewer__hint {
    font-size: var(--text-xs);
    max-width: 400px;
  }

  .db-viewer__connections {
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
    width: 100%;
    max-width: 400px;
    text-align: left;
  }

  .db-viewer__connections h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-3);
  }

  .db-viewer__conn-info {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .db-viewer__conn-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: var(--text-xs);
  }

  .db-viewer__conn-row code {
    font-family: var(--font-mono);
    color: var(--color-accent);
    background: var(--color-bg-tertiary);
    padding: 2px var(--space-2);
    border-radius: var(--radius-sm);
  }
</style>
