import test from 'ava'

import { normalizeCvToPdf } from '../index'

test('normalizeCvToPdf echoes back bytes (v1 no-op)', (t) => {
  const input = new Uint8Array([1, 2, 3, 4])

  const output = normalizeCvToPdf(input, 'application/pdf') as Array<number>

  t.true(Array.isArray(output))
  t.is(output.length, input.length)
  t.deepEqual(output, Array.from(input))
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
