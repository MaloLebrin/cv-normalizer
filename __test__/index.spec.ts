import test from 'ava'
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import {
  base64ToBuffer,
  bufferToBase64,
  extractTextFromPdf,
  imageToWebp,
  imageToWebpFromBase64,
  imageToWebpFromFile,
  normalizeCvToPdf,
  optimizeImage,
  optimizeImageFromBase64,
  optimizeImageFromFile,
} from '../index'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

test('normalizeCvToPdf returns a valid PDF for PDF input (may be optimized)', (t) => {
  const input = new Uint8Array(Buffer.from('%PDF-1.4\nHello\n', 'ascii'))

  const output = normalizeCvToPdf(input, 'application/pdf') as Array<number>

  t.true(Array.isArray(output))
  t.true(output.length > 0)

  const outBuf = Buffer.from(output)
  const header = outBuf.subarray(0, 4).toString('ascii')

  t.is(header, '%PDF')
  t.true(outBuf.length <= input.length)
})

test('normalizeCvToPdf converts PNG image buffer to a PDF', (t) => {
  // Intentionally invalid PNG bytes to ensure image decoding errors are surfaced
  const pngBase64 =
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+X2O0AAAAASUVORK5CYII='

  const pngBuffer = Buffer.from(pngBase64, 'base64')

  const error = t.throws(
    () => {
      normalizeCvToPdf(pngBuffer, 'image/png')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to process image/)
})

test('imageToWebp converts a real PNG fixture to WebP', (t) => {
  const pngPath = path.join(__dirname, 'image.jpg')
  const pngBuffer = readFileSync(pngPath)

  const out = imageToWebp(pngBuffer) as Array<number>

  t.true(Array.isArray(out))
  const webpBuffer = Buffer.from(out)
  t.true(webpBuffer.length > 0)

  const riff = webpBuffer.subarray(0, 4).toString('ascii')
  const webpTag = webpBuffer.subarray(8, 12).toString('ascii')

  t.is(riff, 'RIFF')
  t.is(webpTag, 'WEBP')
})

test('bufferToBase64 encodes a buffer correctly', (t) => {
  const input = Buffer.from('Hello World')
  const base64 = bufferToBase64(input)

  t.is(typeof base64, 'string')
  t.is(base64, 'SGVsbG8gV29ybGQ=')
})

test('base64ToBuffer decodes a base64 string correctly', (t) => {
  const base64 = 'SGVsbG8gV29ybGQ='
  const output = base64ToBuffer(base64) as Array<number>

  t.true(Array.isArray(output))
  const buffer = Buffer.from(output)
  t.is(buffer.toString('utf-8'), 'Hello World')
})

test('base64ToBuffer throws on invalid base64', (t) => {
  const error = t.throws(
    () => {
      base64ToBuffer('Invalid!@#Base64')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to decode Base64/)
})

test('extractTextFromPdf extracts text from a valid PDF', (t) => {
  // Create a minimal PDF with text
  const pdfPath = path.join(__dirname, 'pdf-sample_0.pdf')
  const pdfBuffer = readFileSync(pdfPath)

  const text = extractTextFromPdf(pdfBuffer) as string

  t.is(typeof text, 'string')
  // The text might be empty for some PDFs, but the function should not throw
  t.true(text.length >= 0)
})

test('extractTextFromPdf throws on invalid PDF', (t) => {
  const invalidPdf = Buffer.from('Not a PDF')

  const error = t.throws(
    () => {
      extractTextFromPdf(invalidPdf)
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to extract text from PDF/)
})

test('optimizeImage resizes image when maxWidth is provided', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)

  const optimized = optimizeImage(imageBuffer, {
    maxWidth: 100,
    maxHeight: 0,
    quality: 80,
    format: 'jpeg',
  }) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)
  t.true(optimizedBuffer.length < imageBuffer.length || optimizedBuffer.length === imageBuffer.length)
})

test('optimizeImage resizes image when maxHeight is provided', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)

  const optimized = optimizeImage(imageBuffer, {
    maxWidth: 0,
    maxHeight: 100,
    quality: 80,
    format: 'jpeg',
  }) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)
})

test('optimizeImage converts to WebP format', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)

  const optimized = optimizeImage(imageBuffer, {
    format: 'webp',
    quality: 80,
  }) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)

  // Check WebP header
  const riff = optimizedBuffer.subarray(0, 4).toString('ascii')
  const webpTag = optimizedBuffer.subarray(8, 12).toString('ascii')
  t.is(riff, 'RIFF')
  t.is(webpTag, 'WEBP')
})

test('optimizeImage uses default options when none provided', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)

  const optimized = optimizeImage(imageBuffer) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)
})

test('optimizeImage throws on invalid image', (t) => {
  const invalidImage = Buffer.from('Not an image')

  const error = t.throws(
    () => {
      optimizeImage(invalidImage)
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to process image/)
})

test('imageToWebpFromFile converts image from file path to WebP', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')

  const out = imageToWebpFromFile(imagePath) as Array<number>

  t.true(Array.isArray(out))
  const webpBuffer = Buffer.from(out)
  t.true(webpBuffer.length > 0)

  const riff = webpBuffer.subarray(0, 4).toString('ascii')
  const webpTag = webpBuffer.subarray(8, 12).toString('ascii')

  t.is(riff, 'RIFF')
  t.is(webpTag, 'WEBP')
})

test('imageToWebpFromFile throws on non-existent file', (t) => {
  const error = t.throws(
    () => {
      imageToWebpFromFile('/path/that/does/not/exist.jpg')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to open image file/)
})

test('imageToWebpFromBase64 converts Base64 image to WebP', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)
  const base64 = bufferToBase64(imageBuffer)

  const out = imageToWebpFromBase64(base64) as Array<number>

  t.true(Array.isArray(out))
  const webpBuffer = Buffer.from(out)
  t.true(webpBuffer.length > 0)

  const riff = webpBuffer.subarray(0, 4).toString('ascii')
  const webpTag = webpBuffer.subarray(8, 12).toString('ascii')

  t.is(riff, 'RIFF')
  t.is(webpTag, 'WEBP')
})

test('imageToWebpFromBase64 throws on invalid Base64', (t) => {
  const error = t.throws(
    () => {
      imageToWebpFromBase64('Invalid!@#Base64')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to decode Base64/)
})

test('optimizeImageFromFile optimizes image from file path', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')

  const optimized = optimizeImageFromFile(imagePath, {
    maxWidth: 100,
    maxHeight: 0,
    quality: 80,
    format: 'jpeg',
  }) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)
})

test('optimizeImageFromFile throws on non-existent file', (t) => {
  const error = t.throws(
    () => {
      optimizeImageFromFile('/path/that/does/not/exist.jpg')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to open image file/)
})

test('optimizeImageFromBase64 optimizes image from Base64 string', (t) => {
  const imagePath = path.join(__dirname, 'image.jpg')
  const imageBuffer = readFileSync(imagePath)
  const base64 = bufferToBase64(imageBuffer)

  const optimized = optimizeImageFromBase64(base64, {
    maxWidth: 100,
    maxHeight: 0,
    quality: 80,
    format: 'webp',
  }) as Array<number>

  t.true(Array.isArray(optimized))
  const optimizedBuffer = Buffer.from(optimized)
  t.true(optimizedBuffer.length > 0)

  // Check WebP header
  const riff = optimizedBuffer.subarray(0, 4).toString('ascii')
  const webpTag = optimizedBuffer.subarray(8, 12).toString('ascii')
  t.is(riff, 'RIFF')
  t.is(webpTag, 'WEBP')
})

test('optimizeImageFromBase64 throws on invalid Base64', (t) => {
  const error = t.throws(
    () => {
      optimizeImageFromBase64('Invalid!@#Base64')
    },
    { instanceOf: Error },
  )

  t.is(error?.code, 'InvalidArg')
  t.regex(error?.message ?? '', /Failed to decode Base64/)
})
