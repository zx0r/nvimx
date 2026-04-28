class Nvimx < Formula
  desc "Fast and flexible Neovim profile manager"
  homepage "https://github.com/zx0r/nvimx"
  version "0.1.0"
  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-aarch64-apple-darwin.tar.gz"
      sha256 "fa374e27799458eb3647c81de1232c28ac921cefb5bd7a8ebac5f9fb871f0bab"
    end
    if Hardware::CPU.intel?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-x86_64-apple-darwin.tar.gz"
      sha256 "a7f25702f9db785623161d24c5dc4adc0ce964b7584ac0a5cd2b540f7048f248"
    end
  end
  if OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0ce49971a3ca9f241491006fc1e4763bfff577c1f727bda2fa88e930c0d618e7"
    end
    if Hardware::CPU.intel?
      url "https://github.com/zx0r/nvimx/releases/download/v0.1.0/nvimx-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "e74ff63ec5dba5ca6b770cea55351ad936dde9b5d61e910af869f02bb08a49db"
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
