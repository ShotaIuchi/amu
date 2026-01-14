#!/bin/bash
# デモ用ソースディレクトリ作成
mkdir -p ~/company/dotdemo
mkdir -p ~/myself/dotdemo
mkdir -p ~/local/hoge

# ターゲットディレクトリ作成
mkdir -p ~/.demo/hoge

# サンプルファイル作成
echo "# company config" > ~/company/dotdemo/work.conf
echo "# myself config" > ~/myself/dotdemo/personal.conf
echo "# local config" > ~/local/hoge/local.conf

echo "Demo setup complete!"
