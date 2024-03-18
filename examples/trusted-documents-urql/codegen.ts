import { type CodegenConfig } from '@graphql-codegen/cli'
 
const config: CodegenConfig = {
  schema: 'schema.graphql',
  documents: ['src/**/*.ts'],
  generates: {
    './src/gql/': {
      preset: 'client-preset',
      presetConfig: {
        persistedDocuments: true,
        fragmentMasking: false, // avoid noise
      }
    }
  }
}

export default config
