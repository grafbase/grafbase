const fs = require('fs')
const childProcess = require('node:child_process')
const pkg = JSON.parse(fs.readFileSync('package.json'))

childProcess.execSync(
  `git switch main && git pull && git tag sdk-${pkg.version} && git push --tags`
)
