use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const NOERROR: &str = "";
type ErrorMessage = String;
#[derive(Serialize, Deserialize, Debug)]
struct WWWAuthenticateHeader {
    header: String,
    error: ErrorMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenRequest {
    token_request: String,
    error: ErrorMessage,
}

#[derive(Serialize, Deserialize, Debug)]
struct ClientState {
    state: String,
    error: ErrorMessage,
}
#[derive(Serialize, Deserialize, Debug)]
struct TokenResponse {
    token_response: String,
    error: ErrorMessage,
}

pub fn error_json_retval(message: &str) -> String {
    let error_obj = kagippcore::crystal::JSONRetVal {
        retval: "".to_string(),
        error: message.to_string(),
    };
    // this should be unable to fail
    serde_json::to_string(&error_obj).expect("failed to encode JSONRetVal into string")
}

#[wasm_bindgen]
pub fn token_request(header_s: String, nr: u16) -> String {
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        let mut client_state = "".to_string();
        let mut token_request = "".to_string();
        let mut error: String = NOERROR.to_string();

        // parse WWW-Authenticate header
        let www_authenticate_header: WWWAuthenticateHeader = serde_json::from_str(&header_s)?;
        if www_authenticate_header.error != NOERROR {
            error = format!("{}\n{}", error, www_authenticate_header.error);
        }
        if error == NOERROR {
            let www_authenticate_header_cstr =
                kagippcore::crystal::encode_string_for_crystal(www_authenticate_header.header)?;
            let state_token_request_retval_cstr: *const i8 =
                unsafe { kagippcore::client::gen_token_request(www_authenticate_header_cstr, nr) };
            let state_token_request_retval_json = unsafe {
                kagippcore::crystal::decode_string_from_crystal(state_token_request_retval_cstr)
            }?;
            let state_token_request_retval_tuple: kagippcore::crystal::JSONRetVal =
                serde_json::from_str(&state_token_request_retval_json)?;
            let state_token_request_json = state_token_request_retval_tuple.retval;
            error = state_token_request_retval_tuple.error;
            if error == NOERROR {
                let state_token_request_tuple: kagippcore::client::StateTokenRequestRetval =
                    serde_json::from_str(&state_token_request_json)?;
                token_request = state_token_request_tuple.token_request.to_string();
                client_state = state_token_request_tuple.state.to_string();
                error = state_token_request_tuple.error;
            }
        }

        let client_state = ClientState {
            state: client_state,
            error: error.to_string(),
        };
        let token_request = TokenRequest {
            token_request,
            error: error.to_string(),
        };
        let rv = (client_state, token_request);
        let rv_s = serde_json::to_string(&rv)?;
        let out = rv_s;
        // always end like this
        Ok::<String, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}

#[wasm_bindgen]
pub async fn token_finalization(
    header_s: String,
    client_state_s: String,
    token_response_s: String,
) -> String {
    begin_panic_handling!();
    let result = panic::catch_unwind(|| {
        let mut tokens: Vec<String> = [].to_vec();
        let mut error: String = NOERROR.to_string();

        // parse WWW-Authenticate header
        let www_authenticate_header: WWWAuthenticateHeader = serde_json::from_str(&header_s)?;
        if www_authenticate_header.error != NOERROR {
            error = format!("{}\n{}", error, www_authenticate_header.error);
        }
        // parse client's state
        let client_state: ClientState = serde_json::from_str(&client_state_s)?;
        if client_state.error != NOERROR {
            error = format!("{}\n{}", error, client_state.error);
        }
        // parse token response
        let token_response: TokenResponse = serde_json::from_str(&token_response_s)?;
        if token_response.error != NOERROR {
            error = format!("{}\n{}", error, token_response.error);
        }

        if error == NOERROR {
            let www_authenticate_header_cstr =
                kagippcore::crystal::encode_string_for_crystal(www_authenticate_header.header)?;
            let client_state_cstr =
                kagippcore::crystal::encode_string_for_crystal(client_state.state)?;
            let token_response_cstr =
                kagippcore::crystal::encode_string_for_crystal(token_response.token_response)?;
            let tokens_retval_cstr: *const i8 = unsafe {
                kagippcore::client::gen_token(
                    www_authenticate_header_cstr,
                    client_state_cstr,
                    token_response_cstr,
                )
            };
            let tokens_retval_json =
                unsafe { kagippcore::crystal::decode_string_from_crystal(tokens_retval_cstr)? };
            let tokens_retval_tuple: kagippcore::crystal::JSONRetVal =
                serde_json::from_str(&tokens_retval_json)?;
            let tokens_json = tokens_retval_tuple.retval;
            error = tokens_retval_tuple.error;
            if error == NOERROR {
                let tokens_tuple: kagippcore::client::JSONTokens =
                    serde_json::from_str(&tokens_json)?;
                tokens = tokens_tuple.tokens;
                error = tokens_tuple.error;
            }
        }

        let rv = kagippcore::client::JSONTokens { tokens, error };
        let rv_s = serde_json::to_string(&rv)?;
        let out = rv_s;
        // always end like this
        Ok::<String, Box<dyn std::error::Error>>(out)
    });
    end_panic_handling!();
    result
}
