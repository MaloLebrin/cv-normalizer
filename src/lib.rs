#![deny(clippy::all)]

mod base64;
mod image;
mod normalize;
mod pdf;
mod utils;

// Re-export all NAPI functions
pub use base64::{base64_to_buffer, buffer_to_base64};
pub use image::{image_to_webp, optimize_image, ImageOptimizeOptions};
pub use normalize::normalize_cv_to_pdf;
pub use pdf::extract_text_from_pdf;
