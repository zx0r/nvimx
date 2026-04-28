class Nvimx < Formula
  desc "A fast and flexible Neovim profile manager for running, isolating, and maintaining multiple configurations with ease"
  homepage "https://github.com/zx0r/nvimx"
  version "0.1.0"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-aarch64-apple-darwin.tar.xz"
      sha256 "b877f42d9fe99e6dbb2f4327e98e66cceda1eeb25b27b317ef23c364c8cb74c8"
    end
    if Hardware::CPU.intel?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-x86_64-apple-darwin.tar.xz"
      sha256 "9cd35f1b79b6c028d7516d5c59285b43608f454dc6c468acfe3576231f5b2c18"
    end
  end
  if OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-aarch64-unknown-linux-gnu.tar.xz"
      sha256 "cd41b541d762490029baac4f5f2eec9c090bf88849269c81439fc021a3b3378b"
    end
    if Hardware::CPU.intel?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "b71c08dc1808f2a59bffd9fbf937cff0c534772995a16baf703b8a989134cc59"
    end
  end
  license "MIT"

  BINARY_ALIASES = {
    "aarch64-apple-darwin": {},
    "aarch64-unknown-linux-gnu": {},
    "aarch64-unknown-linux-musl-dynamic": {},
    "aarch64-unknown-linux-musl-static": {},
    "x86_64-apple-darwin": {},
    "x86_64-unknown-linux-gnu": {},
    "x86_64-unknown-linux-musl-dynamic": {},
    "x86_64-unknown-linux-musl-static": {}
  }

  def target_triple
    cpu = Hardware::CPU.arm? ? "aarch64" : "x86_64"
    os = OS.mac? ? "apple-darwin" : "unknown-linux-gnu"

    "#{cpu}-#{os}"
  end

  def install_binary_aliases!
    BINARY_ALIASES[target_triple.to_sym].each do |source, dests|
      dests.each do |dest|
        bin.install_symlink bin/source.to_s => dest
      end
    end
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "nvimx"
    end
    if OS.mac? && Hardware::CPU.intel?
      bin.install "nvimx"
    end
    if OS.linux? && Hardware::CPU.arm?
      bin.install "nvimx"
    end
    if OS.linux? && Hardware::CPU.intel?
      bin.install "nvimx"
    end

    install_binary_aliases!

    # Homebrew will automatically install these, so we don't need to do that
    doc_files = Dir["README.*", "readme.*", "LICENSE", "LICENSE.*", "CHANGELOG.*"]
    leftover_contents = Dir["*"] - doc_files

    # Install any leftover files in pkgshare; these are probably config or
    # sample files.
    pkgshare.install(*leftover_contents) unless leftover_contents.empty?
  end
end
