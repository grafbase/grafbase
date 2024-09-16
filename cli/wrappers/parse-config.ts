import { writeFileSync } from 'fs'
import path from 'path'

const configFile = process.argv.slice(2)[0]
const resultFile = process.argv.slice(2)[1]

process.env['GRAFBASE_PROJECT_GRAFBASE_DIR'] = path.dirname(configFile)

let config
try {
  config = require(configFile)
} catch (error) {
  console.error(error)
  process.exit(1)
}

if (config.default == null) {
  console.error('Default configuration not defined in grafbase.config.ts. Did you remember to export it?')
  process.exit(2)
}

writeFileSync(resultFile, config.default.toString(), {
  flag: 'w',
})
