import { GrafbaseContext } from '../context'
import { Resolver, ResolversParentTypes, QuerySumArgs, MyType } from '../types'

const SumResolver: Resolver<
  MyType,
  ResolversParentTypes['Query'],
  GrafbaseContext,
  QuerySumArgs
> = (_, args, ctx, info) => {
  const { a, b } = args
  const total = a + b

  return {
    inputA: a,
    inputB: b,
    total
  }
}

export default SumResolver
