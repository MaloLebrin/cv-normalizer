use std::fs;
use std::io::{BufReader, Cursor};
use std::path::Path;

use base64::{engine::general_purpose, Engine as _};
use exif::{In, Reader as ExifReader, Tag};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::metadata::Orientation as ImageOrientation;
use image::ImageDecoder;
use image::ImageReader;
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat};
use napi::bindgen_prelude::Uint8Array;
use napi::{Error, Status};
use napi_derive::napi;
use walkdir::WalkDir;

use crate::utils::{calculate_target_size, map_image_error};

/// Read EXIF orientation from raw image bytes (JPEG, etc.) when present.
/// Returns None if no valid orientation tag is found.
fn orientation_from_exif_bytes(bytes: &[u8]) -> Option<ImageOrientation> {
  let mut reader = BufReader::new(Cursor::new(bytes));
  let exif = ExifReader::new().read_from_container(&mut reader).ok()?;
  let field = exif.get_field(Tag::Orientation, In::PRIMARY)?;
  let v = field.value.get_uint(0)?;
  ImageOrientation::from_exif(v as u8)
}

/// Load an image from bytes and apply EXIF orientation so the image is displayed correctly
/// (no sideways or upside-down output when re-encoding to WebP or other formats).
/// Uses the exif crate to read orientation when the decoder does not provide it.
pub(crate) fn load_image_with_orientation(bytes: &[u8]) -> Result<DynamicImage, image::ImageError> {
  let orientation_from_exif = orientation_from_exif_bytes(bytes);

  let reader = ImageReader::new(Cursor::new(bytes))
    .with_guessed_format()
    .map_err(image::ImageError::IoError)?;
  let mut decoder = reader.into_decoder()?;
  let orientation_from_decoder = decoder.orientation().ok();
  let orientation = orientation_from_exif
    .or(orientation_from_decoder)
    .unwrap_or(ImageOrientation::NoTransforms);

  let mut img = DynamicImage::from_decoder(decoder)?;
  img.apply_orientation(orientation);
  Ok(img)
}

/// Load an image from a file path and apply EXIF orientation.
fn load_image_with_orientation_from_path(path: &Path) -> Result<DynamicImage, image::ImageError> {
  let bytes = fs::read(path).map_err(image::ImageError::IoError)?;
  let orientation_from_exif = orientation_from_exif_bytes(&bytes);

  let reader = ImageReader::new(Cursor::new(&bytes))
    .with_guessed_format()
    .map_err(image::ImageError::IoError)?;
  let mut decoder = reader.into_decoder()?;
  let orientation_from_decoder = decoder.orientation().ok();
  let orientation = orientation_from_exif
    .or(orientation_from_decoder)
    .unwrap_or(ImageOrientation::NoTransforms);

  let mut img = DynamicImage::from_decoder(decoder)?;
  img.apply_orientation(orientation);
  Ok(img)
}

/// Convert any supported image format (PNG, JPEG, …) to WebP.
///
/// This mirrors the behavior of the Transformer example on the NAPI-RS homepage:
/// it decodes the image from memory and re-encodes it as WebP in memory.
///
/// **Input:** Buffer (Uint8Array) - Binary image data in memory
#[napi]
pub fn image_to_webp(bytes: Uint8Array) -> napi::Result<Vec<u8>> {
  let input = bytes.to_vec();
  let img = load_image_with_orientation(&input).map_err(map_image_error)?;

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
  let img = load_image_with_orientation_from_path(Path::new(&path)).map_err(|e| {
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
  let bytes = general_purpose::STANDARD
    .decode(&base64)
    .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to decode Base64: {e}")))?;

  let img = load_image_with_orientation(&bytes).map_err(map_image_error)?;

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
  let img = load_image_with_orientation(&input).map_err(map_image_error)?;

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
    // Determine the maximum side constraint
    let max_side = if max_w > 0 && max_h > 0 {
      // Both constraints: use the smaller of the two to fit within both
      max_w.min(max_h)
    } else if max_w > 0 {
      // Only width constraint: use width as max_side
      max_w
    } else if max_h > 0 {
      // Only height constraint: use height as max_side
      max_h
    } else {
      // No constraints (shouldn't happen due to condition above)
      orig_w.max(orig_h)
    };
    
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, max_side);
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
  let img = load_image_with_orientation_from_path(Path::new(&path)).map_err(|e| {
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
    // Determine the maximum side constraint
    let max_side = if max_w > 0 && max_h > 0 {
      // Both constraints: use the smaller of the two to fit within both
      max_w.min(max_h)
    } else if max_w > 0 {
      // Only width constraint: use width as max_side
      max_w
    } else if max_h > 0 {
      // Only height constraint: use height as max_side
      max_h
    } else {
      // No constraints (shouldn't happen due to condition above)
      orig_w.max(orig_h)
    };
    
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, max_side);
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
  let bytes = general_purpose::STANDARD
    .decode(&base64)
    .map_err(|e| Error::new(Status::InvalidArg, format!("Failed to decode Base64: {e}")))?;

  let img = load_image_with_orientation(&bytes).map_err(map_image_error)?;

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
    // Determine the maximum side constraint
    let max_side = if max_w > 0 && max_h > 0 {
      // Both constraints: use the smaller of the two to fit within both
      max_w.min(max_h)
    } else if max_w > 0 {
      // Only width constraint: use width as max_side
      max_w
    } else if max_h > 0 {
      // Only height constraint: use height as max_side
      max_h
    } else {
      // No constraints (shouldn't happen due to condition above)
      orig_w.max(orig_h)
    };
    
    let (final_w, final_h) = calculate_target_size(orig_w, orig_h, max_side);
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

/// Convert an image to WebP format with optimized quality for web.
///
/// Uses the image crate's WebP encoder which provides good quality for web use.
fn encode_to_webp_optimized(img: &DynamicImage) -> Result<Vec<u8>, Error> {
  let mut out = Vec::new();
  {
    let mut cursor = Cursor::new(&mut out);
    img
      .write_to(&mut cursor, ImageFormat::WebP)
      .map_err(map_image_error)?;
  }
  Ok(out)
}

/// Statistics about the conversion process.
#[napi(object)]
pub struct ConversionStats {
  /// Number of files successfully converted
  pub converted: u32,
  /// Number of files skipped (already WebP or not an image)
  pub skipped: u32,
  /// Number of files that failed to convert
  pub errors: u32,
  /// List of error messages for failed conversions
  pub error_messages: Vec<String>,
}

/// Convert all images in a directory (and subdirectories) to WebP format.
///
/// This function recursively walks through the directory tree and converts
/// all supported image formats to WebP. Files that are already WebP are skipped.
/// Original files are preserved (not deleted).
///
/// **Input:** Directory path (String) - Path to the parent directory
///
/// **Returns:** Statistics about the conversion process
#[napi]
pub fn convert_images_to_webp_recursive(dir_path: String) -> napi::Result<ConversionStats> {
  let path = Path::new(&dir_path);

  if !path.exists() {
    return Err(Error::new(
      Status::InvalidArg,
      format!("Directory does not exist: {}", dir_path),
    ));
  }

  if !path.is_dir() {
    return Err(Error::new(
      Status::InvalidArg,
      format!("Path is not a directory: {}", dir_path),
    ));
  }

  let mut converted = 0u32;
  let mut skipped = 0u32;
  let mut errors = 0u32;
  let mut error_messages = Vec::new();

  // Walk through directory recursively
  for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
    let file_path = entry.path();

    // Skip if not a file
    if !file_path.is_file() {
      continue;
    }

    // Get file extension
    let extension = file_path
      .extension()
      .and_then(|ext| ext.to_str())
      .map(|s| s.to_lowercase());

    let Some(ext) = extension else {
      skipped += 1;
      continue;
    };

    // Skip if already WebP
    if ext == "webp" {
      skipped += 1;
      continue;
    }

    // Try to detect image format from extension
    // If format detection fails, try to open the file anyway
    let format_opt = ImageFormat::from_extension(&ext);
    let is_potential_image = format_opt.is_some()
      || matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "ico" | "tiff" | "tif");

    if !is_potential_image {
      skipped += 1;
      continue;
    }

    // Check if WebP file already exists
    let webp_path = file_path.with_extension("webp");
    if webp_path.exists() {
      skipped += 1;
      continue;
    }

    // Try to convert the image (this will fail if it's not a valid image)
    match load_image_with_orientation_from_path(file_path) {
      Ok(img) => {
        match encode_to_webp_optimized(&img) {
          Ok(webp_data) => {
            match fs::write(&webp_path, webp_data) {
              Ok(_) => {
                converted += 1;
              }
              Err(e) => {
                errors += 1;
                error_messages.push(format!(
                  "Failed to write WebP file '{}': {}",
                  webp_path.display(),
                  e
                ));
              }
            }
          }
          Err(e) => {
            errors += 1;
            error_messages.push(format!(
              "Failed to encode WebP for '{}': {}",
              file_path.display(),
              e
            ));
          }
        }
      }
      Err(e) => {
        errors += 1;
        error_messages.push(format!(
          "Failed to open image '{}': {}",
          file_path.display(),
          e
        ));
      }
    }
  }

  // Return statistics
  Ok(ConversionStats {
    converted,
    skipped,
    errors,
    error_messages,
  })
}
