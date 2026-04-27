#[macro_use]
extern crate panic_handler;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod types;
pub use types::*;

use kagippcore::crystal::{
    decode_string_from_crystal, encode_string_for_crystal, JSONRetVal,
};

// -----------------------------------------------------------------------------
// --------------------------  Helper Functions  -------------------------------
// -----------------------------------------------------------------------------

/// Convert a Rust String to a C-compatible string pointer (i8 for compatibility with core)
/// Caller is responsible for freeing this with privacy_pass_free_string
fn string_to_c_char(s: String) -> *mut i8 {
    match CString::new(s) {
        Ok(c_string) => c_string.into_raw() as *mut i8,
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert a C string pointer to a Rust String
/// # Safety
/// Caller must ensure ptr is a valid null-terminated C string
unsafe fn c_char_to_string(ptr: *const i8) -> Result<String, String> {
    if ptr.is_null() {
        return Err("Null pointer passed to FFI".to_string());
    }

    let c_str = unsafe { CStr::from_ptr(ptr as *const c_char) };
    c_str
        .to_str()
        .map(|s| s.to_string())
        .map_err(|e| format!("Invalid UTF-8 string: {}", e))
}

/// Reclaim and free a C string that was allocated by Rust (via CString::into_raw)
/// # Safety
/// ptr must have been allocated by CString::into_raw or encode_string_for_crystal
unsafe fn free_rust_cstr(ptr: *const i8) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr as *mut c_char);
    }
}

/// Create an error JSON response
fn create_error_response(error_msg: &str) -> String {
    serde_json::json!({
        "error": error_msg,
    })
    .to_string()
}

// -----------------------------------------------------------------------------
// --------------------------  Public FFI API  ---------------------------------
// -----------------------------------------------------------------------------

/// Generate a Privacy Pass token request
///
/// # Parameters
/// - `www_authenticate_header`: JSON string containing the WWW-Authenticate header
///   Format: {"header": "PrivateToken challenge=..., token-key=...", "error": ""}
/// - `token_count`: Number of tokens to request (typically 1-10)
///
/// # Returns
/// JSON string containing the client state and token request:
/// {
///   "client_state": {"state": "<hex-encoded-state>", "error": ""},
///   "token_request": {"token_request": "<base64-encoded>", "error": ""}
/// }
///
/// # Safety
/// - Caller must pass a valid null-terminated C string for www_authenticate_header
/// - Caller MUST call privacy_pass_free_string on the returned pointer
/// - Returns null pointer on catastrophic failure
#[no_mangle]
pub unsafe extern "C" fn privacy_pass_token_request(
    www_authenticate_header: *const i8,
    token_count: u16,
) -> *mut i8 {
    begin_panic_handling!();

    let result = panic::catch_unwind(|| {
        // Parse input
        let header_json = unsafe { c_char_to_string(www_authenticate_header)? };
        let header_obj: WWWAuthenticateHeader = serde_json::from_str(&header_json)
            .map_err(|e| format!("Failed to parse header JSON: {}", e))?;

        if !header_obj.error.is_empty() {
            return Ok(string_to_c_char(
                create_error_response(&format!("Input error: {}", header_obj.error))
            ));
        }

        // Convert to format expected by core library
        let header_cstr = encode_string_for_crystal(header_obj.header)
            .map_err(|e| format!("Failed to encode header: {}", e))?;

        // Call core library function
        let state_token_request_cstr = unsafe {
            kagippcore::client::gen_token_request(header_cstr, token_count)
        };

        // Free intermediate C string (already consumed by gen_token_request)
        unsafe { free_rust_cstr(header_cstr); }

        // Parse result from core library
        let result_json = unsafe { decode_string_from_crystal(state_token_request_cstr) };
        // Free intermediate C string (already read by decode_string_from_crystal)
        unsafe { free_rust_cstr(state_token_request_cstr); }
        let result_json = result_json
            .map_err(|e| format!("Failed to decode core library response: {}", e))?;
        let result_obj: JSONRetVal = serde_json::from_str(&result_json)
            .map_err(|e| format!("Failed to parse core library result: {}", e))?;

        if !result_obj.error.is_empty() {
            return Ok(string_to_c_char(create_error_response(&result_obj.error)));
        }

        // Parse the inner result
        let state_token_request: kagippcore::client::StateTokenRequestRetval =
            serde_json::from_str(&result_obj.retval)
                .map_err(|e| format!("Failed to parse token request result: {}", e))?;

        // Format response as expected by Dart
        let response = TokenRequestResult {
            client_state: ClientState {
                state: state_token_request.state,
                error: state_token_request.error.clone(),
            },
            token_request: TokenRequest {
                token_request: state_token_request.token_request,
                error: state_token_request.error,
            },
        };

        let response_json = serde_json::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;

        Ok::<*mut i8, String>(string_to_c_char(response_json))
    });

    match result {
        Ok(Ok(ptr)) => ptr,
        Ok(Err(e)) => string_to_c_char(create_error_response(&e)),
        Err(_) => string_to_c_char(create_error_response("Panic occurred in token_request")),
    }
}

/// Finalize Privacy Pass tokens from server response
///
/// # Parameters
/// - `www_authenticate_header`: JSON string with the original WWW-Authenticate header
/// - `client_state`: JSON string with the client state from token_request
/// - `token_response`: JSON string with the token response from the issuer
///
/// # Returns
/// JSON string containing the finalized tokens:
/// {
///   "tokens": ["<base64-token-1>", "<base64-token-2>", ...],
///   "error": ""
/// }
///
/// # Safety
/// - All parameters must be valid null-terminated C strings
/// - Caller MUST call privacy_pass_free_string on the returned pointer
/// - Returns null pointer on catastrophic failure
#[no_mangle]
pub unsafe extern "C" fn privacy_pass_token_finalization(
    www_authenticate_header: *const i8,
    client_state: *const i8,
    token_response: *const i8,
) -> *mut i8 {
    begin_panic_handling!();

    let result = panic::catch_unwind(|| {
        // Parse inputs
        let header_json = unsafe { c_char_to_string(www_authenticate_header)? };
        let state_json = unsafe { c_char_to_string(client_state)? };
        let response_json = unsafe { c_char_to_string(token_response)? };

        let header_obj: WWWAuthenticateHeader = serde_json::from_str(&header_json)
            .map_err(|e| format!("Failed to parse header JSON: {}", e))?;
        let state_obj: ClientState = serde_json::from_str(&state_json)
            .map_err(|e| format!("Failed to parse state JSON: {}", e))?;
        let response_obj: TokenResponse = serde_json::from_str(&response_json)
            .map_err(|e| format!("Failed to parse response JSON: {}", e))?;

        // Check for input errors
        if !header_obj.error.is_empty() {
            return Ok(string_to_c_char(
                create_error_response(&format!("Header error: {}", header_obj.error))
            ));
        }
        if !state_obj.error.is_empty() {
            return Ok(string_to_c_char(
                create_error_response(&format!("State error: {}", state_obj.error))
            ));
        }
        if !response_obj.error.is_empty() {
            return Ok(string_to_c_char(
                create_error_response(&format!("Response error: {}", response_obj.error))
            ));
        }

        // Convert to format expected by core library
        let header_cstr = encode_string_for_crystal(header_obj.header)
            .map_err(|e| format!("Failed to encode header: {}", e))?;
        let state_cstr = encode_string_for_crystal(state_obj.state)
            .map_err(|e| format!("Failed to encode state: {}", e))?;
        let response_cstr = encode_string_for_crystal(response_obj.token_response)
            .map_err(|e| format!("Failed to encode response: {}", e))?;

        // Call core library function
        let tokens_cstr = unsafe {
            kagippcore::client::gen_token(header_cstr, state_cstr, response_cstr)
        };

        // Free intermediate C strings (already consumed by gen_token)
        unsafe {
            free_rust_cstr(header_cstr);
            free_rust_cstr(state_cstr);
            free_rust_cstr(response_cstr);
        }

        // Parse result from core library
        let tokens_result_json = unsafe { decode_string_from_crystal(tokens_cstr) };
        // Free intermediate C string (already read by decode_string_from_crystal)
        unsafe { free_rust_cstr(tokens_cstr); }
        let tokens_result_json = tokens_result_json
            .map_err(|e| format!("Failed to decode core library response: {}", e))?;
        let tokens_result_obj: JSONRetVal = serde_json::from_str(&tokens_result_json)
            .map_err(|e| format!("Failed to parse core library result: {}", e))?;

        if !tokens_result_obj.error.is_empty() {
            return Ok(string_to_c_char(create_error_response(&tokens_result_obj.error)));
        }

        // Parse the tokens
        let tokens: kagippcore::client::JSONTokens = serde_json::from_str(&tokens_result_obj.retval)
            .map_err(|e| format!("Failed to parse tokens: {}", e))?;

        // Return as-is (already in correct format)
        let response_json = serde_json::to_string(&tokens)
            .map_err(|e| format!("Failed to serialize tokens: {}", e))?;

        Ok::<*mut i8, String>(string_to_c_char(response_json))
    });

    match result {
        Ok(Ok(ptr)) => ptr,
        Ok(Err(e)) => string_to_c_char(create_error_response(&e)),
        Err(_) => string_to_c_char(create_error_response("Panic occurred in token_finalization")),
    }
}

/// Free a string allocated by Rust
///
/// # Safety
/// - ptr must be a pointer returned by one of the privacy_pass_* functions
/// - ptr must not be null
/// - ptr must not have been freed already
/// - After calling this, ptr must not be used again
#[no_mangle]
pub unsafe extern "C" fn privacy_pass_free_string(ptr: *mut i8) {
    if ptr.is_null() {
        return;
    }
    // Take ownership back and let it drop
    unsafe {
        let _ = CString::from_raw(ptr as *mut c_char);
    }
}

/// Get the library version as a C string.
///
/// # Returns
/// Pointer to a null-terminated C string containing the package version (e.g., "0.1.0").
///
/// # Safety
/// Caller must free the returned pointer with `privacy_pass_free_string`.
#[no_mangle]
pub unsafe extern "C" fn privacy_pass_version() -> *mut i8 {
    CString::new(env!("CARGO_PKG_VERSION")).unwrap().into_raw()
}

// -----------------------------------------------------------------------------
// ---------------------------  Unit Tests  ------------------------------------
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_conversion() {
        let test_str = "Hello, World!".to_string();
        let c_str = string_to_c_char(test_str.clone());
        assert!(!c_str.is_null());

        let recovered = unsafe { c_char_to_string(c_str) };
        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), test_str);

        unsafe { privacy_pass_free_string(c_str); }
    }

    #[test]
    fn test_null_pointer_handling() {
        let result = unsafe { c_char_to_string(std::ptr::null()) };
        assert!(result.is_err());
    }

}
