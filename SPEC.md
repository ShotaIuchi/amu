# amu Implementation Specification

## Overview

A CLI tool that merges multiple source directories into a single target directory using symbolic links. Internally uses GNU stow.

Primary use case: dotfiles management (~/.claude/, ~/.config/nvim/, etc.)

## Technology Stack

- Language: Rust
- Distribution: Homebrew
- Dependency: GNU stow (required)

### Crates

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
dirs = "5"
shellexpand = "3"
thiserror = "1"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

## Command Specification

### Help Output

```
amu - Merge multiple source directories into one target with symlinks

Usage: amu <COMMAND>

Commands:
  add      Register a source directory and create symlinks
  remove   Remove symlinks and unregister a source directory
  update   Reapply registered sources for a target
  restore  Restore links from configuration (for new machine setup)
  list     List registered sources
  status   Show status of registered links
  clear    Remove symlinks and clear configuration

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Common Options

#### `--dry-run, -n`

Available for all modifying commands (add, remove, update, restore, clear).
Shows a preview of what would be done without making actual changes.

```bash
amu add ~/work/.claude ~/.claude --dry-run
# [dry-run] add ~/work/.claude -> ~/.claude
#   LINK: agents/reviewer.md => ../../../work/.claude/agents/reviewer.md
```

#### `--all`

Available for commands that handle multiple targets (update, restore, list, status, clear).
Applies to all registered targets.

```bash
amu list --all
amu update --all
```

### `amu add <source> [target]`

Register a source directory and create symlinks.

- `source`: Source directory to link from (required)
- `target`: Target directory to link to (defaults to current directory)
- `--dry-run, -n`: Preview only

```bash
# Explicit specification
amu add ~/work/.claude ~/.claude

# Omit target (current directory becomes target)
cd ~/.claude
amu add ~/work/.claude

# Dry-run
amu add ~/work/.claude ~/.claude -n
```

### `amu remove <source> [target]`

Remove symlinks and unregister from configuration.

- `source`: Source directory to remove (required)
- `target`: Target directory (defaults to current directory)
- `--dry-run, -n`: Preview only

If the source no longer exists, removal from configuration is still performed.

### `amu update [target] [--all] [--source|-s <source>]`

Reapply registered sources (equivalent to stow -R).

- `target`: Target to update (defaults to current directory)
- `--all`: Update all targets
- `--source, -s`: Update all targets that reference the specified source
- `--dry-run, -n`: Preview only

**Normal usage:**
```bash
amu update              # Update current directory
amu update ~/.claude    # Update specific target
amu update --all        # Update all targets
```

**Update from source side:**
```bash
cd ~/work-dotfiles/.claude
git pull
amu update --source .   # Update all targets that reference this source
```

**--source output example:**
```
Updating targets that reference ~/work-dotfiles/.claude:
  ✓ ~/.claude
  ✓ ~/project-a/.claude

Done: 2 target(s) updated
```

### `amu restore [target] [--all]`

Restore links from configuration. For new machine setup.

- `target`: Target to restore (defaults to current directory)
- `--all`: Restore all targets
- `--dry-run, -n`: Preview only

Behavior:
- Creates target directory if it doesn't exist
- Skips sources that don't exist and continues
- Reports success/failure summary at the end

**Output example:**
```
~/.claude:
  ✓ ~/work-dotfiles/.claude
  ✗ ~/personal-dotfiles/.claude (source not found)

~/.config/nvim:
  ✓ ~/dotfiles/nvim

Done: 2 succeeded, 1 failed
```

### `amu list [target] [--all] [--verbose|-v]`

List registered sources.

- `target`: Target to display (defaults to current directory)
- `--all`: Display all targets
- `--verbose, -v`: Also show actual symlinks

**Basic output:**
```
~/.claude:
  - ~/work/.claude
  - ~/personal/.claude
```

**--verbose output:**
```
~/.claude:
  sources:
    - ~/work/.claude
    - ~/personal/.claude
  links:
    ~/.claude/agents/reviewer.md -> ~/work/.claude/agents/reviewer.md
    ~/.claude/agents/planner.md -> ~/personal/.claude/agents/planner.md
    ~/.claude/commands/test.md -> ~/personal/.claude/commands/test.md
```

### `amu status [target] [--all] [--json]`

Show link status. Detects broken links and unapplied changes.

- `target`: Target to check (defaults to current directory)
- `--all`: Check all targets
- `--json`: Output in JSON format

**Detected states:**

| State | Description |
|-------|-------------|
| Ok | Normal (displays link count) |
| SourceNotFound | Source directory does not exist |
| TargetNotFound | Target directory does not exist |
| BrokenLinks | Broken symbolic links exist |
| RealFiles | Real files exist where symlinks are expected |
| Conflicts | Conflicts would occur when running stow |
| PermissionDenied | Permission error |

**Normal output:**
```
~/.claude:
  ✓ ~/work/.claude (3 links)
  ✗ ~/personal/.claude (source not found)

Summary: 1 OK, 0 warning, 1 error
```

**--json output:**
```json
{
  "targets": [
    {
      "path": "~/.claude",
      "sources": [
        {"path": "~/work/.claude", "status": "ok", "link_count": 3},
        {"path": "~/personal/.claude", "status": "error", "message": "source not found"}
      ]
    }
  ],
  "summary": {"ok": 1, "warning": 0, "error": 1}
}
```

Exits with code 1 if there are errors or warnings.

### `amu clear [target] [--all]`

Remove symlinks and delete from configuration. Batch version of remove.

- `target`: Target to clear (defaults to current directory)
- `--all`: Clear all targets
- `--dry-run, -n`: Preview only

```bash
amu clear              # Clear current directory
amu clear ~/.claude    # Clear specific target
amu clear --all        # Clear all targets
```

## Configuration File

### Location

`~/.config/amu/config.yaml`

Can be overridden with the `AMU_CONFIG` environment variable.

### Format

```yaml
targets:
  /Users/username/.claude:
    - /Users/username/work/.claude
    - /Users/username/personal/.claude
  /Users/username/.config/nvim:
    - /Users/username/dotfiles/nvim
```

### Behavior

- Operates with empty state if file doesn't exist
- Automatically created on `add`
- Paths are stored as absolute paths (`~` is expanded)
- Parent directory of config file is created automatically if needed

## stow Dependency

### Startup Check

Checks for `stow` existence before executing any command.

### Error Message When Not Installed

```
Error: stow is not installed

Install with:
  macOS:  brew install stow
  Ubuntu: sudo apt install stow
  Arch:   sudo pacman -S stow
```

### stow Invocation

```bash
# On add (--no-folding preserves directory structure)
stow --no-folding -t <target> -d <source_parent> <source_dirname>

# On remove
stow --no-folding -D -t <target> -d <source_parent> <source_dirname>

# On update
stow --no-folding -R -t <target> -d <source_parent> <source_dirname>

# On dry-run
stow -n -v --no-folding [-D|-R] -t <target> -d <source_parent> <source_dirname>
```

The `--no-folding` option ensures stow creates symlinks to individual files rather than symlinks to directories.

## Error Handling

### On Conflict

Follows stow's default behavior and stops with an error. Does not overwrite.

### Error Messages

| Situation | Message |
|-----------|---------|
| stow not installed | `stow is not installed` + installation instructions |
| source doesn't exist | `Source directory does not exist: <path>` |
| target doesn't exist | `Target directory does not exist: <path>` |
| already registered | `Already registered: <source> -> <target>` |
| not registered | `Not registered: <source> -> <target>` |
| config parse error | `Failed to parse config file: <details>` |
| config save error | `Failed to save config file: <details>` |
| stow command failed | `stow command failed: <details>` |
| IO error | `IO error: <details>` |

## Homebrew Distribution

### Formula

```ruby
class Amu < Formula
  desc "Merge multiple sources into one target with symlinks using stow"
  homepage "https://github.com/ShotaIuchi/amu"
  url "https://github.com/ShotaIuchi/amu/archive/v<version>.tar.gz"
  sha256 "<hash>"
  license "MIT"

  depends_on "stow"
  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/amu"
  end
end
```

## Directory Structure

```
amu/
├── Cargo.toml
├── src/
│   ├── main.rs       # Entry point, command execution
│   ├── cli.rs        # clap definitions
│   ├── config.rs     # Config file I/O, path processing
│   ├── stow.rs       # stow wrapper
│   └── error.rs      # Error types (using thiserror)
├── README.md
├── SPEC.md
└── Formula/
    └── amu.rb
```

## Notes

### Path Processing

- All paths are normalized to absolute paths before storage
- `~` is expanded before storage
- Displayed with `~` abbreviation (abbreviate_path)

### Default Behavior

When target is omitted, the current directory is used.
Use `--all` flag to target all registered targets.

### Exit Codes

- 0: Success
- 1: Error or warning present (status command)
