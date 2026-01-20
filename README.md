# amu

Merge multiple source directories into one target with symlinks.

## Overview

amu merges multiple source directories into a single target directory using symlinks.

![amu demo](demo/amu-demo.gif)

## Installation

### Homebrew

```bash
brew install shotaiuchi/tap/amu
```

### Shell script

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/ShotaIuchi/amu/releases/latest/download/amu-installer.sh | sh
```

### PowerShell

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/ShotaIuchi/amu/releases/latest/download/amu-installer.ps1 | iex"
```

### From source

```bash
cargo install --path .
```

## Usage

### Add a source directory

```bash
# Link ~/dotfiles/claude to ~/.claude
amu add ~/dotfiles/claude ~/.claude

# Or cd to target first
cd ~/.claude
amu add ~/dotfiles/claude

# Preview changes without applying
amu add --dry-run ~/dotfiles/claude ~/.claude
```

### Remove a source directory

```bash
amu remove ~/dotfiles/claude ~/.claude

# Preview changes
amu remove --dry-run ~/dotfiles/claude ~/.claude
```

### Update (reapply) links

```bash
# Update current directory
amu update

# Update specific target
amu update ~/.claude

# Update all targets
amu update --all

# Preview changes
amu update --dry-run
```

### Sync from source

Update all targets that reference a source directory (interactive selection):

```bash
# Sync from current directory
cd ~/work-dotfiles/.claude
git pull
amu sync

# Sync from specific source
amu sync ~/dotfiles/claude

# Preview changes
amu sync --dry-run
```

### Restore links

Restore links from configuration (for new machine setup):

```bash
# Restore current directory
amu restore

# Restore specific target
amu restore ~/.claude

# Restore all targets
amu restore --all

# Preview changes
amu restore --dry-run
```

### List registered sources

```bash
# List current directory (recursive by default)
amu list

# List specific target
amu list ~/.claude

# List all targets
amu list --all

# Non-recursive mode (current target only)
amu list --flat

# Show actual symlinks
amu list ~/.claude --verbose
```

### Check status

```bash
# Check current directory (recursive by default)
amu status

# Check specific target
amu status ~/.claude

# Check all targets
amu status --all

# Non-recursive mode (current target only)
amu status --flat

# JSON output (for scripts)
amu status --json
```

Status checks:
- Link count per source
- Broken symlinks
- Real files (files that should be symlinks)
- Permission issues
- Conflicts

Output example:
```
~/.config:
  ✓ ~/dotfiles/nvim (12 links)
  ! ~/dotfiles/zsh (real files found)
    - .zshrc (expected symlink)

Summary: 1 OK, 1 warning, 0 error
```

### Clear

```bash
# Clear current directory
amu clear

# Clear specific target
amu clear ~/.claude

# Clear all targets
amu clear --all

# Preview changes
amu clear --dry-run
```

## Options

### --dry-run (-n)

Preview changes without applying them. Available for:
- `add`
- `remove`
- `update`
- `sync`
- `restore`
- `clear`

```bash
amu add -n ~/dotfiles/nvim ~/.config/nvim
```

## Behavior

- **Directory conflicts**: Allowed. Files inside are linked individually.
- **File conflicts**: Error. Existing files are not overwritten.

## Configuration

Configuration is stored in `~/.config/amu/config.yaml`:

```yaml
targets:
  ~/.claude:
    - ~/work/.claude
    - ~/personal/.claude
  ~/.config/nvim:
    - ~/dotfiles/nvim
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `AMU_CONFIG` | Override config file path |

## License

MIT
