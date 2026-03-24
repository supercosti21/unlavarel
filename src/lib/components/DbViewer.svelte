<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let conn = $state(null);
  let databases = $state([]);
  let selectedDb = $state(null);
  let tables = $state([]);
  let selectedTable = $state(null);
  let columns = $state([]);
  let queryText = $state("");
  let queryResult = $state(null);
  let loading = $state(true);
  let queryRunning = $state(false);
  let error = $state(null);
  let connectionError = $state(false);
  let newDbName = $state("");
  let showCreateDb = $state(false);
  let tableData = $state(null);
  let confirmDrop = $state(null);

  function friendlyDbError(raw) {
    const msg = String(raw).toLowerCase();
    if (msg.includes("can't connect") || msg.includes("connection refused") || msg.includes("hy000")) {
      return "Database service is not running. Start it from the Dashboard first.";
    }
    if (msg.includes("access denied")) {
      return "Access denied. Check your database credentials.";
    }
    if (msg.includes("unknown database")) {
      return "Database not found. It may have been dropped.";
    }
    if (msg.includes("already exists")) {
      return "A database with that name already exists.";
    }
    if (msg.includes("syntax error") || msg.includes("you have an error in your sql")) {
      return "SQL syntax error. Please review your query.";
    }
    if (msg.includes("no such")) {
      return "Database server not found. Is it installed?";
    }
    return String(raw);
  }

  $effect(() => {
    init();
  });

  async function init() {
    loading = true;
    connectionError = false;
    try {
      conn = await invoke("db_get_connection");
      await loadDatabases();
    } catch (e) {
      const msg = String(e).toLowerCase();
      connectionError = msg.includes("can't connect") || msg.includes("connection refused") || msg.includes("hy000") || msg.includes("no such");
      error = friendlyDbError(e);
    } finally {
      loading = false;
    }
  }

  async function loadDatabases() {
    error = null;
    try {
      databases = await invoke("db_list_databases", { conn });
    } catch (e) {
      error = friendlyDbError(e);
      databases = [];
    }
  }

  async function selectDatabase(name) {
    selectedDb = name;
    selectedTable = null;
    columns = [];
    tableData = null;
    queryResult = null;
    queryText = "";
    confirmDrop = null;
    try {
      tables = await invoke("db_list_tables", { conn, database: name });
    } catch (e) {
      error = friendlyDbError(e);
      tables = [];
    }
  }

  async function selectTable(name) {
    selectedTable = name;
    try {
      columns = await invoke("db_describe_table", { conn, database: selectedDb, table: name });
      tableData = await invoke("db_run_query", {
        conn,
        database: selectedDb,
        query: `SELECT * FROM ${conn.db_type === 'postgresql' ? '"' + name + '"' : '`' + name + '`'} LIMIT 100`,
      });
    } catch (e) {
      error = friendlyDbError(e);
    }
  }

  async function createDatabase() {
    if (!newDbName.trim()) return;
    error = null;
    try {
      await invoke("db_create_database", { conn, name: newDbName.trim() });
      toastStore.success(`Database "${newDbName.trim()}" created`);
      newDbName = "";
      showCreateDb = false;
      await loadDatabases();
    } catch (e) {
      toastStore.error(friendlyDbError(e));
    }
  }

  async function dropDatabase(name) {
    error = null;
    try {
      await invoke("db_drop_database", { conn, name });
      toastStore.success(`Database "${name}" dropped`);
      if (selectedDb === name) {
        selectedDb = null;
        tables = [];
        selectedTable = null;
      }
      confirmDrop = null;
      await loadDatabases();
    } catch (e) {
      toastStore.error(friendlyDbError(e));
      confirmDrop = null;
    }
  }

  async function runQuery() {
    if (!queryText.trim() || !selectedDb) return;
    error = null;
    queryResult = null;
    queryRunning = true;
    try {
      queryResult = await invoke("db_run_query", {
        conn,
        database: selectedDb,
        query: queryText.trim(),
      });
      toastStore.success(queryResult.message || "Query executed");
    } catch (e) {
      toastStore.error(friendlyDbError(e));
    } finally {
      queryRunning = false;
    }
  }

  function handleKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      runQuery();
    }
  }
</script>

<div class="dbm">
  {#if loading}
    <div class="dbm__loading">
      <span class="spinner"></span>
      <span>Connecting to database...</span>
    </div>
  {:else if connectionError}
    <div class="dbm__loading">
      <Icon name="database" size={40} />
      <h3 style="font-size: var(--text-base); color: var(--color-text-secondary)">Database not available</h3>
      <p style="font-size: var(--text-sm); color: var(--color-text-muted); max-width: 300px; text-align: center; line-height: 1.5">
        Start your database service (MySQL, MariaDB, or PostgreSQL) from the Dashboard, then come back here.
      </p>
      <button class="btn-primary" onclick={init} style="margin-top: var(--space-2)">
        <Icon name="refresh" size={14} />
        Retry Connection
      </button>
    </div>
  {:else}
    <!-- Sidebar: database list -->
    <div class="dbm__sidebar">
      <div class="dbm__sidebar-header">
        <h3>Databases</h3>
        <button class="btn-icon" onclick={() => (showCreateDb = !showCreateDb)} aria-label="Create database">
          <Icon name="plus" size={16} />
        </button>
      </div>

      {#if showCreateDb}
        <div class="dbm__create-form">
          <input
            type="text"
            bind:value={newDbName}
            placeholder="new_database"
            onkeydown={(e) => e.key === "Enter" && createDatabase()}
          />
          <button class="btn-primary btn-sm" onclick={createDatabase}>Create</button>
        </div>
      {/if}

      <div class="dbm__db-list">
        {#each databases as db}
          <button
            class="dbm__db-item"
            class:dbm__db-item--active={selectedDb === db.name}
            onclick={() => selectDatabase(db.name)}
          >
            <span class="dbm__db-name">{db.name}</span>
            <span class="dbm__db-meta">{db.tables_count} tables &middot; {db.size}</span>
          </button>
        {/each}
        {#if databases.length === 0}
          <p class="dbm__empty">No databases found</p>
        {/if}
      </div>

      {#if conn}
        <div class="dbm__conn-info">
          <span class="mono">{conn.db_type}</span>
          <span class="mono">{conn.host}:{conn.port}</span>
        </div>
      {/if}
    </div>

    <!-- Main area -->
    <div class="dbm__main">
      {#if error}
        <div class="dbm__error">
          <Icon name="alert-circle" size={14} />
          <span>{error}</span>
        </div>
      {/if}

      {#if !selectedDb}
        <div class="dbm__placeholder">
          <Icon name="database" size={32} />
          <span>Select a database from the sidebar</span>
        </div>
      {:else}
        <!-- Table list -->
        <div class="dbm__tables-bar">
          <div class="dbm__tables-list">
            {#each tables as tbl}
              <button
                class="dbm__table-chip"
                class:dbm__table-chip--active={selectedTable === tbl.name}
                onclick={() => selectTable(tbl.name)}
              >
                {tbl.name}
                <span class="dbm__table-chip-meta">{tbl.rows}</span>
              </button>
            {/each}
          </div>
          {#if selectedDb}
            {#if confirmDrop === selectedDb}
              <div class="dbm__confirm-drop">
                <span>Drop "{selectedDb}"?</span>
                <button class="btn-danger btn-sm" onclick={() => dropDatabase(selectedDb)}>Confirm</button>
                <button class="btn-ghost btn-sm" onclick={() => (confirmDrop = null)}>Cancel</button>
              </div>
            {:else}
              <button class="btn-danger btn-sm" onclick={() => (confirmDrop = selectedDb)}>
                <Icon name="trash" size={12} />
                Drop DB
              </button>
            {/if}
          {/if}
        </div>

        <!-- Table structure -->
        {#if selectedTable && columns.length > 0}
          <details class="dbm__section" open>
            <summary class="dbm__section-header">
              <Icon name="database" size={14} />
              <span>Structure: {selectedTable}</span>
              <span class="dbm__section-count">{columns.length} columns</span>
            </summary>
            <div class="dbm__table-wrap">
              <table class="dbm__table">
                <thead>
                  <tr>
                    <th style="min-width: 160px">Column</th>
                    <th style="min-width: 120px">Type</th>
                    <th style="min-width: 60px">Null</th>
                    <th style="min-width: 60px">Key</th>
                    <th style="min-width: 100px">Default</th>
                    <th style="min-width: 80px">Extra</th>
                  </tr>
                </thead>
                <tbody>
                  {#each columns as col}
                    <tr>
                      <td class="mono">{col.name}</td>
                      <td class="mono">{col.col_type}</td>
                      <td>{col.nullable ? "YES" : "NO"}</td>
                      <td>{col.key}</td>
                      <td class="mono">{col.default_val}</td>
                      <td>{col.extra}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </details>
        {/if}

        <!-- Data preview -->
        {#if tableData && tableData.columns.length > 0}
          <details class="dbm__section" open>
            <summary class="dbm__section-header">
              <Icon name="grid" size={14} />
              <span>Data: {selectedTable}</span>
              <span class="dbm__section-count">{tableData.message}</span>
            </summary>
            <div class="dbm__table-wrap">
              <table class="dbm__table">
                <thead>
                  <tr>
                    <th style="min-width: 40px; color: var(--color-text-muted)">#</th>
                    {#each tableData.columns as col}
                      <th>{col}</th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each tableData.rows as row, i}
                    <tr>
                      <td style="color: var(--color-text-muted)">{i + 1}</td>
                      {#each row as cell}
                        <td title={cell}>{cell}</td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </details>
        {/if}

        <!-- Query runner -->
        <div class="dbm__query">
          <div class="dbm__query-header">
            <h4>
              <Icon name="terminal" size={14} />
              Query
            </h4>
            <div class="dbm__query-actions">
              <span class="dbm__query-hint">Ctrl+Enter to run</span>
              <button class="btn-primary btn-sm" onclick={runQuery} disabled={queryRunning}>
                {#if queryRunning}
                  <span class="spinner spinner--sm"></span>
                {:else}
                  <Icon name="play" size={12} />
                {/if}
                Run
              </button>
            </div>
          </div>
          <textarea
            class="dbm__query-input"
            bind:value={queryText}
            placeholder="SELECT * FROM users LIMIT 10;"
            onkeydown={handleKeydown}
            rows="4"
          ></textarea>
        </div>

        <!-- Query results -->
        {#if queryResult}
          <details class="dbm__section" open>
            <summary class="dbm__section-header">
              <Icon name="check" size={14} />
              <span>Results</span>
              <span class="dbm__section-count">{queryResult.message}</span>
            </summary>
            {#if queryResult.columns.length > 0}
              <div class="dbm__table-wrap">
                <table class="dbm__table">
                  <thead>
                    <tr>
                      <th style="min-width: 40px; color: var(--color-text-muted)">#</th>
                      {#each queryResult.columns as col}
                        <th>{col}</th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody>
                    {#each queryResult.rows as row, i}
                      <tr>
                        <td style="color: var(--color-text-muted)">{i + 1}</td>
                        {#each row as cell}
                          <td title={cell}>{cell}</td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </details>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .dbm {
    display: flex;
    height: 100%;
    gap: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--color-bg-card);
  }

  .dbm__loading, .dbm__placeholder {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    color: var(--color-text-muted);
    padding: var(--space-8);
    flex-direction: column;
  }

  /* Sidebar */
  .dbm__sidebar {
    width: 220px;
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    background: var(--color-bg-secondary);
  }

  .dbm__sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .dbm__sidebar-header h3 {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .dbm__create-form {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-2);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .dbm__create-form input {
    flex: 1;
    font-size: var(--text-xs);
    padding: var(--space-1) var(--space-2);
  }

  .dbm__db-list {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-1);
  }

  .dbm__db-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    text-align: left;
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--color-text-primary);
    transition: background var(--transition-fast);
  }

  .dbm__db-item:hover {
    background: var(--color-bg-hover);
  }

  .dbm__db-item--active {
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .dbm__db-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    font-family: var(--font-mono);
  }

  .dbm__db-meta {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .dbm__empty {
    padding: var(--space-4);
    text-align: center;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .dbm__conn-info {
    padding: var(--space-2) var(--space-3);
    border-top: 1px solid var(--color-border-subtle);
    display: flex;
    flex-direction: column;
    gap: 2px;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  /* Main */
  .dbm__main {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
  }

  .dbm__error {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--text-xs);
  }

  .dbm__tables-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .dbm__tables-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    flex: 1;
  }

  .dbm__table-chip {
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border-subtle);
    color: var(--color-text-secondary);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: var(--space-1);
    transition: all var(--transition-fast);
  }

  .dbm__table-chip:hover {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dbm__table-chip--active {
    background: var(--color-accent-subtle);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .dbm__table-chip-meta {
    color: var(--color-text-muted);
    font-size: 10px;
  }

  .dbm__confirm-drop {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-danger);
    animation: fade-in 150ms ease;
  }

  /* Collapsible sections */
  .dbm__section {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .dbm__section-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--color-bg-tertiary);
    cursor: pointer;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    list-style: none;
    user-select: none;
  }

  .dbm__section-header::-webkit-details-marker {
    display: none;
  }

  .dbm__section-count {
    margin-left: auto;
    font-weight: var(--font-normal);
    color: var(--color-text-muted);
    text-transform: none;
    letter-spacing: normal;
  }

  /* Tables */
  .dbm__table-wrap {
    overflow-x: auto;
  }

  .dbm__table {
    min-width: 100%;
    border-collapse: collapse;
    font-size: var(--text-xs);
  }

  .dbm__table th {
    text-align: left;
    padding: var(--space-2) var(--space-3);
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
    font-weight: var(--font-semibold);
    border-bottom: 1px solid var(--color-border);
    white-space: nowrap;
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .dbm__table td {
    padding: var(--space-1) var(--space-3);
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-primary);
    max-width: 250px;
    min-width: 80px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dbm__table tr:hover td {
    background: var(--color-bg-hover);
  }

  /* Query */
  .dbm__query {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .dbm__query-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .dbm__query-header h4 {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
  }

  .dbm__query-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .dbm__query-actions button {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
  }

  .dbm__query-hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .dbm__query-input {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    background: var(--color-bg-input);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    padding: var(--space-3);
    resize: vertical;
    outline: none;
    line-height: 1.6;
  }

  .dbm__query-input:focus {
    border-color: var(--color-border-focus);
  }
</style>
