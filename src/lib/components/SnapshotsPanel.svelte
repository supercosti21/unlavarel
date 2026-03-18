<script>
  import { invoke } from "@tauri-apps/api/core";

  let { projectName = null, projectPath = "", database = null } = $props();

  let snapshots = $state([]);
  let loading = $state(true);
  let creating = $state(false);
  let error = $state(null);
  let message = $state(null);

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
    error = null;
    try {
      await invoke("create_snapshot", {
        projectName,
        projectPath,
        database,
      });
      message = "Snapshot created";
      await loadSnapshots();
    } catch (e) {
      error = String(e);
    } finally {
      creating = false;
    }
  }

  async function restoreSnap(id) {
    error = null;
    try {
      const result = await invoke("restore_snapshot", {
        snapshotId: id,
        targetPath: projectPath,
        restoreDb: !!database,
      });
      message = result;
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteSnap(id) {
    try {
      await invoke("delete_snapshot", { snapshotId: id });
      await loadSnapshots();
    } catch (e) {
      error = String(e);
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
    <h3>Snapshots</h3>
    {#if projectName}
      <button class="btn-primary" onclick={createSnapshot} disabled={creating}>
        {creating ? "Creating..." : "Create Snapshot"}
      </button>
    {/if}
  </div>

  {#if message}
    <div class="snaps__message badge badge--success">{message}</div>
  {/if}
  {#if error}
    <div class="snaps__message badge badge--danger">{error}</div>
  {/if}

  {#if loading}
    <p class="snaps__muted">Loading snapshots...</p>
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
              <span class="badge badge--neutral">DB</span>
            {/if}
          </div>
          <div class="snaps__item-actions">
            <button class="btn-ghost" onclick={() => restoreSnap(snap.id)}>Restore</button>
            <button class="btn-ghost" onclick={() => deleteSnap(snap.id)}>Delete</button>
          </div>
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
  }

  .snaps__muted {
    color: var(--color-text-muted);
    font-size: var(--text-sm);
    text-align: center;
    padding: var(--space-4);
  }

  .snaps__message {
    padding: var(--space-2);
    font-size: var(--text-xs);
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
</style>
