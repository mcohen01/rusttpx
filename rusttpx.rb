class Rusttpx < Formula
  desc "A next-generation HTTP client for Rust, inspired by Python's HTTPX"
  homepage "https://github.com/mcohen01/rusttpx"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.5/rusttpx-aarch64-apple-darwin"
      sha256 "08c7fe78dff56dd8cddc953c4e66421e99fc7787e1e2364da9f902af554fd407"
    else
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.5/rusttpx-x86_64-apple-darwin"
      sha256 "02d4754079ee99fce9d1bc7ada3873f22bbd2946f111fedace10cc26eacb88c6"
    end
  end

  on_linux do
    url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.5/rusttpx-x86_64-unknown-linux-gnu"
    sha256 "b22f0b2c41d20b3857157979d2f1c55df0e5cca4403573f9edac4ed1296556ee"
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