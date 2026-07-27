<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/AppShell.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailView from '$lib/components/MailView.svelte';
  import { selectedMailboxId, selectedEmailId, emailQueryState, connected } from '$lib/jmap/stores.js';
  import { connect, disconnect, fetchEmailsForMailbox, refreshMailboxes, searchEmails, setupEventListeners } from '$lib/jmap/actions.js';

  onMount(() => {
    // Try to restore session from localStorage
    const saved = localStorage.getItem('jmap-settings');
    if (saved) {
      try {
        const settings = JSON.parse(saved);
        connect(settings);
      } catch (e) { /* ignore stale settings */ }
    }

    // Listen for search events from sidebar
    function onSearch(e: Event) {
      const query = (e as CustomEvent).detail;
      if (query) handleSearch(query);
    }
    window.addEventListener('jmap-search', onSearch);
    return () => window.removeEventListener('jmap-search', onSearch);
  });

  // Fetch emails when mailbox selection changes
  $effect(() => {
    const mailboxId = $selectedMailboxId;
    if (mailboxId) {
      fetchEmailsForMailbox(mailboxId);
    }
  });

  // Fetch full email body when selection changes
  $effect(() => {
    const emailId = $selectedEmailId;
    if (emailId) {
      // The email might already be loaded; if not, fetch it
      // (handled by fetchEmailsForMailbox for the initial load)
    }
  });

  async function handleSearch(query: string) {
    await searchEmails(query);
  }
</script>

<AppShell>
  <Sidebar />
  <MailList />
  <MailView />
</AppShell>
