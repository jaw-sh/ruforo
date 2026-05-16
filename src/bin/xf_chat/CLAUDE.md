# CLAUDE.md

This directory contains the active `xf-chat` binary.

`main.rs` connects to XenForo MySQL through `XF_MYSQL_URL`, constructs `xf::XfLayer`, starts the shared `ChatServer` actor, starts the asset watcher, configures HappyBoard chat permissions, and exposes only the chat WebSocket, test shim, and asset routes.

Important environment variables are documented in the root `CLAUDE.md`. The service is intentionally standalone: it talks to XenForo by sharing the database, not by calling PHP.

When editing startup behavior, keep the `Arc<dyn ChatLayer>` app data shape intact because the shared chat routes downcast through Actix `Data`.
