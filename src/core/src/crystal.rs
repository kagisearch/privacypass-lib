// -----------------------------------------------------------------------------
// --------------------  interfacing with crystal  -----------------------------
// -----------------------------------------------------------------------------

use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// BACKLOG: how should hypothetical errors in error_json_retval be handled?

pub fn encode_string_for_crystal(data: String) -> Result<*const c_char> {
    let c_string = CString::new(data).with_context(|| "encode_string_for_crystal".to_string())?;
    Ok(c_string.into_raw()) // Move ownership to C
}

pub fn encode_bytes_for_crystal(data: Vec<u8>) -> Result<*const c_char> {
    let encoded_data: String = URL_SAFE.encode(&data);
    let encoded_s = encode_string_for_crystal(encoded_data)
        .with_context(|| "encode_bytes_for_crystal".to_string())?;
    Ok(encoded_s)
}

/// # Safety
///
/// Callers must provide a valid NUL terminated string pointer.
pub unsafe fn decode_string_from_crystal(cstr: *const i8) -> Result<String> {
    let c_str: &CStr = unsafe { CStr::from_ptr(cstr) };
    let rust_s = c_str
        .to_str()
        .with_context(|| "decode_string_from_crystal".to_string())?
        .to_string();
    Ok(rust_s)
}

/// # Safety
///
/// Callers must provide a valid NUL terminated string pointer.
pub unsafe fn decode_bytes_from_crystal(cstr: *const i8) -> Result<Vec<u8>> {
    let decoded_s = decode_string_from_crystal(cstr)
        .with_context(|| "decode_bytes_from_crystal 1".to_string())?;
    let decoded_bytes = URL_SAFE
        .decode(decoded_s)
        .with_context(|| "decode_bytes_from_crystal 2".to_string())?;
    Ok(decoded_bytes)
}

pub type CrystalErrorType = std::io::Error;
pub fn crystal_error(message: &str) -> CrystalErrorType {
    std::io::Error::new(std::io::ErrorKind::Other, message)
}

use serde::{Deserialize as seDeserialize, Serialize as seSerialize};

#[derive(seSerialize, seDeserialize)]
pub struct JSONRetVal {
    pub retval: String,
    pub error: String,
}

pub fn error_json_retval(message: &str) -> *const c_char {
    let error_obj = JSONRetVal {
        retval: "".to_string(),
        error: message.to_string(),
    };
    let error_s =
        serde_json::to_string(&error_obj).expect("failed to encode JSONRetVal into string"); // this should be unable to fail
    encode_string_for_crystal(error_s).expect("failed to pass encoded JSONRetVal to Crystal")
}

// -----------------------------------------------------------------------------
// -----------------------------------------------------------------------------
// -----------------------------------------------------------------------------

// FREEING MEM https://dev.to/kgrech/7-ways-to-pass-a-string-between-rust-and-c-4ieb
//             https://jakegoulding.com/rust-ffi-omnibus/string_return/
/*
#[no_mangle]
pub extern "C" fn create_string() -> *const c_char {
    let c_string = CString::new("asd").expect("CString::new failed");
    c_string.into_raw() // Move ownership to C
}
*/

/// # Safety
/// The ptr should be a valid pointer to the string allocated by rust
#[no_mangle]
pub extern "C" fn free_string(ptr: *const c_char) {
    if ptr.is_null() {
        return;
    }
    // Take the ownership back to rust and drop the owner
    let _ = unsafe { CString::from_raw(ptr as *mut _) };
}
