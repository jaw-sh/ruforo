# CLAUDE.md

This directory contains executable entry points.

`xf_chat/` is the active production target. It starts the XenForo-compatible chat WebSocket service and should receive almost all new work.

`forum/` contains the older `ruforo` forum server. It wires the legacy PostgreSQL forum, sessions, templates, permissions, and default chat layer; treat it as legacy unless the task explicitly asks for forum work.

Build the active binary with `cargo build --bin xf-chat`. Use `cargo build --bin ruforo` only for legacy forum changes.
