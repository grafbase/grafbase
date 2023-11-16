import {
  KVListResult,
  KVMetadata,
  KVGetOptions,
  KVSetOptions,
  KVListOptions
} from './context/kv'
import { Classification } from './context/ai'

/**
 * The type of the `context` argument in a Grafbase edge resolver.
 *
 * Reference: https://grafbase.com/docs/edge-gateway/resolvers#context
 *
 * @example
 *
 * import { Context, Info } from '@grafbase/sdk'
 *
 * export default async function(_parent, _args, context: Context) {
 *   // ...
 * }
 */
export type ResolverContext = {
  /** Context about the HTTP request being handled. */
  request: {
    headers: Record<string, any>
  }
  /**
   * Grafbase KV
   *
   * If you want to use this, please make sure to [enable KV in your Grafbase configuration](https://grafbase.com/docs/edge-gateway/resolvers#enable-kv).
   *
   * See the reference documentation: https://grafbase.com/docs/edge-gateway/resolvers#kv
   */
  kv: {
    /** Retrieve the value and metadata for a key. See [the docs](https://grafbase.com/docs/edge-gateway/resolvers#get) for examples. */
    get: (
      key: string,
      options?: KVGetOptions
    ) => Promise<{ metadata: KVMetadata; value: any }>
    /** Create a new key-value pair or update an existing one. See [the docs](https://grafbase.com/docs/edge-gateway/resolvers#set) for examples. */
    set: (key: string, value: any, options?: KVSetOptions) => Promise<void>
    /** Delete a key and its value. */
    delete: (key: string) => Promise<void>
    /** Fetch the list of all keys. See [the docs](https://grafbase.com/docs/edge-gateway/resolvers#list) for examples. */
    list: (options?: KVListOptions) => Promise<KVListResult>
  }
  ai: {
    textLlm: (args: {
      model?: string // closed set of possibilities? see common/grafbase-sdk/src/api/ai/models.rs in grafbase/api
      prompt?: string
      messages?: { role: 'system' | 'user'; content: string }[]
    }) => Promise<{ response: string }>
    textClassification: (args: {
      model?: string
      text: string
    }) => Promise<{ response: [Classification, Classification] }>
    textTranslation: (args: {
      model?: string
      text: string
      /** Defaults to English. */
      from?: string
      to: string
    }) => Promise<{ translated_text: string }>
    // Reference for the return type: https://developers.cloudflare.com/workers-ai/models/embedding/
    textEmbeddings: (args: {
      model?: string
      text: string | string[]
    }) => Promise<{ shape: number[]; data: number[][] }>
    imageClassification: (args: {
      model?: string
      /** The string should be a base64 encoded binary string */
      image: string | number[]
    }) => Promise<{ label: string; score: number }[]>
    speechToText: (args: {
      model?: string
      audio: string | number[]
    }) => Promise<{ text: string }>
  }
}
