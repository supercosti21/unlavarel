<script>
  import { invoke } from "@tauri-apps/api/core";

  let providers = $state([]);
  let sharing = $state(false);
  let shareInfo = $state(null);
  let error = $state(null);
  let { domain = "" } = $props();

  $effect(() => {
    loadProviders();
  });

  async function loadProviders() {
    try {
      providers = await invoke("get_sharing_providers");
    } catch {
      providers = [];
    }
  }

  async function startSharing() {
    sharing = true;
    error = null;
    try {
      shareInfo = await invoke("share_site", { domain });
    } catch (e) {
      error = String(e);
    } finally {
      sharing = false;
    }
  }

  async function stopSharing() {
    try {
      await invoke("stop_sharing", { domain });
      shareInfo = null;
    } catch (e) {
      error = String(e);
    }
  }

  function copyUrl() {
    if (shareInfo?.public_url) {
      navigator.clipboard.writeText(shareInfo.public_url);
    }
  }
</script>

<div class="sharing card">
  <h3>Share Site</h3>

  {#if providers.length === 0}
    <p class="sharing__muted">
      No tunnel providers found. Install
      <code>ngrok</code> or <code>cloudflared</code> to share your site publicly.
    </p>
  {:else if shareInfo}
    <div class="sharing__active">
      <div class="sharing__url-row">
        <span class="badge badge--success">{shareInfo.provider}</span>
        <code class="sharing__url">{shareInfo.public_url}</code>
        <button class="btn-ghost" onclick={copyUrl}>Copy</button>
      </div>
      <button class="btn-danger" onclick={stopSharing}>Stop Sharing</button>
    </div>
  {:else}
    <div class="sharing__providers">
      <span class="sharing__label">Available: {providers.join(", ")}</span>
      <button class="btn-primary" onclick={startSharing} disabled={sharing}>
        {sharing ? "Starting tunnel..." : "Share"}
      </button>
    </div>
  {/if}

  {#if error}
    <div class="sharing__error">{error}</div>
  {/if}
</div>

<style>
  .sharing h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: var(--space-3);
  }

  .sharing__muted {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .sharing__muted code {
    font-family: var(--font-mono);
    color: var(--color-accent);
    background: var(--color-bg-tertiary);
    padding: 1px var(--space-1);
    border-radius: 3px;
  }

  .sharing__active {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .sharing__url-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .sharing__url {
    flex: 1;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--color-success);
    word-break: break-all;
  }

  .sharing__providers {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sharing__label {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .sharing__error {
    margin-top: var(--space-2);
    padding: var(--space-2);
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }
</style>
