<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onComplete } = $props();

  let step = $state(1);
  let setupState = $state(null);
  let preScan = $state(null);
  let loading = $state(true);
  let scanning = $state(false);
  let installing = $state(false);
  let installResults = $state([]);
  let error = $state(null);

  let phpVersion = $state("8.3");
  let database = $state("mariadb");
  let dbVersion = $state("");
  let extras = $state(["redis", "mailpit", "node"]);
  let nodeVersion = $state("22");
  let projectRoot = $state("");

  $effect(() => {
    init();
  });

  async function init() {
    loading = true;
    try {
      setupState = await invoke("check_setup");
      try {
        const settings = await invoke("get_settings");
        projectRoot = settings.project_root;
      } catch {}
      // Pre-scan system
      scanning = true;
      preScan = await invoke("pre_scan_system");
      scanning = false;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function bootstrapPM() {
    installing = true;
    error = null;
    try {
      await invoke("bootstrap_package_manager");
      setupState = await invoke("check_setup");
      step = 2;
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
    }
  }

  function toggleExtra(name) {
    if (extras.includes(name)) {
      extras = extras.filter((e) => e !== name);
    } else {
      extras = [...extras, name];
    }
  }

  function isAlreadyInstalled(name) {
    return preScan?.installed?.some((i) =>
      i.name.toLowerCase().includes(name.toLowerCase())
    );
  }

  async function installStack() {
    installing = true;
    error = null;
    step = 3;
    try {
      installResults = await invoke("install_stack", {
        selection: {
          php_version: phpVersion,
          database,
          database_version: dbVersion || null,
          extras,
          node_version: extras.includes("node") ? nodeVersion : null,
        },
      });
      // Save project root
      if (projectRoot) {
        try {
          const settings = await invoke("get_settings");
          settings.project_root = projectRoot;
          await invoke("save_settings", { settings });
        } catch {}
      }
      step = 4;
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
    }
  }

  async function finishSetup() {
    await invoke("mark_setup_complete");
    // Run discovery to cache installed services
    try { await invoke("discover_services"); } catch {}
    onComplete();
  }

  async function skipSetup() {
    await invoke("mark_setup_complete");
    try { await invoke("discover_services"); } catch {}
    onComplete();
  }
</script>

<div class="wiz">
  <div class="wiz__steps">
    {#each [
      { n: 1, label: "System" },
      { n: 2, label: "Stack" },
      { n: 3, label: "Install" },
      { n: 4, label: "Done" },
    ] as s}
      <div class="wiz__step" class:wiz__step--active={step === s.n} class:wiz__step--done={step > s.n}>
        <span class="wiz__step-num">{step > s.n ? "✓" : s.n}</span>
        <span class="wiz__step-label">{s.label}</span>
      </div>
      {#if s.n < 4}<div class="wiz__step-line" class:wiz__step-line--done={step > s.n}></div>{/if}
    {/each}
  </div>

  <div class="wiz__card">
    {#if loading}
      <div class="wiz__center">
        <div class="wiz__spinner"></div>
        <p>Scanning your system...</p>
      </div>

    {:else if step === 1}
      <h2>System Check</h2>
      <p class="wiz__sub">MacEnv detected the following on your system.</p>

      {#if setupState}
        <div class="wiz__grid">
          <div class="wiz__info-item">
            <span class="wiz__info-label">Platform</span>
            <span class="wiz__info-value">{setupState.platform.os}</span>
          </div>
          <div class="wiz__info-item">
            <span class="wiz__info-label">Architecture</span>
            <span class="wiz__info-value">{setupState.platform.arch}</span>
          </div>
          <div class="wiz__info-item">
            <span class="wiz__info-label">Package Manager</span>
            <span class="wiz__info-value" class:wiz__info-value--ok={setupState.package_manager_available} class:wiz__info-value--err={!setupState.package_manager_available}>
              {setupState.package_manager_name}
            </span>
          </div>
          {#if setupState.platform.linux_distro}
            <div class="wiz__info-item">
              <span class="wiz__info-label">Distribution</span>
              <span class="wiz__info-value">{JSON.stringify(setupState.platform.linux_distro).replace(/"/g, '')}</span>
            </div>
          {/if}
        </div>
      {/if}

      {#if preScan && preScan.installed.length > 0}
        <div class="wiz__prescan">
          <h3>Already Installed</h3>
          <div class="wiz__prescan-list">
            {#each preScan.installed as item}
              <div class="wiz__prescan-item wiz__prescan-item--ok">
                <span class="wiz__prescan-dot"></span>
                <span class="wiz__prescan-name">{item.name}</span>
                <span class="wiz__prescan-ver">{item.version}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if preScan && preScan.missing.length > 0}
        <div class="wiz__prescan">
          <h3>Not Installed</h3>
          <div class="wiz__prescan-list">
            {#each preScan.missing as name}
              <div class="wiz__prescan-item wiz__prescan-item--miss">
                <span class="wiz__prescan-dot"></span>
                <span class="wiz__prescan-name">{name}</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <div class="wiz__actions">
        <button class="wiz__btn wiz__btn--ghost" onclick={skipSetup}>Skip Setup</button>
        {#if setupState?.package_manager_available}
          <button class="wiz__btn wiz__btn--primary" onclick={() => (step = 2)}>Configure Stack</button>
        {:else}
          <button class="wiz__btn wiz__btn--primary" onclick={bootstrapPM} disabled={installing}>
            {installing ? "Installing..." : `Install ${setupState?.package_manager_name}`}
          </button>
        {/if}
      </div>

    {:else if step === 2}
      <h2>Choose Your Stack</h2>
      <p class="wiz__sub">Select what to install. Already installed packages will be skipped.</p>

      <div class="wiz__form">
        <label class="wiz__field">
          <span class="wiz__field-label">PHP Version</span>
          <select bind:value={phpVersion}>
            <option value="8.1">PHP 8.1</option>
            <option value="8.2">PHP 8.2</option>
            <option value="8.3">PHP 8.3</option>
            <option value="8.4">PHP 8.4</option>
          </select>
          {#if isAlreadyInstalled("PHP")}
            <span class="wiz__field-badge">already installed</span>
          {/if}
        </label>

        <label class="wiz__field">
          <span class="wiz__field-label">Database</span>
          <select bind:value={database}>
            <option value="mariadb">MariaDB</option>
            <option value="mysql">MySQL</option>
            <option value="postgresql">PostgreSQL</option>
            <option value="none">None</option>
          </select>
          {#if isAlreadyInstalled("MySQL") || isAlreadyInstalled("MariaDB") || isAlreadyInstalled("PostgreSQL")}
            <span class="wiz__field-badge">already installed</span>
          {/if}
        </label>

        {#if database !== "none" && database !== "mariadb"}
          <label class="wiz__field">
            <span class="wiz__field-label">Version</span>
            <select bind:value={dbVersion}>
              {#if database === "mysql"}
                <option value="8.0">8.0</option>
                <option value="8.4">8.4</option>
              {:else if database === "postgresql"}
                <option value="15">15</option>
                <option value="16">16</option>
                <option value="17">17</option>
              {/if}
            </select>
          </label>
        {/if}

        <div class="wiz__field">
          <span class="wiz__field-label">Extras</span>
          <div class="wiz__toggles">
            {#each [
              { id: "redis", label: "Redis", desc: "Cache store" },
              { id: "memcached", label: "Memcached", desc: "Cache" },
              { id: "mailpit", label: "Mailpit", desc: "Email testing" },
              { id: "node", label: "Node.js", desc: "JS runtime" },
            ] as opt}
              <button
                class="wiz__toggle"
                class:wiz__toggle--on={extras.includes(opt.id)}
                onclick={() => toggleExtra(opt.id)}
              >
                <span class="wiz__toggle-name">{opt.label}</span>
                <span class="wiz__toggle-desc">{opt.desc}</span>
                {#if isAlreadyInstalled(opt.label)}
                  <span class="wiz__field-badge">installed</span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        {#if extras.includes("node")}
          <label class="wiz__field">
            <span class="wiz__field-label">Node.js Version</span>
            <select bind:value={nodeVersion}>
              <option value="18">18 LTS</option>
              <option value="20">20 LTS</option>
              <option value="22">22 LTS</option>
            </select>
          </label>
        {/if}

        <label class="wiz__field">
          <span class="wiz__field-label">Projects Directory</span>
          <input type="text" bind:value={projectRoot} placeholder="/home/user/projects" />
        </label>
      </div>

      <div class="wiz__actions">
        <button class="wiz__btn wiz__btn--ghost" onclick={() => (step = 1)}>Back</button>
        <button class="wiz__btn wiz__btn--primary" onclick={installStack}>Install</button>
      </div>

    {:else if step === 3}
      <div class="wiz__center">
        <div class="wiz__spinner"></div>
        <h2>Installing</h2>
        <p class="wiz__sub">This may take a few minutes. You'll be asked for your password once.</p>
      </div>

    {:else if step === 4}
      <div class="wiz__center">
        <div class="wiz__check-icon">✓</div>
        <h2>All Set</h2>
      </div>
      <div class="wiz__results">
        {#each installResults as result}
          <div
            class="wiz__result"
            class:wiz__result--ok={result.includes("installed")}
            class:wiz__result--fail={result.includes("failed") || result.includes("Error")}
          >
            {result}
          </div>
        {/each}
      </div>
      <div class="wiz__actions wiz__actions--center">
        <button class="wiz__btn wiz__btn--primary" onclick={finishSetup}>Open MacEnv</button>
      </div>
    {/if}

    {#if error}
      <div class="wiz__error">{error}</div>
    {/if}
  </div>
</div>

<style>
  .wiz {
    width: 100%;
    max-width: 520px;
    margin: var(--space-6) auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-6);
  }

  /* Steps indicator */
  .wiz__steps {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0;
    padding: 0 var(--space-4);
  }

  .wiz__step {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
  }

  .wiz__step-num {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    background: var(--color-bg-tertiary);
    color: var(--color-text-muted);
    transition: all var(--transition-normal);
  }

  .wiz__step--active .wiz__step-num {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  .wiz__step--done .wiz__step-num {
    background: var(--color-success);
    color: var(--color-text-on-accent);
  }

  .wiz__step-label {
    font-size: 10px;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .wiz__step--active .wiz__step-label { color: var(--color-text-primary); }

  .wiz__step-line {
    width: 40px;
    height: 2px;
    background: var(--color-border);
    margin: 0 var(--space-2);
    margin-bottom: 18px;
    border-radius: 1px;
  }

  .wiz__step-line--done { background: var(--color-success); }

  /* Card */
  .wiz__card {
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .wiz__card h2 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
  }

  .wiz__sub {
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    margin-top: calc(-1 * var(--space-2));
  }

  .wiz__center {
    text-align: center;
    padding: var(--space-4) 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }

  .wiz__spinner {
    width: 36px;
    height: 36px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .wiz__check-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-success);
    color: var(--color-text-on-accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  /* Info grid (step 1) */
  .wiz__grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .wiz__info-item {
    padding: var(--space-3);
    background: var(--color-bg-tertiary);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .wiz__info-label {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .wiz__info-value {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .wiz__info-value--ok { color: var(--color-success); }
  .wiz__info-value--err { color: var(--color-danger); }

  /* Pre-scan */
  .wiz__prescan {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .wiz__prescan h3 {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .wiz__prescan-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .wiz__prescan-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-full);
    font-size: var(--text-xs);
  }

  .wiz__prescan-item--ok {
    background: var(--color-success-subtle);
    color: var(--color-success);
  }

  .wiz__prescan-item--miss {
    background: var(--color-bg-tertiary);
    color: var(--color-text-muted);
  }

  .wiz__prescan-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .wiz__prescan-item--ok .wiz__prescan-dot { background: var(--color-success); }
  .wiz__prescan-item--miss .wiz__prescan-dot { background: var(--color-text-muted); }

  .wiz__prescan-name { font-weight: var(--font-medium); }

  .wiz__prescan-ver {
    font-family: var(--font-mono);
    font-size: 10px;
    color: inherit;
    opacity: 0.7;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Form (step 2) */
  .wiz__form {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .wiz__field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    position: relative;
  }

  .wiz__field-label {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .wiz__field-badge {
    position: absolute;
    right: var(--space-2);
    top: 0;
    font-size: 10px;
    color: var(--color-success);
    font-weight: var(--font-medium);
  }

  /* Toggle buttons for extras */
  .wiz__toggles {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-2);
  }

  .wiz__toggle {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: var(--space-3);
    border-radius: var(--radius-md);
    background: var(--color-bg-tertiary);
    border: 1px solid var(--color-border-subtle);
    cursor: pointer;
    transition: all var(--transition-fast);
    text-align: left;
    color: var(--color-text-secondary);
    position: relative;
  }

  .wiz__toggle:hover {
    border-color: var(--color-border);
    background: var(--color-bg-hover);
  }

  .wiz__toggle--on {
    border-color: var(--color-accent);
    background: var(--color-accent-subtle);
    color: var(--color-text-primary);
  }

  .wiz__toggle-name {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
  }

  .wiz__toggle-desc {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .wiz__toggle .wiz__field-badge {
    top: var(--space-2);
    right: var(--space-2);
  }

  /* Results */
  .wiz__results {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .wiz__result {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-bg-tertiary);
    color: var(--color-text-secondary);
  }

  .wiz__result--ok { color: var(--color-success); background: var(--color-success-subtle); }
  .wiz__result--fail { color: var(--color-danger); background: var(--color-danger-subtle); }

  /* Actions */
  .wiz__actions {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
    padding-top: var(--space-2);
  }

  .wiz__actions--center { justify-content: center; }

  .wiz__btn {
    padding: var(--space-2) var(--space-5);
    border-radius: var(--radius-md);
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    cursor: pointer;
    border: none;
    transition: all var(--transition-fast);
  }

  .wiz__btn--primary {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  .wiz__btn--primary:hover { background: var(--color-accent-hover); }

  .wiz__btn--ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .wiz__btn--ghost:hover { background: var(--color-bg-hover); color: var(--color-text-primary); }

  .wiz__btn:disabled { opacity: 0.5; cursor: not-allowed; }

  /* Error */
  .wiz__error {
    padding: var(--space-3);
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
  }
</style>
