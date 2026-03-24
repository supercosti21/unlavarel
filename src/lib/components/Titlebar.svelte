<script>
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const appWindow = getCurrentWindow();
  let hovering = $state(false);

  async function close() {
    await appWindow.close();
  }

  async function minimize() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }
</script>

<!-- Invisible drag region across the top of the window -->
<div class="titlebar-drag" data-tauri-drag-region></div>

<!-- macOS-style traffic lights — works on all platforms (decorations: false) -->
<div
  class="traffic-lights"
  role="group"
  aria-label="Window controls"
  onmouseenter={() => (hovering = true)}
  onmouseleave={() => (hovering = false)}
>
  <button class="tl tl--close" onclick={close} aria-label="Close">
    {#if hovering}
      <svg width="8" height="8" viewBox="0 0 8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <line x1="1.5" y1="1.5" x2="6.5" y2="6.5"/><line x1="6.5" y1="1.5" x2="1.5" y2="6.5"/>
      </svg>
    {/if}
  </button>
  <button class="tl tl--minimize" onclick={minimize} aria-label="Minimize">
    {#if hovering}
      <svg width="8" height="8" viewBox="0 0 8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <line x1="1" y1="4" x2="7" y2="4"/>
      </svg>
    {/if}
  </button>
  <button class="tl tl--maximize" onclick={toggleMaximize} aria-label="Maximize">
    {#if hovering}
      <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor">
        <path d="M1 3.5L4 0.5L7 3.5H5V7.5H3V3.5H1Z"/>
      </svg>
    {/if}
  </button>
</div>

<style>
  /* Invisible drag region — full width strip at the top */
  .titlebar-drag {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 38px;
    z-index: 0;
    -webkit-app-region: drag;
  }

  .traffic-lights {
    position: fixed;
    top: 12px;
    left: 14px;
    display: flex;
    gap: 8px;
    z-index: 10;
    -webkit-app-region: no-drag;
  }

  .tl {
    width: 13px;
    height: 13px;
    border-radius: 50%;
    border: none;
    padding: 0;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: filter 120ms ease;
    color: rgba(0, 0, 0, 0.55);
    position: relative;
  }

  .tl:active {
    filter: brightness(0.8);
  }

  .tl--close {
    background: #ff5f57;
  }

  .tl--minimize {
    background: #febc2e;
  }

  .tl--maximize {
    background: #28c840;
  }

  .tl svg {
    width: 8px;
    height: 8px;
  }
</style>
