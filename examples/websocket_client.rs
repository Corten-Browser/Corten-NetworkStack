//! WebSocket Client Example
//!
//! This example demonstrates how to establish a WebSocket connection
//! and send/receive messages.
//!
//! Run with:
//! ```sh
//! cargo run --example websocket_client
//! ```

use network_stack::NetworkStack;
use network_stack_impl::NetworkStackImpl;
use websocket_protocol::{WebSocketMessage, WebSocketState};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WebSocket Client Example ===\n");

    // Create the network stack
    let stack = NetworkStackImpl::new();

    // Connect to WebSocket echo server
    let url = Url::parse("wss://echo.websocket.org/")?;
    let protocols = vec!["chat".to_string()];

    println!("Connecting to: {}", url);

    let mut connection = stack.connect_websocket(url.clone(), protocols).await?;

    println!("✓ Connected!");
    println!("  URL: {}", connection.url);
    println!("  Protocol: {:?}", connection.protocol);
    println!("  Extensions: {:?}", connection.extensions);
    println!("  State: {:?}", connection.state());

    // Send text messages
    println!("\n--- Sending Messages ---");

    let messages = vec![
        "Hello, WebSocket!",
        "This is message 2",
        "Testing echo functionality",
    ];

    for (i, msg) in messages.iter().enumerate() {
        println!("Sending message {}: {}", i + 1, msg);
        let message = WebSocketMessage::Text(msg.to_string());
        connection.send(message).await?;

        // Wait for echo response
        if let Some(received) = connection.receive().await? {
            match received {
                WebSocketMessage::Text(text) => {
                    println!("  ← Received echo: {}", text);
                }
                WebSocketMessage::Binary(data) => {
                    println!("  ← Received binary: {} bytes", data.len());
                }
                WebSocketMessage::Ping(data) => {
                    println!("  ← Received ping: {} bytes", data.len());
                    // Send pong response
                    let pong = WebSocketMessage::Pong(data);
                    connection.send(pong).await?;
                }
                WebSocketMessage::Pong(_) => {
                    println!("  ← Received pong");
                }
                WebSocketMessage::Close(frame) => {
                    println!("  ← Connection closed by server: {:?}", frame);
                    break;
                }
            }
        }
    }

    // Send binary message
    println!("\n--- Sending Binary Data ---");
    let binary_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    println!("Sending {} bytes of binary data", binary_data.len());
    let binary_message = WebSocketMessage::Binary(binary_data);
    connection.send(binary_message).await?;

    if let Some(received) = connection.receive().await? {
        match received {
            WebSocketMessage::Binary(data) => {
                println!("  ← Received binary echo: {:?}", data);
            }
            _ => println!("  ← Received unexpected message type"),
        }
    }

    // Send ping
    println!("\n--- Testing Ping/Pong ---");
    let ping = WebSocketMessage::Ping(vec![]);
    connection.send(ping).await?;
    println!("Sent ping");

    if let Some(received) = connection.receive().await? {
        match received {
            WebSocketMessage::Pong(_) => {
                println!("  ← Received pong");
            }
            _ => println!("  ← Received unexpected message type"),
        }
    }

    // Close connection gracefully
    println!("\n--- Closing Connection ---");
    connection.close(1000, "Normal closure".to_string()).await?;
    println!("✓ Connection closed gracefully");

    Ok(())
}
