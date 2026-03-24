<script>
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let { lines = [], title = "Logs" } = $props();
  let container;
  let scrollLocked = $state(false);
  let showSearch = $state(false);
  let searchQuery = $state("");
  let copied = $state(false);

  let filteredLines = $derived(
    searchQuery
      ? lines.filter(l => l.toLowerCase().includes(searchQuery.toLowerCase()))
      : lines
  );

  $effect(() => {
    if (container && filteredLines.length && !scrollLocked) {
      container.scrollTop = container.scrollHeight;
    }
  });

  function copyAll() {
    const text = filteredLines.join("\n");
    navigator.clipboard.writeText(text).then(() => {
      copied = true;
      toastStore.success("Copied to clipboard");
      setTimeout(() => (copied = false), 2000);
    });
  }

  function toggleSearch() {
    showSearch = !showSearch;
    if (!showSearch) searchQuery = "";
  }

  function clearTerminal() {
    // This only clears the view; parent still holds lines
    if (container) container.textContent = "";
  }
</script>

<div class="terminal">
  <div class="terminal__header">
    <h3 class="terminal__title">{title}</h3>
    <div class="terminal__toolbar">
      {#if showSearch}
        <input
          class="terminal__search"
          type="text"
          placeholder="Filter..."
          bind:value={searchQuery}
        />
      {/if}
      <button class="btn-icon" onclick={toggleSearch} aria-label="Search logs" class:terminal__btn--active={showSearch}>
        <Icon name="search" size={14} />
      </button>
      <button class="btn-icon" onclick={() => (scrollLocked = !scrollLocked)} aria-label="Toggle scroll lock" class:terminal__btn--active={scrollLocked}>
        <Icon name={scrollLocked ? "lock" : "chevron-down"} size={14} />
      </button>
      <button class="btn-icon" onclick={copyAll} aria-label="Copy all">
        <Icon name={copied ? "check" : "copy"} size={14} />
      </button>
    </div>
  </div>
  <pre class="terminal__output" bind:this={container}>{#if filteredLines.length > 0}{#each filteredLines as line}{line}
{/each}{:else}<span class="terminal__placeholder">{searchQuery ? "No matches found" : "Waiting for output..."}</span>{/if}</pre>
</div>

<style>
  .terminal {
    display: flex;
    flex-direction: column;
    background: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .terminal__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-4);
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border);
    gap: var(--space-2);
  }

  .terminal__title {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .terminal__toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .terminal__btn--active {
    color: var(--color-accent);
  }

  .terminal__search {
    width: 160px;
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-2);
    background: var(--color-bg-input);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    color: var(--color-text-primary);
    font-family: var(--font-mono);
    outline: none;
    animation: fade-in 150ms ease;
  }

  .terminal__search:focus {
    border-color: var(--color-border-focus);
  }

  .terminal__output {
    padding: var(--space-4);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.6;
    color: var(--color-text-secondary);
    overflow-y: auto;
    max-height: 300px;
    min-height: 120px;
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
  }

  .terminal__placeholder {
    color: var(--color-text-muted);
    font-style: italic;
  }
</style>
