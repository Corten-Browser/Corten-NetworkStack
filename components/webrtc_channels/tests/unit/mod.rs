//! Unit tests for webrtc_channels component

#[cfg(test)]
mod tests {
    use webrtc_channels::{
        DataChannelMessage, DataChannelOptions, DataChannelState, RtcDataChannel,
    };

    // ===== DataChannelOptions Tests =====

    #[test]
    fn test_data_channel_options_creation() {
        /// Given default options
        /// When creating DataChannelOptions
        /// Then all fields should be initialized correctly
        let options = DataChannelOptions {
            ordered: true,
            max_packet_life_time: Some(3000),
            max_retransmits: None,
            protocol: "my-protocol".to_string(),
            negotiated: false,
            id: Some(42),
        };

        assert!(options.ordered);
        assert_eq!(options.max_packet_life_time, Some(3000));
        assert_eq!(options.max_retransmits, None);
        assert_eq!(options.protocol, "my-protocol");
        assert!(!options.negotiated);
        assert_eq!(options.id, Some(42));
    }

    #[test]
    fn test_data_channel_options_unordered() {
        /// Given unordered option
        /// When creating DataChannelOptions
        /// Then ordered should be false
        let options = DataChannelOptions {
            ordered: false,
            max_packet_life_time: None,
            max_retransmits: Some(5),
            protocol: String::new(),
            negotiated: false,
            id: None,
        };

        assert!(!options.ordered);
        assert_eq!(options.max_retransmits, Some(5));
    }

    #[test]
    fn test_data_channel_options_with_retransmits() {
        /// Given options with max_retransmits
        /// When configuring reliability
        /// Then max_retransmits should be set correctly
        let options = DataChannelOptions {
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: Some(10),
            protocol: "reliable-protocol".to_string(),
            negotiated: false,
            id: None,
        };

        assert_eq!(options.max_retransmits, Some(10));
        assert_eq!(options.max_packet_life_time, None);
    }

    // ===== DataChannelMessage Tests =====

    #[test]
    fn test_data_channel_message_text() {
        /// Given a text message
        /// When creating DataChannelMessage::Text
        /// Then it should contain the text
        let message = DataChannelMessage::Text("Hello, WebRTC!".to_string());

        match message {
            DataChannelMessage::Text(text) => {
                assert_eq!(text, "Hello, WebRTC!");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_data_channel_message_binary() {
        /// Given binary data
        /// When creating DataChannelMessage::Binary
        /// Then it should contain the binary data
        let data = vec![1, 2, 3, 4, 5];
        let message = DataChannelMessage::Binary(data.clone());

        match message {
            DataChannelMessage::Binary(bin_data) => {
                assert_eq!(bin_data, data);
            }
            _ => panic!("Expected Binary variant"),
        }
    }

    #[test]
    fn test_data_channel_message_empty_text() {
        /// Given an empty text message
        /// When creating DataChannelMessage::Text
        /// Then it should handle empty strings
        let message = DataChannelMessage::Text(String::new());

        match message {
            DataChannelMessage::Text(text) => {
                assert_eq!(text, "");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_data_channel_message_empty_binary() {
        /// Given empty binary data
        /// When creating DataChannelMessage::Binary
        /// Then it should handle empty vectors
        let message = DataChannelMessage::Binary(Vec::new());

        match message {
            DataChannelMessage::Binary(bin_data) => {
                assert!(bin_data.is_empty());
            }
            _ => panic!("Expected Binary variant"),
        }
    }

    // ===== DataChannelState Tests =====

    #[test]
    fn test_data_channel_state_connecting() {
        /// Given a connecting state
        /// When checking the state
        /// Then it should be Connecting
        let state = DataChannelState::Connecting;
        assert!(matches!(state, DataChannelState::Connecting));
    }

    #[test]
    fn test_data_channel_state_open() {
        /// Given an open state
        /// When checking the state
        /// Then it should be Open
        let state = DataChannelState::Open;
        assert!(matches!(state, DataChannelState::Open));
    }

    #[test]
    fn test_data_channel_state_closing() {
        /// Given a closing state
        /// When checking the state
        /// Then it should be Closing
        let state = DataChannelState::Closing;
        assert!(matches!(state, DataChannelState::Closing));
    }

    #[test]
    fn test_data_channel_state_closed() {
        /// Given a closed state
        /// When checking the state
        /// Then it should be Closed
        let state = DataChannelState::Closed;
        assert!(matches!(state, DataChannelState::Closed));
    }

    // ===== RtcDataChannel Tests =====

    #[tokio::test]
    async fn test_rtc_data_channel_creation() {
        /// Given channel options
        /// When creating a new data channel
        /// Then it should be created with correct properties
        let options = DataChannelOptions {
            ordered: true,
            max_packet_life_time: None,
            max_retransmits: Some(5),
            protocol: "test-protocol".to_string(),
            negotiated: false,
            id: None,
        };

        let channel = RtcDataChannel::new("test-channel".to_string(), options.clone());
        let label = channel.label().await;
        let stored_options = channel.options().await;

        assert_eq!(label, "test-channel");
        assert_eq!(stored_options.protocol, "test-protocol");
        assert_eq!(stored_options.max_retransmits, Some(5));
    }

    #[tokio::test]
    async fn test_rtc_data_channel_send_text_success() {
        /// Given an open data channel
        /// When sending text data
        /// Then send_text should succeed
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state
        channel.set_state_for_testing(DataChannelState::Open).await;

        let result = channel.send_text("Hello, WebRTC!").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rtc_data_channel_send_binary_success() {
        /// Given an open data channel
        /// When sending binary data
        /// Then send should succeed
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state
        channel.set_state_for_testing(DataChannelState::Open).await;

        let data = vec![1, 2, 3, 4, 5];
        let result = channel.send(&data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rtc_data_channel_recv_text_message() {
        /// Given an open data channel with a text message
        /// When calling recv
        /// Then it should return the text message
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state and add a test message
        channel.set_state_for_testing(DataChannelState::Open).await;
        channel.add_test_message(DataChannelMessage::Text("Test message".to_string())).await;

        let result = channel.recv().await;
        assert!(result.is_some());

        match result.unwrap() {
            Ok(DataChannelMessage::Text(text)) => {
                assert_eq!(text, "Test message");
            }
            _ => panic!("Expected Text message"),
        }
    }

    #[tokio::test]
    async fn test_rtc_data_channel_recv_binary_message() {
        /// Given an open data channel with a binary message
        /// When calling recv
        /// Then it should return the binary message
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state and add a test message
        channel.set_state_for_testing(DataChannelState::Open).await;
        let test_data = vec![10, 20, 30, 40, 50];
        channel.add_test_message(DataChannelMessage::Binary(test_data.clone())).await;

        let result = channel.recv().await;
        assert!(result.is_some());

        match result.unwrap() {
            Ok(DataChannelMessage::Binary(data)) => {
                assert_eq!(data, test_data);
            }
            _ => panic!("Expected Binary message"),
        }
    }

    #[tokio::test]
    async fn test_rtc_data_channel_recv_no_messages() {
        /// Given an open data channel with no messages
        /// When calling recv
        /// Then it should return None
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state
        channel.set_state_for_testing(DataChannelState::Open).await;

        let result = channel.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_rtc_data_channel_close_success() {
        /// Given an open data channel
        /// When calling close
        /// Then the channel should close successfully
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Open state
        channel.set_state_for_testing(DataChannelState::Open).await;

        let result = channel.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rtc_data_channel_close_already_closed() {
        /// Given a closed data channel
        /// When calling close again
        /// Then it should return an error
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Closed state
        channel.set_state_for_testing(DataChannelState::Closed).await;

        let result = channel.close().await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("already closed"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_rtc_data_channel_send_text_when_closed() {
        /// Given a closed data channel
        /// When attempting to send text data
        /// Then it should return an error
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Closed state
        channel.set_state_for_testing(DataChannelState::Closed).await;

        let result = channel.send_text("Hello").await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Closed"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_rtc_data_channel_send_binary_when_closed() {
        /// Given a closed data channel
        /// When attempting to send binary data
        /// Then it should return an error
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Closed state
        channel.set_state_for_testing(DataChannelState::Closed).await;

        let data = vec![1, 2, 3];
        let result = channel.send(&data).await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert!(e.to_string().contains("Closed"));
            }
            _ => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_rtc_data_channel_recv_when_closed() {
        /// Given a closed data channel
        /// When attempting to receive
        /// Then it should return None
        let options = DataChannelOptions::default();
        let channel = RtcDataChannel::new("test".to_string(), options);

        // Set channel to Closed state
        channel.set_state_for_testing(DataChannelState::Closed).await;

        let result = channel.recv().await;
        assert!(result.is_none());
    }
}
