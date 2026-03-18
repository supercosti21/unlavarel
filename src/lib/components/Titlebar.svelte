<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function close() {
    await appWindow.close();
  }
</script>

<header class="titlebar">
  <div class="titlebar__drag" data-tauri-drag-region>
    <span class="titlebar__title" data-tauri-drag-region>MacEnv</span>
  </div>
  <div class="titlebar__controls">
    <button class="titlebar__btn" onclick={minimize} aria-label="Minimize">
      <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
    </button>
    <button class="titlebar__btn" onclick={toggleMaximize} aria-label="Maximize">
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2"><rect x="0.6" y="0.6" width="8.8" height="8.8" rx="1"/></svg>
    </button>
    <button class="titlebar__btn titlebar__btn--close" onclick={close} aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 10 10" stroke="currentColor" stroke-width="1.4"><line x1="1" y1="1" x2="9" y2="9"/><line x1="9" y1="1" x2="1" y2="9"/></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    height: 38px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .titlebar__drag {
    flex: 1;
    height: 100%;
    display: flex;
    align-items: center;
    padding-left: var(--space-4);
  }

  .titlebar__title {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    letter-spacing: 0.02em;
    pointer-events: none;
  }

  .titlebar__controls {
    display: flex;
    height: 100%;
  }

  .titlebar__btn {
    width: 46px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 0;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .titlebar__btn:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .titlebar__btn--close:hover {
    background: var(--color-danger);
    color: var(--color-text-on-accent);
  }
</style>
