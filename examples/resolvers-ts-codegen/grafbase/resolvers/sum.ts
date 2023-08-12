import type { QuerySumArgs, Query } from '../__generated/resolvers'

export default function SumResolver(
  _: unknown,
  args: QuerySumArgs
): Query['sum'] {
  return {
    total: args.a + args.b,
    inputA: args.a,
    inputB: args.b
  }
}
