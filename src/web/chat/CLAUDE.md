# CLAUDE.md

This directory is the shared chat runtime used by the active `xf-chat` binary.

Core pieces:

- `mod.rs` defines HTTP routes for `/chat.ws`, `/test-chat`, `/assets/*`, and the legacy `/chat` page.
- `connection.rs` is the per-WebSocket Actix actor. It handles heartbeat, room state, slash commands (`/join`, `/edit`, `/delete`, `/motd`, `/w`, `/reset`), and forwards typed messages to `ChatServer`.
- `server.rs` is the singleton chat actor. It owns connections, room membership, BBCode rendering, MOTD state, permission checks, persistence calls, and broadcasts.
- `message.rs` defines Actix message types and JSON payloads sent to the browser.
- `implement.rs` defines the `ChatLayer` trait plus shared DTOs. Production uses `XfLayer`; the default layer is for the legacy forum.
- `watcher.rs` notifies clients when rebuilt frontend assets change.

Any protocol change must be mirrored in `resources/ts/types.ts` and the relevant TypeScript handlers.
