import { g, config } from '@grafbase/sdk'

const product = g.type('Product', {
  id: g.id(),
  name: g.string(),
  slug: g.string(),
  price: g.int(),
  onSale: g.boolean()
})

const productCreateInput = g.input('ProductCreateInput', {
  name: g.string(),
  slug: g.string(),
  price: g.int(),
  onSale: g.boolean().optional()
})

g.mutation('productCreate', {
  args: { input: g.inputRef(productCreateInput) },
  resolver: 'products/create',
  returns: g.ref(product).optional()
})

const productUpdateInput = g.input('ProductUpdateInput', {
  name: g.string().optional(),
  slug: g.string().optional(),
  price: g.int().optional(),
  onSale: g.boolean().optional()
})

const productByInput = g.input('ProductByInput', {
  id: g.id().optional(),
  slug: g.string().optional()
})

g.mutation('productUpdate', {
  args: {
    by: g.inputRef(productByInput),
    input: g.inputRef(productUpdateInput)
  },
  resolver: 'products/update',
  returns: g.ref(product).optional()
})

const productDeletePayload = g.type('ProductDeletePayload', {
  deleted: g.boolean()
})

g.mutation('productDelete', {
  args: {
    by: g.inputRef(productByInput)
  },
  resolver: 'products/delete',
  returns: g.ref(productDeletePayload).optional()
})

g.query('product', {
  args: { by: g.inputRef(productByInput) },
  resolver: 'products/single',
  returns: g.ref(product).optional()
})

g.query('products', {
  args: {
    first: g.int().optional(),
    last: g.int().optional(),
    before: g.string().optional(),
    after: g.string().optional()
  },
  resolver: 'products/all',
  returns: g.ref(product).optional().list().optional()
})

export default config({
  schema: g,
  cache: {
    rules: [
      {
        maxAge: 60,
        types: [{ name: 'Query', fields: ['products', 'product'] }],
        mutationInvalidation: 'entity'
      }
    ]
  }
})
