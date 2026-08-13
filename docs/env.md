# Environment Variables

## Master

`noro-master` reads `.env` from the current working directory before starting.
When running from the repo root with `cargo run -p master`, copy `.env.example`
to `.env` in the repo root.

These five are **required** — `Config::from_env` fails and the master does not
start without them:

```env
DATABASE_URL=postgres://postgres:postgres@localhost/noro
DISCORD_CLIENT_ID=...
DISCORD_CLIENT_SECRET=...
NORO_PUBLIC_URL=http://localhost:8080
NORO_WEB_URL=http://localhost:3000
```

They used to have defaults. That was a bug, not a convenience: an instance
started without `NORO_PUBLIC_URL` came up healthy and handed out `localhost`
download links inside *signed* manifests, and one without `NORO_WEB_URL` sent
players to the domain of whoever wrote the fallback. A missing variable is now a
startup error naming the variable.

`NORO_WEB_URL` is where `/oauth2/authorize` redirects for the consent screen.
The launcher no longer needs to know the site address at all — it opens the
master and follows the redirect.

Discord redirect URLs to register:

```text
http://localhost:8080/auth/discord/callback
http://localhost:8080/auth/discord/launcher/callback
```

## CORS

`NORO_ALLOWED_ORIGINS` is a comma-separated list of browser origins allowed to
call the API:

```env
NORO_ALLOWED_ORIGINS=https://noro.example.dev
```

Leave it unset in local development — the master then accepts any origin and
logs a warning at startup. **Set it in production.** The Yggdrasil and agent
endpoints are unaffected either way: game servers and agents call them
server-side and never send an `Origin` header.

## File delivery / CDN

By default every artifact is served by the master itself from
`GET /files/{sha1}`. That endpoint supports `Range`, returns the SHA1 as its
`ETag` and marks responses `immutable`, so it works as a CDN origin as-is.

To move the traffic off the master, point a pull-through CDN at it and set:

```env
NORO_FILES_CDN_URL=https://cdn.example.com
```

Manifests then hand out `{NORO_FILES_CDN_URL}/{sha1}` instead of
`{NORO_PUBLIC_URL}/files/{sha1}`. Nothing else changes: URLs are
content-addressed, so a changed file gets a new URL and the CDN never needs a
purge. Point the CDN's origin at `https://<master>/files/` and leave its cache
key as the path.

Direct S3/R2 upload also exists, but today only server icons and backgrounds
are pushed there — build artifacts are not, so setting these alone will **not**
serve a build:

```env
NORO_S3_ENDPOINT=...
NORO_S3_BUCKET=...
NORO_S3_ACCESS_KEY=...
NORO_S3_SECRET_KEY=...
NORO_S3_REGION=auto
NORO_S3_PUBLIC_URL=https://pub-....r2.dev
```

Set none of them and S3 is simply off. Set *any* of them and the rest become
required, `NORO_S3_PUBLIC_URL` included: a half-filled block used to be read as
"S3 disabled" and files went to the master's disk instead of the bucket, while
`NORO_S3_PUBLIC_URL` used to be guessed as `{endpoint}/{bucket}` — a URL that
does not resolve on R2, baked into signed manifests.

## Web

Nuxt reads env from `web/.env`. Copy `web/.env.example` to `web/.env`.

```env
NUXT_PUBLIC_MASTER_URL=http://localhost:8080
NUXT_PUBLIC_WEB_URL=http://localhost:3000
```

In a production build these are required: `useApi` throws a named error instead
of falling back to a hardcoded domain. In `nuxt dev` an unset variable still
resolves to the local master and site.

## Nuxt dev server on macOS

`bun run dev` sets `TMPDIR=/tmp`. Without it Nuxt fails on startup with

```text
Error: connect EINVAL /var/folders/.../nuxt-vite-node-<pid>-<ts>.sock
```

macOS gives each user a ~49-character `TMPDIR`, Nuxt appends another 61 for its
vite-node IPC socket, and the total exceeds the 104-byte `sun_path` limit in
`sys/un.h` — `connect()` then fails with `EINVAL`. A shorter `TMPDIR` keeps the
path inside the limit.

## Admin CLI

`noro-admin` reads normal process env:

```env
NORO_MASTER_URL=http://localhost:8080
NORO_ADMIN_TOKEN=...
```

The CLI does not load `.env` by itself, so export these in your shell or run
through an env loader if needed.
