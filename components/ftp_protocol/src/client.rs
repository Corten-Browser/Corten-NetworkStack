//! FTP client implementation

use crate::{FtpConfig, responses, commands};
use network_errors::{NetworkError, NetworkResult};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// FTP client
pub struct FtpClient {
    /// Control connection stream
    pub(crate) control_stream: Option<TcpStream>,
    /// Client configuration
    pub config: FtpConfig,
}

impl FtpClient {
    /// Create a new FTP client with the given configuration
    pub fn new(config: FtpConfig) -> Self {
        Self {
            control_stream: None,
            config,
        }
    }

    /// Connect to FTP server
    pub async fn connect(&mut self, host: &str, port: u16) -> NetworkResult<()> {
        let address = format!("{}:{}", host, port);

        // Connect with timeout
        let stream = tokio::time::timeout(
            self.config.timeout,
            TcpStream::connect(&address)
        )
        .await
        .map_err(|_| NetworkError::Timeout(self.config.timeout))?
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        self.control_stream = Some(stream);

        // Read welcome message (220)
        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("Connection failed: {}", response.message)
            ));
        }

        Ok(())
    }

    /// Authenticate with username and password
    pub async fn login(&mut self, username: &str, password: &str) -> NetworkResult<()> {
        // Send USER command
        self.send_command(&commands::format_user(username)).await?;
        let response = self.read_response().await?;

        if response.code == 230 {
            // Already logged in (no password required)
            return Ok(());
        }

        if !response.is_intermediate() && !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("USER command failed: {}", response.message)
            ));
        }

        // Send PASS command
        self.send_command(&commands::format_pass(password)).await?;
        let response = self.read_response().await?;

        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("PASS command failed: {}", response.message)
            ));
        }

        Ok(())
    }

    /// List files in directory
    pub async fn list(&mut self, path: Option<&str>) -> NetworkResult<Vec<String>> {
        // Enter passive mode
        let data_address = self.enter_passive_mode().await?;

        // Connect to data port
        let mut data_stream = TcpStream::connect(&data_address)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        // Send LIST command
        self.send_command(&commands::format_list(path)).await?;
        let response = self.read_response().await?;

        if !response.is_continuation() && !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("LIST command failed: {}", response.message)
            ));
        }

        // Read data from data connection
        let mut data = Vec::new();
        data_stream.read_to_end(&mut data)
            .await
            .map_err(|e| NetworkError::Io(e))?;

        // Read completion response
        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("LIST transfer failed: {}", response.message)
            ));
        }

        // Parse listing
        let listing = String::from_utf8_lossy(&data);
        let files: Vec<String> = listing
            .lines()
            .map(|line| line.to_string())
            .collect();

        Ok(files)
    }

    /// Download file from server
    pub async fn download(&mut self, remote_path: &str) -> NetworkResult<Vec<u8>> {
        // Enter passive mode
        let data_address = self.enter_passive_mode().await?;

        // Connect to data port
        let mut data_stream = TcpStream::connect(&data_address)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        // Send RETR command
        self.send_command(&commands::format_retr(remote_path)).await?;
        let response = self.read_response().await?;

        if !response.is_continuation() && !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("RETR command failed: {}", response.message)
            ));
        }

        // Read file data
        let mut data = Vec::new();
        data_stream.read_to_end(&mut data)
            .await
            .map_err(|e| NetworkError::Io(e))?;

        // Read completion response
        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("RETR transfer failed: {}", response.message)
            ));
        }

        Ok(data)
    }

    /// Upload file to server
    pub async fn upload(&mut self, remote_path: &str, data: &[u8]) -> NetworkResult<()> {
        // Enter passive mode
        let data_address = self.enter_passive_mode().await?;

        // Connect to data port
        let mut data_stream = TcpStream::connect(&data_address)
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

        // Send STOR command
        self.send_command(&commands::format_stor(remote_path)).await?;
        let response = self.read_response().await?;

        if !response.is_continuation() && !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("STOR command failed: {}", response.message)
            ));
        }

        // Write file data
        data_stream.write_all(data)
            .await
            .map_err(|e| NetworkError::Io(e))?;

        // Close data connection
        drop(data_stream);

        // Read completion response
        let response = self.read_response().await?;
        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("STOR transfer failed: {}", response.message)
            ));
        }

        Ok(())
    }

    /// Disconnect from server
    pub async fn quit(&mut self) -> NetworkResult<()> {
        if self.control_stream.is_some() {
            self.send_command(&commands::format_quit()).await?;
            let _ = self.read_response().await; // Ignore response
            self.control_stream = None;
        }
        Ok(())
    }

    /// Enter passive mode and return data connection address
    async fn enter_passive_mode(&mut self) -> NetworkResult<String> {
        self.send_command(&commands::format_pasv()).await?;
        let response = self.read_response().await?;

        if !response.is_success() {
            return Err(NetworkError::ProtocolError(
                format!("PASV command failed: {}", response.message)
            ));
        }

        // Parse PASV response: "227 Entering Passive Mode (h1,h2,h3,h4,p1,p2)"
        let start = response.message.find('(')
            .ok_or_else(|| NetworkError::ProtocolError("Invalid PASV response".to_string()))?;
        let end = response.message.find(')')
            .ok_or_else(|| NetworkError::ProtocolError("Invalid PASV response".to_string()))?;

        let addr_str = &response.message[start + 1..end];
        let parts: Vec<&str> = addr_str.split(',').collect();

        if parts.len() != 6 {
            return Err(NetworkError::ProtocolError("Invalid PASV response format".to_string()));
        }

        let host = format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3]);
        let port1: u16 = parts[4].parse()
            .map_err(|_| NetworkError::ProtocolError("Invalid port in PASV response".to_string()))?;
        let port2: u16 = parts[5].parse()
            .map_err(|_| NetworkError::ProtocolError("Invalid port in PASV response".to_string()))?;
        let port = port1 * 256 + port2;

        Ok(format!("{}:{}", host, port))
    }

    /// Send command to control connection
    async fn send_command(&mut self, command: &str) -> NetworkResult<()> {
        let stream = self.control_stream.as_mut()
            .ok_or_else(|| NetworkError::ConnectionFailed("Not connected".to_string()))?;

        stream.write_all(command.as_bytes())
            .await
            .map_err(|e| NetworkError::Io(e))?;

        Ok(())
    }

    /// Read response from control connection
    async fn read_response(&mut self) -> NetworkResult<responses::FtpResponse> {
        let stream = self.control_stream.as_mut()
            .ok_or_else(|| NetworkError::ConnectionFailed("Not connected".to_string()))?;

        let mut buffer = vec![0u8; 4096];
        let n = stream.read(&mut buffer)
            .await
            .map_err(|e| NetworkError::Io(e))?;

        if n == 0 {
            return Err(NetworkError::ConnectionFailed("Connection closed".to_string()));
        }

        let response_str = String::from_utf8_lossy(&buffer[..n]);
        responses::parse_response(&response_str)
    }
}
