#!/bin/bash
# Create demo source directories
mkdir -p ~/company/dotdemo
mkdir -p ~/myself/dotdemo
mkdir -p ~/local/hoge

# Create target directories
mkdir -p ~/.demo/hoge

# Create sample files
echo "# company config" > ~/company/dotdemo/work.conf
echo "# myself config" > ~/myself/dotdemo/personal.conf
echo "# local config" > ~/local/hoge/local.conf

echo "Demo setup complete!"
