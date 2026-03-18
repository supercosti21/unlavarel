<script>
  let { activePage = "dashboard", onNavigate } = $props();

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: "grid" },
    { id: "projects", label: "Projects", icon: "folder" },
    { id: "php", label: "PHP", icon: "code" },
    { id: "database", label: "Database", icon: "database" },
    { id: "mail", label: "Mail", icon: "mail" },
    { id: "logs", label: "Logs", icon: "terminal" },
  ];
</script>

<aside class="sidebar">
  <div class="sidebar__header">
    <h1 class="sidebar__logo">MacEnv</h1>
    <span class="sidebar__version">v0.1.0</span>
  </div>

  <nav class="sidebar__nav">
    {#each navItems as item}
      <button
        class="sidebar__item"
        class:sidebar__item--active={activePage === item.id}
        onclick={() => onNavigate(item.id)}
      >
        <span class="sidebar__label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="sidebar__footer">
    <button class="sidebar__item" onclick={() => onNavigate("settings")}>
      <span class="sidebar__label">Settings</span>
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    height: 100%;
    background: var(--color-bg-secondary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    padding: var(--space-4) 0;
  }

  .sidebar__header {
    padding: 0 var(--space-4) var(--space-4);
    border-bottom: 1px solid var(--color-border-subtle);
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
  }

  .sidebar__logo {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .sidebar__version {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
  }

  .sidebar__nav {
    flex: 1;
    padding: var(--space-3) var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .sidebar__item {
    width: 100%;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    transition: background var(--transition-fast), color var(--transition-fast);
  }

  .sidebar__item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .sidebar__item--active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .sidebar__item--active:hover {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .sidebar__footer {
    padding: var(--space-2);
    border-top: 1px solid var(--color-border-subtle);
  }
</style>
