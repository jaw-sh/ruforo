# CLAUDE.md

This directory is the XenForo compatibility layer for the active chat service.

`XfLayer` implements `ruforo::web::chat::implement::ChatLayer` and adapts the shared chat actor to XenForo tables:

- `session.rs` reads `xf_session` cookies, extracts `userId` from PHP-serialized blobs, loads users, ignored users, staff status, and avatar URLs.
- `room.rs` lists chat rooms, loads history, and resolves room permissions from XenForo permission cache rows.
- `message.rs` inserts, edits, fetches, and hard-deletes `xf_hb_chat_message` rows.
- `permission.rs` registers HappyBoard chat permission names and converts cached JSON booleans into ruforo permission values.
- `smilie.rs` loads XenForo smilie replacement data for BBCode rendering.
- `orm/` maps the XenForo MySQL tables used by this layer.

Schema details must stay aligned with the XenForo HappyBoard Chat addon. Prefer small, explicit conversions at the boundary instead of leaking XenForo ORM models into shared chat code.
