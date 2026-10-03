# noro-launcher — Agent Guidelines (GEMINI.md)

Detailed repository instructions: **[./INSTRUCTIONS.md](./INSTRUCTIONS.md)**. The
rules below are the short version; where they differ, INSTRUCTIONS.md wins.

## Core Mandates
- **Simplicity First.** Avoid cleverness and speculative abstractions.
- **Public AGPL-3.0 repository.** Comments in English; no private domains,
  credentials or personal paths (use `example.com`).
- **Strict i18n.** Every player-visible string goes through `i18n::t`. Add the
  keys to `noro-shared` (`en.ftl` and `ru.ftl`) first.
- **GPUI textures.** Drop replaced images with `cx.drop_image(old, None)` and
  downscale images to their display size.
- **File size.** Aim for 150 lines per file; 400 is the hard ceiling.
- **4-pt grid** for every UI dimension.
- **No automatic push or tagging.** Commit locally; push and tag only when
  asked. Release tags are `v*` or `launcher-v*`, never `master-v*`.

## Verification
```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all
```
