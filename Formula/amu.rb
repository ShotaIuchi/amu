class Amu < Formula
  desc "Merge multiple sources into one target with symlinks using stow"
  homepage "https://github.com/USERNAME/amu"
  url "https://github.com/USERNAME/amu/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "stow"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "amu", shell_output("#{bin}/amu --version")
  end
end
