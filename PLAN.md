# Plan — br1ef v0.1

## Milestone: Fetch email and print to screen

**Goal:** Run `br1ef`, read `.env` for IMAP credentials, fetch last 7 days of email, print From / Subject / Body to stdout.

## Status

### ✅ Milestone: Fetch email and print to screen

```
$ cargo build && cargo test && cargo clippy
✓ clean build, 4 tests pass, clippy -D warnings clean
```

### ✅ Milestone: Digest stats and progress timer

**DoD:** `br1ef digest` shows data stats (items, bytes, words) before LLM call and elapsed time after.

```
  Generating digest from 12 item(s) (8431 bytes, 1382 words)...
  Digest generated in 3.2s.
```

- [`br1ef-core/src/service/digest.rs`](br1ef-core/src/service/digest.rs) — stats computed before LLM call, timer wraps the call, output via `eprintln!`
- [`br1ef-cli/src/main.rs`](br1ef-cli/src/main.rs) — removed redundant `println!("Digest generated.")` (timer output replaces it)
- [`br1ef-core/src/service/digest_test.rs`](br1ef-core/src/service/digest_test.rs) — 4 tests: empty items, items with summary, agent error propagation, multi-source aggregation

```
$ cargo test
✓ 51 tests pass

$ cargo clippy -- -D warnings
✓ clean
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

---

## Milestone: Items count command

**Goal:** Run `br1ef items` to see how many stored items exist per category (source).

### Status

### Steps

#### 1. Add `get_item_counts_by_source` to `Storage` trait
- Signature: `fn get_item_counts_by_source(&self) -> anyhow::Result<Vec<(String, usize)>>`
- `SqliteStorage`: `SELECT source, COUNT(*) FROM items GROUP BY source ORDER BY source`
- `InMemoryStorage`: group in-memory items by source into a `HashMap`

**DoD:** `cargo build` passes.

#### 2. Tests for counts
- `db_test.rs`: empty table, single source, multiple sources, items inserted across multiple calls
- `memory_test.rs`: same cases

**DoD:** all tests pass via `cargo test`.

#### 3. CLI `items` subcommand
- Add `Items` variant to `Commands` enum
- Handler calls `storage.get_item_counts_by_source()` and prints each source with count
- Include in `print_help()`

**DoD:** `cargo build` passes, `cargo clippy -- -D warnings` clean.

## Out of scope for v0.1

- OAuth (app passwords / regular IMAP password is fine)
- HTML-to-text conversion (raw body dump)
- Attachments
- Other sources (calendar, notifications)
- TUI / fancy output
- Persistent storage
- Scheduling / daemon
