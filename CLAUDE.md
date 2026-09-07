# noro-launcher — Agent Guidelines (CLAUDE.md)

Desktop Minecraft launcher: bootstrapper, core daemon, GPUI interface. Public repository, AGPL-3.0.

Detailed repository instructions: **[./INSTRUCTIONS.md](./INSTRUCTIONS.md)**.
Root project ideology and universal rules: **[../INSTRUCTIONS.md](../INSTRUCTIONS.md)**.

## Quick Summary
- **Two Binaries:** `noro-launcher` (bootstrapper, never updates itself for SmartScreen reputation) and `noro-launcher-core` (daemon + GPUI).
- **Public Hygiene:** English comments only; use `example.com` (no private domains/tokens).
- **GPUI Engine:** Pinned GPUI revision; drop old textures with `cx.drop_image(old, None)` to prevent leaks; downscale RGBA images.
- **Verification:** `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo fmt`.

## Commands
```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all
```
