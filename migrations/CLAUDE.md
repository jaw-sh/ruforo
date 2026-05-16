# CLAUDE.md

This directory contains chronological SQL migrations for the original ruforo PostgreSQL schema.

The production focus is the `xf-chat` binary, which reads XenForo MySQL tables directly and does not apply these migrations at runtime. Treat these files as legacy forum schema history unless the task explicitly targets the old forum application.

When editing migrations:

- Keep `.up.sql` and `.down.sql` pairs aligned.
- Preserve timestamp ordering in filenames.
- Update generated SeaORM entities in `src/orm/` if a legacy PostgreSQL schema change is meant to be used by Rust code.
- Do not model XenForo chat schema changes here; those belong to the XenForo addon/database side and the `src/bin/xf_chat/xf/orm/` models.
