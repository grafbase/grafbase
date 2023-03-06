import {
  ArgumentSetConstNode,
  ArgumentSetNode,
  AstNodes,
  CommentNode,
  DefinitionNode,
  DirectiveConstNode,
  DirectiveLocationNode,
  DirectiveNode,
  OperationType,
  SelectionNode,
  TypeNode,
  TypeSystemExtensionNode,
  ValueConstNode,
  ValueNode,
  isExecutableDirectiveLocation,
  isTypeSystemDirectiveLocation
} from './language'
import { LexicalToken, Token, tokenize } from './tokenize'

export function parse(source: string): AstNodes['Document'] {
  const iterator = tokenize(source)

  /**
   * When the `peekCache` is empty, the `peek()` method requires the
   * following:
   * - `this.next` is set to the next token that is not consumed yet (this
   *   can be any token except for an inline-comment which would have already
   *   been consumed in the prevoius peek)
   * - `this.nextNext` is `undefined`
   *
   * When calling `take()` (which effectively empties the `peekCache`) we
   * restore this state by assigning the value of `this.nextNext` to
   * `this.next`.
   */
  const state: {
    next: LexicalToken | undefined
    nextNext: LexicalToken | undefined
    peekCache:
      | { token: LexicalToken | undefined; comments: CommentNode[] }
      | undefined
  } = {
    next: iterator.next().value,
    nextNext: undefined,
    peekCache: undefined
  }

  function peek() {
    if (!state.peekCache) {
      state.peekCache = { token: undefined, comments: [] }

      let nextToken: Token | undefined = state.next
      while (nextToken && nextToken.type === 'BLOCK_COMMENT') {
        state.peekCache.comments.push({
          kind: 'BlockComment',
          value: nextToken.value
        })
        nextToken = iterator.next().value
      }

      state.peekCache.token = nextToken as LexicalToken

      let nextNext: Token | undefined = iterator.next().value
      while (nextNext && nextNext.type === 'INLINE_COMMENT') {
        state.peekCache.comments.push({
          kind: 'InlineComment',
          value: nextNext.value
        })
        nextNext = iterator.next().value
      }
      state.nextNext = nextNext as LexicalToken
    }
    return state.peekCache
  }

  function take() {
    const token = peek()
    state.next = state.nextNext
    state.nextNext = undefined
    state.peekCache = undefined
    return token
  }

  function assertToken(expected: Token['type'], expectedValue?: string) {
    const { token, comments } = peek()
    if (
      !token ||
      token.type !== expected ||
      (typeof expectedValue === 'string' && token.value !== expectedValue)
    )
      throw new Error(
        `Unexpected ${token ? `token: ${token.value}` : 'EOF'}${
          expectedValue ? ` (expected ${expectedValue})` : ''
        }`
      )
    return { token, comments }
  }

  function isNext(type: Token['type'], value?: string) {
    try {
      assertToken(type, value)
      return true
    } catch {
      return false
    }
  }

  function assertCombinedListLength(
    lists: unknown[][],
    nullableObjects: unknown[],
    expectedPunctuator: string
  ) {
    if (
      lists.every((list) => list.length === 0) &&
      nullableObjects.every((obj) => obj === null)
    )
      assertToken('PUNCTUATOR', expectedPunctuator)
  }

  function takeToken(expected: Token['type'], expectedValue?: string) {
    const next = assertToken(expected, expectedValue)
    take()
    return next
  }

  function takePunctuator(punctuator: string) {
    return takeToken('PUNCTUATOR', punctuator)
  }

  function isNextPunctuator(punctuator: string) {
    const { token } = peek()
    return (
      (token &&
        token.type === 'PUNCTUATOR' &&
        (punctuator === undefined || token.value === punctuator)) ||
      false
    )
  }

  function takeIfNextPunctuator(punctuator: string) {
    return isNextPunctuator(punctuator)
      ? take()
      : { token: undefined, comments: [] }
  }

  function takeList<T>(haultCondition: () => boolean, callback: () => T) {
    const items: T[] = []

    while (haultCondition()) {
      items.push(callback())
    }

    return items
  }

  function takeWrappedList<T>(
    isOptional: boolean,
    startPunctuator: string,
    endPunctuator: string,
    callback: () => T
  ) {
    let commentsOpeningBracket: CommentNode[] = []
    let commentsClosingBracket: CommentNode[] = []
    try {
      commentsOpeningBracket = takePunctuator(startPunctuator).comments
    } catch (err) {
      if (isOptional)
        return {
          items: [],
          commentsOpeningBracket: [],
          commentsClosingBracket: []
        }
      throw err
    }
    const items = takeList(() => {
      const { token, comments } = takeIfNextPunctuator(endPunctuator)
      if (token) commentsClosingBracket = comments
      return !token
    }, callback)
    return { items, commentsOpeningBracket, commentsClosingBracket }
  }

  function takeDelimitedList<T>(
    delimiter: string,
    initializer: Token,
    callback: (comments: CommentNode[]) => T
  ) {
    const items: T[] = []

    let initializerComments: CommentNode[] = []
    try {
      initializerComments = takeToken(
        initializer.type,
        initializer.value
      ).comments
    } catch {
      return { items, initializerComments: [] }
    }

    let comments: CommentNode[] = []
    try {
      comments = takePunctuator(delimiter).comments
    } catch {}

    items.push(callback(comments))

    let token: Token | undefined
    while ((({ token, comments } = takeIfNextPunctuator(delimiter)), token)) {
      items.push(callback(comments))
    }

    return { items, initializerComments }
  }

  function parseDescription(): AstNodes['StringValue'] | null {
    const { token, comments } = peek()
    if (
      token &&
      (token.type === 'STRING_VALUE' || token.type === 'BLOCK_STRING_VALUE')
    ) {
      take()
      return {
        kind: 'StringValue',
        value: token.value,
        block: token.type === 'BLOCK_STRING_VALUE',
        comments
      }
    }
    return null
  }

  function parseName(bad?: string): {
    node: AstNodes['Name']
    comments: CommentNode[]
  } {
    const {
      token: { value },
      comments
    } = takeToken('NAME')
    if (bad && value === bad) throw new Error(`Unexpected token "${bad}"`)
    return { node: { kind: 'Name', value }, comments }
  }

  function parseNamedType(): AstNodes['NamedType'] {
    const name = parseName()
    return { kind: 'NamedType', name: name.node, comments: name.comments }
  }

  function parseType(): TypeNode {
    const open = takeIfNextPunctuator('[')
    if (open.token) {
      const type = parseType()
      const close = takePunctuator(']')

      const listType: AstNodes['ListType'] = {
        kind: 'ListType',
        type,
        comments: [...open.comments, ...type.comments, ...close.comments]
      }
      type.comments = []

      const bang = takeIfNextPunctuator('!')
      if (bang.token) {
        const comments = [...listType.comments, ...bang.comments]
        listType.comments = []
        return { kind: 'NonNullType', type: listType, comments }
      }

      return listType
    }

    const name = parseNamedType()

    const bang = takeIfNextPunctuator('!')
    if (bang.token) {
      const comments = [...name.comments, ...bang.comments]
      name.comments = []
      return { kind: 'NonNullType', type: name, comments }
    }

    return name
  }

  function parseVariable(): AstNodes['Variable'] {
    const { comments } = takePunctuator('$')
    const name = parseName()
    comments.push(...name.comments)
    return { kind: 'Variable', name: name.node, comments }
  }

  function parseValue(isConst: false): ValueNode
  function parseValue(isConst: true): ValueConstNode
  function parseValue(isConst: boolean): ValueNode | ValueConstNode {
    if (isNextPunctuator('$') && !isConst) return parseVariable()
    if (isNextPunctuator('[')) {
      const { items, commentsOpeningBracket, commentsClosingBracket } =
        takeWrappedList<ValueNode | ValueConstNode>(false, '[', ']', () =>
          isConst ? parseValue(true) : parseValue(false)
        )
      return {
        kind: 'ListValue',
        values: items,
        commentsOpeningBracket,
        commentsClosingBracket
      }
    }
    if (isNextPunctuator('{')) {
      const { items, commentsOpeningBracket, commentsClosingBracket } =
        takeWrappedList<AstNodes['ObjectField']>(false, '{', '}', () => {
          const name = parseName()
          const colon = takePunctuator(':')
          const value = isConst ? parseValue(true) : parseValue(false)
          const comments = [...name.comments, ...colon.comments]
          return { kind: 'ObjectField', name: name.node, value, comments }
        })
      return {
        kind: 'ObjectValue',
        fields: items,
        commentsOpeningBracket,
        commentsClosingBracket
      }
    }

    const { token, comments } = take()
    if (!token) throw new Error('Unexpected EOF')
    if (token.type === 'PUNCTUATOR')
      throw new Error(`Unexpected token: ${token.value}`)
    if (token.type === 'INT_VALUE')
      return { kind: 'IntValue', value: token.value, comments }
    if (token.type === 'FLOAT_VALUE')
      return { kind: 'FloatValue', value: token.value, comments }
    if (token.type === 'STRING_VALUE')
      return {
        kind: 'StringValue',
        value: token.value,
        block: false,
        comments
      }
    if (token.type === 'BLOCK_STRING_VALUE')
      return { kind: 'StringValue', value: token.value, block: true, comments }
    if (token.value === 'true' || token.value === 'false')
      return { kind: 'BooleanValue', value: token.value === 'true', comments }
    if (token.value === 'null') return { kind: 'NullValue', comments }
    return { kind: 'EnumValue', value: token.value, comments }
  }

  function parseArgumentSet(isConst: false): ArgumentSetNode | null
  function parseArgumentSet(isConst: true): ArgumentSetConstNode | null
  function parseArgumentSet(isConst: boolean): AstNodes['ArgumentSet'] | null {
    const { items, commentsOpeningBracket, commentsClosingBracket } =
      takeWrappedList<AstNodes['Argument']>(true, '(', ')', () => {
        const name = parseName()
        const colon = takePunctuator(':')
        const value = isConst ? parseValue(true) : parseValue(false)
        const comments = [...name.comments, ...colon.comments]
        return { kind: 'Argument', name: name.node, value, comments }
      })
    return items.length === 0
      ? null
      : {
          kind: 'ArgumentSet',
          args: items,
          commentsOpeningBracket,
          commentsClosingBracket
        }
  }

  function parseDirectives(isConst: false): DirectiveNode[]
  function parseDirectives(isConst: true): DirectiveConstNode[]
  function parseDirectives(
    isConst: boolean
  ): DirectiveNode[] | DirectiveConstNode[] {
    return takeList<AstNodes['Directive']>(
      () => isNextPunctuator('@'),
      () => {
        const at = takePunctuator('@')
        const name = parseName()
        const argumentSet = isConst
          ? parseArgumentSet(true)
          : parseArgumentSet(false)
        const comments = [...at.comments, ...name.comments]
        return { kind: 'Directive', name: name.node, argumentSet, comments }
      }
    )
  }

  function parseTypeCondition(isOptional: false): {
    type: AstNodes['NamedType']
    comments: CommentNode[]
  }
  function parseTypeCondition(
    isOptional: true
  ): { type: AstNodes['NamedType']; comments: CommentNode[] } | null
  function parseTypeCondition(
    isOptional: boolean
  ): { type: AstNodes['NamedType']; comments: CommentNode[] } | null {
    let comments: CommentNode[] = []
    try {
      comments = takeToken('NAME', 'on').comments
    } catch (err) {
      if (!isOptional) throw err
      return null
    }
    return { type: parseNamedType(), comments }
  }

  function parseSelectionSet(isOptional: false): AstNodes['SelectionSet']
  function parseSelectionSet(isOptional: true): AstNodes['SelectionSet'] | null
  function parseSelectionSet(
    isOptional: boolean
  ): AstNodes['SelectionSet'] | null {
    const { items, commentsOpeningBracket, commentsClosingBracket } =
      takeWrappedList<SelectionNode>(isOptional, '{', '}', () => {
        const spread = takeIfNextPunctuator('...')
        if (spread.token) {
          const { token } = peek()
          if (token && token.type === 'NAME' && token.value !== 'on') {
            const name = parseName()
            const directives = parseDirectives(false)
            const comments = [...spread.comments, ...name.comments]
            return {
              kind: 'FragmentSpread',
              name: name.node,
              directives,
              comments
            }
          }
          const typeCondition = parseTypeCondition(true)
          const directives = parseDirectives(false)
          const selectionSet = parseSelectionSet(false)
          const comments = [
            ...spread.comments,
            ...(typeCondition ? typeCondition.comments : [])
          ]
          return {
            kind: 'InlineFragment',
            typeCondition: typeCondition ? typeCondition.type : null,
            directives,
            selectionSet,
            comments
          }
        }

        let alias: {
          node: AstNodes['Name']
          comments: CommentNode[]
        } | null = null
        let name = parseName()

        const colon = takeIfNextPunctuator(':')
        if (colon.token) {
          alias = name
          name = parseName()
        }

        const argumentSet = parseArgumentSet(false)
        const directives = parseDirectives(false)
        const selectionSet = parseSelectionSet(true)

        const comments = [
          ...(alias ? alias.comments : []),
          ...colon.comments,
          ...name.comments
        ]

        return {
          kind: 'Field',
          alias: alias ? alias.node : null,
          name: name.node,
          argumentSet,
          directives,
          selectionSet,
          comments
        }
      })
    return items.length === 0
      ? null
      : {
          kind: 'SelectionSet',
          selections: items,
          commentsOpeningBracket,
          commentsClosingBracket
        }
  }

  function parseOperationType(): {
    value: OperationType
    comments: CommentNode[]
  } {
    const {
      token: { value },
      comments
    } = takeToken('NAME')
    if (value !== 'query' && value !== 'mutation' && value !== 'subscription')
      throw new Error(`Unexpected token "${value}"`)
    return { value, comments }
  }

  function parseInterfaces(): AstNodes['NamedTypeSet'] | null {
    const { items, initializerComments } = takeDelimitedList<
      AstNodes['NamedType']
    >('&', { type: 'NAME', value: 'implements' }, (comments) => {
      const type = parseNamedType()
      type.comments.unshift(...comments)
      return type
    })
    return items.length === 0
      ? null
      : {
          kind: 'NamedTypeSet',
          types: items,
          comments: initializerComments
        }
  }

  function parseDefaultValue(): ValueConstNode | null {
    try {
      takePunctuator('=')
    } catch {
      return null
    }

    return parseValue(true)
  }

  function parseInputValueDefinitionSet(
    startPunctuator: string,
    endPunctuator: string
  ): AstNodes['InputValueDefinitionSet'] | null {
    const { items, commentsOpeningBracket, commentsClosingBracket } =
      takeWrappedList<AstNodes['InputValueDefinition']>(
        true,
        startPunctuator,
        endPunctuator,
        () => {
          const description = parseDescription()
          const name = parseName()
          const colon = takePunctuator(':')
          const type = parseType()
          const defaultValue = parseDefaultValue()
          const directives = parseDirectives(true)
          const comments = [...name.comments, ...colon.comments]
          return {
            kind: 'InputValueDefinition',
            description,
            name: name.node,
            type,
            defaultValue,
            directives,
            comments
          }
        }
      )
    return items.length === 0
      ? null
      : {
          kind: 'InputValueDefinitionSet',
          definitions: items,
          commentsOpeningBracket,
          commentsClosingBracket
        }
  }

  function parseFieldDefinitionSet(): AstNodes['FieldDefinitionSet'] | null {
    const { items, commentsOpeningBracket, commentsClosingBracket } =
      takeWrappedList<AstNodes['FieldDefinition']>(true, '{', '}', () => {
        const description = parseDescription()
        const name = parseName()
        const inputValueDefinitionSet = parseInputValueDefinitionSet('(', ')')
        const colon = takePunctuator(':')
        const type = parseType()
        const directives = parseDirectives(true)
        const comments = [...name.comments, ...colon.comments]
        return {
          kind: 'FieldDefinition',
          description,
          name: name.node,
          inputValueDefinitionSet,
          type,
          directives,
          comments
        }
      })
    return items.length === 0
      ? null
      : {
          kind: 'FieldDefinitionSet',
          definitions: items,
          commentsOpeningBracket,
          commentsClosingBracket
        }
  }

  function parseEnumValue(): AstNodes['EnumValue'] {
    const name = parseName()
    if (
      name.node.value === 'null' ||
      name.node.value === 'true' ||
      name.node.value === 'false'
    )
      throw new Error(`Unexpected token "${name.node.value}"`)
    return {
      kind: 'EnumValue',
      value: name.node.value,
      comments: name.comments
    }
  }

  function parseSchemaDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['SchemaDefinition']
  function parseSchemaDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['SchemaExtension']
  function parseSchemaDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['SchemaDefinition'] | AstNodes['SchemaExtension'] {
    const keyword = takeToken('NAME', 'schema')
    const directives = parseDirectives(true)
    const { items, commentsOpeningBracket, commentsClosingBracket } =
      takeWrappedList<AstNodes['OperationTypeDefinition']>(
        true,
        '{',
        '}',
        () => {
          const operation = parseOperationType()
          const colon = takePunctuator(':')
          const type = parseNamedType()
          return {
            kind: 'OperationTypeDefinition',
            operation: operation.value,
            type,
            comments: [...operation.comments, ...colon.comments]
          }
        }
      )
    const operationTypeDefinitionSet:
      | AstNodes['OperationTypeDefinitionSet']
      | null =
      items.length === 0
        ? null
        : {
            kind: 'OperationTypeDefinitionSet',
            definitions: items,
            commentsOpeningBracket,
            commentsClosingBracket
          }
    const comments = [...(extendComments || []), ...keyword.comments]
    if (extendComments) assertCombinedListLength([directives, items], [], '{')
    return extendComments
      ? {
          kind: 'SchemaExtension',
          directives,
          operationTypeDefinitionSet,
          comments
        }
      : {
          kind: 'SchemaDefinition',
          description,
          directives,
          operationTypeDefinitionSet,
          comments
        }
  }

  function parseScalarTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['ScalarTypeDefinition']
  function parseScalarTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['ScalarTypeExtension']
  function parseScalarTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['ScalarTypeDefinition'] | AstNodes['ScalarTypeExtension'] {
    const keyword = takeToken('NAME', 'scalar')
    const name = parseName()
    const directives = parseDirectives(true)
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments) assertCombinedListLength([directives], [], '@')
    return extendComments
      ? {
          kind: 'ScalarTypeExtension',
          name: name.node,
          directives,
          comments
        }
      : {
          kind: 'ScalarTypeDefinition',
          description,
          name: name.node,
          directives,
          comments
        }
  }

  function parseObjectTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['ObjectTypeDefinition']
  function parseObjectTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['ObjectTypeExtension']
  function parseObjectTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['ObjectTypeDefinition'] | AstNodes['ObjectTypeExtension'] {
    const keyword = takeToken('NAME', 'type')
    const name = parseName()
    const interfaces = parseInterfaces()
    const directives = parseDirectives(true)
    const fieldDefinitionSet = parseFieldDefinitionSet()
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments)
      assertCombinedListLength(
        [directives],
        [interfaces, fieldDefinitionSet],
        '{'
      )
    return extendComments
      ? {
          kind: 'ObjectTypeExtension',
          name: name.node,
          interfaces,
          directives,
          fieldDefinitionSet,
          comments
        }
      : {
          kind: 'ObjectTypeDefinition',
          description,
          name: name.node,
          interfaces,
          directives,
          fieldDefinitionSet,
          comments
        }
  }

  function parseInterfaceTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['InterfaceTypeDefinition']
  function parseInterfaceTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['InterfaceTypeExtension']
  function parseInterfaceTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['InterfaceTypeDefinition'] | AstNodes['InterfaceTypeExtension'] {
    const keyword = takeToken('NAME', 'interface')
    const name = parseName()
    const interfaces = parseInterfaces()
    const directives = parseDirectives(true)
    const fieldDefinitionSet = parseFieldDefinitionSet()
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments)
      assertCombinedListLength(
        [directives],
        [interfaces, fieldDefinitionSet],
        '{'
      )
    return extendComments
      ? {
          kind: 'InterfaceTypeExtension',
          name: name.node,
          interfaces,
          directives,
          fieldDefinitionSet,
          comments
        }
      : {
          kind: 'InterfaceTypeDefinition',
          description,
          name: name.node,
          interfaces,
          directives,
          fieldDefinitionSet,
          comments
        }
  }

  function parseUnionTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['UnionTypeDefinition']
  function parseUnionTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['UnionTypeExtension']
  function parseUnionTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['UnionTypeDefinition'] | AstNodes['UnionTypeExtension'] {
    const keyword = takeToken('NAME', 'union')
    const name = parseName()
    const directives = parseDirectives(true)
    const { items, initializerComments } = takeDelimitedList<
      AstNodes['NamedType']
    >('|', { type: 'PUNCTUATOR', value: '=' }, (comments) => {
      const type = parseNamedType()
      type.comments.unshift(...comments)
      return type
    })
    const types: AstNodes['NamedTypeSet'] | null =
      items.length === 0
        ? null
        : { kind: 'NamedTypeSet', types: items, comments: initializerComments }
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments) assertCombinedListLength([directives], [types], '=')
    return extendComments
      ? {
          kind: 'UnionTypeExtension',
          name: name.node,
          directives,
          types,
          comments
        }
      : {
          kind: 'UnionTypeDefinition',
          description,
          name: name.node,
          directives,
          types,
          comments
        }
  }

  function parseEnumTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['EnumTypeDefinition']
  function parseEnumTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['EnumTypeExtension']
  function parseEnumTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ): AstNodes['EnumTypeDefinition'] | AstNodes['EnumTypeExtension'] {
    const keyword = takeToken('NAME', 'enum')
    const name = parseName()
    const directives = parseDirectives(true)
    const values = takeWrappedList<AstNodes['EnumValueDefinition']>(
      true,
      '{',
      '}',
      () => {
        const description = parseDescription()
        const name = parseEnumValue()
        const directives = parseDirectives(true)
        const comments = name.comments
        name.comments = []
        return {
          kind: 'EnumValueDefinition',
          description,
          name,
          directives,
          comments
        }
      }
    )
    const valueDefinitionSet: AstNodes['EnumValueDefinitionSet'] | null =
      values.items.length === 0
        ? null
        : {
            kind: 'EnumValueDefinitionSet',
            definitions: values.items,
            commentsOpeningBracket: values.commentsOpeningBracket,
            commentsClosingBracket: values.commentsClosingBracket
          }
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments)
      assertCombinedListLength([directives, values.items], [], '{')
    return extendComments
      ? {
          kind: 'EnumTypeExtension',
          name: name.node,
          directives,
          valueDefinitionSet,
          comments
        }
      : {
          kind: 'EnumTypeDefinition',
          description,
          name: name.node,
          directives,
          valueDefinitionSet,
          comments
        }
  }

  function parseInputObjectTypeDefinition(
    extendComments: null,
    description: AstNodes['StringValue'] | null
  ): AstNodes['InputObjectTypeDefinition']
  function parseInputObjectTypeDefinition(
    extendComments: CommentNode[],
    description?: undefined
  ): AstNodes['InputObjectTypeExtension']
  function parseInputObjectTypeDefinition(
    extendComments: CommentNode[] | null,
    description: AstNodes['StringValue'] | null = null
  ):
    | AstNodes['InputObjectTypeDefinition']
    | AstNodes['InputObjectTypeExtension'] {
    const keyword = takeToken('NAME', 'input')
    const name = parseName()
    const directives = parseDirectives(true)
    const inputValueDefinitionSet = parseInputValueDefinitionSet('{', '}')
    const comments = [
      ...(extendComments || []),
      ...keyword.comments,
      ...name.comments
    ]
    if (extendComments)
      assertCombinedListLength([directives], [inputValueDefinitionSet], '{')
    return extendComments
      ? {
          kind: 'InputObjectTypeExtension',
          name: name.node,
          directives,
          inputValueDefinitionSet,
          comments
        }
      : {
          kind: 'InputObjectTypeDefinition',
          description,
          name: name.node,
          directives,
          inputValueDefinitionSet,
          comments
        }
  }

  function parseTypeSystemExtension(): TypeSystemExtensionNode {
    const { comments } = takeToken('NAME', 'extend')

    const {
      token: { value }
    } = assertToken('NAME')
    switch (value) {
      case 'schema':
        return parseSchemaDefinition(comments)
      case 'scalar':
        return parseScalarTypeDefinition(comments)
      case 'type':
        return parseObjectTypeDefinition(comments)
      case 'interface':
        return parseInterfaceTypeDefinition(comments)
      case 'union':
        return parseUnionTypeDefinition(comments)
      case 'enum':
        return parseEnumTypeDefinition(comments)
      case 'input':
        return parseInputObjectTypeDefinition(comments)
      default:
        throw new Error(`Unexpected token "${value}"`)
    }
  }

  function parseDefinition(): DefinitionNode {
    if (isNextPunctuator('{')) {
      return {
        kind: 'OperationDefinition',
        operation: 'query',
        name: null,
        variableDefinitionSet: null,
        directives: [],
        selectionSet: parseSelectionSet(false),
        comments: []
      }
    }

    const description = parseDescription()

    const {
      token: { value }
    } = assertToken('NAME')
    switch (value) {
      case 'query':
      case 'mutation':
      case 'subscription':
        if (description !== null)
          throw new Error(`Unexpected token "${description}"`)
        const operation = parseOperationType()
        const name = isNext('NAME') ? parseName() : null
        const definitions = takeWrappedList<AstNodes['VariableDefinition']>(
          true,
          '(',
          ')',
          () => {
            const variable = parseVariable()
            const colon = takePunctuator(':')
            const type = parseType()
            const defaultValue = parseDefaultValue()
            const directives = parseDirectives(true)
            const comments = [...variable.comments, ...colon.comments]
            variable.comments = []
            return {
              kind: 'VariableDefinition',
              variable,
              type,
              defaultValue,
              directives,
              comments
            }
          }
        )
        const directives = parseDirectives(false)
        const selectionSet = parseSelectionSet(false)
        const comments = [...operation.comments, ...(name ? name.comments : [])]
        return {
          kind: 'OperationDefinition',
          operation: operation.value,
          name: name ? name.node : null,
          variableDefinitionSet:
            definitions.items.length > 0
              ? {
                  kind: 'VariableDefinitionSet',
                  definitions: definitions.items,
                  commentsOpeningBracket: definitions.commentsOpeningBracket,
                  commentsClosingBracket: definitions.commentsClosingBracket
                }
              : null,
          directives,
          selectionSet,
          comments
        }
      case 'fragment': {
        if (description !== null)
          throw new Error(`Unexpected token "${description}"`)
        const keyword = takeToken('NAME', 'fragment')
        const name = parseName('on')
        const typeCondition = parseTypeCondition(false)
        const directives = parseDirectives(false)
        const selectionSet = parseSelectionSet(false)
        const comments = [
          ...keyword.comments,
          ...name.comments,
          ...typeCondition.comments
        ]
        return {
          kind: 'FragmentDefinition',
          name: name.node,
          typeCondition: typeCondition.type,
          directives,
          selectionSet,
          comments
        }
      }
      case 'schema':
        return parseSchemaDefinition(null, description)
      case 'scalar':
        return parseScalarTypeDefinition(null, description)
      case 'type':
        return parseObjectTypeDefinition(null, description)
      case 'interface':
        return parseInterfaceTypeDefinition(null, description)
      case 'union':
        return parseUnionTypeDefinition(null, description)
      case 'enum':
        return parseEnumTypeDefinition(null, description)
      case 'input':
        return parseInputObjectTypeDefinition(null, description)
      case 'directive': {
        const keyword = takeToken('NAME', 'directive')
        const at = takePunctuator('@')
        const name = parseName()
        const inputValueDefinitionSet = parseInputValueDefinitionSet('(', ')')
        const repeatable = isNext('NAME', 'repeatable') ? (take(), true) : false
        const locations = takeDelimitedList<DirectiveLocationNode>(
          '|',
          { type: 'NAME', value: 'on' },
          (delimiterComments) => {
            const name = takeToken('NAME')
            const value = name.token.value as any
            const comments = [...delimiterComments, ...name.comments]
            if (isExecutableDirectiveLocation(value))
              return { kind: 'ExecutableDirectiveLocation', value, comments }
            if (isTypeSystemDirectiveLocation(value))
              return { kind: 'TypeSystemDirectiveLocation', value, comments }
            throw new Error(`Unexpected token "${value}"`)
          }
        )
        const comments = [...keyword.comments, ...at.comments, ...name.comments]
        return {
          kind: 'DirectiveDefinition',
          description,
          name: name.node,
          inputValueDefinitionSet,
          repeatable,
          locationSet: {
            kind: 'DirectiveLocationSet',
            comments: locations.initializerComments,
            locations: locations.items
          },
          comments
        }
      }
      case 'extend':
        if (description !== null) throw new Error('Unexpected token')
        return parseTypeSystemExtension()
      default:
        throw new Error(`Unexpected token "${value}"`)
    }
  }

  const definitions: DefinitionNode[] = []
  while (peek().token) {
    definitions.push(parseDefinition())
  }

  return { kind: 'Document', definitions, comments: peek().comments }
}
