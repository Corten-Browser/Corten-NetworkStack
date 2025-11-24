//! Unit tests for websocket_protocol

use websocket_protocol::{
    ReconnectConfig, ReconnectionEvent, ReconnectingWebSocket,
    WebSocketClient, WebSocketConnection, WebSocketMessage, WebSocketState,
};
use std::time::Duration;
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

// ==================== ReconnectConfig Unit Tests ====================

/// Test ReconnectConfig default values
#[test]
fn test_reconnect_config_defaults() {
    let config = ReconnectConfig::default();
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.max_attempts, None);
    assert_eq!(config.multiplier, 2.0);
}

/// Test ReconnectConfig builder pattern
#[test]
fn test_reconnect_config_builder() {
    let config = ReconnectConfig::new()
        .with_initial_delay_ms(500)
        .with_max_delay_ms(60000)
        .with_max_attempts(10)
        .with_multiplier(1.5);

    assert_eq!(config.initial_delay_ms, 500);
    assert_eq!(config.max_delay_ms, 60000);
    assert_eq!(config.max_attempts, Some(10));
    assert_eq!(config.multiplier, 1.5);
}

/// Test exponential backoff calculation with standard 2x multiplier
#[test]
fn test_exponential_backoff_standard() {
    let config = ReconnectConfig::new()
        .with_initial_delay_ms(1000)
        .with_multiplier(2.0)
        .with_max_delay_ms(30000);

    // Sequence: 1s, 2s, 4s, 8s, 16s, 30s (capped)
    assert_eq!(config.calculate_delay(0), Duration::from_millis(1000));
    assert_eq!(config.calculate_delay(1), Duration::from_millis(2000));
    assert_eq!(config.calculate_delay(2), Duration::from_millis(4000));
    assert_eq!(config.calculate_delay(3), Duration::from_millis(8000));
    assert_eq!(config.calculate_delay(4), Duration::from_millis(16000));
    assert_eq!(config.calculate_delay(5), Duration::from_millis(30000)); // capped at max
}

/// Test that delay is capped at max_delay_ms
#[test]
fn test_delay_capped_at_maximum() {
    let config = ReconnectConfig::new()
        .with_initial_delay_ms(10000)
        .with_multiplier(2.0)
        .with_max_delay_ms(30000);

    // 10s * 2^2 = 40s, but should be capped at 30s
    assert_eq!(config.calculate_delay(2), Duration::from_millis(30000));

    // Further attempts should also be capped
    assert_eq!(config.calculate_delay(10), Duration::from_millis(30000));
}

// ==================== ReconnectionEvent Unit Tests ====================

/// Test ReconnectionEvent variants
#[test]
fn test_reconnection_event_variants() {
    let connecting = ReconnectionEvent::Connecting;
    let connected = ReconnectionEvent::Connected;
    let reconnecting = ReconnectionEvent::Reconnecting { attempt: 2, delay_ms: 4000 };
    let failed = ReconnectionEvent::ReconnectionFailed { total_attempts: 5 };
    let disconnected = ReconnectionEvent::Disconnected;

    assert_eq!(connecting, ReconnectionEvent::Connecting);
    assert_eq!(connected, ReconnectionEvent::Connected);
    assert_eq!(disconnected, ReconnectionEvent::Disconnected);

    match reconnecting {
        ReconnectionEvent::Reconnecting { attempt, delay_ms } => {
            assert_eq!(attempt, 2);
            assert_eq!(delay_ms, 4000);
        }
        _ => panic!("Expected Reconnecting variant"),
    }

    match failed {
        ReconnectionEvent::ReconnectionFailed { total_attempts } => {
            assert_eq!(total_attempts, 5);
        }
        _ => panic!("Expected ReconnectionFailed variant"),
    }
}

/// Test ReconnectionEvent equality
#[test]
fn test_reconnection_event_equality() {
    let event1 = ReconnectionEvent::Reconnecting { attempt: 1, delay_ms: 2000 };
    let event2 = ReconnectionEvent::Reconnecting { attempt: 1, delay_ms: 2000 };
    let event3 = ReconnectionEvent::Reconnecting { attempt: 2, delay_ms: 2000 };

    assert_eq!(event1, event2);
    assert_ne!(event1, event3);
}

// ==================== ReconnectingWebSocket Unit Tests ====================

/// Test ReconnectingWebSocket creation
#[test]
fn test_reconnecting_websocket_creation() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::new()
        .with_initial_delay_ms(500)
        .with_max_attempts(5);

    let ws = ReconnectingWebSocket::new(url.clone(), vec!["protocol1".to_string()], config);

    assert_eq!(ws.url(), &url);
    assert_eq!(ws.config().initial_delay_ms, 500);
    assert_eq!(ws.config().max_attempts, Some(5));
}

/// Test initial event state is Disconnected
#[test]
fn test_reconnecting_websocket_initial_event_state() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::default();
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    assert_eq!(ws.current_event(), ReconnectionEvent::Disconnected);
}

/// Test event subscription
#[test]
fn test_reconnecting_websocket_event_subscription() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::default();
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    let receiver = ws.subscribe_events();
    assert_eq!(*receiver.borrow(), ReconnectionEvent::Disconnected);
}

/// Test is_connected returns false when not connected
#[tokio::test]
async fn test_is_connected_false_when_disconnected() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::default();
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    assert!(!ws.is_connected().await);
}

/// Test send returns error when not connected
#[tokio::test]
async fn test_send_error_when_not_connected() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::default();
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    let result = ws.send(WebSocketMessage::Text("test".to_string())).await;
    assert!(result.is_err());
}

/// Test close succeeds even when not connected
#[tokio::test]
async fn test_close_succeeds_when_not_connected() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::default();
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    let result = ws.close(1000, "Test close".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(ws.current_event(), ReconnectionEvent::Disconnected);
}

/// Test reconnect fails after intentional close
#[tokio::test]
async fn test_reconnect_fails_after_intentional_close() {
    let url = Url::parse("ws://localhost:8080/ws").unwrap();
    let config = ReconnectConfig::new().with_max_attempts(1);
    let ws = ReconnectingWebSocket::new(url, vec![], config);

    // Close the WebSocket intentionally
    ws.close(1000, "Test".to_string()).await.unwrap();

    // Reconnect should fail
    let result = ws.reconnect().await;
    assert!(result.is_err());
}
