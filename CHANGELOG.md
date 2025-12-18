# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive documentation in English
- Detailed API reference with examples
- Performance benchmarks
- Troubleshooting guide

## [1.0.4] - 2025-12-17

### Added
- Modular code structure: separated utilities into individual files
  - `src/normalize.rs` - CV normalization logic
  - `src/pdf.rs` - PDF text extraction and optimization
  - `src/image.rs` - Image conversion and optimization
  - `src/base64.rs` - Base64 encoding/decoding
  - `src/utils.rs` - Shared utilities
- `extractTextFromPdf` - Extract text from PDF documents (2-5x faster than pdf-parse)
- `bufferToBase64` / `base64ToBuffer` - High-performance Base64 utilities
- `optimizeImage` - Image resizing and compression with configurable options
- Comprehensive test suite (13 tests covering all functions)

### Changed
- Improved error messages for better debugging
- Updated documentation with detailed API reference

### Fixed
- Fixed compilation errors after modular refactoring
- Fixed PDF extraction API usage (`extract_text_from_mem` instead of `extract_text_from_read`)
- Fixed missing trait imports (`GenericImageView`, `std::io::Write`)

## [1.0.3] - 2025-12-17

### Changed
- Updated repository URL in package.json for sigstore provenance verification
- Improved PDF optimization fallback behavior

## [1.0.2] - 2025-12-17

### Added
- PDF compression using Ghostscript (optional, falls back gracefully)
- Image downscaling to prevent oversized PDFs (max 2000px longest side)

### Changed
- Migrated CI/CD from yarn to pnpm
- Updated README with production considerations

## [1.0.1] - 2025-12-17

### Added
- Initial implementation of `normalizeCvToPdf`
- Support for PNG/JPEG to PDF conversion
- PDF header validation
- Basic image processing pipeline

### Changed
- Updated project description and metadata

## [1.0.0] - 2025-12-17

### Added
- Initial release
- Basic CV normalization functionality
- NAPI-RS setup with multi-platform support

[Unreleased]: https://github.com/MaloLebrin/cv-normalizer/compare/v1.0.4...HEAD
[1.0.4]: https://github.com/MaloLebrin/cv-normalizer/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/MaloLebrin/cv-normalizer/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/MaloLebrin/cv-normalizer/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/MaloLebrin/cv-normalizer/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/MaloLebrin/cv-normalizer/releases/tag/v1.0.0

