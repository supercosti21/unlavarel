import { invoke } from "@tauri-apps/api/core";
import { toastStore } from "./toast.svelte.js";

function friendlyServiceError(raw, serviceName = "") {
  const msg = String(raw).toLowerCase();
  const name = serviceName || "service";

  if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec") || msg.includes("dismissed")) {
    return `Admin password required to control ${name}. Please authenticate.`;
  }
  if (msg.includes("service failed") || msg.includes("job for")) {
    return `${name} failed to start. Check its configuration or run Health Check in Settings.`;
  }
  if (msg.includes("not found") || msg.includes("no such")) {
    return `${name} is not installed. Install it from Setup Wizard or Settings.`;
  }
  if (msg.includes("already running")) {
    return `${name} is already running.`;
  }
  if (msg.includes("not running") || msg.includes("not loaded")) {
    return `${name} is not running.`;
  }
  if (msg.includes("timeout")) {
    return `${name} took too long to respond. Try again.`;
  }
  if (msg.includes("permission") || msg.includes("denied")) {
    return `Permission denied. Authenticate first to manage ${name}.`;
  }
  return String(raw);
}

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
      error = friendlyServiceError(e);
    } finally {
      loading = false;
    }
  }

  async function refreshDiscovery() {
    try {
      await invoke("discover_services");
      await loadServices();
    } catch (e) {
      error = friendlyServiceError(e);
    }
  }

  async function toggleService(id, currentlyRunning) {
    const command = currentlyRunning ? "stop_service" : "start_service";
    const action = currentlyRunning ? "stop" : "start";
    error = null;
    try {
      const updated = await invoke(command, { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
      toastStore.success(`${id} ${currentlyRunning ? "stopped" : "started"}`);
    } catch (e) {
      const msg = String(e);
      if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec") || msg.includes("dismissed")) {
        needsAuth = true;
      }
      const friendly = friendlyServiceError(e, id);
      error = friendly;
      toastStore.error(friendly);
      await loadServices();
    }
  }

  async function restartService(id) {
    error = null;
    try {
      const updated = await invoke("restart_service", { name: id });
      services = services.map((s) => (s.id === id ? updated : s));
      toastStore.success(`${id} restarted`);
    } catch (e) {
      const msg = String(e);
      if (msg.includes("password") || msg.includes("auth") || msg.includes("pkexec") || msg.includes("dismissed")) {
        needsAuth = true;
      }
      const friendly = friendlyServiceError(e, id);
      error = friendly;
      toastStore.error(friendly);
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
