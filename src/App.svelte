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
  import SharingPanel from "./lib/components/SharingPanel.svelte";
  import SnapshotsPanel from "./lib/components/SnapshotsPanel.svelte";
  import { servicesStore } from "./lib/stores/services.svelte.js";
  import { projectsStore } from "./lib/stores/projects.svelte.js";

  let activePage = $state("dashboard");
  let logLines = $state([]);
  let showSetup = $state(false);
  let checkingSetup = $state(true);
  let showQuickApp = $state(false);
  let activeLogService = $state(null);
  let unlistenLog = $state(null);

  $effect(() => {
    checkFirstRun();
  });

  // Expose start/stop all for system tray
  if (typeof window !== "undefined") {
    window.__macenv_start_all = async () => {
      try {
        const result = await invoke("start_all_services");
        servicesStore.loadServices();
      } catch {}
    };
    window.__macenv_stop_all = async () => {
      try {
        const result = await invoke("stop_all_services");
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
        loadData();
        // Apply saved theme
        const settings = await invoke("get_settings");
        document.documentElement.setAttribute("data-theme", settings.theme);
      }
    } catch {
      showSetup = false;
      loadData();
    } finally {
      checkingSetup = false;
    }
  }

  function loadData() {
    servicesStore.loadServices();
    projectsStore.loadProjects();
  }

  function handleSetupComplete() {
    showSetup = false;
    loadData();
  }

  function handleNavigate(page) {
    activePage = page;
  }

  async function startAll() {
    try {
      await invoke("start_all_services");
      await servicesStore.loadServices();
    } catch {}
  }

  async function stopAll() {
    try {
      await invoke("stop_all_services");
      await servicesStore.loadServices();
    } catch {}
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
    invoke("open_in_browser", { url: `https://${domain}` }).catch(() => {
      invoke("open_in_browser", { url: `http://${domain}` }).catch(() => {});
    });
  }

  async function openInEditor(path) {
    invoke("open_in_editor", { path }).catch(() => {});
  }

  function handleAppCreated(name, path) {
    showQuickApp = false;
    projectsStore.loadProjects();
    activePage = "projects";
  }
</script>

{#if checkingSetup}
  <div class="app app--loading">
    <div class="app__splash">
      <h1>MacEnv</h1>
      <p>Loading...</p>
    </div>
  </div>

{:else if showSetup}
  <div class="app app--setup">
    <SetupWizard onComplete={handleSetupComplete} />
  </div>

{:else}
  <div class="app">
    <Sidebar {activePage} onNavigate={handleNavigate} />

    <div class="app__main">
      <div class="app__content">
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
                {#each servicesStore.services as svc}
                  <button
                    class="dashboard__log-tab"
                    class:dashboard__log-tab--active={activeLogService === svc.name}
                    onclick={() => watchLogs(svc.name)}
                  >{svc.name}</button>
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
            <SiteList
              projects={projectsStore.projects}
              onAdd={() => (showQuickApp = true)}
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

        {:else if activePage === "logs"}
          <div class="page">
            <header class="page__header">
              <h2>Service Logs</h2>
            </header>
            <div class="dashboard__log-tabs">
              {#each servicesStore.services as svc}
                <button
                  class="dashboard__log-tab"
                  class:dashboard__log-tab--active={activeLogService === svc.name}
                  onclick={() => watchLogs(svc.name)}
                >{svc.name}</button>
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

      <StatusBar services={servicesStore.services} />
    </div>
  </div>

  {#if showQuickApp}
    <QuickAppDialog
      onCreated={handleAppCreated}
      onClose={() => (showQuickApp = false)}
    />
  {/if}
{/if}

<style>
  .app {
    display: flex;
    height: 100%;
  }

  .app--loading,
  .app--setup {
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--color-bg-primary);
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
</style>
