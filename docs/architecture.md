# JMAP Desktop — Architecture & Development Roadmap

> How to build all 40 user stories with clean, maintainable architecture.
> Current state analysis → target architecture → phased development plan.

---

## Current State: What Works, What Doesn't Scale

### Architecture right now (flat, monolithic)

```
Frontend                              Backend (Rust)
──────────                            ──────────────
actions.ts ──→ invoke() ──→  commands.rs  ──→  session.rs
stores.ts  ←── window events ←──  (tauri::Emitter)
client.ts ──→ invoke() ──→  client.rs    ──→  JMAP server
types.ts
```

**What's good:**
- Works end-to-end: connect → sync → read → compose → send
- Background sync (poll + EventSource) with Tauri event emission
- Tracing/logging, password redaction, TLS skip
- Env var preconfiguration

**What breaks at scale (the 40 stories):**
- `actions.ts` is a 350-line god-module: connect, disconnect, fetch, mutate, sync, search — all in one file
- `stores.ts` is a flat bag of `writable()` — no domain grouping, no derived views, no computed queries
- `commands.rs` is 300 lines of procedural functions — no service layer, no abstraction
- `session.rs` is a 500-line God struct — holds HTTP client, credentials, session, sync handles, email/mailbox state in 8 Mutex fields
- No offline cache — everything disappears on disconnect
- No command pattern — no undo/redo possible
- No dependency injection — everything reaches into global `Arc<JmapSessionManager>`
- No thread model — email IDs stored in flat array, no thread view
- Components reach into stores directly — no clear data flow
- No shared UI components — Button, Modal, Dropdown duplicated across 5 files

---

## Target Architecture

### Layer diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────────┐  │
│  │ AppShell │ │ Sidebar │ │MailList │ │MailView │ │  Compose     │  │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └──────┬───────┘  │
│       └───────────┴───────────┴───────────┴──────────────┘          │
│                         ↕ reads/writes                               │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              SHARED UI COMPONENTS                             │   │
│  │  Button · Modal · ContextMenu · Dropdown · CommandPalette     │   │
│  │  ConfirmDialog · Toast · Skeleton · EmptyState               │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                ↕
┌─────────────────────────────────────────────────────────────────────┐
│                      APPLICATION LAYER                              │
│  ┌─────────────────┐  ┌──────────────────┐  ┌─────────────────┐   │
│  │ ConnectionService │  │  EmailService    │  │ MailboxService  │   │
│  │ connect/disconnect│  │ fetch/mutate/search│ │ list/create/    │   │
│  │ reconnect        │  │ thread · draft   │  │ rename/delete   │   │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬────────┘   │
│           │                     │                     │             │
│  ┌────────┴─────────┐  ┌───────┴──────────┐  ┌──────┴───────────┐   │
│  │  ComposeService   │  │  SyncService     │  │ SearchService   │   │
│  │ draft · send      │  │ poll · push      │  │ basic · filter  │   │
│  │ schedule · reply  │  │ offline queue   │  │ saved queries   │   │
│  └────────┬─────────┘  └───────┬──────────┘  └──────┬───────────┘   │
│           └────────────────────┴────────────────────┘              │
│                                ↕                                    │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │               COMMAND QUEUE (undo/redo + offline)            │   │
│  │  Enqueue → Execute → Track → Undo ↔ Redo                     │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                ↕
┌─────────────────────────────────────────────────────────────────────┐
│                        STATE LAYER                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │connection│ │ mailboxes│ │  emails  │ │  compose │   + derived  │
│  │  store   │ │  store   │ │  store   │ │  store   │   views      │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │  search  │ │  sync    │ │    ui    │ │ settings │              │
│  │  store   │ │  store   │ │  store   │ │  store   │              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
│                                ↕                                    │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    EVENT BUS                                  │   │
│  │  Typed events: email-changed, mailbox-changed, sync-status   │   │
│  │  Used by: push events → state updates → UI reactions          │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                ↕
┌─────────────────────────────────────────────────────────────────────┐
│                        DATA LAYER                                    │
│  ┌─────────────────────┐    ┌──────────────────────────────────┐  │
│  │   JMAP API Client    │    │       Offline Cache (SQLite)       │  │
│  │   (via Tauri IPC)    │    │  email_repo · mailbox_repo         │  │
│  │  connect · query      │    │  thread_repo · identity_repo        │  │
│  │  get · set · changes  │    │  schema · migrations               │  │
│  └──────────┬──────────┘    └──────────────┬────────────────────┘  │
└─────────────┼──────────────────────────────┼───────────────────────┘
              ↕                              ↕
┌─────────────────────────────────────────────────────────────────────┐
│                     TAURI COMMANDS (thin adapters)                  │
│  connection.rs · email.rs · mailbox.rs · search.rs · attachment.rs   │
└─────────────────────────────────────────────────────────────────────┘
              ↕
┌─────────────────────────────────────────────────────────────────────┐
│                   RUST SERVICES                                     │
│  ┌───────────────┐ ┌───────────────┐ ┌────────────────────────┐     │
│  │ ConnectionMgr │ │  SyncEngine   │ │     CacheLayer         │     │
│  │ HTTP · auth   │ │ poll · EventS │ │ SQLite · read/write    │     │
│  │ reconnect     │ │ backoff · delta│ │ cache-first reads      │     │
│  └───────┬───────┘ └───────┬───────┘ └───────────┬────────────┘     │
└───────────┼─────────────────┼─────────────────────┼──────────────────┘
            ↕                 ↕                     ↕
┌─────────────────────────────────────────────────────────────────────┐
│              JMAP SERVER (Stalwart, Fastmail, Cyrus, etc.)          │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Architectural Principles

1. **Unidirectional data flow**: State → UI → Events → Services → State
2. **Service layer owns business logic**: Stores are dumb containers; services orchestrate
3. **Cache-first reads**: SQLite for speed, JMAP for truth, sync for consistency
4. **Command queue for mutations**: Every write goes through a queue → enables undo + offline
5. **Event bus for decoupling**: Push events, sync status, UI triggers — all typed
6. **Thin command adapters**: Rust `#[tauri::command]` functions are 5-line adapters
7. **Domain types separate from wire types**: Clean models vs JMAP JSON shapes

---

## Frontend Target Structure

```
src/lib/
├── api/                          # Pure data access — no state mutation
│   ├── jmap.ts                    # invoke() wrappers (typed, no side effects)
│   └── types.ts                   # Wire types (JMAP RFC 8620/8621)
│
├── models/                       # Domain models — clean, no JMAP jargon
│   ├── email.ts                  # Email, Thread, Attachment
│   ├── mailbox.ts                # Mailbox, FolderTree
│   ├── identity.ts               # Identity, Account
│   ├── contact.ts                # Contact, Recipient
│   └── search.ts                 # SearchQuery, Filter, SavedSearch
│
├── services/                     # Business logic — orchestrates api + stores
│   ├── connection.service.ts      # connect, disconnect, reconnect, credentials
│   ├── email.service.ts          # fetch, thread view, mutations, auto-read
│   ├── mailbox.service.ts        # list, create, rename, delete, counts
│   ├── compose.service.ts        # draft, send, schedule, reply, forward
│   ├── search.service.ts         # search, advanced filter, saved queries
│   ├── sync.service.ts           # push event handler, change processor
│   └── undo.service.ts           # command queue, undo/redo, offline queue
│
├── stores/                       # Reactive state — dumb containers
│   ├── connection.store.ts       # session, connected, error, reconnecting
│   ├── mailboxes.store.ts        # list, map, tree, selected
│   ├── emails.store.ts           # list, map, thread view, selected
│   ├── compose.store.ts          # mode, draft state, sending status
│   ├── search.store.ts           # query, results, filters, saved queries
│   ├── sync.store.ts             # status, lastSyncAt, offline state
│   ├── ui.store.ts               # theme, pane sizes, sidebar collapsed, shortcuts
│   └── notifications.store.ts    # desktop notification permissions
│
├── commands/                     # Command pattern — serializable actions
│   ├── command.ts                # Command interface, CommandQueue
│   ├── email.commands.ts          # DeleteEmail, MoveEmail, FlagEmail, MarkRead
│   └── mailbox.commands.ts       # CreateMailbox, RenameMailbox, EmptyTrash
│
├── events/                       # Typed event bus
│   ├── event-bus.ts             # TypedEventBus<E>, subscribe, emit
│   └── types.ts                  # All event type definitions
│
├── components/                   # UI components
│   ├── layout/
│   │   ├── AppShell.svelte       # 3-pane container + responsive breakpoints
│   │   └── ResizablePane.svelte  # Drag-to-resize divider
│   ├── sidebar/
│   │   ├── Sidebar.svelte        # Account switcher, folders, labels, search
│   │   ├── FolderTree.svelte     # Recursive mailbox tree with unread counts
│   │   └── AccountSwitcher.svelte
│   ├── mail/
│   │   ├── MailList.svelte       # Virtual-scrolled email list
│   │   ├── MailItem.svelte       # Single email row (reusable, accepts props)
│   │   ├── MailView.svelte       # Email reading pane
│   │   ├── MailHeader.svelte     # From, To, Date, Subject, actions
│   │   ├── MailBody.svelte       # HTML/text renderer, image blocking
│   │   ├── ThreadView.svelte     # Stacked conversation messages
│   │   ├── AttachmentBar.svelte  # File list with download/preview
│   │   └── InlineReply.svelte   # Quick reply at bottom of mail view
│   ├── compose/
│   │   ├── Compose.svelte        # Main compose form (reused for reply/forward)
│   │   ├── RecipientField.svelte # To/Cc/Bcc with autocomplete chips
│   │   ├── AttachmentUploader.svelte
│   │   └── ScheduleSend.svelte  # Date/time picker
│   ├── common/
│   │   ├── Button.svelte
│   │   ├── Modal.svelte
│   │   ├── ContextMenu.svelte
│   │   ├── Dropdown.svelte
│   │   ├── CommandPalette.svelte  # Ctrl+K fuzzy search
│   │   ├── ConfirmDialog.svelte
│   │   ├── Toast.svelte
│   │   ├── Skeleton.svelte
│   │   ├── EmptyState.svelte
│   │   ├── Badge.svelte
│   │   ├── Tooltip.svelte
│   │   └── ProgressBar.svelte
│   └── settings/
│       ├── SettingsPage.svelte
│       ├── AccountSettings.svelte
│       ├── SignatureEditor.svelte
│       ├── RulesEditor.svelte
│       └── ShortcutHelp.svelte
│
├── hooks/                        # Composable logic
│   ├── use-keyboard.ts           # Global keyboard shortcut registration
│   ├── use-resize.ts             # Pane resize observer
│   ├── use-drag-drop.ts          # Drag-and-drop handlers
│   └── use-online-status.ts      # Network connectivity detection
│
├── utils/
│   ├── format.ts                 # Date, address, size formatting
│   ├── sanitize.ts              # DOMPurify wrapper, link safety
│   └── logger.ts
│
└── app.css                       # CSS variables, theme tokens
```

---

## Rust Backend Target Structure

```
src-tauri/src/
├── main.rs                       # Entry point
├── lib.rs                        # Tauri builder, managed state
├── error.rs                      # AppError enum
│
├── commands/                     # Thin Tauri command adapters (~5 lines each)
│   ├── mod.rs
│   ├── connection.rs             # connect, disconnect, get_session
│   ├── email.rs                  # query, get, set keywords, move, delete, send
│   ├── mailbox.rs                # get, create, rename, delete
│   ├── thread.rs                 # get threads
│   ├── search.rs                 # search emails
│   ├── attachment.rs             # upload, download
│   ├── identity.rs               # get/set identities
│   └── cache.rs                  # cache stats, vacuum
│
├── services/                     # Business logic
│   ├── mod.rs
│   ├── connection.rs             # Session discovery, URL rewriting, auth, reconnect
│   ├── email.rs                  # Email CRUD, threading logic, keyword management
│   ├── mailbox.rs                # Mailbox tree operations
│   ├── sync.rs                   # Poll engine, EventSource, change processing
│   ├── attachment.rs             # Upload/download via JMAP binary endpoints
│   ├── identity.rs               # Identity management, signature storage
│   └── cache.rs                  # SQLite operations, cache-first reads
│
├── protocol/                     # JMAP RFC implementation (stateless)
│   ├── mod.rs
│   ├── client.rs                 # HTTP request/response, basic auth, Host header
│   ├── types.rs                  # JMAP request/response structs
│   ├── session.rs                # Session resource discovery
│   └── methods/
│       ├── mod.rs
│       ├── email.rs              # Email/get, Email/set, Email/query, Email/changes
│       ├── mailbox.rs            # Mailbox/get, Mailbox/set, Mailbox/changes
│       └── thread.rs             # Thread/get
│
├── cache/                        # SQLite persistence layer
│   ├── mod.rs
│   ├── db.rs                     # Connection pool, pool size, WAL mode
│   ├── schema.rs                 # CREATE TABLE statements, migrations
│   ├── email_repo.rs             # Read/write/search emails
│   ├── mailbox_repo.rs           # Read/write mailboxes
│   ├── thread_repo.rs            # Read/write thread aggregates
│   └── identity_repo.rs          # Read/write identities/signatures
│
├── models/                       # Domain models (separate from wire types)
│   ├── mod.rs
│   ├── email.rs                  # Clean Email struct (no serde jargon)
│   ├── mailbox.rs
│   ├── thread.rs
│   └── identity.rs
│
└── types.rs                      # Shared types for command args/results
```

---

## Key Design Patterns

### 1. Cache-First Reads (Offline Foundation)

```
User clicks mailbox
        │
        ▼
┌─ EmailService.fetchForMailbox(id) ─────────────────┐
│                                                     │
│  1. Read from SQLite cache (instant, 0ms)           │
│     → Return cached emails + "stale" flag           │
│     → UI renders immediately                        │
│                                                     │
│  2. Fire JMAP query in background                   │
│     → Get new/changed/destroyed IDs                 │
│     → Fetch changed email bodies                    │
│     → Update SQLite cache                           │
│     → Emit "emails-changed" event                   │
│     → UI re-renders with fresh data                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Why:** This is the single biggest architectural change needed.
Without it, offline is impossible and every mailbox switch is a network round-trip.

### 2. Command Queue (Undo + Offline)

```typescript
// Every mutation goes through the queue
interface Command {
  id: string;
  type: 'delete' | 'move' | 'flag' | 'mark-read' | 'send';
  payload: any;
  timestamp: number;
  inverse?: Command;          // For undo
  status: 'pending' | 'applied' | 'undone' | 'failed';
}

class CommandQueue {
  private queue: Command[] = [];
  private undoStack: Command[] = [];

  execute(cmd: Command): Promise<void> {
    // 1. Generate inverse command (for undo)
    cmd.inverse = this.generateInverse(cmd);
    // 2. Apply optimistically to local state
    this.applyOptimistic(cmd);
    // 3. Send to backend (or queue if offline)
    if (this.online) {
      await this.sendToBackend(cmd);
      cmd.status = 'applied';
    } else {
      cmd.status = 'pending'; // queued for later
    }
    // 4. Push to undo stack
    this.undoStack.push(cmd);
  }

  async undo(): Promise<void> {
    const cmd = this.undoStack.pop();
    if (!cmd?.inverse) return;
    await this.execute(cmd.inverse);
    cmd.status = 'undone';
  }

  async syncPending(): Promise<void> {
    // Called on reconnect — flushes offline queue
  }
}
```

**Stories enabled:** Undo/redo (#23), Offline (#18), Bulk actions (#21)

### 3. Thread Model

```typescript
// Currently: flat emailIds: string[]
// Target: threads grouped by threadId

interface ThreadViewModel {
  threadId: string;
  emailIds: string[];
  subject: string;           // Latest subject
  participants: string[];    // Unique senders across thread
  lastMessageAt: string;    // Most recent receivedAt
  preview: string;           // Latest email preview
  isUnread: boolean;         // Any email in thread is unread
  isFlagged: boolean;        // Any email is flagged
  hasAttachment: boolean;    // Any email has attachment
  messageCount: number;
}
```

The `emails.store.ts` maintains both:
- `emailMap: Map<string, Email>` — raw email cache
- `threadViews: derived(emailMap, sort)` — computed thread aggregates

**Stories enabled:** Threading (#11), Conversation view (#4.4)

### 4. Event Bus

```typescript
// Typed events replace window.dispatchEvent / tauri::listen soup
type AppEvent =
  | { type: 'emails-changed'; payload: { created: string[]; updated: string[]; destroyed: string[] } }
  | { type: 'mailboxes-changed'; payload: { created: string[]; updated: string[]; destroyed: string[] } }
  | { type: 'sync-status'; payload: 'syncing' | 'synced' | 'push-connected' | 'offline' }
  | { type: 'command-executed'; payload: Command }
  | { type: 'command-undone'; payload: Command }
  | { type: 'compose:open'; payload: { mode: 'new' | 'reply' | 'forward'; target?: Email } }
  | { type: 'connection:lost' }
  | { type: 'connection:restored' };

class EventBus {
  private listeners = new Map<string, Set<Function>>();
  on<E extends AppEvent['type']>(type: E, handler: (e: Extract<AppEvent, { type: E }>) => void): () => void;
  emit<E extends AppEvent>(event: E): void;
}
```

**Stories enabled:** All cross-component communication, push events, notifications

### 5. Service Layer (Frontend)

```typescript
// services/email.service.ts — owns the business logic
class EmailService {
  constructor(
    private api: JmapApi,           // Pure data access
    private stores: EmailStores,    // State containers
    private events: EventBus,        // Cross-component communication
    private commands: CommandQueue,  // Mutation tracking
  ) {}

  async fetchForMailbox(mailboxId: string) {
    this.stores.loading.set(true);
    try {
      // API call → update store → emit event
      const result = await this.api.queryEmails({ inMailbox: mailboxId }, sort, limit);
      this.stores.ids.set(result.ids);

      // Fetch full bodies for visible window
      const emails = await this.api.getEmails(result.ids);
      this.stores.map.merge(emails);
    } finally {
      this.stores.loading.set(false);
    }
  }

  async delete(emailId: string) {
    // Goes through command queue for undo support
    await this.commands.execute({
      type: 'delete',
      payload: { emailId },
    });
  }
}
```

**Key rule:** Services are the *only* code that writes to stores. Components never call `stores.xxx.set()` directly.

### 6. Rust Service Architecture

```rust
// services/connection.rs — owns connection lifecycle
pub struct ConnectionManager {
    client: Option<reqwest::Client>,
    session: Option<JmapSession>,
    cache: CacheLayer,            // SQLite
}

impl ConnectionManager {
    pub async fn connect(&mut self, settings: ConnectionSettings) -> Result<Session> {
        let client = self.build_client(&settings)?;
        let session = self.discover_session(&client, &settings).await?;
        self.session = Some(session.clone());
        self.client = Some(client);
        // Warm cache: fetch all mailboxes + recent emails
        self.cache.warm(&session).await?;
        Ok(session)
    }
}

// services/sync.rs — owns sync lifecycle
pub struct SyncEngine {
    poll_interval: Duration,
    es_backoff: ExponentialBackoff,
    cache: CacheLayer,
}

impl SyncEngine {
    pub async fn poll_and_diff(&self, conn: &ConnectionManager) -> SyncResult {
        let changes = conn.get_email_changes(&self.last_state).await?;
        let delta = self.cache.apply_email_changes(&changes).await?;
        Ok(delta)  // Returns what changed for event emission
    }
}
```

---

## Dependency Graph: What Blocks What

```
                          ┌─────────────┐
                          │ SQLite cache │  ← Foundation: everything needs this
                          └──────┬──────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
     ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐
     │ Cache-first  │  │ Offline queue│  │ Thread model     │
     │ reads        │  │ (CmdQueue)   │  │ (group emails)   │
     └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘
            │                  │                   │
            └──────────┬───────┴───────────────────┘
                       ▼
              ┌──────────────────┐
              │ Service layer    │  ← Business logic in services, not components
              │ (TS + Rust)      │
              └────────┬─────────┘
                       │
     ┌─────────────────┼─────────────────┐
     ▼                 ▼                 ▼
┌───────────┐   ┌────────────┐   ┌────────────┐
│ Advanced   │   │ Compose    │   │ Search +   │
│ threading  │   │ features   │   │ filters    │
└─────┬─────┘   └─────┬──────┘   └─────┬──────┘
      │               │                │
      └───────────────┼────────────────┘
                      ▼
              ┌──────────────────┐
              │ Power features   │  ← Snooze, rules, tabs, multi-account
              │ (advanced UX)    │
              └──────────────────┘
```

---

## Phased Development Plan

### Phase 0 — Refactor Foundation (pre-req for everything)

**Goal:** Restructure flat code into layered architecture without changing behavior.

| Task | What changes |
|------|-------------|
| Split `actions.ts` → services/ | `email.service.ts`, `mailbox.service.ts`, `connection.service.ts`, `compose.service.ts` |
| Split `stores.ts` → stores/ | One file per domain, clear exports |
| Extract shared UI components | `Button.svelte`, `Modal.svelte`, `ContextMenu.svelte`, `EmptyState.svelte`, `Badge.svelte` |
| Add event bus | Replace `window.dispatchEvent` + `tauri::listen` soup |
| Split `commands.rs` → commands/ | One file per domain, each < 100 lines |
| Split `session.rs` → protocol/ + services/ | Connection logic, protocol types, sync engine separated |

**Deliverable:** Same app, same behavior, clean architecture. No new features.

**Estimated effort:** 1–2 days

---

### Phase 1 — Core Email (Stories 1–10)

**Goal:** Rock-solid core email experience.

| Story | Key implementation |
|-------|-------------------|
| 1. Connect | Already done; add reconnect on network restore |
| 2. Compose | Already done; add HTML compose (rich text toolbar) |
| 3. Reply/Forward | Add Reply All (`Shift+R`); already have reply + forward |
| 4. Read/View | Add attachment bar (#4.2); headers panel (#4.3); remote image blocking (#4.5) |
| 5. Navigation | Already done; add sorting (#5.3) |
| 6. Flag | Already done; add labels/tags with SQLite columns |
| 7. Read/Unread | Already done; add `m` toggle |
| 8. Delete/Archive | Already done; add Archive button (`e` key) |
| 9. Move | Already done; add drag-to-folder (#9.2) |
| 10. Search | Already done; add advanced filters (#10.2); search-as-you-type (#10.3) |

**Deliverable:** Usable daily email client.

**Estimated effort:** 3–5 days

---

### Phase 2 — Offline & Cache (Stories 18, 12, 13, 23)

**Goal:** Offline-capable, with undo and attachments.

| Story | Key implementation |
|-------|-------------------|
| 18. Offline | **SQLite cache layer** (biggest single task): schema, repos, cache-first reads, offline banner |
| 12. Drafts | Auto-save to server every 3s; resume from Drafts folder |
| 13. Attachments | JMAP upload/download via `uploadUrl`/`downloadUrl`; drag-to-compose |
| 23. Undo/Redo | **Command queue**: optimistic updates, inverse commands, toast undo |

**Deliverable:** Works without internet. Send failures don't lose data.

**Estimated effort:** 4–6 days

---

### Phase 3 — Threading & UX Polish (Stories 4.4, 11, 14, 16, 17, 19, 20, 22, 25, 26, 27)

**Goal:** Threaded conversations, polished UX.

| Story | Key implementation |
|-------|-------------------|
| 11. Threading | Thread model, thread view, collapse/expand, mute |
| 14. Signatures | Per-identity signatures with `[Name]`/`[Email]` variables |
| 16. Shortcuts | Help overlay (`?`), all shortcuts from spec |
| 17. Notifications | Desktop notifications API, tray badge (tauri-plugin-notification) |
| 19. Folders | Create/rename/delete; nested hierarchy |
| 20. Auto-complete | Recent recipients + address book search in To field |
| 22. Context menus | Right-click on email, folder, attachment |
| 25. Resizable panes | Drag dividers; persist to localStorage |
| 26. Themes | Light theme CSS; `prefers-color-scheme` support |
| 27. Print | `window.print()` with formatted content |

**Deliverable:** Feature-complete for single-account use.

**Estimated effort:** 4–6 days

---

### Phase 4 — Power Features (Stories 15, 21, 24, 28–37)

**Goal:** Power-user features.

| Story | Key implementation |
|-------|-------------------|
| 15. Multiple identities | Identity picker in compose; auto-detect on reply |
| 21. Bulk actions | Checkbox mode, shift-select, bulk action bar |
| 24. Drag & drop | Email → folder, file → compose |
| 28. Link safety | Open external in browser; hover tooltip; mismatch warning |
| 29. Image blocking | Already done in Phase 1; add per-sender trust |
| 30. Snooze | `setTimeout` or cron; snoozed virtual folder |
| 31. Inline reply | Quick reply textarea at bottom of MailView |
| 32. Keyboard-only | Verify all sequences from spec table |
| 33. Accessibility | Audit: focus rings, aria labels, reduced motion, heading levels |
| 34. Mobile | Responsive breakpoints; collapsed sidebar; stacked layout |
| 35. Tabbed viewing | Tab bar in MailView; `Shift+Enter` open in tab |
| 36. Filter rules | Rule editor UI; client-side sieve evaluation |
| 37. Quick actions | Hover reveal buttons on MailItem; swipe on touch |

**Deliverable:** Competitive with Thunderbird feature set.

**Estimated effort:** 5–8 days

---

### Phase 5 — Advanced (Stories 2.3, 34, 36, 38–40)

**Goal:** Advanced and nice-to-have features.

| Story | Key implementation |
|-------|-------------------|
| 2.3. Schedule send | Date/time picker; queue + cron |
| 34. Mobile gestures | Touch event handlers; swipe actions |
| 36. Rules engine | Persisted rule set; apply on incoming during sync |
| 38. Headers | Raw header panel; SPF/DKIM badges |
| 39. Auto-reply | JMAP VacationResponse or sieve |
| 40. Quota | Parse `accountCapabilities` quota; usage bar |

**Deliverable:** Feature-complete modern email client.

**Estimated effort:** 3–5 days

---

## Migration Strategy: How to Refactor Without Breaking Things

The current code works. The biggest risk is a "big bang" refactor that breaks everything.

### Approach: Strangler Fig Pattern

1. **Write new files alongside old ones** — don't delete `actions.ts` until `services/` is verified
2. **Migrate one service at a time** — start with `connection.service.ts` (simplest), then `mailbox.service.ts`, then `email.service.ts`
3. **Each migration is a self-contained PR** — grep for old imports, update to new service
4. **Same Tauri commands** — backend stays unchanged during frontend refactor
5. **Test each phase end-to-end** — connect → sync → read → compose → send

### Example: Migrating actions.ts → services/

```typescript
// Step 1: Create new service (doesn't touch old code)
// services/connection.service.ts
export class ConnectionService {
  async connect(settings: ConnectionSettings) { /* ... */ }
  async disconnect() { /* ... */ }
}

// Step 2: In +page.svelte, import new service instead of actions.ts
// Before:  import { connect } from '$lib/jmap/actions.js';
// After:   import { connectionService } from '$lib/jmap/services/connection.service.js';

// Step 3: Once all imports migrated, delete actions.ts
```

### Example: Adding SQLite cache

```rust
// Step 1: Add rusqlite dependency
// Step 2: Create cache/ module with schema + repos
// Step 3: Modify email.rs command to read from cache first, fetch from JMAP in background
// Step 4: Sync engine writes to cache on every poll
// Frontend doesn't change at all — same Tauri commands, faster responses
```

---

## Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Offline cache | `rusqlite` (Rust) | Zero-copy reads, WAL mode, runs in Tauri process (no separate DB) |
| Rich text compose | ProseMirror or TipTap | Well-maintained, extensible, Svelte-compatible |
| Virtual scrolling | Custom `IntersectionObserver` | No dependency needed for ~100–1000 items; svelte-virtual-list if needed |
| Drag & drop | HTML5 DnD API + pointer events | Native, no dependency |
| Context menu | Custom `<div>` positioned at cursor | Styling matches theme; native contextmenu suppressed |
| Desktop notifications | `tauri-plugin-notification` | Official plugin, handles OS differences |
| System tray | `tauri-plugin-tray` | Official plugin, badge support |
| Theming | CSS custom properties + `prefers-color-scheme` | Already have `--bg-primary` etc.; just add light theme values |
| Autocomplete | Custom dropdown (no dependency) | Simple fuzz search over recent contacts + address book |
| Keyboard help | Custom modal with `?` trigger | No dependency needed |

---

## What to Build Next (Recommended Order)

```
Phase 0: Refactor (1–2 days)
  ├── Extract services from actions.ts
  ├── Extract stores
  ├── Extract shared UI components
  └── Add event bus

Phase 1: Core email (3–5 days)
  ├── Reply All
  ├── Attachment upload/download
  ├── Archive action
  ├── Sort by column
  ├── Advanced search filters
  └── Remote image blocking

Phase 2: Offline foundation (4–6 days)
  ├── SQLite cache (Rust)
  ├── Cache-first reads
  ├── Command queue
  ├── Undo/redo
  ├── Auto-save drafts
  └── Offline banner

Phase 3: Threading & polish (4–6 days)
  ├── Thread model + view
  ├── Signatures
  ├── Context menus
  ├── Resizable panes
  ├── Light theme
  ├── Desktop notifications
  ├── Auto-complete
  └── Folder management
```

**Total estimated: 12–19 days** to reach a feature-complete, offline-capable, threaded email client.
