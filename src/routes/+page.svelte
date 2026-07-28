<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailView from '$lib/components/MailView.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import {
    selectedMailboxId, selectedEmailId, connected,
  } from '$lib/jmap/stores.js';
  import {
    connect, fetchEmailsForMailbox, searchEmails, getPreconfiguredSettings,
  } from '$lib/jmap/actions.js';

  // ── Inline connect form state ──
  let connectUrl = $state('');
  let connectUser = $state('');
  let connectPass = $state('');
  let connectSkipTls = $state(false);
  let connectError = $state('');
  let connecting = $state(false);
  let restored = $state(false);
  let searchRestore = $state<(() => void) | null>(null);

  onMount(() => {
    (async () => {
      // Priority 1: preconfigured env vars
      const preconf = await getPreconfiguredSettings();
      if (preconf) {
        connectUrl = preconf.serverUrl;
        connectUser = preconf.username;
        connectPass = preconf.password;
        connectSkipTls = !!(preconf as any).skipTlsVerify;
        await connectWithSettings(preconf);
        restored = true;
        return;
      }

      // Priority 2: saved credentials from localStorage
      const saved = localStorage.getItem('jmap-settings');
      if (saved) {
        try {
          const parsed: any = JSON.parse(saved);
          if (parsed.serverUrl && parsed.username && parsed.password) {
            connectUrl = parsed.serverUrl;
            connectUser = parsed.username;
            connectPass = parsed.password;
            connectSkipTls = !!parsed.skipTlsVerify;
            await connectWithSettings(parsed);
            restored = true;
            return;
          }
        } catch (_e) { /* ignore stale settings */ }
      }
    })();

    function onSearch(e: Event) {
      const query = (e as CustomEvent).detail;
      if (query) handleSearch(query);
    }
    window.addEventListener('jmap-search', onSearch);
    return () => window.removeEventListener('jmap-search', onSearch);
  });

  async function connectWithSettings(settings: { serverUrl: string; username: string; password: string; skipTlsVerify?: boolean }) {
    connecting = true;
    connectError = '';
    try {
      await connect(settings);
    } catch (e: any) {
      connectError = e.toString();
    } finally {
      connecting = false;
    }
  }

  async function handleInlineConnect(e: SubmitEvent) {
    e.preventDefault();
    connecting = true;
    connectError = '';
    try {
      const settings = {
        serverUrl: connectUrl.replace(/\/$/, ''),
        username: connectUser,
        password: connectPass,
        skipTlsVerify: connectSkipTls,
      };
      await connect(settings);
      // Save to localStorage for next time
      localStorage.setItem('jmap-settings', JSON.stringify(settings));
    } catch (err: any) {
      connectError = err.toString();
    } finally {
      connecting = false;
    }
  }

  // Fetch emails when mailbox selection changes
  $effect(() => {
    const mailboxId = $selectedMailboxId;
    if (mailboxId && (restored || $connected)) {
      if (searchRestore) searchRestore = null;
      fetchEmailsForMailbox(mailboxId);
    }
  });

  // Auto-mark as read when email is selected (1s delay)
  $effect(() => {
    const emailId = $selectedEmailId;
    if (emailId && $connected) {
      const timer = setTimeout(async () => {
        try {
          const { markAsRead } = await import('$lib/jmap/actions.js');
          await markAsRead(emailId);
        } catch (_e) {}
      }, 1000);
      return () => clearTimeout(timer);
    }
  });

  async function handleSearch(query: string) {
    const restore = await searchEmails(query);
    searchRestore = restore ?? null;
  }

  function handleClearSearch() {
    const restore = searchRestore;
    searchRestore = null;
    if (restore) restore();
  }
</script>

{#if !$connected}
  <!-- Welcome / Connect screen -->
  <div class="welcome">
    <div class="welcome-card">
      <div class="welcome-brand">
        <span class="brand-icon">📧</span>
        <h1>JMAP Desktop</h1>
        <p class="welcome-tagline">Connect to any JMAP server to get started.</p>
      </div>

      <form class="connect-form" onsubmit={handleInlineConnect}>
        <label>
          <span class="field-label">Server URL</span>
          <input type="url" bind:value={connectUrl} placeholder="https://mail.example.com" required />
        </label>
        <label>
          <span class="field-label">Username</span>
          <input type="text" bind:value={connectUser} placeholder="user@example.com" required />
        </label>
        <label>
          <span class="field-label">Password / Token</span>
          <input type="password" bind:value={connectPass} placeholder="••••••••" required />
        </label>
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={connectSkipTls} />
          <span>Skip TLS verification (self-signed certs)</span>
        </label>

        {#if connectError}
          <div class="error-banner" role="alert">
            <span class="error-icon">⚠</span>
            <span>{connectError}</span>
            <button type="button" class="error-dismiss" onclick={() => connectError = ''} aria-label="Dismiss">✕</button>
          </div>
        {/if}

        <button type="submit" class="btn btn-primary btn-full" disabled={connecting || !connectUrl || !connectUser || !connectPass}>
          {#if connecting}
            <span class="spinner"></span> Connecting…
          {:else}
            Connect
          {/if}
        </button>
      </form>

      <div class="welcome-footer">
        {#if connectUrl && connectUser && connectPass && !restored}
          <p class="welcome-hint">
            <span class="hint-icon">💡</span>
            Auto-connect from <code>JMAP_SERVER_URL</code> env var — connecting now…
          </p>
        {:else if connectUrl || connectUser}
          <p class="welcome-hint">
            <span class="hint-icon">💡</span>
            Preconfigured via <code>JMAP_SERVER_URL</code> env var.
          </p>
        {:else}
          <p class="welcome-hint">
            <span class="hint-icon">💡</span>
            Set <code>JMAP_SERVER_URL</code>, <code>JMAP_USERNAME</code>, and <code>JMAP_PASSWORD</code>
            environment variables to auto-connect on launch.
          </p>
        {/if}

        <div class="shortcuts-hint">
          <span class="shortcuts-title">Keyboard shortcuts</span>
          <span><kbd>j</kbd>/<kbd>k</kbd> Navigate</span>
          <span><kbd>r</kbd> Reply</span>
          <span><kbd>c</kbd> Compose</span>
          <span><kbd>x</kbd> Flag</span>
          <span><kbd>Del</kbd> Delete</span>
          <span><kbd>Esc</kbd> Close</span>
        </div>
      </div>
    </div>
  </div>
{:else}
  <!-- Connected: 3-pane layout -->
  {#if searchRestore}
    <div class="search-bar-top">
      <span class="search-indicator">🔍 Search results</span>
      <button class="clear-search" onclick={handleClearSearch}>✕ Clear</button>
    </div>
  {/if}

  <AppShell>
    <Sidebar />
    <MailList />
    <MailView />
  </AppShell>
{/if}

<Toast />

<style>
  /* ── Welcome Screen ── */
  .welcome {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--bg-primary);
    padding: 32px;
  }

  .welcome-card {
    max-width: 440px;
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 16px;
    overflow: hidden;
  }

  .welcome-brand {
    padding: 40px 32px 24px;
    text-align: center;
  }

  .brand-icon {
    font-size: 48px;
    display: block;
    margin-bottom: 12px;
  }

  .welcome-card h1 {
    font-size: 26px;
    margin-bottom: 6px;
    color: var(--fg-primary);
  }

  .welcome-tagline {
    color: var(--fg-muted);
    font-size: 14px;
  }

  .connect-form {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 24px 32px;
    border-top: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    text-align: left;
  }

  .connect-form label {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .field-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .connect-form input[type="url"],
  .connect-form input[type="text"],
  .connect-form input[type="password"] {
    padding: 10px 12px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    outline: none;
    transition: border-color 0.15s;
  }

  .connect-form input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .checkbox-label {
    flex-direction: row;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    color: var(--fg-secondary);
  }

  .checkbox-label input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }

  /* Error banner — dismissible, prominent */
  .error-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: color-mix(in srgb, var(--danger) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--danger) 30%, transparent);
    border-radius: 6px;
    color: var(--danger);
    font-size: 13px;
    line-height: 1.4;
  }

  .error-icon { flex-shrink: 0; }

  .error-dismiss {
    margin-left: auto;
    background: transparent;
    border: none;
    color: var(--fg-muted);
    cursor: pointer;
    padding: 2px 4px;
    font-size: 12px;
    flex-shrink: 0;
  }

  .error-dismiss:hover { color: var(--danger); }

  /* Buttons */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 16px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  .btn:hover {
    background: var(--bg-hover);
    color: var(--fg-primary);
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg-primary);
    border-color: var(--accent);
    font-weight: 600;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-full {
    width: 100%;
    margin-top: 4px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Spinner animation */
  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid color-mix(in srgb, var(--bg-primary) 40%, transparent);
    border-top-color: var(--bg-primary);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Footer */
  .welcome-footer {
    padding: 20px 32px 24px;
  }

  .welcome-hint {
    font-size: 12px;
    color: var(--fg-muted);
    line-height: 1.6;
    margin-bottom: 16px;
  }

  .hint-icon { margin-right: 4px; }

  .welcome-hint code {
    background: var(--bg-tertiary);
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
    color: var(--fg-secondary);
  }

  /* Keyboard shortcuts hint */
  .shortcuts-hint {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    padding-top: 14px;
    border-top: 1px solid var(--border);
  }

  .shortcuts-title {
    display: block;
    width: 100%;
    font-size: 11px;
    color: var(--fg-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 2px;
  }

  .shortcuts-hint span {
    font-size: 11px;
    color: var(--fg-muted);
    display: flex;
    align-items: center;
    gap: 3px;
  }

  kbd {
    display: inline-block;
    padding: 1px 5px;
    font-size: 10px;
    font-family: var(--font-mono);
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 3px;
    color: var(--fg-secondary);
    line-height: 1.6;
  }

  /* ── Search bar top ── */
  .search-bar-top {
    position: fixed;
    top: 0;
    left: var(--sidebar-width);
    right: 0;
    height: 36px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 100;
  }

  .search-indicator {
    font-size: 13px;
    color: var(--fg-secondary);
  }

  .clear-search {
    background: transparent;
    border: none;
    color: var(--accent);
    font-size: 13px;
    cursor: pointer;
    padding: 4px 8px;
  }

  .clear-search:hover {
    text-decoration: underline;
  }
</style>
