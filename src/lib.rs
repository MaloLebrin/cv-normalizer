#![deny(clippy::all)]

use std::io::{Cursor, Write as IoWrite};

use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::{ColorType, DynamicImage, GenericImageView};
use napi::{Error, Status};
use napi_derive::napi;

/// Normalize a CV file to PDF and optionally compress it.
///
/// V1 behavior:
/// - If the mime type is a supported image (`image/png`, `image/jpeg`, `image/jpg`),
///   the image is decoded, optionally downscaled, recompressed as JPEG,
///   and wrapped into a single-page PDF.
/// - For any other mime type, the input bytes are returned unchanged.
#[napi]
pub fn normalize_cv_to_pdf(bytes: Vec<u8>, mime: String) -> napi::Result<Vec<u8>> {
  let mime_lc = mime.to_ascii_lowercase();

  // Only handle images for now. Everything else is returned as-is.
  if !(mime_lc == "image/png"
    || mime_lc == "image/jpeg"
    || mime_lc == "image/jpg"
    || mime_lc == "image/pjpeg")
  {
    return Ok(bytes);
  }

  let img = image::load_from_memory(&bytes).map_err(map_image_error)?;

  // Basic downscaling to avoid huge PDFs: keep longest side <= 2000px
  let max_side: u32 = 2000;
  let (orig_w, orig_h) = img.dimensions();
  let (target_w, target_h) = calculate_target_size(orig_w, orig_h, max_side);

  let resized = if (orig_w, orig_h) == (target_w, target_h) {
    img
  } else {
    img.resize_exact(target_w, target_h, FilterType::Lanczos3)
  };

  let jpeg_bytes = encode_to_jpeg(resized, 80).map_err(map_image_error)?;
  let pdf_bytes = jpeg_to_single_page_pdf(&jpeg_bytes, target_w, target_h);

  Ok(pdf_bytes)
}

fn map_image_error(err: image::ImageError) -> Error {
  Error::new(
    Status::InvalidArg,
    format!("Failed to process image for CV normalization: {err}"),
  )
}

fn calculate_target_size(orig_w: u32, orig_h: u32, max_side: u32) -> (u32, u32) {
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

fn encode_to_jpeg(img: DynamicImage, quality: u8) -> Result<Vec<u8>, image::ImageError> {
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

/// Build a minimal single-page PDF embedding the given JPEG bytes.
///
/// We embed the JPEG as an image XObject with /Filter /DCTDecode and draw it
/// to fill the page. Dimensions are in "points" but we simply reuse the
/// pixel dimensions, which is acceptable for CV images.
fn jpeg_to_single_page_pdf(jpeg_bytes: &[u8], width: u32, height: u32) -> Vec<u8> {
  use std::fmt::Write as FmtWrite;

  let mut pdf = Vec::new();
  let mut xref_positions: Vec<usize> = Vec::new();

  // Helper to start an object and record its offset for the xref table.
  let start_obj = |pdf: &mut Vec<u8>, xref: &mut Vec<usize>, id: u32| {
    xref.push(pdf.len());
    let _ = write!(pdf, "{} 0 obj\n", id);
  };

  pdf.extend_from_slice(b"%PDF-1.4\n");

  // 1: Catalog
  start_obj(&mut pdf, &mut xref_positions, 1);
  pdf.extend_from_slice(b"<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

  // 2: Pages
  start_obj(&mut pdf, &mut xref_positions, 2);
  pdf.extend_from_slice(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

  // 3: Page
  start_obj(&mut pdf, &mut xref_positions, 3);
  let _ = write!(
    pdf,
    concat!(
      "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] ",
      "/Resources << /XObject << /Im0 4 0 R >> /ProcSet [/PDF /ImageC] >> ",
      "/Contents 5 0 R >>\nendobj\n"
    ),
    w = width,
    h = height,
  );

  // 4: Image XObject
  start_obj(&mut pdf, &mut xref_positions, 4);
  let _ = write!(
    pdf,
    concat!(
      "<< /Type /XObject /Subtype /Image /Name /Im0 ",
      "/Width {w} /Height {h} /ColorSpace /DeviceRGB /BitsPerComponent 8 ",
      "/Filter /DCTDecode /Length {len} >>\nstream\n"
    ),
    w = width,
    h = height,
    len = jpeg_bytes.len(),
  );
  pdf.extend_from_slice(jpeg_bytes);
  pdf.extend_from_slice(b"\nendstream\nendobj\n");

  // 5: Content stream drawing the image to fill the page
  start_obj(&mut pdf, &mut xref_positions, 5);
  let mut content = String::new();
  let _ = FmtWrite::write_str(&mut content, "q\n");
  let _ = FmtWrite::write_str(
    &mut content,
    &format!("{} 0 0 {} 0 0 cm\n", width, height),
  );
  let _ = FmtWrite::write_str(&mut content, "/Im0 Do\nQ\n");

  let _ = write!(
    pdf,
    "<< /Length {} >>\nstream\n{}endstream\nendobj\n",
    content.as_bytes().len(),
    content
  );

  // XRef table
  let xref_start = pdf.len();
  let total_objects = xref_positions.len() as u32;

  let _ = write!(pdf, "xref\n0 {}\n", total_objects + 1);
  pdf.extend_from_slice(b"0000000000 65535 f \n");

  for offset in &xref_positions {
    let _ = write!(pdf, "{:010} 00000 n \n", offset);
  }

  let _ = write!(
    pdf,
    "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
    total_objects + 1,
    xref_start
  );

  pdf
}
