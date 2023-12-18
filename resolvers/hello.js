export default function Resolver(_, { name }) {
  return `Hello ${name || 'world'}!`
}
