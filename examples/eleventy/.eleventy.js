module.exports = (config) => {
  config.addFilter('date', (date) =>
    new Intl.DateTimeFormat('en-GB', {
      dateStyle: 'medium',
      timeStyle: 'short'
    }).format(Date.parse(date))
  )

  return {
    dir: {
      input: 'src',
      output: 'public'
    }
  }
}
