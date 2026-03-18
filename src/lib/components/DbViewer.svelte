<script>
  import { invoke } from "@tauri-apps/api/core";

  let conn = $state(null);
  let databases = $state([]);
  let selectedDb = $state(null);
  let tables = $state([]);
  let selectedTable = $state(null);
  let columns = $state([]);
  let queryText = $state("");
  let queryResult = $state(null);
  let loading = $state(true);
  let error = $state(null);
  let newDbName = $state("");
  let showCreateDb = $state(false);
  let tableData = $state(null);

  $effect(() => {
    init();
  });

  async function init() {
    loading = true;
    try {
      conn = await invoke("db_get_connection");
      await loadDatabases();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function loadDatabases() {
    error = null;
    try {
      databases = await invoke("db_list_databases", { conn });
    } catch (e) {
      error = String(e);
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
    try {
      tables = await invoke("db_list_tables", { conn, database: name });
    } catch (e) {
      error = String(e);
      tables = [];
    }
  }

  async function selectTable(name) {
    selectedTable = name;
    try {
      columns = await invoke("db_describe_table", { conn, database: selectedDb, table: name });
      // Auto-load first 100 rows
      tableData = await invoke("db_run_query", {
        conn,
        database: selectedDb,
        query: `SELECT * FROM ${conn.db_type === 'postgresql' ? '"' + name + '"' : '`' + name + '`'} LIMIT 100`,
      });
    } catch (e) {
      error = String(e);
    }
  }

  async function createDatabase() {
    if (!newDbName.trim()) return;
    error = null;
    try {
      await invoke("db_create_database", { conn, name: newDbName.trim() });
      newDbName = "";
      showCreateDb = false;
      await loadDatabases();
    } catch (e) {
      error = String(e);
    }
  }

  async function dropDatabase(name) {
    error = null;
    try {
      await invoke("db_drop_database", { conn, name });
      if (selectedDb === name) {
        selectedDb = null;
        tables = [];
        selectedTable = null;
      }
      await loadDatabases();
    } catch (e) {
      error = String(e);
    }
  }

  async function runQuery() {
    if (!queryText.trim() || !selectedDb) return;
    error = null;
    queryResult = null;
    try {
      queryResult = await invoke("db_run_query", {
        conn,
        database: selectedDb,
        query: queryText.trim(),
      });
    } catch (e) {
      error = String(e);
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
    <div class="dbm__loading">Connecting to database...</div>
  {:else}
    <!-- Sidebar: database list -->
    <div class="dbm__sidebar">
      <div class="dbm__sidebar-header">
        <h3>Databases</h3>
        <button class="btn-ghost dbm__btn-sm" onclick={() => (showCreateDb = !showCreateDb)}>+</button>
      </div>

      {#if showCreateDb}
        <div class="dbm__create-form">
          <input
            type="text"
            bind:value={newDbName}
            placeholder="new_database"
            onkeydown={(e) => e.key === "Enter" && createDatabase()}
          />
          <button class="btn-primary dbm__btn-sm" onclick={createDatabase}>Create</button>
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
        <div class="dbm__error">{error}</div>
      {/if}

      {#if !selectedDb}
        <div class="dbm__placeholder">Select a database from the sidebar</div>
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
            <button class="btn-danger dbm__btn-sm" onclick={() => dropDatabase(selectedDb)}>Drop DB</button>
          {/if}
        </div>

        <!-- Table structure -->
        {#if selectedTable && columns.length > 0}
          <div class="dbm__structure">
            <h4>Structure: {selectedTable}</h4>
            <div class="dbm__table-wrap">
              <table class="dbm__table">
                <thead>
                  <tr>
                    <th>Column</th>
                    <th>Type</th>
                    <th>Null</th>
                    <th>Key</th>
                    <th>Default</th>
                    <th>Extra</th>
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
          </div>
        {/if}

        <!-- Data preview -->
        {#if tableData && tableData.columns.length > 0}
          <div class="dbm__data">
            <h4>Data: {selectedTable} <span class="dbm__data-count">({tableData.message})</span></h4>
            <div class="dbm__table-wrap">
              <table class="dbm__table">
                <thead>
                  <tr>
                    {#each tableData.columns as col}
                      <th>{col}</th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each tableData.rows as row}
                    <tr>
                      {#each row as cell}
                        <td>{cell}</td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}

        <!-- Query runner -->
        <div class="dbm__query">
          <div class="dbm__query-header">
            <h4>Query</h4>
            <div class="dbm__query-actions">
              <span class="dbm__query-hint">Ctrl+Enter to run</span>
              <button class="btn-primary dbm__btn-sm" onclick={runQuery}>Run</button>
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
          <div class="dbm__results">
            <div class="dbm__results-header">
              <h4>Results</h4>
              <span class="dbm__results-meta">{queryResult.message}</span>
            </div>
            {#if queryResult.columns.length > 0}
              <div class="dbm__table-wrap">
                <table class="dbm__table">
                  <thead>
                    <tr>
                      {#each queryResult.columns as col}
                        <th>{col}</th>
                      {/each}
                    </tr>
                  </thead>
                  <tbody>
                    {#each queryResult.rows as row}
                      <tr>
                        {#each row as cell}
                          <td>{cell}</td>
                        {/each}
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
          </div>
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
    color: var(--color-text-muted);
    padding: var(--space-8);
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
    padding: var(--space-3) var(--space-3);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .dbm__sidebar-header h3 {
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .dbm__btn-sm {
    padding: var(--space-1) var(--space-2);
    font-size: var(--text-xs);
  }

  .dbm__create-form {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-2);
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

  /* Structure & Data */
  .dbm__structure, .dbm__data, .dbm__results {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .dbm__structure h4, .dbm__data h4, .dbm__query h4, .dbm__results h4 {
    font-size: var(--text-sm);
    font-weight: var(--font-semibold);
    color: var(--color-text-secondary);
  }

  .dbm__data-count, .dbm__results-meta {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    font-weight: var(--font-normal);
  }

  .dbm__results-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .dbm__table-wrap {
    overflow-x: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }

  .dbm__table {
    width: 100%;
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
  }

  .dbm__table td {
    padding: var(--space-1) var(--space-3);
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-primary);
    max-width: 300px;
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

  .dbm__query-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
