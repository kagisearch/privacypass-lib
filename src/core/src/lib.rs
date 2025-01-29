#[macro_use]
extern crate panic_handler;

use serde::{Deserialize, Serialize};
mod batched_memory_stores;

#[derive(Serialize, Deserialize)]
struct MyTokenReqState {
    nonces_s: Vec<HexNonce>,
    blinds_s: Vec<HexBlind>,
}

#[derive(Serialize, Deserialize)]
struct BatchedTokenTestVector {
    #[serde(with = "hex", alias = "skS")]
    sk_s: Vec<u8>,
    #[serde(with = "hex", alias = "pkS")]
    pk_s: Vec<u8>,
    #[serde(with = "hex")]
    token_challenge: Vec<u8>,
    nonces: Vec<HexNonce>,
    blinds: Vec<HexBlind>,
    #[serde(with = "hex")]
    token_request: Vec<u8>,
    #[serde(with = "hex")]
    token_response: Vec<u8>,
    tokens: Vec<HexToken>,
}

#[derive(Serialize, Deserialize)]
struct PrivateTokenTestVector {
    #[serde(with = "hex", alias = "skS")]
    sk_s: Vec<u8>,
    #[serde(with = "hex", alias = "pkS")]
    pk_s: Vec<u8>,
    #[serde(with = "hex")]
    token_challenge: Vec<u8>,
    #[serde(with = "hex")]
    nonce: Vec<u8>,
    #[serde(with = "hex")]
    blind: Vec<u8>,
    #[serde(with = "hex")]
    token_request: Vec<u8>,
    #[serde(with = "hex")]
    token_response: Vec<u8>,
    #[serde(with = "hex")]
    token: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct HexNonce(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
struct HexBlind(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
struct HexToken(#[serde(with = "hex")] Vec<u8>);

use privacypass::Nonce;
const NONCE_BYTES: usize = std::mem::size_of::<Nonce>();

pub mod client;
mod config;
pub mod crystal;
#[cfg(not(target_arch = "wasm32"))]
mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use config::GroupTokenType;
#[cfg(not(target_arch = "wasm32"))]
pub use server::{GenKeysError, PrivacyPass, RustKeypair, ValidateTokenError};
