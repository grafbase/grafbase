import type { AstNode, AstNodes } from './language'

type Key = string | number

type AstNodeOrList = AstNode | AstNode[]

type VisitorFn<N extends AstNode> = (
  /** The current node being visiting. */
  node: N,
  /** The index or key to this node from the parent node or Array. */
  key: Key | null,
  /** The parent immediately above this node, which may be an Array. */
  parent: AstNode | ReadonlyArray<AstNode> | null,
  /** The key path to get to this node from the root node. */
  path: ReadonlyArray<Key>,
  /**
   * All nodes and Arrays visited before reaching parent of this node.
   * These correspond to array indices in `path`.
   * Note: ancestors includes arrays which contain the parent of visited node.
   */
  ancestors: ReadonlyArray<AstNode | ReadonlyArray<AstNode>>
) => any

type Visitor<N extends AstNode> = {
  enter?: VisitorFn<N>
  leave?: VisitorFn<N>
}

type VisitorMap = Partial<{
  [N in keyof AstNodes]: VisitorFn<AstNodes[N]> | Visitor<AstNodes[N]>
}>

type StackItemBase = {
  item: AstNodeOrList
  key: Key | null
  parent: AstNodeOrList | null
  prev: StackItemLeave | null
}

type StackItemLeave = StackItemBase & {
  type: 'LEAVE'
  edits: [Key | null, any][]
  shouldSkip: boolean
}

type StackItemEnter = StackItemBase & {
  type: 'ENTER'
  leave: StackItemLeave
}

type StackItem = StackItemEnter | StackItemLeave

export function traverse<
  Input extends AstNodeOrList = AstNodeOrList,
  Output = Input
>(root: Input, visitors: VisitorMap): Output {
  const visitorCache: Partial<{ [N in keyof AstNodes]: Visitor<AstNodes[N]> }> =
    {}
  function getVisitor<T extends AstNode>(node: T): Visitor<T> {
    if (!visitorCache[node.kind]) {
      const visitor = visitors[node.kind]
      visitorCache[node.kind] =
        typeof visitor === 'function'
          ? { enter: visitor as VisitorFn<AstNode>, leave: undefined }
          : (visitor as Visitor<AstNode>) || {}
    }
    return visitorCache[node.kind] as Visitor<T>
  }

  const rootStackItems = getStackItemPair(root, null, null, null)
  const stack: StackItem[] = [rootStackItems.enter, rootStackItems.leave]
  const path: Key[] = []
  const ancestors: AstNodeOrList[] = []

  let returnValue = root as unknown as Output

  do {
    const stackItem = stack.shift()!

    if (stackItem.type === 'ENTER') {
      const { item, key, parent, leave, prev } = stackItem

      if (key !== null) path.push(key)
      if (parent !== null) ancestors.push(parent)

      if (Array.isArray(item)) {
        for (let i = item.length - 1; i >= 0; i--) {
          const stackItems = getStackItemPair(item[i], i, item, leave)
          stack.unshift(stackItems.leave)
          stack.unshift(stackItems.enter)
        }
      } else {
        const { enter } = getVisitor(item)
        let updatedItem = item
        if (enter) {
          const result = enter(item, key, parent, path, ancestors)
          if (result === BREAK) break
          if (result === false) {
            leave.shouldSkip = true
            continue
          }
          if (result !== undefined) {
            updatedItem = result
            leave.item = result
            if (prev) prev.edits.push([key, result])
          }
        }

        for (const key in updatedItem) {
          const nested = (updatedItem as any)[key]
          if (isAstNodeOrList(nested)) {
            const stackItems = getStackItemPair(nested, key, updatedItem, leave)
            stack.unshift(stackItems.leave)
            stack.unshift(stackItems.enter)
          }
        }
      }
    } else {
      const {
        item: unchangedItem,
        key,
        parent,
        shouldSkip,
        edits,
        prev
      } = stackItem

      const item = mergeEdits(unchangedItem, edits)
      if (prev && item !== unchangedItem) {
        prev.edits.push([key, item])
      }

      const isRoot = stack.length === 0
      if (isRoot) returnValue = item

      if (!Array.isArray(item) && !shouldSkip) {
        const { leave } = getVisitor(item)
        if (leave) {
          const result = leave(item, key, parent, path, ancestors)
          if (result === BREAK) break
          if (result !== undefined && result !== false) {
            if (isRoot) returnValue = result
            if (prev) prev.edits.push([key, result])
          }
        }
      }

      if (key !== null) path.pop()
      if (parent !== null) ancestors.pop()
    }
  } while (stack.length > 0)

  return returnValue
}

export const BREAK = Object.freeze({})

function getStackItemPair(
  item: AstNodeOrList,
  key: Key | null,
  parent: AstNodeOrList | null,
  prevLeave: StackItemLeave | null
): {
  enter: StackItem
  leave: StackItem
} {
  const leave: StackItem = {
    type: 'LEAVE',
    item,
    key,
    parent,
    edits: [],
    shouldSkip: false,
    prev: prevLeave
  }
  const enter: StackItem = {
    type: 'ENTER',
    item,
    key,
    parent,
    leave,
    prev: prevLeave
  }
  return { enter, leave }
}

function isAstNode(value: any): value is AstNode {
  return value && typeof value === 'object' && 'kind' in value
}

function isAstNodeOrList(value: any): value is AstNodeOrList {
  return (
    isAstNode(value) ||
    (Array.isArray(value) && value.every((n) => isAstNode(n)))
  )
}

function mergeEdits(item: AstNodeOrList, edits: [Key | null, any][]) {
  if (edits.length === 0) return item

  if (Array.isArray(item)) {
    const copy = [...item]
    for (const [key, value] of edits) {
      if (key === null) continue
      copy[key as number] = copy[key as number] === null ? null : value
    }
    return copy.filter((n) => n !== null)
  }

  const copy: any = { ...item }
  for (const [key, value] of edits) {
    if (key === null) continue
    copy[key] = copy[key] === null ? null : value
  }
  return copy
}
