const fs = require('node:fs')
const path = require('node:path')

const { normalizeCvToPdf } = require('../index.js')

function guessMime(filePath) {
  const ext = path.extname(filePath).toLowerCase()
  if (ext === '.png') return 'image/png'
  if (ext === '.jpg' || ext === '.jpeg') return 'image/jpeg'
  if (ext === '.pdf') return 'application/pdf'
  return 'application/octet-stream'
}

async function main() {
  const [, , inputPath, mimeArg, outputPathArg] = process.argv

  if (!inputPath) {
    console.error('Usage: node scripts/normalize-file.cjs <inputPath> [mimeType] [outputPath]')
    process.exit(1)
  }

  const input = fs.readFileSync(inputPath)
  const mime = mimeArg || guessMime(inputPath)

  console.log(`Input: ${inputPath}`)
  console.log(`Mime: ${mime}`)
  console.log(`Original size: ${input.length} bytes`)

  const outArray = normalizeCvToPdf(input, mime)
  const outBuffer = Buffer.from(outArray)

  const outputPath =
    outputPathArg ||
    (() => {
      const dir = path.dirname(inputPath)
      const base = path.basename(inputPath, path.extname(inputPath))
      return path.join(dir, `${base}.normalized.pdf`)
    })()

  fs.writeFileSync(outputPath, outBuffer)

  console.log(`Output written: ${outputPath}`)
  console.log(`Output size: ${outBuffer.length} bytes`)
  if (input.length > 0) {
    const ratio = 1 - outBuffer.length / input.length
    console.log(`Size change: ${(ratio * 100).toFixed(1)}%`)
  }
}

main().catch((err) => {
  console.error('Error during normalization:', err)
  process.exit(1)
})


