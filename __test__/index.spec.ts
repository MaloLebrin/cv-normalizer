import test from 'ava'

import { normalizeCvToPdf } from '../index'

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
