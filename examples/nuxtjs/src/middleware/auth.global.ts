export default defineNuxtRouteMiddleware(async (to) => {
  const user = (await useGithubUser()).value

  if (!user && to.path !== '/sign-in') {
    return navigateTo('/sign-in')
  }
  if (user && to.path === '/sign-in') {
    return navigateTo('/')
  }
})
