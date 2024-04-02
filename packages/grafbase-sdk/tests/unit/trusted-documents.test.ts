import { config, graph } from '../../src/index'
import { describe, expect, it, beforeEach } from '@jest/globals'
import { renderGraphQL } from '../utils'

const g = graph.Standalone()

describe('Trusted documents', () => {
  beforeEach(() => g.clear())

  it('renders when enabled', async () => {
    const cfg = config({
      graph: g,
      trustedDocuments: {
        enabled: true
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @trustedDocuments
      "
    `)
  })

  it('renders bypassHeader', async () => {
    const cfg = config({
      graph: g,
      trustedDocuments: {
        enabled: true,
        bypassHeader: {
          name: 'password',
          value: 'm00se'
        }
      }
    })

    expect(renderGraphQL(cfg)).toMatchInlineSnapshot(`
      "extend schema
        @trustedDocuments(bypassHeaderName: "password", byPassHeaderValue: "m00se")
      "
    `)
  })
})
