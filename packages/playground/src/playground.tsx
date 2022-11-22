import './style.scss'

import type { Fetcher } from '@graphiql/toolkit'
import { GraphiQL, GraphiQLInterface, GraphiQLInterfaceProps, GraphiQLProvider, GraphiQLProviderProps } from 'graphiql'
import type { GraphQLSchema } from 'graphql'
import { useCallback, useEffect, useState } from 'react'
import { GrafbaseLogo } from './components/grafbase-logo'
import { Toolbar } from './components/toolbar'
import { getDefaultQuery } from './utils/defaultTabs'
import { fetcher, getSchema } from './utils/fetcher'
import { renameTabs } from './utils/renameTabs'
import { isLiveQuery, SSEProvider, useSSEContext } from './utils/sse'
import { getStorage } from './utils/storage'

type BaseProps = {
  logo?: React.ReactNode
}

type InterfaceProps = GraphiQLInterfaceProps & BaseProps

const Interface = (props: InterfaceProps) => {
  const { logo, ...rest } = props

  return (
    <GraphiQLInterface {...rest}>
      <GraphiQL.Logo>{logo ?? <GrafbaseLogo />}</GraphiQL.Logo>
      <GraphiQL.Toolbar>
        <Toolbar />
      </GraphiQL.Toolbar>
    </GraphiQLInterface>
  )
}

type ProviderProps = GraphiQLProviderProps & BaseProps

const Provider = (props: ProviderProps) => {
  const { logo, ...rest } = props
  return (
    <GraphiQLProvider
      onTabChange={(tabsState) => {
        const tabNames = tabsState.tabs.map((tab) => tab.title)
        setTimeout(() => renameTabs(tabNames), 0)
        props.onTabChange?.(tabsState)
      }}
      {...rest}
    >
      <Interface isHeadersEditorEnabled={false} logo={logo} />
    </GraphiQLProvider>
  )
}

type PlaygroundProps = GraphiQLProviderProps &
  BaseProps & {
    storageKey?: string
    endpoint?: string
    headers?: Record<string, string> | undefined
  }

const Playground = (props: PlaygroundProps) => {
  const { storageKey = 'grafbase', endpoint, headers, ...rest } = props
  const [schema, setSchema] = useState<GraphQLSchema>()
  const [defaultQuery, setDefaultQuery] = useState('')
  const { sseFetcher } = useSSEContext()

  const getFetcher = useCallback<Fetcher>(
    (graphQLParams, fetcherOpts) => {
      const isLive = fetcherOpts?.documentAST
        ? isLiveQuery(fetcherOpts.documentAST, graphQLParams.operationName || undefined)
        : false
      if (isLive) {
        return sseFetcher({ url: endpoint as string, headers })(graphQLParams, fetcherOpts)
      }
      return fetcher(endpoint as string, { headers })(graphQLParams, fetcherOpts)
    },
    [endpoint, headers, sseFetcher]
  )

  useEffect(() => {
    if (!endpoint || !!schema) return
    const getTabsData = async () => {
      const newSchema = await getSchema(fetcher(endpoint, { headers }))
      if (newSchema) {
        setSchema(newSchema)
        const query = await getDefaultQuery(newSchema, storageKey)
        setDefaultQuery(query)
      }
    }
    getTabsData()
  }, [endpoint, schema])

  // wait for tabs data to be ready
  if (endpoint && !(schema && defaultQuery)) return null

  if (!endpoint) {
    return <Provider {...rest} />
  }

  return (
    <Provider
      {...rest}
      fetcher={getFetcher}
      schema={schema}
      defaultQuery={defaultQuery}
      storage={getStorage(storageKey)}
    />
  )
}

const PlaygroundWithProviders = (props: PlaygroundProps) => {
  return (
    <SSEProvider>
      <Playground {...props} />
    </SSEProvider>
  )
}

export default PlaygroundWithProviders
