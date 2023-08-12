import type { CodegenConfig } from '@graphql-codegen/cli'

const config: CodegenConfig = {
  schema: [
    {
      'http://127.0.0.1:4000/graphql': {
        headers: {
          'x-api-key': 'letmein'
        }
      }
    }
  ],
  generates: {
    'grafbase/__generated/resolvers.ts': {
      plugins: ['typescript', 'typescript-resolvers'],
      config: {
        useIndexSignature: true
      }
    }
  }
}

export default config
