# demo

amu のデモ GIF を作成するためのスクリプト集。

## 必要なツール（macOS）

### VHS

ターミナル操作を録画して GIF を生成するツール。

```bash
brew install vhs
```

### ffmpeg（オプション）

GIF のリサイズに使用。

```bash
brew install ffmpeg
```

## 使い方

### 1. デモ環境のセットアップ

```bash
./demo-setup.sh
```

以下のディレクトリとファイルが作成される：
- `~/company/dotdemo/work.conf`
- `~/myself/dotdemo/personal.conf`
- `~/local/hoge/local.conf`
- `~/.demo/hoge/`

### 2. GIF の録画

```bash
vhs demo.tape
```

`amu-demo.gif` が生成される。

### 3. GIF のリサイズ（オプション）

```bash
./resize.sh
```

`amu-demo.min.gif`（900px幅、10fps）が生成される。

### 4. クリーンアップ

```bash
./demo-cleanup.sh
```

デモ用ディレクトリと amu の設定をすべて削除。

## ファイル一覧

| ファイル | 説明 |
|----------|------|
| `demo-setup.sh` | デモ用ディレクトリ・ファイル作成 |
| `demo.tape` | VHS 録画スクリプト |
| `resize.sh` | GIF リサイズスクリプト |
| `demo-cleanup.sh` | デモ環境クリーンアップ |
