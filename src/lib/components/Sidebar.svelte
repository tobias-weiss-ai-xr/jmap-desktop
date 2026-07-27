<script lang="ts">
  import { session, selectedMailboxId, mailboxes, topLevelMailboxes } from '$lib/jmap/stores.js';

  // Unread count for a mailbox
  function unreadCount(id: string): number {
    return $mailboxes.find((m) => m.id === id)?.unreadEmails ?? 0;
  }

  // Filter mailboxes by role
  const roleOrder: Record<string, number> = {
    inbox: 0,
    flagged: 1,
    sent: 2,
    drafts: 3,
    archive: 4,
    junk: 5,
    trash: 6,
  };
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <h2 class="sidebar-title">
      {#if $session}
        {$session.accounts[$session.primaryAccounts['urn:ietf:params:jmap:mail']]?.name ?? 'JMAP Mail'}
      {:else}
        JMAP Desktop
      {/if}
    </h2>
  </div>

  <nav class="mailbox-list">
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
            <span class="unread-badge">{count}</span>
          {/if}
        </button>
      {/each}
    {:else if $session}
      <p class="muted sidebar-empty">No mailboxes found</p>
    {:else}
      <p class="muted sidebar-empty">Not connected</p>
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
  }

  .sidebar-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--fg-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mailbox-list {
    flex: 1;
    overflow-y: auto;
    padding: 4px 0;
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

  .mailbox-item:hover {
    background: var(--bg-hover);
  }

  .mailbox-item.selected {
    background: var(--bg-selected);
    color: var(--fg-primary);
  }

  .mailbox-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .mailbox-name {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

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

  .sidebar-footer {
    padding: 8px 16px;
    border-top: 1px solid var(--border);
  }

  .settings-link {
    font-size: 12px;
    color: var(--fg-muted);
  }
</style>
