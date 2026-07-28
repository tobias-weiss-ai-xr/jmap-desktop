# JMAP Desktop — Complete User Stories

> Every interaction a user expects from a modern email client.
> Each story has numbered acceptance criteria describing the exact expected behavior.

---

## Table of Contents

1. [Account & Connection](#1-account--connection)
2. [Compose & Send](#2-compose--send)
3. [Reply & Forward](#3-reply--forward)
4. [Reading & Viewing](#4-reading--viewing)
5. [Email List Navigation](#5-email-list-navigation)
6. [Organizing: Flag, Tag, Label](#6-organizing-flag-tag-label)
7. [Marking Read / Unread](#7-marking-read--unread)
8. [Deleting & Archiving](#8-deleting--archiving)
9. [Moving & Filing](#9-moving--filing)
10. [Search](#10-search)
11. [Threading / Conversation View](#11-threading--conversation-view)
12. [Drafts](#12-drafts)
13. [Attachments](#13-attachments)
14. [Signatures](#14-signatures)
15. [Multiple Identities / Send-As](#15-multiple-identities--send-as)
16. [Keyboard Shortcuts](#16-keyboard-shortcuts)
17. [Notification & Alerts](#17-notification--alerts)
18. [Offline & Resilience](#18-offline--resilience)
19. [Mailbox & Folder Management](#19-mailbox--folder-management)
20. [Contact Auto-Complete](#20-contact-auto-complete)
21. [Bulk Actions](#21-bulk-actions)
22. [Right-Click Context Menu](#22-right-click-context-menu)
23. [Undo / Redo](#23-undo--redo)
24. [Drag & Drop](#24-drag--drop)
25. [Resizable Panes](#25-resizable-panes)
26. [Themes](#26-themes)
27. [Print](#27-print)
28. [External Link Safety](#28-external-link-safety)
29. [Remote Image Blocking](#29-remote-image-blocking)
30. [Snooze](#30-snooze)
31. [Quick Reply / Inline Reply](#31-quick-reply--inline-reply)
32. [Keyboard-Only Workflow](#32-keyboard-only-workflow)
33. [Accessibility](#33-accessibility)
34. [Mobile / Small Window](#34-mobile--small-window)
35. [Tabbed Email Viewing](#35-tabbed-email-viewing)
36. [Email Filtering Rules](#36-email-filtering-rules)
37. [Quick Actions (Swipe / Hover)](#37-quick-actions-swipe--hover)
38. [Message Headers Inspection](#38-message-headers-inspection)
39. [Vacation / Auto-Reply](#39-vacation--auto-reply)
40. [Quota & Storage Info](#40-quota--storage-info)

---

## 1. Account & Connection

### 1.1 Connect to a JMAP server

**As** a user  
**I want** to connect to my email server  
**So that** I can access my mail

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | App opens with no saved credentials | Welcome screen with empty Server URL, Username, Password fields |
| 2 | Type server URL, username, password | Fields accept input; button enables when all three are non-empty |
| 3 | Check "Skip TLS verification" (optional) | For self-signed / dev certificates |
| 4 | Click "Connect" | Button shows spinner "⟳ Connecting…", fields disabled |
| 5 | Connection succeeds | Toast "✓ Connected to JMAP server"; 3-pane layout appears; mailboxes load; inbox auto-selected |
| 6 | Connection fails (wrong credentials, network error, timeout) | Dismissible red error banner with details; form stays populated for retry |
| 7 | Env vars `JMAP_SERVER_URL`, `JMAP_USERNAME`, `JMAP_PASSWORD` set | Fields pre-fill; app auto-connects on load without requiring button click |
| 8 | `JMAP_SKIP_TLS_VERIFY=true/1` | TLS skip checkbox auto-checked |
| 9 | After successful connect | Credentials saved to localStorage; next launch auto-fills and auto-connects |

### 1.2 Disconnect from server

**As** a user  
**I want** to disconnect cleanly  
**So that** I can switch accounts or stop syncing

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click ⏻ in sidebar header | Disconnects from server, stops sync |
| 2 | All state clears | Mail list, mail view, mailboxes, sync status reset to defaults |
| 3 | localStorage credentials removed | Next launch won't auto-connect |
| 4 | Welcome screen returns | Empty form; user can enter new credentials |

### 1.3 Manage multiple accounts

**As** a user with multiple email accounts  
**I want** to add, switch between, and remove accounts  
**So that** I can manage all my mail in one place

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "Add account" in settings | Second connect form appears |
| 2 | Connect second account | Account appears in account switcher in sidebar |
| 3 | Click an account in the switcher | Mailboxes, email list, and view switch to that account |
| 4 | Unified inbox (optional) | Toggle to show emails from all accounts in one list |
| 5 | Remove an account | Account removed from switcher; associated data cleared |
| 6 | Per-account settings | Each account has its own signature, identity, display name |

---

## 2. Compose & Send

### 2.1 Write a new email

**As** a user  
**I want** to compose and send an email  
**So that** I can communicate with someone

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "✉ Compose" in sidebar, or press `c` | Compose form opens as overlay |
| 2 | Focus auto-placed in "To" field | User can start typing immediately |
| 3 | Type recipient email(s), comma-separated | Addresses appear in field; Tab moves to next field |
| 4 | Tab to Cc / Bcc | Optional; leave empty to hide |
| 5 | Tab to Subject | Type subject line |
| 6 | Tab to body | Text area for message content |
| 7 | Press `Ctrl+Enter` or click "Send" | Spinner "Sending…" on button |
| 8 | Send succeeds | Toast "✓ Email sent"; compose closes; email appears in Sent folder |
| 9 | Send fails (network, server rejection) | Error shown inline below form; compose stays open with content preserved |
| 10 | To field empty + Send clicked | Inline validation "Recipient is required" |
| 11 | Subject empty + Send clicked | Inline validation "Subject is required" |
| 12 | Press `Escape` or click "✕" / "Cancel" | Compose closes without sending; no confirmation dialog |

### 2.2 Compose in HTML mode

**As** a user  
**I want** to write formatted emails with bold, lists, links  
**So that** my emails look professional

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Open compose | Rich text toolbar appears above body: **B**, *I*, `Code`, List, Link, Quote |
| 2 | Select text, click Bold | Text wrapped in `<strong>` |
| 3 | Type bullet points | Rendered as `<ul><li>` |
| 4 | Insert a link via toolbar | Link dialog with URL and display text fields |
| 5 | Toggle plain-text / HTML mode | User can switch between rich text and raw text editing |
| 6 | HTML content sends correctly | Email arrives with formatting intact |

### 2.3 Schedule send (send later)

**As** a user  
**I want** to write an email now but send it later  
**So that** it arrives at the right time

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | In compose footer, click "⏱ Schedule" button next to Send | Dropdown: Tomorrow 8 AM, Monday 9 AM, Custom date/time |
| 2 | Pick a time | Email saved as scheduled; toast "✓ Email scheduled for [time]" |
| 3 | Scheduled emails listed in Drafts/Scheduled folder | Editable or cancellable before send time |
| 4 | Scheduled time arrives | Email sends automatically |

---

## 3. Reply & Forward

### 3.1 Reply

**As** a user reading an email  
**I want** to reply to the sender  
**So that** I can respond in context

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "↩ Reply" or press `r` | Compose opens in reply mode |
| 2 | "To" field | Pre-filled with original sender's address |
| 3 | Subject | `Re: [original]` (no double `Re:`) |
| 4 | Body | Original message quoted below (sender, date, subject, body) |
| 5 | Cursor | Placed above quote block, ready to type |
| 6 | `Ctrl+Enter` sends | Sent with `inReplyTo` reference to original |

### 3.2 Reply All

**As** a user  
**I want** to reply to everyone on the email  
**So that** all participants see my response

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "↩↩ Reply All" or press `Shift+R` | Compose opens with all original recipients in To/Cc |
| 2 | "To" field | Original sender + all "To" recipients (except self) |
| 3 | "Cc" field | All original Cc recipients (except self) |
| 4 | Subject / Body | Same as single reply |

### 3.3 Forward

**As** a user  
**I want** to forward an email to someone else  
**So that** I can share information

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "↗ Forward" or press `f` | Compose opens in forward mode |
| 2 | "To" field | Empty — user must type recipient |
| 3 | Subject | `Fwd: [original]` (no double `Fwd:`) |
| 4 | Body | Forwarded block with sender, date, subject, full body |
| 5 | Attachments from original | Included in forwarded email |

---

## 4. Reading & Viewing

### 4.1 Select and read an email

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click a mailbox in sidebar | Mail list loads with skeleton shimmer, then populates |
| 2 | Click an email in the list | Selected email highlighted; right panel shows full content |
| 3 | HTML email body | Sanitized rendering (DOMPurify), images, links clickable, scrollable |
| 4 | Plain-text email body | Rendered in `<pre>` with monospace font, line wrapping |
| 5 | No body content | Muted "No content available" message |
| 6 | 1 second after selection | Auto-marks as read (bold/unread styling removes; unread count updates) |
| 7 | Empty mailbox | "📭 No emails" with mailbox name shown |

### 4.2 View email with attachments

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Email has `hasAttachment: true` | Attachment row shown below subject with file icons, names, sizes |
| 2 | Click "📥 Download" on an attachment | File downloads via JMAP `downloadUrl`; progress indicator if large |
| 3 | Click "👁 Preview" (image/PDF) | Opens preview inline or in system viewer |
| 4 | Multiple attachments | Each listed separately; "Download all" option available |

### 4.3 View full email headers

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "⋮" menu → "View headers" | Collapsible raw headers panel (Received, Message-ID, X-Spam, etc.) |
| 2 | Headers shown in monospace `<pre>` | Copyable text for debugging/troubleshooting |
| 3 | Click again to hide | Headers collapse back |

### 4.4 View thread / conversation

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Open an email that is part of a thread | All messages in thread shown chronologically, newest at bottom |
| 2 | Earlier messages collapsed | "Show earlier messages" button expands them |
| 3 | Each message in thread shows sender, date, snippet | Click to expand full content of that message |
| 4 | Thread view toggle | User can switch between "conversation" and "single message" mode |

### 4.5 View tracking pixels blocked

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Email contains remote images / tracking pixels | Images blocked by default; placeholder boxes shown |
| 2 | "Load remote images" button in header | Click to load all remote images for this email |
| 3 | Per-sender trust | Option to "Always load from [sender]" for future emails |
| 4 | Local/inline images (CID) | Always shown; not affected by remote blocking |

---

## 5. Email List Navigation

### 5.1 Keyboard navigation

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Press `j` or `↓` | Next email selected, scrolled into view |
| 2 | Press `k` or `↑` | Previous email selected, scrolled into view |
| 3 | Wraps at edges | Bottom → top; top → bottom |
| 4 | Press `Enter` | Opens selected email in mail view |
| 5 | `Shift+J` / `Shift+K` | Moves selection without changing view pane (preview mode) |
| 6 | Press `Home` / `End` | Jump to first / last email |

### 5.2 Visual indicators in list

| # | Indicator | Expected behavior |
|---|-----------|-------------------|
| 1 | Unread email | Left accent border (purple) + bold sender + bold subject |
| 2 | Flagged email | Filled star ★ vs empty ☆ |
| 3 | Has attachment | 📎 icon at bottom-right of item |
| 4 | Selected email | Background highlight color |
| 5 | Hovered email | Subtle background change |
| 6 | Read email | Normal weight text, no accent border |

### 5.3 Sorting

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Default sort | Newest first (receivedAt descending) |
| 2 | Click column header or sort button | Toggle sort: newest ↔ oldest, sender A-Z, subject A-Z |
| 3 | Sort indicator | Arrow icon shows current sort direction |
| 4 | Sort persists | Remembers sort preference per mailbox |

### 5.4 Pagination / virtual scrolling

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Mailbox has > 100 emails | Only visible window rendered (virtual scroll); 100 fetched initially |
| 2 | Scroll to bottom | Automatically loads next batch; no "Load more" button |
| 3 | Scroll indicator | Subtle shadow or count showing "42 of 1,200" |
| 4 | Jump to end | Scroll or keyboard shortcut jumps to oldest |

---

## 6. Organizing: Flag, Tag, Label

### 6.1 Flag (star)

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click ☆ star on email in list | Fills to ★; JMAP `keywords/$flagged` set |
| 2 | Click ★ again | Empties back to ☆ |
| 3 | Right-click email | Context menu includes "Flag" / "Unflag" |
| 4 | Press `x` with email selected | Toggles flag |
| 5 | Flagged virtual folder | Sidebar shows ⭐ Flagged folder; clicking it shows all flagged emails |

### 6.2 Custom labels / tags

**As** a user  
**I want** to apply color-coded labels to emails  
**So that** I can categorize beyond just folders

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Right-click email → "Add label" | Dropdown of user-created labels (each with color dot) |
| 2 | Select a label | Colored dot appears on email in list; label shown in mail view header |
| 3 | Create new label | Name + color picker dialog; label appears in sidebar under "Labels" |
| 4 | Remove label | Click colored dot or use context menu; label removed |
| 5 | Filter by label | Click label in sidebar to see all emails with that label |

---

## 7. Marking Read / Unread

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Select an email | Auto-marks as read after 1 second |
| 2 | Click "Mark unread" in mail view | Email unmarks; unread styling returns; count updates |
| 3 | Press `m` key with email selected | Toggles read/unread |
| 4 | Select multiple emails + mark unread | All selected marked unread at once |
| 5 | Sidebar unread count | Updates immediately across all mailboxes |

---

## 8. Deleting & Archiving

### 8.1 Delete (trash-first)

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "🗑 Delete" or press `Delete` / `#` | Confirmation dialog "Delete this email?" |
| 2 | Click "Delete" in dialog | Email moves to Trash (if Trash mailbox exists) or hard-deleted |
| 3 | Toast "✓ Email deleted" | Dismissible confirmation |
| 4 | Selection moves | To next email (or previous if last) |
| 5 | Click "Cancel" or `Escape` | Dialog closes; email stays |
| 6 | Click overlay background | Same as Cancel |

### 8.2 Archive

**As** a user  
**I want** to archive emails out of my inbox  
**So that** my inbox stays clean but emails are preserved

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Press `e` (archive hotkey) or click "📦 Archive" button | Email moves to Archive mailbox; removed from current list |
| 2 | No confirmation dialog | Archive is non-destructive, so no prompt needed |
| 3 | No Archive mailbox exists | Button hidden or disabled; falls back to "Move to…" |
| 4 | Undo toast | Brief "Archived · Undo" toast; clicking Undo moves it back |

### 8.3 Empty trash

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Right-click Trash folder → "Empty trash" | Confirmation dialog "Permanently delete all emails in Trash?" |
| 2 | Confirm | All emails in Trash destroyed on server; folder shows empty |

---

## 9. Moving & Filing

### 9.1 Move to folder

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "📁 Move" in mail view | Dropdown with all mailboxes except current |
| 2 | Each mailbox shows role icon + name | 📥 Inbox, 📦 Archive, 🗑️ Trash, ⚠️ Junk, 📁 Custom |
| 3 | Click target mailbox | Email moves; list updates; unread counts update |
| 4 | Click outside dropdown | Closes without action |
| 5 | Move fails | Error toast; email stays in current mailbox |

### 9.2 Drag email to folder

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Drag email from list to sidebar folder | Visual drop target highlight on folder |
| 2 | Drop on folder | Email moves to that folder |
| 3 | Drag fails / no valid target | Email snaps back to original position |
| 4 | Multi-email drag | Dragging a selected range moves all |

---

## 10. Search

### 10.1 Basic search

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click search box in sidebar | Placeholder "Search emails…" |
| 2 | Type query and press `Enter` | JMAP text search fires; results replace email list |
| 3 | Top bar shows "🔍 Search results" with "✕ Clear" | Active search indicator |
| 4 | Press `Escape` in search box or click "✕ Clear" | Restores previous mailbox view |
| 5 | No results | "📭 No emails" empty state |

### 10.2 Advanced search / filters

**As** a user  
**I want** to filter emails by sender, date, has-attachment, etc.  
**So that** I can find specific messages quickly

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "🔽" next to search box | Filter panel: From, To, Subject, Date range, Has attachment, Has flag |
| 2 | Fill any combination + search | JMAP filter built from combination (AND) |
| 3 | "Unread only" quick filter toggle | Shows only unread emails in current mailbox |
| 4 | "Flagged only" quick filter toggle | Shows only flagged emails |
| 5 | "Has attachment" quick filter | Shows only emails with attachments |
| 6 | Saved searches | User can save filter combinations as named "Saved Searches" in sidebar |

### 10.3 Search as you type

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Type in search box | After 300ms debounce, search fires automatically (no Enter needed) |
| 2 | Results update live | List updates as user types more characters |
| 3 | Empty search box (backspace all) | Results clear; mailbox view restored |

---

## 11. Threading / Conversation View

**As** a user  
**I want** emails grouped into conversations  
**So that** I can see the full context of a discussion

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Thread view is default (or togglable) | Email list shows threads instead of individual messages |
| 2 | Thread row shows latest sender, subject, snippet, count (e.g. "3 messages") |
| 3 | Unread count is per-thread | Shows if any message in thread is unread |
| 4 | Click thread | Opens conversation view: all messages stacked chronologically |
| 5 | Collapse / expand | Individual messages can be collapsed to show only sender + date |
| 6 | Label / flag on any message | Applies to that specific message, thread indicator shows aggregate |
| 7 | Delete from thread | Deletes that specific message; thread remains if other messages exist |
| 8 | Mute thread | Future messages in thread auto-archived; no notification |

---

## 12. Drafts

### 12.1 Auto-save draft

**As** a user writing an email  
**I want** my work saved automatically  
**So that** I don't lose it if the app crashes

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Open compose and start typing | After 3 seconds of inactivity, draft auto-saves to server (Drafts mailbox) |
| 2 | Continue typing | Draft updates on server after each pause |
| 3 | Close compose (Escape) | Draft preserved in Drafts folder |
| 4 | Click Drafts in sidebar | List of saved drafts; each shows To, Subject, timestamp |
| 5 | Click a draft | Opens compose pre-filled with saved content |
| 6 | Send the draft | Draft removed from Drafts folder; email sent |

### 12.2 Manual save

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Press `Ctrl+S` in compose | Manual save; toast "✓ Draft saved" |
| 2 | Draft folder | Saved drafts appear in Drafts mailbox |

---

## 13. Attachments

### 13.1 Add attachment

**As** a user composing  
**I want** to attach files to my email  
**So that** I can share documents

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "📎 Attach" button in compose toolbar | File picker opens (system native dialog) |
| 2 | Select file(s) | File name(s) appear below compose body with size; removable with ✕ |
| 3 | Drag file onto compose window | File attaches same as picker |
| 4 | Total attachment size shown | E.g. "3 files (4.2 MB)" |
| 5 | Size limit warning | If total > server limit, warning shown before send |
| 6 | Send with attachments | Files uploaded via JMAP `uploadUrl`; referenced in email body |

### 13.2 Download attachment

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | View email with attachments | Attachment bar shows file name, size, type icon |
| 2 | Click "📥 Download" | Downloads via JMAP `downloadUrl`; file saves to system Downloads folder |
| 3 | Click file name | Same as download (or opens in system viewer if supported) |
| 4 | Large file download | Progress bar shown |

---

## 14. Signatures

**As** a user  
**I want** to have an email signature  
**So that** my emails include my name, title, and contact info

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Open Settings → Signatures | Signature editor with rich text support |
| 2 | Type or paste signature content | Preview shows how it will look |
| 3 | Insert variables | `[Name]`, `[Email]`, `[Title]`, `[Company]` auto-filled from account |
| 4 | "Apply to all new emails" toggle | Signature auto-inserted at bottom of every new compose |
| 5 | "Apply to replies/forwards" toggle | Signature inserted below quote block |
| 6 | Multiple signatures | Can create and switch between signatures; set a default |

---

## 15. Multiple Identities / Send-As

**As** a user with aliases  
**I want** to choose which address I send from  
**So that** replies go to the right address

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "From" dropdown in compose (or click the From address) | Shows all configured identities: primary + aliases |
| 2 | Select an alias | "From" field updates; email sends from that address |
| 3 | Identity auto-detection | When replying, "From" auto-selects the identity that received the original email |
| 4 | Settings → Identities | Add/remove/verify email addresses; each can have its own signature |

---

## 16. Keyboard Shortcuts

### 16.1 Global shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `c` | New compose | Anywhere (not in input/textarea) |
| `r` | Reply to selected/open email | Anywhere |
| `R` (Shift+r) | Reply all | Anywhere |
| `f` | Forward | Anywhere |
| `e` | Archive selected email | Anywhere |
| `#` or `Delete` | Delete selected (with confirm) | Anywhere |
| `x` | Toggle flag | Anywhere |
| `m` | Toggle read/unread | Anywhere |
| `s` | Toggle spam mark | Anywhere |
| `Escape` | Close compose / dialog / search | Context-aware |
| `?` | Show keyboard shortcut help | Anywhere (opens overlay) |
| `/` | Focus search box | Anywhere |

### 16.2 Navigation shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `j` / `↓` | Next email | Mail list focused |
| `k` / `↑` | Previous email | Mail list focused |
| `Enter` | Open selected email | Mail list focused |
| `Home` | Jump to first email | Mail list focused |
| `End` | Jump to last email | Mail list focused |
| `Shift+j` | Next mailbox | Sidebar focused |
| `Shift+k` | Previous mailbox | Sidebar focused |
| `Tab` | Cycle focus: sidebar → list → view | Global |

### 16.3 Compose shortcuts

| Key | Action | Context |
|-----|--------|---------|
| `Ctrl+Enter` | Send | Compose open |
| `Ctrl+S` | Save draft | Compose open |
| `Escape` | Close compose | Compose open |
| `Tab` | Next field (To → Cc → Bcc → Subject → Body) | Compose focused |

### 16.4 Shortcut help overlay

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Press `?` | Full-screen semi-transparent overlay shows all shortcuts organized by category |
| 2 | Press `Escape` or `?` again | Overlay closes |
| 3 | Grouped by: Navigation, Actions, Compose, Mailbox | Clear visual hierarchy |

---

## 17. Notification & Alerts

### 17.1 Desktop notifications

**As** a user  
**I want** to be notified of new emails  
**So that** I don't miss important messages

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | New email arrives (sync or push) | System desktop notification: sender, subject preview, app icon |
| 2 | Click notification | App window opens/focuses; email selected |
| 3 | Notification settings | Toggle on/off in Settings; per-account; do-not-disturb hours |
| 4 | Only for inbox | Notifications only for new inbox messages, not for Junk/Sent/etc. |

### 17.2 Unread badge (system tray)

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | New unread email | System tray icon shows unread count badge |
| 2 | All read | Badge disappears |
| 3 | Click tray icon | Shows/hides or focuses app window |

### 17.3 Toast notifications

| # | Type | Expected behavior |
|---|------|-------------------|
| 1 | Success (sent, archived, etc.) | Green ✓ toast, auto-dismisses after 3 seconds |
| 2 | Error (send failed, network) | Red ✕ toast, stays 8 seconds, dismissible |
| 3 | Info (sync status) | Blue ℹ toast, auto-dismisses after 3 seconds |
| 4 | Undo-able actions | "Archived · Undo" toast with clickable "Undo" |

---

## 18. Offline & Resilience

**As** a user  
**I want** the app to work when offline  
**So that** I can still read old emails and queue outgoing mail

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Network disconnects | Banner shows "⚠ Offline — changes will sync when reconnected" |
| 2 | Previously loaded emails | Still viewable (cached in SQLite) |
| 3 | Click email not yet cached | Shows "Email not available offline" |
| 4 | Compose while offline | Compose works; email queued locally; sends automatically on reconnect |
| 5 | Flag / read / delete while offline | Actions queued; synced on reconnect |
| 6 | Network reconnects | Banner clears; queued operations sync; new emails fetched |
| 7 | Conflict resolution | Server-side wins for conflicts; user notified of changes |

---

## 19. Mailbox & Folder Management

### 19.1 Create / rename / delete folders

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Right-click sidebar → "New folder" | Name input dialog; optional parent folder picker |
| 2 | Type name, confirm | Folder appears in sidebar under parent (or at top level) |
| 3 | Right-click folder → "Rename" | Inline edit or dialog; name updates everywhere |
| 4 | Right-click folder → "Delete" | Confirmation "Delete folder and all emails?"; non-recoverable |
| 5 | Cannot delete system folders | Inbox, Sent, Drafts, Trash, Junk, Archive cannot be deleted |

### 19.2 Nested / hierarchical folders

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Folder has subfolders | Sidebar shows indented hierarchy with collapse/expand arrows |
| 2 | Click arrow | Collapses/expands children |
| 3 | Total unread = sum | Parent folder shows aggregate unread count |

---

## 20. Contact Auto-Complete

**As** a user typing in the To field  
**I want** addresses to auto-complete from my contacts and recent recipients  
**So that** I don't have to type full email addresses

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Type 3+ characters in To/Cc/Bcc | Dropdown shows matching contacts: name, email, avatar/initials |
| 2 | Sources: address book, recent recipients, email headers | Matches against name and email |
| 3 | Arrow keys navigate dropdown | Up/down to select; Enter to confirm |
| 4 | Click a suggestion | Address added to field; multiple addresses separated by commas or pills |
| 5 | Backspace on pill | Removes last added address |
| 6 | No matches | Dropdown closes; user types full address |

---

## 21. Bulk Actions

**As** a user  
**I want** to select multiple emails and act on them at once  
**So that** I can triage efficiently

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click checkbox on an email (or long-press) | Email enters multi-select mode; checkbox fills |
| 2 | Click more emails | Each added to selection; count badge shows "3 selected" |
| 3 | Shift+click | Selects range from first clicked to current |
| 4 | Select all checkbox | Toggles all emails in current view |
| 5 | Bulk action bar appears | "Mark read · Mark unread · Flag · Archive · Delete · Move" |
| 6 | Click any bulk action | Applies to all selected emails; progress shown |
| 7 | `Escape` | Exits multi-select mode; clears selection |

---

## 22. Right-Click Context Menu

| # | Context | Expected menu items |
|---|---------|---------------------|
| 1 | Right-click email in list | Reply · Reply All · Forward · Flag · Mark read/unread · Archive · Delete · Move to → · Label → · View headers |
| 2 | Right-click folder in sidebar | New subfolder · Rename · Delete · Empty trash (if Trash) · Mark all read · Properties |
| 3 | Right-click attachment in mail view | Download · Preview · Save as |

---

## 23. Undo / Redo

**As** a user  
**I want** to undo my last action  
**So that** I can recover from mistakes

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Delete an email | Toast: "Archived · **Undo**" or "Deleted · **Undo**" |
| 2 | Click "Undo" within 5 seconds | Email restored to original mailbox/position |
| 3 | Undo expires | After 5 seconds, toast fades; action is permanent |
| 4 | Move email to folder | Toast: "Moved to Archive · **Undo**" |
| 5 | Mark as read | Toast: "Marked as read · **Undo**" |
| 6 | Press `Ctrl+Z` | Undoes last destructive action (same as toast undo) |

---

## 24. Drag & Drop

| # | Source → Target | Expected behavior |
|---|-----------------|-------------------|
| 1 | Email → sidebar folder | Move email to that folder (same as "Move" button) |
| 2 | File from desktop → compose window | Attach file |
| 3 | File from desktop → mail view | Attach to new reply draft |
| 4 | Folder in sidebar | Reorder folders by dragging (optional) |

---

## 25. Resizable Panes

**As** a user  
**I want** to resize the sidebar and mail list  
**So that** I can see more or less of each panel

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Drag sidebar right edge | Width resizes (min 180px, max 400px) |
| 2 | Drag mail list right edge | Width resizes (min 250px, max 600px) |
| 3 | Double-click divider | Reset to default width |
| 4 | Width persists | Saved to localStorage |

---

## 26. Themes

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Settings → Appearance → Theme | Options: Dark (default), Light, System (auto) |
| 2 | Dark theme | Current Tokyo Night color scheme |
| 3 | Light theme | Light backgrounds, dark text, adjusted borders |
| 4 | System | Follows OS preference (prefers-color-scheme media query) |
| 5 | Theme persists | Saved to localStorage; applies on next launch |

---

## 27. Print

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "🖨 Print" or press `Ctrl+P` | System print dialog opens |
| 2 | Print content | Email subject, from/to/date headers, body, attachments list |
| 3 | Print preview | Respects system print preview |

---

## 28. External Link Safety

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click a link in HTML email | Opens in system browser (not in-app) |
| 2 | Hover over link | Status bar / tooltip shows full URL |
| 3 | Suspicious link detection | If URL text differs from href, show warning "The display text doesn't match the link destination" |
| 4 | `target="_blank"` with `rel="noopener noreferrer"` | Security best practice |

---

## 29. Remote Image Blocking

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | HTML email contains `<img src="https://tracker.example.com/pixel.gif">` | Blocked; grey placeholder box shown |
| 2 | "Load images" button in mail header | Click to load all remote images for this email |
| 3 | "Always trust this sender" checkbox | Future emails from this sender auto-load images |
| 4 | Per-account setting | Settings → Privacy → "Block remote images" toggle |

---

## 30. Snooze

**As** a user  
**I want** to temporarily hide an email and have it come back later  
**So that** I can focus on what matters now

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "⏰ Snooze" on email | Options: Later today, Tomorrow, This weekend, Next week, Pick date/time |
| 2 | Select time | Email disappears from current view; toast "Snoozed until [time]" |
| 3 | Snoozed folder | Emails in a "Snoozed" virtual folder until their time arrives |
| 4 | Time arrives | Email reappears in Inbox (or top of list) with "🔔 Snoozed" indicator |
| 5 | Click snoozed email → "Unsnooze" | Returns to original position immediately |

---

## 31. Quick Reply / Inline Reply

**As** a user  
**I want** to reply without leaving my reading pane  
**So that** I can respond quickly without a full compose overlay

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | At bottom of email view, click reply text box | Inline reply area expands below the email body |
| 2 | Type reply text | Simple text area (not full compose); no To/Subject fields |
| 3 | Press `Ctrl+Enter` or click "Send" | Reply sent; inline area collapses |
| 4 | Press `Escape` | Inline reply collapses without sending |
| 5 | "Open in full compose" link | Switches to full compose overlay with all fields |

---

## 32. Keyboard-Only Workflow

**As** a power user  
**I want** to do everything without touching the mouse  
**So that** I can process email at maximum speed

| # | Task | Key sequence |
|---|------|-------------|
| 1 | Open app → read first email | (auto-connects) → `j`/`k` to navigate → already selected |
| 2 | Read email → reply → send | Email selected → `r` → type reply → `Ctrl+Enter` |
| 3 | Read email → archive → next | `e` (archives, moves to next) |
| 4 | Read email → delete → next | `#` → `Enter` (confirm) |
| 5 | Flag email → move on | `x` → `j` |
| 6 | Compose new email → send | `c` → type → `Ctrl+Enter` |
| 7 | Search → select result → open | `/` → type → `Enter` → `j`/`k` → `Enter` |
| 8 | Switch mailbox | `Shift+j` / `Shift+k` |
| 9 | Toggle shortcut help | `?` |
| 10 | Quit | `Ctrl+Q` |

---

## 33. Accessibility

| # | Aspect | Expected behavior |
|---|--------|-------------------|
| 1 | Screen reader | All interactive elements have aria-labels, roles, and live regions |
| 2 | Keyboard focus | Visible focus ring on all focusable elements; Tab order is logical |
| 3 | Color contrast | All text meets WCAG AA contrast ratio in both themes |
| 4 | Reduced motion | Respects `prefers-reduced-motion`: no animations, no shimmer |
| 5 | Font scaling | App respects OS font size / DPI settings |
| 6 | Semantic HTML | `<nav>`, `<main>`, `<article>`, `<button>`, proper heading levels |
| 7 | Dialog trapping | Focus trapped inside modals; Escape closes |

---

## 34. Mobile / Small Window

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Window width < 768px | Sidebar collapses to icon-only strip or hamburger menu |
| 2 | Click hamburger | Sidebar slides in as overlay |
| 3 | Mail list + view stacked | Only one shown at a time; back button to return |
| 4 | Compose | Full-screen overlay (no side-by-side) |
| 5 | Touch gestures | Swipe right = archive, swipe left = delete, swipe to flag |

---

## 35. Tabbed Email Viewing

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Middle-click or `Shift+Enter` on email | Opens email in a new tab within the mail view area |
| 2 | Tab bar shows open emails | Each tab: sender name or subject (truncated); ✕ to close |
| 3 | Switch between tabs | Click tab or `Ctrl+Tab` / `Ctrl+Shift+Tab` |
| 4 | Close all tabs | "Close all" option in tab bar context menu |
| 5 | Tab persists | Open tabs survive mailbox navigation |

---

## 36. Email Filtering Rules

**As** a user  
**I want** to set up rules that automatically sort my incoming mail  
**So that** my inbox stays organized

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Settings → Rules → "Add rule" | Rule builder: IF (condition) THEN (action) |
| 2 | Conditions: From contains, Subject contains, Has attachment, To/CC is | Dropdown/checkbox selectors |
| 3 | Actions: Move to folder, Mark as read, Flag, Add label, Reject | Single or multiple |
| 4 | Rules apply in order | Drag to reorder; "Stop evaluating" checkbox per rule |
| 5 | Client-side + server-side | Rules saved; applied on incoming emails during sync |

---

## 37. Quick Actions (Swipe / Hover)

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Hover over email in list (desktop) | Quick action buttons fade in: 🗑 Delete, 📦 Archive, ★ Flag |
| 2 | Click quick action | Instant action; no confirmation for archive/flag; confirm for delete |
| 3 | Swipe right (touch / trackpad) | Archive |
| 4 | Swipe left | Delete / Flag (configurable) |

---

## 38. Message Headers Inspection

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "⋮" → "View source" / "View headers" | Raw MIME headers in monospace scrollable panel |
| 2 | Copy headers | Button to copy all headers to clipboard |
| 3 | SPF / DKIM / DMARC status | If available, show pass/fail badges in mail view header |

---

## 39. Vacation / Auto-Reply

**As** a user going on vacation  
**I want** to set up an automatic reply  
**So that** people know I'm away

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Settings → Vacation / Auto-Reply | Toggle on/off, start date, end date, message body |
| 2 | Enable | Server-side sieve/vacation rule configured via JMAP |
| 3 | Auto-reply sends once per sender | Deduplication: each sender gets at most one auto-reply |
| 4 | End date reached | Auto-reply automatically disables; toggle updates |

---

## 40. Quota & Storage Info

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Sidebar footer or account menu | Shows storage usage: "2.4 GB of 15 GB used" |
| 2 | Progress bar | Visual indicator: green < 70%, yellow 70–90%, red > 90% |
| 3 | Hover / click | Breakdown by mailbox size (if server supports it) |
| 4 | Quota exceeded warning | Toast + banner: "Storage almost full — delete old emails or empty trash" |
