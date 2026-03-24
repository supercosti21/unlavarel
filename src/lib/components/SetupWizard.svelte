<script>
  import { invoke } from "@tauri-apps/api/core";
  import Icon from "./Icon.svelte";
  import { toastStore } from "../stores/toast.svelte.js";

  let { onComplete } = $props();

  let step = $state(1);
  let setupState = $state(null);
  let preScan = $state(null);
  let loading = $state(true);
  let scanning = $state(false);
  let installing = $state(false);
  let installResults = $state([]);
  let error = $state(null);
  let needsPassword = $state(false);

  // Stack selection — will be pre-filled from scan
  let phpVersion = $state("8.3");
  let database = $state("mariadb");
  let dbVersion = $state("");
  let extras = $state(["redis", "mailpit", "node"]);
  let nodeVersion = $state("22");
  let projectRoot = $state("");

  // Track what's already installed (by id)
  let installedIds = $derived(new Set((preScan?.installed || []).map(i => i.id)));

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

      // Smart pre-selection based on what's already installed
      applyPreSelection();

      // Check if we need elevated privileges (Linux)
      needsPassword = setupState.platform.os === "Linux";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function applyPreSelection() {
    if (!preScan) return;

    for (const item of preScan.installed) {
      // Pre-select PHP version from installed
      if (item.id === "php" && item.version_number) {
        const ver = item.version_number;
        if (["8.1", "8.2", "8.3", "8.4"].includes(ver)) {
          phpVersion = ver;
        }
      }

      // Pre-select database from installed
      if (item.id === "mariadb") {
        database = "mariadb";
      } else if (item.id === "mysql") {
        database = "mysql";
      } else if (item.id === "postgresql") {
        database = "postgresql";
      }

      // Pre-select node version
      if (item.id === "node" && item.version_number) {
        const ver = item.version_number;
        if (["18", "20", "22"].includes(ver)) {
          nodeVersion = ver;
        }
      }

      // Auto-toggle extras that are installed
      if (item.id === "redis" && !extras.includes("redis")) extras = [...extras, "redis"];
      if (item.id === "memcached" && !extras.includes("memcached")) extras = [...extras, "memcached"];
      if (item.id === "mailpit" && !extras.includes("mailpit")) extras = [...extras, "mailpit"];
      if (item.id === "node" && !extras.includes("node")) extras = [...extras, "node"];
    }
  }

  function isInstalled(id) {
    return preScan?.installed?.some(i => i.id === id);
  }

  function getInstalledVersion(id) {
    const item = preScan?.installed?.find(i => i.id === id);
    return item?.version_number || null;
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
      extras = extras.filter(e => e !== name);
    } else {
      extras = [...extras, name];
    }
  }

  // Count how many packages need to be installed
  let toInstallCount = $derived.by(() => {
    let count = 0;
    if (!isInstalled("php")) count++;
    if (database !== "none" && !isInstalled(database)) count++;
    if (!isInstalled("nginx")) count++;
    if (!isInstalled("composer")) count++;
    for (const ext of extras) {
      if (!isInstalled(ext)) count++;
    }
    if (!isInstalled("dnsmasq")) count++;
    if (!isInstalled("mkcert")) count++;
    return count;
  });

  let alreadyInstalledCount = $derived.by(() => {
    let count = 0;
    if (isInstalled("php")) count++;
    if (database !== "none" && isInstalled(database)) count++;
    if (isInstalled("nginx")) count++;
    if (isInstalled("composer")) count++;
    for (const ext of extras) {
      if (isInstalled(ext)) count++;
    }
    if (isInstalled("dnsmasq")) count++;
    if (isInstalled("mkcert")) count++;
    return count;
  });

  async function handleInstall() {
    // If on Linux and no cached password, ask for it first
    if (needsPassword) {
      try {
        const hasPwd = await invoke("has_session_password");
        if (!hasPwd) {
          // Show password step
          step = "password";
          return;
        }
      } catch {}
    }
    await installStack();
  }

  let passwordInput = $state("");
  let passwordError = $state(null);
  let passwordSaving = $state(false);

  async function submitPassword() {
    if (!passwordInput) return;
    passwordSaving = true;
    passwordError = null;
    try {
      await invoke("save_session_password", { password: passwordInput });
      passwordInput = "";
      await installStack();
    } catch (e) {
      passwordError = "Incorrect password. Please try again.";
      passwordInput = "";
    } finally {
      passwordSaving = false;
    }
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
      step = 2; // Go back to selection
    } finally {
      installing = false;
    }
  }

  async function finishSetup() {
    await invoke("mark_setup_complete");
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
      <div class="wiz__step" class:wiz__step--active={step === s.n || (step === "password" && s.n === 2)} class:wiz__step--done={typeof step === "number" && step > s.n}>
        <span class="wiz__step-num">
          {#if typeof step === "number" && step > s.n}
            <Icon name="check" size={14} />
          {:else}
            {s.n}
          {/if}
        </span>
        <span class="wiz__step-label">{s.label}</span>
      </div>
      {#if s.n < 4}<div class="wiz__step-line" class:wiz__step-line--done={typeof step === "number" && step > s.n}></div>{/if}
    {/each}
  </div>

  <div class="wiz__card">
    {#if loading}
      <div class="wiz__center">
        <span class="spinner spinner--lg"></span>
        <p>{scanning ? "Scanning your system..." : "Loading..."}</p>
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
          <h3>
            <Icon name="check" size={14} />
            Already Installed ({preScan.installed.length})
          </h3>
          <div class="wiz__prescan-list">
            {#each preScan.installed as item}
              <div class="wiz__prescan-item wiz__prescan-item--ok">
                <span class="wiz__prescan-dot"></span>
                <span class="wiz__prescan-name">{item.name}</span>
                {#if item.version_number}
                  <span class="wiz__prescan-ver">{item.version_number}</span>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if preScan && preScan.missing.length > 0}
        <div class="wiz__prescan">
          <h3>
            <Icon name="plus" size={14} />
            Available to Install ({preScan.missing.length})
          </h3>
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
          <button class="wiz__btn wiz__btn--primary" onclick={() => (step = 2)}>
            Configure Stack
            <Icon name="chevron-down" size={14} />
          </button>
        {:else}
          <button class="wiz__btn wiz__btn--primary" onclick={bootstrapPM} disabled={installing}>
            {#if installing}<span class="spinner spinner--sm"></span>{/if}
            {installing ? "Installing..." : `Install ${setupState?.package_manager_name}`}
          </button>
        {/if}
      </div>

    {:else if step === 2}
      <h2>Choose Your Stack</h2>
      <p class="wiz__sub">
        {#if alreadyInstalledCount > 0}
          {alreadyInstalledCount} already installed.
        {/if}
        {#if toInstallCount > 0}
          {toInstallCount} will be installed.
        {:else}
          Everything is already installed!
        {/if}
      </p>

      <div class="wiz__form">
        <label class="wiz__field">
          <span class="wiz__field-label">
            PHP Version
            {#if isInstalled("php")}
              <span class="wiz__field-badge wiz__field-badge--ok">
                <Icon name="check" size={10} />
                v{getInstalledVersion("php")} installed
              </span>
            {/if}
          </span>
          <select bind:value={phpVersion}>
            <option value="8.1">PHP 8.1</option>
            <option value="8.2">PHP 8.2</option>
            <option value="8.3">PHP 8.3</option>
            <option value="8.4">PHP 8.4</option>
          </select>
        </label>

        <label class="wiz__field">
          <span class="wiz__field-label">
            Database
            {#if isInstalled("mariadb") || isInstalled("mysql") || isInstalled("postgresql")}
              <span class="wiz__field-badge wiz__field-badge--ok">
                <Icon name="check" size={10} />
                installed
              </span>
            {/if}
          </span>
          <select bind:value={database}>
            <option value="mariadb">MariaDB</option>
            <option value="mysql">MySQL</option>
            <option value="postgresql">PostgreSQL</option>
            <option value="none">None</option>
          </select>
        </label>

        {#if database !== "none" && database !== "mariadb"}
          <label class="wiz__field">
            <span class="wiz__field-label">Database Version</span>
            <select bind:value={dbVersion}>
              {#if database === "mysql"}
                <option value="8.0">MySQL 8.0</option>
                <option value="8.4">MySQL 8.4</option>
              {:else if database === "postgresql"}
                <option value="15">PostgreSQL 15</option>
                <option value="16">PostgreSQL 16</option>
                <option value="17">PostgreSQL 17</option>
              {/if}
            </select>
          </label>
        {/if}

        <div class="wiz__field">
          <span class="wiz__field-label">Extras</span>
          <div class="wiz__toggles">
            {#each [
              { id: "redis", label: "Redis", desc: "In-memory cache & queue", icon: "zap" },
              { id: "memcached", label: "Memcached", desc: "Distributed cache", icon: "zap" },
              { id: "mailpit", label: "Mailpit", desc: "Email testing tool", icon: "mail" },
              { id: "node", label: "Node.js", desc: "JavaScript runtime", icon: "code" },
            ] as opt}
              <button
                class="wiz__toggle"
                class:wiz__toggle--on={extras.includes(opt.id)}
                onclick={() => toggleExtra(opt.id)}
              >
                <div class="wiz__toggle-header">
                  <Icon name={opt.icon} size={14} />
                  <span class="wiz__toggle-name">{opt.label}</span>
                </div>
                <span class="wiz__toggle-desc">{opt.desc}</span>
                {#if isInstalled(opt.id)}
                  <span class="wiz__field-badge wiz__field-badge--ok">
                    <Icon name="check" size={10} />
                    installed
                  </span>
                {/if}
              </button>
            {/each}
          </div>
        </div>

        {#if extras.includes("node")}
          <label class="wiz__field">
            <span class="wiz__field-label">
              Node.js Version
              {#if isInstalled("node")}
                <span class="wiz__field-badge wiz__field-badge--ok">
                  <Icon name="check" size={10} />
                  v{getInstalledVersion("node")} installed
                </span>
              {/if}
            </span>
            <select bind:value={nodeVersion}>
              <option value="18">Node.js 18 LTS</option>
              <option value="20">Node.js 20 LTS</option>
              <option value="22">Node.js 22 LTS</option>
            </select>
          </label>
        {/if}

        <label class="wiz__field">
          <span class="wiz__field-label">Projects Directory</span>
          <input type="text" bind:value={projectRoot} placeholder="/home/user/projects" />
          <span class="wiz__field-hint">Where your project folders live</span>
        </label>
      </div>

      <div class="wiz__actions">
        <button class="wiz__btn wiz__btn--ghost" onclick={() => (step = 1)}>Back</button>
        {#if toInstallCount > 0}
          <button class="wiz__btn wiz__btn--primary" onclick={handleInstall}>
            Install {toInstallCount} package{toInstallCount !== 1 ? 's' : ''}
          </button>
        {:else}
          <button class="wiz__btn wiz__btn--primary" onclick={finishSetup}>
            <Icon name="check" size={14} />
            Continue — all set!
          </button>
        {/if}
      </div>

    {:else if step === "password"}
      <div class="wiz__center">
        <div class="wiz__pwd-icon">
          <Icon name="lock" size={24} />
        </div>
        <h2>Authentication Required</h2>
        <p class="wiz__sub">Your system password is needed to install packages. It will be cached for this session only.</p>
      </div>

      <div class="wiz__pwd-form">
        <input
          type="password"
          bind:value={passwordInput}
          placeholder="System password"
          onkeydown={(e) => e.key === "Enter" && submitPassword()}
          autofocus
        />
        {#if passwordError}
          <div class="wiz__pwd-error">
            <Icon name="alert-circle" size={12} />
            {passwordError}
          </div>
        {/if}
      </div>

      <div class="wiz__actions">
        <button class="wiz__btn wiz__btn--ghost" onclick={() => (step = 2)}>Back</button>
        <button class="wiz__btn wiz__btn--primary" onclick={submitPassword} disabled={passwordSaving || !passwordInput}>
          {#if passwordSaving}
            <span class="spinner spinner--sm"></span>
            Verifying...
          {:else}
            Authenticate & Install
          {/if}
        </button>
      </div>

    {:else if step === 3}
      <div class="wiz__center">
        <span class="spinner spinner--lg"></span>
        <h2>Installing</h2>
        <p class="wiz__sub">This may take a few minutes. Please wait...</p>
      </div>

    {:else if step === 4}
      <div class="wiz__center">
        <div class="wiz__check-icon">
          <Icon name="check" size={24} />
        </div>
        <h2>All Set!</h2>
      </div>
      <div class="wiz__results">
        {#each installResults as result}
          <div
            class="wiz__result"
            class:wiz__result--ok={result.includes("installed")}
            class:wiz__result--skip={result.includes("already") || result.includes("skipped")}
            class:wiz__result--fail={result.includes("failed") || result.includes("Error")}
          >
            {#if result.includes("installed")}
              <Icon name="check" size={12} />
            {:else if result.includes("failed")}
              <Icon name="x" size={12} />
            {:else}
              <Icon name="info" size={12} />
            {/if}
            {result}
          </div>
        {/each}
      </div>
      <div class="wiz__actions wiz__actions--center">
        <button class="wiz__btn wiz__btn--primary" onclick={finishSetup}>
          Open MacEnv
          <Icon name="chevron-down" size={14} />
        </button>
      </div>
    {/if}

    {#if error}
      <div class="wiz__error">
        <Icon name="alert-circle" size={14} />
        {error}
      </div>
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
    transition: background var(--transition-normal);
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
    animation: fade-in 200ms ease;
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

  .wiz__check-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-success);
    color: var(--color-text-on-accent);
    display: flex;
    align-items: center;
    justify-content: center;
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
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .wiz__prescan-list {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    max-height: 200px;
    overflow-y: auto;
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
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .wiz__field-badge {
    font-size: 10px;
    font-weight: var(--font-medium);
    text-transform: none;
    letter-spacing: normal;
    display: inline-flex;
    align-items: center;
    gap: 2px;
  }

  .wiz__field-badge--ok {
    color: var(--color-success);
  }

  .wiz__field-hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
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
    gap: var(--space-1);
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

  .wiz__toggle-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
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
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
  }

  /* Password step */
  .wiz__pwd-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--color-accent-subtle);
    color: var(--color-accent);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .wiz__pwd-form {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: center;
  }

  .wiz__pwd-form input {
    width: 100%;
    max-width: 280px;
    text-align: center;
    font-size: var(--text-sm);
    letter-spacing: 0.1em;
  }

  .wiz__pwd-error {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-xs);
    color: var(--color-danger);
    animation: fade-in 150ms ease;
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
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .wiz__result--ok { color: var(--color-success); background: var(--color-success-subtle); }
  .wiz__result--skip { color: var(--color-text-muted); }
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
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
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
    display: flex;
    align-items: center;
    gap: var(--space-2);
    animation: fade-in 150ms ease;
  }
</style>
