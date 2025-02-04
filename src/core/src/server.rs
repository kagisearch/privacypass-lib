// -----------------------------------------------------------------------------
// -----------------------  Privacy Pass Code  ---------------------------------
// -----------------------------------------------------------------------------

#![allow(unreachable_patterns)] // used to catch possible error types not yet defined by dependencies

use crate::config::{batched_tokens_mod, GroupTokenType, MemoryKeyStore, VoprfGroup, VERBOSE};

use crate::batched_memory_stores::MemoryNonceStore;
use crate::crystal::{
    crystal_error, decode_bytes_from_crystal, decode_string_from_crystal,
    encode_string_for_crystal, error_json_retval, CrystalErrorType, JSONRetVal,
};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use batched_tokens_mod::{
    server::{serialize_public_key, RedeemTokenError, Server},
    BatchedToken, BlindedElement, PublicKey, TokenRequest, TokenResponse,
};
use generic_array::GenericArray;
use http::{HeaderName, HeaderValue};
use privacypass::batched_tokens_ristretto255::server::{
    CreateKeypairError, IssueTokenResponseError,
};
use privacypass::{auth::authenticate::TokenChallenge, TokenType, TruncatedTokenKeyId};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::ffi::c_char;
use thiserror::Error;
use tls_codec::{Deserialize as TlsDeserializeTrait, Serialize as TlsSerializeTrait, TlsVecU16};
use tls_codec_derive::{TlsDeserialize, TlsSerialize, TlsSize};

#[derive(Serialize, Deserialize)]
struct KeyPair {
    sk: String,
    pk: String,
    token_type: u16,
    error: String,
}
#[derive(Serialize, Deserialize)]
struct JSONTokens {
    tokens: Vec<String>,
    error: String,
}
#[derive(Serialize, Deserialize)]
struct HexNonce(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
struct HexBlind(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
struct StateTokenRequestRetval {
    token_request: String,
    state: String,
    error: String,
}

#[derive(Serialize, Deserialize)]
struct MyTokenReqState {
    nonces_s: Vec<HexNonce>,
    blinds_s: Vec<HexBlind>,
}

/// Token request as specified in the spec, but with a modifiable list of blinded_elements
/// Adapted from privacypass/src/batched_tokens_ristretto.rs
#[derive(Debug, TlsDeserialize, TlsSerialize, TlsSize)]
pub struct MyTokenRequest {
    token_type: TokenType,
    truncated_token_key_id: TruncatedTokenKeyId,
    blinded_elements: TlsVecU16<BlindedElement>,
}
impl MyTokenRequest {
    /// Returns the number of blinded elements
    #[must_use]
    pub fn nr(&self) -> usize {
        self.blinded_elements.len()
    }

    pub fn truncate(&mut self, max_elements: usize) {
        if self.nr() <= max_elements {
            return;
        }

        for _ in max_elements..self.nr() {
            self.blinded_elements.pop();
        }
    }

    pub fn to_token_request(&self) -> Result<TokenRequest, tls_codec::Error> {
        let res_vec = self.tls_serialize_detached()?;
        let token_request = TokenRequest::tls_deserialize(&mut res_vec.as_slice());
        token_request
    }
}

use privacypass::auth::authenticate::build_www_authenticate_header;
use voprf::{derive_key, Group, Mode};

use privacypass::auth::authenticate::RedemptionContext;

#[no_mangle]
pub extern "C" fn gen_keys() -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        // sample randomness for key generation
        let mut seed = GenericArray::<_, <VoprfGroup as Group>::ScalarLen>::default();
        OsRng.fill_bytes(&mut seed);

        // setting domain separation for VOPRF secret key generation
        // as recommended by RFC 9578 (PP issuance protocol), section 5.5
        let info = b"PrivacyPass";

        // generate keys
        let rt = tokio::runtime::Runtime::new()?;
        let key_store = MemoryKeyStore::default();
        let server = Server::new();
        let public_key = rt.block_on(async {
            server
                .create_keypair_with_params(&key_store, &seed, info)
                .await
        })?;

        // serialise keys
        let pk_s = URL_SAFE.encode(serialize_public_key(public_key));
        let sk_bytes = match derive_key::<VoprfGroup>(&seed, info, Mode::Voprf) {
            Ok(res) => Ok(res.to_bytes()),
            Err(_) => Err(crystal_error("failed generating secret key")),
        }?;
        let sk_s = URL_SAFE.encode(sk_bytes);

        // construct keypair structure
        let keypair: KeyPair = KeyPair {
            pk: pk_s,
            sk: sk_s,
            token_type: GroupTokenType as u16,
            error: "".to_string(),
        };
        let keypair_json = serde_json::to_string(&keypair)?;

        if VERBOSE {
            println!("R: Issuer keypair {}", keypair_json);
        }

        let rv = JSONRetVal {
            retval: keypair_json,
            error: "".to_string(),
        };
        let rv_s = serde_json::to_string(&rv)?;
        let out = encode_string_for_crystal(rv_s)?;

        // always end like this
        Ok::<*const i8, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

#[no_mangle]
pub extern "C" fn gen_token_challenge(
    issuer_name_cstr: *const i8,
    origin_info_cstr: *const i8,
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        let issuer_name_s = unsafe { decode_string_from_crystal(issuer_name_cstr)? };
        let origin_info_s = unsafe { decode_string_from_crystal(origin_info_cstr)? };
        let redemption_context: Option<RedemptionContext> = None;

        let token_challenge: TokenChallenge = TokenChallenge::new(
            GroupTokenType,
            &issuer_name_s,
            redemption_context,
            &[origin_info_s],
        );

        let token_challenge_s = token_challenge.to_base64()?;
        if VERBOSE {
            println!("R: TokenChallenge: {:?}", token_challenge_s);
        }

        let rv = JSONRetVal {
            retval: token_challenge_s,
            error: "".to_string(),
        };
        let rv_s = serde_json::to_string(&rv)?;
        let out = encode_string_for_crystal(rv_s)?;

        // always end like this
        Ok::<*const i8, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

#[no_mangle]
// NOTE: pass max_age = 0 for no max-age component in header
pub extern "C" fn gen_www_authenticate_header(
    token_challenge_c: *const i8,
    token_key_c: *const i8,
    max_age_u32: u32,
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        // parse inputs
        let token_challenge_s = unsafe { decode_string_from_crystal(token_challenge_c)? };
        let token_key_s = unsafe { decode_string_from_crystal(token_key_c)? };

        // prepare WWW-Authenticate header value
        let token_challenge = TokenChallenge::from_base64(&token_challenge_s)?;
        let token_key_bytes: Vec<u8> = URL_SAFE.decode(token_key_s)?;
        let max_age: Option<u32> = if max_age_u32 == 0 {
            None
        } else {
            Some(max_age_u32)
        };
        let (_, www_authenticate_header) =
            build_www_authenticate_header(&token_challenge, token_key_bytes.as_slice(), max_age)?;

        // encode header value to pass to return
        let www_authenticate_header_s = www_authenticate_header.to_str()?.to_string();
        let rv = JSONRetVal {
            retval: www_authenticate_header_s,
            error: "".to_string(),
        };
        let rv_s = serde_json::to_string(&rv)?;
        let out = encode_string_for_crystal(rv_s)?;

        // always end like this
        Ok::<*const i8, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

#[no_mangle]
pub extern "C" fn gen_token_response(
    sk_cstr: *const i8,
    token_request_cstr: *const i8,
    max_nr: u16, // max number of BlindedElements that a client can send and get a response for
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        let rt = tokio::runtime::Runtime::new()?;
        let sk_s = unsafe { decode_string_from_crystal(sk_cstr)? };
        let private_key = URL_SAFE.decode(sk_s.as_bytes())?;
        let token_request_s = unsafe { decode_string_from_crystal(token_request_cstr)? };
        let token_request_bytes = URL_SAFE.decode(token_request_s)?;

        // parse token request
        let mut token_request = TokenRequest::tls_deserialize(&mut token_request_bytes.as_slice())?;
        let max_nr_usize = usize::from(max_nr);
        if token_request.nr() > max_nr_usize {
            let mut temp_token_request =
                MyTokenRequest::tls_deserialize(&mut token_request_bytes.as_slice())?;
            temp_token_request.truncate(max_nr_usize);
            token_request = temp_token_request.to_token_request()?;
            if VERBOSE {
                println!(
                    "R: TokenRequest was truncated to {:?} elements",
                    token_request.nr()
                );
            }
        }

        let key_store = MemoryKeyStore::default();
        let server = Server::new();
        rt.block_on(async {
            let _public_key = server.set_key(&key_store, &private_key).await?;
            Ok::<PublicKey, Box<dyn std::error::Error>>(_public_key)
        })?;

        // generate token response
        let token_response = rt.block_on(async {
            let _token_response = server
                .issue_token_response(&key_store, token_request)
                .await?;
            Ok::<TokenResponse, Box<dyn std::error::Error>>(_token_response)
        })?;

        let res_vec = token_response.tls_serialize_detached()?;

        let rv = JSONRetVal {
            retval: URL_SAFE.encode(res_vec),
            error: "".to_string(),
        };

        let rv_s = serde_json::to_string(&rv)?;
        let out = encode_string_for_crystal(rv_s)?;

        // always end like this
        Ok::<*const i8, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

#[no_mangle]
pub extern "C" fn validate_token(
    sk_cstr: *const i8,
    token_cstr: *const i8,
    token_challenge_cstr: *const i8,
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        // create tokio runtime
        let rt = tokio::runtime::Runtime::new()?;

        // parse inputs
        let private_key = unsafe { decode_bytes_from_crystal(sk_cstr)? };
        // let token_bytes = decode_bytes_from_crystal(token_cstr)?;
        let token_s = unsafe { decode_string_from_crystal(token_cstr)? };
        let token_s_2 = token_s.clone();
        let token_bytes = URL_SAFE.decode(token_s)?;
        let token_bytes_2 = token_bytes.clone();

        // check we did get the right amount of bytes for a token
        match token_bytes.len() == std::mem::size_of::<BatchedToken>() {
            true => Ok(()),
            false => Err(crystal_error("incorrect number of bytes for a token")),
        }?;

        // check we didn't get an alternative URL_SAFE encoding due to malleability of base64
        // NOTE: may be overkill, dependingo n how URL_SAFE.decode is implemented
        let token_s_prime = URL_SAFE.encode(token_bytes_2);
        match token_s_2 == token_s_prime {
            true => Ok(()),
            false => Err(crystal_error("received alternative encoding of token")),
        }?;

        // token challenge for possible assert check (see below)
        let token_challenge_s = unsafe { decode_string_from_crystal(token_challenge_cstr)? };
        let token_challenge = TokenChallenge::from_base64(&token_challenge_s)?;
        let challenge_digest = token_challenge.digest()?;

        // load secret key
        let key_store = MemoryKeyStore::default();
        let server = Server::new();

        // NOTE: this line loads the public key into the keystore.
        // this allows correctly redeeming the token later on.
        rt.block_on(async {
            let _public_key = server.set_key(&key_store, &private_key).await?;
            Ok::<PublicKey, Box<dyn std::error::Error>>(_public_key)
        })?;

        // the following is kind of a hack:
        // it deals with tls_codec::Error giving a very uninformative error message
        // and not working nice with the `anyhow` crate
        let token: BatchedToken = match BatchedToken::tls_deserialize(&mut token_bytes.as_slice()) {
            Ok(val) => Ok(val),
            Err(_loll) => Err(crystal_error("failed to tls_deserialize token")),
        }?;

        // check challenge digest manually.
        // NOTE: likely uneeded, happening within redeem_token via VOPRF evaluation
        match token.challenge_digest() == challenge_digest.as_slice() {
            true => Ok(()),
            false => Err(crystal_error("direct TokenChallenge digest fails")),
        }?;

        // create empty nonce_store
        // NOTE: To avoid double redemption of tokens, a nonce store should be
        //       implemented somewhere. This can be done at Crystal level.
        //       This nonce_store is required by the rust library, even if empty.
        let nonce_store = MemoryNonceStore::default();

        // verify token is valid
        let valid = rt.block_on(async {
            match server.redeem_token(&key_store, &nonce_store, token.clone())
                .await {
                Ok(_) => Ok::<bool, CrystalErrorType>(true),
                Err(err) => match err {
                    RedeemTokenError::InvalidToken => Ok::<bool, CrystalErrorType>(false),
                    RedeemTokenError::DoubleSpending => Err(crystal_error("doubly spent token (should never hit this)")), // we just created an empty nonce_store, how did you hit this???
                    RedeemTokenError::KeyIdNotFound => Err(crystal_error("key id not found")), // we just loaded the key, is the token for some key that just expired?
                    _ => Err(crystal_error("unrecognized RedeemTokenError, was the privacypass-rust library updated with a new one?"))
                }
            }
        })?;
        let valid_s = match valid {
            true => "1",
            false => "0",
        };

        let rv = JSONRetVal {
            retval: valid_s.to_string(),
            error: "".to_string(),
        };
        let rv_s = serde_json::to_string(&rv)?;
        let out = encode_string_for_crystal(rv_s)?;

        Ok::<*const i8, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

pub struct PrivacyPass {}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ValidateTokenError {
    #[error("failed to serialize token challenge")]
    Serialize(#[from] privacypass::auth::authenticate::SerializationError),
    #[error("incorrect number of bytes ({0}) for a token")]
    WrongTokenSize(usize),
    #[error("failed to construct keypair")]
    CreateKeypair(#[from] CreateKeypairError),
    #[error("failed to deserialize token")]
    TlsDeserialize(#[from] tls_codec::Error),
    #[error("direct TokenChallenge digest fails")]
    ChallengeDigest,
    #[error("doubly spent token (should never hit this)")]
    DoubleSpending,
    #[error("key id not found")]
    KeyIdNotFound,
    #[error("failed to redeem token")]
    RedeemToken(#[from] RedeemTokenError),
}

#[derive(Error, Debug)]
pub enum GenKeysError {
    #[error("failed to construct keypair")]
    CreateKeypair(#[from] CreateKeypairError),
    #[error("failed generating secret key")]
    DeriveKey(voprf::Error),
}

#[derive(Error, Debug)]
pub enum GenTokenResponseError {
    #[error("requested {0} tokens, max is {1}")]
    RequestedTooManyTokens(usize, usize),
    #[error("failed to construct keypair")]
    CreateKeypair(#[from] CreateKeypairError),
    #[error("failed to issue token response")]
    IssueTokenResponse(#[from] IssueTokenResponseError),
}

#[derive(Debug)]
pub struct RustKeypair {
    pub public_key: Vec<u8>,
    pub secret_key: [u8; 32],
    pub token_type: TokenType,
}

impl PrivacyPass {
    pub fn new() -> Self {
        PrivacyPass {}
    }

    pub async fn validate_token(
        &self,
        token: &[u8],
        private_key: &[u8],
    ) -> Result<bool, ValidateTokenError> {
        if token.len() != std::mem::size_of::<BatchedToken>() {
            return Err(ValidateTokenError::WrongTokenSize(token.len()));
        }

        // Needed to make sure public key is in key store.
        let server = Server::new();
        let key_store = MemoryKeyStore::default();
        let nonce_store = MemoryNonceStore::default();
        let _pub_key = server.set_key(&key_store, private_key).await?;

        let tkn = token.to_vec();
        let token = BatchedToken::tls_deserialize(&mut tkn.as_slice())?;

        match server
            .redeem_token(&key_store, &nonce_store, token.clone())
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => match err {
                RedeemTokenError::InvalidToken => Ok(false),
                RedeemTokenError::DoubleSpending => Err(ValidateTokenError::DoubleSpending), // we just created an empty nonce_store, how did you hit this???
                RedeemTokenError::KeyIdNotFound => Err(ValidateTokenError::KeyIdNotFound), // we just loaded the key, is the token for some key that just expired?
                e => Err(ValidateTokenError::RedeemToken(e)),
            },
        }
    }

    pub async fn gen_keys(&self) -> Result<RustKeypair, GenKeysError> {
        // sample randomness for key generation
        let mut seed = GenericArray::<_, <VoprfGroup as Group>::ScalarLen>::default();
        OsRng.fill_bytes(&mut seed);

        // setting domain separation for VOPRF secret key generation
        // as recommended by RFC 9578 (PP issuance protocol), section 5.5
        let info = b"PrivacyPass";
        let server = Server::new();
        let key_store = MemoryKeyStore::default();

        let public_key = server
            .create_keypair_with_params(&key_store, &seed, info)
            .await?;

        let secret_key =
            derive_key::<VoprfGroup>(&seed, info, Mode::Voprf).map_err(GenKeysError::DeriveKey)?;

        Ok(RustKeypair {
            public_key: serialize_public_key(public_key),
            secret_key: secret_key.to_bytes(),
            token_type: TokenType::BatchedTokenRistretto255,
        })
    }

    pub fn gen_www_authenticate_header(
        token_key: &[u8],
    ) -> Result<(HeaderName, HeaderValue), String> {
        let token_challenge = Self::gen_token_challenge();
        build_www_authenticate_header(&token_challenge, token_key, None)
            .or(Err("invalid token challenge".to_string()))
    }

    pub fn gen_token_challenge() -> TokenChallenge {
        TokenChallenge::new(
            GroupTokenType,
            "privacy-pass-issuer.kagi.com",
            None, /* redemption_context */
            &["privacy-pass-origin.kagi.com".to_string()],
        )
    }

    pub async fn gen_token_response(
        &self,
        private_key: &[u8],
        token_request: TokenRequest,
        max_requests: usize,
    ) -> Result<TokenResponse, GenTokenResponseError> {
        if token_request.nr() > max_requests {
            return Err(GenTokenResponseError::RequestedTooManyTokens(
                token_request.nr(),
                max_requests,
            ));
        }

        let server = Server::new();
        let key_store = MemoryKeyStore::default();

        server.set_key(&key_store, private_key).await?;
        Ok(server
            .issue_token_response(&key_store, token_request)
            .await?)
    }
}

impl Default for PrivacyPass {
    fn default() -> Self {
        Self::new()
    }
}
