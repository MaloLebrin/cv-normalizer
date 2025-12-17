import test from 'ava'

import { normalizeCvToPdf } from '../index'

test('normalizeCvToPdf echoes back bytes (v1 no-op)', (t) => {
  const input = new Uint8Array([1, 2, 3, 4])

  const output = normalizeCvToPdf(input, 'application/pdf')

  t.true(output instanceof Uint8Array)
  t.is(output.length, input.length)
  t.deepEqual(Array.from(output), Array.from(input))
})
