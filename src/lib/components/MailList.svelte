<script lang="ts">
  import { emailIds, emails, selectedEmailId, selectedMailboxId, isLoadingEmails, mailboxes } from '$lib/jmap/stores.js';
  import { toggleFlag, deleteEmail } from '$lib/jmap/actions.js';

  let listEl: HTMLDivElement | undefined = $state();

  function currentMailboxName(): string {
    const id = $selectedMailboxId;
    if (!id) return '';
    return $mailboxes.find((m) => m.id === id)?.name ?? '';
  }

  // Auto-focus the list for keyboard navigation when emails load
  $effect(() => {
    if ($emailIds.length > 0 && listEl) {
      listEl.focus();
    }
  });

  function formatRelative(dateStr: string): string {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);
    if (diffDays === 0) return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return date.toLocaleDateString(undefined, { weekday: 'short' });
    return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  }

  function formatSender(email: any): string {
    const from = email.from?.[0];
    if (!from) return '(no sender)';
    return from.name || from.email;
  }

  function truncate(str: string | undefined, len: number = 80): string {
    if (!str) return '';
    return str.length > len ? str.slice(0, len) + '…' : str;
  }

  function isFlagged(email: any): boolean {
    return !!(email.keywords && (email.keywords.$flagged || email.keywords.$starred));
  }

  function isUnseen(email: any): boolean {
    return !(email.keywords && email.keywords.$seen);
  }

  function handleSelect(id: string) {
    selectedEmailId.set(id);
  }

  function handleListKeydown(e: KeyboardEvent) {
    const ids = $emailIds;
    if (ids.length === 0) return;
    const currentIdx = ids.indexOf($selectedEmailId ?? '');

    if (e.key === 'j' || e.key === 'ArrowDown') {
      e.preventDefault();
      const next = currentIdx < ids.length - 1 ? currentIdx + 1 : 0;
      selectedEmailId.set(ids[next]);
      scrollToItem(ids[next]);
    } else if (e.key === 'k' || e.key === 'ArrowUp') {
      e.preventDefault();
      const prev = currentIdx > 0 ? currentIdx - 1 : ids.length - 1;
      selectedEmailId.set(ids[prev]);
      scrollToItem(ids[prev]);
    } else if (e.key === 'Enter') {
      // Already selected via keyboard nav, just ensure focus stays
      if ($selectedEmailId) scrollToItem($selectedEmailId);
    } else if (e.key === 'r') {
      // Reply shortcut
      window.dispatchEvent(new CustomEvent('jmap-reply'));
    } else if (e.key === 'c') {
      // Compose shortcut
      window.dispatchEvent(new CustomEvent('jmap-compose'));
    } else if (e.key === 'x') {
      // Flag/unflag current
      const id = $selectedEmailId;
      const email = id ? $emails.get(id) : null;
      if (id && email) toggleFlag(id, isFlagged(email));
    } else if (e.key === 'Delete' || e.key === '#') {
      if ($selectedEmailId) {
        window.dispatchEvent(new CustomEvent('jmap-delete-current'));
      }
    }
  }

  function scrollToItem(id: string) {
    const el = document.querySelector(`[data-email-id="${id}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }

  // Component: Skeleton row for loading state
</script>

<div class="mail-list" role="list" aria-label="Email list">
  <!-- Mailbox label -->
  {#if currentMailboxName()}
    <div class="mailbox-label">
      <span>{currentMailboxName()}</span>
      {#if $emailIds.length > 0}
        <span class="count-badge">{$emailIds.length}</span>
      {/if}
    </div>
  {/if}

  {#if $isLoadingEmails}
    <div class="loading" role="status" aria-label="Loading emails">
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
      <div class="skeleton-header"><div class="skeleton-line short"></div><div class="skeleton-line tiny"></div></div>
      <div class="skeleton-body"><div class="skeleton-line"></div></div>
    </div>
  {:else if $emailIds.length === 0 && !$selectedMailboxId}
    <div class="empty">
      <div class="empty-icon">←</div>
      <p>Select a mailbox</p>
    </div>
  {:else if $emailIds.length === 0}
    <div class="empty">
      <div class="empty-icon">📭</div>
      <p>{currentMailboxName() ? 'No emails' : 'Empty'}</p>
    </div>
  {:else}
    <div class="mail-items" role="listbox" bind:this={listEl} onkeydown={handleListKeydown} tabindex="0">
      {#each $emailIds as id (id)}
        {@const email = $emails.get(id)}
        {#if email}
          {@const flagged = isFlagged(email)}
          {@const unseen = isUnseen(email)}
          <div
            class="mail-item"
            class:selected={$selectedEmailId === id}
            class:unread={unseen}
            role="option"
            tabindex="-1"
            aria-selected={$selectedEmailId === id}
            data-email-id={id}
            onclick={() => handleSelect(id)}
            oncontextmenu={(e) => { e.preventDefault(); toggleFlag(id, flagged); }}
          >
            <button
              class="mail-flag"
              class:flagged={flagged}
              onclick={(e) => { e.stopPropagation(); toggleFlag(id, flagged); }}
              aria-label={flagged ? 'Remove flag' : 'Flag email'}
            >
              {flagged ? '★' : '☆'}
            </button>
            <div class="mail-content">
              <div class="mail-item-header">
                <span class="mail-sender">{formatSender(email)}</span>
                <span class="mail-date">{formatRelative(email.receivedAt)}</span>
              </div>
              <div class="mail-subject">{email.subject || '(no subject)'}</div>
              <div class="mail-preview muted">{truncate(email.preview)}</div>
            </div>
            {#if email.hasAttachment}
              <span class="mail-attachment">📎</span>
            {/if}
          </div>
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

  /* Mailbox label */
  .mailbox-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 14px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--fg-muted);
    border-bottom: 1px solid var(--border);
  }

  .count-badge {
    font-size: 10px;
    font-weight: 600;
    color: var(--fg-secondary);
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 8px;
  }

  /* Skeleton loading */
  .loading { padding: 0; }

  .skeleton-header {
    display: flex;
    justify-content: space-between;
    padding: 10px 14px 2px;
  }

  .skeleton-body {
    padding: 0 14px 4px;
  }

  .skeleton-line {
    height: 12px;
    border-radius: 3px;
    background: var(--bg-hover);
    animation: shimmer 1.5s infinite;
  }

  .skeleton-line.short {
    width: 40%;
  }

  .skeleton-line.tiny {
    width: 20%;
    height: 10px;
  }

  @keyframes shimmer {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.8; }
  }

  /* Items */
  .mail-items { flex: 1; overflow-y: auto; outline: none; }

  .mail-item {
    display: flex;
    align-items: flex-start;
    padding: 10px 14px;
    background: transparent;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
    color: var(--fg-secondary);
    cursor: pointer;
    font-family: inherit;
    font-size: inherit;
    position: relative;
    transition: background 0.08s;
  }

  .mail-item:hover { background: var(--bg-hover); }
  .mail-item.selected { background: var(--bg-selected); color: var(--fg-primary); }
  .mail-item.unread { border-left: 3px solid var(--accent); padding-left: 11px; }
  .mail-item.unread .mail-sender,
  .mail-item.unread .mail-subject { font-weight: 600; color: var(--fg-primary); }
  .mail-item:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }

  .mail-flag {
    font-size: 13px;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0.3;
    padding: 0 4px 0 0;
    line-height: 1;
    flex-shrink: 0;
    transition: opacity 0.15s;
  }

  .mail-flag.flagged { opacity: 1; }
  .mail-flag:hover { opacity: 1; }
  .mail-flag:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; border-radius: 2px; }

  .mail-content {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
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
    min-width: 0;
  }

  .mail-date { font-size: 11px; color: var(--fg-muted); white-space: nowrap; margin-left: 8px; flex-shrink: 0; }
  .mail-subject { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mail-preview { font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mail-attachment { position: absolute; right: 14px; bottom: 8px; font-size: 12px; opacity: 0.6; }

  .loading, .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 48px 32px;
    color: var(--fg-muted);
    font-size: 13px;
  }

  .empty-icon {
    font-size: 32px;
    opacity: 0.3;
  }
</style>
