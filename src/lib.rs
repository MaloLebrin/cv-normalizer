#![deny(clippy::all)]

mod base64;
mod image;
mod normalize;
mod pdf;
mod utils;

// Re-export all NAPI functions
pub use base64::{base64_to_buffer, buffer_to_base64};
pub use image::{
  image_to_webp, image_to_webp_from_base64, image_to_webp_from_file, optimize_image,
  optimize_image_from_base64, optimize_image_from_file, ImageOptimizeOptions,
};
pub use normalize::normalize_cv_to_pdf;
pub use pdf::extract_text_from_pdf;
