import { config, graph } from '../../src/index'
import { describe, expect, it } from '@jest/globals'
import { renderGraphQL } from '../utils'

describe('Config', () => {
  it('renders a configuration that uses extra configuration settings', () => {
    const cfg = config({
      graph: graph.Single(),
      experimental: {
        kv: true
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @experimental(kv: true)
      
      "
  `)
  })
})
