# noro-launcher — Agent Guidelines (AGENTS.md)

Detailed repository instructions: **[./INSTRUCTIONS.md](./INSTRUCTIONS.md)**.
Root project ideology and universal rules: **[../INSTRUCTIONS.md](../INSTRUCTIONS.md)**.

## Mandatory Launcher Rules:
- **Public AGPL-3.0 Repository:** All code comments in English. No private domains, internal credentials, or personal paths (use `example.com`).
- **Strict i18n Localization:** ALL user-visible strings must go through `i18n::t`. Add keys to `noro-shared` (`en.ftl` and `ru.ftl`) first.
- **GPUI Texture Memory:** Drop replaced images in loops via `cx.drop_image(old, None)` when no other `Arc` references remain. Downscale images to display size.
- **No Automatic Git Push / Tagging:** Local `git commit` is encouraged. NEVER `git push` or push release tags automatically without explicit user instruction.
- **Strict Tag Naming (`v*` only):** NEVER use `master-v*` tags. Standard release tags only (e.g., `v1.7.12`).

## Verification:
```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo fmt --all
```
