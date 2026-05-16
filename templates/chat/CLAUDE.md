# CLAUDE.md

This directory contains reusable chat template fragments.

`main.html` is the primary chat UI structure embedded by chat pages and shims. `template.html` contains browser-side markup patterns used by the chat frontend.

Keep IDs like `chat-scroller`, `chat-motd`, and `new-message-form` aligned with `resources/ts/chat.ts`, `resources/ts/input.ts`, and the SCSS selectors. Avoid moving script-sensitive elements without updating the TypeScript.
