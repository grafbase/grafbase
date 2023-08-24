import { g, config } from '@grafbase/sdk'

const product = g.type('Product', {
  id: g.id(),
  name: g.string(),
  slug: g.string(),
  price: g.int(),
  onSale: g.boolean().optional()
})

const productCreateInput = g.input('ProductCreateInput', {
  name: g.string(),
  slug: g.string(),
  price: g.int(),
  onSale: g.boolean().optional()
})

g.mutation('productCreate', {
  args: {
    input: g.inputRef(productCreateInput)
  },
  resolver: 'products/create',
  returns: g.ref(product).optional()
})

g.query('products', {
  resolver: 'products/all',
  returns: g.ref(product).optional().list().optional()
})

export default config({
  schema: g
})
