import { GrafbaseContext, GrafbaseInfo } from '../context'
import type { ResolversParentTypes, QuerySumArgs, MyType } from '../types'

const SumResolver: (
  parent: ResolversParentTypes['Query'],
  args: QuerySumArgs,
  ctx: GrafbaseContext,
  info: GrafbaseInfo
) => MyType = (_, args, ctx, info) => {
  const { a, b } = args
  const total = a + b

  return {
    inputA: a,
    inputB: b,
    total
  }
}

export default SumResolver
