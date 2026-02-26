# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Focus

Only the `xf-chat` binary matters. The forum binary (`ruforo`) and all non-chat code are legacy/dead. All work should focus on the chat system and its XenForo integration.

## Build & Run

```bash
# Build the chat binary
cargo build --bin xf-chat
cargo build --release --bin xf-chat

# Run (requires .env with XF_MYSQL_URL, CHAT_WS_BIND, etc.)
cargo run --bin xf-chat

# Lint
cargo clippy --bin xf-chat

# Format
cargo fmt

# Tests (library-level only, no integration tests)
cargo test --lib
cargo test --lib permission::test   # Permission system tests specifically
```

## Environment

Configured via `.env` (see `.env.example`). Key variables for xf-chat:

- `XF_MYSQL_URL` - MySQL connection to the XenForo 2.x database (required)
- `CHAT_WS_BIND` - Listen address (default `127.0.0.1:8080`)
- `CHAT_WS_URL` / `XF_WS_URL` - Public WebSocket URLs for native/XF clients
- `XF_PUBLIC_URL` - XenForo base URL (used for avatar paths)
- `CHAT_ASSET_DIR` - Path to compiled frontend assets
- `SALT` - Used for CSP nonce generation

## Architecture

### Dual-Mode Design via ChatLayer Trait

The chat system uses a trait abstraction (`ChatLayer`) to support two backends:

```
src/web/chat/implement.rs   - ChatLayer trait definition + default (PostgreSQL) impl
src/bin/xf_chat/xf/mod.rs   - XfLayer struct implements ChatLayer for MySQL/XenForo
```

**Only `XfLayer` is used in production.** The `default::Layer` in `implement.rs` is a development stub with many unimplemented methods (TODOs for delete/edit).

### Actor Model (Actix)

The chat uses Actix's actor system with three core components:

1. **ChatServer** (`src/web/chat/server.rs`) - Singleton actor that owns all state:
   - `connections: HashMap<usize, Connection>` - all live WebSocket connections
   - `rooms: HashMap<u32, HashSet<usize>>` - which connection IDs are in each room
   - `constructor: Constructor` - BbCode parser with pre-loaded smilies
   - Handles all message types (Connect, Disconnect, Join, Post, Edit, Delete, Restart)
   - Broadcasts messages to room members, manages presence

2. **Connection** (`src/web/chat/connection.rs`) - Per-WebSocket actor:
   - Parses slash commands from text frames: `/join`, `/delete`, `/edit`, `/reset`
   - Plain text (non-command) is treated as a chat message to current room
   - Runs heartbeat: pings every 5s (`HEARTBEAT_INTERVAL`), disconnects after 30s silence (`CLIENT_TIMEOUT`)
   - Uses `send_or_reply` to forward messages to ChatServer (handles full mailbox gracefully)

3. **Message types** (`src/web/chat/message.rs`) - Actix message structs:
   - `Connect`, `Disconnect`, `Join`, `Post`, `Edit`, `Delete`, `Restart`, `Reply`
   - `SanitaryPost` / `SanitaryPosts` - server-to-client JSON with rendered BbCode HTML

### Message Flow

```
Client WebSocket → Connection actor (parse command)
    → ChatServer actor (validate permissions, persist via ChatLayer)
    → Broadcast SanitaryPost JSON to room connections
```

Messages are stored as raw BbCode in the database. On output, they pass through the BbCode pipeline: `tokenize()` → `Parser::parse()` → `Constructor::build()` (with smilie replacement). The raw text is also sent as `message_raw` (HTML-sanitized) for edit support.

### Routes (src/web/chat/mod.rs)

| Route | Handler | Binary | Purpose |
|-------|---------|--------|---------|
| `GET /chat.ws` | `view_xf_chat_socket` | xf-chat | WebSocket (reads `xf_session` cookie) |
| `GET /chat.ws` | `view_chat_socket` | forum | WebSocket (reads forum session) |
| `GET /test-chat` | `view_chat_shim` | xf-chat | Standalone chat page with CSP nonce |
| `GET /assets/*` | `view_public_file` | xf-chat | Static assets (directory traversal protected) |
| `GET /chat` | `view_chat` | forum | Chat embedded in forum layout |

## XenForo Shim (`src/bin/xf_chat/xf/`)

The xf-chat binary reads directly from the XenForo MySQL database. It does NOT communicate with XenForo PHP code at runtime — it's a standalone Rust service that shares the same database.

### Session Authentication (`xf/session.rs`)

The most fragile integration point. XenForo stores PHP-serialized session data:

1. Client sends `xf_session` cookie
2. `get_session_key_from_request()` extracts cookie value
3. `get_user_id_from_cookie()` queries `xf_session` table by cookie value
4. Regex parses PHP-serialized `session_data` blob to extract `userId`: `s:6:\\?"?userId\\?"?;i:(?P<user_id>\d+);`
5. `get_session_with_user_id()` fetches user profile from `xf_user` + ignored users from `xf_user_ignored`

Send permission requires: `user_state='valid'`, `is_banned=false`, `message_count>0`.

### Permission System (`xf/permission.rs`, `xf/room.rs`)

Maps XenForo's cached permission system to Rust:

1. On startup, `configure()` registers the HappyBoard permission names into a `Category`
2. Per-request, `can_read_room()` looks up user's `permission_combination_id`, then queries `xf_permission_cache_content` for `hb_chat_room` content type
3. Cached JSON is parsed into `CategoryValues` and checked for `hbChatRoomView`

Permission names (from the HappyBoard/Chat XF addon):
`hbChatRoomView`, `hbChatMessageSend`, `hbChatMessageEditOwn`, `hbChatMessageEditOther`, `hbChatMessageDeleteOwn`, `hbChatMessageDeleteOther`, `hbChatMessageViewDeleted`, `hbChatMessageReport`, `hbChatMessageUndelete`, `hbChatRoomMotd`

### Message CRUD (`xf/message.rs`)

Operates on `xf_hb_chat_message` table. Timestamps use `Decimal(16,6)` for microsecond precision (matching the PHP addon's schema). Deletes are hard deletes (not soft delete like the PHP side).

### XF Database Tables (ORM: `src/bin/xf_chat/xf/orm/`)

- `xf_user` - User accounts
- `xf_session` - Session data (PHP-serialized blobs)
- `xf_hb_chat_message` - Chat messages (HappyBoard addon)
- `xf_hb_chat_room` - Chat rooms
- `xf_permission_cache_content` - XF cached permissions
- `xf_permission_combination` - Permission group combos
- `xf_smilie` - Emoticons
- `xf_user_ignored` - User ignore list

## XenForo Addon Reference

The PHP side lives at `~/Source/xf_kiwifarms/src/addons/HappyBoard/Chat/`. The addon defines the `xf_hb_chat_*` tables, permissions XML, and originally used AJAX polling (not WebSockets). The Rust binary replaces the polling mechanism with real-time WebSockets while sharing the same database schema.

## Key Conventions

- Database keys are `u32`, dates are `i64` (unix timestamps), WebSocket connection IDs are `usize`
- Integer type comments at top of `implement.rs` and `message.rs`
- Avatar URLs follow XF convention: `{XF_PUBLIC_URL}/data/avatars/m/{id/1000}/{id}.jpg?{avatar_date}`
- Guest users have `id: 0`, `username: "Guest"`
- ChatServer mailbox capacity is 32 (`ctx.set_mailbox_capacity(32)`)
- The ChatServer is `Supervised` — it can be restarted by staff via `/reset` command

## Recent Issues

Memory cleanup and WebSocket heartbeat tuning have been ongoing concerns (see recent commit history). Connection pool settings in `main.rs` are tuned for high concurrency: 100 max connections, 5 min, 300s idle/max lifetime.
