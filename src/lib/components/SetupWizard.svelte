<script>
  import { invoke } from "@tauri-apps/api/core";

  let { onComplete } = $props();

  let step = $state(1);
  let setupState = $state(null);
  let loading = $state(true);
  let installing = $state(false);
  let installResults = $state([]);
  let error = $state(null);

  // Stack selection
  let phpVersion = $state("8.3");
  let database = $state("mysql");
  let dbVersion = $state("8.4");
  let extras = $state(["redis", "mailpit", "node"]);
  let nodeVersion = $state("22");

  $effect(() => {
    checkSetup();
  });

  async function checkSetup() {
    loading = true;
    try {
      setupState = await invoke("check_setup");
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
      await checkSetup();
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

  async function installStack() {
    installing = true;
    error = null;
    step = 3;
    try {
      installResults = await invoke("install_stack", {
        selection: {
          php_version: phpVersion,
          database,
          database_version: dbVersion,
          extras,
          node_version: extras.includes("node") ? nodeVersion : null,
        },
      });
      step = 4;
    } catch (e) {
      error = String(e);
    } finally {
      installing = false;
    }
  }

  async function finishSetup() {
    await invoke("mark_setup_complete");
    onComplete();
  }
</script>

<div class="wizard">
  <div class="wizard__header">
    <h1 class="wizard__title">Welcome to MacEnv</h1>
    <div class="wizard__steps">
      {#each [1, 2, 3, 4] as s}
        <span
          class="wizard__step"
          class:wizard__step--active={step === s}
          class:wizard__step--done={step > s}
        >{s}</span>
      {/each}
    </div>
  </div>

  <div class="wizard__content">
    {#if loading}
      <p class="wizard__loading">Detecting your system...</p>

    {:else if step === 1}
      <div class="wizard__section">
        <h2>System Check</h2>
        {#if setupState}
          <div class="wizard__info">
            <div class="wizard__info-row">
              <span>Platform</span>
              <span class="badge badge--neutral">{setupState.platform.os}</span>
            </div>
            <div class="wizard__info-row">
              <span>Architecture</span>
              <span class="badge badge--neutral">{setupState.platform.arch}</span>
            </div>
            {#if setupState.platform.linux_distro}
              <div class="wizard__info-row">
                <span>Distribution</span>
                <span class="badge badge--neutral">{JSON.stringify(setupState.platform.linux_distro)}</span>
              </div>
            {/if}
            <div class="wizard__info-row">
              <span>Package Manager</span>
              <span class="badge" class:badge--success={setupState.package_manager_available} class:badge--danger={!setupState.package_manager_available}>
                {setupState.package_manager_name} {setupState.package_manager_available ? "(ready)" : "(not found)"}
              </span>
            </div>
          </div>

          {#if setupState.package_manager_available}
            <button class="btn-primary" onclick={() => (step = 2)}>Continue</button>
          {:else}
            <p>Your package manager needs to be installed first.</p>
            <button class="btn-primary" onclick={bootstrapPM} disabled={installing}>
              {installing ? "Installing..." : `Install ${setupState.package_manager_name}`}
            </button>
          {/if}
        {/if}
      </div>

    {:else if step === 2}
      <div class="wizard__section">
        <h2>Choose Your Stack</h2>

        <div class="wizard__field">
          <label>PHP Version</label>
          <select bind:value={phpVersion}>
            <option value="8.1">PHP 8.1</option>
            <option value="8.2">PHP 8.2</option>
            <option value="8.3">PHP 8.3</option>
            <option value="8.4">PHP 8.4</option>
          </select>
        </div>

        <div class="wizard__field">
          <label>Database</label>
          <select bind:value={database}>
            <option value="mysql">MySQL</option>
            <option value="postgresql">PostgreSQL</option>
          </select>
        </div>

        <div class="wizard__field">
          <label>Database Version</label>
          <select bind:value={dbVersion}>
            {#if database === "mysql"}
              <option value="8.0">MySQL 8.0</option>
              <option value="8.4">MySQL 8.4</option>
            {:else}
              <option value="15">PostgreSQL 15</option>
              <option value="16">PostgreSQL 16</option>
              <option value="17">PostgreSQL 17</option>
            {/if}
          </select>
        </div>

        <div class="wizard__field">
          <label>Extras</label>
          <div class="wizard__checkboxes">
            {#each [
              { id: "redis", label: "Redis" },
              { id: "memcached", label: "Memcached" },
              { id: "mailpit", label: "Mailpit (email testing)" },
              { id: "node", label: "Node.js" },
            ] as opt}
              <label class="wizard__checkbox">
                <input
                  type="checkbox"
                  checked={extras.includes(opt.id)}
                  onchange={() => toggleExtra(opt.id)}
                />
                {opt.label}
              </label>
            {/each}
          </div>
        </div>

        {#if extras.includes("node")}
          <div class="wizard__field">
            <label>Node.js Version</label>
            <select bind:value={nodeVersion}>
              <option value="18">Node.js 18 LTS</option>
              <option value="20">Node.js 20 LTS</option>
              <option value="22">Node.js 22 LTS</option>
            </select>
          </div>
        {/if}

        <div class="wizard__actions">
          <button class="btn-ghost" onclick={() => (step = 1)}>Back</button>
          <button class="btn-primary" onclick={installStack}>Install Stack</button>
        </div>
      </div>

    {:else if step === 3}
      <div class="wizard__section">
        <h2>Installing...</h2>
        <div class="wizard__progress">
          <div class="wizard__spinner"></div>
          <p>Installing packages via {setupState?.package_manager_name}...</p>
          <p class="wizard__hint">This may take a few minutes.</p>
        </div>
        {#if installResults.length > 0}
          <ul class="wizard__results">
            {#each installResults as result}
              <li class:wizard__result--ok={result.includes("installed")}
                  class:wizard__result--fail={result.includes("failed")}>{result}</li>
            {/each}
          </ul>
        {/if}
      </div>

    {:else if step === 4}
      <div class="wizard__section">
        <h2>Setup Complete</h2>
        <ul class="wizard__results">
          {#each installResults as result}
            <li class:wizard__result--ok={result.includes("installed")}
                class:wizard__result--fail={result.includes("failed")}>{result}</li>
          {/each}
        </ul>
        <button class="btn-primary" onclick={finishSetup}>Go to Dashboard</button>
      </div>
    {/if}

    {#if error}
      <div class="wizard__error">{error}</div>
    {/if}
  </div>
</div>

<style>
  .wizard {
    max-width: 560px;
    margin: 0 auto;
    padding: var(--space-8) var(--space-6);
  }

  .wizard__header {
    text-align: center;
    margin-bottom: var(--space-8);
  }

  .wizard__title {
    font-size: var(--text-2xl);
    font-weight: var(--font-semibold);
    margin-bottom: var(--space-4);
  }

  .wizard__steps {
    display: flex;
    justify-content: center;
    gap: var(--space-2);
  }

  .wizard__step {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xs);
    font-weight: var(--font-semibold);
    background: var(--color-bg-tertiary);
    color: var(--color-text-muted);
  }

  .wizard__step--active {
    background: var(--color-accent);
    color: var(--color-text-on-accent);
  }

  .wizard__step--done {
    background: var(--color-success);
    color: var(--color-text-on-accent);
  }

  .wizard__content {
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    padding: var(--space-6);
  }

  .wizard__section {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .wizard__section h2 {
    font-size: var(--text-lg);
    font-weight: var(--font-semibold);
  }

  .wizard__loading {
    text-align: center;
    color: var(--color-text-muted);
    padding: var(--space-8);
  }

  .wizard__info {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .wizard__info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .wizard__field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .wizard__field label {
    font-size: var(--text-sm);
    font-weight: var(--font-medium);
    color: var(--color-text-secondary);
  }

  .wizard__checkboxes {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .wizard__checkbox {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    cursor: pointer;
    color: var(--color-text-primary);
  }

  .wizard__checkbox input {
    accent-color: var(--color-accent);
  }

  .wizard__actions {
    display: flex;
    justify-content: space-between;
    margin-top: var(--space-4);
  }

  .wizard__progress {
    text-align: center;
    padding: var(--space-6);
  }

  .wizard__spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin: 0 auto var(--space-4);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .wizard__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    margin-top: var(--space-2);
  }

  .wizard__results {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: var(--text-sm);
    font-family: var(--font-mono);
  }

  .wizard__result--ok {
    color: var(--color-success);
  }

  .wizard__result--fail {
    color: var(--color-danger);
  }

  .wizard__error {
    margin-top: var(--space-4);
    padding: var(--space-3);
    background: var(--color-danger-subtle);
    color: var(--color-danger);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
  }
</style>
