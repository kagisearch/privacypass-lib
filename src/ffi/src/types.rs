// Type definitions for FFI boundary
// These types match the JSON structures used in the API

use serde::{Deserialize, Serialize};

/// Result from token request generation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequestResult {
    pub client_state: ClientState,
    pub token_request: TokenRequest,
}

/// Client state that must be preserved between request and finalization
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientState {
    pub state: String,
    pub error: String,
}

/// Token request to send to the issuer
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequest {
    pub token_request: String,
    pub error: String,
}

/// WWW-Authenticate header wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WWWAuthenticateHeader {
    pub header: String,
    pub error: String,
}

/// Token response from issuer
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponse {
    pub token_response: String,
    pub error: String,
}
