# CLAUDE.md

This module wraps the sibling `bbcode-rs` crate for ruforo rendering.

`ChatBBCode` owns a parser and renderer configured with XenForo smilie replacements supplied by the active `ChatLayer`. Chat messages are stored as raw BBCode and rendered to sanitized HTML before being sent to clients.

The custom `[ditto]...[/ditto]` tag renders as a button and treats its body as verbatim text. Tests in this module cover basic tags, sanitization, smilie replacement, code blocks, and visible-content checks.

When changing rendering behavior, verify both `parse()` and `ChatBBCode` paths, because templates and chat output use them differently.
