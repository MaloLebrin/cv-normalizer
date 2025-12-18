use std::io::Write as IoWrite;
use std::process::Command;

use napi::bindgen_prelude::Uint8Array;
use napi::{Error, Status};
use napi_derive::napi;
use tempfile::NamedTempFile;

/// Extract text content from a PDF document.
///
/// This replaces pdf-parse (JS) with a native Rust implementation using pdf-extract.
/// Returns the extracted text as a single string, with pages separated by newlines.
#[napi]
pub fn extract_text_from_pdf(bytes: Uint8Array) -> napi::Result<String> {
  let input = bytes.to_vec();

  let text = pdf_extract::extract_text_from_mem(&input).map_err(|e| {
    Error::new(
      Status::InvalidArg,
      format!("Failed to extract text from PDF: {e}"),
    )
  })?;

  Ok(text)
}

/// Try to optimize/compress a PDF using Ghostscript (`gs`) if available.
///
/// - Returns `Some(optimized_bytes)` when optimization succeeds and the result is strictly smaller.
/// - Returns `None` if Ghostscript is not available or any step fails (callers should fall back to
///   the original bytes).
pub(crate) fn try_optimize_pdf_with_ghostscript(input: &[u8]) -> Option<Vec<u8>> {
  // Create temporary input file
  let mut in_file = NamedTempFile::new().ok()?;
  IoWrite::write_all(&mut in_file, input).ok()?;

  // Create temporary output file
  let out_file = NamedTempFile::new().ok()?;
  let out_path = out_file.path().to_path_buf();

  // Run Ghostscript to recompress the PDF.
  // -dPDFSETTINGS=/screen is an aggressive but reasonable default for CVs.
  let status = Command::new("gs")
    .arg("-sDEVICE=pdfwrite")
    .arg("-dCompatibilityLevel=1.4")
    .arg("-dPDFSETTINGS=/screen")
    .arg("-dNOPAUSE")
    .arg("-dQUIET")
    .arg("-dBATCH")
    .arg(format!("-sOutputFile={}", out_path.to_string_lossy()))
    .arg(in_file.path())
    .status()
    .ok()?;

  if !status.success() {
    return None;
  }

  let optimized = std::fs::read(&out_path).ok()?;
  if optimized.is_empty() || optimized.len() >= input.len() {
    return None;
  }

  Some(optimized)
}
