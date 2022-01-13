# ruforo - The most user friendly
_Name pending._

PROJECT_NAME is a traditional web forum built in Rust.

## Stack
 - Rust
   - Actix-Web
   - Askama for Templating
   - SeaQL (sqlx) for ORM
 - Postgres
 - Webpack

## Aspirations
 - Minimal bloat.
 - No-JS, Tor compatability.
 - Unit tested.
 - Event driven WebSocket subscriptions.
 - Total replacement for XenForo.

## Environment
 - Example `.env` file
   + NOTE: AWS variables will likely be migrated to DB
```
DATABASE_URL=postgres://rfuser:rfpass@localhost/ruforo
SALT=GPIb5gy10Vw/SEj5f+cjeA
AWS_ACCESS_KEY_ID=testaccesskey
AWS_SECRET_ACCESS_KEY=testsecretkey
DIR_TMP=./tmp     # used for file uploads
SESSION_TIME=1440 # 1 day in minutes
```
 - node and webpack
   + Install [npm](https://nodejs.org/en/download/).
   + Run `npm install` from the root directory to install node dependencies.
   + Run `npx webpack` from the root directory to deploy browser-friendly resource files.

### WebM Validation Notes
 - https://www.webmproject.org/docs/container/
 - VP8
 - VP9
 - AV1
 - OPUS
 - VORBIS

## Contributions
### Code Guidelines
 - We use [rustfmt](https://github.com/rust-lang/rustfmt).
 - `cargo clippy` whenever possible.
 - Try to eliminate warnings.

### Database Guidelines
 - Any data which would apply to two types of content (i.e. posts, chat messages, profile posts) should interact with the `ugc` tables, not individual content type tables.
 - Usernames should be referenced by `user_id,created_at DESC` from `user_name`. User rows can be deleted, but a historical reference for their name will be added to this table. This complies with [GDPR software requirements](https://gdpr.eu/right-to-be-forgotten).
