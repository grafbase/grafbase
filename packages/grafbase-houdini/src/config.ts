import type { ConfigFile } from 'houdini'

/** Configure the default set of scalars supported by Grafbase */
export default function config(config: ConfigFile): ConfigFile {
  return {
    ...config,
    scalars: {
      ...config.scalars,
      Date: {
        type: 'Date',
        marshal(val: Date) {
          return val.toISOString()
        },
        unmarshal(val: string) {
          return new Date(val)
        },
        ...config.scalars?.Date
      },
      DateTime: {
        type: 'Date',
        marshal(val: Date) {
          return val.toISOString()
        },
        unmarshal(val: string) {
          return new Date(val)
        },
        ...config.scalars?.DateTime
      },
      Email: {
        type: 'string',
        ...config.scalars?.Email
      },
      IPAddress: {
        type: 'string',
        ...config.scalars?.IPAddress
      },
      Timestamp: {
        type: 'number',
        ...config.scalars?.Timestamp
      },
      PhoneNumber: {
        type: 'string',
        ...config.scalars?.PhoneNumber
      },
      JSON: {
        type: 'Record<string, any>',
        ...config.scalars?.JSON
      }
    }
  }
}
