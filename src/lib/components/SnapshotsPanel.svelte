<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let { projectName = null, projectPath = "", database = null } = $props();

  let snapshots = $state([]);
  let loading = $state(true);
  let creating = $state(false);
  let confirmDelete = $state(null);
  let confirmRestore = $state(null);

  function friendlySnapError(raw) {
    const msg = String(raw).toLowerCase();
    if (msg.includes("no space") || msg.includes("disk")) return "Not enough disk space to create snapshot.";
    if (msg.includes("permission") || msg.includes("denied")) return "Permission denied. Check folder permissions.";
    if (msg.includes("not found")) return "Snapshot or target directory not found.";
    if (msg.includes("database")) return "Database backup failed. Is the DB service running?";
    return String(raw);
  }

  $effect(() => {
    loadSnapshots();
  });

  async function loadSnapshots() {
    loading = true;
    try {
      snapshots = await invoke("list_snapshots", { projectName });
    } catch {
      snapshots = [];
    } finally {
      loading = false;
    }
  }

  async function createSnapshot() {
    if (!projectName || !projectPath) return;
    creating = true;
    try {
      await invoke("create_snapshot", { projectName, projectPath, database });
      toastStore.success("Snapshot created");
      await loadSnapshots();
    } catch (e) {
      toastStore.error(friendlySnapError(e));
    } finally {
      creating = false;
    }
  }

  async function restoreSnap(id) {
    try {
      const result = await invoke("restore_snapshot", {
        snapshotId: id,
        targetPath: projectPath,
        restoreDb: !!database,
      });
      toastStore.success(result);
      confirmRestore = null;
    } catch (e) {
      toastStore.error(friendlySnapError(e));
      confirmRestore = null;
    }
  }

  async function deleteSnap(id) {
    try {
      await invoke("delete_snapshot", { snapshotId: id });
      toastStore.success("Snapshot deleted");
      confirmDelete = null;
      await loadSnapshots();
    } catch (e) {
      toastStore.error(friendlySnapError(e));
      confirmDelete = null;
    }
  }

  function formatSize(bytes) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1048576).toFixed(1)} MB`;
  }
</script>

<div class="snaps">
  <div class="snaps__header">
    <h3>
      <Icon name="download" size={14} />
      Snapshots
    </h3>
    {#if projectName}
      <button class="btn-primary btn-sm" onclick={createSnapshot} disabled={creating}>
        {#if creating}
          <span class="spinner spinner--sm"></span>
        {:else}
          <Icon name="plus" size={12} />
        {/if}
        {creating ? "Creating..." : "Create"}
      </button>
    {/if}
  </div>

  {#if loading}
    <div class="snaps__muted">
      <span class="spinner spinner--sm"></span>
      Loading snapshots...
    </div>
  {:else if snapshots.length === 0}
    <p class="snaps__muted">No snapshots yet.</p>
  {:else}
    <div class="snaps__list">
      {#each snapshots as snap}
        <div class="snaps__item">
          <div class="snaps__item-info">
            <span class="snaps__item-date">{snap.created_at}</span>
            <span class="snaps__item-size mono">{formatSize(snap.size_bytes)}</span>
            {#if snap.db_dump_path}
              <span class="badge badge--neutral">
                <Icon name="database" size={10} />
                DB
              </span>
            {/if}
          </div>

          {#if confirmDelete === snap.id}
            <div class="snaps__confirm">
              <span>Delete?</span>
              <button class="btn-danger btn-sm" onclick={() => deleteSnap(snap.id)}>Confirm</button>
              <button class="btn-ghost btn-sm" onclick={() => (confirmDelete = null)}>Cancel</button>
            </div>
          {:else if confirmRestore === snap.id}
            <div class="snaps__confirm">
              <span>Restore?</span>
              <button class="btn-primary btn-sm" onclick={() => restoreSnap(snap.id)}>Confirm</button>
              <button class="btn-ghost btn-sm" onclick={() => (confirmRestore = null)}>Cancel</button>
            </div>
          {:else}
            <div class="snaps__item-actions">
              <button class="btn-icon" onclick={() => (confirmRestore = snap.id)} aria-label="Restore snapshot">
                <Icon name="upload" size={14} />
              </button>
              <button class="btn-icon" onclick={() => (confirmDelete = snap.id)} aria-label="Delete snapshot">
                <Icon name="trash" size={14} />
              </button>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .snaps {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .snaps__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .snaps__header h3 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .snaps__header button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .snaps__muted {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-align: center;
    padding: var(--space-4);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
  }

  .snaps__list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .snaps__item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border-subtle);
    font-size: var(--text-sm);
  }

  .snaps__item:hover {
    background: var(--color-bg-hover);
  }

  .snaps__item-info {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .snaps__item-date {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .snaps__item-size {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .snaps__item-actions {
    display: flex;
    gap: var(--space-1);
  }

  .snaps__confirm {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    animation: fade-in 150ms ease;
  }
</style>
