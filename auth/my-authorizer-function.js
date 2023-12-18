export default function ({ request }) {
  const { headers } = request

  const jwt = headers['authorization']

  console.log("#");

  // Verify JWT...

  return { identity: { sub: 'user1', groups: ['g1'] } }
}
