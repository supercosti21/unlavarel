<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onSuccess, onCancel } = $props();

  let password = $state("");
  let saving = $state(false);
  let error = $state(null);

  async function submit() {
    if (!password) return;
    saving = true;
    error = null;
    try {
      await invoke("save_session_password", { password });
      onSuccess();
    } catch (e) {
      error = "Incorrect password. Please try again.";
      password = "";
    } finally {
      saving = false;
    }
  }

  function handleKeydown(e) {
    if (e.key === "Enter") submit();
    if (e.key === "Escape") onCancel();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pwd-overlay" onclick={onCancel}>
  <div class="pwd-dialog" onclick={(e) => e.stopPropagation()}>
    <div class="pwd-icon">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
        <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
      </svg>
    </div>
    <h3>Authentication Required</h3>
    <p>MacEnv needs your password to manage system services. It will be remembered for this session only.</p>

    <input
      type="password"
      bind:value={password}
      placeholder="Password"
      onkeydown={handleKeydown}
      autofocus
    />

    {#if error}
      <span class="pwd-error">{error}</span>
    {/if}

    <div class="pwd-actions">
      <button class="pwd-btn pwd-btn--ghost" onclick={onCancel}>Cancel</button>
      <button class="pwd-btn pwd-btn--primary" onclick={submit} disabled={saving || !password}>
        {saving ? "Verifying..." : "Authenticate"}
      </button>
    </div>
  </div>
</div>

<style>
  .pwd-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .pwd-dialog {
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    width: 360px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    box-shadow: var(--shadow-elevated);
    text-align: center;
  }

  .pwd-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-accent-subtle);
    color: var(--color-accent);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .pwd-dialog h3 {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
  }

  .pwd-dialog p {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .pwd-dialog input {
    width: 100%;
    text-align: center;
    font-size: var(--text-sm);
    letter-spacing: 0.1em;
  }

  .pwd-error {
    font-size: var(--text-xs);
    color: var(--color-danger);
  }

  .pwd-actions {
    display: flex;
    gap: var(--space-2);
    width: 100%;
  }

  .pwd-btn {
    flex: 1;
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    border: none;
    transition: all var(--transition-fast);
  }

  .pwd-btn--primary {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  .pwd-btn--primary:hover { background: var(--color-accent-hover); }
  .pwd-btn--primary:disabled { opacity: 0.5; cursor: not-allowed; }

  .pwd-btn--ghost {
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
  }

  .pwd-btn--ghost:hover { background: var(--color-bg-hover); }
</style>
