use clap::{Parser, ValueEnum};
use http::Method;
use rusttpx::Client;
use std::time::Duration;
use url::Url;
use colored::*;

fn print_basic_colorized_json(json: &str) {
    // Parse the JSON to get proper data type information
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => {
            print_json_value(&value, 0);
        }
        Err(_) => {
            // Fallback to simple line-by-line if parsing fails
            for line in json.lines() {
                println!("{}", line);
            }
        }
    }
}

fn print_json_value(value: &serde_json::Value, indent: usize) {
    let indent_str = "  ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            println!("{}{}", indent_str, "{".white());
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (i, (key, val)) in entries.iter().enumerate() {
                let is_last = i == entries.len() - 1;
                print!("{}  {}\"{}\"{}: ",
                    indent_str,
                    "".white(),
                    key.yellow(),
                    "".white()
                );
                print_json_value(val, indent + 1);
                if !is_last {
                    print!("{}", ",".white());
                }
                println!();
            }
            print!("{}{}", indent_str, "}".white());
        }
        serde_json::Value::Array(arr) => {
            println!("{}{}", indent_str, "[".white());
            for (i, item) in arr.iter().enumerate() {
                let is_last = i == arr.len() - 1;
                print_json_value(item, indent + 1);
                if !is_last {
                    print!("{}", ",".white());
                }
                println!();
            }
            print!("{}{}", indent_str, "]".white());
        }
        serde_json::Value::String(s) => {
            print!("\"{}\"", s.green());
        }
        serde_json::Value::Number(n) => {
            print!("{}", n.to_string().bright_blue());
        }
        serde_json::Value::Bool(b) => {
            print!("{}", b.to_string().bright_magenta());
        }
        serde_json::Value::Null => {
            print!("{}", "null".bright_red());
        }
    }
}

#[derive(Parser)]
#[command(name = "rusttpx")]
#[command(about = "A next-generation HTTP client for Rust")]
#[command(version)]
#[command(disable_version_flag = true)]
struct Cli {
    /// URL to request
    #[arg(value_name = "URL")]
    url: Option<String>,
    
    /// HTTP method to use
    #[arg(short, long, default_value = "get")]
    method: MethodArg,
    
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

    // Handle subcommands first
    if let Some(command) = cli.command {
        match command {
            Commands::Test { base_url } => {
                let base_url = base_url.parse::<Url>()?;
                
                println!("{}", "🧪 Testing rusttpx client".blue().bold());
                println!("Base URL: {}", base_url.to_string().cyan());
                println!();
                
                // Test 1: Basic GET request
                println!("{}", "1. Testing GET request...".yellow());
                let client = Client::builder()
                    .timeout(Duration::from_secs(cli.timeout))
                    .build();
                let response = client.get(base_url.join("/get")?).send().await?;
                let status = response.status();
                let status_str = match status.as_u16() {
                    200..=299 => format!("{}", status).green().bold(),
                    300..=399 => format!("{}", status).yellow().bold(),
                    400..=599 => format!("{}", status).red().bold(),
                    _ => format!("{}", status).white().bold(),
                };
                println!("   Status: {}", status_str);
                println!("   Content-Type: {:?}", response.headers().get("content-type").unwrap_or(&"".parse().unwrap()));
                println!();
                
                // Test 2: POST with JSON
                println!("{}", "2. Testing POST with JSON...".yellow());
                let json_data = serde_json::json!({
                    "test": "rusttpx",
                    "version": "0.1.0"
                });
                let response = client
                    .post(base_url.join("/post")?)
                    .json(&json_data)?
                    .send()
                    .await?;
                let status = response.status();
                let status_str = match status.as_u16() {
                    200..=299 => format!("{}", status).green().bold(),
                    300..=399 => format!("{}", status).yellow().bold(),
                    400..=599 => format!("{}", status).red().bold(),
                    _ => format!("{}", status).white().bold(),
                };
                println!("   Status: {}", status_str);
                println!();
                
                // Test 3: Custom headers
                println!("{}", "3. Testing custom headers...".yellow());
                let response = client
                    .get(base_url.join("/headers")?)
                    .header("User-Agent", "rusttpx-cli/0.1.0")?
                    .header("X-Test-Header", "test-value")?
                    .send()
                    .await?;
                let status = response.status();
                let status_str = match status.as_u16() {
                    200..=299 => format!("{}", status).green().bold(),
                    300..=399 => format!("{}", status).yellow().bold(),
                    400..=599 => format!("{}", status).red().bold(),
                    _ => format!("{}", status).white().bold(),
                };
                println!("   Status: {}", status_str);
                let body = response.text().await?;
                if body.contains("rusttpx-cli") {
                    println!("   {} Custom headers sent successfully", "✅".green());
                }
                println!();
                
                // Test 4: Error handling
                println!("{}", "4. Testing error handling...".yellow());
                match client.get(base_url.join("/status/404")?).send().await {
                    Ok(response) => {
                        let status = response.status();
                        let status_str = format!("{}", status).red().bold();
                        println!("   Status: {} (expected 404)", status_str);
                    }
                    Err(e) => {
                        println!("   {} Error: {}", "❌".red(), e.to_string().red());
                    }
                }
                println!();
                
                println!("{}", "✅ All tests completed!".green().bold());
                return Ok(());
            }
        }
    }

    // Handle regular HTTP request
    let url = cli.url.ok_or("URL is required. Use 'rusttpx <URL>' or 'rusttpx --help' for more information.")?;

    let mut client_builder = Client::builder()
        .timeout(Duration::from_secs(cli.timeout));

    // Configure redirect following
    if cli.no_follow_redirects {
        client_builder = client_builder.no_redirect();
    } else if cli.follow_redirects {
        client_builder = client_builder.redirect(10); // Follow up to 10 redirects
    }

    let client = client_builder.build();

    let url = url.parse::<Url>()?;
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

    // Display results
    if cli.show_headers {
        // Colorize status code
        let status = response.status();
        let status_str = match status.as_u16() {
            200..=299 => format!("{}", status).green().bold(),
            300..=399 => format!("{}", status).yellow().bold(),
            400..=599 => format!("{}", status).red().bold(),
            _ => format!("{}", status).white().bold(),
        };
        println!("Status: {}", status_str);
        
        println!("Headers:");
        for (name, value) in response.headers() {
            println!("  {}: {}", name.to_string().cyan(), value.to_str().unwrap_or("").white());
        }
        println!();
    }
    
    if cli.show_body {
        let content_type = response.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let body = response.text().await?;

        if content_type.contains("application/json") || content_type.contains("+json") {
            // Pretty-print and colorize JSON body
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json_value) => {
                    let pretty = serde_json::to_string_pretty(&json_value).unwrap_or(body.clone());
                    print_basic_colorized_json(&pretty);
                }
                Err(_) => {
                    // If not valid JSON, just print as plain text
                    println!("{}", body);
                }
            }
        } else {
            // Not JSON, just print as plain text
            println!("{}", body);
        }
    }

    Ok(())
} 