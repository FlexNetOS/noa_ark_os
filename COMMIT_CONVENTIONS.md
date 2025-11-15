# Commit Message Conventions

This repository uses [Conventional Commits](https://www.conventionalcommits.org/) for all mergeable changes.

## Format

```
<type>(optional scope): <description>
```

- **type** — one of `feat`, `fix`, `docs`, `refactor`, `test`, `build`, `ci`, `chore`, or `revert`.
- **scope** — optional component or directory (e.g., `ui`, `server`, `docs`).
- **description** — short summary written in the imperative mood.

## Examples

- `feat(ui): add ai assist button to kanban cards`
- `docs: document ai provider configuration`
- `ci: run roadmap export in workflow`

## Body and footer

Include additional context in the body when necessary. Reference issues or breaking changes in the footer using the standard Conventional Commits syntax.

## Verification

CI checks validate commit messages on pull requests. Use `pnpm commitlint` (when available) or manually review your message before pushing.
