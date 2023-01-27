import type { AstNode, AstNodes, CommentNode } from './language'
import { traverse } from './traverse'

type Indentation = '+' | '-'

type LineItem =
  | string
  | {
      type: 'soft_line'
      alt: string
      prefix: string
      indentation?: Indentation
    }

type Printed = LineItem | { type: 'hard_line'; indentation?: Indentation }

function hardLine(indentation?: Indentation): Printed {
  return { type: 'hard_line', indentation }
}

function softLine(
  alt: string,
  prefix: string = '',
  indentation?: Indentation
): Printed {
  return { type: 'soft_line', alt, prefix, indentation }
}

export type Stringified<N extends AstNode> = {
  [K in keyof N]: K extends
    | 'comments'
    | 'commentsOpeningBracket'
    | 'commentsClosingBracket'
    ? N[K]
    : N[K] extends boolean | number | string
    ? N[K]
    : N[K] extends any[]
    ? Printed[][]
    : null extends N[K]
    ? Printed[] | null
    : Printed[]
}

type Visitor<N extends AstNode> = (
  node: Stringified<N>,
  key: string | number | null,
  parent: AstNode | ReadonlyArray<AstNode> | null
) => Printed[]

type VisitorMap = Omit<
  {
    [K in keyof AstNodes]: { leave: Visitor<AstNodes[K]> }
  },
  'BlockComment' | 'InlineComment'
>

export function print(
  ast: AstNode | AstNode[],
  {
    indentationStep = '  ',
    maxLineLength = 80,
    preserveComments = false,
    pretty = false
  }: {
    indentationStep?: string
    maxLineLength?: number
    preserveComments?: boolean
    pretty?: boolean
  } = {}
): string {
  const SPACE = pretty ? ' ' : ''

  function printComment(comment: CommentNode) {
    return preserveComments
      ? [
          ...join(
            comment.value.split('\n').map((line) => ['#', SPACE, line]),
            [hardLine()]
          ),
          hardLine()
        ]
      : []
  }

  function printComments(comments: CommentNode[]) {
    const before: Printed[] = []
    const after: Printed[] = []
    for (let i = 0; i < comments.length; i++) {
      const comment = comments[i]
      if (comment.kind === 'BlockComment') {
        before.push(...printComment(comment))
      } else {
        after.unshift(...printComment(comment))
      }
    }
    if (before.length === 0 && after.length === 0) return []
    return [hardLine(), ...before, ...after]
  }

  function printWrappedListWithComments(
    list: Printed[][],
    openingBracketPunctuator: string,
    spacer: string,
    delimiter: string,
    closingBracketPunctuator: string,
    commentsOpeningBracket: CommentNode[],
    commentsClosingBracket: CommentNode[],
    forceMultiLine: boolean = false
  ) {
    const openingBracket = printComments(commentsOpeningBracket)
    const closingBracket = printComments(commentsClosingBracket)

    const shouldPrintMultiLine =
      pretty &&
      (forceMultiLine || hasHardLine(list) || closingBracket.length > 0)

    return [
      ...openingBracket,
      openingBracketPunctuator,
      shouldPrintMultiLine ? hardLine('+') : softLine(spacer, undefined, '+'),
      ...join(list, [shouldPrintMultiLine ? hardLine() : softLine(delimiter)]),
      shouldPrintMultiLine
        ? hardLine('-')
        : softLine(closingBracket.length > 0 ? '\n' : spacer, undefined, '-'),
      ...closingBracket.slice(1),
      closingBracketPunctuator
    ]
  }

  function withSpace(list: Printed[] | null) {
    return list && list.length > 0 ? [SPACE, ...list] : []
  }

  function printDescription(
    description: Printed[] | null,
    comments: CommentNode[]
  ) {
    return description
      ? [
          ...description,
          pretty && (!preserveComments || comments.length === 0)
            ? hardLine()
            : ''
        ]
      : []
  }

  const visitorMap: VisitorMap = {
    Argument: {
      leave: (node) => [
        ...printComments(node.comments),
        ...node.name,
        ':',
        SPACE,
        ...node.value
      ]
    },
    ArgumentSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.args,
          '(',
          '',
          ',' + SPACE,
          ')',
          node.commentsOpeningBracket,
          node.commentsClosingBracket
        )
    },
    BooleanValue: {
      leave: (node) => [...printComments(node.comments), '' + node.value]
    },
    Directive: {
      leave: (node) => [
        ...printComments(node.comments),
        '@',
        ...node.name,
        ...(node.argumentSet || [])
      ]
    },
    DirectiveDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'directive',
        SPACE,
        '@',
        ...node.name,
        ...(node.inputValueDefinitionSet || []),
        node.repeatable ? ' repeatable ' : ' ',
        ...node.locationSet
      ]
    },
    DirectiveLocationSet: {
      leave: (node) =>
        hasHardLine(node.locations)
          ? [
              ...printComments(node.comments),
              'on',
              hardLine(),
              ...node.locations.flatMap((location, index) => [
                !pretty && index === 0 ? '' : '|',
                SPACE,
                ...location,
                pretty ? hardLine() : softLine('')
              ])
            ]
          : [
              ...printComments(node.comments),
              'on',
              softLine(' ', '|' + SPACE),
              ...join(node.locations, [softLine(SPACE), '|', SPACE]),
              softLine('')
            ]
    },
    Document: {
      leave: (node) => {
        const comments = printComments(node.comments)
        return [
          ...join(
            node.definitions.map((definition) => {
              while (
                typeof definition[0] === 'object' &&
                definition[0].type === 'hard_line'
              )
                definition.shift()
              return definition
            }),
            [hardLine(), pretty ? hardLine() : '']
          ),
          pretty && comments.length > 0 ? hardLine() : '',
          ...comments
        ]
      }
    },
    EnumTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'enum ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.valueDefinitionSet)
      ]
    },
    EnumTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend enum ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.valueDefinitionSet)
      ]
    },
    EnumValue: {
      leave: (node) => [...printComments(node.comments), node.value]
    },
    EnumValueDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        ...node.name,
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    EnumValueDefinitionSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.definitions,
          '{',
          '',
          ',',
          '}',
          node.commentsOpeningBracket,
          node.commentsClosingBracket,
          true
        )
    },
    ExecutableDirectiveLocation: {
      leave: (node) => [...printComments(node.comments), node.value]
    },
    Field: {
      leave: (node) => [
        ...printComments(node.comments),
        ...(node.alias ? [...node.alias, ':', SPACE] : []),
        ...node.name,
        ...(node.argumentSet || []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.selectionSet)
      ]
    },
    FieldDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        ...node.name,
        ...(node.inputValueDefinitionSet || []),
        ':',
        SPACE,
        ...node.type,
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    FieldDefinitionSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.definitions,
          '{',
          '',
          ',',
          '}',
          node.commentsOpeningBracket,
          node.commentsClosingBracket,
          true
        )
    },
    FloatValue: {
      leave: (node) => [...printComments(node.comments), node.value]
    },
    FragmentDefinition: {
      leave: (node) => [
        ...printComments(node.comments),
        'fragment ',
        ...node.name,
        ...(node.typeCondition ? [' on ', ...node.typeCondition] : []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.selectionSet)
      ]
    },
    FragmentSpread: {
      leave: (node) => [
        ...printComments(node.comments),
        '...',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    InlineFragment: {
      leave: (node) => [
        ...printComments(node.comments),
        '...',
        ...(node.typeCondition ? ['on ', ...node.typeCondition] : []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.selectionSet)
      ]
    },
    InputObjectTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'input ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.inputValueDefinitionSet)
      ]
    },
    InputObjectTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend input ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.inputValueDefinitionSet)
      ]
    },
    InputValueDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        ...node.name,
        ':',
        SPACE,
        ...node.type,
        ...(node.defaultValue ? [SPACE, '=', SPACE, ...node.defaultValue] : []),
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    InputValueDefinitionSet: {
      leave: (node, _key, parent) => {
        const [startPunctuator, endPunctuator, forceMultiline] =
          isSingleNode(parent) &&
          (parent.kind === 'DirectiveDefinition' ||
            parent.kind === 'FieldDefinition')
            ? ['(', ')', false]
            : ['{', '}', true]
        return printWrappedListWithComments(
          node.definitions,
          startPunctuator,
          '',
          ',',
          endPunctuator,
          node.commentsOpeningBracket,
          node.commentsClosingBracket,
          forceMultiline
        )
      }
    },
    InterfaceTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'interface ',
        ...node.name,
        ...(node.interfaces || []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.fieldDefinitionSet)
      ]
    },
    InterfaceTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend interface ',
        ...node.name,
        ...(node.interfaces || []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.fieldDefinitionSet)
      ]
    },
    IntValue: {
      leave: (node) => [...printComments(node.comments), node.value]
    },
    ListType: {
      leave: (node) => [...printComments(node.comments), '[', ...node.type, ']']
    },
    ListValue: {
      leave: (node) =>
        printWrappedListWithComments(
          node.values,
          '[',
          '',
          ',' + SPACE,
          ']',
          node.commentsOpeningBracket,
          node.commentsClosingBracket
        )
    },
    Name: { leave: (node) => [node.value] },
    NamedType: {
      leave: (node) => [...printComments(node.comments), ...node.name]
    },
    NamedTypeSet: {
      leave: (node, _key, parent) => {
        const useHardLines = hasHardLine(node.types)
        const [
          initializer = '',
          beforeInitializer = '',
          afterInitializer = [],
          delimiter = [useHardLines ? hardLine() : softLine(',' + SPACE)]
        ] = isSingleNode(parent)
          ? parent.kind === 'ObjectTypeDefinition' ||
            parent.kind === 'ObjectTypeExtension' ||
            parent.kind === 'InterfaceTypeDefinition' ||
            parent.kind === 'InterfaceTypeExtension'
            ? [
                'implements',
                ' ',
                useHardLines
                  ? [hardLine(), '&' + SPACE]
                  : [softLine(' ', '&' + SPACE)],
                [useHardLines ? hardLine() : softLine(SPACE), '&', SPACE]
              ]
            : parent.kind === 'UnionTypeDefinition' ||
              parent.kind === 'UnionTypeExtension'
            ? [
                '=',
                SPACE,
                useHardLines
                  ? [hardLine(), '|' + SPACE]
                  : [softLine(SPACE, '|' + SPACE)],
                [useHardLines ? hardLine() : softLine(SPACE), '|', SPACE]
              ]
            : []
          : []

        const comments = printComments(node.comments)
        return [
          ...comments,
          comments.length > 0 ? '' : beforeInitializer,
          initializer,
          ...afterInitializer,
          ...join(node.types, delimiter)
        ]
      }
    },
    NonNullType: {
      leave: (node) => [...printComments(node.comments), ...node.type, '!']
    },
    NullValue: {
      leave: (node) => [...printComments(node.comments), 'null']
    },
    ObjectField: {
      leave: (node) => [
        ...printComments(node.comments),
        ...node.name,
        ':',
        SPACE,
        ...node.value
      ]
    },
    ObjectTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'type ',
        ...node.name,
        ...(node.interfaces || []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.fieldDefinitionSet)
      ]
    },
    ObjectTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend type ',
        ...node.name,
        ...(node.interfaces || []),
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.fieldDefinitionSet)
      ]
    },
    ObjectValue: {
      leave: (node) =>
        printWrappedListWithComments(
          node.fields,
          '{',
          SPACE,
          ',' + SPACE,
          '}',
          node.commentsOpeningBracket,
          node.commentsClosingBracket
        )
    },
    OperationDefinition: {
      leave: (node) =>
        !node.operation &&
        !node.name &&
        !node.variableDefinitionSet &&
        node.directives.length === 0
          ? node.selectionSet
          : [
              ...printComments(node.comments),
              node.operation,
              ...(node.name ? [' ', ...node.name] : []),
              ...(node.variableDefinitionSet || []),
              ...withSpace(join(node.directives, [SPACE])),
              ...withSpace(node.selectionSet)
            ]
    },
    OperationTypeDefinition: {
      leave: (node) => [
        ...printComments(node.comments),
        node.operation,
        ':',
        SPACE,
        ...node.type
      ]
    },
    OperationTypeDefinitionSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.definitions,
          '{',
          '',
          ',',
          '}',
          node.commentsOpeningBracket,
          node.commentsClosingBracket,
          true
        )
    },
    ScalarTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'scalar ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    ScalarTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend scalar ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    SchemaDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'schema',
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.operationTypeDefinitionSet)
      ]
    },
    SchemaExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend schema',
        ...withSpace(join(node.directives, [SPACE])),
        ...withSpace(node.operationTypeDefinitionSet)
      ]
    },
    SelectionSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.selections,
          '{',
          '',
          ',',
          '}',
          node.commentsOpeningBracket,
          node.commentsClosingBracket,
          true
        )
    },
    StringValue: {
      leave: (node) => [
        ...printComments(node.comments),
        ...(node.block
          ? [
              '"""',
              hardLine(),
              node.value.replace(/"""/g, '\\"""'),
              hardLine(),
              '"""'
            ]
          : [JSON.stringify(node.value)])
      ]
    },
    TypeSystemDirectiveLocation: {
      leave: (node) => [...printComments(node.comments), node.value]
    },
    UnionTypeDefinition: {
      leave: (node) => [
        ...printDescription(node.description, node.comments),
        ...printComments(node.comments),
        'union ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...(node.types || [])
      ]
    },
    UnionTypeExtension: {
      leave: (node) => [
        ...printComments(node.comments),
        'extend union ',
        ...node.name,
        ...withSpace(join(node.directives, [SPACE])),
        ...(node.types || [])
      ]
    },
    Variable: {
      leave: (node) => [...printComments(node.comments), '$', ...node.name]
    },
    VariableDefinition: {
      leave: (node) => [
        ...printComments(node.comments),
        ...node.variable,
        ':',
        SPACE,
        ...node.type,
        ...(node.defaultValue ? [SPACE, '=', SPACE, ...node.defaultValue] : []),
        ...withSpace(join(node.directives, [SPACE]))
      ]
    },
    VariableDefinitionSet: {
      leave: (node) =>
        printWrappedListWithComments(
          node.definitions,
          '(',
          '',
          ',' + SPACE,
          ')',
          node.commentsOpeningBracket,
          node.commentsClosingBracket
        )
    }
  }

  const list = traverse<AstNode | AstNode[], Printed[]>(ast, {
    BlockComment: {
      leave(node, _key, _parent, path) {
        return path.length > 1 ? node : printComment(node)
      }
    },
    InlineComment: {
      leave(node, _key, _parent, path) {
        return path.length > 1 ? node : printComment(node)
      }
    },
    ...(visitorMap as any)
  })

  let printed = ''
  let currentLine: LineItem[] = []
  let indentation = ''

  function handleIndentation(i?: Indentation) {
    if (i === '+') indentation += indentationStep
    if (i === '-') indentation = indentation.slice(indentationStep.length)
  }

  function printLine(list: LineItem[], breakLines: boolean) {
    let printed = ''
    for (let i = 0; i < list.length; i++) {
      const item = list[i]
      if (typeof item === 'string') {
        printed += item
      } else if (item.type === 'soft_line') {
        if (breakLines) {
          handleIndentation(item.indentation)
          printed += '\n' + indentation + item.prefix
        } else {
          printed += item.alt
        }
      }
    }
    return printed
  }

  function printCurrentLine() {
    const printedLine = printLine(currentLine, false)
    if (!pretty || printedLine.length <= maxLineLength) {
      printed += printedLine
    } else {
      printed += printLine(currentLine, true)
    }

    currentLine = []
  }

  for (const item of list.flat()) {
    if (typeof item === 'object' && item.type === 'hard_line') {
      printCurrentLine()
      handleIndentation(item.indentation)
      printed += '\n' + indentation
    } else {
      currentLine.push(item)
    }
  }
  printCurrentLine()
  return printed.replace(/^\n*/, '').replace(/\n*$/, pretty ? '\n' : '')
}

function isSingleNode(
  node: AstNode | ReadonlyArray<AstNode> | null
): node is AstNode {
  return node ? !Array.isArray(node) : false
}

function join(list: Printed[] | Printed[][], delimiter: Printed[]) {
  const joined: Printed[] = []
  for (let i = 0; i < list.length; i++) {
    if (i > 0) joined.push(...delimiter)
    const item = list[i]
    joined.push(...(Array.isArray(item) ? item : [item]))
  }
  return joined
}

function hasHardLine(list: Printed[][]) {
  for (let i = 0; i < list.length; i++)
    for (let j = 0; j < list[i].length; j++) {
      const item = list[i][j]
      if (typeof item === 'object' && item.type === 'hard_line') return true
    }
  return false
}
