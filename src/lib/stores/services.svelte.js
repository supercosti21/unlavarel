import { invoke } from "@tauri-apps/api/core";

function createServicesStore() {
  let services = $state([]);
  let loading = $state(false);
  let error = $state(null);
  let needsAuth = $state(false);

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
    error = null;
    try {
      const updated = await invoke(command, { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
    } catch (e) {
      const msg = String(e);
      if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec") || msg.includes("dismissed")) {
        needsAuth = true;
      }
      error = msg;
      await loadServices();
    }
  }

  async function restartService(id) {
    error = null;
    try {
      const updated = await invoke("restart_service", { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
    } catch (e) {
      const msg = String(e);
      if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec") || msg.includes("dismissed")) {
        needsAuth = true;
      }
      error = msg;
      await loadServices();
    }
  }

  function clearError() {
    error = null;
    needsAuth = false;
  }

  return {
    get services() { return services; },
    get loading() { return loading; },
    get error() { return error; },
    get needsAuth() { return needsAuth; },
    loadServices,
    refreshDiscovery,
    toggleService,
    restartService,
    clearError,
  };
}

export const servicesStore = createServicesStore();
