//! Networking module for the Logos programming language
//! Provides HTTP, WebSocket, and other networking capabilities

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Represents an HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// HTTP methods supported
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

/// Represents an HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Configuration for HTTP clients
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_redirects: u32,
    pub user_agent: String,
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_redirects: 5,
            user_agent: "Logos-HTTP/1.0".to_string(),
            default_headers: HashMap::new(),
        }
    }
}

/// HTTP client for making requests
pub struct HttpClient {
    config: HttpClientConfig,
    client: reqwest::Client,
}

impl HttpClient {
    /// Create a new HTTP client with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = HttpClientConfig::default();
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()?;

        Ok(Self { config, client })
    }

    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: HttpClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects as usize))
            .default_headers(
                config.default_headers
                    .iter()
                    .map(|(k, v)| (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(), reqwest::header::HeaderValue::from_str(v).unwrap()))
                    .collect()
            )
            .build()?;

        Ok(Self { config, client })
    }

    /// Make an HTTP request
    pub async fn request(&self, req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let mut builder = self.client.request(
            match req.method {
                HttpMethod::GET => reqwest::Method::GET,
                HttpMethod::POST => reqwest::Method::POST,
                HttpMethod::PUT => reqwest::Method::PUT,
                HttpMethod::DELETE => reqwest::Method::DELETE,
                HttpMethod::PATCH => reqwest::Method::PATCH,
                HttpMethod::HEAD => reqwest::Method::HEAD,
                HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
            },
            req.url.as_str(),
        );

        // Add headers
        for (key, value) in req.headers {
            builder = builder.header(&key, value);
        }

        // Add body if present
        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        let response = builder.send().await?;
        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.text().await?;

        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }

    /// Make a GET request
    pub async fn get(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request(HttpRequest {
            method: HttpMethod::GET,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }).await
    }

    /// Make a POST request
    pub async fn post(&self, url: &str, body: Option<String>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request(HttpRequest {
            method: HttpMethod::POST,
            url: url.to_string(),
            headers: HashMap::new(),
            body,
        }).await
    }

    /// Make a PUT request
    pub async fn put(&self, url: &str, body: Option<String>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request(HttpRequest {
            method: HttpMethod::PUT,
            url: url.to_string(),
            headers: HashMap::new(),
            body,
        }).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, url: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        self.request(HttpRequest {
            method: HttpMethod::DELETE,
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        }).await
    }
}

/// WebSocket connection
pub struct WebSocketConnection {
    url: String,
    connected: Arc<Mutex<bool>>,
    // In a real implementation, we would use a WebSocket library
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new(url: String) -> Self {
        Self {
            url,
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, we would establish the WebSocket connection
        println!("Connecting to WebSocket at: {}", self.url);
        
        // Update connection status
        if let Ok(mut connected) = self.connected.lock() {
            *connected = true;
        }
        
        Ok(())
    }

    /// Send a message through the WebSocket
    pub async fn send(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(connected) = self.connected.lock() {
            if *connected {
                println!("Sending WebSocket message: {}", message);
                // In a real implementation, we would send the message
                Ok(())
            } else {
                Err("WebSocket not connected".into())
            }
        } else {
            Err("Could not access connection status".into())
        }
    }

    /// Receive a message from the WebSocket
    pub async fn receive(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(connected) = self.connected.lock() {
            if *connected {
                // In a real implementation, we would receive a message
                Ok("Sample message from WebSocket".to_string())
            } else {
                Err("WebSocket not connected".into())
            }
        } else {
            Err("Could not access connection status".into())
        }
    }

    /// Disconnect from the WebSocket server
    pub fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(mut connected) = self.connected.lock() {
            *connected = false;
        }
        println!("Disconnected from WebSocket at: {}", self.url);
        Ok(())
    }

    /// Check if the WebSocket is connected
    pub fn is_connected(&self) -> bool {
        if let Ok(connected) = self.connected.lock() {
            *connected
        } else {
            false
        }
    }
}

/// Network utility functions
pub mod utils {
    use std::net::{TcpStream, UdpSocket};
    use std::io::{Read, Write};
    
    /// Check if a port is open on a given host
    pub fn is_port_open(host: &str, port: u16) -> bool {
        let addr = format!("{}:{}", host, port);
        TcpStream::connect_timeout(
            &addr.parse().expect("Invalid address"),
            std::time::Duration::from_secs(1),
        ).is_ok()
    }
    
    /// Perform a simple DNS lookup (placeholder implementation)
    pub fn dns_lookup(hostname: &str) -> Result<String, Box<dyn std::error::Error>> {
        // In a real implementation, we would perform a DNS lookup
        // For now, we'll just return the hostname as IP (this is just a placeholder)
        Ok(hostname.to_string())
    }
    
    /// Ping a host (placeholder implementation)
    pub fn ping_host(host: &str) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
        // In a real implementation, we would actually ping the host
        // For now, we'll just return a mock duration
        Ok(std::time::Duration::from_millis(10))
    }
}

/// Main networking interface for the Logos runtime
pub struct NetworkInterface {
    http_client: HttpClient,
    websockets: HashMap<String, WebSocketConnection>,
}

impl NetworkInterface {
    /// Create a new network interface with default HTTP client
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            http_client: HttpClient::new()?,
            websockets: HashMap::new(),
        })
    }
    
    /// Create a new network interface with custom HTTP configuration
    pub fn with_http_config(config: HttpClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            http_client: HttpClient::with_config(config)?,
            websockets: HashMap::new(),
        })
    }
    
    /// Get a reference to the HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }
    
    /// Create and store a WebSocket connection
    pub fn create_websocket(&mut self, id: String, url: String) -> String {
        let ws = WebSocketConnection::new(url);
        self.websockets.insert(id.clone(), ws);

        // Return the ID of the created websocket
        id
    }
    
    /// Get a WebSocket connection by ID
    pub fn get_websocket(&self, id: &str) -> Option<&WebSocketConnection> {
        self.websockets.get(id)
    }
    
    /// Remove a WebSocket connection
    pub fn remove_websocket(&mut self, id: &str) -> Option<WebSocketConnection> {
        self.websockets.remove(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_creation() {
        let client = HttpClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws = WebSocketConnection::new("ws://localhost:8080".to_string());
        assert_eq!(ws.url, "ws://localhost:8080");
        assert!(!ws.is_connected());
    }

    #[test]
    fn test_network_interface_creation() {
        let net_if = NetworkInterface::new();
        assert!(net_if.is_ok());
    }
}