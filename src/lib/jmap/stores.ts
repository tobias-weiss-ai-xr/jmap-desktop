/**
 * Reactive Svelte stores for JMAP data.
 * Uses classic writable stores (works in both .svelte and .ts files).
 */

import { writable, derived, get } from 'svelte/store';
import type { JMAPSession, Mailbox, Email } from './types.js';

// ── Session ──
export const session = writable<JMAPSession | null>(null);
export const connected = derived(session, ($s) => $s !== null);

// ── Selected ──
export const selectedAccountId = writable<string>('');
export const selectedMailboxId = writable<string | null>(null);
export const selectedEmailId = writable<string | null>(null);

// ── Mailboxes ──
export const mailboxes = writable<Mailbox[]>([]);
export const mailboxMap = derived(mailboxes, ($m) => {
  const map = new Map<string, Mailbox>();
  for (const mb of $m) map.set(mb.id, mb);
  return map;
});
export const topLevelMailboxes = derived(mailboxes, ($m) =>
  $m.filter((mb) => mb.parentId === null || mb.parentId === undefined)
    .sort((a, b) => a.sortOrder - b.sortOrder)
);

// ── Email List ──
export const emailIds = writable<string[]>([]);
export const emails = writable<Map<string, Email>>(new Map());
export const emailQueryState = writable<string>('');
export const isLoadingEmails = writable(false);

export const currentEmail = derived(
  [emails, selectedEmailId],
  ([$emails, $id]) => $id ? $emails.get($id) ?? null : null
);

// ── Sync ──
export const isSyncing = writable(false);
export const lastSyncAt = writable<Date | null>(null);
export const syncStatus = writable<string>('disconnected');
