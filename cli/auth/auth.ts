export default function (parent, args, context, info) {
  return { identity: { sub: 'user1', groups: ['g1'] } }
}
