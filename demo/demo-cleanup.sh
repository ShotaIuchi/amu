#!/bin/bash
# Clear amu configuration
amu clear --all 2>/dev/null || true

# Delete temporary directories
rm -rf ~/.demo
rm -rf ~/company/dotdemo
rm -rf ~/myself/dotdemo
rm -rf ~/local/hoge
rmdir ~/company ~/myself ~/local 2>/dev/null || true

echo "Demo cleanup complete!"
