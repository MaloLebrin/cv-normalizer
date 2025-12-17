#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Normalize a CV file to PDF and optionally compress it.
///
/// V1 implementation is a no-op: it just echoes back the input bytes.
/// This lets us validate the NAPI-RS wiring and integration from Node/Strapi
/// before implementing the real image/PDF/DOCX pipeline.
#[napi]
pub fn normalize_cv_to_pdf(bytes: Uint8Array, _mime: String) -> Result<Uint8Array> {
  Ok(bytes)
}
