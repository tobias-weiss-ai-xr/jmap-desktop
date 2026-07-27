<script lang="ts">
  import { currentEmail } from '$lib/jmap/stores.js';

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

  // Render email body HTML safely (will be sanitized in production)
  function getBodyHtml(email: any): string {
    if (email.htmlBody?.length > 0) {
      const part = email.htmlBody[0];
      if (email.bodyValues?.[part.partId]) {
        return email.bodyValues[part.partId].value;
      }
    }
    if (email.textBody?.length > 0) {
      const part = email.textBody[0];
      if (email.bodyValues?.[part.partId]) {
        return `<pre style="white-space:pre-wrap;font-family:var(--font-mono);font-size:13px;color:var(--fg-secondary)">${email.bodyValues[part.partId].value}</pre>`;
      }
    }
    return `<p class="muted">No content available</p>`;
  }
</script>

<div class="mail-view">
  {#if $currentEmail}
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
        <div class="mail-attachments">
          📎 {$currentEmail.attachments?.length ?? 1} attachment(s)
        </div>
      {/if}

      <div class="mail-actions">
        <button class="btn btn-primary">↩ Reply</button>
        <button class="btn">↗ Forward</button>
      </div>
    </div>

    <div class="mail-body">
      {@html getBodyHtml($currentEmail)}
    </div>
  {:else}
    <div class="mail-empty">
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

  .mail-header {
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
  }

  .mail-subject {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--fg-primary);
  }

  .mail-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    color: var(--fg-secondary);
    margin-bottom: 12px;
  }

  .meta-label {
    color: var(--fg-muted);
    margin-right: 4px;
    font-size: 12px;
  }

  .mail-attachments {
    font-size: 12px;
    color: var(--fg-muted);
    margin-bottom: 8px;
  }

  .mail-actions {
    display: flex;
    gap: 8px;
  }

  .btn {
    padding: 6px 14px;
    font-size: 13px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
  }

  .btn:hover {
    background: var(--bg-hover);
    color: var(--fg-primary);
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
    font-weight: 500;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .mail-body {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }

  .mail-empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>
