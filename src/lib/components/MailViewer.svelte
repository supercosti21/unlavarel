<script>
  import Icon from "./Icon.svelte";

  let { mailpitUrl = "http://localhost:8025" } = $props();
  let available = $state(false);
  let checking = $state(true);
  let messages = $state([]);
  let selectedMessage = $state(null);
  let messageCount = $state(0);
  let unreadCount = $state(0);
  let retryTimer = $state(null);
  let loadingMessage = $state(false);

  $effect(() => {
    checkMailpit();
    retryTimer = setInterval(() => {
      if (available) fetchMessages();
      else checkMailpit();
    }, 10000);
    return () => {
      if (retryTimer) clearInterval(retryTimer);
    };
  });

  async function checkMailpit() {
    checking = true;
    try {
      const response = await fetch(`${mailpitUrl}/api/v1/messages?limit=1`);
      if (response.ok) {
        available = true;
        await fetchMessages();
      } else {
        available = false;
      }
    } catch {
      available = false;
    } finally {
      checking = false;
    }
  }

  async function fetchMessages() {
    try {
      const response = await fetch(`${mailpitUrl}/api/v1/messages?limit=50`);
      if (!response.ok) return;
      const data = await response.json();
      messages = data.messages || [];
      messageCount = data.messages_count || messages.length;
      unreadCount = data.unread || 0;
    } catch {}
  }

  async function openMessage(msg) {
    loadingMessage = true;
    try {
      const response = await fetch(`${mailpitUrl}/api/v1/message/${msg.ID}`);
      if (response.ok) {
        selectedMessage = await response.json();
      }
      // Mark as read
      if (!msg.Read) {
        fetch(`${mailpitUrl}/api/v1/messages`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ IDs: [msg.ID], Read: true }),
        }).then(() => fetchMessages()).catch(() => {});
      }
    } catch {} finally {
      loadingMessage = false;
    }
  }

  async function deleteMessage(id) {
    try {
      await fetch(`${mailpitUrl}/api/v1/messages`, {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ IDs: [id] }),
      });
      if (selectedMessage?.ID === id) selectedMessage = null;
      await fetchMessages();
    } catch {}
  }

  async function deleteAll() {
    try {
      await fetch(`${mailpitUrl}/api/v1/messages`, { method: "DELETE" });
      messages = [];
      selectedMessage = null;
      messageCount = 0;
      unreadCount = 0;
    } catch {}
  }

  function formatDate(dateStr) {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now - d;
    if (diffMs < 60000) return "just now";
    if (diffMs < 3600000) return `${Math.floor(diffMs / 60000)}m ago`;
    if (diffMs < 86400000) return `${Math.floor(diffMs / 3600000)}h ago`;
    return d.toLocaleDateString(undefined, { month: "short", day: "numeric" });
  }

  function senderName(msg) {
    if (msg.From?.Name) return msg.From.Name;
    if (msg.From?.Address) return msg.From.Address;
    return "Unknown";
  }

  function senderEmail(msg) {
    return msg.From?.Address || "";
  }

  function recipientText(msg) {
    if (!msg.To?.length) return "";
    return msg.To.map(t => t.Address || t.Name).join(", ");
  }
</script>

<div class="mail-viewer">
  <div class="mail-viewer__header">
    <div class="mail-viewer__title-row">
      <h2 class="mail-viewer__title">Mail Inbox</h2>
      {#if available && messageCount > 0}
        <span class="badge badge--neutral">{messageCount} messages</span>
        {#if unreadCount > 0}
          <span class="badge badge--info">{unreadCount} unread</span>
        {/if}
      {/if}
    </div>
    <div class="mail-viewer__actions">
      {#if checking}
        <span class="badge badge--neutral"><span class="spinner spinner--sm"></span> Checking...</span>
      {:else if available}
        <span class="badge badge--success"><Icon name="check" size={10} /> Running</span>
      {:else}
        <span class="badge badge--danger"><Icon name="x" size={10} /> Offline</span>
      {/if}
      <button class="btn-icon" onclick={() => { if (available) fetchMessages(); else checkMailpit(); }} aria-label="Refresh">
        <Icon name="refresh" size={16} />
      </button>
      {#if available && messages.length > 0}
        <button class="btn-ghost btn-sm" onclick={deleteAll}>
          <Icon name="trash" size={14} /> Clear all
        </button>
      {/if}
    </div>
  </div>

  {#if available}
    <div class="mail-viewer__body">
      <!-- Message list -->
      <div class="mail-list">
        {#if messages.length === 0}
          <div class="mail-list__empty">
            <Icon name="mail" size={32} />
            <p>No messages yet</p>
            <p class="mail-list__hint">Emails sent by your app will appear here.</p>
          </div>
        {:else}
          {#each messages as msg (msg.ID)}
            <button
              class="mail-item"
              class:mail-item--unread={!msg.Read}
              class:mail-item--selected={selectedMessage?.ID === msg.ID}
              onclick={() => openMessage(msg)}
            >
              <div class="mail-item__top">
                <span class="mail-item__sender" class:mail-item__sender--unread={!msg.Read}>
                  {senderName(msg)}
                </span>
                <span class="mail-item__date">{formatDate(msg.Created)}</span>
              </div>
              <div class="mail-item__subject">{msg.Subject || "(no subject)"}</div>
              {#if msg.Snippet}
                <div class="mail-item__snippet">{msg.Snippet}</div>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      <!-- Message detail -->
      <div class="mail-detail">
        {#if loadingMessage}
          <div class="mail-detail__loading">
            <span class="spinner"></span>
          </div>
        {:else if selectedMessage}
          <div class="mail-detail__header">
            <div class="mail-detail__meta">
              <h3 class="mail-detail__subject">{selectedMessage.Subject || "(no subject)"}</h3>
              <div class="mail-detail__from">
                <strong>{senderName(selectedMessage)}</strong>
                <span class="mail-detail__email">&lt;{senderEmail(selectedMessage)}&gt;</span>
              </div>
              <div class="mail-detail__to">
                To: {recipientText(selectedMessage)}
              </div>
              <div class="mail-detail__date">{new Date(selectedMessage.Created).toLocaleString()}</div>
            </div>
            <button class="btn-icon" onclick={() => deleteMessage(selectedMessage.ID)} aria-label="Delete message">
              <Icon name="trash" size={16} />
            </button>
          </div>
          <div class="mail-detail__body">
            {#if selectedMessage.HTML}
              <iframe
                class="mail-detail__iframe"
                srcdoc={selectedMessage.HTML}
                title="Email content"
                sandbox="allow-same-origin"
              ></iframe>
            {:else}
              <pre class="mail-detail__text">{selectedMessage.Text || ""}</pre>
            {/if}
          </div>
        {:else}
          <div class="mail-detail__empty">
            <Icon name="mail" size={40} />
            <p>Select a message to read</p>
          </div>
        {/if}
      </div>
    </div>
  {:else if !checking}
    <div class="mail-viewer__offline">
      <Icon name="mail" size={48} />
      <h3>Mailpit is not running</h3>
      <p>Start the Mailpit service from the Dashboard to view captured emails.</p>
      <p class="mail-viewer__hint">Auto-retrying every 10 seconds...</p>
    </div>
  {/if}
</div>

<style>
  .mail-viewer {
    display: flex;
    flex-direction: column;
    height: 100%;
    gap: var(--space-4);
  }

  .mail-viewer__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .mail-viewer__title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .mail-viewer__title {
    font-size: var(--text-xl);
    font-weight: var(--font-semibold);
  }

  .mail-viewer__actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .mail-viewer__body {
    flex: 1;
    display: flex;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    overflow: hidden;
    min-height: 0;
  }

  /* Message list */
  .mail-list {
    width: 320px;
    flex-shrink: 0;
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
    background: var(--color-bg-secondary);
  }

  .mail-list__empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    gap: var(--space-2);
    padding: var(--space-6);
    text-align: center;
  }

  .mail-list__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .mail-item {
    width: 100%;
    text-align: left;
    padding: var(--space-3) var(--space-4);
    border: none;
    border-bottom: 1px solid var(--color-border-subtle);
    background: transparent;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
    transition: background var(--transition-fast);
    border-radius: 0;
  }

  .mail-item:hover {
    background: var(--color-bg-hover);
  }

  .mail-item--selected {
    background: var(--color-accent-subtle);
  }

  .mail-item--unread {
    background: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }

  .mail-item__top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .mail-item__sender {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mail-item__sender--unread {
    color: var(--color-text-primary);
    font-weight: var(--font-semibold);
  }

  .mail-item__date {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    flex-shrink: 0;
    margin-left: var(--space-2);
  }

  .mail-item__subject {
    font-size: var(--text-sm);
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mail-item__snippet {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Message detail */
  .mail-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--color-bg-primary);
  }

  .mail-detail__loading,
  .mail-detail__empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    gap: var(--space-3);
  }

  .mail-detail__header {
    padding: var(--space-4);
    border-bottom: 1px solid var(--color-border-subtle);
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-shrink: 0;
  }

  .mail-detail__meta {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .mail-detail__subject {
    font-size: var(--text-base);
    font-weight: var(--font-semibold);
    color: var(--color-text-primary);
  }

  .mail-detail__from {
    font-size: var(--text-sm);
    color: var(--color-text-primary);
  }

  .mail-detail__email {
    color: var(--color-text-muted);
    font-size: var(--text-xs);
  }

  .mail-detail__to {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  .mail-detail__date {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .mail-detail__body {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }

  .mail-detail__iframe {
    width: 100%;
    height: 100%;
    border: none;
    background: #fff;
  }

  .mail-detail__text {
    padding: var(--space-4);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--color-text-primary);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .mail-viewer__offline {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--color-text-muted);
    gap: var(--space-3);
    text-align: center;
  }

  .mail-viewer__offline h3 {
    font-size: var(--text-base);
    color: var(--color-text-secondary);
  }

  .mail-viewer__offline p {
    font-size: var(--text-sm);
    max-width: 360px;
  }

  .mail-viewer__hint {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }
</style>
