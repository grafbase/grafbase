// this file defines the client plugins that is injected into the
// user's application
import type { GraphQLObject, ClientPlugin } from 'houdini'
import ReconnectingEventSource from 'reconnecting-eventsource'
import { apply_patch } from 'jsonpatch'

// houdini client by the grafbase plugin.
const plugin: ClientPlugin = () => {
  let unsubscribe: (() => void) | null = null

  return {
    // if the artifact is "live" then we need to mark the policy as CacheAndNetwork
    // so that we always make it to the network phase to start listening.
    start(ctx, { next }) {
      if (ctx.artifact.pluginData?.['@grafbase/houdini']?.live) {
        ctx.policy = 'CacheAndNetwork'
      }

      next(ctx)
    },

    // when we get to the network phase, we only care if we are dealing with a live
    // document
    network(ctx, { client, next, resolve, marshalVariables }) {
      // only process live documents on the browser
      if (
        !ctx.artifact.pluginData?.['@grafbase/houdini']?.live ||
        typeof globalThis.window === 'undefined'
      ) {
        return next(ctx)
      }

      // we are sending the network request for a live document

      // if we got this far then we have to assume that a fetch
      // means that we want fresh data from the server. that means
      // that we should always unsubscribe and create a new one
      unsubscribe?.()

      const headers = Object.entries(ctx.fetchParams?.headers ?? {}).reduce<
        Record<string, string>
      >((headers, [key, value]) => ({ ...headers, [key]: value }), {})

      const searchParams = new URLSearchParams({
        ...headers,
        query: ctx.text,
        variables: JSON.stringify(marshalVariables(ctx))
      })

      // construct the url we will use to send the request
      const url = new URL(client.url)
      url.search = searchParams.toString()

      // subscribe to the url
      unsubscribe = subscribe({
        url,
        onMessage({ data, errors }) {
          resolve(ctx, {
            fetching: false,
            data,
            errors,
            partial: false,
            stale: false,
            variables: ctx.variables ?? null,
            source: 'network'
          })
        },
        onError({ data, errors }) {
          resolve(ctx, {
            partial: true,
            stale: false,
            source: 'network',
            data,
            errors,
            fetching: false,
            variables: ctx.variables ?? null
          })
        }
      })
    },

    cleanup() {
      unsubscribe?.()
    }
  }
}

function subscribe({
  url,
  onMessage,
  onError
}: {
  url: URL
  onMessage: (data: {
    data: GraphQLObject | null
    errors: { message: string }[]
  }) => void
  onError: (data: {
    data: GraphQLObject | null
    errors: { message: string }[]
  }) => void
}): () => void {
  // we want to hold onto the last value we get from the request
  // so we can apply a patch
  let lastValue: GraphQLObject = {}

  // instantiate the event source
  const source = new ReconnectingEventSource(url)

  source.onmessage = (event) => {
    // the event payload is a string, turn it into an object
    const payload = JSON.parse(event.data)

    // the value to set
    let value: GraphQLObject

    // if we have a patch, apply it
    if (payload.patch) {
      value = apply_patch(lastValue, payload.patch)
    }
    // we could have errors
    else if (payload.errors) {
      source.close()
      onError(payload)
      return
    }
    // or the payload contains the full data
    else {
      value = payload.data
    }

    // save this value
    lastValue = value

    // push the value back to user
    onMessage({ data: value, errors: payload.errors ?? null })

    // cleanup
    if (source.readyState === 2) {
      onError({ data: null, errors: [{ message: 'connection closed' }] })
      source.close()
    }
  }

  return () => source.close()
}

export default () => plugin
