use image::ImageError;
use napi::{Error, Status};

/// Map image crate errors to NAPI errors
pub fn map_image_error(err: ImageError) -> Error {
  Error::new(
    Status::InvalidArg,
    format!("Failed to process image: {err}"),
  )
}

/// Calculate target size maintaining aspect ratio
pub fn calculate_target_size(orig_w: u32, orig_h: u32, max_side: u32) -> (u32, u32) {
  if orig_w <= max_side && orig_h <= max_side {
    return (orig_w, orig_h);
  }

  if orig_w >= orig_h {
    let ratio = max_side as f32 / orig_w as f32;
    let h = (orig_h as f32 * ratio).round().max(1.0) as u32;
    (max_side, h)
  } else {
    let ratio = max_side as f32 / orig_h as f32;
    let w = (orig_w as f32 * ratio).round().max(1.0) as u32;
    (w, max_side)
  }
}

pub fn is_pdf_mime(mime: &str) -> bool {
  mime == "application/pdf" || mime == "application/x-pdf"
}

pub fn is_supported_image_mime(mime: &str) -> bool {
  mime == "image/png" || mime == "image/jpeg" || mime == "image/jpg" || mime == "image/pjpeg"
}

