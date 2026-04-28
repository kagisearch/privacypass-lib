#ifndef KAGIPP_FFI_H
#define KAGIPP_FFI_H

#include <stdint.h>

/**
 * Generate a Privacy Pass token request
 *
 * # Parameters
 * - `www_authenticate_header`: JSON string containing the WWW-Authenticate header
 *   Format: {"header": "PrivateToken challenge=..., token-key=...", "error": ""}
 * - `token_count`: Number of tokens to request (typically 1-10)
 *
 * # Returns
 * JSON string containing the client state and token request:
 * {
 *   "client_state": {"state": "<hex-encoded-state>", "error": ""},
 *   "token_request": {"token_request": "<base64-encoded>", "error": ""}
 * }
 *
 * # Safety
 * - Caller must pass a valid null-terminated C string for www_authenticate_header
 * - Caller MUST call privacy_pass_free_string on the returned pointer
 * - Returns null pointer on catastrophic failure
 */
int8_t *privacy_pass_token_request(const int8_t *www_authenticate_header, uint16_t token_count);

/**
 * Finalize Privacy Pass tokens from server response
 *
 * # Parameters
 * - `www_authenticate_header`: JSON string with the original WWW-Authenticate header
 * - `client_state`: JSON string with the client state from token_request
 * - `token_response`: JSON string with the token response from the issuer
 *
 * # Returns
 * JSON string containing the finalized tokens:
 * {
 *   "tokens": ["<base64-token-1>", "<base64-token-2>", ...],
 *   "error": ""
 * }
 *
 * # Safety
 * - All parameters must be valid null-terminated C strings
 * - Caller MUST call privacy_pass_free_string on the returned pointer
 * - Returns null pointer on catastrophic failure
 */
int8_t *privacy_pass_token_finalization(const int8_t *www_authenticate_header,
                                        const int8_t *client_state,
                                        const int8_t *token_response);

/**
 * Free a string allocated by Rust
 *
 * # Safety
 * - ptr must be a pointer returned by one of the privacy_pass_* functions
 * - ptr must not be null
 * - ptr must not have been freed already
 * - After calling this, ptr must not be used again
 */
void privacy_pass_free_string(int8_t *ptr);

/**
 * Get the library version as a C string.
 *
 * # Returns
 * Pointer to a null-terminated C string containing the package version (e.g., "0.1.0").
 *
 * # Safety
 * Caller must free the returned pointer with `privacy_pass_free_string`.
 */
int8_t *privacy_pass_version(void);

#endif  /* KAGIPP_FFI_H */
