# CLAUDE.md

This directory contains SCSS modules for the browser UI.

`main.scss` imports the partials and is the Vite style entry point. The chat-specific work is mostly in `chat.scss`, with shared layout, nav, form, modal, user, and structural styles split into neighboring files.

When editing styles:

- Prefer changing source SCSS here rather than generated `public/assets/style.css`.
- Keep chat message, user list, MOTD, modal, and form class names synchronized with the TypeScript in `resources/ts/` and the Askama templates.
- `_variables.scss` and `var.scss` define shared values; check them before introducing new ad hoc colors or spacing.
