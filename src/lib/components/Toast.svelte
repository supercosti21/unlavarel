<script>
  import { toastStore } from '../stores/toast.svelte.js';

  const iconPaths = {
    success: 'M20 6L9 17l-5-5',
    error: 'M18 6L6 18M6 6l12 12',
    info: 'M12 16v-4m0-4h.01M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20z',
    warning: 'M12 9v4m0 4h.01M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z',
  };
</script>

{#if toastStore.toasts.length > 0}
  <div class="toast-container" role="status" aria-live="polite">
    {#each toastStore.toasts as toast (toast.id)}
      <div class="toast toast--{toast.type}">
        <svg class="toast__icon" width="16" height="16" viewBox="0 0 24 24" fill="none"
             stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d={iconPaths[toast.type] || iconPaths.info} />
        </svg>
        <span class="toast__message">{toast.message}</span>
        <button class="toast__close" onclick={() => toastStore.dismiss(toast.id)} aria-label="Dismiss">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
               stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: var(--space-12);
    right: var(--space-4);
    z-index: var(--z-toast);
    display: flex;
    flex-direction: column-reverse;
    gap: var(--space-2);
    pointer-events: none;
    max-width: 380px;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-left: 3px solid var(--color-text-muted);
    box-shadow: var(--shadow-elevated);
    font-size: var(--text-sm);
    color: var(--color-text-primary);
    animation: slide-up 200ms ease forwards;
  }

  .toast--success {
    border-left-color: var(--color-success);
    background: var(--color-success-subtle);
  }

  .toast--success .toast__icon {
    color: var(--color-success);
  }

  .toast--error {
    border-left-color: var(--color-danger);
    background: var(--color-danger-subtle);
  }

  .toast--error .toast__icon {
    color: var(--color-danger);
  }

  .toast--info {
    border-left-color: var(--color-info);
    background: var(--color-info-subtle);
  }

  .toast--info .toast__icon {
    color: var(--color-info);
  }

  .toast--warning {
    border-left-color: var(--color-warning);
    background: var(--color-warning-subtle);
  }

  .toast--warning .toast__icon {
    color: var(--color-warning);
  }

  .toast__icon {
    flex-shrink: 0;
  }

  .toast__message {
    flex: 1;
    line-height: var(--leading-tight);
  }

  .toast__close {
    flex-shrink: 0;
    padding: 2px;
    background: transparent;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    transition: color var(--transition-fast);
  }

  .toast__close:hover {
    color: var(--color-text-primary);
  }
</style>
