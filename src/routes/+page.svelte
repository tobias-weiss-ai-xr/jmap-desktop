<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailView from '$lib/components/MailView.svelte';
  import Toast from '$lib/components/Toast.svelte';
  import {
    selectedMailboxId, selectedEmailId,
    connected,
  } from '$lib/jmap/stores.js';
  import {
    connect, fetchEmailsForMailbox, searchEmails,
  } from '$lib/jmap/actions.js';

  let restored = $state(false);
  let searchRestore = $state<(() => void) | null>(null);

  onMount(() => {
    // Try to restore session from localStorage
    const saved = localStorage.getItem('jmap-settings');
    if (saved) {
      try {
        const settings = JSON.parse(saved);
        if (settings.serverUrl && settings.username && settings.password) {
          connect(settings).then(() => { restored = true; }).catch(() => {});
        }
      } catch (_e) { /* ignore stale settings */ }
    }

    // Listen for search events from sidebar
    function onSearch(e: Event) {
      const query = (e as CustomEvent).detail;
      if (query) handleSearch(query);
    }
    window.addEventListener('jmap-search', onSearch);
    return () => window.removeEventListener('jmap-search', onSearch);
  });

  // Fetch emails when mailbox selection changes (not on initial mount unless restored)
  $effect(() => {
    const mailboxId = $selectedMailboxId;
    if (mailboxId && (restored || $connected)) {
      // Clear any active search when switching mailboxes
      if (searchRestore) {
        searchRestore = null;
      }
      fetchEmailsForMailbox(mailboxId);
    }
  });

  // Auto-mark as read when email is selected (1s delay for UX)
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
    if (restore) {
      restore();
    }
  }
</script>

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

<Toast />

<style>
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
