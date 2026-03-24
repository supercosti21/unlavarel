<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let providers = $state([]);
  let sharing = $state(false);
  let shareInfo = $state(null);
  let { domain = "" } = $props();
  let copyFeedback = $state(false);

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
    try {
      shareInfo = await invoke("share_site", { domain });
      toastStore.success("Site shared publicly");
    } catch (e) {
      toastStore.error(String(e));
    } finally {
      sharing = false;
    }
  }

  async function stopSharing() {
    try {
      await invoke("stop_sharing", { domain });
      shareInfo = null;
      toastStore.info("Sharing stopped");
    } catch (e) {
      toastStore.error(String(e));
    }
  }

  function copyUrl() {
    if (shareInfo?.public_url) {
      navigator.clipboard.writeText(shareInfo.public_url).then(() => {
        copyFeedback = true;
        toastStore.success("URL copied to clipboard");
        setTimeout(() => (copyFeedback = false), 2000);
      });
    }
  }
</script>

<div class="sharing">
  <h3>
    <Icon name="external-link" size={14} />
    Share Site
  </h3>

  {#if providers.length === 0}
    <p class="sharing__muted">
      No tunnel providers found. Install
      <code>ngrok</code> or <code>cloudflared</code> to share your site publicly.
    </p>
  {:else if shareInfo}
    <div class="sharing__active">
      <div class="sharing__url-row">
        <span class="badge badge--success">{shareInfo.provider}</span>
        <code class="sharing__url" title={shareInfo.public_url}>{shareInfo.public_url}</code>
        <button class="btn-icon" onclick={copyUrl} aria-label="Copy URL">
          <Icon name={copyFeedback ? "check" : "copy"} size={14} />
        </button>
      </div>
      <button class="btn-danger btn-sm" onclick={stopSharing}>
        <Icon name="stop" size={12} />
        Stop Sharing
      </button>
    </div>
  {:else}
    <div class="sharing__providers">
      <span class="sharing__label">Available: {providers.join(", ")}</span>
      <button class="btn-primary btn-sm" onclick={startSharing} disabled={sharing}>
        {#if sharing}
          <span class="spinner spinner--sm"></span>
        {:else}
          <Icon name="external-link" size={12} />
        {/if}
        {sharing ? "Starting..." : "Share"}
      </button>
    </div>
  {/if}
</div>

<style>
  .sharing {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .sharing h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .sharing__muted {
    font-size: var(--text-sm);
    color: var(--color-text-muted);
  }

  .sharing__active {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .sharing__active button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    align-self: flex-start;
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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .sharing__providers {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .sharing__providers button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .sharing__label {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }
</style>
