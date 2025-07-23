use axum::{extract::Query, response::Html, routing::get, Router};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing_subscriber;

async fn hello_world() -> Html<&'static str> {
    Html("<h1>Hello from Server 1!</h1><p>This is server1 running on port 8080 for Docker DNS resolution testing.</p><p><a href=\"/test-dns?target=server2:8080\">Test DNS to Server2</a></p><p><a href=\"/test-google\">Test External Connectivity (Google)</a></p>")
}

async fn test_dns(Query(params): Query<HashMap<String, String>>) -> Html<String> {
    let default_target = "server2:8080".to_string();
    let target = params.get("target").unwrap_or(&default_target);

    match reqwest::get(&format!("http://{}", target)).await {
        Ok(response) => {
            match response.text().await {
                Ok(body) => Html(format!(
                    "<h1>DNS Test Result from Server 1</h1><p>✅ Successfully connected to {}</p><p>Response:</p><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                    target, body
                )),
                Err(e) => Html(format!(
                    "<h1>DNS Test Result from Server 1</h1><p>❌ Error reading response from {}: {}</p><p><a href=\"/\">Back to home</a></p>", 
                    target, e
                )),
            }
        },
        Err(e) => Html(format!(
            "<h1>DNS Test Result from Server 1</h1><p>❌ Failed to connect to {}: {}</p><p><a href=\"/\">Back to home</a></p>", 
            target, e
        )),
    }
}

async fn test_google() -> Html<String> {
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
                        "<h1>External Connectivity Test from Server 1</h1><p>✅ Successfully connected to Google.com</p><p>Status: {}</p><p>Response (truncated):</p><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                        status, truncated_body
                    ))
                },
                Err(e) => Html(format!(
                    "<h1>External Connectivity Test from Server 1</h1><p>❌ Error reading response from Google.com: {}</p><p><a href=\"/\">Back to home</a></p>", 
                    e
                )),
            }
        },
        Err(e) => Html(format!(
            "<h1>External Connectivity Test from Server 1</h1><p>❌ Failed to connect to Google.com: {}</p><p><a href=\"/\">Back to home</a></p>", 
            e
        )),
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/test-dns", get(test_dns))
        .route("/test-google", get(test_google));

    // Run it with hyper on localhost:8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Server1 listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
