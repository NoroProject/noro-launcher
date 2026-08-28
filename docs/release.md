# Releasing

The launcher is built here; the master server distributes it. They release
independently, and the master is what players actually download from.

| Workflow | Trigger | Result |
|---|---|---|
| `ci.yml` | every PR and push to `master` | fmt, clippy, tests |
| `release-launcher.yml` | tag `launcher-v*` | launcher binaries on a GitHub Release |

The master **does not compile the launcher**. Its `launcher_builder` pulls the
finished release assets by tag, signs them with ed25519 and puts them in the
file store. Without Actions there is no other way to ship a launcher version.

## Order

1. Tag `launcher-v<version>` and push the tag. `release-launcher.yml` builds
   core and the bootstrapper for all five targets and attaches them to the
   release.
2. In the admin panel, start a launcher build for that tag: the master
   downloads the assets, signs them, and begins serving `/api/launcher/version`.
3. Publish the build — the manifest is rebuilt and re-signed.

The tag must match `version` in `Cargo.toml`; CI checks this and fails the run
if they drift. A mismatch leaves the "update available" banner up permanently,
since installing it fetches the same build again.

If a change touches the manifest format, the master has to be updated before
clients get the new core.

## The bootstrapper is only ever updated by hand

`crates/noro_launcher` is the binary that **never updates itself after the
first install** — that is how it accumulates SmartScreen reputation on Windows.
The master serves it to new installations only; existing ones keep the copy they
have, indefinitely.

So changes to the bootstrapper — the loading window, signature checking,
starting core — never reach installations that already exist. Those have to be
reinstalled. In development it's just a matter of replacing your local binary.
