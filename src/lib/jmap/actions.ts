/**
 * JMAP actions — functions that wire stores to Tauri commands.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { get } from 'svelte/store';
import {
  session, connected, mailboxes, selectedAccountId, selectedMailboxId,
  selectedEmailId, emailIds, emails, emailQueryState,
  isLoadingEmails, isSyncing, syncStatus, lastSyncAt,
} from './stores.js';
import { getMailboxes as fetchMailboxes, queryEmails, getEmails } from './client.js';
import type { ConnectionSettings, Email, Mailbox } from './types.js';
import { addError, addSuccess } from '$lib/toast.svelte.js';

// ── Configuration ──

/** Check for preconfigured settings from env vars (JMAP_SERVER_URL etc.) */
export async function getPreconfiguredSettings(): Promise<ConnectionSettings | null> {
  try {
    const result = await invoke<any>('get_preconfigured_settings');
    if (!result) return null;
    return result as ConnectionSettings;
  } catch (_e) {
    return null;
  }
}

// ── Connection ──

export async function connect(settings: ConnectionSettings) {
  try {
    const result: any = await invoke('connect_jmap', { settings });
    session.set(result);

    const accountId = result?.primaryAccounts?.['urn:ietf:params:jmap:mail'];
    if (accountId) selectedAccountId.set(accountId);

    await refreshMailboxes();
    setupEventListeners();
    addSuccess('Connected to JMAP server');
  } catch (e: any) {
    addError(`Connection failed: ${e}`);
    throw e;
  }
}

export async function disconnect() {
  cleanupEventListeners();
  try {
    await invoke('disconnect_jmap');
  } catch (_e) { /* ignore on disconnect */ }
  session.set(null);
  mailboxes.set([]);
  emailIds.set([]);
  emails.set(new Map());
  selectedMailboxId.set(null);
  selectedEmailId.set(null);
  syncStatus.set('disconnected');
  lastSyncAt.set(null);
}

// ── Mailboxes ──

export async function refreshMailboxes() {
  try {
    const mbs: any[] = await fetchMailboxes();
    mailboxes.set(mbs as unknown as Mailbox[]);

    if (!get(selectedMailboxId)) {
      const inbox = mbs.find((m: any) => m.role === 'inbox');
      if (inbox) selectedMailboxId.set(inbox.id);
    }
  } catch (e: any) {
    addError(`Failed to fetch mailboxes: ${e}`);
  }
}

// ── Emails ──

export async function fetchEmailsForMailbox(mailboxId: string | null) {
  if (!mailboxId) { emailIds.set([]); emails.set(new Map()); return; }

  isLoadingEmails.set(true);
  try {
    const result: any = await queryEmails(
      { inMailbox: mailboxId },
      [{ property: 'receivedAt', isAscending: false }],
      100, 0,
    );

    emailIds.set(result.ids ?? []);
    emailQueryState.set(result.queryState ?? '');

    if (result.ids?.length > 0) {
      const response: any = await getEmails(result.ids);
      const map = new Map<string, Email>();
      for (const email of response.list ?? []) map.set(email.id, email);
      emails.set(map);

      if (!get(selectedEmailId) && result.ids.length > 0) {
        selectedEmailId.set(result.ids[0]);
      }
    } else {
      emails.set(new Map());
    }
  } catch (e: any) {
    addError(`Failed to fetch emails: ${e}`);
  } finally {
    isLoadingEmails.set(false);
  }
}

export async function fetchEmail(id: string): Promise<Email | null> {
  try {
    const response: any = await getEmails([id]);
    const email = response.list?.[0];
    if (email) {
      const existing = get(emails);
      const updated = new Map(existing);
      updated.set(id, email);
      emails.set(updated);
      return email;
    }
  } catch (e: any) {
    addError(`Failed to fetch email: ${e}`);
  }
  return null;
}

/**
 * Search emails and replace the current email list with results.
 * Returns a function to restore the previous mailbox view.
 */
export async function searchEmails(text: string): Promise<(() => void) | null> {
  if (!text.trim()) return null;
  try {
    const result: any = await invoke('search_emails', { text, limit: 50 });
    if (result.ids?.length > 0) {
      const response: any = await getEmails(result.ids);
      const map = new Map<string, Email>();
      for (const email of response.list ?? []) map.set(email.id, email);
      emailIds.set(result.ids ?? []);
      emails.set(map);
    } else {
      emailIds.set([]);
      emails.set(new Map());
    }

    // Return restore function that re-fetches the current mailbox
    const restoreMailbox = get(selectedMailboxId);
    return () => {
      if (restoreMailbox) {
        fetchEmailsForMailbox(restoreMailbox);
      }
    };
  } catch (e: any) {
    addError(`Search failed: ${e}`);
    return null;
  }
}

// ── Mutations ──

export async function markAsRead(id: string) {
  try {
    await invoke('mark_seen', { id, seen: true });
    updateLocalEmail(id, (e) => ({ ...e, keywords: { ...e.keywords, $seen: true } }));
  } catch (e: any) {
    addError(`Failed to mark as read: ${e}`);
  }
}

export async function markAsUnread(id: string) {
  try {
    await invoke('mark_seen', { id, seen: false });
    updateLocalEmail(id, (e) => {
      const kw = { ...e.keywords };
      delete kw.$seen;
      return { ...e, keywords: kw };
    });
  } catch (e: any) {
    addError(`Failed to mark as unread: ${e}`);
  }
}

export async function toggleFlag(id: string, flagged: boolean) {
  const newValue = !flagged;
  try {
    await invoke('toggle_flagged', { id, value: newValue });
    updateLocalEmail(id, (e) => {
      const kw = { ...e.keywords };
      if (newValue) {
        kw.$flagged = true;
      } else {
        delete kw.$flagged;
      }
      return { ...e, keywords: kw };
    });
  } catch (e: any) {
    addError(`Failed to toggle flag: ${e}`);
  }
}

export async function moveToMailbox(id: string, toMailboxId: string) {
  try {
    await invoke('move_email', { id, toMailboxId });
    // Optimistic update: replace all mailboxIds with just the target
    updateLocalEmail(id, (e) => ({ ...e, mailboxIds: { [toMailboxId]: true } }));
    refreshMailboxes();
  } catch (e: any) {
    addError(`Failed to move email: ${e}`);
  }
}

export async function deleteEmail(id: string) {
  try {
    await invoke('delete_email', { id });
    const newIds = get(emailIds).filter((eid: string) => eid !== id);
    emailIds.set(newIds);
    const newMap = new Map(get(emails));
    newMap.delete(id);
    emails.set(newMap);
    if (get(selectedEmailId) === id) selectedEmailId.set(newIds[0] ?? null);
    refreshMailboxes();
    addSuccess('Email deleted');
  } catch (e: any) {
    addError(`Failed to delete email: ${e}`);
  }
}

export async function sendEmail(params: {
  to: string[]; cc?: string[]; bcc?: string[];
  subject: string; bodyText: string; bodyHtml?: string; replyToId?: string;
}) {
  const accountId = get(selectedAccountId);
  const sess = get(session);
  if (!sess || !accountId) throw new Error('Not connected');

  await invoke('send_email', {
    from: sess.username,
    to: params.to,
    cc: params.cc ?? null,
    bcc: params.bcc ?? null,
    subject: params.subject,
    bodyText: params.bodyText,
    bodyHtml: params.bodyHtml ?? null,
    replyToId: params.replyToId ?? null,
  });
  addSuccess('Email sent');
}

function updateLocalEmail(id: string, updater: (e: Email) => Email) {
  const existing = get(emails).get(id);
  if (existing) {
    const updated = new Map(get(emails));
    updated.set(id, updater(existing));
    emails.set(updated);
  }
}

// ── Push Events ──

let _cleanup: Array<() => void> = [];

export function setupEventListeners() {
  cleanupEventListeners();

  const unlisten1 = listen<any>('jmap://mailboxes-changed', async () => {
    await refreshMailboxes();
  });

  const unlisten2 = listen<any>('jmap://emails-changed', async (event) => {
    const { created, updated, destroyed } = event.payload as any;
    const toFetch = [...(created ?? []), ...(updated ?? [])].filter(
      (id: string) => !get(emails).has(id)
    );
    if (toFetch.length > 0) {
      const response: any = await getEmails(toFetch);
      const map = new Map(get(emails));
      for (const email of response.list ?? []) map.set(email.id, email);
      emails.set(map);
    }
    if (destroyed?.length > 0) {
      const newIds = get(emailIds).filter((id: string) => !destroyed.includes(id));
      emailIds.set(newIds);
      const map = new Map(get(emails));
      for (const id of destroyed) map.delete(id);
      emails.set(map);
    }
    refreshMailboxes();
  });

  const unlisten3 = listen<string>('jmap://sync-status', (event) => {
    syncStatus.set(event.payload);
    if (event.payload === 'synced' || event.payload === 'push-connected') {
      lastSyncAt.set(new Date());
      isSyncing.set(false);
    } else if (event.payload === 'syncing' || event.payload === 'starting') {
      isSyncing.set(true);
    }
  });

  _cleanup = [unlisten1, unlisten2, unlisten3].map((p) => () => p.then((fn) => fn()));
}

export function cleanupEventListeners() {
  _cleanup.forEach((fn) => fn());
  _cleanup = [];
}
