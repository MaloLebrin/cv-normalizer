const { convertImagesToWebpRecursive } = require('../index.js')

async function main() {
  const [, , dirPath] = process.argv

  if (!dirPath) {
    console.error('Usage: node scripts/convert-to-webp.cjs <directoryPath>')
    console.error('Example: node scripts/convert-to-webp.cjs ./images')
    process.exit(1)
  }

  console.log(`Converting images to WebP in: ${dirPath}`)
  console.log('This may take a while depending on the number of images...\n')

  const startTime = Date.now()

  try {
    const stats = convertImagesToWebpRecursive(dirPath)

    const duration = ((Date.now() - startTime) / 1000).toFixed(2)

    console.log('\n✅ Conversion completed!')
    console.log(`📊 Statistics:`)
    console.log(`   - Converted: ${stats.converted} file(s)`)
    console.log(`   - Skipped: ${stats.skipped} file(s)`)
    console.log(`   - Errors: ${stats.errors} file(s)`)
    console.log(`   - Duration: ${duration}s`)

    if (stats.errors > 0 && stats.errorMessages.length > 0) {
      console.log('\n⚠️  Errors encountered:')
      stats.errorMessages.forEach((msg, idx) => {
        console.log(`   ${idx + 1}. ${msg}`)
      })
    }

    if (stats.converted > 0) {
      console.log(`\n✨ Successfully converted ${stats.converted} image(s) to WebP format!`)
    }
  } catch (err) {
    console.error('\n❌ Error during conversion:', err.message)
    if (err.code) {
      console.error(`   Error code: ${err.code}`)
    }
    process.exit(1)
  }
}

main().catch((err) => {
  console.error('Unexpected error:', err)
  process.exit(1)
})




