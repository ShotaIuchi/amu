# dotlink

Dotfiles linker using GNU stow.

## Overview

dotlink manages symlinks for dotfiles by merging multiple source directories into a single target directory. It uses GNU stow internally.

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
dotlink add ~/dotfiles/claude ~/.claude

# Or cd to target first
cd ~/.claude
dotlink add ~/dotfiles/claude
```

### Remove a source directory

```bash
dotlink remove ~/dotfiles/claude ~/.claude
```

### Update (reapply) links

```bash
# Update specific target
dotlink update ~/.claude

# Update all targets
dotlink update
```

### List registered sources

```bash
# List all
dotlink list

# List specific target
dotlink list ~/.claude
```

### Check status

```bash
dotlink status
```

## Behavior

- **Directory conflicts**: Allowed. Files inside are linked individually.
- **File conflicts**: Error. Existing files are not overwritten.

## Configuration

Configuration is stored in `~/.config/dotlink/config.yaml`:

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
| `DOTLINK_CONFIG` | Override config file path |

## License

MIT
