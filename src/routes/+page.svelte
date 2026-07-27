<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailView from '$lib/components/MailView.svelte';
  import {
    selectedMailboxId, selectedEmailId, emailQueryState,
    connected, isSyncing,
  } from '$lib/jmap/stores.js';
  import {
    connect, disconnect, fetchEmailsForMailbox,
    refreshMailboxes, searchEmails, setupEventListeners,
  } from '$lib/jmap/actions.js';

  let restored = $state(false);
  let searchRestore = $state<(() => void) | null>(null);

  onMount(() => {
    // Try to restore session — note: password is NOT persisted for security.
    // User will need to re-enter password if the app restarts.
    const saved = localStorage.getItem('jmap-settings');
    if (saved) {
      try {
        const settings = JSON.parse(saved);
        if (settings.serverUrl && settings.username && settings.password) {
          connect(settings).then(() => { restored = true; }).catch(() => {});
        }
        // If no password, just pre-fill settings page — don't attempt connect
      } catch (_e) { /* ignore stale settings */ }
    }

    // Listen for search events from sidebar
    function onSearch(e: Event) {
      const query = (e as CustomEvent).detail;
      if (query) handleSearch(query);
    }
    window.addEventListener('jmap-search', onSearch);
    return () => {
      window.removeEventListener('jmap-search', onSearch);
      // Don't call disconnect here — cleanup happens when user explicitly disconnects
    };
  });

  // Fetch emails when mailbox selection changes (but not on initial mount unless restored)
  $effect(() => {
    const mailboxId = $selectedMailboxId;
    if (mailboxId && (restored || $connected)) {
      fetchEmailsForMailbox(mailboxId);
    }
  });

  // Auto-mark as read when email is selected
  $effect(() => {
    const emailId = $selectedEmailId;
    if (emailId && $connected) {
      // Mark as read after a brief delay (optimistic UX)
      const timer = setTimeout(async () => {
        const { markAsRead } = await import('$lib/jmap/actions.js');
        try { await markAsRead(emailId); } catch (_) {}
      }, 1000);
      return () => clearTimeout(timer);
    }
  });

  async function handleSearch(query: string) {
    const restore = await searchEmails(query);
    searchRestore = restore ?? null;
  }

  function handleClearSearch() {
    if (searchRestore) {
      searchRestore();
      searchRestore = null;
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
