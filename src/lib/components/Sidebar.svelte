<script>
  import Icon from "./Icon.svelte";

  let { activePage = "dashboard", onNavigate } = $props();

  const navItems = [
    { id: "dashboard", label: "Dashboard", icon: "grid" },
    { id: "projects", label: "Projects", icon: "folder" },
    { id: "php", label: "PHP", icon: "code" },
    { id: "database", label: "Database", icon: "database" },
    { id: "mail", label: "Mail", icon: "mail" },
    { id: "config", label: "Config", icon: "settings" },
    { id: "logs", label: "Logs", icon: "terminal" },
  ];

  const allItems = [...navItems, { id: "settings", label: "Settings", icon: "settings" }];

  function handleKeydown(e) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const idx = allItems.findIndex(i => i.id === activePage);
    const next = e.key === "ArrowDown"
      ? (idx + 1) % allItems.length
      : (idx - 1 + allItems.length) % allItems.length;
    onNavigate(allItems[next].id);
  }
</script>

<aside class="sidebar">
  <div class="sidebar__header">
    <h1 class="sidebar__logo">Unlavarel</h1>
    <span class="sidebar__version">v0.1.0</span>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <nav class="sidebar__nav" aria-label="Main navigation" onkeydown={handleKeydown}>
    {#each navItems as item}
      <button
        class="sidebar__item"
        class:sidebar__item--active={activePage === item.id}
        aria-current={activePage === item.id ? "page" : undefined}
        onclick={() => onNavigate(item.id)}
      >
        <Icon name={item.icon} size={18} />
        <span class="sidebar__label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="sidebar__footer">
    <button
      class="sidebar__item"
      class:sidebar__item--active={activePage === "settings"}
      aria-current={activePage === "settings" ? "page" : undefined}
      onclick={() => onNavigate("settings")}
      onkeydown={handleKeydown}
    >
      <Icon name="settings" size={18} />
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
    padding-top: 0;
  }

  .sidebar__header {
    padding: 0 var(--space-4) var(--space-4);
    padding-top: var(--space-4);
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
    padding-left: var(--space-4);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    transition: background var(--transition-fast), color var(--transition-fast);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    position: relative;
  }

  .sidebar__item:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .sidebar__item--active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .sidebar__item--active::before {
    content: '';
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 20px;
    border-radius: var(--radius-full);
    background: var(--color-accent);
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
