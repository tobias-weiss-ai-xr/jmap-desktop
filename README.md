# jmap-desktop

Native JMAP desktop email client for Linux, built with **Tauri 2 + Rust + TypeScript**.

> Think Thunderbird, but for JMAP.

## Status

🚧 **Early development** — scaffolding in progress.

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
├── Rust Backend (jmap-client crate, background sync, system integration)
└── Web Frontend (TypeScript + SvelteKit, virtual lists, rich compose)
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Backend | Rust + [stalwartlabs/jmap-client](https://github.com/stalwartlabs/jmap-client) |
| Frontend | TypeScript + SvelteKit |
| JMAP protocol | [RFC 8620](https://jmap.io/spec-core.html) + [RFC 8621](https://jmap.io/spec-mail.html) |

## License

MIT
