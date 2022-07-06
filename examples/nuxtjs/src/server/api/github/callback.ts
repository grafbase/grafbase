export default defineEventHandler(async (event) => {
  const { code } = useQuery(event)

  if (!code) {
    return sendRedirect(event, '/')
  }
  const response: any = await $fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    body: {
      client_id: process.env.GITHUB_CLIENT_ID,
      client_secret: process.env.GITHUB_CLIENT_SECRET,
      code
    }
  })
  if (response.error) {
    return sendRedirect(event, '/')
  }

  setCookie(event, 'gh_token', response.access_token, { path: '/' })

  return sendRedirect(event, '/')
})
