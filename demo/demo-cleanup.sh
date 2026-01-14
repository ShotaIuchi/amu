#!/bin/bash
# amuの設定をクリア
amu clear --all 2>/dev/null || true

# 一時ディレクトリ削除
rm -rf ~/.demo
rm -rf ~/company/dotdemo
rm -rf ~/myself/dotdemo
rm -rf ~/local/hoge
rmdir ~/company ~/myself ~/local 2>/dev/null || true

echo "Demo cleanup complete!"
