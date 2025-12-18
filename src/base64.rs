use base64::{engine::general_purpose, Engine as _};
use napi::bindgen_prelude::Uint8Array;
use napi::{Error, Status};
use napi_derive::napi;

/// Convert a buffer to Base64 string.
#[napi]
pub fn buffer_to_base64(buffer: Uint8Array) -> String {
  general_purpose::STANDARD.encode(&buffer.to_vec())
}

/// Convert a Base64 string to a buffer.
#[napi]
pub fn base64_to_buffer(base64: String) -> napi::Result<Vec<u8>> {
  general_purpose::STANDARD
    .decode(&base64)
    .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to decode Base64: {e}")))
}
