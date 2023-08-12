import type {
  ResolversParentTypes,
  QuerySumArgs,
  MyType
} from '../__generated/resolvers'

const SumResolver: (
  parent: ResolversParentTypes['Query'],
  args: QuerySumArgs
) => MyType = (_, args) => {
  const { a, b } = args
  const total = a + b

  return {
    inputA: a,
    inputB: b,
    total
  }
}

export default SumResolver
