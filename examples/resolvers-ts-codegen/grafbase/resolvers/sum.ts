import type { QuerySumArgs, Query } from '../__generated/resolvers'

export default function SumResolver(_: any, args: QuerySumArgs): Query['sum'] {
  return args.a + args.b
}
