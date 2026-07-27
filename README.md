# jmap-desktop

Native JMAP desktop email client for Linux, built with **Tauri 2 + Rust + TypeScript**.

> Think Thunderbird, but for JMAP.

## Status

🚧 **Early development** — scaffolding complete, core JMAP protocol implemented.

## Goals

- Native Linux desktop client for JMAP email
- Works with any JMAP server (Stalwart, Fastmail, Cyrus, etc.)
- Background sync & push notifications
- Offline support
- Contacts & calendar (CardDAV/CalDAV)
- Small footprint, fast startup

## Architecture

```
Tauri 2 Shell
├── Rust Backend (JMAP session, HTTP client, background sync, system integration)
└── Web Frontend (SvelteKit + Svelte 5, reactive stores, 3-pane mail layout)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Backend | Rust + reqwest (native TLS, JSON/HTTP JMAP client) |
| Frontend | TypeScript + [SvelteKit](https://kit.svelte.dev) + [Svelte 5](https://svelte.dev) |
| Styling | CSS custom properties (Tokyo Night-inspired dark theme) |
| JMAP protocol | [RFC 8620](https://jmap.io/spec-core.html) (Core) + [RFC 8621](https://jmap.io/spec-mail.html) (Mail) |

## Project Structure

```
jmap-desktop/
├── src/                          # SvelteKit frontend
│   ├── lib/
│   │   ├── components/           # UI components
│   │   │   ├── AppShell.svelte    # Main layout shell
│   │   │   ├── Sidebar.svelte     # Mailbox tree with unread counts
│   │   │   ├── MailList.svelte    # Email list with virtual scroll
│   │   │   └── MailView.svelte    # Email reader with HTML/text body
│   │   └── jmap/                 # JMAP client layer
│   │       ├── client.ts          # Tauri IPC command wrappers
│   │       ├── types.ts           # Full JMAP TypeScript types
│   │       └── stores.ts          # Svelte reactive stores
│   └── routes/
│       ├── +page.svelte           # Main 3-pane mail view
│       └── settings/              # Account connection settings
├── src-tauri/                     # Rust backend
│   └── src/
│       ├── commands.rs            # Tauri commands (exposed to frontend)
│       ├── error.rs               # Error types
│       └── jmap/
│           ├── session.rs         # JMAP session discovery & HTTP client
│           └── client.rs           # Mailbox/get, Email/query, Email/get
├── package.json
└── Cargo.toml
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 18+
- Linux: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev libdbus-1-dev`

### Run

```bash
npm install
npm run tauri dev
```

### Check

```bash
npm run check          # TypeScript/Svelte type checking
cargo check            # Rust compilation check (from src-tauri/)
```

## What's Implemented

- ✅ Tauri 2 + SvelteKit 5 project structure
- ✅ JMAP session discovery (`/.well-known/jmap`)
- ✅ Basic auth (username/password)
- ✅ `Mailbox/get` — fetch all mailboxes
- ✅ `Email/query` — query emails by filter
- ✅ `Email/get` — fetch full email with body
- ✅ 3-pane mail UI (sidebar → mail list → mail viewer)
- ✅ Dark theme with Tokyo Night-inspired palette
- ✅ Settings page for account connection
- ✅ TypeScript JMAP type definitions (RFC 8620/8621)

## TODO

- [ ] `Email/set` — send, move, flag, delete
- [ ] `Mailbox/changes` + `Email/changes` — incremental sync
- [ ] EventSource push (real-time updates)
- [ ] Local SQLite/offline cache
- [ ] Thread view (collapse/expand)
- [ ] Compose with rich text (Markdown or ProseMirror)
- [ ] Search with `Email/query` filters
- [ ] Attachments — upload/download via JMAP uploadUrl/downloadUrl
- [ ] Contact support (CardDAV or JMAP contacts)
- [ ] Calendar support (CalDAV)
- [ ] Keyboard shortcuts
- [ ] System tray with notification badge

## License

MIT
