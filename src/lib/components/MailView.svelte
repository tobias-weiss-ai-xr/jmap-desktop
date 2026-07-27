<script lang="ts">
  import { currentEmail, selectedEmailId, emails, mailboxes } from '$lib/jmap/stores.js';
  import { markAsRead, markAsUnread, toggleFlag, deleteEmail, moveToMailbox } from '$lib/jmap/actions.js';
  import Compose from '$lib/components/Compose.svelte';

  let showCompose = $state(false);
  let composeMode = $state<'reply' | 'forward' | 'new'>('new');
  let composeTarget = $state<any>(null);
  let showMoveMenu = $state(false);

  function formatAddress(addr: any): string {
    if (!addr) return '';
    return addr.name ? `${addr.name} <${addr.email}>` : addr.email;
  }

  function formatAddresses(list: any[] | undefined): string {
    return list?.map(formatAddress).join(', ') ?? '';
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleString();
  }

  function getBodyHtml(email: any): string {
    if (email.htmlBody?.length > 0) {
      const part = email.htmlBody[0];
      if (email.bodyValues?.[part.partId]) return email.bodyValues[part.partId].value;
    }
    if (email.textBody?.length > 0) {
      const part = email.textBody[0];
      if (email.bodyValues?.[part.partId]) {
        return `<pre style="white-space:pre-wrap;font-family:var(--font-mono);font-size:13px;color:var(--fg-secondary)">${email.bodyValues[part.partId].value}</pre>`;
      }
    }
    return `<p class="muted">No content available</p>`;
  }

  function handleReply() {
    composeMode = 'reply';
    composeTarget = $currentEmail;
    showCompose = true;
  }

  function handleForward() {
    composeMode = 'forward';
    composeTarget = $currentEmail;
    showCompose = true;
  }

  function handleComposeDone() {
    showCompose = false;
    composeTarget = null;
  }

  async function handleDelete() {
    if ($selectedEmailId) {
      await deleteEmail($selectedEmailId);
    }
  }

  async function handleToggleRead() {
    if (!$currentEmail) return;
    const seen = $currentEmail.keywords.$seen;
    if (seen) {
      await markAsUnread($currentEmail.id);
    } else {
      await markAsRead($currentEmail.id);
    }
  }

  // Listen for compose events from sidebar
  $effect(() => {
    function onCompose() {
      composeMode = 'new';
      composeTarget = null;
      showCompose = true;
    }
    window.addEventListener('jmap-compose', onCompose);
    return () => window.removeEventListener('jmap-compose', onCompose);
  });
</script>

<div class="mail-view">
  {#if showCompose}
    <Compose
      replyTo={composeMode === 'reply' ? composeTarget : null}
      forwardOf={composeMode === 'forward' ? composeTarget : null}
      onSend={handleComposeDone}
      onClose={handleComposeDone}
    />
  {:else if $currentEmail}
    <div class="mail-header">
      <h1 class="mail-subject">{$currentEmail.subject || '(no subject)'}</h1>

      <div class="mail-meta">
        <div class="mail-from">
          <span class="meta-label">From:</span>
          <span>{formatAddresses($currentEmail.from)}</span>
        </div>
        <div class="mail-to">
          <span class="meta-label">To:</span>
          <span>{formatAddresses($currentEmail.to)}</span>
        </div>
        {#if $currentEmail.cc?.length}
          <div class="mail-cc">
            <span class="meta-label">Cc:</span>
            <span>{formatAddresses($currentEmail.cc)}</span>
          </div>
        {/if}
        <div class="mail-date">
          <span class="meta-label">Date:</span>
          <span>{formatDate($currentEmail.receivedAt)}</span>
        </div>
      </div>

      {#if $currentEmail.hasAttachment}
        <div class="mail-attachments">📎 {$currentEmail.attachments?.length ?? 1} attachment(s)</div>
      {/if}

      <div class="mail-actions">
        <button class="btn btn-primary" onclick={handleReply}>↩ Reply</button>
        <button class="btn" onclick={handleForward}>↗ Forward</button>
        <button class="btn" onclick={handleToggleRead}>
          {$currentEmail.keywords.$seen ? 'Mark unread' : 'Mark read'}
        </button>
        <div class="action-separator"></div>
        <button class="btn btn-danger" onclick={handleDelete}>🗑 Delete</button>
      </div>
    </div>

    <div class="mail-body">
      {@html getBodyHtml($currentEmail)}
    </div>
  {:else}
    <div class="mail-empty">
      <div class="empty-icon">📧</div>
      <p class="muted">Select an email to read</p>
    </div>
  {/if}
</div>

<style>
  .mail-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--bg-primary);
  }

  .mail-header { padding: 20px 24px; border-bottom: 1px solid var(--border); }
  .mail-subject { font-size: 20px; font-weight: 600; margin-bottom: 12px; color: var(--fg-primary); }

  .mail-meta { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--fg-secondary); margin-bottom: 12px; }
  .meta-label { color: var(--fg-muted); margin-right: 4px; font-size: 12px; }
  .mail-attachments { font-size: 12px; color: var(--fg-muted); margin-bottom: 8px; }

  .mail-actions { display: flex; gap: 8px; align-items: center; }
  .action-separator { width: 1px; height: 20px; background: var(--border); }

  .btn {
    padding: 6px 14px; font-size: 13px; font-family: inherit;
    background: var(--bg-tertiary); color: var(--fg-secondary);
    border: 1px solid var(--border); border-radius: 4px; cursor: pointer;
  }
  .btn:hover { background: var(--bg-hover); color: var(--fg-primary); }

  .btn-primary { background: var(--accent); color: var(--bg-primary); border-color: var(--accent); font-weight: 500; }
  .btn-primary:hover { background: var(--accent-hover); }

  .btn-danger { color: var(--danger); border-color: var(--danger); opacity: 0.7; }
  .btn-danger:hover { background: var(--danger); color: var(--bg-primary); opacity: 1; }

  .mail-body { flex: 1; overflow-y: auto; padding: 24px; }

  .mail-empty {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px;
  }

  .empty-icon { font-size: 48px; opacity: 0.3; }
</style>
