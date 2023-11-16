/** The metadata for a key in KV. Data of any shape can be stored. */
export type KVMetadata = any

// https://developers.cloudflare.com/kv/api/list-keys/
/** The result of a [`list()`](https://grafbase.com/docs/edge-gateway/resolvers#list) call.  */
export type KVListResult = {
  keys: KVListKey[]
  /** If `list_complete` is false then you will receive a `cursor` value you can pass to `list()` to obtain the next set of results. */
  list_complete: boolean
  cursor?: string
}

export type KVListKey = {
  customKey: string
  /** Only returned if the key has an expiration set. */
  expiration?: number
  metadata?: KVMetadata
}

// source of truth: https://github.com/grafbase/api/blob/main/common/grafbase-sdk/src/api/kv/input.rs
export type KVGetOptions = {
  /**
   * The cacheTtl parameter must be an integer that is greater than or equal to 60, which is the default.
   * It defines the length of time in seconds that a KV result is cached in the global network location
   * that it is accessed from.
   */
  ttl?: number
  /**  Type of the value. */
  type?: 'text' | 'json' | 'arraybuffer' | 'stream'
}

export type KVSetOptions = {
  /** Value will expire at specified time. Seconds since epoch. */
  expires?: number
  /** Time to live of the value in seconds. */
  ttl?: number
  /** Arbitrary JSON to be associated with a key/value pair. */
  metadata?: KVMetadata
}

export type KVListOptions = {
  /** A string prefix you can use to filter all keys */
  prefix?: string
  /** Maximum number of keys returned. The default is 1,000, which is the maximum. It is unlikely that you will want to change this default but it is included for completeness. */
  limit?: number
  /** Used for paginating responses. */
  cursor?: string
}
