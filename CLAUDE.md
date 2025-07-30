# Claude Development Notes

## Git Commit Style Preferences

When committing: review `git diff`

- Use conventional commit format: `type: subject line`
- Keep subject line concise and descriptive
- No marketing language or promotional text in commit messages
- No "Generated with Claude" or similar attribution in commit messages
- Follow existing project patterns from git log
- Prefer just a subject and no body, unless the change is particularly complex

Example good commit messages from this project:
- `test: allow dead code in test utility methods`
- `fix: improve error handling`
- `feat: add a --fallback option to .static to support SPAs`
- `refactor: remove axum dependency, consolidate unix socket, tcp and tls handling`

## Check Script

Run `./scripts/check.sh` to verify code quality before committing. Use `cargo fmt` to fix formatting issues.
