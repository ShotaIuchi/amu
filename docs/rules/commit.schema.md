# Commit Message Schema

## Format

```
<type>: <subject>

[body]

[footer]
```

## Type (Required)

| Type | Description |
|------|-------------|
| `feat` | Add new feature |
| `fix` | Bug fix |
| `docs` | Documentation only changes |
| `style` | Changes that do not affect code meaning (whitespace, formatting, etc.) |
| `refactor` | Code changes that neither fix bugs nor add features |
| `perf` | Performance improvements |
| `test` | Add or modify tests |
| `chore` | Changes to build process or auxiliary tools |

## Subject (Required)

- Recommended 50 characters or less
- Do not end with a period
- Use imperative mood

## Body (Optional)

- Describe the reason and background of the change
- Recommended to wrap at 72 characters

## Footer (Optional)

- Breaking changes: `BREAKING CHANGE: <description>`
- Issue references: `Closes #123`, `Fixes #456`

## Examples

```
feat: add user authentication feature

Implemented login functionality using OAuth2.0.
Supports Google and GitHub providers.

Closes #42
```

```
fix: fix error when search results are empty
```

```
chore: bump version to 0.1.6
```
