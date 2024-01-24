export default function Resolver(_, { name }) {
  return `Goodbye ${name || 'good sir'}!`
}
