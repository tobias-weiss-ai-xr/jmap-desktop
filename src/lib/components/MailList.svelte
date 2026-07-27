<script lang="ts">
  import { emailIds, emails, selectedEmailId, isLoadingEmails } from '$lib/jmap/stores.js';
  import { toggleFlag } from '$lib/jmap/actions.js';

  let listEl: HTMLDivElement | undefined = $state();

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

  function handleSelectKeydown(e: KeyboardEvent, id: string) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleSelect(id);
    }
  }

  function handleContextMenu(e: MouseEvent, id: string, flagged: boolean) {
    e.preventDefault();
    toggleFlag(id, flagged);
  }

  // Keyboard navigation: j/k to move up/down, Enter to select
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
    } else if (e.key === 'x') {
      // Flag/unflag current
      const id = $selectedEmailId;
      const email = id ? $emails.get(id) : null;
      if (id && email) toggleFlag(id, isFlagged(email));
    } else if (e.key === 'Delete' || e.key === '#') {
      // Delete current (dispatches custom event for MailView to handle)
      if ($selectedEmailId) {
        window.dispatchEvent(new CustomEvent('jmap-delete-current'));
      }
    }
  }

  function scrollToItem(id: string) {
    const el = document.querySelector(`[data-email-id="${id}"]`);
    el?.scrollIntoView({ block: 'nearest' });
  }
</script>

<div class="mail-list" role="list" aria-label="Email list">
  {#if $isLoadingEmails}
    <div class="loading">Loading…</div>
  {:else if $emailIds.length === 0}
    <div class="empty">No emails</div>
  {:else}
    <div class="mail-items" role="listbox" bind:this={listEl} onkeydown={handleListKeydown} tabindex="0">
      {#each $emailIds as id (id)}
        {@const email = $emails.get(id)}
        {#if email}
          {@const flagged = isFlagged(email)}
          {@const unseen = isUnseen(email)}
          <!-- svelte-ignore a11y_role_supports_aria_props -->
          <div
            class="mail-item"
            class:selected={$selectedEmailId === id}
            class:unread={unseen}
            role="button"
            tabindex="0"
            aria-selected={$selectedEmailId === id}
            data-email-id={id}
            onclick={() => handleSelect(id)}
            onkeydown={(e) => handleSelectKeydown(e, id)}
            oncontextmenu={(e) => handleContextMenu(e, id, flagged)}
          >
            <div class="mail-item-header">
              <button
                class="mail-flag"
                class:flagged={flagged}
                onclick={(e) => { e.stopPropagation(); toggleFlag(id, flagged); }}
                aria-label={flagged ? 'Remove flag' : 'Flag email'}
              >
                {flagged ? '⭐' : '☆'}
              </button>
              <span class="mail-sender">{formatSender(email)}</span>
              <span class="mail-date">{formatRelative(email.receivedAt)}</span>
            </div>
            <div class="mail-subject">{email.subject || '(no subject)'}</div>
            <div class="mail-preview muted">{truncate(email.preview)}</div>
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

  .mail-items { flex: 1; overflow-y: auto; }

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
    font-size: inherit;
    position: relative;
  }

  .mail-item:hover { background: var(--bg-hover); }
  .mail-item.selected { background: var(--bg-selected); color: var(--fg-primary); }
  .mail-item.unread { border-left: 3px solid var(--accent); padding-left: 11px; }
  .mail-item.unread .mail-sender,
  .mail-item.unread .mail-subject { font-weight: 600; color: var(--fg-primary); }
  .mail-item:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }

  .mail-item-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .mail-flag {
    font-size: 14px;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0.5;
    margin-right: 4px;
    padding: 0;
    line-height: 1;
  }

  .mail-flag.flagged { opacity: 1; }
  .mail-flag:hover { opacity: 1; }
  .mail-flag:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; border-radius: 2px; }

  .mail-sender {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .mail-date { font-size: 11px; color: var(--fg-muted); white-space: nowrap; margin-left: 8px; }
  .mail-subject { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mail-preview { font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .mail-attachment { position: absolute; right: 14px; bottom: 8px; font-size: 12px; }

  .loading, .empty { padding: 32px; text-align: center; color: var(--fg-muted); font-size: 13px; }
</style>
