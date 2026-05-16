# CLAUDE.md

This directory contains the vanilla TypeScript chat client.

`chat.ts` is the entry point. It imports SCSS, wires modules together on `DOMContentLoaded`, configures WebSocket callbacks, handles room joins from `window.location.hash`, and tracks per-room permissions from the server.

Module responsibilities:

- `ws.ts` manages WebSocket connection, reconnects, room joins, and pending message tracking.
- `messages.ts` builds, edits, removes, and cleans up chat message DOM.
- `input.ts` owns the textarea/form submission flow.
- `toolbar.ts`, `mentions.ts`, `users.ts`, and `scroll.ts` manage focused UI behaviors.
- `types.ts` mirrors server JSON payloads from `src/web/chat/message.rs` and `src/web/chat/implement.rs`.

Keep protocol changes synchronized with the Rust chat actor and message structs.
