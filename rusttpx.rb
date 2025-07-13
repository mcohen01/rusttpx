class Rusttpx < Formula
  desc "A next-generation HTTP client for Rust, inspired by Python's HTTPX"
  homepage "https://github.com/mcohen01/rusttpx"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.6/rusttpx-aarch64-apple-darwin"
      sha256 "d4cad6af6d3638ebe41e8d1b8e98028a7c5d56adf9df610abeb3eaa4185f91cc"
    else
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.6/rusttpx-x86_64-apple-darwin"
      sha256 "c67e8bd5d99c59ef5a3a22156fa157818b5f703a1e3843009e0841268ea44b9d"
    end
  end

  on_linux do
    url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.6/rusttpx-x86_64-unknown-linux-gnu"
    sha256 "77c7d687364ef9095d4c56dac22e3acd1cbc0609e5576f226077bfaf7d4dc6fe"
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