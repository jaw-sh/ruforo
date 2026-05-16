# CLAUDE.md

This directory contains SeaORM entity models for the XenForo MySQL tables used by `xf-chat`.

These models are separate from `src/orm/`, which represents the legacy ruforo PostgreSQL schema. Files here map XenForo tables such as `xf_user`, `xf_session`, `xf_hb_chat_message`, `xf_hb_chat_room`, `xf_permission_cache_content`, and related permission/smilie tables.

When changing these files:

- Keep column names and Rust types faithful to the XenForo schema.
- Check relation definitions before using `find_also_related`.
- Remember that chat message timestamps use decimal microsecond precision to match the PHP addon.
- Avoid broad regeneration churn unless the task is explicitly a schema sync.
