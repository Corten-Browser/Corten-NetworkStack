//! Unit tests for websocket_protocol

use websocket_protocol::{WebSocketClient, WebSocketConnection, WebSocketMessage, WebSocketState};
use url::Url;

/// Test creating Text message variant
#[test]
fn test_websocket_message_text() {
    let msg = WebSocketMessage::Text("Hello".to_string());
    match msg {
        WebSocketMessage::Text(text) => assert_eq!(text, "Hello"),
        _ => panic!("Expected Text variant"),
    }
}

/// Test creating Binary message variant
#[test]
fn test_websocket_message_binary() {
    let data = vec![1, 2, 3, 4];
    let msg = WebSocketMessage::Binary(data.clone());
    match msg {
        WebSocketMessage::Binary(d) => assert_eq!(d, data),
        _ => panic!("Expected Binary variant"),
    }
}

/// Test creating Ping message variant
#[test]
fn test_websocket_message_ping() {
    let payload = vec![1, 2, 3];
    let msg = WebSocketMessage::Ping(payload.clone());
    match msg {
        WebSocketMessage::Ping(p) => assert_eq!(p, payload),
        _ => panic!("Expected Ping variant"),
    }
}

/// Test creating Pong message variant
#[test]
fn test_websocket_message_pong() {
    let payload = vec![4, 5, 6];
    let msg = WebSocketMessage::Pong(payload.clone());
    match msg {
        WebSocketMessage::Pong(p) => assert_eq!(p, payload),
        _ => panic!("Expected Pong variant"),
    }
}

/// Test creating Close message variant
#[test]
fn test_websocket_message_close() {
    let msg = WebSocketMessage::Close(None);
    match msg {
        WebSocketMessage::Close(_) => (),
        _ => panic!("Expected Close variant"),
    }
}

/// Test WebSocketMessage is cloneable
#[test]
fn test_websocket_message_clone() {
    let msg = WebSocketMessage::Text("test".to_string());
    let cloned = msg.clone();
    match cloned {
        WebSocketMessage::Text(text) => assert_eq!(text, "test"),
        _ => panic!("Expected Text variant"),
    }
}

/// Test WebSocketMessage is debuggable
#[test]
fn test_websocket_message_debug() {
    let msg = WebSocketMessage::Text("test".to_string());
    let debug_str = format!("{:?}", msg);
    assert!(debug_str.contains("Text"));
}

/// Test WebSocketState enum values
#[test]
fn test_websocket_state_connecting() {
    let state = WebSocketState::Connecting;
    assert_eq!(state, WebSocketState::Connecting);
}

#[test]
fn test_websocket_state_open() {
    let state = WebSocketState::Open;
    assert_eq!(state, WebSocketState::Open);
}

#[test]
fn test_websocket_state_closing() {
    let state = WebSocketState::Closing;
    assert_eq!(state, WebSocketState::Closing);
}

#[test]
fn test_websocket_state_closed() {
    let state = WebSocketState::Closed;
    assert_eq!(state, WebSocketState::Closed);
}

/// Test WebSocketState is Copy
#[test]
fn test_websocket_state_copy() {
    let state = WebSocketState::Open;
    let copied = state;
    assert_eq!(state, copied);
}

/// Test WebSocketState is debuggable
#[test]
fn test_websocket_state_debug() {
    let state = WebSocketState::Open;
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Open"));
}

/// Test WebSocketConnection creation
#[test]
fn test_websocket_connection_new() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let (conn, _, _) = WebSocketConnection::new(
        url.clone(),
        Some("chat".to_string()),
        vec!["permessage-deflate".to_string()],
    );
    assert_eq!(conn.url, url);
    assert_eq!(conn.protocol, Some("chat".to_string()));
    assert_eq!(conn.extensions, vec!["permessage-deflate".to_string()]);
    assert_eq!(conn.state(), WebSocketState::Connecting);
}

/// Test WebSocketConnection state method
#[test]
fn test_websocket_connection_state() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let (conn, _, _) = WebSocketConnection::new(url, None, vec![]);
    assert_eq!(conn.state(), WebSocketState::Connecting);
}

/// Test WebSocketClient creation
#[test]
fn test_websocket_client_new() {
    let client = WebSocketClient::new();
    // Just verify it can be created
    drop(client);
}

/// Test WebSocketClient default
#[test]
fn test_websocket_client_default() {
    let client = WebSocketClient::default();
    drop(client);
}

/// Test message send/receive through channels
#[tokio::test]
async fn test_websocket_connection_send_recv() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let (mut conn, tx_in, mut rx_out) = WebSocketConnection::new(url, None, vec![]);

    // Send a message through the connection
    let msg = WebSocketMessage::Text("Hello".to_string());
    conn.send(msg.clone()).await.expect("Failed to send");

    // Receive it from the output channel
    let received = rx_out.recv().await.expect("Failed to receive");
    match received {
        WebSocketMessage::Text(text) => assert_eq!(text, "Hello"),
        _ => panic!("Expected Text message"),
    }

    // Send a message through the input channel
    let msg2 = WebSocketMessage::Binary(vec![1, 2, 3]);
    tx_in.send(msg2).await.expect("Failed to send to input");

    // Receive it from the connection
    let received2 = conn.recv().await.expect("Should have message");
    match received2 {
        Ok(WebSocketMessage::Binary(data)) => assert_eq!(data, vec![1, 2, 3]),
        _ => panic!("Expected Binary message"),
    }
}

/// Test connection close
#[tokio::test]
async fn test_websocket_connection_close() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let (mut conn, _, mut rx_out) = WebSocketConnection::new(url, None, vec![]);

    // Close the connection
    conn.close(1000, "Normal close".to_string())
        .await
        .expect("Failed to close");

    // Verify state changed to Closed
    assert_eq!(conn.state(), WebSocketState::Closed);

    // Verify close message was sent
    let received = rx_out.recv().await.expect("Should have close message");
    match received {
        WebSocketMessage::Close(Some(frame)) => {
            assert_eq!(frame.code, 1000);
            assert_eq!(frame.reason, "Normal close");
        }
        _ => panic!("Expected Close message"),
    }
}
