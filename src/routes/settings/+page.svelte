<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { session, connected, mailboxes } from '$lib/jmap/stores.js';

  let serverUrl = $state('');
  let username = $state('');
  let password = $state('');
  let error = $state('');
  let connecting = $state(false);

  // Auto-fill saved credentials
  onMount(() => {
    const saved = localStorage.getItem('jmap-settings');
    if (saved) {
      try {
        const settings = JSON.parse(saved);
        serverUrl = settings.serverUrl || '';
        username = settings.username || '';
      } catch (e) { /* ignore */ }
    }
  });

  async function handleConnect() {
    connecting = true;
    error = '';
    try {
      const result: any = await invoke('connect_jmap', {
        settings: {
          serverUrl: serverUrl.replace(/\/$/, ''),
          username,
          password,
        },
      });
      // Save credentials (password not persisted)
      localStorage.setItem('jmap-settings', JSON.stringify({
        serverUrl: serverUrl.replace(/\/$/, ''),
        username,
      }));
      session.set(result as any);
      // Load mailboxes
      const mbs: any[] = await invoke('get_mailboxes');
      mailboxes.set(mbs);
    } catch (e: any) {
      error = e.toString();
    } finally {
      connecting = false;
    }
  }

  async function handleDisconnect() {
    try {
      await invoke('disconnect_jmap');
    } catch (_) { /* ignore */ }
    session.set(null);
    mailboxes.set([]);
    localStorage.removeItem('jmap-settings');
  }
</script>

<svelte:head>
  <title>Settings — JMAP Desktop</title>
</svelte:head>

<div class="settings-page">
  <h1>Settings</h1>

  <section class="settings-section">
    <h2>JMAP Account</h2>

    {#if !$connected}
      <form class="settings-form" onsubmit={(e) => { e.preventDefault(); handleConnect(); }}>
        <label>
          Server URL
          <input type="url" bind:value={serverUrl} placeholder="https://mail.example.com" required />
        </label>
        <label>
          Username
          <input type="text" bind:value={username} placeholder="user@example.com" required />
        </label>
        <label>
          Password / Token
          <input type="password" bind:value={password} placeholder="••••••••" required />
        </label>

        {#if error}
          <p class="text-danger">{error}</p>
        {/if}

        <button type="submit" class="btn btn-primary" disabled={connecting}>
          {connecting ? 'Connecting…' : 'Connect'}
        </button>
      </form>
    {:else}
      <div class="connected-info">
        <p>✅ Connected as <strong>{$session?.username}</strong></p>
        <p class="muted">{$session?.accounts[$session?.primaryAccounts['urn:ietf:params:jmap:mail']]?.name ?? ''}</p>
        <button class="btn" onclick={handleDisconnect}>Disconnect</button>
      </div>
    {/if}
  </section>
</div>

<style>
  .settings-page {
    max-width: 500px;
    margin: 0 auto;
    padding: 32px;
  }

  h1 {
    font-size: 24px;
    margin-bottom: 24px;
  }

  h2 {
    font-size: 16px;
    margin-bottom: 16px;
    color: var(--fg-secondary);
  }

  .settings-section {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 16px;
  }

  .settings-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
    color: var(--fg-muted);
  }

  input {
    padding: 8px 12px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    outline: none;
  }

  input:focus {
    border-color: var(--accent);
  }

  .btn {
    padding: 8px 16px;
    font-size: 14px;
    font-family: inherit;
    background: var(--bg-tertiary);
    color: var(--fg-secondary);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    align-self: flex-start;
  }

  .btn:hover {
    background: var(--bg-hover);
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

  .connected-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .connected-info p {
    font-size: 14px;
  }
</style>
