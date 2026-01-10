# amu

Merge multiple sources into one target with symlinks using stow.

## Overview

amu manages symlinks for dotfiles by merging multiple source directories into a single target directory. It uses GNU stow internally.

## Installation

### Prerequisites

GNU stow must be installed:

```bash
# macOS
brew install stow

# Ubuntu/Debian
sudo apt install stow

# Arch Linux
sudo pacman -S stow
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
```

### Remove a source directory

```bash
amu remove ~/dotfiles/claude ~/.claude
```

### Update (reapply) links

```bash
# Update specific target
amu update ~/.claude

# Update all targets
amu update

# Update all targets that reference a source
cd ~/work-dotfiles/.claude
git pull
amu update --source .
```

### Restore all links

Restore all links from configuration (for new machine setup):

```bash
amu restore
```

### List registered sources

```bash
# List all
amu list

# List specific target
amu list ~/.claude

# Show actual symlinks
amu list --verbose
```

### Check status

```bash
amu status
```

### Clear

```bash
# Clear current directory
amu clear

# Clear specific target
amu clear ~/.claude

# Clear all targets
amu clear --all
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
