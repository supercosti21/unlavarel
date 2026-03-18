import { invoke } from "@tauri-apps/api/core";

function createServicesStore() {
  let services = $state([]);
  let loading = $state(false);
  let error = $state(null);

  async function loadServices() {
    loading = true;
    error = null;
    try {
      services = await invoke("get_services");
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function refreshDiscovery() {
    try {
      await invoke("discover_services");
      await loadServices();
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleService(id, currentlyRunning) {
    const command = currentlyRunning ? "stop_service" : "start_service";
    try {
      const updated = await invoke(command, { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
    } catch (e) {
      error = String(e);
      // Reload to get real state
      await loadServices();
    }
  }

  async function restartService(id) {
    try {
      const updated = await invoke("restart_service", { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
    } catch (e) {
      error = String(e);
      await loadServices();
    }
  }

  return {
    get services() {
      return services;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    loadServices,
    refreshDiscovery,
    toggleService,
    restartService,
  };
}

export const servicesStore = createServicesStore();
