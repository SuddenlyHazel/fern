use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use iroh::EndpointId;

use crate::server::{CreateResponse, GuestInfo, UpdateResponse, RemoveResponse};

/// HTTP client for interacting with the Fern API server
#[derive(Debug, Clone)]
pub struct FernApiClient {
    client: Client,
    base_url: String,
}

/// Request payload for creating a new guest module
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub guest_name: String,
    pub module: Vec<u8>,
}

/// Request payload for updating an existing guest module
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub guest_name: String,
    pub module: Vec<u8>,
}

/// Error response from the API
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
}

impl FernApiClient {
    /// Create a new API client with the specified base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Create a new API client with default localhost configuration
    pub fn localhost() -> Self {
        Self::new("http://localhost:3000")
    }

    /// Create a new API client with custom reqwest client
    pub fn with_client(base_url: impl Into<String>, client: Client) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Get the full URL for an API endpoint
    fn api_url(&self, path: &str) -> String {
        format!("{}/api{}", self.base_url, path)
    }

    /// Handle API response and convert errors
    async fn handle_response<T: for<'de> Deserialize<'de>>(response: Response) -> Result<T> {
        if response.status().is_success() {
            response.json::<T>().await
                .map_err(|e| anyhow!("Failed to parse response: {}", e))
        } else {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            Err(anyhow!("API request failed with status {}: {}", status, error_text))
        }
    }

    /// List all guests
    /// 
    /// Makes a GET request to `/api/guest` to retrieve information about all guests.
    /// 
    /// # Returns
    /// 
    /// A vector of `GuestInfo` containing details about each guest including name,
    /// endpoint ID, and module hash.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn list_guests(&self) -> Result<Vec<GuestInfo>> {
        let response = self.client
            .get(&self.api_url("/guest"))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        Self::handle_response(response).await
    }

    /// Create a new guest module
    /// 
    /// Makes a POST request to `/api/guest` to create a new guest with the specified
    /// name and module bytecode.
    /// 
    /// # Arguments
    /// 
    /// * `guest_name` - The name for the new guest
    /// * `module` - The compiled module bytecode
    /// 
    /// # Returns
    /// 
    /// A `CreateResponse` containing the endpoint ID of the newly created guest.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the request fails, the guest name already exists,
    /// or the response cannot be parsed.
    pub async fn create_guest(&self, guest_name: String, module: Vec<u8>) -> Result<CreateResponse> {
        let request_body = CreateModuleRequest {
            guest_name,
            module,
        };

        let response = self.client
            .post(&self.api_url("/guest"))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        Self::handle_response(response).await
    }

    /// Update an existing guest module
    /// 
    /// Makes a PUT request to `/api/guest` to update an existing guest's module
    /// with new bytecode.
    /// 
    /// # Arguments
    /// 
    /// * `guest_name` - The name of the guest to update
    /// * `module` - The new compiled module bytecode
    /// 
    /// # Returns
    /// 
    /// An `UpdateResponse` containing success status, new module hash, and
    /// optionally the previous module hash.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the request fails, the guest doesn't exist,
    /// or the response cannot be parsed.
    pub async fn update_guest(&self, guest_name: String, module: Vec<u8>) -> Result<UpdateResponse> {
        let request_body = UpdateModuleRequest {
            guest_name,
            module,
        };

        let response = self.client
            .put(&self.api_url("/guest"))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        Self::handle_response(response).await
    }

    /// Delete an existing guest module
    ///
    /// Makes a DELETE request to `/api/guest/{name}` to remove an existing guest
    /// and shut down its instance.
    ///
    /// # Arguments
    ///
    /// * `guest_name` - The name of the guest to remove
    ///
    /// # Returns
    ///
    /// A `RemoveResponse` containing success status and a descriptive message.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the guest doesn't exist,
    /// or the response cannot be parsed.
    pub async fn remove_guest(&self, guest_name: String) -> Result<RemoveResponse> {
        let response = self.client
            .delete(&format!("{}/api/guest/{}", self.base_url, guest_name))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        Self::handle_response(response).await
    }

    /// Check if the API server is reachable
    ///
    /// Makes a GET request to `/api/guest` to verify connectivity.
    /// This is a simple health check that doesn't require any specific data.
    ///
    /// # Returns
    ///
    /// `true` if the server responds successfully, `false` otherwise.
    pub async fn health_check(&self) -> bool {
        match self.list_guests().await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get information about a specific guest by name
    /// 
    /// This is a convenience method that lists all guests and filters by name.
    /// 
    /// # Arguments
    /// 
    /// * `guest_name` - The name of the guest to find
    /// 
    /// # Returns
    /// 
    /// An `Option<GuestInfo>` containing the guest information if found.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the request to list guests fails.
    pub async fn get_guest_by_name(&self, guest_name: &str) -> Result<Option<GuestInfo>> {
        let guests = self.list_guests().await?;
        Ok(guests.into_iter().find(|guest| guest.name == guest_name))
    }

    /// Check if a guest exists by name
    /// 
    /// This is a convenience method that checks if a guest with the given name exists.
    /// 
    /// # Arguments
    /// 
    /// * `guest_name` - The name of the guest to check
    /// 
    /// # Returns
    /// 
    /// `true` if the guest exists, `false` otherwise.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the request to list guests fails.
    pub async fn guest_exists(&self, guest_name: &str) -> Result<bool> {
        Ok(self.get_guest_by_name(guest_name).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = FernApiClient::new("http://example.com:8080");
        assert_eq!(client.base_url, "http://example.com:8080");
        
        let localhost_client = FernApiClient::localhost();
        assert_eq!(localhost_client.base_url, "http://localhost:3000");
    }

    #[test]
    fn test_api_url_generation() {
        let client = FernApiClient::new("http://localhost:3000");
        assert_eq!(client.api_url("/guest"), "http://localhost:3000/api/guest");
        assert_eq!(client.api_url("/health"), "http://localhost:3000/api/health");
    }
}