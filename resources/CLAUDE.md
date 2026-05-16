# CLAUDE.md

This directory contains frontend source consumed by Vite.

The important entry points are:

- `resources/ts/chat.ts` for the chat client JavaScript bundle.
- `resources/css/main.scss` for the SCSS bundle.

The compiled output goes to `public/assets/`. The Rust `xf-chat` binary serves those assets for the XenForo chat shim and sends a timestamp to templates for cache busting.

Keep DOM selectors in sync with `templates/chat_shim.html`, `templates/chat/`, and chat-related classes in `resources/css/`.
