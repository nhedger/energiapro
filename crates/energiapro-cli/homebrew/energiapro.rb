class Energiapro < Formula
  desc "Command-line interface for the EnergiaPro API"
  homepage "https://github.com/nhedger/energiapro"
  version "{{version}}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/nhedger/energiapro/releases/download/cli-v#{version}/energiapro-aarch64-apple-darwin"
      sha256 :no_check
    else
      url "https://github.com/nhedger/energiapro/releases/download/cli-v#{version}/energiapro-x86_64-apple-darwin"
      sha256 :no_check
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/nhedger/energiapro/releases/download/cli-v#{version}/energiapro-x86_64-unknown-linux-gnu"
      sha256 :no_check
    else
      odie "Unsupported Linux architecture: #{Hardware::CPU.arch}"
    end
  end

  def install
    binary = if OS.mac? && Hardware::CPU.arm?
      "energiapro-aarch64-apple-darwin"
    elsif OS.mac? && Hardware::CPU.intel?
      "energiapro-x86_64-apple-darwin"
    elsif OS.linux? && Hardware::CPU.intel?
      "energiapro-x86_64-unknown-linux-gnu"
    else
      odie "Unsupported platform"
    end

    bin.install binary => "energiapro"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/energiapro --version")
  end
end
