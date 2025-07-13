# rusttpx

A next-generation HTTP client for Rust, inspired by Python's HTTPX library. Built with async/await support, comprehensive HTTP features, and a beautiful CLI interface.

## Features

### Core HTTP Client
- **Async/await support** - Built on Tokio for high-performance async operations
- **HTTP/1.1 and HTTP/2** - Modern protocol support via reqwest
- **TLS/SSL support** - Secure connections with configurable certificates
- **Request/Response streaming** - Handle large payloads efficiently
- **Cookie management** - Automatic cookie handling and persistence
- **Proxy support** - HTTP and SOCKS proxy configuration
- **Timeout handling** - Configurable connection and request timeouts
- **Redirect following** - Automatic redirect handling with configurable limits
- **Middleware support** - Extensible request/response processing pipeline

### CLI Features
- **Beautiful JSON colorization** - Syntax highlighting by data type (keys, strings, numbers, booleans)
- **Auto-detection** - Automatically detects and colorizes JSON responses
- **Status code coloring** - Visual status indicators (green for 2xx, yellow for 3xx, red for 4xx/5xx)
- **Header display** - Optional response header viewing with color coding
- **Multiple HTTP methods** - GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- **Custom headers** - Easy header injection with `-H` flag
- **Request body support** - Send JSON, form data, or custom content
- **Content-Type handling** - Automatic and manual content type specification

## Installation

### From Source
```bash
git clone https://github.com/yourusername/rusttpx.git
cd rusttpx
cargo build --release
```

### Using Cargo
```bash
cargo install --git https://github.com/yourusername/rusttpx.git
```

### Homebrew (Coming Soon)
```bash
brew install rusttpx
```

## Usage

### Basic Examples

```bash
# Simple GET request
rusttpx https://httpbin.org/get

# POST with JSON body
rusttpx -m post -b '{"name": "test", "value": 123}' https://httpbin.org/post

# Custom headers
rusttpx -H "User-Agent: MyApp/1.0" -H "Authorization: Bearer token123" https://httpbin.org/headers

# Show response headers
rusttpx --show-headers https://httpbin.org/json

# Custom timeout
rusttpx --timeout 10 https://httpbin.org/delay/5
```

### Advanced Examples

```bash
# PUT request with file upload simulation
rusttpx -m put -b '{"id": 1, "status": "updated"}' https://httpbin.org/put

# DELETE request
rusttpx -m delete https://httpbin.org/delete

# PATCH request
rusttpx -m patch -b '{"status": "partial"}' https://httpbin.org/patch

# Custom content type
rusttpx -m post --content-type "application/xml" -b '<data>test</data>' https://httpbin.org/post

# Disable redirect following
rusttpx --no-follow-redirects https://httpbin.org/redirect/3
```

## CLI Reference

```
rusttpx [OPTIONS] <URL>

A next-generation HTTP client for Rust

Arguments:
  <URL>  URL to request

Options:
  -m, --method <METHOD>                    HTTP method to use [default: get]
  -H, --headers <HEADERS>                  Request headers (format: "Name: Value")
  -b, --body <BODY>                        Request body
      --content-type <CONTENT_TYPE>        Content type for the request body [default: application/json]
  -t, --timeout <TIMEOUT>                  Timeout in seconds [default: 30]
  -r, --follow-redirects                   Follow redirects [default: true]
      --no-follow-redirects                Disable redirect following
      --show-headers                       Show response headers
      --show-body                          Show response body [default: true]
  -v, --version                            Show version information
  -h, --help                               Print help

HTTP Methods:
  get, post, put, delete, patch, head, options

Examples:
  rusttpx https://httpbin.org/get
  rusttpx -m post -b '{"key": "value"}' https://httpbin.org/post
  rusttpx -H "User-Agent: MyApp/1.0" https://httpbin.org/headers
  rusttpx --show-headers https://httpbin.org/json
```

## Library Usage

```rust
use rusttpx::Client;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with custom configuration
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(10)
        .build();

    // Make a GET request
    let response = client.get("https://httpbin.org/get")
        .header("User-Agent", "rusttpx/0.1.0")?
        .send()
        .await?;

    println!("Status: {}", response.status());
    println!("Body: {}", response.text().await?);

    // Make a POST request with JSON
    let json_data = serde_json::json!({
        "name": "test",
        "value": 123
    });

    let response = client.post("https://httpbin.org/post")
        .json(&json_data)?
        .send()
        .await?;

    println!("Response: {}", response.text().await?);

    Ok(())
}
```

## Features in Detail

### JSON Colorization
The CLI automatically detects JSON responses and applies syntax highlighting:
- **Keys**: Yellow
- **Strings**: Green  
- **Numbers**: Bright Blue
- **Booleans**: Bright Magenta
- **Null**: Bright Red
- **Syntax**: White (braces, brackets, colons, commas)

### Status Code Coloring
- **2xx (Success)**: Green
- **3xx (Redirect)**: Yellow
- **4xx/5xx (Error)**: Red

### Header Display
When using `--show-headers`, response headers are displayed with:
- **Header names**: Cyan
- **Header values**: White

## Configuration

### Environment Variables
- `NO_COLOR`: Disable color output (if set, no colors will be displayed)

### Timeouts
- Default timeout: 30 seconds
- Configurable via `--timeout` flag
- Affects both connection and request timeouts

### Redirects
- Default: Follow up to 10 redirects
- Disable with `--no-follow-redirects`
- Configure limit in library usage

## Testing

Run the built-in test suite:
```bash
cargo run --features cli -- test
```

This will test various endpoints and features of the client.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by Python's [HTTPX](https://github.com/encode/httpx) library
- Built on top of [reqwest](https://github.com/seanmonstar/reqwest) for HTTP functionality
- Uses [clap](https://github.com/clap-rs/clap) for CLI argument parsing
- JSON colorization inspired by [httpie](https://httpie.io/)

## Roadmap

- [ ] HTTP/2 server push support
- [ ] WebSocket support
- [ ] Request/response streaming improvements
- [ ] More authentication methods
- [ ] Request/response caching
- [ ] Performance benchmarking tools
- [ ] Plugin system for custom middleware
