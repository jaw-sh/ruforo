# CLAUDE.md

This directory contains Actix middleware and extractors for the legacy forum app.

`ClientCtx` authenticates a request from the forum cookie session, loads the user profile, groups, permission data, per-request CSP nonce, and request timing. Route handlers receive it through `FromRequest`, and the middleware inserts the shared context into request extensions.

The active `xf-chat` route uses XenForo cookie/session handling in `src/bin/xf_chat/xf/session.rs`; do not route XenForo auth through `ClientCtx`.

Be careful with middleware ordering in `src/bin/forum/main.rs` if touching this code.
