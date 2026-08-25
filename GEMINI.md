# GEMINI.md

Core rules are consolidated in **[INSTRUCTIONS.md](./INSTRUCTIONS.md)**.

## Core Mandates
- **Simplicity First.** Avoid cleverness.
- **Strict i18n Localization.** ALL user-facing UI text, headings, labels, button titles, empty states, input placeholders, and field hints MUST be localized via Fluent FTL keys (`useT()` / `t(...)`). Hardcoding UI strings in `.vue` or `.rs` files is strictly forbidden.
- **Surgical Edits.** Use targeted replacements.
- **Validation.** Always run `cargo check` / `cargo test -p i18n` and `bun run typecheck`.
- **File Limits.** Enforce ≤150 lines per file.
- **4-pt Grid.** Strictly adhere to the 4-pt grid for all UI dimensions.
- **No Automatic Git Push / Tagging.** Local `git commit` is allowed and encouraged for history tracking. NEVER perform `git push` or create release tags automatically — push ONLY when explicitly instructed by the user.
- **Forbidden `master-v*` Tags.** NEVER create or push `master-v*` tags. Use only standard `v*` tags (e.g. `v1.7.12`).
