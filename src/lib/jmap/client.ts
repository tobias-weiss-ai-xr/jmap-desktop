/**
 * JMAP HTTP client — communicates with Rust backend via Tauri commands.
 *
 * All JMAP requests go through Tauri IPC → Rust backend → JMAP server.
 * This module provides a typed, async interface for the frontend.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  JMAPSession,
  Mailbox,
  Email,
  Thread,
  EmailFilter,
  EmailComparator,
  QueryResponse,
  GetResponse,
  SetResponse,
  ChangesResponse,
} from './types.js';

// ── Connection Settings ──

export interface ConnectionSettings {
  serverUrl: string;
  username: string;
  password: string;
}

// ── Tauri Command Wrappers ──

export async function connect(settings: ConnectionSettings): Promise<JMAPSession> {
  return invoke<JMAPSession>('connect_jmap', { settings });
}

export async function disconnect(): Promise<void> {
  return invoke('disconnect_jmap');
}

export async function getSession(): Promise<JMAPSession> {
  return invoke<JMAPSession>('get_session');
}

// ── Mailboxes ──

export async function getMailboxes(): Promise<Mailbox[]> {
  return invoke<Mailbox[]>('get_mailboxes');
}

// ── Emails ──

export async function queryEmails(
  filter: EmailFilter,
  sort: EmailComparator[],
  limit: number = 50,
  position: number = 0,
  anchor?: string
): Promise<QueryResponse> {
  return invoke<QueryResponse>('query_emails', { filter, sort, limit, position, anchor });
}

export async function getEmails(ids: string[], properties?: string[]): Promise<GetResponse<Email>> {
  return invoke<GetResponse<Email>>('get_emails', { ids, properties });
}

export async function getEmail(id: string): Promise<Email | null> {
  const result = await getEmails([id]);
  return result.list[0] ?? null;
}

// ── Threads ──

export async function getThreads(ids: string[]): Promise<GetResponse<Thread>> {
  return invoke<GetResponse<Thread>>('get_threads', { ids });
}

// ── Changes (for push/sync) ──

export async function getEmailChanges(sinceState: string): Promise<ChangesResponse> {
  return invoke<ChangesResponse>('get_email_changes', { sinceState });
}

export async function getMailboxChanges(sinceState: string): Promise<ChangesResponse> {
  return invoke<ChangesResponse>('get_mailbox_changes', { sinceState });
}

// ── Set (send, move, flag) ──

export async function setEmailKeywords(
  id: string,
  keywords: Record<string, boolean>
): Promise<SetResponse<Email>> {
  return invoke<SetResponse<Email>>('set_email_keywords', { id, keywords });
}

export async function moveEmail(id: string, toMailboxId: string): Promise<SetResponse<Email>> {
  return invoke<SetResponse<Email>>('move_email', { id, toMailboxId });
}

export async function deleteEmail(id: string): Promise<SetResponse<Email>> {
  return invoke<SetResponse<Email>>('delete_email', { id });
}
