# CLAUDE.md

This directory contains Askama templates for server-rendered pages and chat shims.

The production chat path mainly uses `chat_shim.html`, `chat/`, and `container/shim.html`. Many forum, account, member, post, and thread templates support the legacy `ruforo` binary.

When editing templates:

- Keep Askama struct fields in `src/web/` synchronized with template variables.
- Preserve CSP nonce handling where scripts or inline styles are present.
- Keep DOM IDs/classes required by `resources/ts/` and `resources/css/` stable.
