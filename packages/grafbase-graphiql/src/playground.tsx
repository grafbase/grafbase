import './style.css'

import type { Theme } from '@graphiql/react'
import type { Fetcher } from '@graphiql/toolkit'
import type { GraphiQLInterfaceProps, GraphiQLProviderProps } from 'graphiql'
import { GraphiQL, GraphiQLInterface, GraphiQLProvider } from 'graphiql'
import { ReactNode, useCallback } from 'react'
import { GrafbaseLogo } from './components/grafbase-logo'
import { Toolbar } from './components/toolbar'
import { fetcher } from './utils/fetcher'
import { renameTabs } from './utils/rename-tabs'
import { isLiveQuery, SSEProvider, useSSEContext } from './utils/sse'
import { getStorage } from './utils/storage'
import { ThemeProvider } from './utils/theme'
import { validateQuery } from './utils/validate-query'

type PlaygroundProps = Omit<
  GraphiQLProviderProps,
  'children' | 'fetcher' | 'storage'
> &
  GraphiQLInterfaceProps & {
    endpoint: string
    storageKey?: string
    logo?: ReactNode
  }

const Playground = (props: PlaygroundProps) => {
  const {
    storageKey = 'grafbase',
    endpoint,
    logo,
    dangerouslyAssumeSchemaIsValid,
    defaultQuery,
    defaultTabs,
    externalFragments,
    getDefaultFieldNames,
    headers,
    initialTabs,
    inputValueDeprecation,
    introspectionQueryName,
    maxHistoryLength,
    onEditOperationName,
    onSchemaChange,
    onTabChange,
    onTogglePluginVisibility,
    operationName,
    plugins,
    query,
    response,
    schema,
    schemaDescription,
    shouldPersistHeaders,
    validationRules,
    variables,
    visiblePlugin,
    defaultHeaders,
    ...rest
  } = props
  const { sseFetcher } = useSSEContext()

  const getFetcher = useCallback<Fetcher>(
    (graphQLParams, fetcherOpts) => {
      const headers: Record<string, string> | undefined = props.defaultHeaders
        ? JSON.parse(props.defaultHeaders)
        : undefined
      const isExecutable = validateQuery(graphQLParams.query)
      if (!isExecutable) {
        return Promise.reject('Query is not executable')
      }
      const isLive = fetcherOpts?.documentAST
        ? isLiveQuery(
            fetcherOpts.documentAST,
            graphQLParams.operationName || undefined
          )
        : false
      if (isLive) {
        return sseFetcher({ url: endpoint as string, headers })(
          graphQLParams,
          fetcherOpts
        )
      }
      return fetcher(endpoint as string, { headers })(
        graphQLParams,
        fetcherOpts
      )
    },
    [endpoint, props.defaultHeaders, sseFetcher]
  )

  return (
    <GraphiQLProvider
      fetcher={getFetcher}
      storage={getStorage(storageKey)}
      onTabChange={(tabsState) => {
        const tabNames = tabsState.tabs.map((tab) => tab.title)
        setTimeout(() => renameTabs(tabNames), 0)
        onTabChange?.(tabsState)
      }}
      getDefaultFieldNames={getDefaultFieldNames}
      dangerouslyAssumeSchemaIsValid={dangerouslyAssumeSchemaIsValid}
      defaultQuery={defaultQuery}
      defaultHeaders={defaultHeaders}
      defaultTabs={defaultTabs}
      externalFragments={externalFragments}
      headers={headers}
      initialTabs={initialTabs}
      inputValueDeprecation={inputValueDeprecation}
      introspectionQueryName={introspectionQueryName}
      maxHistoryLength={maxHistoryLength}
      onEditOperationName={onEditOperationName}
      onSchemaChange={onSchemaChange}
      onTogglePluginVisibility={onTogglePluginVisibility}
      plugins={plugins}
      visiblePlugin={visiblePlugin}
      operationName={operationName}
      query={query}
      response={response}
      schema={schema}
      schemaDescription={schemaDescription}
      shouldPersistHeaders={shouldPersistHeaders}
      validationRules={validationRules}
      variables={variables}
    >
      <GraphiQLInterface
        isHeadersEditorEnabled={false}
        defaultEditorToolsVisibility={false}
        {...rest}
      >
        <GraphiQL.Logo>{logo ?? <GrafbaseLogo />}</GraphiQL.Logo>
        <GraphiQL.Toolbar>
          <Toolbar />
        </GraphiQL.Toolbar>
      </GraphiQLInterface>
    </GraphiQLProvider>
  )
}

type Props = PlaygroundProps & {
  theme?: Theme
}

const PlaygroundWithProviders = ({ theme, ...props }: Props) => {
  return (
    <ThemeProvider theme={theme}>
      <SSEProvider>
        <Playground {...props} />
      </SSEProvider>
    </ThemeProvider>
  )
}

export default PlaygroundWithProviders
