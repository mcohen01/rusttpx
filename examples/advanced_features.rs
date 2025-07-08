use rusttpx::{Client, Request, Response, middleware::Middleware, Url};
use std::error::Error;
use std::time::Duration;
use futures_util::StreamExt;
use http::Request as HttpRequest;

// Custom middleware that logs requests
struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn process_request(&self, request: HttpRequest<()>) -> Result<HttpRequest<()>, rusttpx::Error> {
        println!("🔄 Making request: {} {}", request.method(), request.uri());
        Ok(request)
    }

    async fn process_response(&self, response: http::Response<()>) -> Result<http::Response<()>, rusttpx::Error> {
        println!("✅ Received response: {}", response.status());
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Advanced rusttpx Features Demo ===\n");
    
    // Create a client with custom configuration
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build();
    
    println!("=== Cookie Handling ===");
    
    // Test cookie handling
    let response = client.get("https://httpbin.org/cookies/set?name=rusttpx&value=awesome".parse::<Url>()?).send().await?;
    println!("Cookies set: {:?}", response.headers().get("set-cookie"));
    
    // Make another request to see if cookies are sent
    let response = client.get("https://httpbin.org/cookies".parse::<Url>()?).send().await?;
    let body = response.text().await?;
    println!("Cookies received: {}", &body[..body.len().min(200)]);
    
    println!("\n=== Timeout Handling ===");
    
    // Test timeout (this should timeout quickly)
    let timeout_client = Client::builder()
        .timeout(Duration::from_millis(100))
        .build();
    
    match timeout_client.get("https://httpbin.org/delay/1".parse::<Url>()?).send().await {
        Ok(_) => println!("Unexpected: Request completed"),
        Err(e) => println!("Expected timeout error: {}", e),
    }
    
    println!("\n=== Streaming Response ===");
    
    // Test streaming response
    let response = client.get("https://httpbin.org/stream/3".parse::<Url>()?).send().await?;
    let mut stream = response.bytes_stream();
    
    let mut total_bytes = 0;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                total_bytes += bytes.len();
                println!("Received chunk: {} bytes", bytes.len());
            }
            Err(e) => {
                println!("Stream error: {}", e);
                break;
            }
        }
    }
    println!("Total bytes received: {}", total_bytes);
    
    println!("\n=== Multipart Upload ===");
    
    // Test multipart upload
    let form_data = vec![
        ("field1", "value1"),
        ("field2", "value2"),
    ];
    
    let response = client
        .post("https://httpbin.org/post".parse::<Url>()?)
        .form(&form_data)?
        .send()
        .await?;
    
    println!("Multipart upload status: {}", response.status());
    let body = response.text().await?;
    println!("Response: {}", &body[..body.len().min(200)]);
    
    println!("\n=== Authentication Example ===");
    
    // Test basic authentication
    let response = client
        .get("https://httpbin.org/basic-auth/user/passwd".parse::<Url>()?)
        .basic_auth("user", Some("passwd"))
        .send()
        .await?;
    
    println!("Auth status: {}", response.status());
    
    println!("\n=== All advanced examples completed! ===");
    
    Ok(())
} 