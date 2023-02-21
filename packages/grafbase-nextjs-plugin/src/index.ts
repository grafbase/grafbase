// Thanks to Contentlayer for the experimental hack
// inspiration with "plugins" hook to test this idea

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
    const isNextDev =
      process.argv.includes('dev') ||
      process.argv.some(
        (_) => _.endsWith('bin/next') || _.endsWith('bin\\next')
      )

    const GRAFBASE_API_URL =
      process.env['GRAFBASE_API_URL'] ||
      process.env['NEXT_PUBLIC_GRAFBASE_API_URL']
    const hasLocalApiUrl = /^http:\/\/(localhost|127\.0\.0\.1)/.test(
      GRAFBASE_API_URL!
    )

    return {
      ...nextConfig,
      redirects: async () => {
        if (hasLocalApiUrl && isNextDev && !devServerStarted) {
          devServerStarted = true
          runGrafbase()
        }

        return nextConfig.redirects?.() ?? []
      }
    }
  }

/**
 * Grafbase CLI default runner.
 * This will run the Grafbase CLI when the API url starts with http://localhost
 *
 * @example
 * ```js
 * // next.config.mjs
 * import { withGrafbase } from '@grafbase/nextjs-plugin'
 *
 * export default withGrafbase({
 *   // Next.js config
 * })
 * ```
 */
export const withGrafbase = createGrafbasePlugin()
