// Based on
// https://github.com/moonrepo/moon/blob/master/packages/cli/postinstall.js
// which is based on
// https://github.com/parcel-bundler/parcel-css/blob/master/cli/postinstall.js

const { existsSync, linkSync, copyFileSync } = require('fs')
const { dirname, join } = require('path')
const { getEnvironmentData } = require('./get-environment-data')

const SUPPORTED_TARGET_TRIPLES = [
  'x86_64-apple-darwin',
  'aarch64-apple-darwin',
  'x86_64-unknown-linux-musl',
  'aarch64-unknown-linux-musl',
  'x86_64-pc-windows-msvc',
]

const linkBinary = (binary, binaryPath) => {
  try {
    linkSync(binaryPath, join(__dirname, binary))
  } catch {
    copyFileSync(binaryPath, join(__dirname, binary))
  }
}

const postinstall = () => {
  const { targetTriple, binary } = getEnvironmentData('grafbase')

  if (!SUPPORTED_TARGET_TRIPLES.includes(targetTriple)) {
    console.error(`'${targetTriple}' is currently unsupported`)
    return
  }

  const packagePath = dirname(require.resolve(`@grafbase/cli-${targetTriple}/package.json`))
  const binaryPath = join(packagePath, binary)

  if (existsSync(binaryPath)) {
    try {
      linkBinary(binary, binaryPath)
    } catch {
      console.error('Could not link or copy "grafbase" binary')
      return
    }
  } else {
    console.error('Could not find "grafbase" binary')
    return
  }
}

postinstall()
