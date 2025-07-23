use axum::{extract::Query, response::Html, routing::get, Router};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber;

async fn hello_world() -> Html<String> {
    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());
    let target_server =
        std::env::var("TARGET_SERVER").unwrap_or_else(|_| "other-server:8080".to_string());

    Html(format!(
        "<h1>Hello from {}!</h1><p>This is {} running on port 8080 for Docker DNS resolution testing.</p><p><a href=\"/test-dns?target={}\">Test DNS to Target Server</a></p><p><a href=\"/test-google\">Test External Connectivity (Google)</a></p>",
        server_name, server_name, target_server
    ))
}

async fn test_dns(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let default_target =
        std::env::var("TARGET_SERVER").unwrap_or_else(|_| "other-server:8080".to_string());
    let target = params.get("target").unwrap_or(&default_target);
    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());

    match reqwest::get(&format!("http://{}", target)).await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => Html(format!(
                    "<h1>DNS Test Result from {}</h1><p>✅ Successfully connected to {}</p><p>Response:</p><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                    server_name, target, body
                )),
                Err(e) => Html(format!(
                    "<h1>DNS Test Result from {}</h1><p>❌ Error reading response from {}: {}</p><p><a href=\"/\">Back to home</a></p>", 
                    server_name, target, e
                )),
            }
        },
        Err(e) => Html(format!(
            "<h1>DNS Test Result from {}</h1><p>❌ Failed to connect to {}: {}</p><p><a href=\"/\">Back to home</a></p>", 
            server_name, target, e
        )),
    }
}

async fn test_google() -> Html<String> {
    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());

    match reqwest::get("https://www.google.com").await {
        Ok(response) => {
            let status = response.status();
            match response.text().await {
                Ok(body) => {
                    let truncated_body = if body.len() > 500 {
                        format!("{}...", &body[..500])
                    } else {
                        body
                    };
                    Html(format!(
                        "<h1>External Connectivity Test from {}</h1><p>✅ Successfully connected to Google.com</p><p>Status: {}</p><p>Response (truncated):</p><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                        server_name, status, truncated_body
                    ))
                },
                Err(e) => Html(format!(
                    "<h1>External Connectivity Test from {}</h1><p>❌ Error reading response from Google.com: {}</p><p><a href=\"/\">Back to home</a></p>", 
                    server_name, e
                )),
            }
        },
        Err(e) => Html(format!(
            "<h1>External Connectivity Test from {}</h1><p>❌ Failed to connect to Google.com: {}</p><p><a href=\"/\">Back to home</a></p>", 
            server_name, e
        )),
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting docker networking test server...");

    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());
    let target_server =
        std::env::var("TARGET_SERVER").unwrap_or_else(|_| "other-server:8080".to_string());

    println!("📝 Configuration:");
    println!("   SERVER_NAME: {}", server_name);
    println!("   TARGET_SERVER: {}", target_server);

    // Build our application with routes
    println!("🔧 Building application routes...");
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/test-dns", get(test_dns))
        .route("/test-google", get(test_google));

    // Run it with hyper on localhost:8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    println!("🌐 Attempting to bind to address: {}", addr);
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("✅ Successfully bound to {}", addr);
            listener
        }
        Err(e) => {
            println!("❌ Failed to bind to {}: {}", addr, e);
            panic!("Failed to bind to address: {}", e);
        }
    };

    println!("🎉 {} listening on {}", server_name, addr);
    println!("🔗 Available endpoints:");
    println!("   - GET /");
    println!("   - GET /test-dns");
    println!("   - GET /test-google");

    println!("🚀 Starting server...");
    if let Err(e) = axum::serve(listener, app).await {
        println!("❌ Server error: {}", e);
        panic!("Server failed: {}", e);
    }
}
