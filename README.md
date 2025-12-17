# `@malolebrin/cv-normalizer`

Native module (Rust + NAPI-RS) to **normalize and compress CV files on the Node.js side**.

- **Images (`image/png`, `image/jpeg`, `image/jpg`, `image/pjpeg`)** → converted to a **single-page PDF**: decode, basic downscale, JPEG recompression, and embedding into a minimal PDF.  
- **PDF (`application/pdf`, `application/x-pdf`)** → **header validation (`%PDF-`)** and then **optional compression** using Ghostscript if available; falls back to the original bytes when optimization fails or is not beneficial.  
- **Other mime types** → currently **pass-through** (bytes returned unchanged).  
- **Standalone image transformation** → `imageToWebp` converts any supported image (PNG, JPEG, …) to **WebP** in memory (similar to the example on the [NAPI-RS homepage](https://napi.rs/)).

The module is designed to be called from a Node/Strapi backend when receiving CV files.

---

## Installation

Once published on npm:

```bash
pnpm add @malolebrin/cv-normalizer
```

For local development in this repo:

```bash
pnpm install
pnpm build
```

The build generates the native binary `cv-normalizer.*.node` and the JS binding file `index.js`.

---

## Node / TypeScript API

Generated signatures (`index.d.ts`):

```ts
export declare function normalizeCvToPdf(
  bytes: Uint8Array,
  mime: string,
): number[]

export declare function imageToWebp(
  bytes: Uint8Array,
): number[]
```

Typical usage from Node:

```ts
import { normalizeCvToPdf, imageToWebp } from '@malolebrin/cv-normalizer'

// buffer: Buffer or Uint8Array containing the CV/image
// mime: string ('image/png', 'image/jpeg', 'application/pdf', etc.)
const pdfArray = normalizeCvToPdf(buffer, mime)
const pdfBuffer = Buffer.from(pdfArray)

const webpArray = imageToWebp(buffer)
const webpBuffer = Buffer.from(webpArray)
```

### `normalizeCvToPdf` behavior

- **Images (`image/png`, `image/jpeg`, `image/jpg`, `image/pjpeg`)**  
  - Decoded via the Rust `image` crate.  
  - Downscaled if needed so that the longest side ≤ 2000 px.  
  - Re-encoded as JPEG (quality ≈ 80).  
  - Embedded into a **single-page PDF** that draws the image over the whole page.

- **PDF (`application/pdf`, `application/x-pdf`)**  
  - Verifies that the bytes start with `"%PDF-"`.  
  - If not → throws a NAPI error (`code: InvalidArg`).  
  - If yes → tries to optimize/compress the PDF using the `gs` (Ghostscript) CLI with `-dPDFSETTINGS=/screen`.  
    - If Ghostscript is not installed, the command fails, or the optimized file is not strictly smaller → returns the original bytes.  
    - If optimization succeeds and produces a smaller file → returns the optimized bytes.

- **Other mime types**  
  - Bytes are returned unchanged (no transformation).

> **Note:** To benefit from PDF compression in production, the `gs` (Ghostscript) binary must be installed and available in the runtime environment.

### `imageToWebp` behavior

- Accepts any image format supported by the `image` crate (PNG, JPEG, …).  
- Decodes the image from memory and re-encodes it as **WebP** using `ImageFormat::WebP`.  
- Returns a `number[]` that can be turned into a Node `Buffer`.

Example:

```ts
import { imageToWebp } from '@malolebrin/cv-normalizer'
import { readFileSync, writeFileSync } from 'node:fs'

const png = readFileSync('my-image.png')
const webpArray = imageToWebp(png)
const webpBuffer = Buffer.from(webpArray)

writeFileSync('my-image.webp', webpBuffer)
```

---

## CLI demo script

To try normalization on **real files** (images or PDFs), a small CLI script is provided:

```bash
pnpm build
pnpm demo /path/to/my_cv.png
pnpm demo /path/to/my_cv.pdf
```

This script (`scripts/normalize-file.cjs`):

- infers or uses the `mimeType` passed as a second argument,  
- calls `normalizeCvToPdf`,  
- writes an output file `<name>.normalized.pdf` next to the input file,  
- prints original and output sizes and the relative size change.

Detailed usage:

```bash
node scripts/normalize-file.cjs <inputPath> [mimeType] [outputPath]
```

Examples:

```bash
# PNG image → PDF
node scripts/normalize-file.cjs ./fixtures/cv.png

# JPEG image with explicit mime
node scripts/normalize-file.cjs ./fixtures/cv.jpg image/jpeg

# Existing PDF (validation + optional compression via Ghostscript)
node scripts/normalize-file.cjs ./fixtures/cv.pdf
```

---

## Development

### Prerequisites

- Recent **Rust** toolchain (edition 2021).  
- **Node.js** ≥ 18 (CI currently runs on Node 20/22/24).  
- **pnpm** as package manager.

### Main commands

```bash
# Install JS dependencies
pnpm install

# Build the native module (all targets configured in package.json)
pnpm build

# Run tests (AVA)
pnpm test

# Lint JS/TS
pnpm lint

# Format Rust / JS / TOML
pnpm format
```

The current tests cover, among other things:

- behavior on a small valid PDF input (remains a valid PDF, not larger than the original),  
- error mapping (`InvalidArg`) when decoding an intentionally invalid PNG (image error path),  
- `imageToWebp` converting a real PNG fixture into a valid WebP file (checking `RIFF` and `WEBP` tags).

---

## Typical integration (Strapi / Node backend)

In a Node/Strapi backend, a recommended pattern is:

```ts
import { normalizeCvToPdf } from '@malolebrin/cv-normalizer'

async function normalizeIncomingCv(file: { buffer: Buffer; mime: string }) {
  const out = normalizeCvToPdf(file.buffer, file.mime)
  const pdfBuffer = Buffer.from(out)

  return {
    ...file,
    buffer: pdfBuffer,
    size: pdfBuffer.length,
    mime: 'application/pdf',
  }
}
```

This helper can be called from a **controller** or **lifecycle hook** right before persisting the CV so that all stored CVs are already **normalized to PDF**.

If you also want to keep a **WebP thumbnail** version of the CV, you can additionally call `imageToWebp` on the original image buffer and store the result alongside the PDF.

---

## CI & Release

- **GitHub Actions CI**  
  - Build, lint and tests are run on a Node.js / OS matrix (Linux, macOS, Windows) using **pnpm** for JS dependencies and `cargo` for the Rust side.  
  - `.node` / `.wasm` artifacts are prebuilt for multiple platforms via `@napi-rs/cli`.

- **npm publishing**  
  - Publishing is handled by CI from git tags (`npm version` + `git push`).  
  - Make sure `NPM_TOKEN` is configured in the repo’s GitHub secrets.  
  - Do **not** run `npm publish` manually: the GitHub Actions pipeline is responsible for publishing precompiled packages.


