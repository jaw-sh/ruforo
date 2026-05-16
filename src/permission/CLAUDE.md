# CLAUDE.md

This directory implements ruforo's compact permission model.

The model is organized as collections of categories and items. Permission values are stored as bit flags (`YES`, `NO`, `NEVER`) and combined into masks for fast `can()` checks.

The legacy forum loads permissions from `src/orm/permission_*` tables. The active XenForo chat layer also reuses these data structures to interpret HappyBoard permission cache JSON in `src/bin/xf_chat/xf/permission.rs` and `room.rs`.

Run `cargo test --lib permission::test` after changing flag merging, masks, category ordering, or collection joins.
