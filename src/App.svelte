<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import ServiceCard from "./lib/components/ServiceCard.svelte";
  import SiteList from "./lib/components/SiteList.svelte";
  import Terminal from "./lib/components/Terminal.svelte";
  import MailViewer from "./lib/components/MailViewer.svelte";
  import DbViewer from "./lib/components/DbViewer.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import SetupWizard from "./lib/components/SetupWizard.svelte";
  import SettingsPage from "./lib/components/SettingsPage.svelte";
  import PhpManager from "./lib/components/PhpManager.svelte";
  import QuickAppDialog from "./lib/components/QuickAppDialog.svelte";
  import ImportProjectDialog from "./lib/components/ImportProjectDialog.svelte";
  import SharingPanel from "./lib/components/SharingPanel.svelte";
  import SnapshotsPanel from "./lib/components/SnapshotsPanel.svelte";
  import ConfigEditor from "./lib/components/ConfigEditor.svelte";
  import PasswordDialog from "./lib/components/PasswordDialog.svelte";
  import Toast from "./lib/components/Toast.svelte";
  import { servicesStore } from "./lib/stores/services.svelte.js";
  import { projectsStore } from "./lib/stores/projects.svelte.js";
  import { toastStore } from "./lib/stores/toast.svelte.js";

  let activePage = $state("dashboard");
  let logLines = $state([]);
  let showSetup = $state(false);
  let checkingSetup = $state(true);
  let showQuickApp = $state(false);
  let showImportProject = $state(false);
  let showPasswordDialog = $state(false);
  let showCommandPalette = $state(false);
  let pendingAction = $state(null);
  let activeLogService = $state(null);
  let unlistenLog = $state(null);
  let refreshInterval = $state(null);
  let foundProjects = $state([]);

  // Page map for keyboard shortcuts (Ctrl+1..6, Ctrl+7=settings)
  const pageShortcuts = {
    "1": "dashboard",
    "2": "projects",
    "3": "php",
    "4": "database",
    "5": "mail",
    "6": "config",
    "7": "logs",
    "8": "settings",
  };

  $effect(() => {
    checkFirstRun();

    // Global keyboard shortcuts
    function handleGlobalKey(e) {
      // Don't trigger when typing in inputs
      if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA" || e.target.tagName === "SELECT") return;

      // Ctrl+1..7 — page navigation
      if ((e.ctrlKey || e.metaKey) && pageShortcuts[e.key]) {
        e.preventDefault();
        activePage = pageShortcuts[e.key];
        return;
      }

      // Ctrl+R — refresh services
      if ((e.ctrlKey || e.metaKey) && e.key === "r") {
        e.preventDefault();
        servicesStore.loadServices();
        toastStore.info("Refreshing services...");
        return;
      }

      // Ctrl+N — new project
      if ((e.ctrlKey || e.metaKey) && e.key === "n") {
        e.preventDefault();
        showQuickApp = true;
        return;
      }

      // Ctrl+K / Cmd+K — command palette
      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        showCommandPalette = !showCommandPalette;
        return;
      }

      // Ctrl+T — open terminal
      if ((e.ctrlKey || e.metaKey) && e.key === "t") {
        e.preventDefault();
        invoke("open_terminal", {}).catch(() => {});
        return;
      }

      // Escape — close modals
      if (e.key === "Escape") {
        showCommandPalette = false;
      }
    }

    window.addEventListener("keydown", handleGlobalKey);
    return () => window.removeEventListener("keydown", handleGlobalKey);
  });

  // Expose start/stop all for system tray
  if (typeof window !== "undefined") {
    window.__unlavarel_start_all = async () => {
      try {
        await invoke("start_all_services");
        servicesStore.loadServices();
      } catch {}
    };
    window.__unlavarel_stop_all = async () => {
      try {
        await invoke("stop_all_services");
        servicesStore.loadServices();
      } catch {}
    };
  }

  async function checkFirstRun() {
    checkingSetup = true;
    try {
      const state = await invoke("check_setup");
      showSetup = state.first_run;
      if (!state.first_run) {
        await loadData();
        // Apply saved theme
        const settings = await invoke("get_settings");
        document.documentElement.setAttribute("data-theme", settings.theme);
        // Auto-start services if enabled and password is cached
        if (settings.auto_start_services) {
          try {
            const hasPwd = await invoke("has_session_password");
            if (hasPwd) {
              await invoke("start_all_services");
              await servicesStore.loadServices();
              toastStore.success("Services auto-started");
            }
          } catch {}
        }
      }
    } catch {
      showSetup = false;
      await loadData();
    } finally {
      checkingSetup = false;
    }
  }

  async function loadData() {
    servicesStore.loadServices();
    projectsStore.loadProjects();

    // Auto-refresh services every 5 seconds
    if (refreshInterval) clearInterval(refreshInterval);
    refreshInterval = setInterval(() => {
      servicesStore.loadServices();
    }, 5000);

    // Auto-check for updates (silent, no error toast)
    invoke("check_for_updates").then((info) => {
      if (info?.update_available) {
        toastStore.info(`Update v${info.latest_version} available — go to Settings`);
      }
    }).catch(() => {});

    // Auto-scan project root for unimported projects
    autoScanProjects();
  }

  async function autoScanProjects() {
    try {
      const settings = await invoke("get_settings");
      if (!settings.project_root) return;
      const scanned = await invoke("scan_projects", { directory: settings.project_root });
      foundProjects = scanned.filter((p) => !p.already_added);
      if (foundProjects.length > 0) {
        toastStore.info(`Found ${foundProjects.length} project${foundProjects.length > 1 ? 's' : ''} in ${settings.project_root}`);
      }
    } catch {}
  }

  function handleSetupComplete() {
    showSetup = false;
    loadData();
  }

  function handleNavigate(page) {
    activePage = page;
  }

  /// Run an action that may need elevation — prompts for password if needed
  async function withElevation(action) {
    try {
      const hasPwd = await invoke("has_session_password");
      if (!hasPwd) {
        // Show password dialog, then run action after auth
        pendingAction = action;
        showPasswordDialog = true;
        return;
      }
      await action();
    } catch (e) {
      // If it fails due to auth, show password dialog
      const msg = String(e);
      if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec")) {
        pendingAction = action;
        showPasswordDialog = true;
      }
    }
  }

  function onPasswordSuccess() {
    showPasswordDialog = false;
    if (pendingAction) {
      const action = pendingAction;
      pendingAction = null;
      action();
    }
  }

  function onPasswordCancel() {
    showPasswordDialog = false;
    pendingAction = null;
  }

  async function startAll() {
    await withElevation(async () => {
      await invoke("start_all_services");
      await servicesStore.loadServices();
    });
  }

  async function stopAll() {
    await withElevation(async () => {
      await invoke("stop_all_services");
      await servicesStore.loadServices();
    });
  }

  async function watchLogs(serviceName) {
    // Unsubscribe previous
    if (unlistenLog) {
      unlistenLog();
      unlistenLog = null;
    }
    if (activeLogService) {
      invoke("unwatch_service_logs", { service: activeLogService }).catch(() => {});
    }

    activeLogService = serviceName;
    logLines = [];

    // Listen for log events
    unlistenLog = await listen(`log:${serviceName}`, (event) => {
      logLines = [...logLines.slice(-500), event.payload];
    });

    // Start streaming
    invoke("watch_service_logs", { service: serviceName }).catch(() => {});
  }

  async function openInBrowser(domain) {
    // Find the project to check if SSL is enabled
    const proj = projectsStore.projects.find((p) => p.domain === domain);
    const protocol = proj?.ssl ? "https" : "http";
    invoke("open_in_browser", { url: `${protocol}://${domain}` }).catch(() => {});
  }

  async function openInEditor(path) {
    invoke("open_in_editor", { path }).catch(() => {});
  }

  function handleAppCreated(name, path) {
    showQuickApp = false;
    projectsStore.loadProjects();
    activePage = "projects";
  }

  // --- Command Palette ---
  let cmdQuery = $state("");
  const commands = [
    { label: "Go to Dashboard", shortcut: "Ctrl+1", action: () => (activePage = "dashboard") },
    { label: "Go to Projects", shortcut: "Ctrl+2", action: () => (activePage = "projects") },
    { label: "Go to PHP Manager", shortcut: "Ctrl+3", action: () => (activePage = "php") },
    { label: "Go to Database", shortcut: "Ctrl+4", action: () => (activePage = "database") },
    { label: "Go to Mail", shortcut: "Ctrl+5", action: () => (activePage = "mail") },
    { label: "Go to Config", shortcut: "Ctrl+6", action: () => (activePage = "config") },
    { label: "Go to Settings", shortcut: "Ctrl+8", action: () => (activePage = "settings") },
    { label: "New Project", shortcut: "Ctrl+N", action: () => (showQuickApp = true) },
    { label: "Import Project", action: () => (showImportProject = true) },
    { label: "Open Terminal", shortcut: "Ctrl+T", action: () => invoke("open_terminal", {}) },
    { label: "Start All Services", action: () => startAll() },
    { label: "Stop All Services", action: () => stopAll() },
    { label: "Refresh Services", shortcut: "Ctrl+R", action: () => servicesStore.loadServices() },
  ];
  let filteredCommands = $derived(
    cmdQuery
      ? commands.filter((c) => c.label.toLowerCase().includes(cmdQuery.toLowerCase()))
      : commands
  );

  function handleProjectImported(name, path) {
    // Don't close — user may want to import more (scan mode)
    projectsStore.loadProjects();
    activePage = "projects";
  }
</script>

{#if checkingSetup}
  <div class="app app--vertical">
    <div class="app--loading">
      <div class="app__splash">
        <h1>Unlavarel</h1>
        <p>Loading...</p>
      </div>
    </div>
  </div>

{:else if showSetup}
  <div class="app app--vertical">
    <div class="app--setup">
      <SetupWizard onComplete={handleSetupComplete} />
    </div>
  </div>

{:else}
  <div class="app app--vertical">
    <div class="app__body">
      <Sidebar {activePage} onNavigate={handleNavigate} />

    <div class="app__main">
      <div class="app__content">
        {#key activePage}
        <div class="page-transition">
        {#if activePage === "dashboard"}
          <div class="dashboard">
            <header class="dashboard__header">
              <h2 class="dashboard__title">Dashboard</h2>
              <div class="dashboard__actions">
                {#if servicesStore.error}
                  <span class="badge badge--danger">{servicesStore.error}</span>
                {/if}
                <button class="btn-primary" onclick={startAll}>Start All</button>
                <button class="btn-ghost" onclick={stopAll}>Stop All</button>
                <button class="btn-ghost" onclick={() => invoke("open_terminal", {})}>Terminal</button>
              </div>
            </header>

            {#if servicesStore.loading}
              <p class="dashboard__loading">Loading services...</p>
            {:else}
              <div class="dashboard__grid">
                {#each servicesStore.services as service (service.name)}
                  <ServiceCard
                    {service}
                    onToggle={servicesStore.toggleService}
                    onRestart={servicesStore.restartService}
                  />
                {/each}
              </div>
            {/if}

            <div class="dashboard__logs">
              <div class="dashboard__log-tabs">
                {#each servicesStore.services.filter(s => s.has_service !== false) as svc}
                  <button
                    class="dashboard__log-tab"
                    class:dashboard__log-tab--active={activeLogService === (svc.id || svc.name)}
                    onclick={() => watchLogs(svc.id || svc.name)}
                  >{svc.id || svc.name}</button>
                {/each}
              </div>
              <Terminal
                lines={logLines}
                title={activeLogService ? `Logs: ${activeLogService}` : "Logs"}
              />
            </div>
          </div>

        {:else if activePage === "projects"}
          <div class="page">
            {#if foundProjects.length > 0}
              <div class="found-projects-banner">
                <span>{foundProjects.length} project{foundProjects.length > 1 ? 's' : ''} found in your project root</span>
                <button class="btn-primary btn-sm" onclick={() => (showImportProject = true)}>
                  Import
                </button>
                <button class="btn-ghost btn-sm" onclick={() => (foundProjects = [])}>
                  Dismiss
                </button>
              </div>
            {/if}
            <SiteList
              projects={projectsStore.projects}
              onAdd={() => (showQuickApp = true)}
              onImport={() => (showImportProject = true)}
              onRemove={projectsStore.removeProject}
              onOpen={(name) => {
                const proj = projectsStore.projects.find((p) => p.name === name);
                if (proj) openInBrowser(proj.domain);
              }}
            />
            {#if projectsStore.projects.length > 0}
              <div class="page__panels">
                {#each projectsStore.projects as proj}
                  <div class="card">
                    <div class="page__proj-header">
                      <strong>{proj.name}</strong>
                      <div class="page__proj-actions">
                        <button class="btn-ghost" onclick={() => openInBrowser(proj.domain)}>Browser</button>
                        <button class="btn-ghost" onclick={() => openInEditor(proj.path)}>Editor</button>
                      </div>
                    </div>
                    <SharingPanel domain={proj.domain} />
                    <SnapshotsPanel
                      projectName={proj.name}
                      projectPath={proj.path}
                      database={proj.database}
                    />
                  </div>
                {/each}
              </div>
            {/if}
          </div>

        {:else if activePage === "php"}
          <PhpManager />

        {:else if activePage === "database"}
          <div class="page page--full">
            <DbViewer />
          </div>

        {:else if activePage === "mail"}
          <div class="page page--full">
            <MailViewer />
          </div>

        {:else if activePage === "config"}
          <div class="page page--full">
            <ConfigEditor />
          </div>

        {:else if activePage === "logs"}
          <div class="page">
            <header class="page__header">
              <h2>Service Logs</h2>
            </header>
            <div class="dashboard__log-tabs">
              {#each servicesStore.services.filter(s => s.has_service !== false) as svc}
                <button
                  class="dashboard__log-tab"
                  class:dashboard__log-tab--active={activeLogService === (svc.id || svc.name)}
                  onclick={() => watchLogs(svc.id || svc.name)}
                >{svc.id || svc.name}</button>
              {/each}
            </div>
            <Terminal
              lines={logLines}
              title={activeLogService ? `Logs: ${activeLogService}` : "Select a service"}
            />
          </div>

        {:else if activePage === "settings"}
          <SettingsPage />
        {/if}
        </div>
        {/key}
      </div>

      <StatusBar services={servicesStore.services} />
    </div>
    </div>
  </div>

  {#if showQuickApp}
    <QuickAppDialog
      onCreated={handleAppCreated}
      onClose={() => (showQuickApp = false)}
    />
  {/if}

  {#if showImportProject}
    <ImportProjectDialog
      onImported={handleProjectImported}
      onClose={() => (showImportProject = false)}
    />
  {/if}

  {#if showPasswordDialog}
    <PasswordDialog
      onSuccess={onPasswordSuccess}
      onCancel={onPasswordCancel}
    />
  {/if}

  {#if showCommandPalette}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="cmd-overlay" onclick={() => (showCommandPalette = false)}>
      <div class="cmd-palette" onclick={(e) => e.stopPropagation()}>
        <input
          class="cmd-input"
          type="text"
          placeholder="Type a command..."
          autofocus
          oninput={(e) => (cmdQuery = e.target.value)}
        />
        <div class="cmd-list">
          {#each filteredCommands as cmd}
            <button class="cmd-item" onclick={() => { showCommandPalette = false; cmd.action(); }}>
              <span class="cmd-label">{cmd.label}</span>
              {#if cmd.shortcut}
                <span class="cmd-shortcut">{cmd.shortcut}</span>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
{/if}

<Toast />

<style>
  .app {
    height: 100%;
  }

  .app--vertical {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .app__body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .app--loading,
  .app--setup {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-primary);
    overflow-y: auto;
  }

  .app__splash {
    text-align: center;
    color: var(--color-text-muted);
  }

  .app__splash h1 {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-2);
  }

  .app__main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .app__content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-6);
  }

  .page-transition {
    animation: slide-up 200ms ease forwards;
  }

  /* Dashboard */
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .dashboard__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .dashboard__title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .dashboard__actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .dashboard__loading {
    color: var(--color-text-muted);
    padding: var(--space-8);
    text-align: center;
  }

  .dashboard__grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--space-4);
  }

  .dashboard__logs {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .dashboard__log-tabs {
    display: flex;
    gap: var(--space-1);
    flex-wrap: wrap;
  }

  .dashboard__log-tab {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    border: none;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .dashboard__log-tab:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dashboard__log-tab--active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  /* Pages */
  .page {
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  .page--full {
    height: 100%;
  }

  .page__header h2 {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .page__panels {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .page__proj-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-4);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .page__proj-actions {
    display: flex;
    gap: var(--space-2);
  }

  /* Found projects banner */
  .found-projects-banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--color-accent-subtle);
    border: 1px solid var(--color-accent);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
  }

  .found-projects-banner span {
    flex: 1;
  }

  /* Command Palette */
  .cmd-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 200;
    display: flex;
    justify-content: center;
    padding-top: 15vh;
  }

  .cmd-palette {
    width: 480px;
    max-height: 400px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-elevated);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    align-self: flex-start;
  }

  .cmd-input {
    padding: var(--space-4);
    border: none;
    border-bottom: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-primary);
    font-size: var(--text-base);
    outline: none;
  }

  .cmd-list {
    overflow-y: auto;
    padding: var(--space-2);
  }

  .cmd-item {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-primary);
    font-size: var(--text-sm);
    text-align: left;
    border: none;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .cmd-item:hover {
    background: var(--color-bg-hover);
  }

  .cmd-label {
    font-weight: var(--font-medium);
  }

  .cmd-shortcut {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-family: var(--font-mono);
    background: var(--color-bg-tertiary);
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }
</style>
