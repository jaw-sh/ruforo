# CLAUDE.md

This directory contains generated SeaORM entities for the legacy PostgreSQL forum schema.

It backs the old `ruforo` binary and modules such as users, posts, threads, attachments, sessions, UGC, and legacy chat tables. It is not the XenForo MySQL mapping used by production chat; that lives under `src/bin/xf_chat/xf/orm/`.

Avoid hand-editing generated entity files unless the change is deliberately narrow. If a migration changes the legacy schema, keep these entities and their relations synchronized with `migrations/`.
