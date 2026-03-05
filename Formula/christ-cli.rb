class ChristCli < Formula
  desc "A beautiful Bible TUI for Christian developers"
  homepage "https://github.com/whoisyurii/christ-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/whoisyurii/christ-cli/releases/download/v#{version}/christ-aarch64-apple-darwin.tar.gz"
    else
      url "https://github.com/whoisyurii/christ-cli/releases/download/v#{version}/christ-x86_64-apple-darwin.tar.gz"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/whoisyurii/christ-cli/releases/download/v#{version}/christ-aarch64-unknown-linux-gnu.tar.gz"
    else
      url "https://github.com/whoisyurii/christ-cli/releases/download/v#{version}/christ-x86_64-unknown-linux-gnu.tar.gz"
    end
  end

  def install
    bin.install "christ"
  end

  test do
    assert_match "christ-cli", shell_output("#{bin}/christ --version")
  end
end
