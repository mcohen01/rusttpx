use clap::{Parser, ValueEnum};
use http::Method;
use rusttpx::Client;
use std::time::Duration;
use url::Url;

#[derive(Parser)]
#[command(name = "rusttpx")]
#[command(about = "A next-generation HTTP client for Rust")]
#[command(version)]
#[command(disable_version_flag = true)]
struct Cli {
    /// HTTP method to use
    #[arg(short, long, default_value = "get")]
    method: MethodArg,
    
    /// URL to request
    #[arg(value_name = "URL")]
    url: String,
    
    /// Request headers (format: "Name: Value")
    #[arg(short = 'H', long, value_delimiter = ',')]
    headers: Vec<String>,
    
    /// Request body
    #[arg(short, long)]
    body: Option<String>,
    
    /// Content type for the request body
    #[arg(long, default_value = "application/json")]
    content_type: String,
    
    /// Timeout in seconds
    #[arg(short, long, default_value = "30")]
    timeout: u64,
    
    /// Follow redirects
    #[arg(short = 'r', long, default_value = "true")]
    follow_redirects: bool,
    
    /// Disable redirect following
    #[arg(long)]
    no_follow_redirects: bool,
    
    /// Show response headers
    #[arg(long)]
    show_headers: bool,
    
    /// Show response body
    #[arg(long, default_value = "true")]
    show_body: bool,
    
    /// Output format
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,
    
    /// Show version information
    #[arg(short, long)]
    version: bool,
    
    /// Test the client with various endpoints
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(ValueEnum, Clone)]
enum MethodArg {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl From<MethodArg> for Method {
    fn from(method: MethodArg) -> Self {
        match method {
            MethodArg::GET => Method::GET,
            MethodArg::POST => Method::POST,
            MethodArg::PUT => Method::PUT,
            MethodArg::DELETE => Method::DELETE,
            MethodArg::PATCH => Method::PATCH,
            MethodArg::HEAD => Method::HEAD,
            MethodArg::OPTIONS => Method::OPTIONS,
        }
    }
}

#[derive(ValueEnum, Clone)]
enum OutputFormat {
    Text,
    Json,
    Headers,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Test the client with various endpoints
    Test {
        /// Test URL to use
        #[arg(long, default_value = "https://httpbin.org")]
        base_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if cli.version {
        println!("rusttpx {}", env!("CARGO_PKG_VERSION"));
        println!("A next-generation HTTP client for Rust");
        println!("Inspired by Python's HTTPX library");
        return Ok(());
    }

    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(cli.timeout));

    // Configure redirect following
    if cli.no_follow_redirects {
        client_builder = client_builder.no_redirect();
    } else if cli.follow_redirects {
        client_builder = client_builder.redirect(10); // Follow up to 10 redirects
    }

    let client = client_builder.build();

    let url = cli.url.parse::<Url>()?;
    let method: Method = cli.method.clone().into();

    let mut request_builder = client.request(method, url);

    // Add headers
    for header in cli.headers {
        if let Some((name, value)) = header.split_once(':') {
            request_builder = request_builder.header(name.trim(), value.trim())?;
        }
    }

    // Add body if provided
    if let Some(body_content) = cli.body {
        request_builder = request_builder
            .header("Content-Type", &cli.content_type)?
            .text(&body_content)?;
    }

    // Make the request
    let response = request_builder.send().await?;

    // Display results based on format
    match cli.format {
        OutputFormat::Text => {
            if cli.show_headers {
                println!("Status: {}", response.status());
                println!("Headers:");
                for (name, value) in response.headers() {
                    println!("  {}: {}", name, value.to_str().unwrap_or(""));
                }
                println!();
            }
            
            if cli.show_body {
                let body = response.text().await?;
                println!("{}", body);
            }
        }
        OutputFormat::Json => {
            let mut headers_map = std::collections::HashMap::new();
            for (name, value) in response.headers() {
                headers_map.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
            }
            
            let json_response = serde_json::json!({
                "status": response.status().as_u16(),
                "headers": headers_map,
                "body": if cli.show_body { 
                    Some(response.text().await?) 
                } else { 
                    None 
                }
            });
            println!("{}", serde_json::to_string_pretty(&json_response)?);
        }
        OutputFormat::Headers => {
            println!("Status: {}", response.status());
            for (name, value) in response.headers() {
                println!("{}: {}", name, value.to_str().unwrap_or(""));
            }
        }
    }

    if let Some(command) = cli.command {
        match command {
            Commands::Test { base_url } => {
                let base_url = base_url.parse::<Url>()?;
                
                println!("🧪 Testing rusttpx client with {}", base_url);
                println!();
                
                // Test 1: Basic GET request
                println!("1. Testing GET request...");
                let response = client.get(base_url.join("/get")?).send().await?;
                println!("   Status: {}", response.status());
                println!("   Content-Type: {:?}", response.headers().get("content-type").unwrap_or(&"".parse().unwrap()));
                println!();
                
                // Test 2: POST with JSON
                println!("2. Testing POST with JSON...");
                let json_data = serde_json::json!({
                    "test": "rusttpx",
                    "version": "0.1.0"
                });
                let response = client
                    .post(base_url.join("/post")?)
                    .json(&json_data)?
                    .send()
                    .await?;
                println!("   Status: {}", response.status());
                println!();
                
                // Test 3: Custom headers
                println!("3. Testing custom headers...");
                let response = client
                    .get(base_url.join("/headers")?)
                    .header("User-Agent", "rusttpx-cli/0.1.0")?
                    .header("X-Test-Header", "test-value")?
                    .send()
                    .await?;
                println!("   Status: {}", response.status());
                let body = response.text().await?;
                if body.contains("rusttpx-cli") {
                    println!("   ✅ Custom headers sent successfully");
                }
                println!();
                
                // Test 4: Error handling
                println!("4. Testing error handling...");
                match client.get(base_url.join("/status/404")?).send().await {
                    Ok(response) => {
                        println!("   Status: {} (expected 404)", response.status());
                    }
                    Err(e) => {
                        println!("   ❌ Error: {}", e);
                    }
                }
                println!();
                
                println!("✅ All tests completed!");
            }
        }
    }

    Ok(())
} 