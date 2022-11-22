import { GraphQLField, GraphQLSchema } from 'graphql'
import { prettify } from './prettify'
import { getPlaygroundTabs, setPlaygroundQuery, setPlaygroundTabs } from './storage'

const placeholders = {
  create: {
    title: 'create',
    text: `# Welcome to Grafbase!
# 
# Here you can query and mutate your data using GraphQL.
# Click the Execute button to run your GraphQL operation.
# 
# For more information on how GraphQL works read our documentation:
# https://grafbase.com/docs/reference/mutations#create
# 
# Here's an example of a create mutation:\n`
  },
  delete: {
    title: 'delete',
    text: `# Delete example
# 
# Click the Execute button to run your GraphQL operation.
# 
# For more information on how GraphQL works read our documentation:
# https://grafbase.com/docs/reference/mutations#delete
# 
# Here's an example of a delete mutation:\n`
  },
  pagination: {
    title: 'pagination',
    text: `# Pagination example
# 
# Here you can see an example of how you paginate a collection.
# 
# All collections must have one of the following pagination arguments:
# - "first" / "last" - Int! - number of nodes per page
# - "before" / "after" - String! - cursor to retrieve nodes before or after in the collection
# 
# For more information on how GraphQL pagination works read:
# https://grafbase.com/docs/reference/pagination
# 
# Here you can see an example of how to paginate a collection:\n`
  }
}

const typeToPlaceholder = {
  ID: '"ID#1"',
  String: '"Foo bar"',
  Boolean: false,
  Float: 0.5,
  Int: 5,
  Email: '"baz@foo.bar"',
  IPAddress: '"123.123.123.123"',
  URL: '"https://foo.bar/"',
  DateTime: '"2020-01-01T00:00:00.000Z"',
  Timestamp: '"2020-01-01T00:00:00.000Z"',
  JSON: '{ "foo": "bar" }'
}

type PlaceholderTypes = keyof typeof typeToPlaceholder

const isPreferredType = (name: string) => {
  return /user|customer/.test(name.toLowerCase())
}

const filterPreferredType = (fields: GraphQLField<any, any, any>[]) => {
  const filtered = fields.filter((f) => isPreferredType(f.name))
  return filtered.length > 0 ? filtered : fields
}

/**
 * Hack to handle `@oneOf` type
 * `${type}ByInput` has to be nullable (not required) for backwards compatibility
 */
const isOneOfType = (name: string, type: string) => name === 'by' && type.endsWith('ByInput')

type MappedArgs = {
  name: string
  type: string
  required: boolean
  placeholder?: typeof typeToPlaceholder[PlaceholderTypes]
  nodes?: MappedArgs[]
  isByInput?: Boolean
}

const mapArg = (schema: GraphQLSchema, a: any, maxLevel = 1): MappedArgs => {
  const { name, type, isByInput } = a as MappedArgs
  const typeName = type?.toString?.()?.replace(/\[|\]|!/gi, '') as PlaceholderTypes // remove "!", "[" and "]"
  const placeholder = typeToPlaceholder[typeName]
  const edges: MappedArgs[] =
    // @ts-ignore
    Object.values(schema.getType(typeName)?.getFields?.() ?? {}) ?? []
  const nodes =
    maxLevel > 0 ? edges.map((n) => mapArg(schema, { ...n, isByInput: isOneOfType(name, typeName) }, maxLevel - 1)) : []
  const required = maxLevel > 0 ? a.type.toString().endsWith('!') || !!isByInput || !placeholder : false

  return {
    name,
    type: typeName,
    required,
    placeholder,
    nodes
  }
}

const filterNestedNodes = (nodes: MappedArgs[], maxNodes = 1) => {
  let count = 0
  return nodes.filter((n) => {
    if (!n.nodes?.length) return true
    if (count < maxNodes) {
      count++
      return true
    }
    return false
  })
}

/** commas should be handled by GraphQL print() */
const stringifyMap = (forInput = true) => {
  return (res: string, a: MappedArgs): string => {
    if (a.required) {
      if (a.nodes?.length) {
        const nodes = forInput && a.nodes.length > 1 ? filterNestedNodes(a.nodes) : a.nodes
        const stringifiedNodes = nodes.reduce(stringifyMap(forInput), '')
        if (!stringifiedNodes) return res
        return forInput ? `${res} ${a.name}: { ${stringifiedNodes} }` : `${res} ${a.name} { ${stringifiedNodes} }`
      }
      if (a.placeholder !== undefined) {
        return forInput ? `${res} ${a.name}: ${a.placeholder}` : `${res} ${a.name}`
      }
    }
    return res
  }
}

const getExampleMutation = async (schema: GraphQLSchema) => {
  let createExample = ''
  let deleteExample = ''

  const mutationFields = schema.getMutationType()?.getFields() ?? {}
  const mutations = Object.values(mutationFields)
  const filteredMutations = filterPreferredType(mutations)

  const createMutation = filteredMutations.find((f) => f.name.includes('Create'))
  if (createMutation) {
    const name = createMutation.name
    const mappedArgs = createMutation.args.map((a) => mapArg(schema, a, 4))
    // @ts-ignore
    const payloadFields = createMutation?.type?.getFields?.() ?? {}
    const mappedPayloadFields = Object.values(payloadFields).map((a) => mapArg(schema, a, 3))

    const stringifiedArgs = mappedArgs.reduce(stringifyMap(true), '')
    const stringifiedPayload = mappedPayloadFields.reduce(stringifyMap(false), '')

    const mutationQuery = `mutation ${placeholders['create'].title} { ${name}(${stringifiedArgs}) { ${stringifiedPayload} } }`

    try {
      const prettifyMutation = await prettify(mutationQuery)
      if (prettifyMutation) {
        createExample = prettifyMutation
      }
    } catch {}
  }

  const deleteMutation = filteredMutations.find((f) => f.name.includes('Delete'))
  if (deleteMutation) {
    const name = deleteMutation.name
    const mappedArgs = deleteMutation.args.map((a) => mapArg(schema, a, 4))
    // @ts-ignore
    const payloadFields = deleteMutation?.type?.getFields?.() ?? {}
    const mappedPayloadFields = Object.values(payloadFields).map((a) => mapArg(schema, a, 3))

    const stringifiedArgs = mappedArgs.reduce(stringifyMap(true), '')
    const stringifiedPayload = mappedPayloadFields.reduce(stringifyMap(false), '')

    const mutationQuery = `mutation ${placeholders['delete'].title} { ${name}(${stringifiedArgs}) { ${stringifiedPayload} } }`

    try {
      const prettifyMutation = await prettify(mutationQuery)
      if (prettifyMutation) {
        deleteExample = prettifyMutation
      }
    } catch {}
  }

  return { createExample, deleteExample }
}

const getExamplePagination = async (schema: GraphQLSchema) => {
  let query = ''

  const queryFields = schema.getQueryType()?.getFields() ?? {}
  const collections = Object.values(queryFields)
  const filteredCollections = filterPreferredType(collections)

  const collectionQuery = filteredCollections.find((f) => f.name.includes('Collection'))

  if (collectionQuery) {
    const name = collectionQuery.name
    // @ts-ignore
    const payloadFields = collectionQuery?.type?.getFields?.() ?? {}
    const mappedPayloadFields = Object.values(payloadFields).map((a) => mapArg(schema, a, 3))

    const stringifiedPayload = mappedPayloadFields.reduce(stringifyMap(false), '')

    const paginationQuery = `query ${placeholders['pagination'].title} { ${name}(first: 5) { ${stringifiedPayload} } }`

    try {
      const prettifyMutation = await prettify(paginationQuery)
      if (prettifyMutation) {
        query = prettifyMutation
      }
    } catch {}
  }

  return { paginationExample: query }
}

const guid = (): string => {
  const s4 = () =>
    Math.floor((1 + Math.random()) * 0x10000)
      .toString(16)
      .substring(1)
  return `${s4()}${s4()}-${s4()}-${s4()}-${s4()}-${s4()}${s4()}${s4()}`
}

const getEmptyTabData = () => ({
  id: guid(),
  title: '<untitled>',
  query: null,
  variables: null,
  headers: null,
  operationName: null,
  response: null
})

export const getDefaultQuery = async (schema: GraphQLSchema, storageKey: string) => {
  let defaultQuery = ''
  const rawTabs = getPlaygroundTabs(storageKey)
  const query = !!rawTabs && JSON.parse(rawTabs)?.tabs?.[0]?.query

  if (query) {
    defaultQuery = query
  } else {
    const { createExample, deleteExample } = await getExampleMutation(schema)
    const { paginationExample } = await getExamplePagination(schema)

    defaultQuery = placeholders['create'].text + createExample
    const initialTabsState = {
      activeTabIndex: 0,
      tabs: [
        {
          ...getEmptyTabData(),
          title: placeholders['create'].title,
          query: defaultQuery
        },
        {
          ...getEmptyTabData(),
          title: placeholders['pagination'].title,
          query: placeholders['pagination'].text + paginationExample
        },
        {
          ...getEmptyTabData(),
          title: placeholders['delete'].title,
          query: placeholders['delete'].text + deleteExample
        }
      ]
    }

    setPlaygroundQuery(storageKey, defaultQuery)
    setPlaygroundTabs(storageKey, JSON.stringify(initialTabsState))
  }

  return defaultQuery
}
