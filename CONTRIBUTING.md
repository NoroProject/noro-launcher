# Contributing

Everything a developer needs to build the launcher, point it at a master server,
and ship a release.

## How it fits together

Two binaries come out of this repo:

| Binary | Crate | What it is |
|---|---|---|
| `noro-launcher` | `noro_launcher` | The bootstrapper — the file a player installs |
| `noro-launcher-core` | `noro_core` | The launcher proper, downloaded and started by the bootstrapper |

The split exists because of Windows SmartScreen: the bootstrapper is installed
once and **never updates itself**, so its reputation accumulates. Everything that
changes lives in core, which the bootstrapper re-downloads.

On every start the bootstrapper checks the ed25519 signature of core — not only
after downloading. A bad signature means core is discarded and fetched again.

The rest of the crates make up core:

| Crate | Responsibility |
|---|---|
| `frontend` | The GPUI interface: pages, components, theme |
| `backend` | Sign-in, file sync, Java, launching Minecraft, Discord presence |
| `bridge` | Typed message passing between frontend and backend |
| `mod_link` | Mods and the in-game link to the running client |

`schema`, `i18n` and `mc_mod_utils` are shared with the master server and live in
[noro-shared](https://github.com/NoroProject/noro-shared) — including all
translation strings, so a new UI key is added there, not here.

## Running locally

```bash
cargo run -p noro_core
```

That starts core directly and skips the bootstrapper, which is what you want
almost always. Building the bootstrapper separately is only interesting if you're
changing the splash screen or the signature check.

By default core talks to `http://localhost:8080`. To point it elsewhere:

```bash
NORO_MASTER_URL=https://master.example.com cargo run -p noro_core
```

You need an account on whichever master you point at — the launcher signs in
through it.

`.env.example` lists the variables, but **nothing reads `.env` automatically**;
export them yourself or put them in front of the command.

| Variable | Effect |
|---|---|
| `NORO_MASTER_URL` | Master address |
| `NORO_SIGNING_PUBKEY` | Public key for the core signature check |
| `NORO_SENTRY_DSN` | Crash reporting; empty means the SDK never starts |
| `NORO_SPLASH_PREVIEW=1` | Show the loading window without a real download |
| `DISCORD_APP_ID` | Rich Presence application |

## Where the master address actually comes from

Both the address and the public key are resolved in the same order, most
specific first:

1. **Stamped into the binary** — see below
2. **Environment variable** — `NORO_MASTER_URL` / `NORO_SIGNING_PUBKEY`
3. **`bootstrap.json`** in the launcher's data directory
4. **Compiled in** at build time, via `option_env!`
5. `http://localhost:8080` as a last resort for the address

Step 1 is the interesting one. Releases are built with a 512-byte placeholder
sitting in the binary between the markers `NORO_CFG_START:` and `:NORO_CFG_END`
(see `crates/noro_launcher/src/embedded_config.rs`). When a master hands out a
release, it rewrites the JSON in that slot with **its own** address and public
key.

The practical consequence: the released binaries are not tied to any particular
master. The same file becomes "the launcher for your network" the moment your
master serves it.

## Releasing

Tag and push:

```bash
git tag launcher-v2.0.2
git push origin launcher-v2.0.2
```

`release-launcher.yml` builds five targets — Linux x86_64 and aarch64, macOS
Intel and Apple Silicon, Windows x86_64 — and attaches them to a GitHub Release.

**The tag must match `version` in `Cargo.toml`.** CI fails the run if it doesn't,
because a mismatch leaves players with an "update available" banner that never
goes away: installing it fetches the build they already have.

Getting it to players is a second step, and it happens on the master, not here:

1. In the admin panel, start a launcher build for that tag.
2. The master downloads the release assets, signs core with ed25519, stamps the
   bootstrapper with its own address and key, and stores both.
3. Publish the build — the manifest is rebuilt and re-signed, and
   `/api/launcher/version` starts offering the new version.

Remember that step 3 only reaches **new** installations for the bootstrapper.
Existing ones keep the copy they have; only core updates.

## House rules

Full conventions are in [INSTRUCTIONS.md](INSTRUCTIONS.md). The short version:

- Keep it simple — no speculative abstractions.
- Aim for 150 lines per file; 400 is the hard ceiling. Over 150 wants a one-line
  reason at the top.
- One component, one file.
- Dark theme first, colours from tokens only, every dimension a multiple of 4.
- **UI text is English only**, and it goes through `i18n` keys rather than
  literals — the keys live in `noro-shared`.
- Comments explain *why*, not *what*. If the code says it, the comment shouldn't.

Before opening a pull request:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs exactly these three, and clippy warnings are errors there.

The toolchain is pinned in `rust-toolchain.toml`. It only takes effect through
rustup — if `rustc` comes from Homebrew, the file is ignored and you will drift
from CI.
