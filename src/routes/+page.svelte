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

  async function connectWithSettings(settings: { serverUrl: string; username: string; password: string }) {
    try {
      await connect(settings);
    } catch (e: any) {
      connectError = e.toString();
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
      <h1>📧 JMAP Desktop</h1>
      <p class="welcome-tagline">Connect to any JMAP server to get started.</p>

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

        {#if connectError}
          <p class="text-danger">{connectError}</p>
        {/if}

        <button type="submit" class="btn btn-primary btn-full" disabled={connecting || !connectUrl || !connectUser || !connectPass}>
          {connecting ? 'Connecting…' : 'Connect'}
        </button>
      </form>

      <p class="welcome-hint">
        {#if connectUrl || connectUser}
          Preconfigured via <code>JMAP_SERVER_URL</code> env var? Click Connect to use those settings.
        {:else}
          Set <code>JMAP_SERVER_URL</code>, <code>JMAP_USERNAME</code>, and <code>JMAP_PASSWORD</code>
          environment variables to auto-connect on launch.
        {/if}
      </p>
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
    max-width: 420px;
    width: 100%;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 32px;
    text-align: center;
  }

  .welcome-card h1 {
    font-size: 28px;
    margin-bottom: 8px;
    color: var(--fg-primary);
  }

  .welcome-tagline {
    color: var(--fg-secondary);
    margin-bottom: 24px;
    font-size: 14px;
  }

  .connect-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
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
  }

  .connect-form input {
    padding: 10px 12px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-primary);
    border: 1px solid var(--border);
    border-radius: 6px;
    outline: none;
  }

  .connect-form input:focus {
    border-color: var(--accent);
  }

  .welcome-hint {
    margin-top: 16px;
    font-size: 12px;
    color: var(--fg-muted);
    line-height: 1.5;
  }

  .welcome-hint code {
    background: var(--bg-tertiary);
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 11px;
  }

  .btn {
    padding: 10px 16px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
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
