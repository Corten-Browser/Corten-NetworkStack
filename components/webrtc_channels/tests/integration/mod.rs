//! Integration tests for webrtc_channels component

#[cfg(test)]
mod tests {
    use webrtc_channels::{
        DataChannelMessage, DataChannelOptions, DataChannelState, RtcDataChannel,
    };

    #[tokio::test]
    async fn test_data_channel_full_lifecycle() {
        /// Given a new data channel
        /// When going through full lifecycle (create, send, recv, close)
        /// Then all operations should work correctly

        // Create channel with specific options
        let options = DataChannelOptions {
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: Some(3),
            protocol: "integration-test".to_string(),
            negotiated: false,
            id: None,
        };

        let channel = RtcDataChannel::new("test-channel".to_string(), options);

        // Verify channel label
        assert_eq!(channel.label().await, "test-channel");

        // Set channel to open state (simulating WebRTC handshake completion)
        channel.set_state_for_testing(DataChannelState::Open).await;

        // Send text message
        assert!(channel.send_text("Hello from integration test!").await.is_ok());

        // Send binary message
        let binary_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(channel.send(&binary_data).await.is_ok());

        // Add test messages to simulate receiving
        channel
            .add_test_message(DataChannelMessage::Text(
                "Response message".to_string(),
            ))
            .await;
        channel
            .add_test_message(DataChannelMessage::Binary(vec![1, 2, 3, 4]))
            .await;

        // Receive and verify text message
        let msg1 = channel.recv().await;
        assert!(msg1.is_some());
        match msg1.unwrap() {
            Ok(DataChannelMessage::Text(text)) => {
                assert_eq!(text, "Response message");
            }
            _ => panic!("Expected text message"),
        }

        // Receive and verify binary message
        let msg2 = channel.recv().await;
        assert!(msg2.is_some());
        match msg2.unwrap() {
            Ok(DataChannelMessage::Binary(data)) => {
                assert_eq!(data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Expected binary message"),
        }

        // Close the channel
        assert!(channel.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_data_channel_error_handling() {
        /// Given various error scenarios
        /// When operations are performed
        /// Then appropriate errors should be returned

        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("error-test".to_string(), options);

        // Try to send when channel is not open (still connecting)
        let result = channel.send_text("This should fail").await;
        assert!(result.is_err());

        // Open the channel
        channel.set_state_for_testing(DataChannelState::Open).await;

        // Verify send works now
        assert!(channel.send_text("This should succeed").await.is_ok());

        // Close the channel
        assert!(channel.close().await.is_ok());

        // Try to close again (should error)
        let result = channel.close().await;
        assert!(result.is_err());

        // Try to send when closed (should error)
        let result = channel.send_text("This should also fail").await;
        assert!(result.is_err());

        // Try to receive when closed (should return None)
        let result = channel.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_data_channel_ordering_configuration() {
        /// Given different ordering configurations
        /// When creating data channels
        /// Then options should be preserved correctly

        // Ordered channel
        let ordered_opts = DataChannelOptions {
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: Some(10),
            protocol: "ordered-test".to_string(),
            negotiated: false,
            id: None,
        };

        let ordered_channel = RtcDataChannel::new("ordered".to_string(), ordered_opts.clone());
        let stored_opts = ordered_channel.options().await;
        assert!(stored_opts.ordered);
        assert_eq!(stored_opts.max_retransmits, Some(10));

        // Unordered channel with packet lifetime
        let unordered_opts = DataChannelOptions {
            ordered: false,
            max_packet_life_time: Some(5000),
            max_retransmits: None,
            protocol: "unordered-test".to_string(),
            negotiated: false,
            id: None,
        };

        let unordered_channel = RtcDataChannel::new("unordered".to_string(), unordered_opts.clone());
        let stored_opts = unordered_channel.options().await;
        assert!(!stored_opts.ordered);
        assert_eq!(stored_opts.max_packet_life_time, Some(5000));
        assert_eq!(stored_opts.max_retransmits, None);
    }

    #[tokio::test]
    async fn test_multiple_messages_in_sequence() {
        /// Given an open data channel
        /// When sending and receiving multiple messages in sequence
        /// Then all messages should be handled correctly

        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("multi-msg-test".to_string(), options);

        channel.set_state_for_testing(DataChannelState::Open).await;

        // Add multiple messages to the queue
        for i in 0..5 {
            channel
                .add_test_message(DataChannelMessage::Text(format!("Message {}", i)))
                .await;
        }

        // Receive all messages in order
        for i in 0..5 {
            let msg = channel.recv().await;
            assert!(msg.is_some());
            match msg.unwrap() {
                Ok(DataChannelMessage::Text(text)) => {
                    assert_eq!(text, format!("Message {}", i));
                }
                _ => panic!("Expected text message"),
            }
        }

        // No more messages should be available
        let msg = channel.recv().await;
        assert!(msg.is_none());
    }
}
