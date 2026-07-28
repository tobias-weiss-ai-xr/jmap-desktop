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
import { logger } from '$lib/logger.js';

// ── Configuration ──

/** Check for preconfigured settings from env vars (JMAP_SERVER_URL etc.) */
export async function getPreconfiguredSettings(): Promise<ConnectionSettings | null> {
  try {
    const result = await invoke<any>('get_preconfigured_settings');
    if (!result) return null;
    logger.info('config', 'preconfigured settings loaded', {
      serverUrl: result.serverUrl,
      username: result.username,
    });
    return result as ConnectionSettings;
  } catch (e: any) {
    logger.warn('config', 'failed to load preconfigured settings', e);
    return null;
  }
}

// ── Connection ──

export async function connect(settings: ConnectionSettings) {
  logger.info('jmap', 'connecting', { serverUrl: settings.serverUrl, username: settings.username });
  try {
    const result: any = await logger.time('info', 'jmap', 'connect', () =>
      invoke('connect_jmap', { settings })
    );
    session.set(result);

    const accountId = result?.primaryAccounts?.['urn:ietf:params:jmap:mail'];
    if (accountId) selectedAccountId.set(accountId);

    logger.info('jmap', 'connected', { accountId });
    await refreshMailboxes();
    setupEventListeners();
    addSuccess('Connected to JMAP server');
  } catch (e: any) {
    logger.error('jmap', 'connect failed', e);
    addError(`Connection failed: ${e}`);
    throw e;
  }
}

export async function disconnect() {
  logger.info('jmap', 'disconnecting');
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
  logger.info('jmap', 'disconnected');
}

// ── Mailboxes ──

export async function refreshMailboxes() {
  try {
    const mbs: any[] = await logger.time('debug', 'jmap', 'fetch mailboxes', () => fetchMailboxes());
    mailboxes.set(mbs as unknown as Mailbox[]);
    logger.debug('jmap', `fetched ${mbs.length} mailboxes`);

    if (!get(selectedMailboxId)) {
      const inbox = mbs.find((m: any) => m.role === 'inbox');
      if (inbox) {
        selectedMailboxId.set(inbox.id);
        logger.debug('jmap', 'auto-selected inbox', { id: inbox.id });
      }
    }
  } catch (e: any) {
    logger.error('jmap', 'fetch mailboxes failed', e);
    addError(`Failed to fetch mailboxes: ${e}`);
  }
}

// ── Emails ──

export async function fetchEmailsForMailbox(mailboxId: string | null) {
  if (!mailboxId) { emailIds.set([]); emails.set(new Map()); return; }

  isLoadingEmails.set(true);
  try {
    const result: any = await logger.time('debug', 'jmap', 'query emails', () =>
      queryEmails(
        { inMailbox: mailboxId },
        [{ property: 'receivedAt', isAscending: false }],
        100, 0,
      )
    );

    emailIds.set(result.ids ?? []);
    emailQueryState.set(result.queryState ?? '');
    logger.debug('jmap', `queried ${result.ids?.length ?? 0} emails in mailbox`, {
      mailboxId,
      total: result.total,
    });

    if (result.ids?.length > 0) {
      const response: any = await logger.time('debug', 'jmap', 'fetch email bodies', () =>
        getEmails(result.ids)
      );
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
    logger.error('jmap', 'fetch emails failed', e);
    addError(`Failed to fetch emails: ${e}`);
  } finally {
    isLoadingEmails.set(false);
  }
}

export async function fetchEmail(id: string): Promise<Email | null> {
  try {
    const response: any = await logger.time('debug', 'jmap', `fetch email ${id}`, () =>
      getEmails([id])
    );
    const email = response.list?.[0];
    if (email) {
      const existing = get(emails);
      const updated = new Map(existing);
      updated.set(id, email);
      emails.set(updated);
      return email;
    }
  } catch (e: any) {
    logger.error('jmap', `fetch email ${id} failed`, e);
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
  logger.info('jmap', 'searching', { query: text });
  try {
    const result: any = await logger.time('info', 'jmap', 'search', () =>
      invoke('search_emails', { text, limit: 50 })
    );
    logger.info('jmap', `search found ${result.ids?.length ?? 0} results`);

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

    const restoreMailbox = get(selectedMailboxId);
    return () => {
      if (restoreMailbox) {
        fetchEmailsForMailbox(restoreMailbox);
      }
    };
  } catch (e: any) {
    logger.error('jmap', 'search failed', e);
    addError(`Search failed: ${e}`);
    return null;
  }
}

// ── Mutations ──

export async function markAsRead(id: string) {
  try {
    await invoke('mark_seen', { id, seen: true });
    logger.debug('jmap', `marked as read: ${id}`);
    updateLocalEmail(id, (e) => ({ ...e, keywords: { ...e.keywords, $seen: true } }));
  } catch (e: any) {
    logger.error('jmap', `mark as read failed: ${id}`, e);
    addError(`Failed to mark as read: ${e}`);
  }
}

export async function markAsUnread(id: string) {
  try {
    await invoke('mark_seen', { id, seen: false });
    logger.debug('jmap', `marked as unread: ${id}`);
    updateLocalEmail(id, (e) => {
      const kw = { ...e.keywords };
      delete kw.$seen;
      return { ...e, keywords: kw };
    });
  } catch (e: any) {
    logger.error('jmap', `mark as unread failed: ${id}`, e);
    addError(`Failed to mark as unread: ${e}`);
  }
}

export async function toggleFlag(id: string, flagged: boolean) {
  const newValue = !flagged;
  try {
    await invoke('toggle_flagged', { id, value: newValue });
    logger.debug('jmap', `toggled flag: ${id} → ${newValue}`);
    updateLocalEmail(id, (e) => {
      const kw = { ...e.keywords };
      if (newValue) kw.$flagged = true; else delete kw.$flagged;
      return { ...e, keywords: kw };
    });
  } catch (e: any) {
    logger.error('jmap', `toggle flag failed: ${id}`, e);
    addError(`Failed to toggle flag: ${e}`);
  }
}

export async function moveToMailbox(id: string, toMailboxId: string) {
  try {
    await invoke('move_email', { id, toMailboxId });
    logger.info('jmap', `moved email ${id} → mailbox ${toMailboxId}`);
    updateLocalEmail(id, (e) => ({ ...e, mailboxIds: { [toMailboxId]: true } }));
    refreshMailboxes();
  } catch (e: any) {
    logger.error('jmap', `move email failed: ${id}`, e);
    addError(`Failed to move email: ${e}`);
  }
}

export async function deleteEmail(id: string) {
  try {
    await invoke('delete_email', { id });
    logger.info('jmap', `deleted email ${id}`);
    const newIds = get(emailIds).filter((eid: string) => eid !== id);
    emailIds.set(newIds);
    const newMap = new Map(get(emails));
    newMap.delete(id);
    emails.set(newMap);
    if (get(selectedEmailId) === id) selectedEmailId.set(newIds[0] ?? null);
    refreshMailboxes();
    addSuccess('Email deleted');
  } catch (e: any) {
    logger.error('jmap', `delete email failed: ${id}`, e);
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

  logger.info('jmap', 'sending email', {
    to: params.to,
    cc: params.cc,
    subject: params.subject,
  });

  try {
    await logger.time('info', 'jmap', 'send email', () =>
      invoke('send_email', {
        from: sess.username,
        to: params.to,
        cc: params.cc ?? null,
        bcc: params.bcc ?? null,
        subject: params.subject,
        bodyText: params.bodyText,
        bodyHtml: params.bodyHtml ?? null,
        replyToId: params.replyToId ?? null,
      })
    );
    addSuccess('Email sent');
  } catch (e: any) {
    logger.error('jmap', 'send email failed', e);
    addError(`Failed to send email: ${e}`);
    throw e;
  }
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

  const unlisten1 = listen<any>('jmap://mailboxes-changed', async (event) => {
    logger.debug('push', 'mailboxes changed', event.payload);
    await refreshMailboxes();
  });

  const unlisten2 = listen<any>('jmap://emails-changed', async (event) => {
    const { created, updated, destroyed } = event.payload as any;
    logger.debug('push', 'emails changed', {
      created: created?.length ?? 0,
      updated: updated?.length ?? 0,
      destroyed: destroyed?.length ?? 0,
    });

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
    logger.debug('sync', event.payload);
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
