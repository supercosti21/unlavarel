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

  async function toggleService(name, currentlyRunning) {
    const command = currentlyRunning ? "stop_service" : "start_service";
    try {
      const updated = await invoke(command, { name });
      services = services.map((s) => (s.name === name ? updated : s));
    } catch (e) {
      error = String(e);
    }
  }

  async function restartService(name) {
    try {
      await invoke("stop_service", { name });
      const updated = await invoke("start_service", { name });
      services = services.map((s) => (s.name === name ? updated : s));
    } catch (e) {
      error = String(e);
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
    toggleService,
    restartService,
  };
}

export const servicesStore = createServicesStore();
