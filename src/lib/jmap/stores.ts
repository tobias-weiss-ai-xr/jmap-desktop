/**
 * Reactive Svelte stores for JMAP data.
 *
 * The Rust backend pushes state changes; these stores are the frontend source of truth.
 */

import { writable, derived } from 'svelte/store';
import type { JMAPSession, Mailbox, Email } from './types.js';

// ── Session ──

export const session = writable<JMAPSession | null>(null);
export const connected = derived(session, ($s) => $s !== null);

// ── Selected Account ──

export const selectedAccountId = writable<string>('');
export const selectedMailboxId = writable<string | null>(null);
export const selectedEmailId = writable<string | null>(null);

// ── Mailboxes ──

export const mailboxes = writable<Mailbox[]>([]);
export const mailboxMap = derived(mailboxes, ($m) => {
  const map = new Map<string, Mailbox>();
  for (const mb of $m) {
    map.set(mb.id, mb);
  }
  return map;
});

// Derived: top-level mailboxes (no parent)
export const topLevelMailboxes = derived(mailboxes, ($m) =>
  $m
    .filter((mb) => mb.parentId === null || mb.parentId === undefined)
    .sort((a, b) => a.sortOrder - b.sortOrder)
);

// ── Email List ──

export const emailIds = writable<string[]>([]);
export const emails = writable<Map<string, Email>>(new Map());
export const emailQueryState = writable<string>('');
export const isLoadingEmails = writable(false);

// Current email
export const currentEmail = derived(
  [emails, selectedEmailId],
  ([$emails, $id]) => $id ? $emails.get($id) ?? null : null
);

// ── Sync State ──

export const isSyncing = writable(false);
export const lastSyncAt = writable<Date | null>(null);
export const syncStatus = writable<string>('disconnected'); // disconnected | syncing | synced | error
