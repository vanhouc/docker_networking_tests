use axum::{extract::Query, response::Html, routing::get, Router};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::process::Command;
use tracing_subscriber;

async fn hello_world() -> Html<String> {
    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());
    let target_server =
        std::env::var("TARGET_SERVER").unwrap_or_else(|_| "other-server:8080".to_string());

    Html(format!(
        "<h1>Hello from {}!</h1><p>This is {} running on port 8080 for Docker DNS resolution testing.</p><p><a href=\"/test-dns?target={}\">Test DNS to Target Server</a></p><p><a href=\"/test-google\">Test External Connectivity (Google)</a></p><p><a href=\"/test-ping\">Test Ping to HWDocker001.RTLSBench.net</a></p>",
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

async fn test_ping() -> Html<String> {
    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "Unknown Server".to_string());
    let target_host = "HWDocker001.RTLSBench.net";

    // Platform-specific ping command
    let mut cmd = Command::new("ping");
    if cfg!(target_os = "windows") {
        // Windows: ping -n 4 hostname
        cmd.arg("-n").arg("4");
    } else {
        // Unix/Linux: ping -c 4 hostname
        cmd.arg("-c").arg("4");
    }
    cmd.arg(target_host);

    // Execute ping command (4 pings)
    match cmd.output().await
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() {
                // Parse ping statistics
                let ping_stats = parse_ping_output(&stdout);
                Html(format!(
                    "<h1>Ping Test Result from {}</h1><p>✅ Successfully pinged {}</p><h3>Ping Statistics:</h3><pre>{}</pre><h3>Full Output:</h3><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                    server_name, target_host, ping_stats, stdout
                ))
            } else {
                Html(format!(
                    "<h1>Ping Test Result from {}</h1><p>❌ Ping to {} failed</p><h3>Error Output:</h3><pre>{}</pre><h3>Standard Output:</h3><pre>{}</pre><p><a href=\"/\">Back to home</a></p>", 
                    server_name, target_host, stderr, stdout
                ))
            }
        },
        Err(e) => Html(format!(
            "<h1>Ping Test Result from {}</h1><p>❌ Failed to execute ping command: {}</p><p><a href=\"/\">Back to home</a></p>", 
            server_name, e
        )),
    }
}

fn parse_ping_output(output: &str) -> String {
    let mut stats = String::new();
    
    // Detect platform based on output format
    let is_windows = output.contains("Reply from") || output.contains("Lost =");
    
    if is_windows {
        // Windows ping output parsing
        parse_windows_ping(output, &mut stats);
    } else {
        // Unix/Linux ping output parsing
        parse_unix_ping(output, &mut stats);
    }
    
    if stats.is_empty() {
        stats.push_str("Unable to parse ping statistics from output");
    }
    
    stats
}

fn parse_windows_ping(output: &str, stats: &mut String) {
    // Count successful pings by looking for "Reply from" lines
    let reply_count = output.matches("Reply from").count();
    
    // Look for packet loss information
    if let Some(line) = output.lines().find(|line| line.contains("Lost =")) {
        // Parse Windows ping statistics line like "Packets: Sent = 4, Received = 4, Lost = 0 (0% loss),"
        if let Some(sent_pos) = line.find("Sent = ") {
            if let Some(received_pos) = line.find("Received = ") {
                if let Some(lost_pos) = line.find("Lost = ") {
                    let sent_str = &line[sent_pos + 7..].split(',').next().unwrap_or("0");
                    let received_str = &line[received_pos + 11..].split(',').next().unwrap_or("0");
                    let lost_str = &line[lost_pos + 7..].split(' ').next().unwrap_or("0");
                    
                    stats.push_str(&format!("Packets sent: {}\n", sent_str));
                    stats.push_str(&format!("Packets received: {}\n", received_str));
                    stats.push_str(&format!("Packets lost: {}\n", lost_str));
                    
                    // Calculate loss percentage
                    if let (Ok(sent), Ok(received)) = (sent_str.parse::<i32>(), received_str.parse::<i32>()) {
                        let loss_percent = if sent > 0 { ((sent - received) as f64 / sent as f64) * 100.0 } else { 0.0 };
                        stats.push_str(&format!("Packet loss: {:.1}%\n", loss_percent));
                    }
                }
            }
        }
    } else {
        // Fallback if we can't parse the statistics line
        stats.push_str(&format!("Packets sent: 4\n"));
        stats.push_str(&format!("Successful replies: {}\n", reply_count));
        stats.push_str(&format!("Packet loss: {:.1}%\n", ((4 - reply_count) as f64 / 4.0) * 100.0));
    }
    
    // Look for timing information in reply lines (Windows format)
    let mut times = Vec::new();
    for line in output.lines() {
        if line.contains("time=") || line.contains("time<") {
            // Extract time from lines like "Reply from 192.168.1.1: bytes=32 time=1ms TTL=64"
            if let Some(time_pos) = line.find("time") {
                let time_part = &line[time_pos..];
                if let Some(ms_pos) = time_part.find("ms") {
                    let time_str = &time_part[5..ms_pos]; // Skip "time=" or "time<"
                    if let Ok(time_val) = time_str.parse::<f64>() {
                        times.push(time_val);
                    }
                }
            }
        }
    }
    
    if !times.is_empty() {
        let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        
        stats.push_str(&format!("RTT min: {:.1}ms\n", min_time));
        stats.push_str(&format!("RTT avg: {:.1}ms\n", avg_time));
        stats.push_str(&format!("RTT max: {:.1}ms\n", max_time));
    }
}

fn parse_unix_ping(output: &str, stats: &mut String) {
    // Look for statistics summary line like "4 packets transmitted, 4 received, 0% packet loss, time 3003ms"
    for line in output.lines() {
        if line.contains("packets transmitted") && line.contains("received") {
            let parts: Vec<&str> = line.split(", ").collect();
            
            for part in parts {
                if part.contains("transmitted") {
                    let transmitted = part.split_whitespace().next().unwrap_or("0");
                    stats.push_str(&format!("Packets transmitted: {}\n", transmitted));
                }
                if part.contains("received") {
                    let received = part.split_whitespace().next().unwrap_or("0");
                    stats.push_str(&format!("Packets received: {}\n", received));
                }
                if part.contains("packet loss") || part.contains("loss") {
                    if let Some(percent_pos) = part.find('%') {
                        let loss_str = &part[..percent_pos];
                        if let Some(space_pos) = loss_str.rfind(' ') {
                            let loss = &loss_str[space_pos + 1..];
                            stats.push_str(&format!("Packet loss: {}%\n", loss));
                        }
                    }
                }
                if part.contains("time") {
                    if let Some(time_start) = part.find("time ") {
                        let time_part = &part[time_start + 5..];
                        if let Some(ms_pos) = time_part.find("ms") {
                            let time_str = &time_part[..ms_pos];
                            stats.push_str(&format!("Total time: {}ms\n", time_str));
                        }
                    }
                }
            }
            break;
        }
    }
    
    // Look for RTT statistics line like "rtt min/avg/max/mdev = 0.045/0.045/0.045/0.000 ms"
    for line in output.lines() {
        if line.contains("rtt min/avg/max") {
            if let Some(equals_pos) = line.find(" = ") {
                let values_part = &line[equals_pos + 3..];
                if let Some(ms_pos) = values_part.find(" ms") {
                    let values_str = &values_part[..ms_pos];
                    let values: Vec<&str> = values_str.split('/').collect();
                    
                    if values.len() >= 4 {
                        stats.push_str(&format!("RTT min: {}ms\n", values[0]));
                        stats.push_str(&format!("RTT avg: {}ms\n", values[1]));
                        stats.push_str(&format!("RTT max: {}ms\n", values[2]));
                        stats.push_str(&format!("RTT mdev: {}ms\n", values[3]));
                    }
                }
            }
            break;
        }
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
        .route("/test-google", get(test_google))
        .route("/test-ping", get(test_ping));

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
    println!("   - GET /test-ping");

    println!("🚀 Starting server...");
    if let Err(e) = axum::serve(listener, app).await {
        println!("❌ Server error: {}", e);
        panic!("Server failed: {}", e);
    }
}
