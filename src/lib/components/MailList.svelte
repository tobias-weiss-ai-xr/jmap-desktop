<script lang="ts">
  import { emailIds, emails, selectedEmailId, isLoadingEmails } from '$lib/jmap/stores.js';

  // Date formatting
  function formatRelative(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffDays === 0) {
      return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return date.toLocaleDateString(undefined, { weekday: 'short' });
    } else {
      return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
    }
  }

  // Format sender name
  function formatSender(email: any): string {
    const from = email.from?.[0];
    if (!from) return '(no sender)';
    return from.name || from.email;
  }

  // Truncate preview
  function truncate(str: string, len: number = 80): string {
    return str.length > len ? str.slice(0, len) + '…' : str;
  }
</script>

<div class="mail-list">
  {#if $isLoadingEmails}
    <div class="loading">Loading…</div>
  {:else if $emailIds.length === 0}
    <div class="empty">No emails</div>
  {:else}
    <div class="mail-items">
      {#each $emailIds as id (id)}
        {@const email = $emails.get(id)}
        {#if email}
          <button
            class="mail-item"
            class:selected={$selectedEmailId === id}
            class:unread={!email.keywords.$seen}
            onclick={() => (selectedEmailId.set(id))}
          >
            <div class="mail-item-header">
              <span class="mail-sender">{formatSender(email)}</span>
              <span class="mail-date">{formatRelative(email.receivedAt)}</span>
            </div>
            <div class="mail-subject">{email.subject || '(no subject)'}</div>
            <div class="mail-preview muted">{truncate(email.preview)}</div>
            {#if email.hasAttachment}
              <span class="mail-attachment">📎</span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .mail-list {
    width: var(--mail-list-width);
    min-width: var(--mail-list-width);
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border);
    background: var(--bg-primary);
  }

  .mail-items {
    flex: 1;
    overflow-y: auto;
  }

  .mail-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 10px 14px;
    gap: 3px;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    color: var(--fg-secondary);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    position: relative;
  }

  .mail-item:hover {
    background: var(--bg-hover);
  }

  .mail-item.selected {
    background: var(--bg-selected);
    color: var(--fg-primary);
  }

  .mail-item.unread {
    border-left: 3px solid var(--accent);
    padding-left: 11px;
  }

  .mail-item.unread .mail-sender,
  .mail-item.unread .mail-subject {
    font-weight: 600;
    color: var(--fg-primary);
  }

  .mail-item-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .mail-sender {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    margin-right: 8px;
  }

  .mail-date {
    font-size: 11px;
    color: var(--fg-muted);
    white-space: nowrap;
  }

  .mail-subject {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mail-preview {
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .mail-attachment {
    position: absolute;
    right: 14px;
    bottom: 8px;
    font-size: 12px;
  }

  .loading,
  .empty {
    padding: 32px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 13px;
  }
</style>
