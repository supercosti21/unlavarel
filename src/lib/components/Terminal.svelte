<script>
  let { lines = [], title = "Logs" } = $props();
  let container;

  $effect(() => {
    if (container && lines.length) {
      container.scrollTop = container.scrollHeight;
    }
  });
</script>

<div class="terminal">
  <div class="terminal__header">
    <h3 class="terminal__title">{title}</h3>
    <div class="terminal__dots">
      <span class="terminal__dot terminal__dot--red"></span>
      <span class="terminal__dot terminal__dot--yellow"></span>
      <span class="terminal__dot terminal__dot--green"></span>
    </div>
  </div>
  <pre class="terminal__output" bind:this={container}>{#if lines.length > 0}{#each lines as line}{line}
{/each}{:else}<span class="terminal__placeholder">Waiting for output...</span>{/if}</pre>
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
  }

  .terminal__title {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .terminal__dots {
    display: flex;
    gap: var(--space-1);
  }

  .terminal__dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .terminal__dot--red { background: var(--color-danger); }
  .terminal__dot--yellow { background: var(--color-warning); }
  .terminal__dot--green { background: var(--color-success); }

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
