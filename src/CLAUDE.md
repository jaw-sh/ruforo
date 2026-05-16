# CLAUDE.md

This is the Rust crate root for ruforo.

The current project focus is `xf-chat`: a standalone Actix WebSocket service that uses the shared chat implementation in `src/web/chat/` and the XenForo compatibility layer in `src/bin/xf_chat/xf/`.

Most other modules are legacy forum code for the older `ruforo` binary. They still compile and may be referenced by shared types, templates, or tests, but avoid expanding them unless the task explicitly targets the old forum.

Key shared modules:

- `web/chat/` is the production chat actor, WebSocket, payload, and backend trait code.
- `bbcode/` adapts the local `bbcode-rs` dependency for chat rendering and sanitization.
- `permission/` is a compact permission model used by both legacy code and XenForo permission parsing.
- `orm/` is generated SeaORM for the legacy PostgreSQL forum schema, not XenForo.
