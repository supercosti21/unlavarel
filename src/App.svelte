<script>
  import { invoke } from "@tauri-apps/api/core";
  import Sidebar from "./lib/components/Sidebar.svelte";
  import ServiceCard from "./lib/components/ServiceCard.svelte";
  import SiteList from "./lib/components/SiteList.svelte";
  import Terminal from "./lib/components/Terminal.svelte";
  import MailViewer from "./lib/components/MailViewer.svelte";
  import DbViewer from "./lib/components/DbViewer.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import SetupWizard from "./lib/components/SetupWizard.svelte";
  import { servicesStore } from "./lib/stores/services.svelte.js";
  import { projectsStore } from "./lib/stores/projects.svelte.js";

  let activePage = $state("dashboard");
  let logLines = $state([]);
  let showSetup = $state(false);
  let checkingSetup = $state(true);

  $effect(() => {
    checkFirstRun();
  });

  async function checkFirstRun() {
    checkingSetup = true;
    try {
      const state = await invoke("check_setup");
      showSetup = state.first_run;
      if (!state.first_run) {
        servicesStore.loadServices();
        projectsStore.loadProjects();
      }
    } catch {
      // If check fails, show dashboard anyway
      showSetup = false;
      servicesStore.loadServices();
      projectsStore.loadProjects();
    } finally {
      checkingSetup = false;
    }
  }

  function handleSetupComplete() {
    showSetup = false;
    servicesStore.loadServices();
    projectsStore.loadProjects();
  }

  function handleNavigate(page) {
    activePage = page;
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
              {#if servicesStore.error}
                <span class="badge badge--danger">{servicesStore.error}</span>
              {/if}
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

            <Terminal lines={logLines} />
          </div>

        {:else if activePage === "projects"}
          <div class="page">
            <SiteList
              projects={projectsStore.projects}
              onAdd={() => {}}
              onRemove={projectsStore.removeProject}
              onOpen={() => {}}
            />
          </div>

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
            <Terminal lines={logLines} title="All Services" />
          </div>

        {:else if activePage === "settings"}
          <div class="page">
            <header class="page__header">
              <h2>Settings</h2>
            </header>
            <p class="page__placeholder">Settings coming soon.</p>
          </div>
        {/if}
      </div>

      <StatusBar services={servicesStore.services} />
    </div>
  </div>
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

  .page__placeholder {
    color: var(--color-text-muted);
    padding: var(--space-8);
    text-align: center;
  }
</style>
