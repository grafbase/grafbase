export type AvailableTools =
  | 'Pathfinder'
  | 'SchemaDocumentation'
  | 'SchemaDefinition'

type Theme = 'light' | 'dark'

export type UseCliAppStore = {
  theme: Theme
  toggleTheme: () => void
  visibleTool: AvailableTools
  setVisibleTool: ({ visibleTool }: { visibleTool: AvailableTools }) => void
}
