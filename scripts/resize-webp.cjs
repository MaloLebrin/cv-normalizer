const fs = require('node:fs')
const path = require('node:path')
const { optimizeImageFromFile } = require('../index.js')

function findImageFiles(dirPath, fileList = [], extensions = ['.webp']) {
  const files = fs.readdirSync(dirPath)

  for (const file of files) {
    const filePath = path.join(dirPath, file)
    const stat = fs.statSync(filePath)

    if (stat.isDirectory()) {
      findImageFiles(filePath, fileList, extensions)
    } else if (stat.isFile()) {
      const ext = path.extname(filePath).toLowerCase()
      if (extensions.includes(ext)) {
        fileList.push(filePath)
      }
    }
  }

  return fileList
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

function parseArguments() {
  const args = process.argv.slice(2)
  const options = {
    dirPath: null,
    maxWidth: 1920,
    maxHeight: 1080,
    quality: 85,
    replace: false,
    outputDir: null,
    responsive: false,
    breakpoints: null, // Array of widths, e.g. [320, 640, 1024, 1920]
    sourceExtensions: ['.webp'], // Extensions to process as source images
  }

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    switch (arg) {
      case '--max-width':
      case '-w':
        options.maxWidth = parseInt(args[++i], 10)
        break
      case '--max-height':
      case '-h':
        options.maxHeight = parseInt(args[++i], 10)
        break
      case '--quality':
      case '-q':
        options.quality = parseInt(args[++i], 10)
        break
      case '--replace':
      case '-r':
        options.replace = true
        break
      case '--output-dir':
      case '-o':
        options.outputDir = args[++i]
        break
      case '--responsive':
      case '--breakpoints':
        options.responsive = true
        // Check if next arg is a comma-separated list of breakpoints
        if (i + 1 < args.length && !args[i + 1].startsWith('-')) {
          const breakpointsStr = args[++i]
          options.breakpoints = breakpointsStr.split(',').map((b) => parseInt(b.trim(), 10)).filter((b) => !isNaN(b))
        }
        break
      case '--source-extensions':
        const extensionsStr = args[++i]
        options.sourceExtensions = extensionsStr.split(',').map((e) => e.trim().startsWith('.') ? e.trim() : '.' + e.trim())
        break
      case '--help':
        return { help: true, options: null }
      default:
        if (!options.dirPath && !arg.startsWith('-')) {
          options.dirPath = arg
        }
        break
    }
  }

  // Default breakpoints for responsive if not specified
  if (options.responsive && !options.breakpoints) {
    options.breakpoints = [320, 640, 1024, 1920]
  }

  // For responsive mode, use output directory by default if not specified
  // Will be resolved relative to the source directory in main()
  if (options.responsive && !options.outputDir) {
    options.outputDir = 'responsive'
  }

  return { help: false, options }
}

function printHelp() {
  console.log(`
Usage: node scripts/resize-webp.cjs <directoryPath> [options]

Options:
  -w, --max-width <number>        Maximum width in pixels (default: 1920)
  -h, --max-height <number>       Maximum height in pixels (default: 1080)
  -q, --quality <number>          WebP quality 1-100 (default: 85)
  -r, --replace                   Replace original files (default: creates .optimized.webp files)
  -o, --output-dir <path>         Output directory (keeps original structure)
  --responsive, --breakpoints     Generate multiple sizes for responsive images
                                  Optionally specify breakpoints: --breakpoints 320,640,1024,1920
                                  Default breakpoints: 320, 640, 1024, 1920
  --source-extensions <exts>      Comma-separated list of source image extensions (default: .webp)
                                  Example: --source-extensions .jpg,.jpeg,.png,.webp
  --help                          Show this help message

Examples:
  # Resize all WebP images to max 1920x1080, create .optimized.webp files
  node scripts/resize-webp.cjs ./images

  # Generate responsive images (320w, 640w, 1024w, 1920w) from originals
  # Output goes to ./responsive directory (default), originals are NOT copied
  node scripts/resize-webp.cjs ./images --responsive

  # Generate responsive images with custom breakpoints and output directory
  node scripts/resize-webp.cjs ./images --breakpoints 400,800,1200,1600 --output-dir ./web-images

  # Generate responsive from JPG/PNG sources (converts to WebP)
  # Output goes to ./responsive directory, only WebP files are created
  node scripts/resize-webp.cjs ./images --responsive --source-extensions .jpg,.jpeg,.png

  # Resize to max 1280x720 and replace original files
  node scripts/resize-webp.cjs ./images --max-width 1280 --max-height 720 --replace

  # Resize with custom quality and save to output directory
  node scripts/resize-webp.cjs ./images --quality 90 --output-dir ./optimized
`)
}

function generateOutputPath(filePath, options, width = null) {
  const ext = path.extname(filePath)
  const dir = path.dirname(filePath)
  const baseName = path.basename(filePath, ext)

  if (options.outputDir) {
    // Calculate relative path from input dir
    const relativePath = path.relative(options.dirPath, filePath)
    const relativeDir = path.dirname(relativePath)
    const outputBase = path.basename(relativePath, ext)
    let outputPath
    
    // For responsive mode, put all images in the output dir (flat or with structure)
    if (options.responsive && width) {
      // For responsive, create files directly in output dir or maintain structure
      if (relativeDir === '.') {
        // File is at root of input dir
        outputPath = path.join(options.outputDir, `${outputBase}-${width}w.webp`)
      } else {
        // Maintain subdirectory structure
        outputPath = path.join(options.outputDir, relativeDir, `${outputBase}-${width}w.webp`)
      }
    } else {
      // Non-responsive mode
      outputPath = path.join(options.outputDir, relativeDir, outputBase)
      if (width) {
        outputPath += `-${width}w.webp`
      } else {
        outputPath += ext === '.webp' ? '.webp' : '.webp'
      }
    }
    // Ensure output directory exists
    fs.mkdirSync(path.dirname(outputPath), { recursive: true })
    return outputPath
  } else if (options.replace && !width) {
    return filePath
  } else {
    // Create file with width suffix for responsive, or .optimized.webp for single resize
    if (width) {
      return path.join(dir, `${baseName}-${width}w.webp`)
    } else {
      return path.join(dir, `${baseName}.optimized.webp`)
    }
  }
}

async function processImage(filePath, options, stats) {
  try {
    const originalStats = fs.statSync(filePath)
    const originalSize = originalStats.size

    const results = []

    if (options.responsive && options.breakpoints) {
      // Generate multiple sizes for responsive images
      for (const width of options.breakpoints) {
        try {
          const optimizedBuffer = Buffer.from(
            optimizeImageFromFile(filePath, {
              maxWidth: width,
              maxHeight: 0, // No height limit for responsive
              quality: options.quality,
              format: 'webp',
            }),
          )

          const outputPath = generateOutputPath(filePath, options, width)
          fs.writeFileSync(outputPath, optimizedBuffer)

          const optimizedSize = optimizedBuffer.length
          const sizeReduction = originalSize > 0 ? ((originalSize - optimizedSize) / originalSize) * 100 : 0

          stats.processed++
          stats.totalOriginalSize += originalSize
          stats.totalOptimizedSize += optimizedSize

          if (sizeReduction > 0) {
            stats.totalSaved += originalSize - optimizedSize
          }

          results.push({
            success: true,
            width,
            output: outputPath,
            optimizedSize,
            sizeReduction,
          })
        } catch (err) {
          stats.errors++
          stats.errorMessages.push(`${filePath} (${width}w): ${err.message}`)
          results.push({
            success: false,
            width,
            error: err.message,
          })
        }
      }
    } else {
      // Single resize
      const optimizedBuffer = Buffer.from(
        optimizeImageFromFile(filePath, {
          maxWidth: options.maxWidth || undefined,
          maxHeight: options.maxHeight || undefined,
          quality: options.quality,
          format: 'webp',
        }),
      )

      const optimizedSize = optimizedBuffer.length
      const outputPath = generateOutputPath(filePath, options)

      fs.writeFileSync(outputPath, optimizedBuffer)

      const sizeReduction = originalSize > 0 ? ((originalSize - optimizedSize) / originalSize) * 100 : 0

      stats.processed++
      stats.totalOriginalSize += originalSize
      stats.totalOptimizedSize += optimizedSize

      if (sizeReduction > 0) {
        stats.totalSaved += originalSize - optimizedSize
      }

      results.push({
        success: true,
        output: outputPath,
        originalSize,
        optimizedSize,
        sizeReduction,
      })
    }

    return {
      success: true,
      file: filePath,
      originalSize,
      results,
    }
  } catch (err) {
    stats.errors++
    stats.errorMessages.push(`${filePath}: ${err.message}`)
    return {
      success: false,
      file: filePath,
      error: err.message,
    }
  }
}

async function main() {
  const { help, options } = parseArguments()

  if (help || !options || !options.dirPath) {
    printHelp()
    process.exit(options && options.dirPath ? 0 : 1)
  }

  const dirPath = path.resolve(options.dirPath)

  if (!fs.existsSync(dirPath)) {
    console.error(`❌ Error: Directory does not exist: ${dirPath}`)
    process.exit(1)
  }

  if (!fs.statSync(dirPath).isDirectory()) {
    console.error(`❌ Error: Path is not a directory: ${dirPath}`)
    process.exit(1)
  }

  // Create output directory if specified
  if (options.outputDir) {
    // If outputDir is relative, make it relative to the source directory
    if (!path.isAbsolute(options.outputDir)) {
      options.outputDir = path.join(dirPath, options.outputDir)
    } else {
      options.outputDir = path.resolve(options.outputDir)
    }
    fs.mkdirSync(options.outputDir, { recursive: true })
  }

  console.log(`🖼️  Resizing WebP images in: ${dirPath}`)
  if (options.responsive) {
    console.log(`   Responsive breakpoints: ${options.breakpoints.join('w, ')}w`)
  } else {
    console.log(`   Max dimensions: ${options.maxWidth || 'unlimited'}x${options.maxHeight || 'unlimited'}`)
  }
  console.log(`   Quality: ${options.quality}`)
  if (options.responsive) {
    console.log(`   Mode: Generate responsive sizes (${options.breakpoints.length} sizes per image)`)
    console.log(`   Output directory: ${options.outputDir}`)
    console.log(`   ⚠️  Original images will NOT be copied to output directory`)
  } else {
    console.log(`   Mode: ${options.replace ? 'Replace originals' : options.outputDir ? `Output to: ${options.outputDir}` : 'Create .optimized.webp files'}`)
  }
  console.log('   This may take a while...\n')

  const startTime = Date.now()
  const stats = {
    processed: 0,
    errors: 0,
    errorMessages: [],
    totalOriginalSize: 0,
    totalOptimizedSize: 0,
    totalSaved: 0,
  }

  // Find all image files recursively
  const imageFiles = findImageFiles(dirPath, [], options.sourceExtensions)

  if (imageFiles.length === 0) {
    const extList = options.sourceExtensions.join(', ')
    console.log(`⚠️  No image files found with extensions: ${extList}`)
    process.exit(0)
  }

  if (options.responsive) {
    console.log(`📋 Found ${imageFiles.length} source image(s) to process`)
    console.log(`   Generating responsive sizes: ${options.breakpoints.join('w, ')}w\n`)
  } else {
    console.log(`📋 Found ${imageFiles.length} image file(s) to process\n`)
  }

  // Process each file
  for (const filePath of imageFiles) {
    const result = await processImage(filePath, options, stats)
    if (result.success) {
      if (options.responsive && result.results) {
        // Show all generated sizes
        const sizes = result.results
          .filter((r) => r.success)
          .map((r) => `${r.width}w`)
          .join(', ')
        console.log(`✅ ${path.relative(dirPath, result.file)} → ${sizes}`)
      } else {
        const singleResult = result.results[0]
        const reduction = singleResult.sizeReduction > 0 ? ` (${singleResult.sizeReduction.toFixed(1)}% smaller)` : ''
        console.log(`✅ ${path.relative(dirPath, result.file)}${reduction}`)
      }
    } else {
      console.log(`❌ ${path.relative(dirPath, result.file)}: ${result.error}`)
    }
  }

  const duration = ((Date.now() - startTime) / 1000).toFixed(2)

  console.log('\n' + '='.repeat(60))
  console.log('✅ Processing completed!')
  console.log(`📊 Statistics:`)
  console.log(`   - Processed: ${stats.processed} file(s)`)
  console.log(`   - Errors: ${stats.errors} file(s)`)
  console.log(`   - Original total size: ${formatBytes(stats.totalOriginalSize)}`)
  console.log(`   - Optimized total size: ${formatBytes(stats.totalOptimizedSize)}`)
  if (stats.totalSaved > 0) {
    const totalReduction = ((stats.totalSaved / stats.totalOriginalSize) * 100).toFixed(1)
    console.log(`   - Total saved: ${formatBytes(stats.totalSaved)} (${totalReduction}% reduction)`)
  }
  console.log(`   - Duration: ${duration}s`)

  if (stats.errors > 0 && stats.errorMessages.length > 0) {
    console.log('\n⚠️  Errors encountered:')
    stats.errorMessages.slice(0, 10).forEach((msg, idx) => {
      console.log(`   ${idx + 1}. ${msg}`)
    })
    if (stats.errorMessages.length > 10) {
      console.log(`   ... and ${stats.errorMessages.length - 10} more errors`)
    }
  }
}

main().catch((err) => {
  console.error('\n❌ Unexpected error:', err)
  process.exit(1)
})

