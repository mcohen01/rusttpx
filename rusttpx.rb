class Rusttpx < Formula
  desc "A next-generation HTTP client for Rust, inspired by Python's HTTPX"
  homepage "https://github.com/mcohen01/rusttpx"
  version "0.1.8"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.8/rusttpx-aarch64-apple-darwin"
      sha256 "ae515b13ecd793400ac9f0bb0c45e2175d11eb3136489882b7f2fbed94d610ad"
    else
      url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.8/rusttpx-x86_64-apple-darwin"
      sha256 "eeed842b3c41896b4650cd5a6fecb2defb309d201b41fab9c8dffa7022a7de6f"
    end
  end

  on_linux do
    url "https://github.com/mcohen01/rusttpx/releases/download/v0.1.8/rusttpx-x86_64-unknown-linux-gnu"
    sha256 "ea31bef3f89d4ab803aa9216ffbee017f15b6d7351c8587ee359d14c11779ab3"
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