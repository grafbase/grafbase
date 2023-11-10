import { config, g } from '@grafbase/sdk'

const address = g.type('Address', {
  street: g.string().optional(),
})

g.model('User', {
  name: g.string(),
  address: g.ref(address).optional(),
})

export default config({ schema: g })
