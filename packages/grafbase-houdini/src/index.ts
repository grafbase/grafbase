import { ArtifactKind, plugin, PluginHooks } from 'houdini'
import * as graphql from 'graphql'

// define the plugin
export default plugin('grafbase-houdini', async (): Promise<PluginHooks> => {
  return {
    /**
     * Add the client plugin to the runtime
     */
    clientPlugins: {
      '@grafbase/houdini/client': null
    },

    /** Configure the default set of scalars supported by Grafbase */
    config: '@grafbase/houdini/config',

    /**
     * We want to perform special logic for the the @live directive so we're going to persist
     * data in the artifact if we detect it
     */
    artifactData({ document }) {
      // only consider queries
      if (document.kind !== ArtifactKind.Query) {
        return
      }

      // look at the original document the user passed (only one definition)
      const queryDefinition = document.originalParsed
        .definitions[0] as graphql.OperationDefinitionNode

      // consider the artifact live if the query contains the live directive
      return {
        live: !!queryDefinition?.directives?.find(
          (directive) => directive.name.value === 'live'
        )
      }
    }
  }
})
