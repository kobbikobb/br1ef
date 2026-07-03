# Plan — br1ef v0.1

## Milestone: Fetch email and print to screen

**Goal:** Run `br1ef`, read `.env` for IMAP credentials, fetch last 7 days of email, print From / Subject / Body to stdout.

## Status

✅ All steps complete.

```
$ cargo build && cargo test && cargo clippy
✓ clean build, 4 tests pass, clippy -D warnings clean
```

## Steps

### 1. Add dependencies

| Crate | Why |
|---|---|
| `imap` | IMAP client |
| `native-tls` | TLS for IMAP |
| `dotenvy` | Load `.env` |
| `mailparse` | Parse MIME bodies |
| `chrono` | Date range (last 7 days) |
| `anyhow` | Error handling |

**DoD:** `cargo check` passes with all deps added.

### 2. Implement `ImapSource` in `br1ef-core`

- Struct `ImapConfig { host, port, username, password }`
- `ImapSource::fetch()` → connects via IMAP, searches SINCE date, fetches envelopes + body
- Returns `Vec<Item>` (Item already has id, title, body, source, urgent)
  - id = IMAP UID
  - title = Subject header
  - body = plain-text body extracted from MIME (prefer text/plain over text/html)
  - source = "imap"
  - urgent = false (always for now)

**DoD:** `cargo build` passes. Real validation happens in step 3.

### 3. Implement CLI in `br1ef-cli`

- Load `.env` via `dotenvy`
- Read `IMAP_HOST`, `IMAP_PORT`, `IMAP_USERNAME`, `IMAP_PASSWORD`
- Build `ImapConfig`, call `ImapSource::fetch()`
- Print each item: a blank line, `From:`, `Subject:`, then body

**DoD:** `cargo run` with a valid `.env` prints emails from the last 7 days.

### 4. Create sample `.env.example`

```
IMAP_HOST=imap.gmail.com
IMAP_PORT=993
IMAP_USERNAME=you@gmail.com
IMAP_PASSWORD=your-app-password
```

**DoD:** File exists at repo root.

### 5. `.gitignore` and polish

- Add `.env` to `.gitignore`
- Handle non-UTF8 bodies gracefully

**DoD:** `cargo build` passes, `cargo test` passes, `cargo clippy` clean.

---

## Out of scope for v0.1

- OAuth (app passwords / regular IMAP password is fine)
- HTML-to-text conversion (raw body dump)
- Attachments
- Other sources (calendar, notifications)
- TUI / fancy output
- Persistent storage
- Scheduling / daemon
