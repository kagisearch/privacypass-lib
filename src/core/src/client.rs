#![allow(unreachable_patterns)] // used to catch possible error types not yet defined by dependencies

use crate::config::{batched_tokens_mod, VoprfGroup};
use crate::crystal::{
    crystal_error, decode_bytes_from_crystal, decode_string_from_crystal,
    encode_string_for_crystal, error_json_retval, CrystalErrorType, JSONRetVal,
};
use crate::NONCE_BYTES;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use batched_tokens_mod::{
    client::{Client, IssueTokenError, IssueTokenRequestError},
    server::deserialize_public_key,
    EvaluatedElement, SerializationError, TokenResponse, NS,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::ffi::c_char;
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
pub struct JSONTokens {
    pub tokens: Vec<String>,
    pub error: String,
}
#[derive(Serialize, Deserialize)]
struct HexNonce(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
struct HexBlind(#[serde(with = "hex")] Vec<u8>);

#[derive(Serialize, Deserialize)]
pub struct StateTokenRequestRetval {
    pub token_request: String,
    pub state: String,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
struct MyTokenReqState {
    nonces_s: Vec<HexNonce>,
    blinds_s: Vec<HexBlind>,
}

use privacypass::auth::authenticate::parse_www_authenticate_header;
use voprf::{Error as voprfError, Group};

use http::header::HeaderValue;

/// Token request as specified in the spec, but with a public list of evaluated_elements
/// Adapted from privacypass/src/batched_tokens_ristretto.rs
/// NOTE: this is a "hack" to count the number of evaluated elements, since this cannot
/// be accessed from TokenResponse within privacypass-rust
#[derive(Debug, TlsDeserialize, TlsSerialize, TlsSize)]
pub struct MyTokenResponse {
    evaluated_elements: TlsVecU16<EvaluatedElement>,
    evaluated_proof: [u8; NS + NS],
}
impl MyTokenResponse {
    /// Returns the number of evaluated elements
    #[must_use]
    pub fn nr(&self) -> usize {
        self.evaluated_elements.len()
    }
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, SerializationError> {
        let mut bytes = bytes;
        Self::tls_deserialize(&mut bytes).map_err(|_| SerializationError::InvalidData)
    }
}
pub fn token_response_nr(bytes: &[u8]) -> Result<usize, CrystalErrorType> {
    match MyTokenResponse::try_from_bytes(bytes) {
        Ok(resp) => Ok(resp.nr()),
        Err(_) => Err(crystal_error("failed to deserialise MyTokenResponse")),
    }
}

/// # Safety
///
/// Callers must provide a valid NUL terminated string pointer.
#[no_mangle]
pub unsafe extern "C" fn gen_token_request(
    www_authenticate_header_cstr: *const i8,
    nr: u16,
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        let www_authenticate_header_s =
            unsafe { decode_string_from_crystal(www_authenticate_header_cstr) }?;
        let header_value: HeaderValue = HeaderValue::from_str(&www_authenticate_header_s)?;
        let challenges = parse_www_authenticate_header(&header_value)?;
        match challenges.len() {
            1 => Ok(()),
            _ => Err(crystal_error("more than one TokenChallenge in header")), // currently not as planned
        }?;
        let challenge = &challenges[0];

        // parse issuer public key
        let public_key = match deserialize_public_key(challenge.token_key()) {
        Ok(res) => Ok(res),
        Err(err) => match err {
            voprfError::Info => Err(crystal_error("Size of info is longer then [`u16::MAX`]")),
            voprfError::Input => Err(crystal_error("Size of input is empty or longer then [`u16::MAX`].")),
            voprfError::DeriveKeyPair => Err(crystal_error("Size of info and seed together are longer then `u16::MAX - 3`.")),
            voprfError::Deserialization => Err(crystal_error("Failure to deserialize bytes")),
            voprfError::Batch => Err(crystal_error("Batched items are more then [`u16::MAX`] or length don't match.")),
            voprfError::ProofVerification => Err(crystal_error("In verifiable mode, occurs when the proof failed to verify")),
            voprfError::Protocol => Err(crystal_error("The protocol has failed and can't be completed.")),
            _ => Err(crystal_error("unrecognized voprf::Error, was the the voprf-rust library updated with a new one?"))
        }}?;
        let token_challenge = challenge.token_challenge();
        // let max_age = challenge.max_age(); // currently unused

        let client = Client::new(public_key);

        // generate token nonces

        let mut nonces = Vec::with_capacity(nr as usize);

        for _ in 0..nr {
            let mut nonce = [0u8; NONCE_BYTES];
            OsRng.fill_bytes(&mut nonce);
            nonces.push(nonce);
        }

        // serialise token nonces

        let nonces_s: Vec<_> = nonces
            .iter()
            .map(|nonce| HexNonce(nonce.clone().to_vec()))
            .collect();

        // generate blinding factors

        let blinds = (0..nr)
            .map(|_| <VoprfGroup as Group>::Scalar::random(&mut OsRng))
            .collect::<Vec<_>>();

        // serialise blinding factors

        let blinds_s = blinds
            .iter()
            .map(|blind| HexBlind(blind.to_bytes().to_vec()))
            .collect::<Vec<_>>();

        // create a token request corresponding to the challenge, nonces and blinding factors

        let (token_request, _) =
            client.issue_token_request_with_params(token_challenge, nonces, blinds)?;

        // serialise token request

        let token_request_byes = token_request.tls_serialize_detached()?;
        let token_request_s = URL_SAFE.encode(token_request_byes);

        let state_vector = MyTokenReqState { nonces_s, blinds_s };
        let state_vector_s = serde_json::to_string_pretty(&state_vector)?;

        let state_token_request_rv = StateTokenRequestRetval {
            token_request: token_request_s,
            state: state_vector_s,
            error: "".to_string(),
        };

        let state_token_request_rv_s = serde_json::to_string(&state_token_request_rv)?;
        let rv = JSONRetVal {
            retval: state_token_request_rv_s,
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
pub unsafe extern "C" fn gen_token(
    www_authenticate_header_cstr: *const i8,
    client_state_cstr: *const i8,
    token_response_cstr: *const i8,
) -> *const c_char {
    // NOTE: the value of result below would not be *const i8
    //       if the begin_panic_handling and end_panic_handling macros where not there
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        // parse inputs
        let www_authenticate_header_s =
            unsafe { decode_string_from_crystal(www_authenticate_header_cstr) }?;
        let header_value: HeaderValue = HeaderValue::from_str(&www_authenticate_header_s)?;
        let challenges = parse_www_authenticate_header(&header_value)?;
        match challenges.len() {
            1 => Ok(()),
            _ => Err(crystal_error("more than one TokenChallenge in header")), // currently not as planned
        }?;
        let challenge = &challenges[0];
        let token_response_bytes = unsafe { decode_bytes_from_crystal(token_response_cstr) }?;
        let client_state_s = unsafe { decode_string_from_crystal(client_state_cstr) }?;

        // parse issuer public key
        let public_key = match deserialize_public_key(challenge.token_key()) {
            Ok(res) => Ok(res),
            Err(_) => Err(crystal_error("failed to deserialize public key")),
        }?;
        let client = Client::new(public_key);

        // parse token response
        let token_response: TokenResponse =
            match TokenResponse::try_from_bytes(token_response_bytes.as_slice()) {
                Ok(resp) => Ok(resp),
                Err(_) => Err(crystal_error("failed to deserialise TokenResponse")),
            }?;

        // parse nonce and blinding term previously sampled
        let state_vector: MyTokenReqState = match serde_json::from_str(&client_state_s) {
            Ok(req) => Ok(req),
            Err(_) => Err(crystal_error("failed deserializing client's state")),
        }?;

        let nonces: Vec<[u8; NONCE_BYTES]> = match state_vector
            .nonces_s
            .iter()
            .map(|nonce| <[u8; NONCE_BYTES]>::try_from(nonce.0.clone()))
            .collect()
        {
            Ok(res) => Ok(res),
            Err(_) => Err(crystal_error(
                "client state vector nonces are longer than expected, can't parse",
            )),
        }?;

        let blinds = match state_vector
        .blinds_s
        .iter()
        .map(|blind| VoprfGroup::deserialize_scalar(&blind.0))
        .collect::<Result<Vec<_>, _>>() {
            Ok(res) => Ok(res),
            Err(err) => match err {
                voprfError::Info => Err(crystal_error("Size of info is longer then [`u16::MAX`]")),
                voprfError::Input => Err(crystal_error("Size of input is empty or longer then [`u16::MAX`].")),
                voprfError::DeriveKeyPair => Err(crystal_error("Size of info and seed together are longer then `u16::MAX - 3`.")),
                voprfError::Deserialization => Err(crystal_error("Failure to deserialize bytes")),
                voprfError::Batch => Err(crystal_error("Batched items are more then [`u16::MAX`] or length don't match.")),
                voprfError::ProofVerification => Err(crystal_error("In verifiable mode, occurs when the proof failed to verify")),
                voprfError::Protocol => Err(crystal_error("The protocol has failed and can't be completed.")),
                _ => Err(crystal_error("unrecognized voprf::Error, was the the voprf-rust library updated with a new one?"))
            }
        }?;

        // parse token challenge from origin
        let token_challenge = challenge.token_challenge();

        // regenerate original token request sent to issuer
        let (_, mut token_states) = match client
        .issue_token_request_with_params(token_challenge, nonces, blinds) {
            Ok(res) => Ok(res),
            Err(err) => match err {
                IssueTokenRequestError::BlindingError => Err(crystal_error("failed to blind token")),
                IssueTokenRequestError::InvalidTokenChallenge => Err(crystal_error("invalid TokenChallenge")),
                _ => Err(crystal_error("unrecognized IssueTokenRequestError, was the privacypass-rust library updated with a new one?"))
            }
        }?;

        // count how many tokens can be generated with the TokenResponse, and get rid of state if got too much
        let nr: usize = token_response_nr(token_response_bytes.as_slice())?;
        if token_states.len() > nr {
            for _ in nr..token_states.len() {
                token_states.pop();
            }
        }

        // unblind token (this is where Finalize happens)
        let raw_tokens = match client.issue_tokens(&token_response, &token_states) {
            Ok(res) => Ok(res),
            Err(err) => match err {
                IssueTokenError::InvalidTokenResponse => Err(crystal_error("invalid TokenResponse")),
                _ => Err(crystal_error("unrecognized InvalidTokenResponse, was the privacypass-rust library updated with a new one?"))
            }
        }?;

        // serialise token
        let tokens_buf = raw_tokens
            .into_iter()
            .map(|token| token.tls_serialize_detached())
            .collect::<Result<Vec<Vec<u8>>, _>>()?;

        let tokens_s = tokens_buf
            .into_iter()
            .map(|token_buf| URL_SAFE.encode(token_buf))
            .collect::<Vec<_>>();

        let tokens_rv = JSONTokens {
            tokens: tokens_s,
            error: "".to_string(),
        };

        let tokens_rv_s = serde_json::to_string(&tokens_rv)?;
        let rv = JSONRetVal {
            retval: tokens_rv_s,
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
