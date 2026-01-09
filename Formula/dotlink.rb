class Dotlink < Formula
  desc "Dotfiles linker using GNU stow"
  homepage "https://github.com/USERNAME/dotlink"
  url "https://github.com/USERNAME/dotlink/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256"
  license "MIT"

  depends_on "rust" => :build
  depends_on "stow"

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "dotlink", shell_output("#{bin}/dotlink --version")
  end
end
