# JMAP Desktop — Conversation User Stories

> Behavioral specs: each story describes a complete user flow from the person's perspective,
> with step-by-step acceptance criteria the app must satisfy.

---

## 1. Write a new email

**As** a user  
**I want** to compose and send a new email  
**So that** I can communicate with someone

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "✉ Compose" in sidebar, or press `c` anywhere in the app | Compose form opens as an overlay replacing the mail view |
| 2 | Focus is placed in the "To" field automatically | User can start typing immediately without clicking |
| 3 | Type a recipient email address in "To" | Address appears in the field; tab/click moves to next field |
| 4 | Optionally type Cc/Bcc recipients | Fields expand only when used; can leave empty |
| 5 | Type a subject line | Subject populates the header shown in compose view |
| 6 | Type message body in the textarea | Text appears as plain-text, monospace font, auto-resizes |
| 7 | Press `Ctrl+Enter` | Email sends immediately; spinner shows "Sending…" on button |
| 8 | Send succeeds | Toast notification "✓ Email sent"; compose closes; email appears in Sent mailbox |
| 9 | Send fails (network error, server rejection) | Error message appears below the form; compose stays open with content preserved; user can retry |
| 10 | Press `Escape` or click "✕" | Compose closes without sending; no confirmation dialog |
| 11 | Click "Cancel" | Same as Escape — closes without sending |
| 12 | Recipient field is empty and user clicks Send | Validation error "Recipient is required" appears inline; email is not sent |
| 13 | Subject is empty and user clicks Send | Validation error "Subject is required" appears inline |
| 14 | User is not connected (server disconnected mid-compose) | Send attempt shows error; compose stays open |

---

## 2. Reply to an email

**As** a user  
**I want** to reply to an email I'm reading  
**So that** I can respond in context

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Have an email selected and visible in MailView | Email content is displayed |
| 2 | Click "↩ Reply" button, or press `r` | Compose opens in reply mode with pre-filled fields |
| 3 | "To" field | Pre-filled with the original sender's address (Name <email> or bare email) |
| 4 | Subject field | Pre-filled with `Re: [original subject]` (no double `Re:` if already present) |
| 5 | Body | Contains quoted original message block: sender, date, subject, and preview text |
| 6 | Cursor/focus | Placed at the top of the body, above the quote block |
| 7 | User edits body, presses `Ctrl+Enter` | Email sends with `replyToId` referencing the original email |
| 8 | Reply from sidebar "Compose" button | Opens blank compose (not reply) — reply only via `r` key or Reply button on open email |
| 9 | Press `Escape` in reply mode | Compose closes, returns to reading the original email |

---

## 3. Forward an email

**As** a user  
**I want** to forward an email to someone else  
**So that** I can share information

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Have an email selected and visible in MailView | Email content is displayed |
| 2 | Click "↗ Forward" button | Compose opens in forward mode |
| 3 | "To" field | Empty — user must type the recipient |
| 4 | Subject field | Pre-filled with `Fwd: [original subject]` (no double `Fwd:` if already present) |
| 5 | Body | Contains forwarded message block: sender, date, subject, preview |
| 6 | User types a recipient and body text, presses `Ctrl+Enter` | Email sends |
| 7 | Press `Escape` | Closes compose, returns to original email |

---

## 4. Read an email

**As** a user  
**I want** to select and read an email from the list  
**So that** I can see its content

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click a mailbox folder in the sidebar | Mail list loads emails from that folder with skeleton loading shimmer |
| 2 | Email list populates | Shows sender, subject, preview, relative date, flag star, attachment indicator |
| 3 | Click an email in the list | Email highlights as selected; right panel shows full email |
| 4 | Selected email shows full content | Subject, From, To, Cc (if any), Date, action buttons, then body |
| 5 | HTML email body | Rendered sanitized (DOMPurify), scrollable |
| 6 | Plain-text email body | Rendered in `<pre>` with monospace font, line wrapping |
| 7 | Email with no body | Shows muted text "No content available" |
| 8 | After 1 second, email auto-marks as read | Bold/unread styling removes; unread count decreases in sidebar |
| 9 | No emails in mailbox | Shows "📭 No emails" empty state with mailbox name |
| 10 | No mailbox selected | Shows "← Select a mailbox" empty state |

---

## 5. Navigate the email list

**As** a user  
**I want** to navigate emails with my keyboard  
**So that** I can triage my inbox efficiently

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Press `j` or `↓` | Next email in list is selected and scrolled into view |
| 2 | Press `k` or `↑` | Previous email in list is selected and scrolled into view |
| 3 | Selection wraps | At bottom, `j` wraps to top; at top, `k` wraps to bottom |
| 4 | Press `Enter` | Current selection opens in mail view (same as click) |
| 5 | Selected email | Visually highlighted with background color |
| 6 | Unread emails | Left accent border + bold sender/subject text |
| 7 | Flagged emails | Star is filled (★ vs ☆) |
| 8 | Right-click an email | Toggles flag on that email |
| 9 | Mailbox header | Shows mailbox name and total count badge (e.g. "Inbox • 42") |

---

## 6. Flag / unflag an email

**As** a user  
**I want** to star or unstar an email  
**So that** I can mark important messages

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click the ☆ star on an email in the list | Star fills to ★; JMAP `keywords/$flagged` set to true |
| 2 | Click ★ again | Star empties back to ☆; `keywords/$flagged` removed |
| 3 | Right-click an email in the list | Toggles flag (same as clicking the star) |
| 4 | Press `x` with an email selected | Toggles flag on the currently selected email |
| 5 | Flag persists after refresh | Starred state survives mailbox navigation and sync |

---

## 7. Mark read / unread

**As** a user  
**I want** to mark emails as read or unread  
**So that** I can manage what I still need to look at

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Select an email (click or keyboard nav) | After 1 second, email auto-marks as read |
| 2 | Click "Mark unread" button in MailView | Email unmarks as read; unread styling returns in list |
| 3 | Click "Mark read" button | Email marks as read; unread styling removes |
| 4 | Unread count in sidebar badge | Updates immediately when read/unread status changes |
| 5 | Total unread in footer | Updates to reflect current total across all mailboxes |

---

## 8. Delete an email

**As** a user  
**I want** to delete an email  
**So that** I can clean up my inbox

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "🗑 Delete" button, or press `Delete`/`#` key | Confirmation dialog appears: "Delete this email?" |
| 2 | Confirm dialog shows "Delete" and "Cancel" buttons | Focus starts on Delete for quick keyboard confirm |
| 3 | Click "Delete" | Email moves to Trash mailbox (if exists) or is destroyed; toast "✓ Email deleted" |
| 4 | Email removed from current list | Selection moves to next email (or previous if last) |
| 5 | Click "Cancel" or press `Escape` on dialog | Dialog closes, email remains |
| 6 | Click outside dialog overlay | Same as Cancel — dialog closes safely |

---

## 9. Move an email

**As** a user  
**I want** to move an email to another folder  
**So that** I can organize my mail

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click "📁 Move" button in MailView | Dropdown menu appears with all mailboxes except the current one |
| 2 | Each mailbox shows role icon + name | 📥 Inbox, 📦 Archive, 🗑️ Trash, ⚠️ Junk, 📁 [custom name] |
| 3 | Click a target mailbox | Email moves; disappears from current list; mailbox unread counts update |
| 4 | Click outside dropdown | Menu closes without action |
| 5 | Move fails | Error toast shown; email stays in current mailbox |

---

## 10. Search emails

**As** a user  
**I want** to search across emails  
**So that** I can find specific messages

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Type in search box in sidebar | Text field is active, placeholder "Search emails…" |
| 2 | Press `Enter` | JMAP search fires; email list shows matching results |
| 3 | Top bar appears | Shows "🔍 Search results" with "✕ Clear" button |
| 4 | Click "✕ Clear" or press `Escape` in search | Restores previous mailbox view |
| 5 | No results | Email list shows "📭 No emails" |
| 6 | Search is empty and Enter pressed | No action (guard: empty query ignored) |

---

## 11. Connect to a JMAP server

**As** a user  
**I want** to connect to my email server  
**So that** I can access my mail

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | App opens with no saved credentials | Welcome screen shows with empty form fields |
| 2 | User enters Server URL, Username, Password | Fields accept input; validation requires all three |
| 3 | Optionally checks "Skip TLS verification" | For self-signed certs / dev environments |
| 4 | Clicks "Connect" or form submit | Spinner appears on button: "⟳ Connecting…" |
| 5 | Connection succeeds | Toast "✓ Connected to JMAP server"; 3-pane layout appears; mailboxes load; inbox auto-selected |
| 6 | Connection fails | Red error banner appears with details and ✕ dismiss button; form stays populated for retry |
| 7 | Env vars `JMAP_SERVER_URL`, `JMAP_USERNAME`, `JMAP_PASSWORD` are set | Fields pre-fill; app auto-connects on load without requiring button click |
| 8 | `JMAP_SKIP_TLS_VERIFY=true/1` env var set | TLS skip checkbox auto-checked |
| 9 | Credentials saved to localStorage | Next app launch auto-fills and auto-connects |
| 10 | Settings page also has connect form | Separate page accessible via ⚙ link in sidebar footer |

---

## 12. Disconnect

**As** a user  
**I want** to disconnect from the server  
**So that** I can switch accounts or stop syncing

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | Click ⏻ disconnect button in sidebar | App disconnects from JMAP server |
| 2 | All state clears | Mail list, mail view, mailboxes, sync status reset |
| 3 | localStorage credentials removed | Next launch won't auto-connect |
| 4 | Welcome screen returns | Shows empty connect form; no credentials pre-filled |

---

## 13. Manage mailboxes

**As** a user  
**I want** to see and navigate my mail folders  
**So that** I can organize my mail

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | After connecting | Sidebar shows all mailboxes ordered by role (Inbox, Sent, Drafts, Archive, Junk, Trash, custom) |
| 2 | Click a mailbox | Email list loads from that mailbox; header shows name + count |
| 3 | Mailbox with unread emails | Shows unread count badge (purple pill with number) |
| 4 | Unread count > 999 | Badge shows "999+" |
| 5 | No mailboxes found | Shows muted "No mailboxes found" text |
| 6 | Sync receives mailbox changes | Mailbox list updates dynamically |
| 7 | Footer shows total unread | Aggregated across all mailboxes, or "All read" |

---

## 14. Background sync

**As** a user  
**I want** my emails to stay up to date automatically  
**So that** I don't miss new messages

### Acceptance Criteria

| # | Step | Expected behavior |
|---|------|-------------------|
| 1 | After connecting | Sync badge in sidebar shows ✓; sync status is "synced" or "push-connected" |
| 2 | Polling sync runs every 30 seconds | Badge briefly shows spinning ⟳ during sync; returns to ✓ when done |
| 3 | EventSource (push) connects | Status shows "push-connected"; near-instant updates |
| 4 | New email arrives | Appears in list automatically; unread count updates |
| 5 | Email deleted on server | Removed from local list |
| 6 | Email flagged on server | Flag star updates locally |
| 7 | Mailbox changes on server | Sidebar mailbox list updates |
| 8 | Connection lost | Sync status updates; app retries with exponential backoff |
