<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";

  let { onSuccess, onCancel } = $props();

  let password = $state("");
  let saving = $state(false);
  let error = $state(null);
  let inputEl;

  $effect(() => {
    if (inputEl) inputEl.focus();
  });

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

  function handleOverlayKeydown(e) {
    // Focus trap: keep Tab within dialog
    if (e.key === "Tab") {
      const dialog = e.currentTarget.querySelector('.pwd-dialog');
      const focusable = dialog?.querySelectorAll('input, button:not(:disabled)');
      if (!focusable || focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="pwd-overlay" onclick={onCancel} onkeydown={handleOverlayKeydown}>
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="pwd-dialog" role="dialog" aria-modal="true" aria-labelledby="pwd-title" onclick={(e) => e.stopPropagation()}>
    <div class="pwd-icon">
      <Icon name="lock" size={24} />
    </div>
    <h3 id="pwd-title">Authentication Required</h3>
    <p>Unlavarel needs your system password to start and stop services. Your password is only stored in memory for this session.</p>

    <input
      type="password"
      bind:value={password}
      bind:this={inputEl}
      placeholder="System password"
      onkeydown={handleKeydown}
    />

    {#if error}
      <div class="pwd-error">
        <Icon name="alert-circle" size={12} />
        <span>{error}</span>
      </div>
    {/if}

    <div class="pwd-actions">
      <button class="pwd-btn pwd-btn--ghost" onclick={onCancel}>Cancel</button>
      <button class="pwd-btn pwd-btn--primary" onclick={submit} disabled={saving || !password}>
        {#if saving}
          <span class="spinner spinner--sm"></span>
          Verifying...
        {:else}
          Authenticate
        {/if}
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
    z-index: var(--z-password);
    animation: fade-in 150ms ease;
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
    animation: slide-up 200ms ease;
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
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--color-danger);
    animation: fade-in 150ms ease;
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
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
