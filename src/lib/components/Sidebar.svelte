<script lang="ts">
  import { session, selectedMailboxId, mailboxes, topLevelMailboxes, syncStatus } from '$lib/jmap/stores.js';
  import { disconnect } from '$lib/jmap/actions.js';

  function unreadCount(id: string): number {
    return $mailboxes.find((m) => m.id === id)?.unreadEmails ?? 0;
  }

  function totalUnread(): number {
    return $mailboxes.reduce((sum, m) => sum + m.unreadEmails, 0);
  }

  const roleOrder: Record<string, number> = {
    inbox: 0, starred: 1, flagged: 1, sent: 2, drafts: 3, archive: 4, junk: 5, trash: 6,
  };

  // Search
  let searchQuery = $state('');
  let searchActive = $state(false);

  function handleSearch() {
    if (searchQuery.trim()) {
      searchActive = true;
      const event = new CustomEvent('jmap-search', { detail: searchQuery });
      window.dispatchEvent(event);
    }
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSearch();
    }
    if (e.key === 'Escape') {
      searchQuery = '';
      searchActive = false;
      (e.target as HTMLInputElement).blur();
    }
  }

  function mailboxIcon(role: string | null | undefined): string {
    switch (role) {
      case 'inbox': return '📥';
      case 'sent': return '📤';
      case 'drafts': return '📝';
      case 'trash': return '🗑️';
      case 'junk': return '⚠️';
      case 'flagged': return '⭐';
      case 'archive': return '📦';
      default: return '📁';
    }
  }

  function handleDisconnect() {
    disconnect();
    // Clear saved credentials so we don't auto-reconnect
    localStorage.removeItem('jmap-settings');
  }
</script>

<aside class="sidebar" role="navigation" aria-label="Mail folders">
  <div class="sidebar-header">
    <h2 class="sidebar-title" title={$session?.username ?? ''}>
      {$session?.username ?? 'JMAP Desktop'}
    </h2>
    <div class="header-actions">
      <button
        class="sync-badge"
        class:syncing={$syncStatus === 'syncing'}
        title="Sync status: {$syncStatus}"
        aria-label="Sync: {$syncStatus}"
        disabled
      >
        {#if $syncStatus === 'syncing'}
          ⟳
        {:else if $syncStatus === 'synced' || $syncStatus === 'push-connected'}
          ✓
        {:else}
          ○
        {/if}
      </button>
      <button
        class="disconnect-btn"
        onclick={handleDisconnect}
        title="Disconnect"
        aria-label="Disconnect from server"
      >
        ⏻
      </button>
    </div>
  </div>

  <div class="search-bar">
    <input
      type="search"
      bind:value={searchQuery}
      placeholder="Search emails…"
      onkeydown={handleSearchKeydown}
      onfocus={() => searchActive = true}
      aria-label="Search emails"
    />
  </div>

  <nav class="mailbox-list">
    <button class="compose-btn" onclick={() => window.dispatchEvent(new CustomEvent('jmap-compose'))}>
      ✉ Compose
    </button>

    {#if $topLevelMailboxes.length > 0}
      {#each $topLevelMailboxes.sort((a, b) =>
        (roleOrder[a.role ?? 'zzz'] ?? 99) - (roleOrder[b.role ?? 'zzz'] ?? 99)
      ) as mailbox (mailbox.id)}
        {@const count = unreadCount(mailbox.id)}
        <button
          class="mailbox-item"
          class:selected={$selectedMailboxId === mailbox.id}
          onclick={() => (selectedMailboxId.set(mailbox.id))}
          aria-current={$selectedMailboxId === mailbox.id ? 'true' : undefined}
        >
          <span class="mailbox-icon" data-role={mailbox.role ?? 'folder'}>
            {mailboxIcon(mailbox.role)}
          </span>
          <span class="mailbox-name">{mailbox.name}</span>
          {#if count > 0}
            <span class="unread-badge">{count > 999 ? '999+' : count}</span>
          {/if}
        </button>
      {/each}
    {:else if $session}
      <p class="muted sidebar-empty">No mailboxes found</p>
    {:else}
      <div class="sidebar-empty">
        <p class="muted">Not connected</p>
        <a href="/settings" class="connect-link">Connect →</a>
      </div>
    {/if}
  </nav>

  <div class="sidebar-footer">
    {#if totalUnread() > 0}
      <span class="unread-summary">{totalUnread()} unread</span>
    {:else}
      <span class="unread-summary">All read</span>
    {/if}
    <a href="/settings" class="settings-link" title="Settings">⚙</a>
  </div>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    min-width: var(--sidebar-width);
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
  }

  .sidebar-header {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sidebar-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--fg-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .sync-badge {
    font-size: 14px;
    color: var(--fg-muted);
    background: transparent;
    border: none;
    cursor: default;
    padding: 2px;
    line-height: 1;
  }

  .sync-badge.syncing {
    color: var(--warning);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .disconnect-btn {
    font-size: 15px;
    background: transparent;
    border: none;
    color: var(--fg-muted);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 3px;
    line-height: 1;
    opacity: 0.6;
    transition: opacity 0.15s, color 0.15s;
  }

  .disconnect-btn:hover {
    opacity: 1;
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 10%, transparent);
  }

  .search-bar {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }

  .search-bar input {
    width: 100%;
    padding: 7px 10px;
    font-size: 13px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
    transition: border-color 0.15s;
  }

  .search-bar input:focus {
    border-color: var(--accent);
  }

  .compose-btn {
    display: block;
    width: calc(100% - 24px);
    margin: 8px 12px;
    padding: 8px;
    font-size: 13px;
    font-family: inherit;
    font-weight: 600;
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    text-align: center;
    transition: background 0.15s;
  }

  .compose-btn:hover {
    background: var(--accent-hover);
  }

  .mailbox-list {
    flex: 1;
    overflow-y: auto;
  }

  .mailbox-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 7px 12px;
    gap: 8px;
    background: transparent;
    border: none;
    color: var(--fg-secondary);
    cursor: pointer;
    text-align: left;
    font-size: 13px;
    font-family: inherit;
    border-radius: 0;
    transition: background 0.1s;
  }

  .mailbox-item:hover { background: var(--bg-hover); }
  .mailbox-item.selected { background: var(--bg-selected); color: var(--fg-primary); font-weight: 500; }

  .mailbox-icon { font-size: 16px; flex-shrink: 0; width: 20px; text-align: center; }
  .mailbox-name { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .unread-badge {
    font-size: 11px;
    font-weight: 600;
    color: var(--mail-unread);
    background: color-mix(in srgb, var(--mail-unread) 15%, transparent);
    padding: 1px 6px;
    border-radius: 10px;
    min-width: 22px;
    text-align: center;
  }

  .sidebar-empty {
    padding: 16px;
    text-align: center;
    font-size: 13px;
  }

  .connect-link {
    font-size: 13px;
    color: var(--accent);
    display: block;
    margin-top: 8px;
  }

  .sidebar-footer {
    padding: 8px 12px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .unread-summary {
    font-size: 11px;
    color: var(--fg-muted);
  }

  .settings-link {
    font-size: 15px;
    color: var(--fg-muted);
    transition: color 0.15s;
  }

  .settings-link:hover {
    color: var(--fg-primary);
    text-decoration: none;
  }
</style>
