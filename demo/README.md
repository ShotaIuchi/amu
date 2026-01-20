# demo

Scripts for creating amu demo GIF.

## Required Tools (macOS)

### VHS

A tool for recording terminal operations and generating GIFs.

```bash
brew install vhs
```

### ffmpeg (Optional)

Used for resizing GIFs.

```bash
brew install ffmpeg
```

## Usage

### 1. Setup Demo Environment

```bash
./demo-setup.sh
```

The following directories and files will be created:
- `~/company/dotdemo/work.conf`
- `~/myself/dotdemo/personal.conf`
- `~/local/hoge/local.conf`
- `~/.demo/hoge/`

### 2. Record GIF

```bash
vhs demo.tape
```

`amu-demo.gif` will be generated.

### 3. Resize GIF (Optional)

```bash
./resize.sh
```

`amu-demo.min.gif` (900px width, 10fps) will be generated.

### 4. Cleanup

```bash
./demo-cleanup.sh
```

Deletes all demo directories and amu configuration.

## File List

| File | Description |
|------|-------------|
| `demo-setup.sh` | Create demo directories and files |
| `demo.tape` | VHS recording script |
| `resize.sh` | GIF resize script |
| `demo-cleanup.sh` | Demo environment cleanup |
