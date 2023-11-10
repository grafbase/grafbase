const VENDORS = {
  PC: 'pc',
  APPLE: 'apple',
  UNKNOWN: 'unknown',
}

const OPERATING_SYSTEMS = {
  WINDOWS: 'windows',
  DARWIN: 'darwin',
  LINUX: 'linux',
}

const ARCHITECTURES = {
  X86_64: 'x86_64',
  AARCH64: 'aarch64',
}

const ENVIRONMENTS = {
  MUSL: 'musl',
  MSVC: 'msvc',
}

const EXTENSIONS = {
  EXE: 'exe',
}

const OPERATING_SYSTEM_TRANSLATIONS = {
  win32: OPERATING_SYSTEMS.WINDOWS,
  darwin: OPERATING_SYSTEMS.DARWIN,
  linux: OPERATING_SYSTEMS.LINUX,
}

const VENDOR_TRANSLATIONS = {
  win32: VENDORS.PC,
  darwin: VENDORS.APPLE,
  linux: VENDORS.UNKNOWN,
}

const ARCHITECTURE_TRANSLATIONS = {
  x64: ARCHITECTURES.X86_64,
  arm64: ARCHITECTURES.AARCH64,
}

const ENVIRONMENT_TRANSLATIONS = {
  [OPERATING_SYSTEMS.LINUX]: ENVIRONMENTS.MUSL,
  [OPERATING_SYSTEMS.WINDOWS]: ENVIRONMENTS.MSVC,
}

const EXTENSION_TRANSLATIONS = {
  [OPERATING_SYSTEMS.WINDOWS]: EXTENSIONS.EXE,
}

const getEnvironmentData = (binaryName) => {
  const operatingSystem = OPERATING_SYSTEM_TRANSLATIONS[process.platform] ?? process.platform
  const vendor = VENDOR_TRANSLATIONS[process.platform]
  const arch = ARCHITECTURE_TRANSLATIONS[process.arch] ?? process.arch
  const environment = ENVIRONMENT_TRANSLATIONS[operatingSystem]

  const targetTripleParts = [arch, vendor, operatingSystem, environment].filter((value) => value !== undefined)

  const binaryExtension = EXTENSION_TRANSLATIONS[operatingSystem]

  const binaryWithExtension = binaryExtension ? `${binaryName}.${binaryExtension}` : binaryName

  return { targetTriple: targetTripleParts.join('-'), binary: binaryWithExtension }
}

module.exports = { getEnvironmentData }
