# AGENTS.md

Full project instructions are consolidated in **[INSTRUCTIONS.md](./INSTRUCTIONS.md)**.
All agents must follow the rules defined there.

## Mandatory Agent Rules:
- **Strict i18n Localization:** ALL user-facing text (titles, labels, buttons, badges, input placeholders, field hints, tooltips) MUST be localized using Fluent FTL translation keys in `ru.ftl` and `en.ftl`. Hardcoded strings or untranslated placeholders in UI components are forbidden.
- **No Automatic Git Push / Tagging:** Local `git commit` is encouraged for clean history. However, NEVER perform `git push` or create/push release tags automatically. Push to GitHub ONLY when explicitly requested by the user.
- **Strict Tag Naming (`v*` only):** NEVER create or push tags prefixed with `master-v*`. ONLY standard release tags starting with `v*` (e.g., `v1.7.12`) are permitted.
