export default function HelloResolver(_, { name = 'world' }) {
  return {
    name,
  };
}
