export const renameTabs = (tabNames: string[]) => {
  const buttons = document.querySelectorAll(
    'div[role="tab"] button[aria-controls="graphiql-session"]'
  )
  if (buttons) {
    let index = 1
    Array.from(buttons).forEach((el, i) => {
      const closeButton = el.children.item(0)
      const tabName = tabNames[i]
      if (tabName === '<untitled>') {
        el.innerHTML = `New Tab ${index}`
        index++
        if (closeButton) {
          el.appendChild(closeButton)
        }
      }
    })
  }
}
