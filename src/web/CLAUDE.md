# CLAUDE.md

This directory contains Actix route handlers and Askama template adapters.

`chat/` is the current production-relevant area. The other route modules (`account`, `forum`, `member`, `post`, `thread`, etc.) belong to the older forum binary and should generally be treated as legacy.

`mod.rs` registers routes in order; order matters because Actix stops at the first match. Template structs here reference files under `templates/`, so keep route data, template fields, and frontend DOM expectations synchronized.
