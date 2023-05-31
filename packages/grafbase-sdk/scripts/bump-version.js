const fs = require('fs')
const semver = require('semver')

const pkg = JSON.parse(fs.readFileSync('package.json'))
pkg.version = semver.inc(pkg.version, process.argv.slice(2)[0])
fs.writeFileSync('package.json', `${JSON.stringify(pkg, null, 2)}\n`)

const changelog = `## Breaking

TODO

## Features

TODO

## Fixes

TODO
`

fs.writeFileSync(`changelog/${pkg.version}.md`, changelog)

var fullChangelog = "# Changelog\n\n"

fs.readdirSync('changelog/').sort().reverse().forEach(file => {
  const stat = fs.statSync(`changelog/${file}`)
  const modified = stat.mtime.toDateString()
  const version = file.replace('.md', '')

  fullChangelog += `## [${version}] - ${modified}\n\n[CHANGELOG](changelog/${file})\n\n`
})

fs.writeFileSync('CHANGELOG.md', fullChangelog)