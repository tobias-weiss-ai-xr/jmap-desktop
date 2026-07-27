<script lang="ts">
  import { session, selectedMailboxId, mailboxes, topLevelMailboxes, syncStatus } from '$lib/jmap/stores.js';

  function unreadCount(id: string): number {
    return $mailboxes.find((m) => m.id === id)?.unreadEmails ?? 0;
  }

  const roleOrder: Record<string, number> = {
    inbox: 0, starred: 1, flagged: 1, sent: 2, drafts: 3, archive: 4, junk: 5, trash: 6,
  };

  // Search
  let searchQuery = $state('');
  let searchOpen = $state(false);

  function handleSearch() {
    if (searchQuery.trim()) {
      // Navigate to search results
      const event = new CustomEvent('jmap-search', { detail: searchQuery });
      window.dispatchEvent(event);
    }
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h2 class="sidebar-title">
      {#if $session}
        {$session.accounts[$session.primaryAccounts?.['urn:ietf:params:jmap:mail']]?.name ?? 'JMAP Mail'}
      {:else}
        JMAP Desktop
      {/if}
    </h2>
    <div class="sync-badge" class:syncing={$syncStatus === 'syncing'} title="Sync status: {$syncStatus}">
      {#if $syncStatus === 'syncing'}
        ⟳
      {:else if $syncStatus === 'synced' || $syncStatus === 'push-connected'}
        ✓
      {:else}
        ○
      {/if}
    </div>
  </div>

  <div class="search-bar">
    <input
      type="text"
      bind:value={searchQuery}
      placeholder="Search emails…"
      onkeydown={(e) => e.key === 'Enter' && handleSearch()}
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
        >
          <span class="mailbox-icon" data-role={mailbox.role ?? 'folder'}>
            {mailbox.role === 'inbox' ? '📥' :
             mailbox.role === 'sent' ? '📤' :
             mailbox.role === 'drafts' ? '📝' :
             mailbox.role === 'trash' ? '🗑️' :
             mailbox.role === 'junk' ? '⚠️' :
             mailbox.role === 'flagged' ? '⭐' :
             mailbox.role === 'archive' ? '📦' : '📁'}
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
    <a href="/settings" class="settings-link">⚙ Settings</a>
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
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .sidebar-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--fg-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .sync-badge {
    font-size: 14px;
    color: var(--fg-muted);
    flex-shrink: 0;
  }

  .sync-badge.syncing {
    color: var(--warning);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .search-bar {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
  }

  .search-bar input {
    width: 100%;
    padding: 6px 10px;
    font-size: 13px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
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
    font-weight: 500;
    background: var(--accent);
    color: var(--bg-primary);
    border: none;
    border-radius: 4px;
    cursor: pointer;
    text-align: center;
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
    padding: 6px 12px;
    gap: 8px;
    background: transparent;
    border: none;
    color: var(--fg-secondary);
    cursor: pointer;
    text-align: left;
    font-size: 13px;
    font-family: inherit;
  }

  .mailbox-item:hover { background: var(--bg-hover); }
  .mailbox-item.selected { background: var(--bg-selected); color: var(--fg-primary); }

  .mailbox-icon { font-size: 16px; flex-shrink: 0; }
  .mailbox-name { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

  .unread-badge {
    font-size: 11px;
    font-weight: 600;
    color: var(--mail-unread);
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 10px;
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
    padding: 8px 16px;
    border-top: 1px solid var(--border);
  }

  .settings-link { font-size: 12px; color: var(--fg-muted); }
</style>
