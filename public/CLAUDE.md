# CLAUDE.md

This directory is the static web root.

`public/index.html` is only a placeholder. Runtime chat assets are built by Vite from `resources/ts/` and `resources/css/` into `public/assets/`, then served by the `xf-chat` binary through `/assets/*` using `CHAT_ASSET_DIR`.

Do not treat generated files under `public/assets/` as source. Change frontend behavior in `resources/ts/` and styling in `resources/css/`, then rebuild with `npm run build`.
