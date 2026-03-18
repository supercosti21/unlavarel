import { invoke } from "@tauri-apps/api/core";

function createProjectsStore() {
  let projects = $state([]);
  let loading = $state(false);
  let error = $state(null);

  async function loadProjects() {
    loading = true;
    error = null;
    try {
      projects = await invoke("get_projects");
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function addProject(name, path) {
    try {
      const project = await invoke("add_project", { name, path });
      projects = [...projects, project];
    } catch (e) {
      error = String(e);
    }
  }

  async function removeProject(name) {
    try {
      await invoke("remove_project", { name });
      projects = projects.filter((p) => p.name !== name);
    } catch (e) {
      error = String(e);
    }
  }

  return {
    get projects() {
      return projects;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    loadProjects,
    addProject,
    removeProject,
  };
}

export const projectsStore = createProjectsStore();
