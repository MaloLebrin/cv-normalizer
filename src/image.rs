use std::io::Cursor;

use base64::{engine::general_purpose, Engine as _};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat};
use napi::bindgen_prelude::Uint8Array;
use napi::{Error, Status};
use napi_derive::napi;

use crate::utils::{calculate_target_size, map_image_error};

/// Convert any supported image format (PNG, JPEG, …) to WebP.
///
/// This mirrors the behavior of the Transformer example on the NAPI-RS homepage:
/// it decodes the image from memory and re-encodes it as WebP in memory.
///
/// **Input:** Buffer (Uint8Array) - Binary image data in memory
#[napi]
pub fn image_to_webp(bytes: Uint8Array) -> napi::Result<Vec<u8>> {
  let input = bytes.to_vec();
  let img = image::load_from_memory(&input).map_err(map_image_error)?;

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);
    img
      .write_to(&mut cursor, ImageFormat::WebP)
      .map_err(map_image_error)?;
  }

  Ok(out)
}

/// Convert an image file to WebP format.
///
/// **Input:** File path (String) - Path to the image file on disk
#[napi]
pub fn image_to_webp_from_file(path: String) -> napi::Result<Vec<u8>> {
  let img = image::open(&path).map_err(|e| {
    Error::new(
      Status::InvalidArg,
      format!("Failed to open image file '{}': {e}", path),
    )
  })?;

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);
    img
      .write_to(&mut cursor, ImageFormat::WebP)
      .map_err(map_image_error)?;
  }

  Ok(out)
}

/// Convert a Base64-encoded image to WebP format.
///
/// **Input:** Base64 string (String) - Base64-encoded image data
#[napi]
pub fn image_to_webp_from_base64(base64: String) -> napi::Result<Vec<u8>> {
  let bytes = general_purpose::STANDARD.decode(&base64).map_err(|e| {
    Error::new(
      Status::InvalidArg,
      format!("Failed to decode Base64: {e}"),
    )
  })?;

  let img = image::load_from_memory(&bytes).map_err(map_image_error)?;

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);
    img
      .write_to(&mut cursor, ImageFormat::WebP)
      .map_err(map_image_error)?;
  }

  Ok(out)
}

/// Optimize an image: resize and/or compress.
///
/// Options:
/// - `max_width`: Maximum width in pixels (0 = no limit)
/// - `max_height`: Maximum height in pixels (0 = no limit)
/// - `quality`: JPEG quality (1-100, only used if format is JPEG)
/// - `format`: Output format ("jpeg", "png", "webp", or "auto" to keep original)
#[napi(object)]
pub struct ImageOptimizeOptions {
  pub max_width: Option<u32>,
  pub max_height: Option<u32>,
  pub quality: Option<u8>,
  pub format: Option<String>,
}

/// Optimize an image from a buffer.
///
/// **Input:** Buffer (Uint8Array) - Binary image data in memory
#[napi]
pub fn optimize_image(
  bytes: Uint8Array,
  options: Option<ImageOptimizeOptions>,
) -> napi::Result<Vec<u8>> {
  let input = bytes.to_vec();
  let img = image::load_from_memory(&input).map_err(map_image_error)?;

  let opts = options.unwrap_or(ImageOptimizeOptions {
    max_width: None,
    max_height: None,
    quality: Some(80),
    format: Some("auto".to_string()),
  });

  let (orig_w, orig_h) = img.dimensions();
  let max_w = opts.max_width.unwrap_or(0);
  let max_h = opts.max_height.unwrap_or(0);

  let resized = if (max_w > 0 && orig_w > max_w) || (max_h > 0 && orig_h > max_h) {
    let target_w = if max_w > 0 && orig_w > max_w {
      max_w
    } else {
      orig_w
    };
    let target_h = if max_h > 0 && orig_h > max_h {
      max_h
    } else {
      orig_h
    };
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, target_w.max(target_h));
    img.resize_exact(final_w, final_h, FilterType::Lanczos3)
  } else {
    img
  };

  let format_str = opts.format.as_deref().unwrap_or("auto");
  let quality = opts.quality.unwrap_or(80).clamp(1, 100);

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);

    match format_str {
      "jpeg" | "jpg" => {
        let (w, h) = resized.dimensions();
        let rgb = resized.to_rgb8();
        let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
          .encode(&rgb, w, h, ColorType::Rgb8.into())
          .map_err(map_image_error)?;
      }
      "png" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      "webp" => {
        resized
          .write_to(&mut cursor, ImageFormat::WebP)
          .map_err(map_image_error)?;
      }
      "auto" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      _ => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
    }
  }

  Ok(out)
}

/// Optimize an image from a file path.
///
/// **Input:** File path (String) - Path to the image file on disk
#[napi]
pub fn optimize_image_from_file(
  path: String,
  options: Option<ImageOptimizeOptions>,
) -> napi::Result<Vec<u8>> {
  let img = image::open(&path).map_err(|e| {
    Error::new(
      Status::InvalidArg,
      format!("Failed to open image file '{}': {e}", path),
    )
  })?;

  let opts = options.unwrap_or(ImageOptimizeOptions {
    max_width: None,
    max_height: None,
    quality: Some(80),
    format: Some("auto".to_string()),
  });

  let (orig_w, orig_h) = img.dimensions();
  let max_w = opts.max_width.unwrap_or(0);
  let max_h = opts.max_height.unwrap_or(0);

  let resized = if (max_w > 0 && orig_w > max_w) || (max_h > 0 && orig_h > max_h) {
    let target_w = if max_w > 0 && orig_w > max_w {
      max_w
    } else {
      orig_w
    };
    let target_h = if max_h > 0 && orig_h > max_h {
      max_h
    } else {
      orig_h
    };
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, target_w.max(target_h));
    img.resize_exact(final_w, final_h, FilterType::Lanczos3)
  } else {
    img
  };

  let format_str = opts.format.as_deref().unwrap_or("auto");
  let quality = opts.quality.unwrap_or(80).clamp(1, 100);

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);

    match format_str {
      "jpeg" | "jpg" => {
        let (w, h) = resized.dimensions();
        let rgb = resized.to_rgb8();
        let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
          .encode(&rgb, w, h, ColorType::Rgb8.into())
          .map_err(map_image_error)?;
      }
      "png" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      "webp" => {
        resized
          .write_to(&mut cursor, ImageFormat::WebP)
          .map_err(map_image_error)?;
      }
      "auto" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      _ => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
    }
  }

  Ok(out)
}

/// Optimize an image from a Base64-encoded string.
///
/// **Input:** Base64 string (String) - Base64-encoded image data
#[napi]
pub fn optimize_image_from_base64(
  base64: String,
  options: Option<ImageOptimizeOptions>,
) -> napi::Result<Vec<u8>> {
  let bytes = general_purpose::STANDARD.decode(&base64).map_err(|e| {
    Error::new(
      Status::InvalidArg,
      format!("Failed to decode Base64: {e}"),
    )
  })?;

  let img = image::load_from_memory(&bytes).map_err(map_image_error)?;

  let opts = options.unwrap_or(ImageOptimizeOptions {
    max_width: None,
    max_height: None,
    quality: Some(80),
    format: Some("auto".to_string()),
  });

  let (orig_w, orig_h) = img.dimensions();
  let max_w = opts.max_width.unwrap_or(0);
  let max_h = opts.max_height.unwrap_or(0);

  let resized = if (max_w > 0 && orig_w > max_w) || (max_h > 0 && orig_h > max_h) {
    let target_w = if max_w > 0 && orig_w > max_w {
      max_w
    } else {
      orig_w
    };
    let target_h = if max_h > 0 && orig_h > max_h {
      max_h
    } else {
      orig_h
    };
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, target_w.max(target_h));
    img.resize_exact(final_w, final_h, FilterType::Lanczos3)
  } else {
    img
  };

  let format_str = opts.format.as_deref().unwrap_or("auto");
  let quality = opts.quality.unwrap_or(80).clamp(1, 100);

  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);

    match format_str {
      "jpeg" | "jpg" => {
        let (w, h) = resized.dimensions();
        let rgb = resized.to_rgb8();
        let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
          .encode(&rgb, w, h, ColorType::Rgb8.into())
          .map_err(map_image_error)?;
      }
      "png" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      "webp" => {
        resized
          .write_to(&mut cursor, ImageFormat::WebP)
          .map_err(map_image_error)?;
      }
      "auto" => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
      _ => {
        resized
          .write_to(&mut cursor, ImageFormat::Png)
          .map_err(map_image_error)?;
      }
    }
  }

  Ok(out)
}

pub(crate) fn encode_to_jpeg(img: DynamicImage, quality: u8) -> Result<Vec<u8>, image::ImageError> {
  let (w, h) = img.dimensions();
  let rgb = img.to_rgb8();

  let mut jpeg_bytes = Vec::new();
  {
    let mut cursor = Cursor::new(&mut jpeg_bytes);
    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);
    encoder.encode(&rgb, w, h, ColorType::Rgb8.into())?;
  }

  Ok(jpeg_bytes)
}
