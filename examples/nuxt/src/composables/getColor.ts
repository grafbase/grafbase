const colors = [
  '#9747FF',
  '#FFAA47',
  '#be123c',
  '#b45309',
  '#a21caf',
  '#4d7c0f',
  '#6d28d9',
  '#0f766e',
  '#1d4ed8'
]

let assignedColors: Record<string, string> = {}

const getColor = (id: string) => {
  if (assignedColors[id]) {
    return assignedColors[id]
  }

  assignedColors[id] =
    colors[Object.keys(assignedColors).length % colors.length]

  return assignedColors[id]
}

export default getColor
