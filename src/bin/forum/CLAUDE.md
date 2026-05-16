# CLAUDE.md

This directory contains the legacy `ruforo` web forum binary.

`main.rs` initializes dotenv, logging, ffmpeg, global/session/filesystem modules, the PostgreSQL database, cookie sessions, `ClientCtx`, error handlers, legacy permissions, and all `src/web/` routes.

The default chat layer started here is for the old forum stack and is not the production XenForo chat integration. For current chat work, use `src/bin/xf_chat/` and `src/web/chat/`.

Avoid adding new product behavior here unless the task is specifically about reviving or maintaining the old forum binary.
