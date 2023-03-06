export type Token = LexicalToken | CommentToken

export type LexicalToken = {
  type: typeof LEXICAL_TOKENS[number]['t']
  value: string
}

export type CommentToken = {
  type: typeof COMMENTS[number]['t']
  value: string
}

export function* tokenize(_source: string): IterableIterator<Token> {
  /**
   * We prepend the source string with a line feed so that full-line comments
   * at the beginning of the source string are picked up by the regular
   * expression correctly. This does not affect the output in any other way as
   * the line feed character is an ignored token on its own.
   */
  let source = '\n' + _source

  let match: RegExpMatchArray | null = null
  nextToken: while (source !== '') {
    /**
     * IMPORTANT: Check comments before ignored tokens, because full-line
     * comments assert for a line break at the beginning of the current
     * source.
     */
    for (const { r, t, v } of COMMENTS) {
      if ((match = source.match(r))) {
        source = source.substring(match[0].length)
        yield { type: t, value: v(match[0]) }
        continue nextToken
      }
    }
    for (const regex of IGNORED_TOKENS)
      if ((match = source.match(regex))) {
        source = source.substring(match[0].length)
        continue nextToken
      }
    for (const { r, t, v } of LEXICAL_TOKENS)
      if ((match = source.match(r))) {
        source = source.substring(match[0].length)
        yield { type: t, value: v(match[0]) }
        continue nextToken
      }
    throw new Error('Syntax error: ' + displaySource(source))
  }
}

/**
 * Comments also are ignored tokens according to the spec. But compared to all
 * other ignored tokens, comments can carry information. That's why we treat
 * them differently. In particular this enabled persisting comments when
 * printing an AST.
 *
 * Comments usually refer to different items depending on their position:
 * - If a comment is the only non-ignored token in the whole line, then it
 *   usually refers to the token _after_ it.
 * - If there are other non-ignored tokens in the same line as the comment,
 *   then it usually refers to the token _before_ it.
 *
 * Example source:
 *
 * query Example {
 *   id # This comment tells something about the id field.
 *   name
 *   # This comment tells something about the email field.
 *   # It can also span accross multiple lines
 *   email
 * }
 */
const COMMENTS = [
  /** Full-line */
  {
    r: /^((\n|\r(?!\n)|\r\n)[\t ]*#.*)+/,
    t: 'BLOCK_COMMENT',
    v: (input: string) =>
      normalizeIndentation(
        input
          .split(/\n|\r(?!\n)|\r\n/g)
          .map((line) => line.replace(/^[\t ]*#/, '').replace(/[\t ]*$/, ''))
      )
  },
  /** Inline */
  {
    r: /^#.*/,
    t: 'INLINE_COMMENT',
    v: (input: string) => input.replace(/^#[\t ]*/, '').replace(/[\t ]*$/, '')
  }
] as const

const IGNORED_TOKENS = [
  /** Unicode BOM */
  /^\ufeff/,
  /** White Space */
  /^[\t ]+/,
  /** Line Terminators */
  /^\n|\r(?!\n)|\r\n/,
  /** Insignificant Commas */
  /^,+/
] as const

const noop = (input: string) => input

const LEXICAL_TOKENS = [
  /** Punctuators */
  { r: /^([!$&():=@\[\]{\|}]|\.\.\.)/, t: 'PUNCTUATOR', v: noop },
  /** Name */
  { r: /^[_a-zA-Z][_a-zA-Z0-9]*/, t: 'NAME', v: noop },
  /** IntValue */
  { r: /^-?(0|[1-9])[0-9]*(?![\._a-zA-Z0-9])/, t: 'INT_VALUE', v: noop },
  /** FloatValue */
  {
    r: /^-?(0|[1-9][0-9]*)(\.[0-9]+[eE][+-]?[0-9]+|\.[0-9]+|[eE][+-]?[0-9]+)(?![\._a-zA-Z0-9])/,
    t: 'FLOAT_VALUE',
    v: noop
  },
  /**
   * StringValue (block string)
   *
   * IMPORTANT: check block strings before regular strings so that six quotes
   * are evaluated as empty block string and not three empty regular strings
   */
  {
    r: /^"""([^"\\]|"[^"]|""[^"]|\\[^"]|\\"[^"]|\\""[^"]|\\""")*"""/s,
    t: 'BLOCK_STRING_VALUE',
    v: (input: string) =>
      normalizeIndentation(
        input
          .substring(3, input.length - 3)
          .replace(/\\"""/g, '"""')
          .split(/\n|\r(?!\n)|\r\n/g)
      )
  },
  /** StringValue */
  {
    r: /^"([^"\\]|\\u{[0-9a-fA-F]+}|\\u[0-9a-fA-F]{4}|\\["\\\/bfnrt])*"/,
    t: 'STRING_VALUE',
    v: (input: string) => {
      let output = input.substring(1, input.length - 1)
      for (const { r, f } of STRING_SEMANTICS) {
        output = output.replace(r, f)
      }
      return output
    }
  }
] as const

function displaySource(source: string) {
  const currentLine = source.split('\n')[0]
  return currentLine.length > 20
    ? currentLine.substring(0, 20) + '...'
    : currentLine
}

const STRING_SEMANTICS = [
  /** variable width unicode characters */
  {
    r: /\\u{[0-9a-fA-F]+}/g,
    f: (match: string) => {
      const charCode = parseInt(match.substring(3, match.length - 1), 16)
      if (!isValidCharCode(charCode))
        throw new Error('Syntax error: Invalid char code ' + match)
      return String.fromCharCode(charCode)
    }
  },
  /** supplementary characters */
  {
    r: /\\u[0-9a-fA-F]{4}\\u[0-9a-fA-F]{4}/g,
    f: (match: string) => {
      const leading = parseInt(match.substring(2, 6), 16)
      const trailing = parseInt(match.substring(8, 12), 16)
      if (leading >= 0xd800 && leading <= 0xdbff) {
        if (trailing < 0xdc00 || trailing > 0xdfff)
          throw new Error(
            'Syntax error: Invalid supplementary character ' + match
          )
        return String.fromCharCode(
          (leading - 0xd800) * 0x400 + (trailing - 0xdc00) + 0x10000
        )
      }
      // Parse both character individually
      return match
    }
  },
  /** fixed width unicode characters */
  {
    r: /\\u[0-9a-fA-F]{4}/g,
    f: (match: string) => {
      const charCode = parseInt(match.substring(2), 16)
      if (!isValidCharCode(charCode))
        throw new Error('Syntax error: Invalid char code ' + match)
      return String.fromCharCode(charCode)
    }
  },
  /** escaped characters */
  {
    r: /\\["\\\/bfnrt]/g,
    f: (match: string) => {
      const escaped = match[1]
      return ESCAPED_CHARACTERS[escaped] || escaped
    }
  }
] as const

const ESCAPED_CHARACTERS: Record<string, string> = {
  b: '\b',
  f: '\f',
  n: '\n',
  r: '\r',
  t: '\t'
}

function isValidCharCode(charCode: number) {
  return (
    Number.isInteger(charCode) &&
    (isBetween(charCode, 0x0000, 0xd7ff) ||
      isBetween(charCode, 0xe000, 0x10ffff))
  )
}

function isBetween(n: number, lower: number, upper: number) {
  return n >= lower && n <= upper
}

function normalizeIndentation(lines: string[]) {
  let commonIndent: number | null = null

  for (let i = 1; i < lines.length; i++) {
    const line = lines[i]

    const leadingWhitespace = line.match(/^[\t ]*/)
    const indent = leadingWhitespace ? leadingWhitespace[0].length : 0

    if (
      indent < line.length &&
      (commonIndent === null || indent < commonIndent)
    )
      commonIndent = indent
  }

  if (commonIndent && commonIndent > 0)
    for (let i = 1; i < lines.length; i++)
      lines[i] = lines[i].slice(commonIndent)

  return lines
    .join('\n')
    .replace(/^[\t\n ]*/, '')
    .replace(/[\t\n ]*$/, '')
}
