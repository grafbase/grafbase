import type { NextConfig } from 'next'

import { spawn } from 'child_process'

let devServerStarted = false

const runGrafbase = () => {
  spawn('grafbase', ['dev'], {
    stdio: 'inherit'
  })
}

const createGrafbasePlugin =
  () =>
  (nextConfig: Partial<NextConfig> = {}): Partial<NextConfig> => {
    // could be either `next dev` or just `next`
    const isNextDev =
      process.argv.includes('dev') ||
      process.argv.some(
        (_) => _.endsWith('bin/next') || _.endsWith('bin\\next')
      )

    return {
      ...nextConfig,
      // Since Next.js doesn't provide some kind of real "plugin system" we're (ab)using the `redirects` option here
      // in order to hook into and block the `next build` and initial `next dev` run.
      redirects: async () => {
        // TODO: Check if process.env.GRAFBASE_API_URL is localhost and boot up...

        if (isNextDev && !devServerStarted) {
          devServerStarted = true
          runGrafbase()
        }

        return nextConfig.redirects?.() ?? []
      }
    }
  }

export const withGrafbase = createGrafbasePlugin()
