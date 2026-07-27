<script lang="ts">
  import { sendEmail } from '$lib/jmap/actions.js';

  interface Props {
    onSend?: () => void;
    onClose?: () => void;
    replyTo?: any;
    forwardOf?: any;
    initialTo?: string;
    initialSubject?: string;
  }

  let {
    onSend,
    onClose,
    replyTo,
    forwardOf,
    initialTo = '',
    initialSubject = '',
  }: Props = $props();

  let to = $state(initialTo);
  let cc = $state('');
  let bcc = $state('');
  let subject = $state(initialSubject);
  let body = $state('');
  let sending = $state(false);
  let error = $state('');

  // Pre-fill for reply
  $effect(() => {
    if (replyTo) {
      const fromEmail = replyTo.from?.[0]?.email || replyTo.from?.[0]?.name || '';
      to = fromEmail;
      subject = replyTo.subject?.startsWith('Re:') ? replyTo.subject : `Re: ${replyTo.subject}`;
      body = `\n\n--- Original Message ---\nFrom: ${replyTo.from?.[0]?.name || ''} <${replyTo.from?.[0]?.email || ''}>\nDate: ${replyTo.receivedAt}\nSubject: ${replyTo.subject}\n\n${replyTo.preview || ''}`;
    }
    if (forwardOf) {
      subject = forwardOf.subject?.startsWith('Fwd:') ? forwardOf.subject : `Fwd: ${forwardOf.subject}`;
      body = `\n\n--- Forwarded Message ---\nFrom: ${forwardOf.from?.[0]?.name || ''} <${forwardOf.from?.[0]?.email || ''}>\nDate: ${forwardOf.receivedAt}\nSubject: ${forwardOf.subject}\n\n${forwardOf.preview || ''}`;
    }
  });

  function parseRecipients(str: string): string[] {
    return str.split(',').map((s) => s.trim()).filter(Boolean);
  }

  async function handleSend() {
    if (!to.trim()) { error = 'Recipient is required'; return; }
    if (!subject.trim()) { error = 'Subject is required'; return; }

    sending = true;
    error = '';

    try {
      await sendEmail({
        to: parseRecipients(to),
        cc: cc.trim() ? parseRecipients(cc) : undefined,
        bcc: bcc.trim() ? parseRecipients(bcc) : undefined,
        subject,
        bodyText: body,
        replyToId: replyTo?.id,
      });
      onSend?.();
    } catch (e: any) {
      error = e.toString();
    } finally {
      sending = false;
    }
  }

  // Keyboard shortcut: Ctrl+Enter to send
  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleSend();
    }
    if (e.key === 'Escape') {
      onClose?.();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="compose" onkeydown={handleKeydown}>
  <div class="compose-header">
    <h2>{replyTo ? '↩ Reply' : forwardOf ? '↗ Forward' : '✉ New Email'}</h2>
    <button class="close-btn" onclick={onClose}>✕</button>
  </div>

  <div class="compose-fields">
    <label class="field">
      <span class="field-label">To</span>
      <input type="text" bind:value={to} placeholder="recipient@example.com" />
    </label>

    <label class="field">
      <span class="field-label">Cc</span>
      <input type="text" bind:value={cc} placeholder="cc@example.com (optional)" />
    </label>

    <label class="field">
      <span class="field-label">Bcc</span>
      <input type="text" bind:value={bcc} placeholder="bcc@example.com (optional)" />
    </label>

    <label class="field">
      <span class="field-label">Subject</span>
      <input type="text" bind:value={subject} placeholder="Email subject" />
    </label>
  </div>

  <textarea
    class="compose-body"
    bind:value={body}
    placeholder="Write your message…"
    rows="12"
  ></textarea>

  {#if error}
    <p class="text-danger compose-error">{error}</p>
  {/if}

  <div class="compose-footer">
    <button class="btn btn-primary" onclick={handleSend} disabled={sending}>
      {sending ? 'Sending…' : 'Send'} (Ctrl+Enter)
    </button>
    <button class="btn" onclick={onClose}>Cancel</button>
  </div>
</div>

<style>
  .compose {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .compose-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }

  .compose-header h2 {
    font-size: 15px;
    font-weight: 600;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--fg-muted);
    font-size: 18px;
    cursor: pointer;
    padding: 4px 8px;
  }

  .close-btn:hover {
    color: var(--fg-primary);
    background: var(--bg-hover);
  }

  .compose-fields {
    padding: 0 16px;
  }

  .field {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }

  .field-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--fg-muted);
    min-width: 60px;
    text-transform: uppercase;
  }

  .field input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--fg-primary);
    font-size: 14px;
    font-family: inherit;
    outline: none;
  }

  .compose-body {
    flex: 1;
    padding: 16px;
    background: transparent;
    border: none;
    color: var(--fg-secondary);
    font-size: 14px;
    font-family: var(--font-mono);
    line-height: 1.6;
    resize: none;
    outline: none;
  }

  .compose-error {
    padding: 8px 16px;
    font-size: 13px;
  }

  .compose-footer {
    display: flex;
    gap: 8px;
    padding: 12px 16px;
    border-top: 1px solid var(--border);
  }

  .btn {
    padding: 8px 16px;
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

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
