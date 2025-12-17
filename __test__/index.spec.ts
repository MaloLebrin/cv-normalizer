import test from 'ava'
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

import { imageToWebp, normalizeCvToPdf } from '../index'

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
  t.regex(error?.message ?? '', /Failed to process image for CV normalization/)
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
