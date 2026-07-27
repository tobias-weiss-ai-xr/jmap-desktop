<script lang="ts">
  import AppShell from '$lib/components/AppShell.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import MailList from '$lib/components/MailList.svelte';
  import MailView from '$lib/components/MailView.svelte';
</script>

<AppShell>
  <Sidebar />
  <MailList />
  <MailView />
</AppShell>
