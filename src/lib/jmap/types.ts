/**
 * JMAP TypeScript type definitions.
 *
 * Based on RFC 8620 (JMAP Core) and RFC 8621 (JMAP Mail).
 * @see https://jmap.io/spec-mail.html
 */

// --- Core JMAP Types ---

export interface JMAPRequest<A = unknown> {
  using: string[];
  methodCalls: JMAPMethodCall<A>[];
  createdIds?: Record<string, string>;
}

export interface JMAPMethodCall<A = unknown> {
  name: string;
  args: A;
  callId: string;
}

export interface JMAPResponse {
  sessionState: string;
  methodResponses: JMAPMethodResponse[];
  createdIds?: Record<string, string>;
}

export interface JMAPMethodResponse {
  name: string;
  args: unknown[];
  callId: string;
}

export interface JMAPError {
  type: string;
  status: number;
  detail: string;
}

// --- Session ---

export interface JMAPSession {
  capabilities: Record<string, Record<string, unknown>>;
  accounts: Record<string, JMAPAccount>;
  primaryAccounts: Record<string, string>;
  username: string;
  apiUrl: string;
  downloadUrl: string;
  uploadUrl: string;
  eventSourceUrl?: string;
  state: string;
}

export interface JMAPAccount {
  name: string;
  isPersonal: boolean;
  isReadOnly: boolean;
  accountCapabilities: Record<string, Record<string, unknown>>;
}

// --- Connection Settings ---

export interface ConnectionSettings {
  serverUrl: string;
  username: string;
  password: string;
  skipTlsVerify?: boolean;
}

// --- Mailbox (RFC 8621 §2) ---

export interface Mailbox {
  id: string;
  name: string;
  parentId?: string | null;
  role?: MailboxRole | null;
  sortOrder: number;
  totalEmails: number;
  unreadEmails: number;
  totalThreads: number;
  unreadThreads: number;
  myRights: MailboxRights;
  emailQuery?: string | null;
  isSubscribed: boolean;
}

export type MailboxRole =
  | 'all'
  | 'archive'
  | 'drafts'
  | 'flagged'
  | 'inbox'
  | 'junk'
  | 'sent'
  | 'trash';

export interface MailboxRights {
  mayReadItems: boolean;
  mayAddItems: boolean;
  mayRemoveItems: boolean;
  maySetSeen: boolean;
  maySetKeywords: boolean;
  mayCreateChild: boolean;
  mayRename: boolean;
  mayDelete: boolean;
  maySubmit: boolean;
}

// --- Email (RFC 8621 §4) ---

export interface Email {
  id: string;
  blobId: string;
  threadId: string;
  mailboxIds: Record<string, boolean>;
  keywords: Record<string, boolean>;
  from?: EmailAddress[];
  to?: EmailAddress[];
  cc?: EmailAddress[];
  bcc?: EmailAddress[];
  replyTo?: EmailAddress[];
  subject: string;
  sentAt: string;
  receivedAt: string;
  size: number;
  preview: string;
  hasAttachment: boolean;
  header?: Record<string, string[]>;
  bodyValues?: Record<string, BodyValue>;
  htmlBody?: EmailBodyPart[];
  textBody?: EmailBodyPart[];
  attachments?: EmailBodyPart[];
}

export interface EmailAddress {
  name: string;
  email: string;
}

export interface EmailAddressFilter {
  exactMatch: boolean;
}

export interface EmailBodyPart {
  partId: string;
  blobId: string;
  size: number;
  type: string;
  charset?: string;
  headers?: Record<string, string[]>;
  disposition?: string;
  cid?: string;
  language?: string;
  location?: string;
  subParts?: EmailBodyPart[];
  bodyValues?: Record<string, BodyValue>;
  name?: string;
  path?: string;
  link?: string;
}

export interface BodyValue {
  value: string;
  encoding: string;
  isTrusted: boolean;
}

// --- Thread (RFC 8621 §3) ---

export interface Thread {
  id: string;
  emailIds: string[];
}

// --- Filter & Sort (RFC 8621 §4.5) ---

export interface EmailFilterCondition {
  inMailbox?: string;
  before?: string;
  after?: string;
  subject?: string | null;
  from?: string | null;
  to?: string | null;
  cc?: string | null;
  hasKeyword?: string | null;
  notKeyword?: string | null;
  hasAttachment?: boolean | null;
  text?: string | null;
}

export type EmailFilter =
  | EmailFilterCondition
  | { operator: 'AND' | 'OR' | 'NOT'; conditions: EmailFilter[] };

export interface EmailComparator {
  property: string;
  isAscending?: boolean;
  collation?: string;
}

// --- Query ---

export interface QueryParams {
  accountId: string;
  filter: EmailFilter;
  sort: EmailComparator[];
  position?: number;
  anchor?: string;
  anchorOffset?: number;
  limit?: number;
}

export interface QueryResponse {
  accountId: string;
  queryState: string;
  canCalculateChanges: boolean;
  position: number;
  ids: string[];
  total?: number;
  limit?: number;
}

// --- Get ---

export interface GetParams {
  accountId: string;
  ids: string[] | null;
  properties?: string[] | null;
}

export interface GetResponse<T> {
  accountId: string;
  state: string;
  list: T[];
  notFound: string[];
}

// --- Set ---

export interface EmailSetParams {
  accountId: string;
  ifInState?: string;
  create?: Record<string, Partial<Email>>;
  update?: Record<string, Partial<Email>>;
  destroy?: string[];
}

export interface SetResponse<T> {
  accountId: string;
  oldState?: string;
  newState: string;
  created: Record<string, T>;
  updated: Record<string, T | null>;
  destroyed: Record<string, string | null>;
  notCreated: Record<string, JMAPSetError>;
  notUpdated: Record<string, JMAPSetError>;
  notDestroyed: Record<string, JMAPSetError>;
}

export interface JMAPSetError {
  type: string;
  description?: string;
  properties?: string[];
}

// --- Changes / Push ---

export interface ChangesParams {
  sinceState: string;
  maxChanges?: number;
}

export interface ChangesResponse {
  oldState: string;
  newState: string;
  hasMoreChanges: boolean;
  created: string[];
  updated: string[];
  destroyed: string[];
}
