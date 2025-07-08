use rusttpx::{Client, Request, Response, Url};
use std::error::Error;
use http::Method;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new client
    let client = Client::new();
    
    println!("=== Basic GET Request ===");
    
    // Make a simple GET request
    let response = client.get("https://httpbin.org/get".parse::<Url>()?).send().await?;
    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());
    
    let body = response.text().await?;
    println!("Body length: {} bytes", body.len());
    println!("First 200 chars: {}", &body[..body.len().min(200)]);
    
    println!("\n=== POST Request with JSON ===");
    
    // Make a POST request with JSON data
    let json_data = serde_json::json!({
        "name": "rusttpx",
        "version": "0.1.0",
        "language": "rust"
    });
    
    let response = client
        .post("https://httpbin.org/post".parse::<Url>()?)
        .json(&json_data)?
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    let response_body = response.text().await?;
    println!("Response: {}", &response_body[..response_body.len().min(300)]);
    
    println!("\n=== Custom Request with Headers ===");
    
    // Create a custom request with headers
    let request = Request::new(
        Method::GET,
        "https://httpbin.org/headers".parse()?
    )
    .header("User-Agent", "rusttpx/0.1.0")?
    .header("Accept", "application/json")?;
    
    let response = client.send(request).await?;
    println!("Status: {}", response.status());
    let body = response.text().await?;
    println!("Response: {}", &body[..body.len().min(200)]);
    
    println!("\n=== Error Handling Example ===");
    
    // Demonstrate error handling
    match client.get("https://httpbin.org/status/404".parse::<Url>()?).send().await {
        Ok(response) => {
            println!("Unexpected success: {}", response.status());
        }
        Err(e) => {
            println!("Expected error for 404: {}", e);
        }
    }
    
    println!("\n=== All examples completed successfully! ===");
    
    Ok(())
} 