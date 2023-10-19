import { g, auth, config } from '@grafbase/sdk'

// Welcome to Grafbase!
// Define your data models, integrate auth, permission rules, custom resolvers, search, and more with Grafbase.

// Integrate Auth
// https://grafbase.com/docs/auth
const authProvider = auth.Authorizer({
    name: 'allowAll',
})

// Define Data Models
// https://grafbase.com/docs/database

g.model('Question', {
    id: g.id(),
    author: g.relation(() => user).optional(),
    content: g.string(),
    getAnswer: g.string().resolver('getAnswer')
})

const user = g.model('User', {
    name: g.string(),
    // Extend models with resolvers
    // https://grafbase.com/docs/edge-gateway/resolvers
})

export default config({
    schema: g,
    // Integrate Auth
    // https://grafbase.com/docs/auth
    auth: {
        providers: [authProvider],
        rules: (rules) => {
            rules.private()
        }
    },
    experimental: {
        kv: true
    }
})
