module.exports = {
  root: true,
  // This tells ESLint to load the config from the package `eslint-config-grafbase`
  extends: ['grafbase'],
  settings: {
    next: {
      rootDir: ['apps/*/']
    }
  }
}
