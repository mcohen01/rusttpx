class Rusttpx < Formula
  desc "A next-generation HTTP client for Rust, inspired by Python's HTTPX"
  homepage "https://github.com/mcohen01/rusttpx"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.0/rusttpx-aarch64-apple-darwin"
      sha256 "PLACEHOLDER_SHA256_FOR_ARM64"
    else
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.0/rusttpx-x86_64-apple-darwin"
      sha256 "PLACEHOLDER_SHA256_FOR_X86_64"
    end
  end

  on_linux do
    url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.0/rusttpx-x86_64-unknown-linux-gnu"
    sha256 "PLACEHOLDER_SHA256_FOR_LINUX"
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "rusttpx-aarch64-apple-darwin" => "rusttpx"
      else
        bin.install "rusttpx-x86_64-apple-darwin" => "rusttpx"
      end
    else
      bin.install "rusttpx-x86_64-unknown-linux-gnu" => "rusttpx"
    end
  end

  test do
    system "#{bin}/rusttpx", "--version"
  end
end 