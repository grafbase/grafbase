var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __esm = (fn, res) => function __init() {
  return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
};
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// node_modules/picomatch/lib/constants.js
var require_constants = __commonJS({
  "node_modules/picomatch/lib/constants.js"(exports2, module2) {
    "use strict";
    var path4 = require("path");
    var WIN_SLASH = "\\\\/";
    var WIN_NO_SLASH = `[^${WIN_SLASH}]`;
    var DOT_LITERAL = "\\.";
    var PLUS_LITERAL = "\\+";
    var QMARK_LITERAL = "\\?";
    var SLASH_LITERAL = "\\/";
    var ONE_CHAR = "(?=.)";
    var QMARK = "[^/]";
    var END_ANCHOR = `(?:${SLASH_LITERAL}|$)`;
    var START_ANCHOR = `(?:^|${SLASH_LITERAL})`;
    var DOTS_SLASH = `${DOT_LITERAL}{1,2}${END_ANCHOR}`;
    var NO_DOT = `(?!${DOT_LITERAL})`;
    var NO_DOTS = `(?!${START_ANCHOR}${DOTS_SLASH})`;
    var NO_DOT_SLASH = `(?!${DOT_LITERAL}{0,1}${END_ANCHOR})`;
    var NO_DOTS_SLASH = `(?!${DOTS_SLASH})`;
    var QMARK_NO_DOT = `[^.${SLASH_LITERAL}]`;
    var STAR = `${QMARK}*?`;
    var POSIX_CHARS = {
      DOT_LITERAL,
      PLUS_LITERAL,
      QMARK_LITERAL,
      SLASH_LITERAL,
      ONE_CHAR,
      QMARK,
      END_ANCHOR,
      DOTS_SLASH,
      NO_DOT,
      NO_DOTS,
      NO_DOT_SLASH,
      NO_DOTS_SLASH,
      QMARK_NO_DOT,
      STAR,
      START_ANCHOR
    };
    var WINDOWS_CHARS = {
      ...POSIX_CHARS,
      SLASH_LITERAL: `[${WIN_SLASH}]`,
      QMARK: WIN_NO_SLASH,
      STAR: `${WIN_NO_SLASH}*?`,
      DOTS_SLASH: `${DOT_LITERAL}{1,2}(?:[${WIN_SLASH}]|$)`,
      NO_DOT: `(?!${DOT_LITERAL})`,
      NO_DOTS: `(?!(?:^|[${WIN_SLASH}])${DOT_LITERAL}{1,2}(?:[${WIN_SLASH}]|$))`,
      NO_DOT_SLASH: `(?!${DOT_LITERAL}{0,1}(?:[${WIN_SLASH}]|$))`,
      NO_DOTS_SLASH: `(?!${DOT_LITERAL}{1,2}(?:[${WIN_SLASH}]|$))`,
      QMARK_NO_DOT: `[^.${WIN_SLASH}]`,
      START_ANCHOR: `(?:^|[${WIN_SLASH}])`,
      END_ANCHOR: `(?:[${WIN_SLASH}]|$)`
    };
    var POSIX_REGEX_SOURCE = {
      alnum: "a-zA-Z0-9",
      alpha: "a-zA-Z",
      ascii: "\\x00-\\x7F",
      blank: " \\t",
      cntrl: "\\x00-\\x1F\\x7F",
      digit: "0-9",
      graph: "\\x21-\\x7E",
      lower: "a-z",
      print: "\\x20-\\x7E ",
      punct: "\\-!\"#$%&'()\\*+,./:;<=>?@[\\]^_`{|}~",
      space: " \\t\\r\\n\\v\\f",
      upper: "A-Z",
      word: "A-Za-z0-9_",
      xdigit: "A-Fa-f0-9"
    };
    module2.exports = {
      MAX_LENGTH: 1024 * 64,
      POSIX_REGEX_SOURCE,
      // regular expressions
      REGEX_BACKSLASH: /\\(?![*+?^${}(|)[\]])/g,
      REGEX_NON_SPECIAL_CHARS: /^[^@![\].,$*+?^{}()|\\/]+/,
      REGEX_SPECIAL_CHARS: /[-*+?.^${}(|)[\]]/,
      REGEX_SPECIAL_CHARS_BACKREF: /(\\?)((\W)(\3*))/g,
      REGEX_SPECIAL_CHARS_GLOBAL: /([-*+?.^${}(|)[\]])/g,
      REGEX_REMOVE_BACKSLASH: /(?:\[.*?[^\\]\]|\\(?=.))/g,
      // Replace globs with equivalent patterns to reduce parsing time.
      REPLACEMENTS: {
        "***": "*",
        "**/**": "**",
        "**/**/**": "**"
      },
      // Digits
      CHAR_0: 48,
      /* 0 */
      CHAR_9: 57,
      /* 9 */
      // Alphabet chars.
      CHAR_UPPERCASE_A: 65,
      /* A */
      CHAR_LOWERCASE_A: 97,
      /* a */
      CHAR_UPPERCASE_Z: 90,
      /* Z */
      CHAR_LOWERCASE_Z: 122,
      /* z */
      CHAR_LEFT_PARENTHESES: 40,
      /* ( */
      CHAR_RIGHT_PARENTHESES: 41,
      /* ) */
      CHAR_ASTERISK: 42,
      /* * */
      // Non-alphabetic chars.
      CHAR_AMPERSAND: 38,
      /* & */
      CHAR_AT: 64,
      /* @ */
      CHAR_BACKWARD_SLASH: 92,
      /* \ */
      CHAR_CARRIAGE_RETURN: 13,
      /* \r */
      CHAR_CIRCUMFLEX_ACCENT: 94,
      /* ^ */
      CHAR_COLON: 58,
      /* : */
      CHAR_COMMA: 44,
      /* , */
      CHAR_DOT: 46,
      /* . */
      CHAR_DOUBLE_QUOTE: 34,
      /* " */
      CHAR_EQUAL: 61,
      /* = */
      CHAR_EXCLAMATION_MARK: 33,
      /* ! */
      CHAR_FORM_FEED: 12,
      /* \f */
      CHAR_FORWARD_SLASH: 47,
      /* / */
      CHAR_GRAVE_ACCENT: 96,
      /* ` */
      CHAR_HASH: 35,
      /* # */
      CHAR_HYPHEN_MINUS: 45,
      /* - */
      CHAR_LEFT_ANGLE_BRACKET: 60,
      /* < */
      CHAR_LEFT_CURLY_BRACE: 123,
      /* { */
      CHAR_LEFT_SQUARE_BRACKET: 91,
      /* [ */
      CHAR_LINE_FEED: 10,
      /* \n */
      CHAR_NO_BREAK_SPACE: 160,
      /* \u00A0 */
      CHAR_PERCENT: 37,
      /* % */
      CHAR_PLUS: 43,
      /* + */
      CHAR_QUESTION_MARK: 63,
      /* ? */
      CHAR_RIGHT_ANGLE_BRACKET: 62,
      /* > */
      CHAR_RIGHT_CURLY_BRACE: 125,
      /* } */
      CHAR_RIGHT_SQUARE_BRACKET: 93,
      /* ] */
      CHAR_SEMICOLON: 59,
      /* ; */
      CHAR_SINGLE_QUOTE: 39,
      /* ' */
      CHAR_SPACE: 32,
      /*   */
      CHAR_TAB: 9,
      /* \t */
      CHAR_UNDERSCORE: 95,
      /* _ */
      CHAR_VERTICAL_LINE: 124,
      /* | */
      CHAR_ZERO_WIDTH_NOBREAK_SPACE: 65279,
      /* \uFEFF */
      SEP: path4.sep,
      /**
       * Create EXTGLOB_CHARS
       */
      extglobChars(chars) {
        return {
          "!": { type: "negate", open: "(?:(?!(?:", close: `))${chars.STAR})` },
          "?": { type: "qmark", open: "(?:", close: ")?" },
          "+": { type: "plus", open: "(?:", close: ")+" },
          "*": { type: "star", open: "(?:", close: ")*" },
          "@": { type: "at", open: "(?:", close: ")" }
        };
      },
      /**
       * Create GLOB_CHARS
       */
      globChars(win32) {
        return win32 === true ? WINDOWS_CHARS : POSIX_CHARS;
      }
    };
  }
});

// node_modules/picomatch/lib/utils.js
var require_utils = __commonJS({
  "node_modules/picomatch/lib/utils.js"(exports2) {
    "use strict";
    var path4 = require("path");
    var win32 = process.platform === "win32";
    var {
      REGEX_BACKSLASH,
      REGEX_REMOVE_BACKSLASH,
      REGEX_SPECIAL_CHARS,
      REGEX_SPECIAL_CHARS_GLOBAL
    } = require_constants();
    exports2.isObject = (val) => val !== null && typeof val === "object" && !Array.isArray(val);
    exports2.hasRegexChars = (str) => REGEX_SPECIAL_CHARS.test(str);
    exports2.isRegexChar = (str) => str.length === 1 && exports2.hasRegexChars(str);
    exports2.escapeRegex = (str) => str.replace(REGEX_SPECIAL_CHARS_GLOBAL, "\\$1");
    exports2.toPosixSlashes = (str) => str.replace(REGEX_BACKSLASH, "/");
    exports2.removeBackslashes = (str) => {
      return str.replace(REGEX_REMOVE_BACKSLASH, (match) => {
        return match === "\\" ? "" : match;
      });
    };
    exports2.supportsLookbehinds = () => {
      const segs = process.version.slice(1).split(".").map(Number);
      if (segs.length === 3 && segs[0] >= 9 || segs[0] === 8 && segs[1] >= 10) {
        return true;
      }
      return false;
    };
    exports2.isWindows = (options) => {
      if (options && typeof options.windows === "boolean") {
        return options.windows;
      }
      return win32 === true || path4.sep === "\\";
    };
    exports2.escapeLast = (input, char, lastIdx) => {
      const idx = input.lastIndexOf(char, lastIdx);
      if (idx === -1)
        return input;
      if (input[idx - 1] === "\\")
        return exports2.escapeLast(input, char, idx - 1);
      return `${input.slice(0, idx)}\\${input.slice(idx)}`;
    };
    exports2.removePrefix = (input, state = {}) => {
      let output = input;
      if (output.startsWith("./")) {
        output = output.slice(2);
        state.prefix = "./";
      }
      return output;
    };
    exports2.wrapOutput = (input, state = {}, options = {}) => {
      const prepend = options.contains ? "" : "^";
      const append = options.contains ? "" : "$";
      let output = `${prepend}(?:${input})${append}`;
      if (state.negated === true) {
        output = `(?:^(?!${output}).*$)`;
      }
      return output;
    };
  }
});

// node_modules/picomatch/lib/scan.js
var require_scan = __commonJS({
  "node_modules/picomatch/lib/scan.js"(exports2, module2) {
    "use strict";
    var utils = require_utils();
    var {
      CHAR_ASTERISK,
      /* * */
      CHAR_AT,
      /* @ */
      CHAR_BACKWARD_SLASH,
      /* \ */
      CHAR_COMMA,
      /* , */
      CHAR_DOT,
      /* . */
      CHAR_EXCLAMATION_MARK,
      /* ! */
      CHAR_FORWARD_SLASH,
      /* / */
      CHAR_LEFT_CURLY_BRACE,
      /* { */
      CHAR_LEFT_PARENTHESES,
      /* ( */
      CHAR_LEFT_SQUARE_BRACKET,
      /* [ */
      CHAR_PLUS,
      /* + */
      CHAR_QUESTION_MARK,
      /* ? */
      CHAR_RIGHT_CURLY_BRACE,
      /* } */
      CHAR_RIGHT_PARENTHESES,
      /* ) */
      CHAR_RIGHT_SQUARE_BRACKET
      /* ] */
    } = require_constants();
    var isPathSeparator = (code) => {
      return code === CHAR_FORWARD_SLASH || code === CHAR_BACKWARD_SLASH;
    };
    var depth = (token) => {
      if (token.isPrefix !== true) {
        token.depth = token.isGlobstar ? Infinity : 1;
      }
    };
    var scan = (input, options) => {
      const opts = options || {};
      const length = input.length - 1;
      const scanToEnd = opts.parts === true || opts.scanToEnd === true;
      const slashes = [];
      const tokens = [];
      const parts = [];
      let str = input;
      let index = -1;
      let start = 0;
      let lastIndex = 0;
      let isBrace = false;
      let isBracket = false;
      let isGlob = false;
      let isExtglob = false;
      let isGlobstar = false;
      let braceEscaped = false;
      let backslashes = false;
      let negated = false;
      let negatedExtglob = false;
      let finished = false;
      let braces = 0;
      let prev;
      let code;
      let token = { value: "", depth: 0, isGlob: false };
      const eos = () => index >= length;
      const peek = () => str.charCodeAt(index + 1);
      const advance = () => {
        prev = code;
        return str.charCodeAt(++index);
      };
      while (index < length) {
        code = advance();
        let next;
        if (code === CHAR_BACKWARD_SLASH) {
          backslashes = token.backslashes = true;
          code = advance();
          if (code === CHAR_LEFT_CURLY_BRACE) {
            braceEscaped = true;
          }
          continue;
        }
        if (braceEscaped === true || code === CHAR_LEFT_CURLY_BRACE) {
          braces++;
          while (eos() !== true && (code = advance())) {
            if (code === CHAR_BACKWARD_SLASH) {
              backslashes = token.backslashes = true;
              advance();
              continue;
            }
            if (code === CHAR_LEFT_CURLY_BRACE) {
              braces++;
              continue;
            }
            if (braceEscaped !== true && code === CHAR_DOT && (code = advance()) === CHAR_DOT) {
              isBrace = token.isBrace = true;
              isGlob = token.isGlob = true;
              finished = true;
              if (scanToEnd === true) {
                continue;
              }
              break;
            }
            if (braceEscaped !== true && code === CHAR_COMMA) {
              isBrace = token.isBrace = true;
              isGlob = token.isGlob = true;
              finished = true;
              if (scanToEnd === true) {
                continue;
              }
              break;
            }
            if (code === CHAR_RIGHT_CURLY_BRACE) {
              braces--;
              if (braces === 0) {
                braceEscaped = false;
                isBrace = token.isBrace = true;
                finished = true;
                break;
              }
            }
          }
          if (scanToEnd === true) {
            continue;
          }
          break;
        }
        if (code === CHAR_FORWARD_SLASH) {
          slashes.push(index);
          tokens.push(token);
          token = { value: "", depth: 0, isGlob: false };
          if (finished === true)
            continue;
          if (prev === CHAR_DOT && index === start + 1) {
            start += 2;
            continue;
          }
          lastIndex = index + 1;
          continue;
        }
        if (opts.noext !== true) {
          const isExtglobChar = code === CHAR_PLUS || code === CHAR_AT || code === CHAR_ASTERISK || code === CHAR_QUESTION_MARK || code === CHAR_EXCLAMATION_MARK;
          if (isExtglobChar === true && peek() === CHAR_LEFT_PARENTHESES) {
            isGlob = token.isGlob = true;
            isExtglob = token.isExtglob = true;
            finished = true;
            if (code === CHAR_EXCLAMATION_MARK && index === start) {
              negatedExtglob = true;
            }
            if (scanToEnd === true) {
              while (eos() !== true && (code = advance())) {
                if (code === CHAR_BACKWARD_SLASH) {
                  backslashes = token.backslashes = true;
                  code = advance();
                  continue;
                }
                if (code === CHAR_RIGHT_PARENTHESES) {
                  isGlob = token.isGlob = true;
                  finished = true;
                  break;
                }
              }
              continue;
            }
            break;
          }
        }
        if (code === CHAR_ASTERISK) {
          if (prev === CHAR_ASTERISK)
            isGlobstar = token.isGlobstar = true;
          isGlob = token.isGlob = true;
          finished = true;
          if (scanToEnd === true) {
            continue;
          }
          break;
        }
        if (code === CHAR_QUESTION_MARK) {
          isGlob = token.isGlob = true;
          finished = true;
          if (scanToEnd === true) {
            continue;
          }
          break;
        }
        if (code === CHAR_LEFT_SQUARE_BRACKET) {
          while (eos() !== true && (next = advance())) {
            if (next === CHAR_BACKWARD_SLASH) {
              backslashes = token.backslashes = true;
              advance();
              continue;
            }
            if (next === CHAR_RIGHT_SQUARE_BRACKET) {
              isBracket = token.isBracket = true;
              isGlob = token.isGlob = true;
              finished = true;
              break;
            }
          }
          if (scanToEnd === true) {
            continue;
          }
          break;
        }
        if (opts.nonegate !== true && code === CHAR_EXCLAMATION_MARK && index === start) {
          negated = token.negated = true;
          start++;
          continue;
        }
        if (opts.noparen !== true && code === CHAR_LEFT_PARENTHESES) {
          isGlob = token.isGlob = true;
          if (scanToEnd === true) {
            while (eos() !== true && (code = advance())) {
              if (code === CHAR_LEFT_PARENTHESES) {
                backslashes = token.backslashes = true;
                code = advance();
                continue;
              }
              if (code === CHAR_RIGHT_PARENTHESES) {
                finished = true;
                break;
              }
            }
            continue;
          }
          break;
        }
        if (isGlob === true) {
          finished = true;
          if (scanToEnd === true) {
            continue;
          }
          break;
        }
      }
      if (opts.noext === true) {
        isExtglob = false;
        isGlob = false;
      }
      let base = str;
      let prefix = "";
      let glob = "";
      if (start > 0) {
        prefix = str.slice(0, start);
        str = str.slice(start);
        lastIndex -= start;
      }
      if (base && isGlob === true && lastIndex > 0) {
        base = str.slice(0, lastIndex);
        glob = str.slice(lastIndex);
      } else if (isGlob === true) {
        base = "";
        glob = str;
      } else {
        base = str;
      }
      if (base && base !== "" && base !== "/" && base !== str) {
        if (isPathSeparator(base.charCodeAt(base.length - 1))) {
          base = base.slice(0, -1);
        }
      }
      if (opts.unescape === true) {
        if (glob)
          glob = utils.removeBackslashes(glob);
        if (base && backslashes === true) {
          base = utils.removeBackslashes(base);
        }
      }
      const state = {
        prefix,
        input,
        start,
        base,
        glob,
        isBrace,
        isBracket,
        isGlob,
        isExtglob,
        isGlobstar,
        negated,
        negatedExtglob
      };
      if (opts.tokens === true) {
        state.maxDepth = 0;
        if (!isPathSeparator(code)) {
          tokens.push(token);
        }
        state.tokens = tokens;
      }
      if (opts.parts === true || opts.tokens === true) {
        let prevIndex;
        for (let idx = 0; idx < slashes.length; idx++) {
          const n = prevIndex ? prevIndex + 1 : start;
          const i = slashes[idx];
          const value = input.slice(n, i);
          if (opts.tokens) {
            if (idx === 0 && start !== 0) {
              tokens[idx].isPrefix = true;
              tokens[idx].value = prefix;
            } else {
              tokens[idx].value = value;
            }
            depth(tokens[idx]);
            state.maxDepth += tokens[idx].depth;
          }
          if (idx !== 0 || value !== "") {
            parts.push(value);
          }
          prevIndex = i;
        }
        if (prevIndex && prevIndex + 1 < input.length) {
          const value = input.slice(prevIndex + 1);
          parts.push(value);
          if (opts.tokens) {
            tokens[tokens.length - 1].value = value;
            depth(tokens[tokens.length - 1]);
            state.maxDepth += tokens[tokens.length - 1].depth;
          }
        }
        state.slashes = slashes;
        state.parts = parts;
      }
      return state;
    };
    module2.exports = scan;
  }
});

// node_modules/picomatch/lib/parse.js
var require_parse = __commonJS({
  "node_modules/picomatch/lib/parse.js"(exports2, module2) {
    "use strict";
    var constants3 = require_constants();
    var utils = require_utils();
    var {
      MAX_LENGTH,
      POSIX_REGEX_SOURCE,
      REGEX_NON_SPECIAL_CHARS,
      REGEX_SPECIAL_CHARS_BACKREF,
      REPLACEMENTS
    } = constants3;
    var expandRange = (args, options) => {
      if (typeof options.expandRange === "function") {
        return options.expandRange(...args, options);
      }
      args.sort();
      const value = `[${args.join("-")}]`;
      try {
        new RegExp(value);
      } catch (ex) {
        return args.map((v) => utils.escapeRegex(v)).join("..");
      }
      return value;
    };
    var syntaxError = (type, char) => {
      return `Missing ${type}: "${char}" - use "\\\\${char}" to match literal characters`;
    };
    var parse2 = (input, options) => {
      if (typeof input !== "string") {
        throw new TypeError("Expected a string");
      }
      input = REPLACEMENTS[input] || input;
      const opts = { ...options };
      const max = typeof opts.maxLength === "number" ? Math.min(MAX_LENGTH, opts.maxLength) : MAX_LENGTH;
      let len = input.length;
      if (len > max) {
        throw new SyntaxError(`Input length: ${len}, exceeds maximum allowed length: ${max}`);
      }
      const bos = { type: "bos", value: "", output: opts.prepend || "" };
      const tokens = [bos];
      const capture = opts.capture ? "" : "?:";
      const win32 = utils.isWindows(options);
      const PLATFORM_CHARS = constants3.globChars(win32);
      const EXTGLOB_CHARS = constants3.extglobChars(PLATFORM_CHARS);
      const {
        DOT_LITERAL,
        PLUS_LITERAL,
        SLASH_LITERAL,
        ONE_CHAR,
        DOTS_SLASH,
        NO_DOT,
        NO_DOT_SLASH,
        NO_DOTS_SLASH,
        QMARK,
        QMARK_NO_DOT,
        STAR,
        START_ANCHOR
      } = PLATFORM_CHARS;
      const globstar = (opts2) => {
        return `(${capture}(?:(?!${START_ANCHOR}${opts2.dot ? DOTS_SLASH : DOT_LITERAL}).)*?)`;
      };
      const nodot = opts.dot ? "" : NO_DOT;
      const qmarkNoDot = opts.dot ? QMARK : QMARK_NO_DOT;
      let star = opts.bash === true ? globstar(opts) : STAR;
      if (opts.capture) {
        star = `(${star})`;
      }
      if (typeof opts.noext === "boolean") {
        opts.noextglob = opts.noext;
      }
      const state = {
        input,
        index: -1,
        start: 0,
        dot: opts.dot === true,
        consumed: "",
        output: "",
        prefix: "",
        backtrack: false,
        negated: false,
        brackets: 0,
        braces: 0,
        parens: 0,
        quotes: 0,
        globstar: false,
        tokens
      };
      input = utils.removePrefix(input, state);
      len = input.length;
      const extglobs = [];
      const braces = [];
      const stack = [];
      let prev = bos;
      let value;
      const eos = () => state.index === len - 1;
      const peek = state.peek = (n = 1) => input[state.index + n];
      const advance = state.advance = () => input[++state.index] || "";
      const remaining = () => input.slice(state.index + 1);
      const consume = (value2 = "", num = 0) => {
        state.consumed += value2;
        state.index += num;
      };
      const append = (token) => {
        state.output += token.output != null ? token.output : token.value;
        consume(token.value);
      };
      const negate = () => {
        let count = 1;
        while (peek() === "!" && (peek(2) !== "(" || peek(3) === "?")) {
          advance();
          state.start++;
          count++;
        }
        if (count % 2 === 0) {
          return false;
        }
        state.negated = true;
        state.start++;
        return true;
      };
      const increment = (type) => {
        state[type]++;
        stack.push(type);
      };
      const decrement = (type) => {
        state[type]--;
        stack.pop();
      };
      const push = (tok) => {
        if (prev.type === "globstar") {
          const isBrace = state.braces > 0 && (tok.type === "comma" || tok.type === "brace");
          const isExtglob = tok.extglob === true || extglobs.length && (tok.type === "pipe" || tok.type === "paren");
          if (tok.type !== "slash" && tok.type !== "paren" && !isBrace && !isExtglob) {
            state.output = state.output.slice(0, -prev.output.length);
            prev.type = "star";
            prev.value = "*";
            prev.output = star;
            state.output += prev.output;
          }
        }
        if (extglobs.length && tok.type !== "paren") {
          extglobs[extglobs.length - 1].inner += tok.value;
        }
        if (tok.value || tok.output)
          append(tok);
        if (prev && prev.type === "text" && tok.type === "text") {
          prev.value += tok.value;
          prev.output = (prev.output || "") + tok.value;
          return;
        }
        tok.prev = prev;
        tokens.push(tok);
        prev = tok;
      };
      const extglobOpen = (type, value2) => {
        const token = { ...EXTGLOB_CHARS[value2], conditions: 1, inner: "" };
        token.prev = prev;
        token.parens = state.parens;
        token.output = state.output;
        const output = (opts.capture ? "(" : "") + token.open;
        increment("parens");
        push({ type, value: value2, output: state.output ? "" : ONE_CHAR });
        push({ type: "paren", extglob: true, value: advance(), output });
        extglobs.push(token);
      };
      const extglobClose = (token) => {
        let output = token.close + (opts.capture ? ")" : "");
        let rest;
        if (token.type === "negate") {
          let extglobStar = star;
          if (token.inner && token.inner.length > 1 && token.inner.includes("/")) {
            extglobStar = globstar(opts);
          }
          if (extglobStar !== star || eos() || /^\)+$/.test(remaining())) {
            output = token.close = `)$))${extglobStar}`;
          }
          if (token.inner.includes("*") && (rest = remaining()) && /^\.[^\\/.]+$/.test(rest)) {
            const expression = parse2(rest, { ...options, fastpaths: false }).output;
            output = token.close = `)${expression})${extglobStar})`;
          }
          if (token.prev.type === "bos") {
            state.negatedExtglob = true;
          }
        }
        push({ type: "paren", extglob: true, value, output });
        decrement("parens");
      };
      if (opts.fastpaths !== false && !/(^[*!]|[/()[\]{}"])/.test(input)) {
        let backslashes = false;
        let output = input.replace(REGEX_SPECIAL_CHARS_BACKREF, (m, esc, chars, first, rest, index) => {
          if (first === "\\") {
            backslashes = true;
            return m;
          }
          if (first === "?") {
            if (esc) {
              return esc + first + (rest ? QMARK.repeat(rest.length) : "");
            }
            if (index === 0) {
              return qmarkNoDot + (rest ? QMARK.repeat(rest.length) : "");
            }
            return QMARK.repeat(chars.length);
          }
          if (first === ".") {
            return DOT_LITERAL.repeat(chars.length);
          }
          if (first === "*") {
            if (esc) {
              return esc + first + (rest ? star : "");
            }
            return star;
          }
          return esc ? m : `\\${m}`;
        });
        if (backslashes === true) {
          if (opts.unescape === true) {
            output = output.replace(/\\/g, "");
          } else {
            output = output.replace(/\\+/g, (m) => {
              return m.length % 2 === 0 ? "\\\\" : m ? "\\" : "";
            });
          }
        }
        if (output === input && opts.contains === true) {
          state.output = input;
          return state;
        }
        state.output = utils.wrapOutput(output, state, options);
        return state;
      }
      while (!eos()) {
        value = advance();
        if (value === "\0") {
          continue;
        }
        if (value === "\\") {
          const next = peek();
          if (next === "/" && opts.bash !== true) {
            continue;
          }
          if (next === "." || next === ";") {
            continue;
          }
          if (!next) {
            value += "\\";
            push({ type: "text", value });
            continue;
          }
          const match = /^\\+/.exec(remaining());
          let slashes = 0;
          if (match && match[0].length > 2) {
            slashes = match[0].length;
            state.index += slashes;
            if (slashes % 2 !== 0) {
              value += "\\";
            }
          }
          if (opts.unescape === true) {
            value = advance();
          } else {
            value += advance();
          }
          if (state.brackets === 0) {
            push({ type: "text", value });
            continue;
          }
        }
        if (state.brackets > 0 && (value !== "]" || prev.value === "[" || prev.value === "[^")) {
          if (opts.posix !== false && value === ":") {
            const inner = prev.value.slice(1);
            if (inner.includes("[")) {
              prev.posix = true;
              if (inner.includes(":")) {
                const idx = prev.value.lastIndexOf("[");
                const pre = prev.value.slice(0, idx);
                const rest2 = prev.value.slice(idx + 2);
                const posix = POSIX_REGEX_SOURCE[rest2];
                if (posix) {
                  prev.value = pre + posix;
                  state.backtrack = true;
                  advance();
                  if (!bos.output && tokens.indexOf(prev) === 1) {
                    bos.output = ONE_CHAR;
                  }
                  continue;
                }
              }
            }
          }
          if (value === "[" && peek() !== ":" || value === "-" && peek() === "]") {
            value = `\\${value}`;
          }
          if (value === "]" && (prev.value === "[" || prev.value === "[^")) {
            value = `\\${value}`;
          }
          if (opts.posix === true && value === "!" && prev.value === "[") {
            value = "^";
          }
          prev.value += value;
          append({ value });
          continue;
        }
        if (state.quotes === 1 && value !== '"') {
          value = utils.escapeRegex(value);
          prev.value += value;
          append({ value });
          continue;
        }
        if (value === '"') {
          state.quotes = state.quotes === 1 ? 0 : 1;
          if (opts.keepQuotes === true) {
            push({ type: "text", value });
          }
          continue;
        }
        if (value === "(") {
          increment("parens");
          push({ type: "paren", value });
          continue;
        }
        if (value === ")") {
          if (state.parens === 0 && opts.strictBrackets === true) {
            throw new SyntaxError(syntaxError("opening", "("));
          }
          const extglob = extglobs[extglobs.length - 1];
          if (extglob && state.parens === extglob.parens + 1) {
            extglobClose(extglobs.pop());
            continue;
          }
          push({ type: "paren", value, output: state.parens ? ")" : "\\)" });
          decrement("parens");
          continue;
        }
        if (value === "[") {
          if (opts.nobracket === true || !remaining().includes("]")) {
            if (opts.nobracket !== true && opts.strictBrackets === true) {
              throw new SyntaxError(syntaxError("closing", "]"));
            }
            value = `\\${value}`;
          } else {
            increment("brackets");
          }
          push({ type: "bracket", value });
          continue;
        }
        if (value === "]") {
          if (opts.nobracket === true || prev && prev.type === "bracket" && prev.value.length === 1) {
            push({ type: "text", value, output: `\\${value}` });
            continue;
          }
          if (state.brackets === 0) {
            if (opts.strictBrackets === true) {
              throw new SyntaxError(syntaxError("opening", "["));
            }
            push({ type: "text", value, output: `\\${value}` });
            continue;
          }
          decrement("brackets");
          const prevValue = prev.value.slice(1);
          if (prev.posix !== true && prevValue[0] === "^" && !prevValue.includes("/")) {
            value = `/${value}`;
          }
          prev.value += value;
          append({ value });
          if (opts.literalBrackets === false || utils.hasRegexChars(prevValue)) {
            continue;
          }
          const escaped = utils.escapeRegex(prev.value);
          state.output = state.output.slice(0, -prev.value.length);
          if (opts.literalBrackets === true) {
            state.output += escaped;
            prev.value = escaped;
            continue;
          }
          prev.value = `(${capture}${escaped}|${prev.value})`;
          state.output += prev.value;
          continue;
        }
        if (value === "{" && opts.nobrace !== true) {
          increment("braces");
          const open = {
            type: "brace",
            value,
            output: "(",
            outputIndex: state.output.length,
            tokensIndex: state.tokens.length
          };
          braces.push(open);
          push(open);
          continue;
        }
        if (value === "}") {
          const brace = braces[braces.length - 1];
          if (opts.nobrace === true || !brace) {
            push({ type: "text", value, output: value });
            continue;
          }
          let output = ")";
          if (brace.dots === true) {
            const arr = tokens.slice();
            const range = [];
            for (let i = arr.length - 1; i >= 0; i--) {
              tokens.pop();
              if (arr[i].type === "brace") {
                break;
              }
              if (arr[i].type !== "dots") {
                range.unshift(arr[i].value);
              }
            }
            output = expandRange(range, opts);
            state.backtrack = true;
          }
          if (brace.comma !== true && brace.dots !== true) {
            const out = state.output.slice(0, brace.outputIndex);
            const toks = state.tokens.slice(brace.tokensIndex);
            brace.value = brace.output = "\\{";
            value = output = "\\}";
            state.output = out;
            for (const t of toks) {
              state.output += t.output || t.value;
            }
          }
          push({ type: "brace", value, output });
          decrement("braces");
          braces.pop();
          continue;
        }
        if (value === "|") {
          if (extglobs.length > 0) {
            extglobs[extglobs.length - 1].conditions++;
          }
          push({ type: "text", value });
          continue;
        }
        if (value === ",") {
          let output = value;
          const brace = braces[braces.length - 1];
          if (brace && stack[stack.length - 1] === "braces") {
            brace.comma = true;
            output = "|";
          }
          push({ type: "comma", value, output });
          continue;
        }
        if (value === "/") {
          if (prev.type === "dot" && state.index === state.start + 1) {
            state.start = state.index + 1;
            state.consumed = "";
            state.output = "";
            tokens.pop();
            prev = bos;
            continue;
          }
          push({ type: "slash", value, output: SLASH_LITERAL });
          continue;
        }
        if (value === ".") {
          if (state.braces > 0 && prev.type === "dot") {
            if (prev.value === ".")
              prev.output = DOT_LITERAL;
            const brace = braces[braces.length - 1];
            prev.type = "dots";
            prev.output += value;
            prev.value += value;
            brace.dots = true;
            continue;
          }
          if (state.braces + state.parens === 0 && prev.type !== "bos" && prev.type !== "slash") {
            push({ type: "text", value, output: DOT_LITERAL });
            continue;
          }
          push({ type: "dot", value, output: DOT_LITERAL });
          continue;
        }
        if (value === "?") {
          const isGroup = prev && prev.value === "(";
          if (!isGroup && opts.noextglob !== true && peek() === "(" && peek(2) !== "?") {
            extglobOpen("qmark", value);
            continue;
          }
          if (prev && prev.type === "paren") {
            const next = peek();
            let output = value;
            if (next === "<" && !utils.supportsLookbehinds()) {
              throw new Error("Node.js v10 or higher is required for regex lookbehinds");
            }
            if (prev.value === "(" && !/[!=<:]/.test(next) || next === "<" && !/<([!=]|\w+>)/.test(remaining())) {
              output = `\\${value}`;
            }
            push({ type: "text", value, output });
            continue;
          }
          if (opts.dot !== true && (prev.type === "slash" || prev.type === "bos")) {
            push({ type: "qmark", value, output: QMARK_NO_DOT });
            continue;
          }
          push({ type: "qmark", value, output: QMARK });
          continue;
        }
        if (value === "!") {
          if (opts.noextglob !== true && peek() === "(") {
            if (peek(2) !== "?" || !/[!=<:]/.test(peek(3))) {
              extglobOpen("negate", value);
              continue;
            }
          }
          if (opts.nonegate !== true && state.index === 0) {
            negate();
            continue;
          }
        }
        if (value === "+") {
          if (opts.noextglob !== true && peek() === "(" && peek(2) !== "?") {
            extglobOpen("plus", value);
            continue;
          }
          if (prev && prev.value === "(" || opts.regex === false) {
            push({ type: "plus", value, output: PLUS_LITERAL });
            continue;
          }
          if (prev && (prev.type === "bracket" || prev.type === "paren" || prev.type === "brace") || state.parens > 0) {
            push({ type: "plus", value });
            continue;
          }
          push({ type: "plus", value: PLUS_LITERAL });
          continue;
        }
        if (value === "@") {
          if (opts.noextglob !== true && peek() === "(" && peek(2) !== "?") {
            push({ type: "at", extglob: true, value, output: "" });
            continue;
          }
          push({ type: "text", value });
          continue;
        }
        if (value !== "*") {
          if (value === "$" || value === "^") {
            value = `\\${value}`;
          }
          const match = REGEX_NON_SPECIAL_CHARS.exec(remaining());
          if (match) {
            value += match[0];
            state.index += match[0].length;
          }
          push({ type: "text", value });
          continue;
        }
        if (prev && (prev.type === "globstar" || prev.star === true)) {
          prev.type = "star";
          prev.star = true;
          prev.value += value;
          prev.output = star;
          state.backtrack = true;
          state.globstar = true;
          consume(value);
          continue;
        }
        let rest = remaining();
        if (opts.noextglob !== true && /^\([^?]/.test(rest)) {
          extglobOpen("star", value);
          continue;
        }
        if (prev.type === "star") {
          if (opts.noglobstar === true) {
            consume(value);
            continue;
          }
          const prior = prev.prev;
          const before = prior.prev;
          const isStart = prior.type === "slash" || prior.type === "bos";
          const afterStar = before && (before.type === "star" || before.type === "globstar");
          if (opts.bash === true && (!isStart || rest[0] && rest[0] !== "/")) {
            push({ type: "star", value, output: "" });
            continue;
          }
          const isBrace = state.braces > 0 && (prior.type === "comma" || prior.type === "brace");
          const isExtglob = extglobs.length && (prior.type === "pipe" || prior.type === "paren");
          if (!isStart && prior.type !== "paren" && !isBrace && !isExtglob) {
            push({ type: "star", value, output: "" });
            continue;
          }
          while (rest.slice(0, 3) === "/**") {
            const after = input[state.index + 4];
            if (after && after !== "/") {
              break;
            }
            rest = rest.slice(3);
            consume("/**", 3);
          }
          if (prior.type === "bos" && eos()) {
            prev.type = "globstar";
            prev.value += value;
            prev.output = globstar(opts);
            state.output = prev.output;
            state.globstar = true;
            consume(value);
            continue;
          }
          if (prior.type === "slash" && prior.prev.type !== "bos" && !afterStar && eos()) {
            state.output = state.output.slice(0, -(prior.output + prev.output).length);
            prior.output = `(?:${prior.output}`;
            prev.type = "globstar";
            prev.output = globstar(opts) + (opts.strictSlashes ? ")" : "|$)");
            prev.value += value;
            state.globstar = true;
            state.output += prior.output + prev.output;
            consume(value);
            continue;
          }
          if (prior.type === "slash" && prior.prev.type !== "bos" && rest[0] === "/") {
            const end = rest[1] !== void 0 ? "|$" : "";
            state.output = state.output.slice(0, -(prior.output + prev.output).length);
            prior.output = `(?:${prior.output}`;
            prev.type = "globstar";
            prev.output = `${globstar(opts)}${SLASH_LITERAL}|${SLASH_LITERAL}${end})`;
            prev.value += value;
            state.output += prior.output + prev.output;
            state.globstar = true;
            consume(value + advance());
            push({ type: "slash", value: "/", output: "" });
            continue;
          }
          if (prior.type === "bos" && rest[0] === "/") {
            prev.type = "globstar";
            prev.value += value;
            prev.output = `(?:^|${SLASH_LITERAL}|${globstar(opts)}${SLASH_LITERAL})`;
            state.output = prev.output;
            state.globstar = true;
            consume(value + advance());
            push({ type: "slash", value: "/", output: "" });
            continue;
          }
          state.output = state.output.slice(0, -prev.output.length);
          prev.type = "globstar";
          prev.output = globstar(opts);
          prev.value += value;
          state.output += prev.output;
          state.globstar = true;
          consume(value);
          continue;
        }
        const token = { type: "star", value, output: star };
        if (opts.bash === true) {
          token.output = ".*?";
          if (prev.type === "bos" || prev.type === "slash") {
            token.output = nodot + token.output;
          }
          push(token);
          continue;
        }
        if (prev && (prev.type === "bracket" || prev.type === "paren") && opts.regex === true) {
          token.output = value;
          push(token);
          continue;
        }
        if (state.index === state.start || prev.type === "slash" || prev.type === "dot") {
          if (prev.type === "dot") {
            state.output += NO_DOT_SLASH;
            prev.output += NO_DOT_SLASH;
          } else if (opts.dot === true) {
            state.output += NO_DOTS_SLASH;
            prev.output += NO_DOTS_SLASH;
          } else {
            state.output += nodot;
            prev.output += nodot;
          }
          if (peek() !== "*") {
            state.output += ONE_CHAR;
            prev.output += ONE_CHAR;
          }
        }
        push(token);
      }
      while (state.brackets > 0) {
        if (opts.strictBrackets === true)
          throw new SyntaxError(syntaxError("closing", "]"));
        state.output = utils.escapeLast(state.output, "[");
        decrement("brackets");
      }
      while (state.parens > 0) {
        if (opts.strictBrackets === true)
          throw new SyntaxError(syntaxError("closing", ")"));
        state.output = utils.escapeLast(state.output, "(");
        decrement("parens");
      }
      while (state.braces > 0) {
        if (opts.strictBrackets === true)
          throw new SyntaxError(syntaxError("closing", "}"));
        state.output = utils.escapeLast(state.output, "{");
        decrement("braces");
      }
      if (opts.strictSlashes !== true && (prev.type === "star" || prev.type === "bracket")) {
        push({ type: "maybe_slash", value: "", output: `${SLASH_LITERAL}?` });
      }
      if (state.backtrack === true) {
        state.output = "";
        for (const token of state.tokens) {
          state.output += token.output != null ? token.output : token.value;
          if (token.suffix) {
            state.output += token.suffix;
          }
        }
      }
      return state;
    };
    parse2.fastpaths = (input, options) => {
      const opts = { ...options };
      const max = typeof opts.maxLength === "number" ? Math.min(MAX_LENGTH, opts.maxLength) : MAX_LENGTH;
      const len = input.length;
      if (len > max) {
        throw new SyntaxError(`Input length: ${len}, exceeds maximum allowed length: ${max}`);
      }
      input = REPLACEMENTS[input] || input;
      const win32 = utils.isWindows(options);
      const {
        DOT_LITERAL,
        SLASH_LITERAL,
        ONE_CHAR,
        DOTS_SLASH,
        NO_DOT,
        NO_DOTS,
        NO_DOTS_SLASH,
        STAR,
        START_ANCHOR
      } = constants3.globChars(win32);
      const nodot = opts.dot ? NO_DOTS : NO_DOT;
      const slashDot = opts.dot ? NO_DOTS_SLASH : NO_DOT;
      const capture = opts.capture ? "" : "?:";
      const state = { negated: false, prefix: "" };
      let star = opts.bash === true ? ".*?" : STAR;
      if (opts.capture) {
        star = `(${star})`;
      }
      const globstar = (opts2) => {
        if (opts2.noglobstar === true)
          return star;
        return `(${capture}(?:(?!${START_ANCHOR}${opts2.dot ? DOTS_SLASH : DOT_LITERAL}).)*?)`;
      };
      const create = (str) => {
        switch (str) {
          case "*":
            return `${nodot}${ONE_CHAR}${star}`;
          case ".*":
            return `${DOT_LITERAL}${ONE_CHAR}${star}`;
          case "*.*":
            return `${nodot}${star}${DOT_LITERAL}${ONE_CHAR}${star}`;
          case "*/*":
            return `${nodot}${star}${SLASH_LITERAL}${ONE_CHAR}${slashDot}${star}`;
          case "**":
            return nodot + globstar(opts);
          case "**/*":
            return `(?:${nodot}${globstar(opts)}${SLASH_LITERAL})?${slashDot}${ONE_CHAR}${star}`;
          case "**/*.*":
            return `(?:${nodot}${globstar(opts)}${SLASH_LITERAL})?${slashDot}${star}${DOT_LITERAL}${ONE_CHAR}${star}`;
          case "**/.*":
            return `(?:${nodot}${globstar(opts)}${SLASH_LITERAL})?${DOT_LITERAL}${ONE_CHAR}${star}`;
          default: {
            const match = /^(.*?)\.(\w+)$/.exec(str);
            if (!match)
              return;
            const source2 = create(match[1]);
            if (!source2)
              return;
            return source2 + DOT_LITERAL + match[2];
          }
        }
      };
      const output = utils.removePrefix(input, state);
      let source = create(output);
      if (source && opts.strictSlashes !== true) {
        source += `${SLASH_LITERAL}?`;
      }
      return source;
    };
    module2.exports = parse2;
  }
});

// node_modules/picomatch/lib/picomatch.js
var require_picomatch = __commonJS({
  "node_modules/picomatch/lib/picomatch.js"(exports2, module2) {
    "use strict";
    var path4 = require("path");
    var scan = require_scan();
    var parse2 = require_parse();
    var utils = require_utils();
    var constants3 = require_constants();
    var isObject = (val) => val && typeof val === "object" && !Array.isArray(val);
    var picomatch = (glob, options, returnState = false) => {
      if (Array.isArray(glob)) {
        const fns = glob.map((input) => picomatch(input, options, returnState));
        const arrayMatcher = (str) => {
          for (const isMatch of fns) {
            const state2 = isMatch(str);
            if (state2)
              return state2;
          }
          return false;
        };
        return arrayMatcher;
      }
      const isState = isObject(glob) && glob.tokens && glob.input;
      if (glob === "" || typeof glob !== "string" && !isState) {
        throw new TypeError("Expected pattern to be a non-empty string");
      }
      const opts = options || {};
      const posix = utils.isWindows(options);
      const regex = isState ? picomatch.compileRe(glob, options) : picomatch.makeRe(glob, options, false, true);
      const state = regex.state;
      delete regex.state;
      let isIgnored = () => false;
      if (opts.ignore) {
        const ignoreOpts = { ...options, ignore: null, onMatch: null, onResult: null };
        isIgnored = picomatch(opts.ignore, ignoreOpts, returnState);
      }
      const matcher = (input, returnObject = false) => {
        const { isMatch, match, output } = picomatch.test(input, regex, options, { glob, posix });
        const result = { glob, state, regex, posix, input, output, match, isMatch };
        if (typeof opts.onResult === "function") {
          opts.onResult(result);
        }
        if (isMatch === false) {
          result.isMatch = false;
          return returnObject ? result : false;
        }
        if (isIgnored(input)) {
          if (typeof opts.onIgnore === "function") {
            opts.onIgnore(result);
          }
          result.isMatch = false;
          return returnObject ? result : false;
        }
        if (typeof opts.onMatch === "function") {
          opts.onMatch(result);
        }
        return returnObject ? result : true;
      };
      if (returnState) {
        matcher.state = state;
      }
      return matcher;
    };
    picomatch.test = (input, regex, options, { glob, posix } = {}) => {
      if (typeof input !== "string") {
        throw new TypeError("Expected input to be a string");
      }
      if (input === "") {
        return { isMatch: false, output: "" };
      }
      const opts = options || {};
      const format = opts.format || (posix ? utils.toPosixSlashes : null);
      let match = input === glob;
      let output = match && format ? format(input) : input;
      if (match === false) {
        output = format ? format(input) : input;
        match = output === glob;
      }
      if (match === false || opts.capture === true) {
        if (opts.matchBase === true || opts.basename === true) {
          match = picomatch.matchBase(input, regex, options, posix);
        } else {
          match = regex.exec(output);
        }
      }
      return { isMatch: Boolean(match), match, output };
    };
    picomatch.matchBase = (input, glob, options, posix = utils.isWindows(options)) => {
      const regex = glob instanceof RegExp ? glob : picomatch.makeRe(glob, options);
      return regex.test(path4.basename(input));
    };
    picomatch.isMatch = (str, patterns, options) => picomatch(patterns, options)(str);
    picomatch.parse = (pattern, options) => {
      if (Array.isArray(pattern))
        return pattern.map((p) => picomatch.parse(p, options));
      return parse2(pattern, { ...options, fastpaths: false });
    };
    picomatch.scan = (input, options) => scan(input, options);
    picomatch.compileRe = (state, options, returnOutput = false, returnState = false) => {
      if (returnOutput === true) {
        return state.output;
      }
      const opts = options || {};
      const prepend = opts.contains ? "" : "^";
      const append = opts.contains ? "" : "$";
      let source = `${prepend}(?:${state.output})${append}`;
      if (state && state.negated === true) {
        source = `^(?!${source}).*$`;
      }
      const regex = picomatch.toRegex(source, options);
      if (returnState === true) {
        regex.state = state;
      }
      return regex;
    };
    picomatch.makeRe = (input, options = {}, returnOutput = false, returnState = false) => {
      if (!input || typeof input !== "string") {
        throw new TypeError("Expected a non-empty string");
      }
      let parsed = { negated: false, fastpaths: true };
      if (options.fastpaths !== false && (input[0] === "." || input[0] === "*")) {
        parsed.output = parse2.fastpaths(input, options);
      }
      if (!parsed.output) {
        parsed = parse2(input, options);
      }
      return picomatch.compileRe(parsed, options, returnOutput, returnState);
    };
    picomatch.toRegex = (source, options) => {
      try {
        const opts = options || {};
        return new RegExp(source, opts.flags || (opts.nocase ? "i" : ""));
      } catch (err) {
        if (options && options.debug === true)
          throw err;
        return /$^/;
      }
    };
    picomatch.constants = constants3;
    module2.exports = picomatch;
  }
});

// node_modules/picomatch/index.js
var require_picomatch2 = __commonJS({
  "node_modules/picomatch/index.js"(exports2, module2) {
    "use strict";
    module2.exports = require_picomatch();
  }
});

// node_modules/kleur/colors.js
var require_colors = __commonJS({
  "node_modules/kleur/colors.js"(exports2) {
    var FORCE_COLOR;
    var NODE_DISABLE_COLORS;
    var NO_COLOR;
    var TERM;
    var isTTY = true;
    if (typeof process !== "undefined") {
      ({ FORCE_COLOR, NODE_DISABLE_COLORS, NO_COLOR, TERM } = process.env || {});
      isTTY = process.stdout && process.stdout.isTTY;
    }
    var $ = exports2.$ = {
      enabled: !NODE_DISABLE_COLORS && NO_COLOR == null && TERM !== "dumb" && (FORCE_COLOR != null && FORCE_COLOR !== "0" || isTTY)
    };
    function init(x, y) {
      let rgx = new RegExp(`\\x1b\\[${y}m`, "g");
      let open = `\x1B[${x}m`, close = `\x1B[${y}m`;
      return function(txt) {
        if (!$.enabled || txt == null)
          return txt;
        return open + (!!~("" + txt).indexOf(close) ? txt.replace(rgx, close + open) : txt) + close;
      };
    }
    exports2.reset = init(0, 0);
    exports2.bold = init(1, 22);
    exports2.dim = init(2, 22);
    exports2.italic = init(3, 23);
    exports2.underline = init(4, 24);
    exports2.inverse = init(7, 27);
    exports2.hidden = init(8, 28);
    exports2.strikethrough = init(9, 29);
    exports2.black = init(30, 39);
    exports2.red = init(31, 39);
    exports2.green = init(32, 39);
    exports2.yellow = init(33, 39);
    exports2.blue = init(34, 39);
    exports2.magenta = init(35, 39);
    exports2.cyan = init(36, 39);
    exports2.white = init(37, 39);
    exports2.gray = init(90, 39);
    exports2.grey = init(90, 39);
    exports2.bgBlack = init(40, 49);
    exports2.bgRed = init(41, 49);
    exports2.bgGreen = init(42, 49);
    exports2.bgYellow = init(43, 49);
    exports2.bgBlue = init(44, 49);
    exports2.bgMagenta = init(45, 49);
    exports2.bgCyan = init(46, 49);
    exports2.bgWhite = init(47, 49);
  }
});

// node_modules/semver/internal/constants.js
var require_constants2 = __commonJS({
  "node_modules/semver/internal/constants.js"(exports2, module2) {
    var SEMVER_SPEC_VERSION = "2.0.0";
    var MAX_LENGTH = 256;
    var MAX_SAFE_INTEGER = Number.MAX_SAFE_INTEGER || /* istanbul ignore next */
    9007199254740991;
    var MAX_SAFE_COMPONENT_LENGTH = 16;
    var MAX_SAFE_BUILD_LENGTH = MAX_LENGTH - 6;
    var RELEASE_TYPES = [
      "major",
      "premajor",
      "minor",
      "preminor",
      "patch",
      "prepatch",
      "prerelease"
    ];
    module2.exports = {
      MAX_LENGTH,
      MAX_SAFE_COMPONENT_LENGTH,
      MAX_SAFE_BUILD_LENGTH,
      MAX_SAFE_INTEGER,
      RELEASE_TYPES,
      SEMVER_SPEC_VERSION,
      FLAG_INCLUDE_PRERELEASE: 1,
      FLAG_LOOSE: 2
    };
  }
});

// node_modules/semver/internal/debug.js
var require_debug = __commonJS({
  "node_modules/semver/internal/debug.js"(exports2, module2) {
    var debug = typeof process === "object" && process.env && process.env.NODE_DEBUG && /\bsemver\b/i.test(process.env.NODE_DEBUG) ? (...args) => console.error("SEMVER", ...args) : () => {
    };
    module2.exports = debug;
  }
});

// node_modules/semver/internal/re.js
var require_re = __commonJS({
  "node_modules/semver/internal/re.js"(exports2, module2) {
    var {
      MAX_SAFE_COMPONENT_LENGTH,
      MAX_SAFE_BUILD_LENGTH,
      MAX_LENGTH
    } = require_constants2();
    var debug = require_debug();
    exports2 = module2.exports = {};
    var re = exports2.re = [];
    var safeRe = exports2.safeRe = [];
    var src = exports2.src = [];
    var t = exports2.t = {};
    var R = 0;
    var LETTERDASHNUMBER = "[a-zA-Z0-9-]";
    var safeRegexReplacements = [
      ["\\s", 1],
      ["\\d", MAX_LENGTH],
      [LETTERDASHNUMBER, MAX_SAFE_BUILD_LENGTH]
    ];
    var makeSafeRegex = (value) => {
      for (const [token, max] of safeRegexReplacements) {
        value = value.split(`${token}*`).join(`${token}{0,${max}}`).split(`${token}+`).join(`${token}{1,${max}}`);
      }
      return value;
    };
    var createToken = (name, value, isGlobal) => {
      const safe = makeSafeRegex(value);
      const index = R++;
      debug(name, index, value);
      t[name] = index;
      src[index] = value;
      re[index] = new RegExp(value, isGlobal ? "g" : void 0);
      safeRe[index] = new RegExp(safe, isGlobal ? "g" : void 0);
    };
    createToken("NUMERICIDENTIFIER", "0|[1-9]\\d*");
    createToken("NUMERICIDENTIFIERLOOSE", "\\d+");
    createToken("NONNUMERICIDENTIFIER", `\\d*[a-zA-Z-]${LETTERDASHNUMBER}*`);
    createToken("MAINVERSION", `(${src[t.NUMERICIDENTIFIER]})\\.(${src[t.NUMERICIDENTIFIER]})\\.(${src[t.NUMERICIDENTIFIER]})`);
    createToken("MAINVERSIONLOOSE", `(${src[t.NUMERICIDENTIFIERLOOSE]})\\.(${src[t.NUMERICIDENTIFIERLOOSE]})\\.(${src[t.NUMERICIDENTIFIERLOOSE]})`);
    createToken("PRERELEASEIDENTIFIER", `(?:${src[t.NUMERICIDENTIFIER]}|${src[t.NONNUMERICIDENTIFIER]})`);
    createToken("PRERELEASEIDENTIFIERLOOSE", `(?:${src[t.NUMERICIDENTIFIERLOOSE]}|${src[t.NONNUMERICIDENTIFIER]})`);
    createToken("PRERELEASE", `(?:-(${src[t.PRERELEASEIDENTIFIER]}(?:\\.${src[t.PRERELEASEIDENTIFIER]})*))`);
    createToken("PRERELEASELOOSE", `(?:-?(${src[t.PRERELEASEIDENTIFIERLOOSE]}(?:\\.${src[t.PRERELEASEIDENTIFIERLOOSE]})*))`);
    createToken("BUILDIDENTIFIER", `${LETTERDASHNUMBER}+`);
    createToken("BUILD", `(?:\\+(${src[t.BUILDIDENTIFIER]}(?:\\.${src[t.BUILDIDENTIFIER]})*))`);
    createToken("FULLPLAIN", `v?${src[t.MAINVERSION]}${src[t.PRERELEASE]}?${src[t.BUILD]}?`);
    createToken("FULL", `^${src[t.FULLPLAIN]}$`);
    createToken("LOOSEPLAIN", `[v=\\s]*${src[t.MAINVERSIONLOOSE]}${src[t.PRERELEASELOOSE]}?${src[t.BUILD]}?`);
    createToken("LOOSE", `^${src[t.LOOSEPLAIN]}$`);
    createToken("GTLT", "((?:<|>)?=?)");
    createToken("XRANGEIDENTIFIERLOOSE", `${src[t.NUMERICIDENTIFIERLOOSE]}|x|X|\\*`);
    createToken("XRANGEIDENTIFIER", `${src[t.NUMERICIDENTIFIER]}|x|X|\\*`);
    createToken("XRANGEPLAIN", `[v=\\s]*(${src[t.XRANGEIDENTIFIER]})(?:\\.(${src[t.XRANGEIDENTIFIER]})(?:\\.(${src[t.XRANGEIDENTIFIER]})(?:${src[t.PRERELEASE]})?${src[t.BUILD]}?)?)?`);
    createToken("XRANGEPLAINLOOSE", `[v=\\s]*(${src[t.XRANGEIDENTIFIERLOOSE]})(?:\\.(${src[t.XRANGEIDENTIFIERLOOSE]})(?:\\.(${src[t.XRANGEIDENTIFIERLOOSE]})(?:${src[t.PRERELEASELOOSE]})?${src[t.BUILD]}?)?)?`);
    createToken("XRANGE", `^${src[t.GTLT]}\\s*${src[t.XRANGEPLAIN]}$`);
    createToken("XRANGELOOSE", `^${src[t.GTLT]}\\s*${src[t.XRANGEPLAINLOOSE]}$`);
    createToken("COERCE", `${"(^|[^\\d])(\\d{1,"}${MAX_SAFE_COMPONENT_LENGTH}})(?:\\.(\\d{1,${MAX_SAFE_COMPONENT_LENGTH}}))?(?:\\.(\\d{1,${MAX_SAFE_COMPONENT_LENGTH}}))?(?:$|[^\\d])`);
    createToken("COERCERTL", src[t.COERCE], true);
    createToken("LONETILDE", "(?:~>?)");
    createToken("TILDETRIM", `(\\s*)${src[t.LONETILDE]}\\s+`, true);
    exports2.tildeTrimReplace = "$1~";
    createToken("TILDE", `^${src[t.LONETILDE]}${src[t.XRANGEPLAIN]}$`);
    createToken("TILDELOOSE", `^${src[t.LONETILDE]}${src[t.XRANGEPLAINLOOSE]}$`);
    createToken("LONECARET", "(?:\\^)");
    createToken("CARETTRIM", `(\\s*)${src[t.LONECARET]}\\s+`, true);
    exports2.caretTrimReplace = "$1^";
    createToken("CARET", `^${src[t.LONECARET]}${src[t.XRANGEPLAIN]}$`);
    createToken("CARETLOOSE", `^${src[t.LONECARET]}${src[t.XRANGEPLAINLOOSE]}$`);
    createToken("COMPARATORLOOSE", `^${src[t.GTLT]}\\s*(${src[t.LOOSEPLAIN]})$|^$`);
    createToken("COMPARATOR", `^${src[t.GTLT]}\\s*(${src[t.FULLPLAIN]})$|^$`);
    createToken("COMPARATORTRIM", `(\\s*)${src[t.GTLT]}\\s*(${src[t.LOOSEPLAIN]}|${src[t.XRANGEPLAIN]})`, true);
    exports2.comparatorTrimReplace = "$1$2$3";
    createToken("HYPHENRANGE", `^\\s*(${src[t.XRANGEPLAIN]})\\s+-\\s+(${src[t.XRANGEPLAIN]})\\s*$`);
    createToken("HYPHENRANGELOOSE", `^\\s*(${src[t.XRANGEPLAINLOOSE]})\\s+-\\s+(${src[t.XRANGEPLAINLOOSE]})\\s*$`);
    createToken("STAR", "(<|>)?=?\\s*\\*");
    createToken("GTE0", "^\\s*>=\\s*0\\.0\\.0\\s*$");
    createToken("GTE0PRE", "^\\s*>=\\s*0\\.0\\.0-0\\s*$");
  }
});

// node_modules/semver/internal/parse-options.js
var require_parse_options = __commonJS({
  "node_modules/semver/internal/parse-options.js"(exports2, module2) {
    var looseOption = Object.freeze({ loose: true });
    var emptyOpts = Object.freeze({});
    var parseOptions = (options) => {
      if (!options) {
        return emptyOpts;
      }
      if (typeof options !== "object") {
        return looseOption;
      }
      return options;
    };
    module2.exports = parseOptions;
  }
});

// node_modules/semver/internal/identifiers.js
var require_identifiers = __commonJS({
  "node_modules/semver/internal/identifiers.js"(exports2, module2) {
    var numeric = /^[0-9]+$/;
    var compareIdentifiers = (a, b) => {
      const anum = numeric.test(a);
      const bnum = numeric.test(b);
      if (anum && bnum) {
        a = +a;
        b = +b;
      }
      return a === b ? 0 : anum && !bnum ? -1 : bnum && !anum ? 1 : a < b ? -1 : 1;
    };
    var rcompareIdentifiers = (a, b) => compareIdentifiers(b, a);
    module2.exports = {
      compareIdentifiers,
      rcompareIdentifiers
    };
  }
});

// node_modules/semver/classes/semver.js
var require_semver = __commonJS({
  "node_modules/semver/classes/semver.js"(exports2, module2) {
    var debug = require_debug();
    var { MAX_LENGTH, MAX_SAFE_INTEGER } = require_constants2();
    var { safeRe: re, t } = require_re();
    var parseOptions = require_parse_options();
    var { compareIdentifiers } = require_identifiers();
    var SemVer = class _SemVer {
      constructor(version, options) {
        options = parseOptions(options);
        if (version instanceof _SemVer) {
          if (version.loose === !!options.loose && version.includePrerelease === !!options.includePrerelease) {
            return version;
          } else {
            version = version.version;
          }
        } else if (typeof version !== "string") {
          throw new TypeError(`Invalid version. Must be a string. Got type "${typeof version}".`);
        }
        if (version.length > MAX_LENGTH) {
          throw new TypeError(
            `version is longer than ${MAX_LENGTH} characters`
          );
        }
        debug("SemVer", version, options);
        this.options = options;
        this.loose = !!options.loose;
        this.includePrerelease = !!options.includePrerelease;
        const m = version.trim().match(options.loose ? re[t.LOOSE] : re[t.FULL]);
        if (!m) {
          throw new TypeError(`Invalid Version: ${version}`);
        }
        this.raw = version;
        this.major = +m[1];
        this.minor = +m[2];
        this.patch = +m[3];
        if (this.major > MAX_SAFE_INTEGER || this.major < 0) {
          throw new TypeError("Invalid major version");
        }
        if (this.minor > MAX_SAFE_INTEGER || this.minor < 0) {
          throw new TypeError("Invalid minor version");
        }
        if (this.patch > MAX_SAFE_INTEGER || this.patch < 0) {
          throw new TypeError("Invalid patch version");
        }
        if (!m[4]) {
          this.prerelease = [];
        } else {
          this.prerelease = m[4].split(".").map((id) => {
            if (/^[0-9]+$/.test(id)) {
              const num = +id;
              if (num >= 0 && num < MAX_SAFE_INTEGER) {
                return num;
              }
            }
            return id;
          });
        }
        this.build = m[5] ? m[5].split(".") : [];
        this.format();
      }
      format() {
        this.version = `${this.major}.${this.minor}.${this.patch}`;
        if (this.prerelease.length) {
          this.version += `-${this.prerelease.join(".")}`;
        }
        return this.version;
      }
      toString() {
        return this.version;
      }
      compare(other) {
        debug("SemVer.compare", this.version, this.options, other);
        if (!(other instanceof _SemVer)) {
          if (typeof other === "string" && other === this.version) {
            return 0;
          }
          other = new _SemVer(other, this.options);
        }
        if (other.version === this.version) {
          return 0;
        }
        return this.compareMain(other) || this.comparePre(other);
      }
      compareMain(other) {
        if (!(other instanceof _SemVer)) {
          other = new _SemVer(other, this.options);
        }
        return compareIdentifiers(this.major, other.major) || compareIdentifiers(this.minor, other.minor) || compareIdentifiers(this.patch, other.patch);
      }
      comparePre(other) {
        if (!(other instanceof _SemVer)) {
          other = new _SemVer(other, this.options);
        }
        if (this.prerelease.length && !other.prerelease.length) {
          return -1;
        } else if (!this.prerelease.length && other.prerelease.length) {
          return 1;
        } else if (!this.prerelease.length && !other.prerelease.length) {
          return 0;
        }
        let i = 0;
        do {
          const a = this.prerelease[i];
          const b = other.prerelease[i];
          debug("prerelease compare", i, a, b);
          if (a === void 0 && b === void 0) {
            return 0;
          } else if (b === void 0) {
            return 1;
          } else if (a === void 0) {
            return -1;
          } else if (a === b) {
            continue;
          } else {
            return compareIdentifiers(a, b);
          }
        } while (++i);
      }
      compareBuild(other) {
        if (!(other instanceof _SemVer)) {
          other = new _SemVer(other, this.options);
        }
        let i = 0;
        do {
          const a = this.build[i];
          const b = other.build[i];
          debug("prerelease compare", i, a, b);
          if (a === void 0 && b === void 0) {
            return 0;
          } else if (b === void 0) {
            return 1;
          } else if (a === void 0) {
            return -1;
          } else if (a === b) {
            continue;
          } else {
            return compareIdentifiers(a, b);
          }
        } while (++i);
      }
      // preminor will bump the version up to the next minor release, and immediately
      // down to pre-release. premajor and prepatch work the same way.
      inc(release, identifier, identifierBase) {
        switch (release) {
          case "premajor":
            this.prerelease.length = 0;
            this.patch = 0;
            this.minor = 0;
            this.major++;
            this.inc("pre", identifier, identifierBase);
            break;
          case "preminor":
            this.prerelease.length = 0;
            this.patch = 0;
            this.minor++;
            this.inc("pre", identifier, identifierBase);
            break;
          case "prepatch":
            this.prerelease.length = 0;
            this.inc("patch", identifier, identifierBase);
            this.inc("pre", identifier, identifierBase);
            break;
          case "prerelease":
            if (this.prerelease.length === 0) {
              this.inc("patch", identifier, identifierBase);
            }
            this.inc("pre", identifier, identifierBase);
            break;
          case "major":
            if (this.minor !== 0 || this.patch !== 0 || this.prerelease.length === 0) {
              this.major++;
            }
            this.minor = 0;
            this.patch = 0;
            this.prerelease = [];
            break;
          case "minor":
            if (this.patch !== 0 || this.prerelease.length === 0) {
              this.minor++;
            }
            this.patch = 0;
            this.prerelease = [];
            break;
          case "patch":
            if (this.prerelease.length === 0) {
              this.patch++;
            }
            this.prerelease = [];
            break;
          case "pre": {
            const base = Number(identifierBase) ? 1 : 0;
            if (!identifier && identifierBase === false) {
              throw new Error("invalid increment argument: identifier is empty");
            }
            if (this.prerelease.length === 0) {
              this.prerelease = [base];
            } else {
              let i = this.prerelease.length;
              while (--i >= 0) {
                if (typeof this.prerelease[i] === "number") {
                  this.prerelease[i]++;
                  i = -2;
                }
              }
              if (i === -1) {
                if (identifier === this.prerelease.join(".") && identifierBase === false) {
                  throw new Error("invalid increment argument: identifier already exists");
                }
                this.prerelease.push(base);
              }
            }
            if (identifier) {
              let prerelease = [identifier, base];
              if (identifierBase === false) {
                prerelease = [identifier];
              }
              if (compareIdentifiers(this.prerelease[0], identifier) === 0) {
                if (isNaN(this.prerelease[1])) {
                  this.prerelease = prerelease;
                }
              } else {
                this.prerelease = prerelease;
              }
            }
            break;
          }
          default:
            throw new Error(`invalid increment argument: ${release}`);
        }
        this.raw = this.format();
        if (this.build.length) {
          this.raw += `+${this.build.join(".")}`;
        }
        return this;
      }
    };
    module2.exports = SemVer;
  }
});

// node_modules/semver/functions/parse.js
var require_parse2 = __commonJS({
  "node_modules/semver/functions/parse.js"(exports2, module2) {
    var SemVer = require_semver();
    var parse2 = (version, options, throwErrors = false) => {
      if (version instanceof SemVer) {
        return version;
      }
      try {
        return new SemVer(version, options);
      } catch (er) {
        if (!throwErrors) {
          return null;
        }
        throw er;
      }
    };
    module2.exports = parse2;
  }
});

// node_modules/semver/functions/valid.js
var require_valid = __commonJS({
  "node_modules/semver/functions/valid.js"(exports2, module2) {
    var parse2 = require_parse2();
    var valid = (version, options) => {
      const v = parse2(version, options);
      return v ? v.version : null;
    };
    module2.exports = valid;
  }
});

// node_modules/semver/functions/clean.js
var require_clean = __commonJS({
  "node_modules/semver/functions/clean.js"(exports2, module2) {
    var parse2 = require_parse2();
    var clean = (version, options) => {
      const s = parse2(version.trim().replace(/^[=v]+/, ""), options);
      return s ? s.version : null;
    };
    module2.exports = clean;
  }
});

// node_modules/semver/functions/inc.js
var require_inc = __commonJS({
  "node_modules/semver/functions/inc.js"(exports2, module2) {
    var SemVer = require_semver();
    var inc = (version, release, options, identifier, identifierBase) => {
      if (typeof options === "string") {
        identifierBase = identifier;
        identifier = options;
        options = void 0;
      }
      try {
        return new SemVer(
          version instanceof SemVer ? version.version : version,
          options
        ).inc(release, identifier, identifierBase).version;
      } catch (er) {
        return null;
      }
    };
    module2.exports = inc;
  }
});

// node_modules/semver/functions/diff.js
var require_diff = __commonJS({
  "node_modules/semver/functions/diff.js"(exports2, module2) {
    var parse2 = require_parse2();
    var diff = (version1, version2) => {
      const v1 = parse2(version1, null, true);
      const v2 = parse2(version2, null, true);
      const comparison = v1.compare(v2);
      if (comparison === 0) {
        return null;
      }
      const v1Higher = comparison > 0;
      const highVersion = v1Higher ? v1 : v2;
      const lowVersion = v1Higher ? v2 : v1;
      const highHasPre = !!highVersion.prerelease.length;
      const lowHasPre = !!lowVersion.prerelease.length;
      if (lowHasPre && !highHasPre) {
        if (!lowVersion.patch && !lowVersion.minor) {
          return "major";
        }
        if (highVersion.patch) {
          return "patch";
        }
        if (highVersion.minor) {
          return "minor";
        }
        return "major";
      }
      const prefix = highHasPre ? "pre" : "";
      if (v1.major !== v2.major) {
        return prefix + "major";
      }
      if (v1.minor !== v2.minor) {
        return prefix + "minor";
      }
      if (v1.patch !== v2.patch) {
        return prefix + "patch";
      }
      return "prerelease";
    };
    module2.exports = diff;
  }
});

// node_modules/semver/functions/major.js
var require_major = __commonJS({
  "node_modules/semver/functions/major.js"(exports2, module2) {
    var SemVer = require_semver();
    var major = (a, loose) => new SemVer(a, loose).major;
    module2.exports = major;
  }
});

// node_modules/semver/functions/minor.js
var require_minor = __commonJS({
  "node_modules/semver/functions/minor.js"(exports2, module2) {
    var SemVer = require_semver();
    var minor = (a, loose) => new SemVer(a, loose).minor;
    module2.exports = minor;
  }
});

// node_modules/semver/functions/patch.js
var require_patch = __commonJS({
  "node_modules/semver/functions/patch.js"(exports2, module2) {
    var SemVer = require_semver();
    var patch = (a, loose) => new SemVer(a, loose).patch;
    module2.exports = patch;
  }
});

// node_modules/semver/functions/prerelease.js
var require_prerelease = __commonJS({
  "node_modules/semver/functions/prerelease.js"(exports2, module2) {
    var parse2 = require_parse2();
    var prerelease = (version, options) => {
      const parsed = parse2(version, options);
      return parsed && parsed.prerelease.length ? parsed.prerelease : null;
    };
    module2.exports = prerelease;
  }
});

// node_modules/semver/functions/compare.js
var require_compare = __commonJS({
  "node_modules/semver/functions/compare.js"(exports2, module2) {
    var SemVer = require_semver();
    var compare = (a, b, loose) => new SemVer(a, loose).compare(new SemVer(b, loose));
    module2.exports = compare;
  }
});

// node_modules/semver/functions/rcompare.js
var require_rcompare = __commonJS({
  "node_modules/semver/functions/rcompare.js"(exports2, module2) {
    var compare = require_compare();
    var rcompare = (a, b, loose) => compare(b, a, loose);
    module2.exports = rcompare;
  }
});

// node_modules/semver/functions/compare-loose.js
var require_compare_loose = __commonJS({
  "node_modules/semver/functions/compare-loose.js"(exports2, module2) {
    var compare = require_compare();
    var compareLoose = (a, b) => compare(a, b, true);
    module2.exports = compareLoose;
  }
});

// node_modules/semver/functions/compare-build.js
var require_compare_build = __commonJS({
  "node_modules/semver/functions/compare-build.js"(exports2, module2) {
    var SemVer = require_semver();
    var compareBuild = (a, b, loose) => {
      const versionA = new SemVer(a, loose);
      const versionB = new SemVer(b, loose);
      return versionA.compare(versionB) || versionA.compareBuild(versionB);
    };
    module2.exports = compareBuild;
  }
});

// node_modules/semver/functions/sort.js
var require_sort = __commonJS({
  "node_modules/semver/functions/sort.js"(exports2, module2) {
    var compareBuild = require_compare_build();
    var sort = (list, loose) => list.sort((a, b) => compareBuild(a, b, loose));
    module2.exports = sort;
  }
});

// node_modules/semver/functions/rsort.js
var require_rsort = __commonJS({
  "node_modules/semver/functions/rsort.js"(exports2, module2) {
    var compareBuild = require_compare_build();
    var rsort = (list, loose) => list.sort((a, b) => compareBuild(b, a, loose));
    module2.exports = rsort;
  }
});

// node_modules/semver/functions/gt.js
var require_gt = __commonJS({
  "node_modules/semver/functions/gt.js"(exports2, module2) {
    var compare = require_compare();
    var gt = (a, b, loose) => compare(a, b, loose) > 0;
    module2.exports = gt;
  }
});

// node_modules/semver/functions/lt.js
var require_lt = __commonJS({
  "node_modules/semver/functions/lt.js"(exports2, module2) {
    var compare = require_compare();
    var lt = (a, b, loose) => compare(a, b, loose) < 0;
    module2.exports = lt;
  }
});

// node_modules/semver/functions/eq.js
var require_eq = __commonJS({
  "node_modules/semver/functions/eq.js"(exports2, module2) {
    var compare = require_compare();
    var eq = (a, b, loose) => compare(a, b, loose) === 0;
    module2.exports = eq;
  }
});

// node_modules/semver/functions/neq.js
var require_neq = __commonJS({
  "node_modules/semver/functions/neq.js"(exports2, module2) {
    var compare = require_compare();
    var neq = (a, b, loose) => compare(a, b, loose) !== 0;
    module2.exports = neq;
  }
});

// node_modules/semver/functions/gte.js
var require_gte = __commonJS({
  "node_modules/semver/functions/gte.js"(exports2, module2) {
    var compare = require_compare();
    var gte = (a, b, loose) => compare(a, b, loose) >= 0;
    module2.exports = gte;
  }
});

// node_modules/semver/functions/lte.js
var require_lte = __commonJS({
  "node_modules/semver/functions/lte.js"(exports2, module2) {
    var compare = require_compare();
    var lte = (a, b, loose) => compare(a, b, loose) <= 0;
    module2.exports = lte;
  }
});

// node_modules/semver/functions/cmp.js
var require_cmp = __commonJS({
  "node_modules/semver/functions/cmp.js"(exports2, module2) {
    var eq = require_eq();
    var neq = require_neq();
    var gt = require_gt();
    var gte = require_gte();
    var lt = require_lt();
    var lte = require_lte();
    var cmp = (a, op, b, loose) => {
      switch (op) {
        case "===":
          if (typeof a === "object") {
            a = a.version;
          }
          if (typeof b === "object") {
            b = b.version;
          }
          return a === b;
        case "!==":
          if (typeof a === "object") {
            a = a.version;
          }
          if (typeof b === "object") {
            b = b.version;
          }
          return a !== b;
        case "":
        case "=":
        case "==":
          return eq(a, b, loose);
        case "!=":
          return neq(a, b, loose);
        case ">":
          return gt(a, b, loose);
        case ">=":
          return gte(a, b, loose);
        case "<":
          return lt(a, b, loose);
        case "<=":
          return lte(a, b, loose);
        default:
          throw new TypeError(`Invalid operator: ${op}`);
      }
    };
    module2.exports = cmp;
  }
});

// node_modules/semver/functions/coerce.js
var require_coerce = __commonJS({
  "node_modules/semver/functions/coerce.js"(exports2, module2) {
    var SemVer = require_semver();
    var parse2 = require_parse2();
    var { safeRe: re, t } = require_re();
    var coerce = (version, options) => {
      if (version instanceof SemVer) {
        return version;
      }
      if (typeof version === "number") {
        version = String(version);
      }
      if (typeof version !== "string") {
        return null;
      }
      options = options || {};
      let match = null;
      if (!options.rtl) {
        match = version.match(re[t.COERCE]);
      } else {
        let next;
        while ((next = re[t.COERCERTL].exec(version)) && (!match || match.index + match[0].length !== version.length)) {
          if (!match || next.index + next[0].length !== match.index + match[0].length) {
            match = next;
          }
          re[t.COERCERTL].lastIndex = next.index + next[1].length + next[2].length;
        }
        re[t.COERCERTL].lastIndex = -1;
      }
      if (match === null) {
        return null;
      }
      return parse2(`${match[2]}.${match[3] || "0"}.${match[4] || "0"}`, options);
    };
    module2.exports = coerce;
  }
});

// node_modules/yallist/iterator.js
var require_iterator = __commonJS({
  "node_modules/yallist/iterator.js"(exports2, module2) {
    "use strict";
    module2.exports = function(Yallist) {
      Yallist.prototype[Symbol.iterator] = function* () {
        for (let walker = this.head; walker; walker = walker.next) {
          yield walker.value;
        }
      };
    };
  }
});

// node_modules/yallist/yallist.js
var require_yallist = __commonJS({
  "node_modules/yallist/yallist.js"(exports2, module2) {
    "use strict";
    module2.exports = Yallist;
    Yallist.Node = Node;
    Yallist.create = Yallist;
    function Yallist(list) {
      var self = this;
      if (!(self instanceof Yallist)) {
        self = new Yallist();
      }
      self.tail = null;
      self.head = null;
      self.length = 0;
      if (list && typeof list.forEach === "function") {
        list.forEach(function(item) {
          self.push(item);
        });
      } else if (arguments.length > 0) {
        for (var i = 0, l = arguments.length; i < l; i++) {
          self.push(arguments[i]);
        }
      }
      return self;
    }
    Yallist.prototype.removeNode = function(node) {
      if (node.list !== this) {
        throw new Error("removing node which does not belong to this list");
      }
      var next = node.next;
      var prev = node.prev;
      if (next) {
        next.prev = prev;
      }
      if (prev) {
        prev.next = next;
      }
      if (node === this.head) {
        this.head = next;
      }
      if (node === this.tail) {
        this.tail = prev;
      }
      node.list.length--;
      node.next = null;
      node.prev = null;
      node.list = null;
      return next;
    };
    Yallist.prototype.unshiftNode = function(node) {
      if (node === this.head) {
        return;
      }
      if (node.list) {
        node.list.removeNode(node);
      }
      var head = this.head;
      node.list = this;
      node.next = head;
      if (head) {
        head.prev = node;
      }
      this.head = node;
      if (!this.tail) {
        this.tail = node;
      }
      this.length++;
    };
    Yallist.prototype.pushNode = function(node) {
      if (node === this.tail) {
        return;
      }
      if (node.list) {
        node.list.removeNode(node);
      }
      var tail = this.tail;
      node.list = this;
      node.prev = tail;
      if (tail) {
        tail.next = node;
      }
      this.tail = node;
      if (!this.head) {
        this.head = node;
      }
      this.length++;
    };
    Yallist.prototype.push = function() {
      for (var i = 0, l = arguments.length; i < l; i++) {
        push(this, arguments[i]);
      }
      return this.length;
    };
    Yallist.prototype.unshift = function() {
      for (var i = 0, l = arguments.length; i < l; i++) {
        unshift(this, arguments[i]);
      }
      return this.length;
    };
    Yallist.prototype.pop = function() {
      if (!this.tail) {
        return void 0;
      }
      var res = this.tail.value;
      this.tail = this.tail.prev;
      if (this.tail) {
        this.tail.next = null;
      } else {
        this.head = null;
      }
      this.length--;
      return res;
    };
    Yallist.prototype.shift = function() {
      if (!this.head) {
        return void 0;
      }
      var res = this.head.value;
      this.head = this.head.next;
      if (this.head) {
        this.head.prev = null;
      } else {
        this.tail = null;
      }
      this.length--;
      return res;
    };
    Yallist.prototype.forEach = function(fn, thisp) {
      thisp = thisp || this;
      for (var walker = this.head, i = 0; walker !== null; i++) {
        fn.call(thisp, walker.value, i, this);
        walker = walker.next;
      }
    };
    Yallist.prototype.forEachReverse = function(fn, thisp) {
      thisp = thisp || this;
      for (var walker = this.tail, i = this.length - 1; walker !== null; i--) {
        fn.call(thisp, walker.value, i, this);
        walker = walker.prev;
      }
    };
    Yallist.prototype.get = function(n) {
      for (var i = 0, walker = this.head; walker !== null && i < n; i++) {
        walker = walker.next;
      }
      if (i === n && walker !== null) {
        return walker.value;
      }
    };
    Yallist.prototype.getReverse = function(n) {
      for (var i = 0, walker = this.tail; walker !== null && i < n; i++) {
        walker = walker.prev;
      }
      if (i === n && walker !== null) {
        return walker.value;
      }
    };
    Yallist.prototype.map = function(fn, thisp) {
      thisp = thisp || this;
      var res = new Yallist();
      for (var walker = this.head; walker !== null; ) {
        res.push(fn.call(thisp, walker.value, this));
        walker = walker.next;
      }
      return res;
    };
    Yallist.prototype.mapReverse = function(fn, thisp) {
      thisp = thisp || this;
      var res = new Yallist();
      for (var walker = this.tail; walker !== null; ) {
        res.push(fn.call(thisp, walker.value, this));
        walker = walker.prev;
      }
      return res;
    };
    Yallist.prototype.reduce = function(fn, initial) {
      var acc;
      var walker = this.head;
      if (arguments.length > 1) {
        acc = initial;
      } else if (this.head) {
        walker = this.head.next;
        acc = this.head.value;
      } else {
        throw new TypeError("Reduce of empty list with no initial value");
      }
      for (var i = 0; walker !== null; i++) {
        acc = fn(acc, walker.value, i);
        walker = walker.next;
      }
      return acc;
    };
    Yallist.prototype.reduceReverse = function(fn, initial) {
      var acc;
      var walker = this.tail;
      if (arguments.length > 1) {
        acc = initial;
      } else if (this.tail) {
        walker = this.tail.prev;
        acc = this.tail.value;
      } else {
        throw new TypeError("Reduce of empty list with no initial value");
      }
      for (var i = this.length - 1; walker !== null; i--) {
        acc = fn(acc, walker.value, i);
        walker = walker.prev;
      }
      return acc;
    };
    Yallist.prototype.toArray = function() {
      var arr = new Array(this.length);
      for (var i = 0, walker = this.head; walker !== null; i++) {
        arr[i] = walker.value;
        walker = walker.next;
      }
      return arr;
    };
    Yallist.prototype.toArrayReverse = function() {
      var arr = new Array(this.length);
      for (var i = 0, walker = this.tail; walker !== null; i++) {
        arr[i] = walker.value;
        walker = walker.prev;
      }
      return arr;
    };
    Yallist.prototype.slice = function(from, to) {
      to = to || this.length;
      if (to < 0) {
        to += this.length;
      }
      from = from || 0;
      if (from < 0) {
        from += this.length;
      }
      var ret = new Yallist();
      if (to < from || to < 0) {
        return ret;
      }
      if (from < 0) {
        from = 0;
      }
      if (to > this.length) {
        to = this.length;
      }
      for (var i = 0, walker = this.head; walker !== null && i < from; i++) {
        walker = walker.next;
      }
      for (; walker !== null && i < to; i++, walker = walker.next) {
        ret.push(walker.value);
      }
      return ret;
    };
    Yallist.prototype.sliceReverse = function(from, to) {
      to = to || this.length;
      if (to < 0) {
        to += this.length;
      }
      from = from || 0;
      if (from < 0) {
        from += this.length;
      }
      var ret = new Yallist();
      if (to < from || to < 0) {
        return ret;
      }
      if (from < 0) {
        from = 0;
      }
      if (to > this.length) {
        to = this.length;
      }
      for (var i = this.length, walker = this.tail; walker !== null && i > to; i--) {
        walker = walker.prev;
      }
      for (; walker !== null && i > from; i--, walker = walker.prev) {
        ret.push(walker.value);
      }
      return ret;
    };
    Yallist.prototype.splice = function(start, deleteCount, ...nodes) {
      if (start > this.length) {
        start = this.length - 1;
      }
      if (start < 0) {
        start = this.length + start;
      }
      for (var i = 0, walker = this.head; walker !== null && i < start; i++) {
        walker = walker.next;
      }
      var ret = [];
      for (var i = 0; walker && i < deleteCount; i++) {
        ret.push(walker.value);
        walker = this.removeNode(walker);
      }
      if (walker === null) {
        walker = this.tail;
      }
      if (walker !== this.head && walker !== this.tail) {
        walker = walker.prev;
      }
      for (var i = 0; i < nodes.length; i++) {
        walker = insert(this, walker, nodes[i]);
      }
      return ret;
    };
    Yallist.prototype.reverse = function() {
      var head = this.head;
      var tail = this.tail;
      for (var walker = head; walker !== null; walker = walker.prev) {
        var p = walker.prev;
        walker.prev = walker.next;
        walker.next = p;
      }
      this.head = tail;
      this.tail = head;
      return this;
    };
    function insert(self, node, value) {
      var inserted = node === self.head ? new Node(value, null, node, self) : new Node(value, node, node.next, self);
      if (inserted.next === null) {
        self.tail = inserted;
      }
      if (inserted.prev === null) {
        self.head = inserted;
      }
      self.length++;
      return inserted;
    }
    function push(self, item) {
      self.tail = new Node(item, self.tail, null, self);
      if (!self.head) {
        self.head = self.tail;
      }
      self.length++;
    }
    function unshift(self, item) {
      self.head = new Node(item, null, self.head, self);
      if (!self.tail) {
        self.tail = self.head;
      }
      self.length++;
    }
    function Node(value, prev, next, list) {
      if (!(this instanceof Node)) {
        return new Node(value, prev, next, list);
      }
      this.list = list;
      this.value = value;
      if (prev) {
        prev.next = this;
        this.prev = prev;
      } else {
        this.prev = null;
      }
      if (next) {
        next.prev = this;
        this.next = next;
      } else {
        this.next = null;
      }
    }
    try {
      require_iterator()(Yallist);
    } catch (er) {
    }
  }
});

// node_modules/lru-cache/index.js
var require_lru_cache = __commonJS({
  "node_modules/lru-cache/index.js"(exports2, module2) {
    "use strict";
    var Yallist = require_yallist();
    var MAX = Symbol("max");
    var LENGTH = Symbol("length");
    var LENGTH_CALCULATOR = Symbol("lengthCalculator");
    var ALLOW_STALE = Symbol("allowStale");
    var MAX_AGE = Symbol("maxAge");
    var DISPOSE = Symbol("dispose");
    var NO_DISPOSE_ON_SET = Symbol("noDisposeOnSet");
    var LRU_LIST = Symbol("lruList");
    var CACHE = Symbol("cache");
    var UPDATE_AGE_ON_GET = Symbol("updateAgeOnGet");
    var naiveLength = () => 1;
    var LRUCache = class {
      constructor(options) {
        if (typeof options === "number")
          options = { max: options };
        if (!options)
          options = {};
        if (options.max && (typeof options.max !== "number" || options.max < 0))
          throw new TypeError("max must be a non-negative number");
        const max = this[MAX] = options.max || Infinity;
        const lc = options.length || naiveLength;
        this[LENGTH_CALCULATOR] = typeof lc !== "function" ? naiveLength : lc;
        this[ALLOW_STALE] = options.stale || false;
        if (options.maxAge && typeof options.maxAge !== "number")
          throw new TypeError("maxAge must be a number");
        this[MAX_AGE] = options.maxAge || 0;
        this[DISPOSE] = options.dispose;
        this[NO_DISPOSE_ON_SET] = options.noDisposeOnSet || false;
        this[UPDATE_AGE_ON_GET] = options.updateAgeOnGet || false;
        this.reset();
      }
      // resize the cache when the max changes.
      set max(mL) {
        if (typeof mL !== "number" || mL < 0)
          throw new TypeError("max must be a non-negative number");
        this[MAX] = mL || Infinity;
        trim(this);
      }
      get max() {
        return this[MAX];
      }
      set allowStale(allowStale) {
        this[ALLOW_STALE] = !!allowStale;
      }
      get allowStale() {
        return this[ALLOW_STALE];
      }
      set maxAge(mA) {
        if (typeof mA !== "number")
          throw new TypeError("maxAge must be a non-negative number");
        this[MAX_AGE] = mA;
        trim(this);
      }
      get maxAge() {
        return this[MAX_AGE];
      }
      // resize the cache when the lengthCalculator changes.
      set lengthCalculator(lC) {
        if (typeof lC !== "function")
          lC = naiveLength;
        if (lC !== this[LENGTH_CALCULATOR]) {
          this[LENGTH_CALCULATOR] = lC;
          this[LENGTH] = 0;
          this[LRU_LIST].forEach((hit) => {
            hit.length = this[LENGTH_CALCULATOR](hit.value, hit.key);
            this[LENGTH] += hit.length;
          });
        }
        trim(this);
      }
      get lengthCalculator() {
        return this[LENGTH_CALCULATOR];
      }
      get length() {
        return this[LENGTH];
      }
      get itemCount() {
        return this[LRU_LIST].length;
      }
      rforEach(fn, thisp) {
        thisp = thisp || this;
        for (let walker = this[LRU_LIST].tail; walker !== null; ) {
          const prev = walker.prev;
          forEachStep(this, fn, walker, thisp);
          walker = prev;
        }
      }
      forEach(fn, thisp) {
        thisp = thisp || this;
        for (let walker = this[LRU_LIST].head; walker !== null; ) {
          const next = walker.next;
          forEachStep(this, fn, walker, thisp);
          walker = next;
        }
      }
      keys() {
        return this[LRU_LIST].toArray().map((k) => k.key);
      }
      values() {
        return this[LRU_LIST].toArray().map((k) => k.value);
      }
      reset() {
        if (this[DISPOSE] && this[LRU_LIST] && this[LRU_LIST].length) {
          this[LRU_LIST].forEach((hit) => this[DISPOSE](hit.key, hit.value));
        }
        this[CACHE] = /* @__PURE__ */ new Map();
        this[LRU_LIST] = new Yallist();
        this[LENGTH] = 0;
      }
      dump() {
        return this[LRU_LIST].map((hit) => isStale(this, hit) ? false : {
          k: hit.key,
          v: hit.value,
          e: hit.now + (hit.maxAge || 0)
        }).toArray().filter((h) => h);
      }
      dumpLru() {
        return this[LRU_LIST];
      }
      set(key, value, maxAge) {
        maxAge = maxAge || this[MAX_AGE];
        if (maxAge && typeof maxAge !== "number")
          throw new TypeError("maxAge must be a number");
        const now = maxAge ? Date.now() : 0;
        const len = this[LENGTH_CALCULATOR](value, key);
        if (this[CACHE].has(key)) {
          if (len > this[MAX]) {
            del(this, this[CACHE].get(key));
            return false;
          }
          const node = this[CACHE].get(key);
          const item = node.value;
          if (this[DISPOSE]) {
            if (!this[NO_DISPOSE_ON_SET])
              this[DISPOSE](key, item.value);
          }
          item.now = now;
          item.maxAge = maxAge;
          item.value = value;
          this[LENGTH] += len - item.length;
          item.length = len;
          this.get(key);
          trim(this);
          return true;
        }
        const hit = new Entry(key, value, len, now, maxAge);
        if (hit.length > this[MAX]) {
          if (this[DISPOSE])
            this[DISPOSE](key, value);
          return false;
        }
        this[LENGTH] += hit.length;
        this[LRU_LIST].unshift(hit);
        this[CACHE].set(key, this[LRU_LIST].head);
        trim(this);
        return true;
      }
      has(key) {
        if (!this[CACHE].has(key))
          return false;
        const hit = this[CACHE].get(key).value;
        return !isStale(this, hit);
      }
      get(key) {
        return get(this, key, true);
      }
      peek(key) {
        return get(this, key, false);
      }
      pop() {
        const node = this[LRU_LIST].tail;
        if (!node)
          return null;
        del(this, node);
        return node.value;
      }
      del(key) {
        del(this, this[CACHE].get(key));
      }
      load(arr) {
        this.reset();
        const now = Date.now();
        for (let l = arr.length - 1; l >= 0; l--) {
          const hit = arr[l];
          const expiresAt = hit.e || 0;
          if (expiresAt === 0)
            this.set(hit.k, hit.v);
          else {
            const maxAge = expiresAt - now;
            if (maxAge > 0) {
              this.set(hit.k, hit.v, maxAge);
            }
          }
        }
      }
      prune() {
        this[CACHE].forEach((value, key) => get(this, key, false));
      }
    };
    var get = (self, key, doUse) => {
      const node = self[CACHE].get(key);
      if (node) {
        const hit = node.value;
        if (isStale(self, hit)) {
          del(self, node);
          if (!self[ALLOW_STALE])
            return void 0;
        } else {
          if (doUse) {
            if (self[UPDATE_AGE_ON_GET])
              node.value.now = Date.now();
            self[LRU_LIST].unshiftNode(node);
          }
        }
        return hit.value;
      }
    };
    var isStale = (self, hit) => {
      if (!hit || !hit.maxAge && !self[MAX_AGE])
        return false;
      const diff = Date.now() - hit.now;
      return hit.maxAge ? diff > hit.maxAge : self[MAX_AGE] && diff > self[MAX_AGE];
    };
    var trim = (self) => {
      if (self[LENGTH] > self[MAX]) {
        for (let walker = self[LRU_LIST].tail; self[LENGTH] > self[MAX] && walker !== null; ) {
          const prev = walker.prev;
          del(self, walker);
          walker = prev;
        }
      }
    };
    var del = (self, node) => {
      if (node) {
        const hit = node.value;
        if (self[DISPOSE])
          self[DISPOSE](hit.key, hit.value);
        self[LENGTH] -= hit.length;
        self[CACHE].delete(hit.key);
        self[LRU_LIST].removeNode(node);
      }
    };
    var Entry = class {
      constructor(key, value, length, now, maxAge) {
        this.key = key;
        this.value = value;
        this.length = length;
        this.now = now;
        this.maxAge = maxAge || 0;
      }
    };
    var forEachStep = (self, fn, node, thisp) => {
      let hit = node.value;
      if (isStale(self, hit)) {
        del(self, node);
        if (!self[ALLOW_STALE])
          hit = void 0;
      }
      if (hit)
        fn.call(thisp, hit.value, hit.key, self);
    };
    module2.exports = LRUCache;
  }
});

// node_modules/semver/classes/range.js
var require_range = __commonJS({
  "node_modules/semver/classes/range.js"(exports2, module2) {
    var Range = class _Range {
      constructor(range, options) {
        options = parseOptions(options);
        if (range instanceof _Range) {
          if (range.loose === !!options.loose && range.includePrerelease === !!options.includePrerelease) {
            return range;
          } else {
            return new _Range(range.raw, options);
          }
        }
        if (range instanceof Comparator) {
          this.raw = range.value;
          this.set = [[range]];
          this.format();
          return this;
        }
        this.options = options;
        this.loose = !!options.loose;
        this.includePrerelease = !!options.includePrerelease;
        this.raw = range.trim().split(/\s+/).join(" ");
        this.set = this.raw.split("||").map((r) => this.parseRange(r.trim())).filter((c) => c.length);
        if (!this.set.length) {
          throw new TypeError(`Invalid SemVer Range: ${this.raw}`);
        }
        if (this.set.length > 1) {
          const first = this.set[0];
          this.set = this.set.filter((c) => !isNullSet(c[0]));
          if (this.set.length === 0) {
            this.set = [first];
          } else if (this.set.length > 1) {
            for (const c of this.set) {
              if (c.length === 1 && isAny(c[0])) {
                this.set = [c];
                break;
              }
            }
          }
        }
        this.format();
      }
      format() {
        this.range = this.set.map((comps) => comps.join(" ").trim()).join("||").trim();
        return this.range;
      }
      toString() {
        return this.range;
      }
      parseRange(range) {
        const memoOpts = (this.options.includePrerelease && FLAG_INCLUDE_PRERELEASE) | (this.options.loose && FLAG_LOOSE);
        const memoKey = memoOpts + ":" + range;
        const cached = cache.get(memoKey);
        if (cached) {
          return cached;
        }
        const loose = this.options.loose;
        const hr = loose ? re[t.HYPHENRANGELOOSE] : re[t.HYPHENRANGE];
        range = range.replace(hr, hyphenReplace(this.options.includePrerelease));
        debug("hyphen replace", range);
        range = range.replace(re[t.COMPARATORTRIM], comparatorTrimReplace);
        debug("comparator trim", range);
        range = range.replace(re[t.TILDETRIM], tildeTrimReplace);
        debug("tilde trim", range);
        range = range.replace(re[t.CARETTRIM], caretTrimReplace);
        debug("caret trim", range);
        let rangeList = range.split(" ").map((comp) => parseComparator(comp, this.options)).join(" ").split(/\s+/).map((comp) => replaceGTE0(comp, this.options));
        if (loose) {
          rangeList = rangeList.filter((comp) => {
            debug("loose invalid filter", comp, this.options);
            return !!comp.match(re[t.COMPARATORLOOSE]);
          });
        }
        debug("range list", rangeList);
        const rangeMap = /* @__PURE__ */ new Map();
        const comparators = rangeList.map((comp) => new Comparator(comp, this.options));
        for (const comp of comparators) {
          if (isNullSet(comp)) {
            return [comp];
          }
          rangeMap.set(comp.value, comp);
        }
        if (rangeMap.size > 1 && rangeMap.has("")) {
          rangeMap.delete("");
        }
        const result = [...rangeMap.values()];
        cache.set(memoKey, result);
        return result;
      }
      intersects(range, options) {
        if (!(range instanceof _Range)) {
          throw new TypeError("a Range is required");
        }
        return this.set.some((thisComparators) => {
          return isSatisfiable(thisComparators, options) && range.set.some((rangeComparators) => {
            return isSatisfiable(rangeComparators, options) && thisComparators.every((thisComparator) => {
              return rangeComparators.every((rangeComparator) => {
                return thisComparator.intersects(rangeComparator, options);
              });
            });
          });
        });
      }
      // if ANY of the sets match ALL of its comparators, then pass
      test(version) {
        if (!version) {
          return false;
        }
        if (typeof version === "string") {
          try {
            version = new SemVer(version, this.options);
          } catch (er) {
            return false;
          }
        }
        for (let i = 0; i < this.set.length; i++) {
          if (testSet(this.set[i], version, this.options)) {
            return true;
          }
        }
        return false;
      }
    };
    module2.exports = Range;
    var LRU = require_lru_cache();
    var cache = new LRU({ max: 1e3 });
    var parseOptions = require_parse_options();
    var Comparator = require_comparator();
    var debug = require_debug();
    var SemVer = require_semver();
    var {
      safeRe: re,
      t,
      comparatorTrimReplace,
      tildeTrimReplace,
      caretTrimReplace
    } = require_re();
    var { FLAG_INCLUDE_PRERELEASE, FLAG_LOOSE } = require_constants2();
    var isNullSet = (c) => c.value === "<0.0.0-0";
    var isAny = (c) => c.value === "";
    var isSatisfiable = (comparators, options) => {
      let result = true;
      const remainingComparators = comparators.slice();
      let testComparator = remainingComparators.pop();
      while (result && remainingComparators.length) {
        result = remainingComparators.every((otherComparator) => {
          return testComparator.intersects(otherComparator, options);
        });
        testComparator = remainingComparators.pop();
      }
      return result;
    };
    var parseComparator = (comp, options) => {
      debug("comp", comp, options);
      comp = replaceCarets(comp, options);
      debug("caret", comp);
      comp = replaceTildes(comp, options);
      debug("tildes", comp);
      comp = replaceXRanges(comp, options);
      debug("xrange", comp);
      comp = replaceStars(comp, options);
      debug("stars", comp);
      return comp;
    };
    var isX = (id) => !id || id.toLowerCase() === "x" || id === "*";
    var replaceTildes = (comp, options) => {
      return comp.trim().split(/\s+/).map((c) => replaceTilde(c, options)).join(" ");
    };
    var replaceTilde = (comp, options) => {
      const r = options.loose ? re[t.TILDELOOSE] : re[t.TILDE];
      return comp.replace(r, (_, M, m, p, pr) => {
        debug("tilde", comp, _, M, m, p, pr);
        let ret;
        if (isX(M)) {
          ret = "";
        } else if (isX(m)) {
          ret = `>=${M}.0.0 <${+M + 1}.0.0-0`;
        } else if (isX(p)) {
          ret = `>=${M}.${m}.0 <${M}.${+m + 1}.0-0`;
        } else if (pr) {
          debug("replaceTilde pr", pr);
          ret = `>=${M}.${m}.${p}-${pr} <${M}.${+m + 1}.0-0`;
        } else {
          ret = `>=${M}.${m}.${p} <${M}.${+m + 1}.0-0`;
        }
        debug("tilde return", ret);
        return ret;
      });
    };
    var replaceCarets = (comp, options) => {
      return comp.trim().split(/\s+/).map((c) => replaceCaret(c, options)).join(" ");
    };
    var replaceCaret = (comp, options) => {
      debug("caret", comp, options);
      const r = options.loose ? re[t.CARETLOOSE] : re[t.CARET];
      const z = options.includePrerelease ? "-0" : "";
      return comp.replace(r, (_, M, m, p, pr) => {
        debug("caret", comp, _, M, m, p, pr);
        let ret;
        if (isX(M)) {
          ret = "";
        } else if (isX(m)) {
          ret = `>=${M}.0.0${z} <${+M + 1}.0.0-0`;
        } else if (isX(p)) {
          if (M === "0") {
            ret = `>=${M}.${m}.0${z} <${M}.${+m + 1}.0-0`;
          } else {
            ret = `>=${M}.${m}.0${z} <${+M + 1}.0.0-0`;
          }
        } else if (pr) {
          debug("replaceCaret pr", pr);
          if (M === "0") {
            if (m === "0") {
              ret = `>=${M}.${m}.${p}-${pr} <${M}.${m}.${+p + 1}-0`;
            } else {
              ret = `>=${M}.${m}.${p}-${pr} <${M}.${+m + 1}.0-0`;
            }
          } else {
            ret = `>=${M}.${m}.${p}-${pr} <${+M + 1}.0.0-0`;
          }
        } else {
          debug("no pr");
          if (M === "0") {
            if (m === "0") {
              ret = `>=${M}.${m}.${p}${z} <${M}.${m}.${+p + 1}-0`;
            } else {
              ret = `>=${M}.${m}.${p}${z} <${M}.${+m + 1}.0-0`;
            }
          } else {
            ret = `>=${M}.${m}.${p} <${+M + 1}.0.0-0`;
          }
        }
        debug("caret return", ret);
        return ret;
      });
    };
    var replaceXRanges = (comp, options) => {
      debug("replaceXRanges", comp, options);
      return comp.split(/\s+/).map((c) => replaceXRange(c, options)).join(" ");
    };
    var replaceXRange = (comp, options) => {
      comp = comp.trim();
      const r = options.loose ? re[t.XRANGELOOSE] : re[t.XRANGE];
      return comp.replace(r, (ret, gtlt, M, m, p, pr) => {
        debug("xRange", comp, ret, gtlt, M, m, p, pr);
        const xM = isX(M);
        const xm = xM || isX(m);
        const xp = xm || isX(p);
        const anyX = xp;
        if (gtlt === "=" && anyX) {
          gtlt = "";
        }
        pr = options.includePrerelease ? "-0" : "";
        if (xM) {
          if (gtlt === ">" || gtlt === "<") {
            ret = "<0.0.0-0";
          } else {
            ret = "*";
          }
        } else if (gtlt && anyX) {
          if (xm) {
            m = 0;
          }
          p = 0;
          if (gtlt === ">") {
            gtlt = ">=";
            if (xm) {
              M = +M + 1;
              m = 0;
              p = 0;
            } else {
              m = +m + 1;
              p = 0;
            }
          } else if (gtlt === "<=") {
            gtlt = "<";
            if (xm) {
              M = +M + 1;
            } else {
              m = +m + 1;
            }
          }
          if (gtlt === "<") {
            pr = "-0";
          }
          ret = `${gtlt + M}.${m}.${p}${pr}`;
        } else if (xm) {
          ret = `>=${M}.0.0${pr} <${+M + 1}.0.0-0`;
        } else if (xp) {
          ret = `>=${M}.${m}.0${pr} <${M}.${+m + 1}.0-0`;
        }
        debug("xRange return", ret);
        return ret;
      });
    };
    var replaceStars = (comp, options) => {
      debug("replaceStars", comp, options);
      return comp.trim().replace(re[t.STAR], "");
    };
    var replaceGTE0 = (comp, options) => {
      debug("replaceGTE0", comp, options);
      return comp.trim().replace(re[options.includePrerelease ? t.GTE0PRE : t.GTE0], "");
    };
    var hyphenReplace = (incPr) => ($0, from, fM, fm, fp, fpr, fb, to, tM, tm, tp, tpr, tb) => {
      if (isX(fM)) {
        from = "";
      } else if (isX(fm)) {
        from = `>=${fM}.0.0${incPr ? "-0" : ""}`;
      } else if (isX(fp)) {
        from = `>=${fM}.${fm}.0${incPr ? "-0" : ""}`;
      } else if (fpr) {
        from = `>=${from}`;
      } else {
        from = `>=${from}${incPr ? "-0" : ""}`;
      }
      if (isX(tM)) {
        to = "";
      } else if (isX(tm)) {
        to = `<${+tM + 1}.0.0-0`;
      } else if (isX(tp)) {
        to = `<${tM}.${+tm + 1}.0-0`;
      } else if (tpr) {
        to = `<=${tM}.${tm}.${tp}-${tpr}`;
      } else if (incPr) {
        to = `<${tM}.${tm}.${+tp + 1}-0`;
      } else {
        to = `<=${to}`;
      }
      return `${from} ${to}`.trim();
    };
    var testSet = (set, version, options) => {
      for (let i = 0; i < set.length; i++) {
        if (!set[i].test(version)) {
          return false;
        }
      }
      if (version.prerelease.length && !options.includePrerelease) {
        for (let i = 0; i < set.length; i++) {
          debug(set[i].semver);
          if (set[i].semver === Comparator.ANY) {
            continue;
          }
          if (set[i].semver.prerelease.length > 0) {
            const allowed = set[i].semver;
            if (allowed.major === version.major && allowed.minor === version.minor && allowed.patch === version.patch) {
              return true;
            }
          }
        }
        return false;
      }
      return true;
    };
  }
});

// node_modules/semver/classes/comparator.js
var require_comparator = __commonJS({
  "node_modules/semver/classes/comparator.js"(exports2, module2) {
    var ANY = Symbol("SemVer ANY");
    var Comparator = class _Comparator {
      static get ANY() {
        return ANY;
      }
      constructor(comp, options) {
        options = parseOptions(options);
        if (comp instanceof _Comparator) {
          if (comp.loose === !!options.loose) {
            return comp;
          } else {
            comp = comp.value;
          }
        }
        comp = comp.trim().split(/\s+/).join(" ");
        debug("comparator", comp, options);
        this.options = options;
        this.loose = !!options.loose;
        this.parse(comp);
        if (this.semver === ANY) {
          this.value = "";
        } else {
          this.value = this.operator + this.semver.version;
        }
        debug("comp", this);
      }
      parse(comp) {
        const r = this.options.loose ? re[t.COMPARATORLOOSE] : re[t.COMPARATOR];
        const m = comp.match(r);
        if (!m) {
          throw new TypeError(`Invalid comparator: ${comp}`);
        }
        this.operator = m[1] !== void 0 ? m[1] : "";
        if (this.operator === "=") {
          this.operator = "";
        }
        if (!m[2]) {
          this.semver = ANY;
        } else {
          this.semver = new SemVer(m[2], this.options.loose);
        }
      }
      toString() {
        return this.value;
      }
      test(version) {
        debug("Comparator.test", version, this.options.loose);
        if (this.semver === ANY || version === ANY) {
          return true;
        }
        if (typeof version === "string") {
          try {
            version = new SemVer(version, this.options);
          } catch (er) {
            return false;
          }
        }
        return cmp(version, this.operator, this.semver, this.options);
      }
      intersects(comp, options) {
        if (!(comp instanceof _Comparator)) {
          throw new TypeError("a Comparator is required");
        }
        if (this.operator === "") {
          if (this.value === "") {
            return true;
          }
          return new Range(comp.value, options).test(this.value);
        } else if (comp.operator === "") {
          if (comp.value === "") {
            return true;
          }
          return new Range(this.value, options).test(comp.semver);
        }
        options = parseOptions(options);
        if (options.includePrerelease && (this.value === "<0.0.0-0" || comp.value === "<0.0.0-0")) {
          return false;
        }
        if (!options.includePrerelease && (this.value.startsWith("<0.0.0") || comp.value.startsWith("<0.0.0"))) {
          return false;
        }
        if (this.operator.startsWith(">") && comp.operator.startsWith(">")) {
          return true;
        }
        if (this.operator.startsWith("<") && comp.operator.startsWith("<")) {
          return true;
        }
        if (this.semver.version === comp.semver.version && this.operator.includes("=") && comp.operator.includes("=")) {
          return true;
        }
        if (cmp(this.semver, "<", comp.semver, options) && this.operator.startsWith(">") && comp.operator.startsWith("<")) {
          return true;
        }
        if (cmp(this.semver, ">", comp.semver, options) && this.operator.startsWith("<") && comp.operator.startsWith(">")) {
          return true;
        }
        return false;
      }
    };
    module2.exports = Comparator;
    var parseOptions = require_parse_options();
    var { safeRe: re, t } = require_re();
    var cmp = require_cmp();
    var debug = require_debug();
    var SemVer = require_semver();
    var Range = require_range();
  }
});

// node_modules/semver/functions/satisfies.js
var require_satisfies = __commonJS({
  "node_modules/semver/functions/satisfies.js"(exports2, module2) {
    var Range = require_range();
    var satisfies = (version, range, options) => {
      try {
        range = new Range(range, options);
      } catch (er) {
        return false;
      }
      return range.test(version);
    };
    module2.exports = satisfies;
  }
});

// node_modules/semver/ranges/to-comparators.js
var require_to_comparators = __commonJS({
  "node_modules/semver/ranges/to-comparators.js"(exports2, module2) {
    var Range = require_range();
    var toComparators = (range, options) => new Range(range, options).set.map((comp) => comp.map((c) => c.value).join(" ").trim().split(" "));
    module2.exports = toComparators;
  }
});

// node_modules/semver/ranges/max-satisfying.js
var require_max_satisfying = __commonJS({
  "node_modules/semver/ranges/max-satisfying.js"(exports2, module2) {
    var SemVer = require_semver();
    var Range = require_range();
    var maxSatisfying = (versions, range, options) => {
      let max = null;
      let maxSV = null;
      let rangeObj = null;
      try {
        rangeObj = new Range(range, options);
      } catch (er) {
        return null;
      }
      versions.forEach((v) => {
        if (rangeObj.test(v)) {
          if (!max || maxSV.compare(v) === -1) {
            max = v;
            maxSV = new SemVer(max, options);
          }
        }
      });
      return max;
    };
    module2.exports = maxSatisfying;
  }
});

// node_modules/semver/ranges/min-satisfying.js
var require_min_satisfying = __commonJS({
  "node_modules/semver/ranges/min-satisfying.js"(exports2, module2) {
    var SemVer = require_semver();
    var Range = require_range();
    var minSatisfying = (versions, range, options) => {
      let min = null;
      let minSV = null;
      let rangeObj = null;
      try {
        rangeObj = new Range(range, options);
      } catch (er) {
        return null;
      }
      versions.forEach((v) => {
        if (rangeObj.test(v)) {
          if (!min || minSV.compare(v) === 1) {
            min = v;
            minSV = new SemVer(min, options);
          }
        }
      });
      return min;
    };
    module2.exports = minSatisfying;
  }
});

// node_modules/semver/ranges/min-version.js
var require_min_version = __commonJS({
  "node_modules/semver/ranges/min-version.js"(exports2, module2) {
    var SemVer = require_semver();
    var Range = require_range();
    var gt = require_gt();
    var minVersion = (range, loose) => {
      range = new Range(range, loose);
      let minver = new SemVer("0.0.0");
      if (range.test(minver)) {
        return minver;
      }
      minver = new SemVer("0.0.0-0");
      if (range.test(minver)) {
        return minver;
      }
      minver = null;
      for (let i = 0; i < range.set.length; ++i) {
        const comparators = range.set[i];
        let setMin = null;
        comparators.forEach((comparator) => {
          const compver = new SemVer(comparator.semver.version);
          switch (comparator.operator) {
            case ">":
              if (compver.prerelease.length === 0) {
                compver.patch++;
              } else {
                compver.prerelease.push(0);
              }
              compver.raw = compver.format();
            case "":
            case ">=":
              if (!setMin || gt(compver, setMin)) {
                setMin = compver;
              }
              break;
            case "<":
            case "<=":
              break;
            default:
              throw new Error(`Unexpected operation: ${comparator.operator}`);
          }
        });
        if (setMin && (!minver || gt(minver, setMin))) {
          minver = setMin;
        }
      }
      if (minver && range.test(minver)) {
        return minver;
      }
      return null;
    };
    module2.exports = minVersion;
  }
});

// node_modules/semver/ranges/valid.js
var require_valid2 = __commonJS({
  "node_modules/semver/ranges/valid.js"(exports2, module2) {
    var Range = require_range();
    var validRange = (range, options) => {
      try {
        return new Range(range, options).range || "*";
      } catch (er) {
        return null;
      }
    };
    module2.exports = validRange;
  }
});

// node_modules/semver/ranges/outside.js
var require_outside = __commonJS({
  "node_modules/semver/ranges/outside.js"(exports2, module2) {
    var SemVer = require_semver();
    var Comparator = require_comparator();
    var { ANY } = Comparator;
    var Range = require_range();
    var satisfies = require_satisfies();
    var gt = require_gt();
    var lt = require_lt();
    var lte = require_lte();
    var gte = require_gte();
    var outside = (version, range, hilo, options) => {
      version = new SemVer(version, options);
      range = new Range(range, options);
      let gtfn, ltefn, ltfn, comp, ecomp;
      switch (hilo) {
        case ">":
          gtfn = gt;
          ltefn = lte;
          ltfn = lt;
          comp = ">";
          ecomp = ">=";
          break;
        case "<":
          gtfn = lt;
          ltefn = gte;
          ltfn = gt;
          comp = "<";
          ecomp = "<=";
          break;
        default:
          throw new TypeError('Must provide a hilo val of "<" or ">"');
      }
      if (satisfies(version, range, options)) {
        return false;
      }
      for (let i = 0; i < range.set.length; ++i) {
        const comparators = range.set[i];
        let high = null;
        let low = null;
        comparators.forEach((comparator) => {
          if (comparator.semver === ANY) {
            comparator = new Comparator(">=0.0.0");
          }
          high = high || comparator;
          low = low || comparator;
          if (gtfn(comparator.semver, high.semver, options)) {
            high = comparator;
          } else if (ltfn(comparator.semver, low.semver, options)) {
            low = comparator;
          }
        });
        if (high.operator === comp || high.operator === ecomp) {
          return false;
        }
        if ((!low.operator || low.operator === comp) && ltefn(version, low.semver)) {
          return false;
        } else if (low.operator === ecomp && ltfn(version, low.semver)) {
          return false;
        }
      }
      return true;
    };
    module2.exports = outside;
  }
});

// node_modules/semver/ranges/gtr.js
var require_gtr = __commonJS({
  "node_modules/semver/ranges/gtr.js"(exports2, module2) {
    var outside = require_outside();
    var gtr = (version, range, options) => outside(version, range, ">", options);
    module2.exports = gtr;
  }
});

// node_modules/semver/ranges/ltr.js
var require_ltr = __commonJS({
  "node_modules/semver/ranges/ltr.js"(exports2, module2) {
    var outside = require_outside();
    var ltr = (version, range, options) => outside(version, range, "<", options);
    module2.exports = ltr;
  }
});

// node_modules/semver/ranges/intersects.js
var require_intersects = __commonJS({
  "node_modules/semver/ranges/intersects.js"(exports2, module2) {
    var Range = require_range();
    var intersects = (r1, r2, options) => {
      r1 = new Range(r1, options);
      r2 = new Range(r2, options);
      return r1.intersects(r2, options);
    };
    module2.exports = intersects;
  }
});

// node_modules/semver/ranges/simplify.js
var require_simplify = __commonJS({
  "node_modules/semver/ranges/simplify.js"(exports2, module2) {
    var satisfies = require_satisfies();
    var compare = require_compare();
    module2.exports = (versions, range, options) => {
      const set = [];
      let first = null;
      let prev = null;
      const v = versions.sort((a, b) => compare(a, b, options));
      for (const version of v) {
        const included = satisfies(version, range, options);
        if (included) {
          prev = version;
          if (!first) {
            first = version;
          }
        } else {
          if (prev) {
            set.push([first, prev]);
          }
          prev = null;
          first = null;
        }
      }
      if (first) {
        set.push([first, null]);
      }
      const ranges = [];
      for (const [min, max] of set) {
        if (min === max) {
          ranges.push(min);
        } else if (!max && min === v[0]) {
          ranges.push("*");
        } else if (!max) {
          ranges.push(`>=${min}`);
        } else if (min === v[0]) {
          ranges.push(`<=${max}`);
        } else {
          ranges.push(`${min} - ${max}`);
        }
      }
      const simplified = ranges.join(" || ");
      const original = typeof range.raw === "string" ? range.raw : String(range);
      return simplified.length < original.length ? simplified : range;
    };
  }
});

// node_modules/semver/ranges/subset.js
var require_subset = __commonJS({
  "node_modules/semver/ranges/subset.js"(exports2, module2) {
    var Range = require_range();
    var Comparator = require_comparator();
    var { ANY } = Comparator;
    var satisfies = require_satisfies();
    var compare = require_compare();
    var subset = (sub, dom, options = {}) => {
      if (sub === dom) {
        return true;
      }
      sub = new Range(sub, options);
      dom = new Range(dom, options);
      let sawNonNull = false;
      OUTER:
        for (const simpleSub of sub.set) {
          for (const simpleDom of dom.set) {
            const isSub = simpleSubset(simpleSub, simpleDom, options);
            sawNonNull = sawNonNull || isSub !== null;
            if (isSub) {
              continue OUTER;
            }
          }
          if (sawNonNull) {
            return false;
          }
        }
      return true;
    };
    var minimumVersionWithPreRelease = [new Comparator(">=0.0.0-0")];
    var minimumVersion = [new Comparator(">=0.0.0")];
    var simpleSubset = (sub, dom, options) => {
      if (sub === dom) {
        return true;
      }
      if (sub.length === 1 && sub[0].semver === ANY) {
        if (dom.length === 1 && dom[0].semver === ANY) {
          return true;
        } else if (options.includePrerelease) {
          sub = minimumVersionWithPreRelease;
        } else {
          sub = minimumVersion;
        }
      }
      if (dom.length === 1 && dom[0].semver === ANY) {
        if (options.includePrerelease) {
          return true;
        } else {
          dom = minimumVersion;
        }
      }
      const eqSet = /* @__PURE__ */ new Set();
      let gt, lt;
      for (const c of sub) {
        if (c.operator === ">" || c.operator === ">=") {
          gt = higherGT(gt, c, options);
        } else if (c.operator === "<" || c.operator === "<=") {
          lt = lowerLT(lt, c, options);
        } else {
          eqSet.add(c.semver);
        }
      }
      if (eqSet.size > 1) {
        return null;
      }
      let gtltComp;
      if (gt && lt) {
        gtltComp = compare(gt.semver, lt.semver, options);
        if (gtltComp > 0) {
          return null;
        } else if (gtltComp === 0 && (gt.operator !== ">=" || lt.operator !== "<=")) {
          return null;
        }
      }
      for (const eq of eqSet) {
        if (gt && !satisfies(eq, String(gt), options)) {
          return null;
        }
        if (lt && !satisfies(eq, String(lt), options)) {
          return null;
        }
        for (const c of dom) {
          if (!satisfies(eq, String(c), options)) {
            return false;
          }
        }
        return true;
      }
      let higher, lower;
      let hasDomLT, hasDomGT;
      let needDomLTPre = lt && !options.includePrerelease && lt.semver.prerelease.length ? lt.semver : false;
      let needDomGTPre = gt && !options.includePrerelease && gt.semver.prerelease.length ? gt.semver : false;
      if (needDomLTPre && needDomLTPre.prerelease.length === 1 && lt.operator === "<" && needDomLTPre.prerelease[0] === 0) {
        needDomLTPre = false;
      }
      for (const c of dom) {
        hasDomGT = hasDomGT || c.operator === ">" || c.operator === ">=";
        hasDomLT = hasDomLT || c.operator === "<" || c.operator === "<=";
        if (gt) {
          if (needDomGTPre) {
            if (c.semver.prerelease && c.semver.prerelease.length && c.semver.major === needDomGTPre.major && c.semver.minor === needDomGTPre.minor && c.semver.patch === needDomGTPre.patch) {
              needDomGTPre = false;
            }
          }
          if (c.operator === ">" || c.operator === ">=") {
            higher = higherGT(gt, c, options);
            if (higher === c && higher !== gt) {
              return false;
            }
          } else if (gt.operator === ">=" && !satisfies(gt.semver, String(c), options)) {
            return false;
          }
        }
        if (lt) {
          if (needDomLTPre) {
            if (c.semver.prerelease && c.semver.prerelease.length && c.semver.major === needDomLTPre.major && c.semver.minor === needDomLTPre.minor && c.semver.patch === needDomLTPre.patch) {
              needDomLTPre = false;
            }
          }
          if (c.operator === "<" || c.operator === "<=") {
            lower = lowerLT(lt, c, options);
            if (lower === c && lower !== lt) {
              return false;
            }
          } else if (lt.operator === "<=" && !satisfies(lt.semver, String(c), options)) {
            return false;
          }
        }
        if (!c.operator && (lt || gt) && gtltComp !== 0) {
          return false;
        }
      }
      if (gt && hasDomLT && !lt && gtltComp !== 0) {
        return false;
      }
      if (lt && hasDomGT && !gt && gtltComp !== 0) {
        return false;
      }
      if (needDomGTPre || needDomLTPre) {
        return false;
      }
      return true;
    };
    var higherGT = (a, b, options) => {
      if (!a) {
        return b;
      }
      const comp = compare(a.semver, b.semver, options);
      return comp > 0 ? a : comp < 0 ? b : b.operator === ">" && a.operator === ">=" ? b : a;
    };
    var lowerLT = (a, b, options) => {
      if (!a) {
        return b;
      }
      const comp = compare(a.semver, b.semver, options);
      return comp < 0 ? a : comp > 0 ? b : b.operator === "<" && a.operator === "<=" ? b : a;
    };
    module2.exports = subset;
  }
});

// node_modules/semver/index.js
var require_semver2 = __commonJS({
  "node_modules/semver/index.js"(exports2, module2) {
    var internalRe = require_re();
    var constants3 = require_constants2();
    var SemVer = require_semver();
    var identifiers = require_identifiers();
    var parse2 = require_parse2();
    var valid = require_valid();
    var clean = require_clean();
    var inc = require_inc();
    var diff = require_diff();
    var major = require_major();
    var minor = require_minor();
    var patch = require_patch();
    var prerelease = require_prerelease();
    var compare = require_compare();
    var rcompare = require_rcompare();
    var compareLoose = require_compare_loose();
    var compareBuild = require_compare_build();
    var sort = require_sort();
    var rsort = require_rsort();
    var gt = require_gt();
    var lt = require_lt();
    var eq = require_eq();
    var neq = require_neq();
    var gte = require_gte();
    var lte = require_lte();
    var cmp = require_cmp();
    var coerce = require_coerce();
    var Comparator = require_comparator();
    var Range = require_range();
    var satisfies = require_satisfies();
    var toComparators = require_to_comparators();
    var maxSatisfying = require_max_satisfying();
    var minSatisfying = require_min_satisfying();
    var minVersion = require_min_version();
    var validRange = require_valid2();
    var outside = require_outside();
    var gtr = require_gtr();
    var ltr = require_ltr();
    var intersects = require_intersects();
    var simplifyRange = require_simplify();
    var subset = require_subset();
    module2.exports = {
      parse: parse2,
      valid,
      clean,
      inc,
      diff,
      major,
      minor,
      patch,
      prerelease,
      compare,
      rcompare,
      compareLoose,
      compareBuild,
      sort,
      rsort,
      gt,
      lt,
      eq,
      neq,
      gte,
      lte,
      cmp,
      coerce,
      Comparator,
      Range,
      satisfies,
      toComparators,
      maxSatisfying,
      minSatisfying,
      minVersion,
      validRange,
      outside,
      gtr,
      ltr,
      intersects,
      simplifyRange,
      subset,
      SemVer,
      re: internalRe.re,
      src: internalRe.src,
      tokens: internalRe.t,
      SEMVER_SPEC_VERSION: constants3.SEMVER_SPEC_VERSION,
      RELEASE_TYPES: constants3.RELEASE_TYPES,
      compareIdentifiers: identifiers.compareIdentifiers,
      rcompareIdentifiers: identifiers.rcompareIdentifiers
    };
  }
});

// node_modules/parse-package-name/dist/index.mjs
function parse(input) {
  const m = RE_SCOPED.exec(input) || RE_NON_SCOPED.exec(input);
  if (!m) {
    throw new Error(`[parse-package-name] invalid package name: ${input}`);
  }
  return {
    name: m[1] || "",
    version: m[2] || "latest",
    path: m[3] || ""
  };
}
var RE_SCOPED, RE_NON_SCOPED;
var init_dist = __esm({
  "node_modules/parse-package-name/dist/index.mjs"() {
    RE_SCOPED = /^(@[^\/]+\/[^@\/]+)(?:@([^\/]+))?(\/.*)?$/;
    RE_NON_SCOPED = /^([^@\/]+)(?:@([^\/]+))?(\/.*)?$/;
  }
});

// node_modules/isexe/windows.js
var require_windows = __commonJS({
  "node_modules/isexe/windows.js"(exports2, module2) {
    module2.exports = isexe;
    isexe.sync = sync;
    var fs = require("fs");
    function checkPathExt(path4, options) {
      var pathext = options.pathExt !== void 0 ? options.pathExt : process.env.PATHEXT;
      if (!pathext) {
        return true;
      }
      pathext = pathext.split(";");
      if (pathext.indexOf("") !== -1) {
        return true;
      }
      for (var i = 0; i < pathext.length; i++) {
        var p = pathext[i].toLowerCase();
        if (p && path4.substr(-p.length).toLowerCase() === p) {
          return true;
        }
      }
      return false;
    }
    function checkStat(stat, path4, options) {
      if (!stat.isSymbolicLink() && !stat.isFile()) {
        return false;
      }
      return checkPathExt(path4, options);
    }
    function isexe(path4, options, cb) {
      fs.stat(path4, function(er, stat) {
        cb(er, er ? false : checkStat(stat, path4, options));
      });
    }
    function sync(path4, options) {
      return checkStat(fs.statSync(path4), path4, options);
    }
  }
});

// node_modules/isexe/mode.js
var require_mode = __commonJS({
  "node_modules/isexe/mode.js"(exports2, module2) {
    module2.exports = isexe;
    isexe.sync = sync;
    var fs = require("fs");
    function isexe(path4, options, cb) {
      fs.stat(path4, function(er, stat) {
        cb(er, er ? false : checkStat(stat, options));
      });
    }
    function sync(path4, options) {
      return checkStat(fs.statSync(path4), options);
    }
    function checkStat(stat, options) {
      return stat.isFile() && checkMode(stat, options);
    }
    function checkMode(stat, options) {
      var mod = stat.mode;
      var uid = stat.uid;
      var gid = stat.gid;
      var myUid = options.uid !== void 0 ? options.uid : process.getuid && process.getuid();
      var myGid = options.gid !== void 0 ? options.gid : process.getgid && process.getgid();
      var u = parseInt("100", 8);
      var g = parseInt("010", 8);
      var o = parseInt("001", 8);
      var ug = u | g;
      var ret = mod & o || mod & g && gid === myGid || mod & u && uid === myUid || mod & ug && myUid === 0;
      return ret;
    }
  }
});

// node_modules/isexe/index.js
var require_isexe = __commonJS({
  "node_modules/isexe/index.js"(exports2, module2) {
    var fs = require("fs");
    var core;
    if (process.platform === "win32" || global.TESTING_WINDOWS) {
      core = require_windows();
    } else {
      core = require_mode();
    }
    module2.exports = isexe;
    isexe.sync = sync;
    function isexe(path4, options, cb) {
      if (typeof options === "function") {
        cb = options;
        options = {};
      }
      if (!cb) {
        if (typeof Promise !== "function") {
          throw new TypeError("callback not provided");
        }
        return new Promise(function(resolve, reject) {
          isexe(path4, options || {}, function(er, is) {
            if (er) {
              reject(er);
            } else {
              resolve(is);
            }
          });
        });
      }
      core(path4, options || {}, function(er, is) {
        if (er) {
          if (er.code === "EACCES" || options && options.ignoreErrors) {
            er = null;
            is = false;
          }
        }
        cb(er, is);
      });
    }
    function sync(path4, options) {
      try {
        return core.sync(path4, options || {});
      } catch (er) {
        if (options && options.ignoreErrors || er.code === "EACCES") {
          return false;
        } else {
          throw er;
        }
      }
    }
  }
});

// node_modules/which/which.js
var require_which = __commonJS({
  "node_modules/which/which.js"(exports2, module2) {
    var isWindows = process.platform === "win32" || process.env.OSTYPE === "cygwin" || process.env.OSTYPE === "msys";
    var path4 = require("path");
    var COLON = isWindows ? ";" : ":";
    var isexe = require_isexe();
    var getNotFoundError = (cmd) => Object.assign(new Error(`not found: ${cmd}`), { code: "ENOENT" });
    var getPathInfo = (cmd, opt) => {
      const colon = opt.colon || COLON;
      const pathEnv = cmd.match(/\//) || isWindows && cmd.match(/\\/) ? [""] : [
        // windows always checks the cwd first
        ...isWindows ? [process.cwd()] : [],
        ...(opt.path || process.env.PATH || /* istanbul ignore next: very unusual */
        "").split(colon)
      ];
      const pathExtExe = isWindows ? opt.pathExt || process.env.PATHEXT || ".EXE;.CMD;.BAT;.COM" : "";
      const pathExt = isWindows ? pathExtExe.split(colon) : [""];
      if (isWindows) {
        if (cmd.indexOf(".") !== -1 && pathExt[0] !== "")
          pathExt.unshift("");
      }
      return {
        pathEnv,
        pathExt,
        pathExtExe
      };
    };
    var which = (cmd, opt, cb) => {
      if (typeof opt === "function") {
        cb = opt;
        opt = {};
      }
      if (!opt)
        opt = {};
      const { pathEnv, pathExt, pathExtExe } = getPathInfo(cmd, opt);
      const found = [];
      const step = (i) => new Promise((resolve, reject) => {
        if (i === pathEnv.length)
          return opt.all && found.length ? resolve(found) : reject(getNotFoundError(cmd));
        const ppRaw = pathEnv[i];
        const pathPart = /^".*"$/.test(ppRaw) ? ppRaw.slice(1, -1) : ppRaw;
        const pCmd = path4.join(pathPart, cmd);
        const p = !pathPart && /^\.[\\\/]/.test(cmd) ? cmd.slice(0, 2) + pCmd : pCmd;
        resolve(subStep(p, i, 0));
      });
      const subStep = (p, i, ii) => new Promise((resolve, reject) => {
        if (ii === pathExt.length)
          return resolve(step(i + 1));
        const ext = pathExt[ii];
        isexe(p + ext, { pathExt: pathExtExe }, (er, is) => {
          if (!er && is) {
            if (opt.all)
              found.push(p + ext);
            else
              return resolve(p + ext);
          }
          return resolve(subStep(p, i, ii + 1));
        });
      });
      return cb ? step(0).then((res) => cb(null, res), cb) : step(0);
    };
    var whichSync = (cmd, opt) => {
      opt = opt || {};
      const { pathEnv, pathExt, pathExtExe } = getPathInfo(cmd, opt);
      const found = [];
      for (let i = 0; i < pathEnv.length; i++) {
        const ppRaw = pathEnv[i];
        const pathPart = /^".*"$/.test(ppRaw) ? ppRaw.slice(1, -1) : ppRaw;
        const pCmd = path4.join(pathPart, cmd);
        const p = !pathPart && /^\.[\\\/]/.test(cmd) ? cmd.slice(0, 2) + pCmd : pCmd;
        for (let j = 0; j < pathExt.length; j++) {
          const cur = p + pathExt[j];
          try {
            const is = isexe.sync(cur, { pathExt: pathExtExe });
            if (is) {
              if (opt.all)
                found.push(cur);
              else
                return cur;
            }
          } catch (ex) {
          }
        }
      }
      if (opt.all && found.length)
        return found;
      if (opt.nothrow)
        return null;
      throw getNotFoundError(cmd);
    };
    module2.exports = which;
    which.sync = whichSync;
  }
});

// node_modules/path-key/index.js
var require_path_key = __commonJS({
  "node_modules/path-key/index.js"(exports2, module2) {
    "use strict";
    var pathKey2 = (options = {}) => {
      const environment = options.env || process.env;
      const platform = options.platform || process.platform;
      if (platform !== "win32") {
        return "PATH";
      }
      return Object.keys(environment).reverse().find((key) => key.toUpperCase() === "PATH") || "Path";
    };
    module2.exports = pathKey2;
    module2.exports.default = pathKey2;
  }
});

// node_modules/cross-spawn/lib/util/resolveCommand.js
var require_resolveCommand = __commonJS({
  "node_modules/cross-spawn/lib/util/resolveCommand.js"(exports2, module2) {
    "use strict";
    var path4 = require("path");
    var which = require_which();
    var getPathKey = require_path_key();
    function resolveCommandAttempt(parsed, withoutPathExt) {
      const env = parsed.options.env || process.env;
      const cwd = process.cwd();
      const hasCustomCwd = parsed.options.cwd != null;
      const shouldSwitchCwd = hasCustomCwd && process.chdir !== void 0 && !process.chdir.disabled;
      if (shouldSwitchCwd) {
        try {
          process.chdir(parsed.options.cwd);
        } catch (err) {
        }
      }
      let resolved;
      try {
        resolved = which.sync(parsed.command, {
          path: env[getPathKey({ env })],
          pathExt: withoutPathExt ? path4.delimiter : void 0
        });
      } catch (e) {
      } finally {
        if (shouldSwitchCwd) {
          process.chdir(cwd);
        }
      }
      if (resolved) {
        resolved = path4.resolve(hasCustomCwd ? parsed.options.cwd : "", resolved);
      }
      return resolved;
    }
    function resolveCommand(parsed) {
      return resolveCommandAttempt(parsed) || resolveCommandAttempt(parsed, true);
    }
    module2.exports = resolveCommand;
  }
});

// node_modules/cross-spawn/lib/util/escape.js
var require_escape = __commonJS({
  "node_modules/cross-spawn/lib/util/escape.js"(exports2, module2) {
    "use strict";
    var metaCharsRegExp = /([()\][%!^"`<>&|;, *?])/g;
    function escapeCommand(arg) {
      arg = arg.replace(metaCharsRegExp, "^$1");
      return arg;
    }
    function escapeArgument(arg, doubleEscapeMetaChars) {
      arg = `${arg}`;
      arg = arg.replace(/(\\*)"/g, '$1$1\\"');
      arg = arg.replace(/(\\*)$/, "$1$1");
      arg = `"${arg}"`;
      arg = arg.replace(metaCharsRegExp, "^$1");
      if (doubleEscapeMetaChars) {
        arg = arg.replace(metaCharsRegExp, "^$1");
      }
      return arg;
    }
    module2.exports.command = escapeCommand;
    module2.exports.argument = escapeArgument;
  }
});

// node_modules/shebang-regex/index.js
var require_shebang_regex = __commonJS({
  "node_modules/shebang-regex/index.js"(exports2, module2) {
    "use strict";
    module2.exports = /^#!(.*)/;
  }
});

// node_modules/shebang-command/index.js
var require_shebang_command = __commonJS({
  "node_modules/shebang-command/index.js"(exports2, module2) {
    "use strict";
    var shebangRegex = require_shebang_regex();
    module2.exports = (string = "") => {
      const match = string.match(shebangRegex);
      if (!match) {
        return null;
      }
      const [path4, argument] = match[0].replace(/#! ?/, "").split(" ");
      const binary = path4.split("/").pop();
      if (binary === "env") {
        return argument;
      }
      return argument ? `${binary} ${argument}` : binary;
    };
  }
});

// node_modules/cross-spawn/lib/util/readShebang.js
var require_readShebang = __commonJS({
  "node_modules/cross-spawn/lib/util/readShebang.js"(exports2, module2) {
    "use strict";
    var fs = require("fs");
    var shebangCommand = require_shebang_command();
    function readShebang(command) {
      const size = 150;
      const buffer = Buffer.alloc(size);
      let fd;
      try {
        fd = fs.openSync(command, "r");
        fs.readSync(fd, buffer, 0, size, 0);
        fs.closeSync(fd);
      } catch (e) {
      }
      return shebangCommand(buffer.toString());
    }
    module2.exports = readShebang;
  }
});

// node_modules/cross-spawn/lib/parse.js
var require_parse3 = __commonJS({
  "node_modules/cross-spawn/lib/parse.js"(exports2, module2) {
    "use strict";
    var path4 = require("path");
    var resolveCommand = require_resolveCommand();
    var escape = require_escape();
    var readShebang = require_readShebang();
    var isWin = process.platform === "win32";
    var isExecutableRegExp = /\.(?:com|exe)$/i;
    var isCmdShimRegExp = /node_modules[\\/].bin[\\/][^\\/]+\.cmd$/i;
    function detectShebang(parsed) {
      parsed.file = resolveCommand(parsed);
      const shebang = parsed.file && readShebang(parsed.file);
      if (shebang) {
        parsed.args.unshift(parsed.file);
        parsed.command = shebang;
        return resolveCommand(parsed);
      }
      return parsed.file;
    }
    function parseNonShell(parsed) {
      if (!isWin) {
        return parsed;
      }
      const commandFile = detectShebang(parsed);
      const needsShell = !isExecutableRegExp.test(commandFile);
      if (parsed.options.forceShell || needsShell) {
        const needsDoubleEscapeMetaChars = isCmdShimRegExp.test(commandFile);
        parsed.command = path4.normalize(parsed.command);
        parsed.command = escape.command(parsed.command);
        parsed.args = parsed.args.map((arg) => escape.argument(arg, needsDoubleEscapeMetaChars));
        const shellCommand = [parsed.command].concat(parsed.args).join(" ");
        parsed.args = ["/d", "/s", "/c", `"${shellCommand}"`];
        parsed.command = process.env.comspec || "cmd.exe";
        parsed.options.windowsVerbatimArguments = true;
      }
      return parsed;
    }
    function parse2(command, args, options) {
      if (args && !Array.isArray(args)) {
        options = args;
        args = null;
      }
      args = args ? args.slice(0) : [];
      options = Object.assign({}, options);
      const parsed = {
        command,
        args,
        options,
        file: void 0,
        original: {
          command,
          args
        }
      };
      return options.shell ? parsed : parseNonShell(parsed);
    }
    module2.exports = parse2;
  }
});

// node_modules/cross-spawn/lib/enoent.js
var require_enoent = __commonJS({
  "node_modules/cross-spawn/lib/enoent.js"(exports2, module2) {
    "use strict";
    var isWin = process.platform === "win32";
    function notFoundError(original, syscall) {
      return Object.assign(new Error(`${syscall} ${original.command} ENOENT`), {
        code: "ENOENT",
        errno: "ENOENT",
        syscall: `${syscall} ${original.command}`,
        path: original.command,
        spawnargs: original.args
      });
    }
    function hookChildProcess(cp, parsed) {
      if (!isWin) {
        return;
      }
      const originalEmit = cp.emit;
      cp.emit = function(name, arg1) {
        if (name === "exit") {
          const err = verifyENOENT(arg1, parsed, "spawn");
          if (err) {
            return originalEmit.call(cp, "error", err);
          }
        }
        return originalEmit.apply(cp, arguments);
      };
    }
    function verifyENOENT(status, parsed) {
      if (isWin && status === 1 && !parsed.file) {
        return notFoundError(parsed.original, "spawn");
      }
      return null;
    }
    function verifyENOENTSync(status, parsed) {
      if (isWin && status === 1 && !parsed.file) {
        return notFoundError(parsed.original, "spawnSync");
      }
      return null;
    }
    module2.exports = {
      hookChildProcess,
      verifyENOENT,
      verifyENOENTSync,
      notFoundError
    };
  }
});

// node_modules/cross-spawn/index.js
var require_cross_spawn = __commonJS({
  "node_modules/cross-spawn/index.js"(exports2, module2) {
    "use strict";
    var cp = require("child_process");
    var parse2 = require_parse3();
    var enoent = require_enoent();
    function spawn(command, args, options) {
      const parsed = parse2(command, args, options);
      const spawned = cp.spawn(parsed.command, parsed.args, parsed.options);
      enoent.hookChildProcess(spawned, parsed);
      return spawned;
    }
    function spawnSync(command, args, options) {
      const parsed = parse2(command, args, options);
      const result = cp.spawnSync(parsed.command, parsed.args, parsed.options);
      result.error = result.error || enoent.verifyENOENTSync(result.status, parsed);
      return result;
    }
    module2.exports = spawn;
    module2.exports.spawn = spawn;
    module2.exports.sync = spawnSync;
    module2.exports._parse = parse2;
    module2.exports._enoent = enoent;
  }
});

// node_modules/strip-final-newline/index.js
function stripFinalNewline(input) {
  const LF = typeof input === "string" ? "\n" : "\n".charCodeAt();
  const CR = typeof input === "string" ? "\r" : "\r".charCodeAt();
  if (input[input.length - 1] === LF) {
    input = input.slice(0, -1);
  }
  if (input[input.length - 1] === CR) {
    input = input.slice(0, -1);
  }
  return input;
}
var init_strip_final_newline = __esm({
  "node_modules/strip-final-newline/index.js"() {
  }
});

// node_modules/npm-run-path/node_modules/path-key/index.js
function pathKey(options = {}) {
  const {
    env = process.env,
    platform = process.platform
  } = options;
  if (platform !== "win32") {
    return "PATH";
  }
  return Object.keys(env).reverse().find((key) => key.toUpperCase() === "PATH") || "Path";
}
var init_path_key = __esm({
  "node_modules/npm-run-path/node_modules/path-key/index.js"() {
  }
});

// node_modules/npm-run-path/index.js
function npmRunPath(options = {}) {
  const {
    cwd = import_node_process.default.cwd(),
    path: path_ = import_node_process.default.env[pathKey()],
    execPath = import_node_process.default.execPath
  } = options;
  let previous;
  const execPathString = execPath instanceof URL ? import_node_url.default.fileURLToPath(execPath) : execPath;
  const cwdString = cwd instanceof URL ? import_node_url.default.fileURLToPath(cwd) : cwd;
  let cwdPath = import_node_path.default.resolve(cwdString);
  const result = [];
  while (previous !== cwdPath) {
    result.push(import_node_path.default.join(cwdPath, "node_modules/.bin"));
    previous = cwdPath;
    cwdPath = import_node_path.default.resolve(cwdPath, "..");
  }
  result.push(import_node_path.default.resolve(cwdString, execPathString, ".."));
  return [...result, path_].join(import_node_path.default.delimiter);
}
function npmRunPathEnv({ env = import_node_process.default.env, ...options } = {}) {
  env = { ...env };
  const path4 = pathKey({ env });
  options.path = env[path4];
  env[path4] = npmRunPath(options);
  return env;
}
var import_node_process, import_node_path, import_node_url;
var init_npm_run_path = __esm({
  "node_modules/npm-run-path/index.js"() {
    import_node_process = __toESM(require("node:process"), 1);
    import_node_path = __toESM(require("node:path"), 1);
    import_node_url = __toESM(require("node:url"), 1);
    init_path_key();
  }
});

// node_modules/mimic-fn/index.js
function mimicFunction(to, from, { ignoreNonConfigurable = false } = {}) {
  const { name } = to;
  for (const property of Reflect.ownKeys(from)) {
    copyProperty(to, from, property, ignoreNonConfigurable);
  }
  changePrototype(to, from);
  changeToString(to, from, name);
  return to;
}
var copyProperty, canCopyProperty, changePrototype, wrappedToString, toStringDescriptor, toStringName, changeToString;
var init_mimic_fn = __esm({
  "node_modules/mimic-fn/index.js"() {
    copyProperty = (to, from, property, ignoreNonConfigurable) => {
      if (property === "length" || property === "prototype") {
        return;
      }
      if (property === "arguments" || property === "caller") {
        return;
      }
      const toDescriptor = Object.getOwnPropertyDescriptor(to, property);
      const fromDescriptor = Object.getOwnPropertyDescriptor(from, property);
      if (!canCopyProperty(toDescriptor, fromDescriptor) && ignoreNonConfigurable) {
        return;
      }
      Object.defineProperty(to, property, fromDescriptor);
    };
    canCopyProperty = function(toDescriptor, fromDescriptor) {
      return toDescriptor === void 0 || toDescriptor.configurable || toDescriptor.writable === fromDescriptor.writable && toDescriptor.enumerable === fromDescriptor.enumerable && toDescriptor.configurable === fromDescriptor.configurable && (toDescriptor.writable || toDescriptor.value === fromDescriptor.value);
    };
    changePrototype = (to, from) => {
      const fromPrototype = Object.getPrototypeOf(from);
      if (fromPrototype === Object.getPrototypeOf(to)) {
        return;
      }
      Object.setPrototypeOf(to, fromPrototype);
    };
    wrappedToString = (withName, fromBody) => `/* Wrapped ${withName}*/
${fromBody}`;
    toStringDescriptor = Object.getOwnPropertyDescriptor(Function.prototype, "toString");
    toStringName = Object.getOwnPropertyDescriptor(Function.prototype.toString, "name");
    changeToString = (to, from, name) => {
      const withName = name === "" ? "" : `with ${name.trim()}() `;
      const newToString = wrappedToString.bind(null, withName, from.toString());
      Object.defineProperty(newToString, "name", toStringName);
      Object.defineProperty(to, "toString", { ...toStringDescriptor, value: newToString });
    };
  }
});

// node_modules/onetime/index.js
var calledFunctions, onetime, onetime_default;
var init_onetime = __esm({
  "node_modules/onetime/index.js"() {
    init_mimic_fn();
    calledFunctions = /* @__PURE__ */ new WeakMap();
    onetime = (function_, options = {}) => {
      if (typeof function_ !== "function") {
        throw new TypeError("Expected a function");
      }
      let returnValue;
      let callCount = 0;
      const functionName = function_.displayName || function_.name || "<anonymous>";
      const onetime2 = function(...arguments_) {
        calledFunctions.set(onetime2, ++callCount);
        if (callCount === 1) {
          returnValue = function_.apply(this, arguments_);
          function_ = null;
        } else if (options.throw === true) {
          throw new Error(`Function \`${functionName}\` can only be called once`);
        }
        return returnValue;
      };
      mimicFunction(onetime2, function_);
      calledFunctions.set(onetime2, callCount);
      return onetime2;
    };
    onetime.callCount = (function_) => {
      if (!calledFunctions.has(function_)) {
        throw new Error(`The given function \`${function_.name}\` is not wrapped by the \`onetime\` package`);
      }
      return calledFunctions.get(function_);
    };
    onetime_default = onetime;
  }
});

// node_modules/human-signals/build/src/realtime.js
var getRealtimeSignals, getRealtimeSignal, SIGRTMIN, SIGRTMAX;
var init_realtime = __esm({
  "node_modules/human-signals/build/src/realtime.js"() {
    getRealtimeSignals = function() {
      const length = SIGRTMAX - SIGRTMIN + 1;
      return Array.from({ length }, getRealtimeSignal);
    };
    getRealtimeSignal = function(value, index) {
      return {
        name: `SIGRT${index + 1}`,
        number: SIGRTMIN + index,
        action: "terminate",
        description: "Application-specific signal (realtime)",
        standard: "posix"
      };
    };
    SIGRTMIN = 34;
    SIGRTMAX = 64;
  }
});

// node_modules/human-signals/build/src/core.js
var SIGNALS;
var init_core = __esm({
  "node_modules/human-signals/build/src/core.js"() {
    SIGNALS = [
      {
        name: "SIGHUP",
        number: 1,
        action: "terminate",
        description: "Terminal closed",
        standard: "posix"
      },
      {
        name: "SIGINT",
        number: 2,
        action: "terminate",
        description: "User interruption with CTRL-C",
        standard: "ansi"
      },
      {
        name: "SIGQUIT",
        number: 3,
        action: "core",
        description: "User interruption with CTRL-\\",
        standard: "posix"
      },
      {
        name: "SIGILL",
        number: 4,
        action: "core",
        description: "Invalid machine instruction",
        standard: "ansi"
      },
      {
        name: "SIGTRAP",
        number: 5,
        action: "core",
        description: "Debugger breakpoint",
        standard: "posix"
      },
      {
        name: "SIGABRT",
        number: 6,
        action: "core",
        description: "Aborted",
        standard: "ansi"
      },
      {
        name: "SIGIOT",
        number: 6,
        action: "core",
        description: "Aborted",
        standard: "bsd"
      },
      {
        name: "SIGBUS",
        number: 7,
        action: "core",
        description: "Bus error due to misaligned, non-existing address or paging error",
        standard: "bsd"
      },
      {
        name: "SIGEMT",
        number: 7,
        action: "terminate",
        description: "Command should be emulated but is not implemented",
        standard: "other"
      },
      {
        name: "SIGFPE",
        number: 8,
        action: "core",
        description: "Floating point arithmetic error",
        standard: "ansi"
      },
      {
        name: "SIGKILL",
        number: 9,
        action: "terminate",
        description: "Forced termination",
        standard: "posix",
        forced: true
      },
      {
        name: "SIGUSR1",
        number: 10,
        action: "terminate",
        description: "Application-specific signal",
        standard: "posix"
      },
      {
        name: "SIGSEGV",
        number: 11,
        action: "core",
        description: "Segmentation fault",
        standard: "ansi"
      },
      {
        name: "SIGUSR2",
        number: 12,
        action: "terminate",
        description: "Application-specific signal",
        standard: "posix"
      },
      {
        name: "SIGPIPE",
        number: 13,
        action: "terminate",
        description: "Broken pipe or socket",
        standard: "posix"
      },
      {
        name: "SIGALRM",
        number: 14,
        action: "terminate",
        description: "Timeout or timer",
        standard: "posix"
      },
      {
        name: "SIGTERM",
        number: 15,
        action: "terminate",
        description: "Termination",
        standard: "ansi"
      },
      {
        name: "SIGSTKFLT",
        number: 16,
        action: "terminate",
        description: "Stack is empty or overflowed",
        standard: "other"
      },
      {
        name: "SIGCHLD",
        number: 17,
        action: "ignore",
        description: "Child process terminated, paused or unpaused",
        standard: "posix"
      },
      {
        name: "SIGCLD",
        number: 17,
        action: "ignore",
        description: "Child process terminated, paused or unpaused",
        standard: "other"
      },
      {
        name: "SIGCONT",
        number: 18,
        action: "unpause",
        description: "Unpaused",
        standard: "posix",
        forced: true
      },
      {
        name: "SIGSTOP",
        number: 19,
        action: "pause",
        description: "Paused",
        standard: "posix",
        forced: true
      },
      {
        name: "SIGTSTP",
        number: 20,
        action: "pause",
        description: 'Paused using CTRL-Z or "suspend"',
        standard: "posix"
      },
      {
        name: "SIGTTIN",
        number: 21,
        action: "pause",
        description: "Background process cannot read terminal input",
        standard: "posix"
      },
      {
        name: "SIGBREAK",
        number: 21,
        action: "terminate",
        description: "User interruption with CTRL-BREAK",
        standard: "other"
      },
      {
        name: "SIGTTOU",
        number: 22,
        action: "pause",
        description: "Background process cannot write to terminal output",
        standard: "posix"
      },
      {
        name: "SIGURG",
        number: 23,
        action: "ignore",
        description: "Socket received out-of-band data",
        standard: "bsd"
      },
      {
        name: "SIGXCPU",
        number: 24,
        action: "core",
        description: "Process timed out",
        standard: "bsd"
      },
      {
        name: "SIGXFSZ",
        number: 25,
        action: "core",
        description: "File too big",
        standard: "bsd"
      },
      {
        name: "SIGVTALRM",
        number: 26,
        action: "terminate",
        description: "Timeout or timer",
        standard: "bsd"
      },
      {
        name: "SIGPROF",
        number: 27,
        action: "terminate",
        description: "Timeout or timer",
        standard: "bsd"
      },
      {
        name: "SIGWINCH",
        number: 28,
        action: "ignore",
        description: "Terminal window size changed",
        standard: "bsd"
      },
      {
        name: "SIGIO",
        number: 29,
        action: "terminate",
        description: "I/O is available",
        standard: "other"
      },
      {
        name: "SIGPOLL",
        number: 29,
        action: "terminate",
        description: "Watched event",
        standard: "other"
      },
      {
        name: "SIGINFO",
        number: 29,
        action: "ignore",
        description: "Request for process information",
        standard: "other"
      },
      {
        name: "SIGPWR",
        number: 30,
        action: "terminate",
        description: "Device running out of power",
        standard: "systemv"
      },
      {
        name: "SIGSYS",
        number: 31,
        action: "core",
        description: "Invalid system call",
        standard: "other"
      },
      {
        name: "SIGUNUSED",
        number: 31,
        action: "terminate",
        description: "Invalid system call",
        standard: "other"
      }
    ];
  }
});

// node_modules/human-signals/build/src/signals.js
var import_os, getSignals, normalizeSignal;
var init_signals = __esm({
  "node_modules/human-signals/build/src/signals.js"() {
    import_os = require("os");
    init_core();
    init_realtime();
    getSignals = function() {
      const realtimeSignals = getRealtimeSignals();
      const signals = [...SIGNALS, ...realtimeSignals].map(normalizeSignal);
      return signals;
    };
    normalizeSignal = function({
      name,
      number: defaultNumber,
      description,
      action,
      forced = false,
      standard
    }) {
      const {
        signals: { [name]: constantSignal }
      } = import_os.constants;
      const supported = constantSignal !== void 0;
      const number = supported ? constantSignal : defaultNumber;
      return { name, number, description, supported, action, forced, standard };
    };
  }
});

// node_modules/human-signals/build/src/main.js
var import_os2, getSignalsByName, getSignalByName, signalsByName, getSignalsByNumber, getSignalByNumber, findSignalByNumber, signalsByNumber;
var init_main = __esm({
  "node_modules/human-signals/build/src/main.js"() {
    import_os2 = require("os");
    init_realtime();
    init_signals();
    getSignalsByName = function() {
      const signals = getSignals();
      return signals.reduce(getSignalByName, {});
    };
    getSignalByName = function(signalByNameMemo, { name, number, description, supported, action, forced, standard }) {
      return {
        ...signalByNameMemo,
        [name]: { name, number, description, supported, action, forced, standard }
      };
    };
    signalsByName = getSignalsByName();
    getSignalsByNumber = function() {
      const signals = getSignals();
      const length = SIGRTMAX + 1;
      const signalsA = Array.from({ length }, (value, number) => getSignalByNumber(number, signals));
      return Object.assign({}, ...signalsA);
    };
    getSignalByNumber = function(number, signals) {
      const signal = findSignalByNumber(number, signals);
      if (signal === void 0) {
        return {};
      }
      const { name, description, supported, action, forced, standard } = signal;
      return {
        [number]: {
          name,
          number,
          description,
          supported,
          action,
          forced,
          standard
        }
      };
    };
    findSignalByNumber = function(number, signals) {
      const signal = signals.find(({ name }) => import_os2.constants.signals[name] === number);
      if (signal !== void 0) {
        return signal;
      }
      return signals.find((signalA) => signalA.number === number);
    };
    signalsByNumber = getSignalsByNumber();
  }
});

// node_modules/execa/lib/error.js
var getErrorPrefix, makeError;
var init_error = __esm({
  "node_modules/execa/lib/error.js"() {
    init_main();
    getErrorPrefix = ({ timedOut, timeout, errorCode, signal, signalDescription, exitCode, isCanceled }) => {
      if (timedOut) {
        return `timed out after ${timeout} milliseconds`;
      }
      if (isCanceled) {
        return "was canceled";
      }
      if (errorCode !== void 0) {
        return `failed with ${errorCode}`;
      }
      if (signal !== void 0) {
        return `was killed with ${signal} (${signalDescription})`;
      }
      if (exitCode !== void 0) {
        return `failed with exit code ${exitCode}`;
      }
      return "failed";
    };
    makeError = ({
      stdout,
      stderr,
      all,
      error,
      signal,
      exitCode,
      command,
      escapedCommand,
      timedOut,
      isCanceled,
      killed,
      parsed: { options: { timeout } }
    }) => {
      exitCode = exitCode === null ? void 0 : exitCode;
      signal = signal === null ? void 0 : signal;
      const signalDescription = signal === void 0 ? void 0 : signalsByName[signal].description;
      const errorCode = error && error.code;
      const prefix = getErrorPrefix({ timedOut, timeout, errorCode, signal, signalDescription, exitCode, isCanceled });
      const execaMessage = `Command ${prefix}: ${command}`;
      const isError = Object.prototype.toString.call(error) === "[object Error]";
      const shortMessage = isError ? `${execaMessage}
${error.message}` : execaMessage;
      const message = [shortMessage, stderr, stdout].filter(Boolean).join("\n");
      if (isError) {
        error.originalMessage = error.message;
        error.message = message;
      } else {
        error = new Error(message);
      }
      error.shortMessage = shortMessage;
      error.command = command;
      error.escapedCommand = escapedCommand;
      error.exitCode = exitCode;
      error.signal = signal;
      error.signalDescription = signalDescription;
      error.stdout = stdout;
      error.stderr = stderr;
      if (all !== void 0) {
        error.all = all;
      }
      if ("bufferedData" in error) {
        delete error.bufferedData;
      }
      error.failed = true;
      error.timedOut = Boolean(timedOut);
      error.isCanceled = isCanceled;
      error.killed = killed && !timedOut;
      return error;
    };
  }
});

// node_modules/execa/lib/stdio.js
var aliases, hasAlias, normalizeStdio;
var init_stdio = __esm({
  "node_modules/execa/lib/stdio.js"() {
    aliases = ["stdin", "stdout", "stderr"];
    hasAlias = (options) => aliases.some((alias) => options[alias] !== void 0);
    normalizeStdio = (options) => {
      if (!options) {
        return;
      }
      const { stdio } = options;
      if (stdio === void 0) {
        return aliases.map((alias) => options[alias]);
      }
      if (hasAlias(options)) {
        throw new Error(`It's not possible to provide \`stdio\` in combination with one of ${aliases.map((alias) => `\`${alias}\``).join(", ")}`);
      }
      if (typeof stdio === "string") {
        return stdio;
      }
      if (!Array.isArray(stdio)) {
        throw new TypeError(`Expected \`stdio\` to be of type \`string\` or \`Array\`, got \`${typeof stdio}\``);
      }
      const length = Math.max(stdio.length, aliases.length);
      return Array.from({ length }, (value, index) => stdio[index]);
    };
  }
});

// node_modules/signal-exit/signals.js
var require_signals = __commonJS({
  "node_modules/signal-exit/signals.js"(exports2, module2) {
    module2.exports = [
      "SIGABRT",
      "SIGALRM",
      "SIGHUP",
      "SIGINT",
      "SIGTERM"
    ];
    if (process.platform !== "win32") {
      module2.exports.push(
        "SIGVTALRM",
        "SIGXCPU",
        "SIGXFSZ",
        "SIGUSR2",
        "SIGTRAP",
        "SIGSYS",
        "SIGQUIT",
        "SIGIOT"
        // should detect profiler and enable/disable accordingly.
        // see #21
        // 'SIGPROF'
      );
    }
    if (process.platform === "linux") {
      module2.exports.push(
        "SIGIO",
        "SIGPOLL",
        "SIGPWR",
        "SIGSTKFLT",
        "SIGUNUSED"
      );
    }
  }
});

// node_modules/signal-exit/index.js
var require_signal_exit = __commonJS({
  "node_modules/signal-exit/index.js"(exports2, module2) {
    var process4 = global.process;
    var processOk = function(process5) {
      return process5 && typeof process5 === "object" && typeof process5.removeListener === "function" && typeof process5.emit === "function" && typeof process5.reallyExit === "function" && typeof process5.listeners === "function" && typeof process5.kill === "function" && typeof process5.pid === "number" && typeof process5.on === "function";
    };
    if (!processOk(process4)) {
      module2.exports = function() {
        return function() {
        };
      };
    } else {
      assert = require("assert");
      signals = require_signals();
      isWin = /^win/i.test(process4.platform);
      EE = require("events");
      if (typeof EE !== "function") {
        EE = EE.EventEmitter;
      }
      if (process4.__signal_exit_emitter__) {
        emitter = process4.__signal_exit_emitter__;
      } else {
        emitter = process4.__signal_exit_emitter__ = new EE();
        emitter.count = 0;
        emitter.emitted = {};
      }
      if (!emitter.infinite) {
        emitter.setMaxListeners(Infinity);
        emitter.infinite = true;
      }
      module2.exports = function(cb, opts) {
        if (!processOk(global.process)) {
          return function() {
          };
        }
        assert.equal(typeof cb, "function", "a callback must be provided for exit handler");
        if (loaded === false) {
          load();
        }
        var ev = "exit";
        if (opts && opts.alwaysLast) {
          ev = "afterexit";
        }
        var remove = function() {
          emitter.removeListener(ev, cb);
          if (emitter.listeners("exit").length === 0 && emitter.listeners("afterexit").length === 0) {
            unload();
          }
        };
        emitter.on(ev, cb);
        return remove;
      };
      unload = function unload2() {
        if (!loaded || !processOk(global.process)) {
          return;
        }
        loaded = false;
        signals.forEach(function(sig) {
          try {
            process4.removeListener(sig, sigListeners[sig]);
          } catch (er) {
          }
        });
        process4.emit = originalProcessEmit;
        process4.reallyExit = originalProcessReallyExit;
        emitter.count -= 1;
      };
      module2.exports.unload = unload;
      emit = function emit2(event, code, signal) {
        if (emitter.emitted[event]) {
          return;
        }
        emitter.emitted[event] = true;
        emitter.emit(event, code, signal);
      };
      sigListeners = {};
      signals.forEach(function(sig) {
        sigListeners[sig] = function listener() {
          if (!processOk(global.process)) {
            return;
          }
          var listeners = process4.listeners(sig);
          if (listeners.length === emitter.count) {
            unload();
            emit("exit", null, sig);
            emit("afterexit", null, sig);
            if (isWin && sig === "SIGHUP") {
              sig = "SIGINT";
            }
            process4.kill(process4.pid, sig);
          }
        };
      });
      module2.exports.signals = function() {
        return signals;
      };
      loaded = false;
      load = function load2() {
        if (loaded || !processOk(global.process)) {
          return;
        }
        loaded = true;
        emitter.count += 1;
        signals = signals.filter(function(sig) {
          try {
            process4.on(sig, sigListeners[sig]);
            return true;
          } catch (er) {
            return false;
          }
        });
        process4.emit = processEmit;
        process4.reallyExit = processReallyExit;
      };
      module2.exports.load = load;
      originalProcessReallyExit = process4.reallyExit;
      processReallyExit = function processReallyExit2(code) {
        if (!processOk(global.process)) {
          return;
        }
        process4.exitCode = code || /* istanbul ignore next */
        0;
        emit("exit", process4.exitCode, null);
        emit("afterexit", process4.exitCode, null);
        originalProcessReallyExit.call(process4, process4.exitCode);
      };
      originalProcessEmit = process4.emit;
      processEmit = function processEmit2(ev, arg) {
        if (ev === "exit" && processOk(global.process)) {
          if (arg !== void 0) {
            process4.exitCode = arg;
          }
          var ret = originalProcessEmit.apply(this, arguments);
          emit("exit", process4.exitCode, null);
          emit("afterexit", process4.exitCode, null);
          return ret;
        } else {
          return originalProcessEmit.apply(this, arguments);
        }
      };
    }
    var assert;
    var signals;
    var isWin;
    var EE;
    var emitter;
    var unload;
    var emit;
    var sigListeners;
    var loaded;
    var load;
    var originalProcessReallyExit;
    var processReallyExit;
    var originalProcessEmit;
    var processEmit;
  }
});

// node_modules/execa/lib/kill.js
var import_node_os, import_signal_exit, DEFAULT_FORCE_KILL_TIMEOUT, spawnedKill, setKillTimeout, shouldForceKill, isSigterm, getForceKillAfterTimeout, spawnedCancel, timeoutKill, setupTimeout, validateTimeout, setExitHandler;
var init_kill = __esm({
  "node_modules/execa/lib/kill.js"() {
    import_node_os = __toESM(require("node:os"), 1);
    import_signal_exit = __toESM(require_signal_exit(), 1);
    DEFAULT_FORCE_KILL_TIMEOUT = 1e3 * 5;
    spawnedKill = (kill, signal = "SIGTERM", options = {}) => {
      const killResult = kill(signal);
      setKillTimeout(kill, signal, options, killResult);
      return killResult;
    };
    setKillTimeout = (kill, signal, options, killResult) => {
      if (!shouldForceKill(signal, options, killResult)) {
        return;
      }
      const timeout = getForceKillAfterTimeout(options);
      const t = setTimeout(() => {
        kill("SIGKILL");
      }, timeout);
      if (t.unref) {
        t.unref();
      }
    };
    shouldForceKill = (signal, { forceKillAfterTimeout }, killResult) => isSigterm(signal) && forceKillAfterTimeout !== false && killResult;
    isSigterm = (signal) => signal === import_node_os.default.constants.signals.SIGTERM || typeof signal === "string" && signal.toUpperCase() === "SIGTERM";
    getForceKillAfterTimeout = ({ forceKillAfterTimeout = true }) => {
      if (forceKillAfterTimeout === true) {
        return DEFAULT_FORCE_KILL_TIMEOUT;
      }
      if (!Number.isFinite(forceKillAfterTimeout) || forceKillAfterTimeout < 0) {
        throw new TypeError(`Expected the \`forceKillAfterTimeout\` option to be a non-negative integer, got \`${forceKillAfterTimeout}\` (${typeof forceKillAfterTimeout})`);
      }
      return forceKillAfterTimeout;
    };
    spawnedCancel = (spawned, context) => {
      const killResult = spawned.kill();
      if (killResult) {
        context.isCanceled = true;
      }
    };
    timeoutKill = (spawned, signal, reject) => {
      spawned.kill(signal);
      reject(Object.assign(new Error("Timed out"), { timedOut: true, signal }));
    };
    setupTimeout = (spawned, { timeout, killSignal = "SIGTERM" }, spawnedPromise) => {
      if (timeout === 0 || timeout === void 0) {
        return spawnedPromise;
      }
      let timeoutId;
      const timeoutPromise = new Promise((resolve, reject) => {
        timeoutId = setTimeout(() => {
          timeoutKill(spawned, killSignal, reject);
        }, timeout);
      });
      const safeSpawnedPromise = spawnedPromise.finally(() => {
        clearTimeout(timeoutId);
      });
      return Promise.race([timeoutPromise, safeSpawnedPromise]);
    };
    validateTimeout = ({ timeout }) => {
      if (timeout !== void 0 && (!Number.isFinite(timeout) || timeout < 0)) {
        throw new TypeError(`Expected the \`timeout\` option to be a non-negative integer, got \`${timeout}\` (${typeof timeout})`);
      }
    };
    setExitHandler = async (spawned, { cleanup, detached }, timedPromise) => {
      if (!cleanup || detached) {
        return timedPromise;
      }
      const removeExitHandler = (0, import_signal_exit.default)(() => {
        spawned.kill();
      });
      return timedPromise.finally(() => {
        removeExitHandler();
      });
    };
  }
});

// node_modules/is-stream/index.js
function isStream(stream) {
  return stream !== null && typeof stream === "object" && typeof stream.pipe === "function";
}
var init_is_stream = __esm({
  "node_modules/is-stream/index.js"() {
  }
});

// node_modules/get-stream/buffer-stream.js
var require_buffer_stream = __commonJS({
  "node_modules/get-stream/buffer-stream.js"(exports2, module2) {
    "use strict";
    var { PassThrough: PassThroughStream } = require("stream");
    module2.exports = (options) => {
      options = { ...options };
      const { array } = options;
      let { encoding } = options;
      const isBuffer = encoding === "buffer";
      let objectMode = false;
      if (array) {
        objectMode = !(encoding || isBuffer);
      } else {
        encoding = encoding || "utf8";
      }
      if (isBuffer) {
        encoding = null;
      }
      const stream = new PassThroughStream({ objectMode });
      if (encoding) {
        stream.setEncoding(encoding);
      }
      let length = 0;
      const chunks = [];
      stream.on("data", (chunk) => {
        chunks.push(chunk);
        if (objectMode) {
          length = chunks.length;
        } else {
          length += chunk.length;
        }
      });
      stream.getBufferedValue = () => {
        if (array) {
          return chunks;
        }
        return isBuffer ? Buffer.concat(chunks, length) : chunks.join("");
      };
      stream.getBufferedLength = () => length;
      return stream;
    };
  }
});

// node_modules/get-stream/index.js
var require_get_stream = __commonJS({
  "node_modules/get-stream/index.js"(exports2, module2) {
    "use strict";
    var { constants: BufferConstants } = require("buffer");
    var stream = require("stream");
    var { promisify } = require("util");
    var bufferStream = require_buffer_stream();
    var streamPipelinePromisified = promisify(stream.pipeline);
    var MaxBufferError = class extends Error {
      constructor() {
        super("maxBuffer exceeded");
        this.name = "MaxBufferError";
      }
    };
    async function getStream2(inputStream, options) {
      if (!inputStream) {
        throw new Error("Expected a stream");
      }
      options = {
        maxBuffer: Infinity,
        ...options
      };
      const { maxBuffer } = options;
      const stream2 = bufferStream(options);
      await new Promise((resolve, reject) => {
        const rejectPromise = (error) => {
          if (error && stream2.getBufferedLength() <= BufferConstants.MAX_LENGTH) {
            error.bufferedData = stream2.getBufferedValue();
          }
          reject(error);
        };
        (async () => {
          try {
            await streamPipelinePromisified(inputStream, stream2);
            resolve();
          } catch (error) {
            rejectPromise(error);
          }
        })();
        stream2.on("data", () => {
          if (stream2.getBufferedLength() > maxBuffer) {
            rejectPromise(new MaxBufferError());
          }
        });
      });
      return stream2.getBufferedValue();
    }
    module2.exports = getStream2;
    module2.exports.buffer = (stream2, options) => getStream2(stream2, { ...options, encoding: "buffer" });
    module2.exports.array = (stream2, options) => getStream2(stream2, { ...options, array: true });
    module2.exports.MaxBufferError = MaxBufferError;
  }
});

// node_modules/merge-stream/index.js
var require_merge_stream = __commonJS({
  "node_modules/merge-stream/index.js"(exports2, module2) {
    "use strict";
    var { PassThrough } = require("stream");
    module2.exports = function() {
      var sources = [];
      var output = new PassThrough({ objectMode: true });
      output.setMaxListeners(0);
      output.add = add;
      output.isEmpty = isEmpty;
      output.on("unpipe", remove);
      Array.prototype.slice.call(arguments).forEach(add);
      return output;
      function add(source) {
        if (Array.isArray(source)) {
          source.forEach(add);
          return this;
        }
        sources.push(source);
        source.once("end", remove.bind(null, source));
        source.once("error", output.emit.bind(output, "error"));
        source.pipe(output, { end: false });
        return this;
      }
      function isEmpty() {
        return sources.length == 0;
      }
      function remove(source) {
        sources = sources.filter(function(it) {
          return it !== source;
        });
        if (!sources.length && output.readable) {
          output.end();
        }
      }
    };
  }
});

// node_modules/execa/lib/stream.js
var import_get_stream, import_merge_stream, handleInput, makeAllStream, getBufferedData, getStreamPromise, getSpawnedResult;
var init_stream = __esm({
  "node_modules/execa/lib/stream.js"() {
    init_is_stream();
    import_get_stream = __toESM(require_get_stream(), 1);
    import_merge_stream = __toESM(require_merge_stream(), 1);
    handleInput = (spawned, input) => {
      if (input === void 0 || spawned.stdin === void 0) {
        return;
      }
      if (isStream(input)) {
        input.pipe(spawned.stdin);
      } else {
        spawned.stdin.end(input);
      }
    };
    makeAllStream = (spawned, { all }) => {
      if (!all || !spawned.stdout && !spawned.stderr) {
        return;
      }
      const mixed = (0, import_merge_stream.default)();
      if (spawned.stdout) {
        mixed.add(spawned.stdout);
      }
      if (spawned.stderr) {
        mixed.add(spawned.stderr);
      }
      return mixed;
    };
    getBufferedData = async (stream, streamPromise) => {
      if (!stream) {
        return;
      }
      stream.destroy();
      try {
        return await streamPromise;
      } catch (error) {
        return error.bufferedData;
      }
    };
    getStreamPromise = (stream, { encoding, buffer, maxBuffer }) => {
      if (!stream || !buffer) {
        return;
      }
      if (encoding) {
        return (0, import_get_stream.default)(stream, { encoding, maxBuffer });
      }
      return import_get_stream.default.buffer(stream, { maxBuffer });
    };
    getSpawnedResult = async ({ stdout, stderr, all }, { encoding, buffer, maxBuffer }, processDone) => {
      const stdoutPromise = getStreamPromise(stdout, { encoding, buffer, maxBuffer });
      const stderrPromise = getStreamPromise(stderr, { encoding, buffer, maxBuffer });
      const allPromise = getStreamPromise(all, { encoding, buffer, maxBuffer: maxBuffer * 2 });
      try {
        return await Promise.all([processDone, stdoutPromise, stderrPromise, allPromise]);
      } catch (error) {
        return Promise.all([
          { error, signal: error.signal, timedOut: error.timedOut },
          getBufferedData(stdout, stdoutPromise),
          getBufferedData(stderr, stderrPromise),
          getBufferedData(all, allPromise)
        ]);
      }
    };
  }
});

// node_modules/execa/lib/promise.js
var nativePromisePrototype, descriptors, mergePromise, getSpawnedPromise;
var init_promise = __esm({
  "node_modules/execa/lib/promise.js"() {
    nativePromisePrototype = (async () => {
    })().constructor.prototype;
    descriptors = ["then", "catch", "finally"].map((property) => [
      property,
      Reflect.getOwnPropertyDescriptor(nativePromisePrototype, property)
    ]);
    mergePromise = (spawned, promise) => {
      for (const [property, descriptor] of descriptors) {
        const value = typeof promise === "function" ? (...args) => Reflect.apply(descriptor.value, promise(), args) : descriptor.value.bind(promise);
        Reflect.defineProperty(spawned, property, { ...descriptor, value });
      }
      return spawned;
    };
    getSpawnedPromise = (spawned) => new Promise((resolve, reject) => {
      spawned.on("exit", (exitCode, signal) => {
        resolve({ exitCode, signal });
      });
      spawned.on("error", (error) => {
        reject(error);
      });
      if (spawned.stdin) {
        spawned.stdin.on("error", (error) => {
          reject(error);
        });
      }
    });
  }
});

// node_modules/execa/lib/command.js
var normalizeArgs, NO_ESCAPE_REGEXP, DOUBLE_QUOTES_REGEXP, escapeArg, joinCommand, getEscapedCommand, SPACES_REGEXP, parseCommand;
var init_command = __esm({
  "node_modules/execa/lib/command.js"() {
    normalizeArgs = (file, args = []) => {
      if (!Array.isArray(args)) {
        return [file];
      }
      return [file, ...args];
    };
    NO_ESCAPE_REGEXP = /^[\w.-]+$/;
    DOUBLE_QUOTES_REGEXP = /"/g;
    escapeArg = (arg) => {
      if (typeof arg !== "string" || NO_ESCAPE_REGEXP.test(arg)) {
        return arg;
      }
      return `"${arg.replace(DOUBLE_QUOTES_REGEXP, '\\"')}"`;
    };
    joinCommand = (file, args) => normalizeArgs(file, args).join(" ");
    getEscapedCommand = (file, args) => normalizeArgs(file, args).map((arg) => escapeArg(arg)).join(" ");
    SPACES_REGEXP = / +/g;
    parseCommand = (command) => {
      const tokens = [];
      for (const token of command.trim().split(SPACES_REGEXP)) {
        const previousToken = tokens[tokens.length - 1];
        if (previousToken && previousToken.endsWith("\\")) {
          tokens[tokens.length - 1] = `${previousToken.slice(0, -1)} ${token}`;
        } else {
          tokens.push(token);
        }
      }
      return tokens;
    };
  }
});

// node_modules/execa/index.js
function execa(file, args, options) {
  const parsed = handleArguments(file, args, options);
  const command = joinCommand(file, args);
  const escapedCommand = getEscapedCommand(file, args);
  validateTimeout(parsed.options);
  let spawned;
  try {
    spawned = import_node_child_process.default.spawn(parsed.file, parsed.args, parsed.options);
  } catch (error) {
    const dummySpawned = new import_node_child_process.default.ChildProcess();
    const errorPromise = Promise.reject(makeError({
      error,
      stdout: "",
      stderr: "",
      all: "",
      command,
      escapedCommand,
      parsed,
      timedOut: false,
      isCanceled: false,
      killed: false
    }));
    return mergePromise(dummySpawned, errorPromise);
  }
  const spawnedPromise = getSpawnedPromise(spawned);
  const timedPromise = setupTimeout(spawned, parsed.options, spawnedPromise);
  const processDone = setExitHandler(spawned, parsed.options, timedPromise);
  const context = { isCanceled: false };
  spawned.kill = spawnedKill.bind(null, spawned.kill.bind(spawned));
  spawned.cancel = spawnedCancel.bind(null, spawned, context);
  const handlePromise = async () => {
    const [{ error, exitCode, signal, timedOut }, stdoutResult, stderrResult, allResult] = await getSpawnedResult(spawned, parsed.options, processDone);
    const stdout = handleOutput(parsed.options, stdoutResult);
    const stderr = handleOutput(parsed.options, stderrResult);
    const all = handleOutput(parsed.options, allResult);
    if (error || exitCode !== 0 || signal !== null) {
      const returnedError = makeError({
        error,
        exitCode,
        signal,
        stdout,
        stderr,
        all,
        command,
        escapedCommand,
        parsed,
        timedOut,
        isCanceled: context.isCanceled || (parsed.options.signal ? parsed.options.signal.aborted : false),
        killed: spawned.killed
      });
      if (!parsed.options.reject) {
        return returnedError;
      }
      throw returnedError;
    }
    return {
      command,
      escapedCommand,
      exitCode: 0,
      stdout,
      stderr,
      all,
      failed: false,
      timedOut: false,
      isCanceled: false,
      killed: false
    };
  };
  const handlePromiseOnce = onetime_default(handlePromise);
  handleInput(spawned, parsed.options.input);
  spawned.all = makeAllStream(spawned, parsed.options);
  return mergePromise(spawned, handlePromiseOnce);
}
function execaCommand(command, options) {
  const [file, ...args] = parseCommand(command);
  return execa(file, args, options);
}
var import_node_buffer, import_node_path2, import_node_child_process, import_node_process2, import_cross_spawn, DEFAULT_MAX_BUFFER, getEnv, handleArguments, handleOutput;
var init_execa = __esm({
  "node_modules/execa/index.js"() {
    import_node_buffer = require("node:buffer");
    import_node_path2 = __toESM(require("node:path"), 1);
    import_node_child_process = __toESM(require("node:child_process"), 1);
    import_node_process2 = __toESM(require("node:process"), 1);
    import_cross_spawn = __toESM(require_cross_spawn(), 1);
    init_strip_final_newline();
    init_npm_run_path();
    init_onetime();
    init_error();
    init_stdio();
    init_kill();
    init_stream();
    init_promise();
    init_command();
    DEFAULT_MAX_BUFFER = 1e3 * 1e3 * 100;
    getEnv = ({ env: envOption, extendEnv, preferLocal, localDir, execPath }) => {
      const env = extendEnv ? { ...import_node_process2.default.env, ...envOption } : envOption;
      if (preferLocal) {
        return npmRunPathEnv({ env, cwd: localDir, execPath });
      }
      return env;
    };
    handleArguments = (file, args, options = {}) => {
      const parsed = import_cross_spawn.default._parse(file, args, options);
      file = parsed.command;
      args = parsed.args;
      options = parsed.options;
      options = {
        maxBuffer: DEFAULT_MAX_BUFFER,
        buffer: true,
        stripFinalNewline: true,
        extendEnv: true,
        preferLocal: false,
        localDir: options.cwd || import_node_process2.default.cwd(),
        execPath: import_node_process2.default.execPath,
        encoding: "utf8",
        reject: true,
        cleanup: true,
        all: false,
        windowsHide: true,
        ...options
      };
      options.env = getEnv(options);
      options.stdio = normalizeStdio(options);
      if (import_node_process2.default.platform === "win32" && import_node_path2.default.basename(file, ".exe") === "cmd") {
        args.unshift("/q");
      }
      return { file, args, options, parsed };
    };
    handleOutput = (options, value, error) => {
      if (typeof value !== "string" && !import_node_buffer.Buffer.isBuffer(value)) {
        return error === void 0 ? void 0 : "";
      }
      if (options.stripFinalNewline) {
        return stripFinalNewline(value);
      }
      return value;
    };
  }
});

// node_modules/builtins/index.js
var require_builtins = __commonJS({
  "node_modules/builtins/index.js"(exports2, module2) {
    "use strict";
    var semver2 = require_semver2();
    var permanentModules = [
      "assert",
      "buffer",
      "child_process",
      "cluster",
      "console",
      "constants",
      "crypto",
      "dgram",
      "dns",
      "domain",
      "events",
      "fs",
      "http",
      "https",
      "module",
      "net",
      "os",
      "path",
      "punycode",
      "querystring",
      "readline",
      "repl",
      "stream",
      "string_decoder",
      "sys",
      "timers",
      "tls",
      "tty",
      "url",
      "util",
      "vm",
      "zlib"
    ];
    var versionLockedModules = {
      freelist: "<6.0.0",
      v8: ">=1.0.0",
      process: ">=1.1.0",
      inspector: ">=8.0.0",
      async_hooks: ">=8.1.0",
      http2: ">=8.4.0",
      perf_hooks: ">=8.5.0",
      trace_events: ">=10.0.0",
      worker_threads: ">=12.0.0",
      "node:test": ">=18.0.0"
    };
    var experimentalModules = {
      worker_threads: ">=10.5.0",
      wasi: ">=12.16.0",
      diagnostics_channel: "^14.17.0 || >=15.1.0"
    };
    module2.exports = ({ version = process.version, experimental = false } = {}) => {
      const builtins = [...permanentModules];
      for (const [name, semverRange] of Object.entries(versionLockedModules)) {
        if (version === "*" || semver2.satisfies(version, semverRange)) {
          builtins.push(name);
        }
      }
      if (experimental) {
        for (const [name, semverRange] of Object.entries(experimentalModules)) {
          if (!builtins.includes(name) && (version === "*" || semver2.satisfies(version, semverRange))) {
            builtins.push(name);
          }
        }
      }
      return builtins;
    };
  }
});

// node_modules/validate-npm-package-name/lib/index.js
var require_lib = __commonJS({
  "node_modules/validate-npm-package-name/lib/index.js"(exports2, module2) {
    "use strict";
    var scopedPackagePattern = new RegExp("^(?:@([^/]+?)[/])?([^/]+?)$");
    var builtins = require_builtins();
    var blacklist = [
      "node_modules",
      "favicon.ico"
    ];
    function validate(name) {
      var warnings = [];
      var errors = [];
      if (name === null) {
        errors.push("name cannot be null");
        return done(warnings, errors);
      }
      if (name === void 0) {
        errors.push("name cannot be undefined");
        return done(warnings, errors);
      }
      if (typeof name !== "string") {
        errors.push("name must be a string");
        return done(warnings, errors);
      }
      if (!name.length) {
        errors.push("name length must be greater than zero");
      }
      if (name.match(/^\./)) {
        errors.push("name cannot start with a period");
      }
      if (name.match(/^_/)) {
        errors.push("name cannot start with an underscore");
      }
      if (name.trim() !== name) {
        errors.push("name cannot contain leading or trailing spaces");
      }
      blacklist.forEach(function(blacklistedName) {
        if (name.toLowerCase() === blacklistedName) {
          errors.push(blacklistedName + " is a blacklisted name");
        }
      });
      builtins({ version: "*" }).forEach(function(builtin) {
        if (name.toLowerCase() === builtin) {
          warnings.push(builtin + " is a core module name");
        }
      });
      if (name.length > 214) {
        warnings.push("name can no longer contain more than 214 characters");
      }
      if (name.toLowerCase() !== name) {
        warnings.push("name can no longer contain capital letters");
      }
      if (/[~'!()*]/.test(name.split("/").slice(-1)[0])) {
        warnings.push(`name can no longer contain special characters ("~'!()*")`);
      }
      if (encodeURIComponent(name) !== name) {
        var nameMatch = name.match(scopedPackagePattern);
        if (nameMatch) {
          var user = nameMatch[1];
          var pkg = nameMatch[2];
          if (encodeURIComponent(user) === user && encodeURIComponent(pkg) === pkg) {
            return done(warnings, errors);
          }
        }
        errors.push("name can only contain URL-friendly characters");
      }
      return done(warnings, errors);
    }
    var done = function(warnings, errors) {
      var result = {
        validForNewPackages: errors.length === 0 && warnings.length === 0,
        validForOldPackages: errors.length === 0,
        warnings,
        errors
      };
      if (!result.warnings.length) {
        delete result.warnings;
      }
      if (!result.errors.length) {
        delete result.errors;
      }
      return result;
    };
    module2.exports = validate;
  }
});

// node_modules/npx-import/lib/utils.js
async function _import(packageWithPath) {
  return await import(packageWithPath);
}
async function _importRelative(installDir, packageWithPath) {
  return await import((0, import_node_url2.pathToFileURL)(_resolveRelative(installDir, packageWithPath)).href);
}
function _resolve(packageWithPath) {
  const require2 = (0, import_node_module.createRequire)(import_meta.url);
  return require2.resolve(packageWithPath);
}
function _resolveRelative(installDir, packageWithPath) {
  return (0, import_node_module.createRequire)((0, import_node_url2.pathToFileURL)(installDir).href).resolve(packageWithPath);
}
var import_node_url2, import_node_module, import_meta;
var init_utils = __esm({
  "node_modules/npx-import/lib/utils.js"() {
    import_node_url2 = require("node:url");
    import_node_module = require("node:module");
    import_meta = {};
  }
});

// node_modules/npx-import/lib/index.js
var lib_exports = {};
__export(lib_exports, {
  getPackageManager: () => getPackageManager,
  npxImport: () => npxImport,
  npxResolve: () => npxResolve
});
async function npxImport(pkg, logger = DEFAULT_LOGGER) {
  const packages = await checkPackagesAvailableLocally(pkg);
  const allPackages = Object.values(packages);
  const localPackages = allPackages.filter((p) => p.imported !== NOT_IMPORTABLE);
  const missingPackages = allPackages.filter((p) => p.imported === NOT_IMPORTABLE);
  if (missingPackages.length > 0) {
    logger(`${missingPackages.length > 1 ? `Packages ${missingPackages.map((p) => p.packageWithPath).join(", ")}` : missingPackages[0].packageWithPath} not available locally. Attempting to use npx to install temporarily.`);
    try {
      await checkNpxVersion();
      const installDir = await installAndReturnDir(missingPackages, logger);
      for (const pkg2 of missingPackages) {
        packages[pkg2.name].imported = await _importRelative(installDir, pkg2.packageWithPath);
        INSTALL_CACHE[pkg2.name] = installDir;
      }
    } catch (e) {
      throw new Error(`npx-import failed for ${missingPackages.map((p) => p.packageWithPath).join(",")} with message:
    ${e.message}

You should install ${missingPackages.map((p) => p.name).join(", ")} locally: 
    ` + installInstructions(missingPackages) + `

`);
    }
  }
  for (const pkg2 of localPackages) {
    INSTALL_CACHE[pkg2.name] = INSTALLED_LOCALLY;
  }
  const results = allPackages.map((p) => p.imported);
  return Array.isArray(pkg) ? results : results[0];
}
function npxResolve(pkg) {
  const { name, path: path4 } = parse(pkg);
  const packageWithPath = [name, path4].join("");
  const cachedDir = INSTALL_CACHE[name];
  if (!cachedDir) {
    throw new Error(`You must call npxImport for a package before calling npxResolve. Got: ${pkg}`);
  } else if (cachedDir === INSTALLED_LOCALLY) {
    return _resolve(packageWithPath);
  } else {
    return _resolveRelative(cachedDir, packageWithPath);
  }
}
async function checkPackagesAvailableLocally(pkg) {
  const packages = {};
  for (const p of Array.isArray(pkg) ? pkg : [pkg]) {
    const { name, version, path: path4, exact } = parseAndValidate(p);
    if (packages[name])
      throw new Error(`npx-import cannot import the same package twice! Got: '${p}' but already saw '${name}' earlier!`);
    const packageWithPath = [name, path4].join("");
    const imported = await tryImport(packageWithPath);
    packages[name] = {
      name,
      packageWithPath,
      version,
      exact,
      path: path4,
      imported,
      local: imported !== NOT_IMPORTABLE
    };
  }
  return packages;
}
function parseAndValidate(p) {
  if (p.match(/^[.\/]/)) {
    throw new Error(`npx-import can only import packages, not relative paths: got ${p}`);
  }
  const { name, version, path: path4 } = parse(p);
  const validation = (0, import_validate_npm_package_name.default)(name);
  if (!validation.validForNewPackages) {
    if (validation.warnings?.some((w) => w.match(/is a core module name/)))
      throw new Error(`npx-import can only import NPM packages, got core module '${name}' from '${p}'`);
    else
      throw new Error(`npx-import can't import invalid package name: parsed name '${name}' from '${p}'`);
  }
  return { name, version, path: path4, exact: import_semver.default.parse(version) !== null };
}
async function tryImport(packageWithPath) {
  try {
    return await _import(packageWithPath);
  } catch (e) {
    return NOT_IMPORTABLE;
  }
}
async function checkNpxVersion() {
  const versionCmd = `npx --version`;
  const { failed, stdout: npmVersion } = await execaCommand(versionCmd);
  if (failed) {
    throw new Error(`Couldn't execute ${versionCmd}. Is npm installed and up-to-date?`);
  }
  if (!import_semver.default.gte(npmVersion, "7.0.0")) {
    throw new Error(`Require npm version 7+. Got '${npmVersion}' when running '${versionCmd}'`);
  }
}
async function installAndReturnDir(packages, logger) {
  const offline = packages.every((p) => p.exact);
  const installPackage = `npx --prefer-${offline ? "offline" : "online"} -y ${packages.map((p) => `-p ${formatForCLI(p)}`).join(" ")}`;
  logger(`Installing... (${installPackage})`);
  const emitPath = WINDOWS ? `set PATH` : `printenv PATH`;
  const fullCmd = `${installPackage} ${emitPath}`;
  const { failed, stdout } = await execaCommand(fullCmd, {
    shell: true
  });
  if (failed) {
    throw new Error(`Failed installing ${packages.map((p) => p.name).join(",")} using: ${installPackage}.`);
  }
  const tempPath = getTempPath(stdout);
  const nodeModulesPath = import_node_path3.default.resolve(tempPath, "..");
  if (!nodeModulesPath.endsWith("node_modules")) {
    throw new Error(`Found NPX temporary path of '${tempPath}' but expected to be able to find a node_modules directory by looking in '..'.`);
  }
  logger(`Installed into ${nodeModulesPath}.`);
  logger(`To skip this step in future, run: ${installInstructions(packages)}`);
  return nodeModulesPath;
}
function installInstructions(packages) {
  return INSTRUCTIONS[getPackageManager()](packages.map(formatForCLI).join(" "));
}
function getPackageManager() {
  const userAgent = process.env.npm_config_user_agent;
  if (userAgent) {
    if (userAgent.startsWith("pnpm"))
      return "pnpm";
    if (userAgent.startsWith("yarn"))
      return "yarn";
    if (userAgent.startsWith("npm"))
      return "npm";
  }
  const execpath = process.env.npm_execpath;
  if (execpath) {
    if (/np[xm]-cli\.js$/.exec(execpath))
      return "npm";
    if (/yarn$/.exec(execpath))
      return "yarn";
  }
  const mainModulePath = process.mainModule?.path;
  if (mainModulePath) {
    if (/\/\.?pnpm\//.exec(mainModulePath))
      return "pnpm";
    if (/\/\.?yarn\//.exec(mainModulePath))
      return "yarn";
  }
  return "npm";
}
function getTempPath(stdout) {
  if (WINDOWS) {
    const paths = stdout.replace(/^PATH=/i, "").replace(/\\\\\\\\/g, "\\\\").replace(/\\r\\n/g, ";").split(";");
    const tempPath = paths.find((p) => /\\npm[-\\]+cache\\_npx\\/.exec(p));
    if (!tempPath)
      throw new Error(`Failed to find temporary install directory. Looking for paths matching '\\npm-cache\\_npx\\' in:
${JSON.stringify(paths)}`);
    return tempPath;
  } else {
    const paths = stdout.split(":");
    const tempPath = paths.find((p) => /\/\.npm\/_npx\//.exec(p));
    if (!tempPath)
      throw new Error(`Failed to find temporary install directory. Looking for paths matching '/.npm/_npx/' in:
${JSON.stringify(paths)}`);
    return tempPath;
  }
}
var import_node_os2, import_node_path3, import_semver, import_validate_npm_package_name, WINDOWS, NOT_IMPORTABLE, INSTALLED_LOCALLY, INSTALL_CACHE, DEFAULT_LOGGER, INSTRUCTIONS, formatForCLI;
var init_lib = __esm({
  "node_modules/npx-import/lib/index.js"() {
    import_node_os2 = __toESM(require("node:os"), 1);
    import_node_path3 = __toESM(require("node:path"), 1);
    import_semver = __toESM(require_semver2(), 1);
    init_dist();
    init_execa();
    import_validate_npm_package_name = __toESM(require_lib(), 1);
    init_utils();
    WINDOWS = import_node_os2.default.platform() === "win32";
    NOT_IMPORTABLE = Symbol();
    INSTALLED_LOCALLY = Symbol();
    INSTALL_CACHE = {};
    DEFAULT_LOGGER = process.env.NPX_IMPORT_QUIET ? () => {
    } : (message) => console.log(`[NPXI] ${message}`);
    INSTRUCTIONS = {
      npm: (packageName) => `npm install --save-dev ${packageName}`,
      pnpm: (packageName) => `pnpm add -D ${packageName}`,
      yarn: (packageName) => `yarn add -D ${packageName}`
    };
    formatForCLI = (p) => {
      const unescaped = `${p.name}@${p.version}`;
      return unescaped.match(/[<>*]/) ? `'${unescaped}'` : unescaped;
    };
  }
});

// node_modules/@miniflare/shared/dist/src/index.js
var require_src = __commonJS({
  "node_modules/@miniflare/shared/dist/src/index.js"(exports2) {
    var __create2 = Object.create;
    var __defProp2 = Object.defineProperty;
    var __getOwnPropDesc2 = Object.getOwnPropertyDescriptor;
    var __getOwnPropNames2 = Object.getOwnPropertyNames;
    var __getProtoOf2 = Object.getPrototypeOf;
    var __hasOwnProp2 = Object.prototype.hasOwnProperty;
    var __markAsModule = (target) => __defProp2(target, "__esModule", { value: true });
    var __export2 = (target, all) => {
      __markAsModule(target);
      for (var name in all)
        __defProp2(target, name, { get: all[name], enumerable: true });
    };
    var __reExport = (target, module22, desc) => {
      if (module22 && typeof module22 === "object" || typeof module22 === "function") {
        for (let key of __getOwnPropNames2(module22))
          if (!__hasOwnProp2.call(target, key) && key !== "default")
            __defProp2(target, key, { get: () => module22[key], enumerable: !(desc = __getOwnPropDesc2(module22, key)) || desc.enumerable });
      }
      return target;
    };
    var __toModule = (module22) => {
      return __reExport(__markAsModule(__defProp2(module22 != null ? __create2(__getProtoOf2(module22)) : {}, "default", module22 && module22.__esModule && "default" in module22 ? { get: () => module22.default, enumerable: true } : { value: module22, enumerable: true })), module22);
    };
    __export2(exports2, {
      Compatibility: () => Compatibility,
      EXTERNAL_SUBREQUEST_LIMIT_BUNDLED: () => EXTERNAL_SUBREQUEST_LIMIT_BUNDLED,
      EXTERNAL_SUBREQUEST_LIMIT_UNBOUND: () => EXTERNAL_SUBREQUEST_LIMIT_UNBOUND,
      INTERNAL_SUBREQUEST_LIMIT: () => INTERNAL_SUBREQUEST_LIMIT,
      InputGate: () => InputGate,
      InputGatedEventTarget: () => InputGatedEventTarget,
      Log: () => Log,
      LogLevel: () => LogLevel2,
      MiniflareError: () => MiniflareError,
      Mutex: () => Mutex,
      NoOpLog: () => NoOpLog,
      Option: () => Option,
      OptionType: () => OptionType,
      OutputGate: () => OutputGate,
      Plugin: () => Plugin,
      RequestContext: () => RequestContext,
      SITES_NO_CACHE_PREFIX: () => SITES_NO_CACHE_PREFIX,
      STRING_SCRIPT_PATH: () => STRING_SCRIPT_PATH,
      Storage: () => Storage,
      ThrowingEventTarget: () => ThrowingEventTarget,
      TypedEventTarget: () => TypedEventTarget,
      addAll: () => addAll,
      arrayCompare: () => arrayCompare,
      assertInRequest: () => assertInRequest,
      base64Decode: () => base64Decode,
      base64Encode: () => base64Encode,
      createSQLiteDB: () => createSQLiteDB,
      defaultClock: () => defaultClock,
      getRequestContext: () => getRequestContext,
      getSQLiteNativeBindingLocation: () => getSQLiteNativeBindingLocation,
      globsToMatcher: () => globsToMatcher,
      kGetConsumer: () => kGetConsumer,
      kSetConsumer: () => kSetConsumer,
      kWrapListener: () => kWrapListener,
      kebabCase: () => kebabCase,
      lexicographicCompare: () => lexicographicCompare,
      logOptions: () => logOptions,
      millisToSeconds: () => millisToSeconds,
      nonCircularClone: () => nonCircularClone,
      numericCompare: () => numericCompare,
      parseRange: () => parseRange,
      prefixError: () => prefixError,
      randomHex: () => randomHex,
      resolveStoragePersist: () => resolveStoragePersist,
      runWithInputGateClosed: () => runWithInputGateClosed,
      sanitisePath: () => sanitisePath,
      spaceCase: () => spaceCase,
      structuredCloneImpl: () => structuredCloneImpl,
      titleCase: () => titleCase,
      usageModelExternalSubrequestLimit: () => usageModelExternalSubrequestLimit,
      viewToArray: () => viewToArray,
      viewToBuffer: () => viewToBuffer,
      waitForOpenInputGate: () => waitForOpenInputGate,
      waitForOpenOutputGate: () => waitForOpenOutputGate,
      waitUntilOnOutputGate: () => waitUntilOnOutputGate
    });
    var import_assert = __toModule(require("assert"));
    var import_path = __toModule(require("path"));
    var import_util = __toModule(require("util"));
    var import_worker_threads = __toModule(require("worker_threads"));
    var import_picomatch = __toModule(require_picomatch2());
    var encoder = new import_util.TextEncoder();
    var numericCompare = new Intl.Collator(void 0, { numeric: true }).compare;
    function arrayCompare(a, b) {
      const minLength = Math.min(a.length, b.length);
      for (let i = 0; i < minLength; i++) {
        const aElement = a[i];
        const bElement = b[i];
        if (aElement < bElement)
          return -1;
        if (aElement > bElement)
          return 1;
      }
      return a.length - b.length;
    }
    function lexicographicCompare(x, y) {
      const xEncoded = encoder.encode(x);
      const yEncoded = encoder.encode(y);
      return arrayCompare(xEncoded, yEncoded);
    }
    function nonCircularClone(value) {
      return JSON.parse(JSON.stringify(value));
    }
    var channel;
    function structuredCloneImpl(value, options) {
      channel ??= new import_worker_threads.MessageChannel();
      channel.port1.unref();
      channel.port2.unref();
      channel.port1.postMessage(value, options?.transfer);
      const message = (0, import_worker_threads.receiveMessageOnPort)(channel.port2);
      (0, import_assert.default)(message !== void 0);
      return message.message;
    }
    function addAll(set, values) {
      for (const value of values)
        set.add(value);
    }
    function viewToArray(view) {
      return new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
    }
    function viewToBuffer(view) {
      return view.buffer.slice(view.byteOffset, view.byteOffset + view.byteLength);
    }
    function base64Encode(value) {
      return Buffer.from(value, "utf8").toString("base64");
    }
    function base64Decode(encoded) {
      return Buffer.from(encoded, "base64").toString("utf8");
    }
    function randomHex(digits = 8) {
      return Array.from(Array(digits)).map(() => Math.floor(Math.random() * 16).toString(16)).join("");
    }
    var SITES_NO_CACHE_PREFIX = "$__MINIFLARE_SITES__$/";
    function globsToMatcher(globs = []) {
      const matchGlobs = [];
      const ignoreGlobs = [];
      for (const glob of globs) {
        if (glob.startsWith("!")) {
          ignoreGlobs.push(glob.slice(1));
        } else {
          matchGlobs.push(glob);
        }
      }
      const isMatch = (0, import_picomatch.default)(matchGlobs, {
        dot: true,
        bash: true,
        contains: true,
        ignore: ignoreGlobs
      });
      return {
        test: (string) => isMatch(string),
        toString: () => globs.join(", ")
      };
    }
    function kebabCase(s) {
      return s.replace(/[A-Z]/g, (sub) => `-${sub.toLowerCase()}`);
    }
    function spaceCase(s) {
      s = s.replace(/(.)([A-Z][a-z]+)/g, "$1 $2");
      return s.replace(/([a-z0-9])([A-Z])/g, "$1 $2");
    }
    function titleCase(s) {
      return spaceCase(s).split(" ").map((s2) => s2 ? s2[0].toUpperCase() + s2.substring(1) : s2).join(" ");
    }
    var urlRegexp = /^([a-z]+:)?\/\//i;
    function resolveStoragePersist(rootPath, persist) {
      if (typeof persist === "string") {
        if (urlRegexp.test(persist))
          return persist;
        return import_path.default.resolve(rootPath, persist);
      }
      return persist;
    }
    var namespaceRegexp = /[/\\:|]/g;
    var dotRegexp = /(^|\/|\\)(\.+)(\/|\\|$)/g;
    var illegalRegexp = /[?<>*"'^\x00-\x1f\x80-\x9f]/g;
    var windowsReservedRegexp = /^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$/i;
    var leadingRegexp = /^[ /\\]+/;
    var trailingRegexp = /[ /\\]+$/;
    function dotReplacement(match, g1, g2, g3) {
      return `${g1}${"".padStart(g2.length, "_")}${g3}`;
    }
    function underscoreReplacement(match) {
      return "".padStart(match.length, "_");
    }
    function sanitisePath(unsafe) {
      return unsafe.replace(namespaceRegexp, import_path.default.sep).replace(dotRegexp, dotReplacement).replace(dotRegexp, dotReplacement).replace(illegalRegexp, "_").replace(windowsReservedRegexp, "_").replace(leadingRegexp, underscoreReplacement).replace(trailingRegexp, underscoreReplacement).substring(0, 255);
    }
    var FEATURES = [
      {
        enableFlag: "nodejs_compat"
      },
      {
        defaultAsOf: "2022-11-30",
        enableFlag: "streams_enable_constructors",
        disableFlag: "streams_disable_constructors"
      },
      {
        defaultAsOf: "2022-11-30",
        enableFlag: "transformstream_enable_standard_constructor",
        disableFlag: "transformstream_disable_standard_constructor"
      },
      {
        defaultAsOf: "2022-10-31",
        enableFlag: "export_commonjs_default",
        disableFlag: "export_commonjs_namespace"
      },
      {
        defaultAsOf: "2022-08-04",
        enableFlag: "r2_list_honor_include"
      },
      {
        defaultAsOf: "2022-03-21",
        enableFlag: "global_navigator",
        disableFlag: "no_global_navigator"
      },
      {
        defaultAsOf: "2021-11-10",
        enableFlag: "durable_object_fetch_requires_full_url",
        disableFlag: "durable_object_fetch_allows_relative_url"
      },
      {
        defaultAsOf: "2021-11-10",
        enableFlag: "fetch_refuses_unknown_protocols",
        disableFlag: "fetch_treats_unknown_protocols_as_http"
      },
      {
        defaultAsOf: "2021-11-03",
        enableFlag: "formdata_parser_supports_files",
        disableFlag: "formdata_parser_converts_files_to_strings"
      },
      {
        enableFlag: "html_rewriter_treats_esi_include_as_void_tag"
      },
      {
        enableFlag: "experimental"
      }
    ];
    var Compatibility = class {
      constructor(compatibilityDate = "1970-01-01", compatibilityFlags = []) {
        this.compatibilityDate = compatibilityDate;
        this.compatibilityFlags = compatibilityFlags;
        this.#rebuildEnabled();
      }
      #enabled = /* @__PURE__ */ new Set();
      #rebuildEnabled() {
        this.#enabled.clear();
        const flags = new Set(this.compatibilityFlags);
        for (const { defaultAsOf, enableFlag, disableFlag } of FEATURES) {
          const disabledExplicitly = disableFlag && flags.has(disableFlag);
          if (disabledExplicitly)
            continue;
          const enabledExplicitly = flags.has(enableFlag);
          const enabledAutomatically = defaultAsOf && numericCompare(defaultAsOf, this.compatibilityDate) <= 0;
          if (enabledExplicitly || enabledAutomatically) {
            this.#enabled.add(enableFlag);
          }
        }
      }
      isEnabled(flag) {
        return this.#enabled.has(flag);
      }
      update(compatibilityDate = "1970-01-01", compatibilityFlags = []) {
        if (this.compatibilityDate === compatibilityDate && this.compatibilityFlags.length === compatibilityFlags.length && this.compatibilityFlags.every((flag, i) => compatibilityFlags[i] === flag)) {
          return false;
        }
        this.compatibilityDate = compatibilityDate;
        this.compatibilityFlags = compatibilityFlags;
        this.#rebuildEnabled();
        return true;
      }
      get enabled() {
        return [...this.#enabled];
      }
    };
    var import_assert2 = __toModule(require("assert"));
    var import_async_hooks = __toModule(require("async_hooks"));
    function parseSubrequestOverride(limit) {
      const parsed = parseInt(limit);
      if (Number.isNaN(parsed))
        return void 0;
      if (parsed < 0)
        return false;
      return parsed;
    }
    var EXTERNAL_SUBREQUEST_LIMIT_OVERRIDE = parseSubrequestOverride(process.env.MINIFLARE_SUBREQUEST_LIMIT);
    var INTERNAL_SUBREQUEST_LIMIT_OVERRIDE = parseSubrequestOverride(process.env.MINIFLARE_INTERNAL_SUBREQUEST_LIMIT);
    var EXTERNAL_SUBREQUEST_LIMIT_BUNDLED = 50;
    var EXTERNAL_SUBREQUEST_LIMIT_UNBOUND = 1e3;
    var INTERNAL_SUBREQUEST_LIMIT = 1e3;
    function usageModelExternalSubrequestLimit(model) {
      return model === "unbound" ? EXTERNAL_SUBREQUEST_LIMIT_UNBOUND : EXTERNAL_SUBREQUEST_LIMIT_BUNDLED;
    }
    var MAX_REQUEST_DEPTH = 16;
    var MAX_PIPELINE_DEPTH = 32;
    var depthError = "Subrequest depth limit exceeded. This request recursed through Workers too many times. This can happen e.g. if you have a Worker or Durable Object that calls other Workers or objects recursively.";
    var requestContextStorage = new import_async_hooks.AsyncLocalStorage();
    function getRequestContext() {
      return requestContextStorage.getStore();
    }
    function assertInRequest() {
      if (!getRequestContext()) {
        throw new Error("Some functionality, such as asynchronous I/O (fetch, Cache API, KV), timeouts (setTimeout, setInterval), and generating random values (crypto.getRandomValues, crypto.subtle.generateKey), can only be performed while handling a request.");
      }
    }
    var RequestContext = class {
      requestDepth;
      pipelineDepth;
      durableObject;
      externalSubrequestLimit;
      internalSubrequestLimit;
      #internalSubrequests = 0;
      #externalSubrequests = 0;
      #currentTime;
      constructor({
        requestDepth = 1,
        pipelineDepth = 1,
        durableObject = false,
        externalSubrequestLimit = EXTERNAL_SUBREQUEST_LIMIT_BUNDLED,
        internalSubrequestLimit = INTERNAL_SUBREQUEST_LIMIT
      } = {}) {
        (0, import_assert2.default)(requestDepth >= 1);
        (0, import_assert2.default)(pipelineDepth >= 1);
        if (requestDepth > MAX_REQUEST_DEPTH) {
          throw new Error(`${depthError}
Workers and objects can recurse up to ${MAX_REQUEST_DEPTH} times.
If you're trying to fetch from an origin server, make sure you've set the \`upstream\` option.`);
        }
        if (pipelineDepth > MAX_PIPELINE_DEPTH) {
          throw new Error(`${depthError}
Service bindings can recurse up to ${MAX_PIPELINE_DEPTH} times.`);
        }
        this.requestDepth = requestDepth;
        this.pipelineDepth = pipelineDepth;
        this.durableObject = durableObject;
        this.externalSubrequestLimit = EXTERNAL_SUBREQUEST_LIMIT_OVERRIDE !== void 0 ? EXTERNAL_SUBREQUEST_LIMIT_OVERRIDE : externalSubrequestLimit;
        this.internalSubrequestLimit = INTERNAL_SUBREQUEST_LIMIT_OVERRIDE !== void 0 ? INTERNAL_SUBREQUEST_LIMIT_OVERRIDE : internalSubrequestLimit;
        this.#currentTime = Date.now();
      }
      runWith(closure) {
        return requestContextStorage.run(this, closure);
      }
      get externalSubrequests() {
        return this.#externalSubrequests;
      }
      get internalSubrequests() {
        return this.#internalSubrequests;
      }
      incrementExternalSubrequests(count = 1) {
        this.#externalSubrequests += count;
        if (this.externalSubrequestLimit !== false && this.#externalSubrequests > this.externalSubrequestLimit) {
          throw new Error(`Too many subrequests. Workers can make up to ${this.externalSubrequestLimit} subrequests per request.
A subrequest is a call to fetch(), a redirect, or a call to any Cache API method.`);
        }
      }
      incrementInternalSubrequests(count = 1) {
        this.#internalSubrequests += count;
        if (this.internalSubrequestLimit !== false && this.#internalSubrequests > this.internalSubrequestLimit) {
          throw new Error(`Too many API requests by single worker invocation. Workers can make up to ${this.internalSubrequestLimit} KV and Durable Object requests per invocation.`);
        }
      }
      get currentTime() {
        return this.#currentTime;
      }
      advanceCurrentTime() {
        this.#currentTime = Date.now();
      }
    };
    var MiniflareError = class extends Error {
      constructor(code, message, cause) {
        super(message);
        this.code = code;
        this.cause = cause;
        Object.setPrototypeOf(this, new.target.prototype);
        this.name = `${new.target.name} [${code}]`;
      }
    };
    function prefixError(prefix, e) {
      if (e.stack) {
        return new Proxy(e, {
          get(target, propertyKey, receiver) {
            const value = Reflect.get(target, propertyKey, receiver);
            return propertyKey === "stack" ? `${prefix}: ${value}` : value;
          }
        });
      }
      return e;
    }
    var kWrapListener = Symbol("kWrapListener");
    var TypedEventTarget = class extends EventTarget {
      #wrappedListeners = /* @__PURE__ */ new WeakMap();
      #wrap(listener) {
        if (!listener)
          return null;
        let wrappedListener = this.#wrappedListeners.get(listener);
        if (wrappedListener)
          return wrappedListener;
        wrappedListener = this[kWrapListener]((event) => {
          if (typeof listener === "function") {
            listener(event);
          } else {
            listener.handleEvent(event);
          }
        });
        this.#wrappedListeners.set(listener, wrappedListener);
        return wrappedListener;
      }
      addEventListener(type, listener, options) {
        super.addEventListener(type, this[kWrapListener] ? this.#wrap(listener) : listener, options);
      }
      removeEventListener(type, listener, options) {
        super.removeEventListener(type, this[kWrapListener] ? this.#wrap(listener) : listener, options);
      }
      dispatchEvent(event) {
        return super.dispatchEvent(event);
      }
    };
    var ThrowingEventTarget = class extends TypedEventTarget {
      #wrappedError;
      [kWrapListener](listener) {
        return (event) => {
          try {
            listener(event);
          } catch (error) {
            event.stopImmediatePropagation();
            this.#wrappedError = error;
          }
        };
      }
      dispatchEvent(event) {
        this.#wrappedError = void 0;
        const result = super.dispatchEvent(event);
        if (this.#wrappedError !== void 0)
          throw this.#wrappedError;
        return result;
      }
    };
    var import_path2 = __toModule(require("path"));
    var import_colors = __toModule(require_colors());
    var cwd = process.cwd();
    var cwdNodeModules = import_path2.default.join(cwd, "node_modules");
    var LogLevel2;
    (function(LogLevel22) {
      LogLevel22[LogLevel22["NONE"] = 0] = "NONE";
      LogLevel22[LogLevel22["ERROR"] = 1] = "ERROR";
      LogLevel22[LogLevel22["WARN"] = 2] = "WARN";
      LogLevel22[LogLevel22["INFO"] = 3] = "INFO";
      LogLevel22[LogLevel22["DEBUG"] = 4] = "DEBUG";
      LogLevel22[LogLevel22["VERBOSE"] = 5] = "VERBOSE";
    })(LogLevel2 || (LogLevel2 = {}));
    var LEVEL_PREFIX = {
      [0]: "",
      [1]: "err",
      [2]: "wrn",
      [3]: "inf",
      [4]: "dbg",
      [5]: "vrb"
    };
    var LEVEL_COLOUR = {
      [0]: import_colors.reset,
      [1]: import_colors.red,
      [2]: import_colors.yellow,
      [3]: import_colors.green,
      [4]: import_colors.grey,
      [5]: (input) => (0, import_colors.dim)((0, import_colors.grey)(input))
    };
    function dimInternalStackLine(line) {
      if (line.startsWith("    at") && (!line.includes(cwd) || line.includes(cwdNodeModules))) {
        return (0, import_colors.dim)(line);
      }
      return line;
    }
    var Log = class {
      constructor(level = 3, opts = {}) {
        this.level = level;
        const prefix = opts.prefix ?? "mf";
        const suffix = opts.suffix ?? "";
        this.#prefix = prefix ? prefix + ":" : "";
        this.#suffix = suffix ? ":" + suffix : "";
      }
      #prefix;
      #suffix;
      log(message) {
        console.log(message);
      }
      logWithLevel(level, message) {
        if (level <= this.level) {
          const prefix = `[${this.#prefix}${LEVEL_PREFIX[level]}${this.#suffix}]`;
          this.log(LEVEL_COLOUR[level](`${prefix} ${message}`));
        }
      }
      error(message) {
        if (this.level < 1) {
          throw message;
        } else if (message.stack) {
          const lines = message.stack.split("\n").map(dimInternalStackLine);
          this.logWithLevel(1, lines.join("\n"));
        } else {
          this.logWithLevel(1, message.toString());
        }
        if (message.cause) {
          this.error(prefixError("Cause", message.cause));
        }
      }
      warn(message) {
        this.logWithLevel(2, message);
      }
      info(message) {
        this.logWithLevel(3, message);
      }
      debug(message) {
        this.logWithLevel(4, message);
      }
      verbose(message) {
        this.logWithLevel(5, message);
      }
    };
    var NoOpLog = class extends Log {
      log() {
      }
      error(message) {
        throw message;
      }
    };
    var OptionType;
    (function(OptionType2) {
      OptionType2[OptionType2["NONE"] = 0] = "NONE";
      OptionType2[OptionType2["BOOLEAN"] = 1] = "BOOLEAN";
      OptionType2[OptionType2["NUMBER"] = 2] = "NUMBER";
      OptionType2[OptionType2["STRING"] = 3] = "STRING";
      OptionType2[OptionType2["STRING_POSITIONAL"] = 4] = "STRING_POSITIONAL";
      OptionType2[OptionType2["BOOLEAN_STRING"] = 5] = "BOOLEAN_STRING";
      OptionType2[OptionType2["BOOLEAN_NUMBER"] = 6] = "BOOLEAN_NUMBER";
      OptionType2[OptionType2["ARRAY"] = 7] = "ARRAY";
      OptionType2[OptionType2["OBJECT"] = 8] = "OBJECT";
    })(OptionType || (OptionType = {}));
    function Option(metadata) {
      return function(prototype, key) {
        (prototype.opts ??= /* @__PURE__ */ new Map()).set(key, metadata);
      };
    }
    var Plugin = class {
      constructor(ctx) {
        this.ctx = ctx;
        this.opts = new.target.prototype.opts;
      }
      #phantom;
      opts;
      assignOptions(options) {
        if (options === void 0 || this.opts === void 0)
          return;
        for (const key of this.opts.keys()) {
          this[key] = options[key];
        }
      }
    };
    function logOptions(plugins, log, options) {
      log.debug("Options:");
      for (const [name, plugin] of plugins) {
        const pluginOptions = options[name];
        for (const [key, meta] of plugin.prototype.opts?.entries() ?? []) {
          const value = pluginOptions[key];
          if (value === void 0 || meta.type === 0)
            continue;
          const keyName = meta?.logName ?? titleCase(typeof key === "symbol" ? "<symbol>" : key);
          let str;
          if (meta.logValue) {
            str = meta.logValue(value);
            if (str === void 0)
              continue;
          } else if (meta.type === 8) {
            str = Object.keys(value).join(", ");
          } else if (meta.type === 7) {
            str = value.join(", ");
          } else {
            str = value.toString();
          }
          log.debug(`- ${keyName}: ${str}`);
        }
      }
    }
    var kGetConsumer = Symbol("kGetConsumer");
    var kSetConsumer = Symbol("kSetConsumer");
    var STRING_SCRIPT_PATH = "<script>";
    function parseRange({ offset, length, suffix }, size) {
      if (suffix !== void 0) {
        if (suffix <= 0) {
          throw new Error("Suffix must be > 0");
        }
        if (suffix > size)
          suffix = size;
        offset = size - suffix;
        length = size - offset;
      }
      if (offset === void 0)
        offset = 0;
      if (length === void 0)
        length = size - offset;
      if (offset < 0)
        throw new Error("Offset must be >= 0");
      if (offset > size)
        throw new Error("Offset must be < size");
      if (length <= 0)
        throw new Error("Length must be > 0");
      if (offset + length > size)
        length = size - offset;
      return { offset, length };
    }
    var Storage = class {
      async getSqliteDatabase() {
        throw new Error("D1 not implemented for this Storage class");
      }
      async hasMany(keys) {
        const results = keys.map(this.has.bind(this));
        let count = 0;
        for (const result of await Promise.all(results))
          if (result)
            count++;
        return count;
      }
      getMany(keys, skipMetadata) {
        return Promise.all(keys.map((key) => this.get(key, skipMetadata)));
      }
      async putMany(data) {
        await Promise.all(data.map(([key, value]) => this.put(key, value)));
      }
      async deleteMany(keys) {
        const results = keys.map(this.delete.bind(this));
        let count = 0;
        for (const result of await Promise.all(results))
          if (result)
            count++;
        return count;
      }
    };
    var defaultClock = () => Date.now();
    function millisToSeconds(millis) {
      return Math.floor(millis / 1e3);
    }
    var import_assert3 = __toModule(require("assert"));
    var import_async_hooks2 = __toModule(require("async_hooks"));
    var import_timers = __toModule(require("timers"));
    var import_promises = __toModule(require("timers/promises"));
    var inputGateStorage = new import_async_hooks2.AsyncLocalStorage();
    var outputGateStorage = new import_async_hooks2.AsyncLocalStorage();
    function waitForOpenInputGate() {
      const inputGate = inputGateStorage.getStore();
      return inputGate?.waitForOpen();
    }
    function runWithInputGateClosed(closure, allowConcurrency = false) {
      if (allowConcurrency)
        return closure();
      const inputGate = inputGateStorage.getStore();
      if (inputGate === void 0)
        return closure();
      return inputGate.runWithClosed(closure);
    }
    function waitForOpenOutputGate() {
      const outputGate = outputGateStorage.getStore();
      return outputGate?.waitForOpen();
    }
    function waitUntilOnOutputGate(promise, allowUnconfirmed = false) {
      if (allowUnconfirmed)
        return promise;
      const outputGate = outputGateStorage.getStore();
      outputGate?.waitUntil(promise);
      return promise;
    }
    var InputGate = class {
      #lockCount = 0;
      #resolveQueue = [];
      #parent;
      constructor(parent) {
        this.#parent = parent;
      }
      async runWith(closure) {
        await this.waitForOpen();
        return inputGateStorage.run(this, closure);
      }
      async waitForOpen() {
        await (0, import_promises.setImmediate)();
        if (this.#lockCount === 0)
          return;
        return new Promise((resolve) => this.#resolveQueue.push(resolve));
      }
      async runWithClosed(closure) {
        this.#lock();
        await Promise.resolve();
        const childInputGate = new InputGate(this);
        try {
          return await inputGateStorage.run(childInputGate, closure);
        } finally {
          (0, import_timers.setImmediate)(this.#unlock);
        }
      }
      #lock() {
        this.#lockCount++;
        if (this.#parent)
          this.#parent.#lock();
      }
      #unlock = async () => {
        (0, import_assert3.default)(this.#lockCount > 0);
        this.#lockCount--;
        while (this.#lockCount === 0 && this.#resolveQueue.length) {
          this.#resolveQueue.shift()();
          await (0, import_promises.setImmediate)();
        }
        if (this.#parent)
          return this.#parent.#unlock();
      };
    };
    var OutputGate = class {
      #waitUntil = [];
      async runWith(closure) {
        try {
          return await outputGateStorage.run(this, closure);
        } finally {
          await this.waitForOpen();
        }
      }
      async waitForOpen() {
        await Promise.all(this.#waitUntil);
      }
      waitUntil(promise) {
        this.#waitUntil.push(promise);
      }
    };
    var InputGatedEventTarget = class extends TypedEventTarget {
      [kWrapListener](listener) {
        const inputGate = inputGateStorage.getStore();
        return inputGate ? async (event) => {
          await inputGate.waitForOpen();
          listener(event);
        } : listener;
      }
    };
    var import_assert4 = __toModule(require("assert"));
    var Mutex = class {
      locked = false;
      resolveQueue = [];
      lock() {
        if (!this.locked) {
          this.locked = true;
          return;
        }
        return new Promise((resolve) => this.resolveQueue.push(resolve));
      }
      unlock() {
        (0, import_assert4.default)(this.locked);
        if (this.resolveQueue.length > 0) {
          this.resolveQueue.shift()?.();
        } else {
          this.locked = false;
        }
      }
      get hasWaiting() {
        return this.resolveQueue.length > 0;
      }
      async runWith(closure) {
        const acquireAwaitable = this.lock();
        if (acquireAwaitable instanceof Promise)
          await acquireAwaitable;
        try {
          const awaitable = closure();
          if (awaitable instanceof Promise)
            return await awaitable;
          return awaitable;
        } finally {
          this.unlock();
        }
      }
    };
    var import_node_path4 = __toModule(require("node:path"));
    async function createSQLiteDB(dbPath) {
      const { npxImport: npxImport2, npxResolve: npxResolve2 } = await Promise.resolve().then(() => (init_lib(), lib_exports));
      const { default: DatabaseConstructor } = await npxImport2("better-sqlite3@8.0.1");
      return new DatabaseConstructor(dbPath, {
        nativeBinding: getSQLiteNativeBindingLocation(npxResolve2("better-sqlite3"))
      });
    }
    function getSQLiteNativeBindingLocation(sqliteResolvePath) {
      return import_node_path4.default.resolve(import_node_path4.default.dirname(sqliteResolvePath), "../build/Release/better_sqlite3.node");
    }
  }
});

// node_modules/@miniflare/kv/dist/src/index.js
var require_src2 = __commonJS({
  "node_modules/@miniflare/kv/dist/src/index.js"(exports2) {
    var __create2 = Object.create;
    var __defProp2 = Object.defineProperty;
    var __getOwnPropDesc2 = Object.getOwnPropertyDescriptor;
    var __getOwnPropNames2 = Object.getOwnPropertyNames;
    var __getProtoOf2 = Object.getPrototypeOf;
    var __hasOwnProp2 = Object.prototype.hasOwnProperty;
    var __markAsModule = (target) => __defProp2(target, "__esModule", { value: true });
    var __export2 = (target, all) => {
      __markAsModule(target);
      for (var name in all)
        __defProp2(target, name, { get: all[name], enumerable: true });
    };
    var __reExport = (target, module22, desc) => {
      if (module22 && typeof module22 === "object" || typeof module22 === "function") {
        for (let key of __getOwnPropNames2(module22))
          if (!__hasOwnProp2.call(target, key) && key !== "default")
            __defProp2(target, key, { get: () => module22[key], enumerable: !(desc = __getOwnPropDesc2(module22, key)) || desc.enumerable });
      }
      return target;
    };
    var __toModule = (module22) => {
      return __reExport(__markAsModule(__defProp2(module22 != null ? __create2(__getProtoOf2(module22)) : {}, "default", module22 && module22.__esModule && "default" in module22 ? { get: () => module22.default, enumerable: true } : { value: module22, enumerable: true })), module22);
    };
    var __decorateClass = (decorators, target, key, kind) => {
      var result = kind > 1 ? void 0 : kind ? __getOwnPropDesc2(target, key) : target;
      for (var i = decorators.length - 1, decorator; i >= 0; i--)
        if (decorator = decorators[i])
          result = (kind ? decorator(target, key, result) : decorator(result)) || result;
      if (kind && result)
        __defProp2(target, key, result);
      return result;
    };
    __export2(exports2, {
      KVNamespace: () => KVNamespace2,
      KVPlugin: () => KVPlugin
    });
    var import_consumers = __toModule(require("stream/consumers"));
    var import_web = __toModule(require("stream/web"));
    var import_util = __toModule(require("util"));
    var import_shared = __toModule(require_src());
    var MIN_CACHE_TTL = 60;
    var MIN_EXPIRATION = -2147483648;
    var MAX_EXPIRATION = 2147483647;
    var MAX_LIST_KEYS = 1e3;
    var MAX_KEY_SIZE = 512;
    var MAX_VALUE_SIZE = 25 * 1024 * 1024;
    var MAX_METADATA_SIZE = 1024;
    var keyTypeError = " on 'KvNamespace': parameter 1 is not of type 'string'.";
    var encoder = new import_util.TextEncoder();
    var decoder = new import_util.TextDecoder();
    var getValueTypes = /* @__PURE__ */ new Set(["text", "json", "arrayBuffer", "stream"]);
    function throwKVError(method, status, message) {
      throw new Error(`KV ${method} failed: ${status} ${message}`);
    }
    function validateKey(method, key) {
      if (key === "")
        throw new TypeError("Key name cannot be empty.");
      if (key === ".")
        throw new TypeError('"." is not allowed as a key name.');
      if (key === "..")
        throw new TypeError('".." is not allowed as a key name.');
      const keyLength = encoder.encode(key).byteLength;
      if (keyLength > MAX_KEY_SIZE) {
        throwKVError(method, 414, `UTF-8 encoded length of ${keyLength} exceeds key length limit of ${MAX_KEY_SIZE}.`);
      }
    }
    function validateGetOptions(options) {
      const string = typeof options === "string";
      const type = string ? options : options?.type ?? "text";
      const cacheTtl = string ? void 0 : options?.cacheTtl;
      if (cacheTtl && (isNaN(cacheTtl) || cacheTtl < MIN_CACHE_TTL)) {
        throwKVError("GET", 400, `Invalid cache_ttl of ${cacheTtl}. Cache TTL must be at least ${MIN_CACHE_TTL}.`);
      }
      if (!getValueTypes.has(type)) {
        throw new TypeError('Unknown response type. Possible types are "text", "arrayBuffer", "json", and "stream".');
      }
      return type;
    }
    function normaliseInt(value) {
      switch (typeof value) {
        case "string":
          return parseInt(value);
        case "number":
          return Math.round(value);
      }
    }
    function convertStoredToGetValue(stored, type) {
      switch (type) {
        case "text":
          return decoder.decode(stored);
        case "arrayBuffer":
          return (0, import_shared.viewToBuffer)(stored);
        case "json":
          return JSON.parse(decoder.decode(stored));
        case "stream":
          return new import_web.ReadableStream({
            type: "bytes",
            async pull(controller) {
              await (0, import_shared.waitForOpenInputGate)();
              controller.enqueue(stored);
              controller.close();
              controller.byobRequest?.respond(0);
            }
          });
      }
    }
    var KVNamespace2 = class {
      #storage;
      #clock;
      #blockGlobalAsyncIO;
      constructor(storage, {
        clock = import_shared.defaultClock,
        blockGlobalAsyncIO = false
      } = {}) {
        this.#storage = storage;
        this.#clock = clock;
        this.#blockGlobalAsyncIO = blockGlobalAsyncIO;
      }
      async get(key, options) {
        if (this.#blockGlobalAsyncIO)
          (0, import_shared.assertInRequest)();
        const ctx = (0, import_shared.getRequestContext)();
        ctx?.incrementInternalSubrequests();
        if (typeof key !== "string") {
          throw new TypeError("Failed to execute 'get'" + keyTypeError);
        }
        validateKey("GET", key);
        const type = validateGetOptions(options);
        const stored = await this.#storage.get(key, true);
        await (0, import_shared.waitForOpenInputGate)();
        ctx?.advanceCurrentTime();
        if (stored === void 0)
          return null;
        return convertStoredToGetValue(stored.value, type);
      }
      async getWithMetadata(key, options) {
        if (this.#blockGlobalAsyncIO)
          (0, import_shared.assertInRequest)();
        const ctx = (0, import_shared.getRequestContext)();
        ctx?.incrementInternalSubrequests();
        if (typeof key !== "string") {
          throw new TypeError("Failed to execute 'getWithMetadata'" + keyTypeError);
        }
        validateKey("GET", key);
        const type = validateGetOptions(options);
        const storedValue = await this.#storage.get(key);
        await (0, import_shared.waitForOpenInputGate)();
        ctx?.advanceCurrentTime();
        if (storedValue === void 0)
          return { value: null, metadata: null };
        const { value, metadata = null } = storedValue;
        return { value: convertStoredToGetValue(value, type), metadata };
      }
      async put(key, value, options = {}) {
        if (this.#blockGlobalAsyncIO)
          (0, import_shared.assertInRequest)();
        const ctx = (0, import_shared.getRequestContext)();
        ctx?.incrementInternalSubrequests();
        if (typeof key !== "string") {
          throw new TypeError("Failed to execute 'put'" + keyTypeError);
        }
        validateKey("PUT", key);
        let stored;
        if (typeof value === "string") {
          stored = encoder.encode(value);
        } else if (value instanceof import_web.ReadableStream) {
          stored = new Uint8Array(await (0, import_consumers.arrayBuffer)(value));
        } else if (value instanceof ArrayBuffer) {
          stored = new Uint8Array(value);
        } else if (ArrayBuffer.isView(value)) {
          stored = (0, import_shared.viewToArray)(value);
        } else {
          throw new TypeError("KV put() accepts only strings, ArrayBuffers, ArrayBufferViews, and ReadableStreams as values.");
        }
        const now = (0, import_shared.millisToSeconds)(this.#clock());
        let expiration = normaliseInt(options.expiration);
        const expirationTtl = normaliseInt(options.expirationTtl);
        if (expirationTtl !== void 0) {
          if (expirationTtl < MIN_EXPIRATION || expirationTtl > MAX_EXPIRATION) {
            throw new TypeError(`Value out of range. Must be between ${MIN_EXPIRATION} and ${MAX_EXPIRATION} (inclusive).`);
          }
          if (isNaN(expirationTtl) || expirationTtl <= 0) {
            throwKVError("PUT", 400, `Invalid expiration_ttl of ${options.expirationTtl}. Please specify integer greater than 0.`);
          }
          if (expirationTtl < MIN_CACHE_TTL) {
            throwKVError("PUT", 400, `Invalid expiration_ttl of ${options.expirationTtl}. Expiration TTL must be at least ${MIN_CACHE_TTL}.`);
          }
          expiration = now + expirationTtl;
        } else if (expiration !== void 0) {
          if (expiration < MIN_EXPIRATION || expiration > MAX_EXPIRATION) {
            throw new TypeError(`Value out of range. Must be between ${MIN_EXPIRATION} and ${MAX_EXPIRATION} (inclusive).`);
          }
          if (isNaN(expiration) || expiration <= now) {
            throwKVError("PUT", 400, `Invalid expiration of ${options.expiration}. Please specify integer greater than the current number of seconds since the UNIX epoch.`);
          }
          if (expiration < now + MIN_CACHE_TTL) {
            throwKVError("PUT", 400, `Invalid expiration of ${options.expiration}. Expiration times must be at least ${MIN_CACHE_TTL} seconds in the future.`);
          }
        }
        if (stored.byteLength > MAX_VALUE_SIZE) {
          throwKVError("PUT", 413, `Value length of ${stored.byteLength} exceeds limit of ${MAX_VALUE_SIZE}.`);
        }
        const metadataLength = options.metadata && encoder.encode(JSON.stringify(options.metadata)).byteLength;
        if (metadataLength && metadataLength > MAX_METADATA_SIZE) {
          throwKVError("PUT", 413, `Metadata length of ${metadataLength} exceeds limit of ${MAX_METADATA_SIZE}.`);
        }
        await (0, import_shared.waitForOpenOutputGate)();
        await this.#storage.put(key, {
          value: stored,
          expiration,
          metadata: options.metadata
        });
        await (0, import_shared.waitForOpenInputGate)();
        ctx?.advanceCurrentTime();
      }
      async delete(key) {
        if (this.#blockGlobalAsyncIO)
          (0, import_shared.assertInRequest)();
        const ctx = (0, import_shared.getRequestContext)();
        ctx?.incrementInternalSubrequests();
        if (typeof key !== "string") {
          throw new TypeError("Failed to execute 'delete'" + keyTypeError);
        }
        validateKey("DELETE", key);
        await (0, import_shared.waitForOpenOutputGate)();
        await this.#storage.delete(key);
        await (0, import_shared.waitForOpenInputGate)();
        ctx?.advanceCurrentTime();
      }
      async list({
        prefix = "",
        limit = MAX_LIST_KEYS,
        cursor
      } = {}) {
        if (this.#blockGlobalAsyncIO)
          (0, import_shared.assertInRequest)();
        const ctx = (0, import_shared.getRequestContext)();
        ctx?.incrementInternalSubrequests();
        if (isNaN(limit) || limit < 1) {
          throwKVError("GET", 400, `Invalid key_count_limit of ${limit}. Please specify an integer greater than 0.`);
        }
        if (limit > MAX_LIST_KEYS) {
          throwKVError("GET", 400, `Invalid key_count_limit of ${limit}. Please specify an integer less than ${MAX_LIST_KEYS}.`);
        }
        const res = await this.#storage.list({ prefix, limit, cursor });
        await (0, import_shared.waitForOpenInputGate)();
        ctx?.advanceCurrentTime();
        return {
          keys: res.keys,
          cursor: res.cursor,
          list_complete: res.cursor === ""
        };
      }
    };
    var import_shared2 = __toModule(require_src());
    var KVPlugin = class extends import_shared2.Plugin {
      kvNamespaces;
      kvPersist;
      #persist;
      constructor(ctx, options) {
        super(ctx);
        this.assignOptions(options);
        this.#persist = (0, import_shared2.resolveStoragePersist)(ctx.rootPath, this.kvPersist);
      }
      getNamespace(storage, namespace, blockGlobalAsyncIO = false) {
        return new KVNamespace2(storage.storage(namespace, this.#persist), {
          blockGlobalAsyncIO
        });
      }
      setup(storageFactory) {
        const blockGlobalAsyncIO = !this.ctx.globalAsyncIO;
        const bindings = {};
        for (const namespace of this.kvNamespaces ?? []) {
          bindings[namespace] = this.getNamespace(storageFactory, namespace, blockGlobalAsyncIO);
        }
        return { bindings };
      }
    };
    __decorateClass([
      (0, import_shared2.Option)({
        type: import_shared2.OptionType.ARRAY,
        name: "kv",
        alias: "k",
        description: "KV namespace to bind",
        logName: "KV Namespaces",
        fromWrangler: ({ kv_namespaces }) => kv_namespaces?.map(({ binding }) => binding)
      })
    ], KVPlugin.prototype, "kvNamespaces", 2);
    __decorateClass([
      (0, import_shared2.Option)({
        type: import_shared2.OptionType.BOOLEAN_STRING,
        description: "Persist KV data (to optional path)",
        logName: "KV Persistence",
        fromWrangler: ({ miniflare }) => miniflare?.kv_persist
      })
    ], KVPlugin.prototype, "kvPersist", 2);
  }
});

// node_modules/@miniflare/storage-memory/dist/src/index.js
var require_src3 = __commonJS({
  "node_modules/@miniflare/storage-memory/dist/src/index.js"(exports2) {
    var __create2 = Object.create;
    var __defProp2 = Object.defineProperty;
    var __getOwnPropDesc2 = Object.getOwnPropertyDescriptor;
    var __getOwnPropNames2 = Object.getOwnPropertyNames;
    var __getProtoOf2 = Object.getPrototypeOf;
    var __hasOwnProp2 = Object.prototype.hasOwnProperty;
    var __markAsModule = (target) => __defProp2(target, "__esModule", { value: true });
    var __export2 = (target, all) => {
      __markAsModule(target);
      for (var name in all)
        __defProp2(target, name, { get: all[name], enumerable: true });
    };
    var __reExport = (target, module22, desc) => {
      if (module22 && typeof module22 === "object" || typeof module22 === "function") {
        for (let key of __getOwnPropNames2(module22))
          if (!__hasOwnProp2.call(target, key) && key !== "default")
            __defProp2(target, key, { get: () => module22[key], enumerable: !(desc = __getOwnPropDesc2(module22, key)) || desc.enumerable });
      }
      return target;
    };
    var __toModule = (module22) => {
      return __reExport(__markAsModule(__defProp2(module22 != null ? __create2(__getProtoOf2(module22)) : {}, "default", module22 && module22.__esModule && "default" in module22 ? { get: () => module22.default, enumerable: true } : { value: module22, enumerable: true })), module22);
    };
    __export2(exports2, {
      LocalStorage: () => LocalStorage,
      MemoryStorage: () => MemoryStorage2,
      cloneMetadata: () => cloneMetadata,
      listFilterMatch: () => listFilterMatch,
      listPaginate: () => listPaginate
    });
    var import_shared = __toModule(require_src());
    function cloneMetadata(metadata) {
      return metadata && (0, import_shared.nonCircularClone)(metadata);
    }
    function listFilterMatch(options, name) {
      return !(options?.prefix !== void 0 && !name.startsWith(options.prefix) || options?.excludePrefix !== void 0 && name.startsWith(options.excludePrefix) || options?.start !== void 0 && (0, import_shared.lexicographicCompare)(name, options.start) < 0 || options?.end !== void 0 && (0, import_shared.lexicographicCompare)(name, options.end) >= 0);
    }
    function listPaginate(options, keys) {
      const resKeys = [];
      const direction = options?.reverse ? -1 : 1;
      keys.sort((a, b) => direction * (0, import_shared.lexicographicCompare)(a.name, b.name));
      const startAfter = options?.cursor ? (0, import_shared.base64Decode)(options.cursor) : "";
      let startIndex = 0;
      if (startAfter !== "") {
        startIndex = keys.findIndex(({ name }) => name === startAfter);
        if (startIndex === -1)
          startIndex = keys.length;
        startIndex++;
      }
      let endIndex = startIndex;
      const prefix = options?.prefix ?? "";
      const delimitedPrefixes = /* @__PURE__ */ new Set();
      for (let i = startIndex; i < keys.length; i++) {
        const key = keys[i];
        const { name } = key;
        endIndex = i;
        if (options?.delimiter !== void 0 && name.startsWith(prefix) && name.slice(prefix.length).includes(options.delimiter)) {
          const { delimiter } = options;
          const objectKey = name.slice(prefix.length);
          const delimitedPrefix = prefix + objectKey.split(delimiter)[0] + delimiter;
          delimitedPrefixes.add(delimitedPrefix);
          while (i < keys.length) {
            const nextKey = keys[i];
            const nextName = nextKey.name;
            if (!nextName.startsWith(delimitedPrefix))
              break;
            endIndex = i;
            i++;
          }
          i--;
        } else {
          resKeys.push(key);
        }
        if (options?.limit !== void 0 && resKeys.length + delimitedPrefixes.size >= options.limit) {
          break;
        }
      }
      const nextCursor = endIndex < keys.length - 1 ? (0, import_shared.base64Encode)(keys[endIndex].name) : "";
      const res = {
        keys: resKeys,
        cursor: nextCursor
      };
      if (options?.delimiter !== void 0) {
        res.delimitedPrefixes = Array.from(delimitedPrefixes);
      }
      return res;
    }
    var import_shared2 = __toModule(require_src());
    var LocalStorage = class extends import_shared2.Storage {
      constructor(clock = import_shared2.defaultClock) {
        super();
        this.clock = clock;
      }
      expired({ expiration }, time = this.clock()) {
        return expiration !== void 0 && expiration <= (0, import_shared2.millisToSeconds)(time);
      }
      async has(key) {
        const stored = await this.hasMaybeExpired(key);
        if (stored === void 0)
          return false;
        if (this.expired(stored)) {
          await this.deleteMaybeExpired(key);
          return false;
        }
        return true;
      }
      async head(key) {
        const stored = await this.headMaybeExpired(key);
        if (stored === void 0)
          return void 0;
        if (this.expired(stored)) {
          await this.deleteMaybeExpired(key);
          return void 0;
        }
        return stored;
      }
      async get(key) {
        const stored = await this.getMaybeExpired(key);
        if (stored === void 0)
          return void 0;
        if (this.expired(stored)) {
          await this.deleteMaybeExpired(key);
          return void 0;
        }
        return stored;
      }
      async getRange(key, range = {}) {
        const stored = await this.getRangeMaybeExpired(key, range);
        if (stored === void 0)
          return void 0;
        if (this.expired(stored)) {
          await this.deleteMaybeExpired(key);
          return void 0;
        }
        return stored;
      }
      async delete(key) {
        const stored = await this.hasMaybeExpired(key);
        const expired = stored !== void 0 && this.expired(stored);
        const deleted = await this.deleteMaybeExpired(key);
        if (!deleted)
          return false;
        return !expired;
      }
      async list(options) {
        const time = this.clock();
        const deletePromises = [];
        let keys = await this.listAllMaybeExpired();
        keys = keys.filter((stored) => {
          if (this.expired(stored, time)) {
            deletePromises.push(this.deleteMaybeExpired(stored.name));
            return false;
          }
          return listFilterMatch(options, stored.name);
        });
        const res = listPaginate(options, keys);
        await Promise.all(deletePromises);
        return res;
      }
    };
    var import_shared3 = __toModule(require_src());
    var MemoryStorage2 = class extends LocalStorage {
      constructor(map = /* @__PURE__ */ new Map(), clock = import_shared3.defaultClock) {
        super(clock);
        this.map = map;
      }
      sqliteDB;
      hasMaybeExpired(key) {
        const stored = this.map.get(key);
        return stored && {
          expiration: stored.expiration,
          metadata: cloneMetadata(stored.metadata)
        };
      }
      headMaybeExpired(key) {
        const stored = this.map.get(key);
        return stored && {
          expiration: stored.expiration,
          metadata: cloneMetadata(stored.metadata)
        };
      }
      getMaybeExpired(key) {
        const stored = this.map.get(key);
        return stored && {
          value: stored.value.slice(),
          expiration: stored.expiration,
          metadata: cloneMetadata(stored.metadata)
        };
      }
      getRangeMaybeExpired(key, range) {
        const stored = this.map.get(key);
        if (stored === void 0)
          return;
        const { value } = stored;
        const size = value.byteLength;
        const { offset, length } = (0, import_shared3.parseRange)(range, size);
        return {
          value: value.slice(offset, offset + length),
          expiration: stored.expiration,
          metadata: cloneMetadata(stored.metadata),
          range: {
            offset,
            length
          }
        };
      }
      put(key, value) {
        this.map.set(key, {
          value: value.value.slice(),
          expiration: value.expiration,
          metadata: cloneMetadata(value.metadata)
        });
      }
      deleteMaybeExpired(key) {
        return this.map.delete(key);
      }
      static entryToStoredKey([name, { expiration, metadata }]) {
        return {
          name,
          expiration,
          metadata: cloneMetadata(metadata)
        };
      }
      listAllMaybeExpired() {
        return Array.from(this.map.entries()).map(MemoryStorage2.entryToStoredKey);
      }
      async getSqliteDatabase() {
        if (this.sqliteDB)
          return this.sqliteDB;
        this.sqliteDB = await (0, import_shared3.createSQLiteDB)(":memory:");
        return this.sqliteDB;
      }
    };
  }
});

// node_modules/graphql/version.js
var require_version = __commonJS({
  "node_modules/graphql/version.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.versionInfo = exports2.version = void 0;
    var version = "16.8.1";
    exports2.version = version;
    var versionInfo = Object.freeze({
      major: 16,
      minor: 8,
      patch: 1,
      preReleaseTag: null
    });
    exports2.versionInfo = versionInfo;
  }
});

// node_modules/graphql/jsutils/devAssert.js
var require_devAssert = __commonJS({
  "node_modules/graphql/jsutils/devAssert.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.devAssert = devAssert;
    function devAssert(condition, message) {
      const booleanCondition = Boolean(condition);
      if (!booleanCondition) {
        throw new Error(message);
      }
    }
  }
});

// node_modules/graphql/jsutils/isPromise.js
var require_isPromise = __commonJS({
  "node_modules/graphql/jsutils/isPromise.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isPromise = isPromise;
    function isPromise(value) {
      return typeof (value === null || value === void 0 ? void 0 : value.then) === "function";
    }
  }
});

// node_modules/graphql/jsutils/isObjectLike.js
var require_isObjectLike = __commonJS({
  "node_modules/graphql/jsutils/isObjectLike.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isObjectLike = isObjectLike;
    function isObjectLike(value) {
      return typeof value == "object" && value !== null;
    }
  }
});

// node_modules/graphql/jsutils/invariant.js
var require_invariant = __commonJS({
  "node_modules/graphql/jsutils/invariant.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.invariant = invariant;
    function invariant(condition, message) {
      const booleanCondition = Boolean(condition);
      if (!booleanCondition) {
        throw new Error(
          message != null ? message : "Unexpected invariant triggered."
        );
      }
    }
  }
});

// node_modules/graphql/language/location.js
var require_location = __commonJS({
  "node_modules/graphql/language/location.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.getLocation = getLocation;
    var _invariant = require_invariant();
    var LineRegExp = /\r\n|[\n\r]/g;
    function getLocation(source, position) {
      let lastLineStart = 0;
      let line = 1;
      for (const match of source.body.matchAll(LineRegExp)) {
        typeof match.index === "number" || (0, _invariant.invariant)(false);
        if (match.index >= position) {
          break;
        }
        lastLineStart = match.index + match[0].length;
        line += 1;
      }
      return {
        line,
        column: position + 1 - lastLineStart
      };
    }
  }
});

// node_modules/graphql/language/printLocation.js
var require_printLocation = __commonJS({
  "node_modules/graphql/language/printLocation.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.printLocation = printLocation;
    exports2.printSourceLocation = printSourceLocation;
    var _location = require_location();
    function printLocation(location) {
      return printSourceLocation(
        location.source,
        (0, _location.getLocation)(location.source, location.start)
      );
    }
    function printSourceLocation(source, sourceLocation) {
      const firstLineColumnOffset = source.locationOffset.column - 1;
      const body = "".padStart(firstLineColumnOffset) + source.body;
      const lineIndex = sourceLocation.line - 1;
      const lineOffset = source.locationOffset.line - 1;
      const lineNum = sourceLocation.line + lineOffset;
      const columnOffset = sourceLocation.line === 1 ? firstLineColumnOffset : 0;
      const columnNum = sourceLocation.column + columnOffset;
      const locationStr = `${source.name}:${lineNum}:${columnNum}
`;
      const lines = body.split(/\r\n|[\n\r]/g);
      const locationLine = lines[lineIndex];
      if (locationLine.length > 120) {
        const subLineIndex = Math.floor(columnNum / 80);
        const subLineColumnNum = columnNum % 80;
        const subLines = [];
        for (let i = 0; i < locationLine.length; i += 80) {
          subLines.push(locationLine.slice(i, i + 80));
        }
        return locationStr + printPrefixedLines([
          [`${lineNum} |`, subLines[0]],
          ...subLines.slice(1, subLineIndex + 1).map((subLine) => ["|", subLine]),
          ["|", "^".padStart(subLineColumnNum)],
          ["|", subLines[subLineIndex + 1]]
        ]);
      }
      return locationStr + printPrefixedLines([
        // Lines specified like this: ["prefix", "string"],
        [`${lineNum - 1} |`, lines[lineIndex - 1]],
        [`${lineNum} |`, locationLine],
        ["|", "^".padStart(columnNum)],
        [`${lineNum + 1} |`, lines[lineIndex + 1]]
      ]);
    }
    function printPrefixedLines(lines) {
      const existingLines = lines.filter(([_, line]) => line !== void 0);
      const padLen = Math.max(...existingLines.map(([prefix]) => prefix.length));
      return existingLines.map(([prefix, line]) => prefix.padStart(padLen) + (line ? " " + line : "")).join("\n");
    }
  }
});

// node_modules/graphql/error/GraphQLError.js
var require_GraphQLError = __commonJS({
  "node_modules/graphql/error/GraphQLError.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.GraphQLError = void 0;
    exports2.formatError = formatError;
    exports2.printError = printError;
    var _isObjectLike = require_isObjectLike();
    var _location = require_location();
    var _printLocation = require_printLocation();
    function toNormalizedOptions(args) {
      const firstArg = args[0];
      if (firstArg == null || "kind" in firstArg || "length" in firstArg) {
        return {
          nodes: firstArg,
          source: args[1],
          positions: args[2],
          path: args[3],
          originalError: args[4],
          extensions: args[5]
        };
      }
      return firstArg;
    }
    var GraphQLError2 = class _GraphQLError extends Error {
      /**
       * An array of `{ line, column }` locations within the source GraphQL document
       * which correspond to this error.
       *
       * Errors during validation often contain multiple locations, for example to
       * point out two things with the same name. Errors during execution include a
       * single location, the field which produced the error.
       *
       * Enumerable, and appears in the result of JSON.stringify().
       */
      /**
       * An array describing the JSON-path into the execution response which
       * corresponds to this error. Only included for errors during execution.
       *
       * Enumerable, and appears in the result of JSON.stringify().
       */
      /**
       * An array of GraphQL AST Nodes corresponding to this error.
       */
      /**
       * The source GraphQL document for the first location of this error.
       *
       * Note that if this Error represents more than one node, the source may not
       * represent nodes after the first node.
       */
      /**
       * An array of character offsets within the source GraphQL document
       * which correspond to this error.
       */
      /**
       * The original error thrown from a field resolver during execution.
       */
      /**
       * Extension fields to add to the formatted error.
       */
      /**
       * @deprecated Please use the `GraphQLErrorOptions` constructor overload instead.
       */
      constructor(message, ...rawArgs) {
        var _this$nodes, _nodeLocations$, _ref;
        const { nodes, source, positions, path: path4, originalError, extensions } = toNormalizedOptions(rawArgs);
        super(message);
        this.name = "GraphQLError";
        this.path = path4 !== null && path4 !== void 0 ? path4 : void 0;
        this.originalError = originalError !== null && originalError !== void 0 ? originalError : void 0;
        this.nodes = undefinedIfEmpty(
          Array.isArray(nodes) ? nodes : nodes ? [nodes] : void 0
        );
        const nodeLocations = undefinedIfEmpty(
          (_this$nodes = this.nodes) === null || _this$nodes === void 0 ? void 0 : _this$nodes.map((node) => node.loc).filter((loc) => loc != null)
        );
        this.source = source !== null && source !== void 0 ? source : nodeLocations === null || nodeLocations === void 0 ? void 0 : (_nodeLocations$ = nodeLocations[0]) === null || _nodeLocations$ === void 0 ? void 0 : _nodeLocations$.source;
        this.positions = positions !== null && positions !== void 0 ? positions : nodeLocations === null || nodeLocations === void 0 ? void 0 : nodeLocations.map((loc) => loc.start);
        this.locations = positions && source ? positions.map((pos) => (0, _location.getLocation)(source, pos)) : nodeLocations === null || nodeLocations === void 0 ? void 0 : nodeLocations.map(
          (loc) => (0, _location.getLocation)(loc.source, loc.start)
        );
        const originalExtensions = (0, _isObjectLike.isObjectLike)(
          originalError === null || originalError === void 0 ? void 0 : originalError.extensions
        ) ? originalError === null || originalError === void 0 ? void 0 : originalError.extensions : void 0;
        this.extensions = (_ref = extensions !== null && extensions !== void 0 ? extensions : originalExtensions) !== null && _ref !== void 0 ? _ref : /* @__PURE__ */ Object.create(null);
        Object.defineProperties(this, {
          message: {
            writable: true,
            enumerable: true
          },
          name: {
            enumerable: false
          },
          nodes: {
            enumerable: false
          },
          source: {
            enumerable: false
          },
          positions: {
            enumerable: false
          },
          originalError: {
            enumerable: false
          }
        });
        if (originalError !== null && originalError !== void 0 && originalError.stack) {
          Object.defineProperty(this, "stack", {
            value: originalError.stack,
            writable: true,
            configurable: true
          });
        } else if (Error.captureStackTrace) {
          Error.captureStackTrace(this, _GraphQLError);
        } else {
          Object.defineProperty(this, "stack", {
            value: Error().stack,
            writable: true,
            configurable: true
          });
        }
      }
      get [Symbol.toStringTag]() {
        return "GraphQLError";
      }
      toString() {
        let output = this.message;
        if (this.nodes) {
          for (const node of this.nodes) {
            if (node.loc) {
              output += "\n\n" + (0, _printLocation.printLocation)(node.loc);
            }
          }
        } else if (this.source && this.locations) {
          for (const location of this.locations) {
            output += "\n\n" + (0, _printLocation.printSourceLocation)(this.source, location);
          }
        }
        return output;
      }
      toJSON() {
        const formattedError = {
          message: this.message
        };
        if (this.locations != null) {
          formattedError.locations = this.locations;
        }
        if (this.path != null) {
          formattedError.path = this.path;
        }
        if (this.extensions != null && Object.keys(this.extensions).length > 0) {
          formattedError.extensions = this.extensions;
        }
        return formattedError;
      }
    };
    exports2.GraphQLError = GraphQLError2;
    function undefinedIfEmpty(array) {
      return array === void 0 || array.length === 0 ? void 0 : array;
    }
    function printError(error) {
      return error.toString();
    }
    function formatError(error) {
      return error.toJSON();
    }
  }
});

// node_modules/graphql/error/syntaxError.js
var require_syntaxError = __commonJS({
  "node_modules/graphql/error/syntaxError.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.syntaxError = syntaxError;
    var _GraphQLError = require_GraphQLError();
    function syntaxError(source, position, description) {
      return new _GraphQLError.GraphQLError(`Syntax Error: ${description}`, {
        source,
        positions: [position]
      });
    }
  }
});

// node_modules/graphql/language/ast.js
var require_ast = __commonJS({
  "node_modules/graphql/language/ast.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.Token = exports2.QueryDocumentKeys = exports2.OperationTypeNode = exports2.Location = void 0;
    exports2.isNode = isNode;
    var Location = class {
      /**
       * The character offset at which this Node begins.
       */
      /**
       * The character offset at which this Node ends.
       */
      /**
       * The Token at which this Node begins.
       */
      /**
       * The Token at which this Node ends.
       */
      /**
       * The Source document the AST represents.
       */
      constructor(startToken, endToken, source) {
        this.start = startToken.start;
        this.end = endToken.end;
        this.startToken = startToken;
        this.endToken = endToken;
        this.source = source;
      }
      get [Symbol.toStringTag]() {
        return "Location";
      }
      toJSON() {
        return {
          start: this.start,
          end: this.end
        };
      }
    };
    exports2.Location = Location;
    var Token = class {
      /**
       * The kind of Token.
       */
      /**
       * The character offset at which this Node begins.
       */
      /**
       * The character offset at which this Node ends.
       */
      /**
       * The 1-indexed line number on which this Token appears.
       */
      /**
       * The 1-indexed column number at which this Token begins.
       */
      /**
       * For non-punctuation tokens, represents the interpreted value of the token.
       *
       * Note: is undefined for punctuation tokens, but typed as string for
       * convenience in the parser.
       */
      /**
       * Tokens exist as nodes in a double-linked-list amongst all tokens
       * including ignored tokens. <SOF> is always the first node and <EOF>
       * the last.
       */
      constructor(kind, start, end, line, column, value) {
        this.kind = kind;
        this.start = start;
        this.end = end;
        this.line = line;
        this.column = column;
        this.value = value;
        this.prev = null;
        this.next = null;
      }
      get [Symbol.toStringTag]() {
        return "Token";
      }
      toJSON() {
        return {
          kind: this.kind,
          value: this.value,
          line: this.line,
          column: this.column
        };
      }
    };
    exports2.Token = Token;
    var QueryDocumentKeys = {
      Name: [],
      Document: ["definitions"],
      OperationDefinition: [
        "name",
        "variableDefinitions",
        "directives",
        "selectionSet"
      ],
      VariableDefinition: ["variable", "type", "defaultValue", "directives"],
      Variable: ["name"],
      SelectionSet: ["selections"],
      Field: ["alias", "name", "arguments", "directives", "selectionSet"],
      Argument: ["name", "value"],
      FragmentSpread: ["name", "directives"],
      InlineFragment: ["typeCondition", "directives", "selectionSet"],
      FragmentDefinition: [
        "name",
        // Note: fragment variable definitions are deprecated and will removed in v17.0.0
        "variableDefinitions",
        "typeCondition",
        "directives",
        "selectionSet"
      ],
      IntValue: [],
      FloatValue: [],
      StringValue: [],
      BooleanValue: [],
      NullValue: [],
      EnumValue: [],
      ListValue: ["values"],
      ObjectValue: ["fields"],
      ObjectField: ["name", "value"],
      Directive: ["name", "arguments"],
      NamedType: ["name"],
      ListType: ["type"],
      NonNullType: ["type"],
      SchemaDefinition: ["description", "directives", "operationTypes"],
      OperationTypeDefinition: ["type"],
      ScalarTypeDefinition: ["description", "name", "directives"],
      ObjectTypeDefinition: [
        "description",
        "name",
        "interfaces",
        "directives",
        "fields"
      ],
      FieldDefinition: ["description", "name", "arguments", "type", "directives"],
      InputValueDefinition: [
        "description",
        "name",
        "type",
        "defaultValue",
        "directives"
      ],
      InterfaceTypeDefinition: [
        "description",
        "name",
        "interfaces",
        "directives",
        "fields"
      ],
      UnionTypeDefinition: ["description", "name", "directives", "types"],
      EnumTypeDefinition: ["description", "name", "directives", "values"],
      EnumValueDefinition: ["description", "name", "directives"],
      InputObjectTypeDefinition: ["description", "name", "directives", "fields"],
      DirectiveDefinition: ["description", "name", "arguments", "locations"],
      SchemaExtension: ["directives", "operationTypes"],
      ScalarTypeExtension: ["name", "directives"],
      ObjectTypeExtension: ["name", "interfaces", "directives", "fields"],
      InterfaceTypeExtension: ["name", "interfaces", "directives", "fields"],
      UnionTypeExtension: ["name", "directives", "types"],
      EnumTypeExtension: ["name", "directives", "values"],
      InputObjectTypeExtension: ["name", "directives", "fields"]
    };
    exports2.QueryDocumentKeys = QueryDocumentKeys;
    var kindValues = new Set(Object.keys(QueryDocumentKeys));
    function isNode(maybeNode) {
      const maybeKind = maybeNode === null || maybeNode === void 0 ? void 0 : maybeNode.kind;
      return typeof maybeKind === "string" && kindValues.has(maybeKind);
    }
    var OperationTypeNode;
    exports2.OperationTypeNode = OperationTypeNode;
    (function(OperationTypeNode2) {
      OperationTypeNode2["QUERY"] = "query";
      OperationTypeNode2["MUTATION"] = "mutation";
      OperationTypeNode2["SUBSCRIPTION"] = "subscription";
    })(OperationTypeNode || (exports2.OperationTypeNode = OperationTypeNode = {}));
  }
});

// node_modules/graphql/language/directiveLocation.js
var require_directiveLocation = __commonJS({
  "node_modules/graphql/language/directiveLocation.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.DirectiveLocation = void 0;
    var DirectiveLocation;
    exports2.DirectiveLocation = DirectiveLocation;
    (function(DirectiveLocation2) {
      DirectiveLocation2["QUERY"] = "QUERY";
      DirectiveLocation2["MUTATION"] = "MUTATION";
      DirectiveLocation2["SUBSCRIPTION"] = "SUBSCRIPTION";
      DirectiveLocation2["FIELD"] = "FIELD";
      DirectiveLocation2["FRAGMENT_DEFINITION"] = "FRAGMENT_DEFINITION";
      DirectiveLocation2["FRAGMENT_SPREAD"] = "FRAGMENT_SPREAD";
      DirectiveLocation2["INLINE_FRAGMENT"] = "INLINE_FRAGMENT";
      DirectiveLocation2["VARIABLE_DEFINITION"] = "VARIABLE_DEFINITION";
      DirectiveLocation2["SCHEMA"] = "SCHEMA";
      DirectiveLocation2["SCALAR"] = "SCALAR";
      DirectiveLocation2["OBJECT"] = "OBJECT";
      DirectiveLocation2["FIELD_DEFINITION"] = "FIELD_DEFINITION";
      DirectiveLocation2["ARGUMENT_DEFINITION"] = "ARGUMENT_DEFINITION";
      DirectiveLocation2["INTERFACE"] = "INTERFACE";
      DirectiveLocation2["UNION"] = "UNION";
      DirectiveLocation2["ENUM"] = "ENUM";
      DirectiveLocation2["ENUM_VALUE"] = "ENUM_VALUE";
      DirectiveLocation2["INPUT_OBJECT"] = "INPUT_OBJECT";
      DirectiveLocation2["INPUT_FIELD_DEFINITION"] = "INPUT_FIELD_DEFINITION";
    })(DirectiveLocation || (exports2.DirectiveLocation = DirectiveLocation = {}));
  }
});

// node_modules/graphql/language/kinds.js
var require_kinds = __commonJS({
  "node_modules/graphql/language/kinds.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.Kind = void 0;
    var Kind;
    exports2.Kind = Kind;
    (function(Kind2) {
      Kind2["NAME"] = "Name";
      Kind2["DOCUMENT"] = "Document";
      Kind2["OPERATION_DEFINITION"] = "OperationDefinition";
      Kind2["VARIABLE_DEFINITION"] = "VariableDefinition";
      Kind2["SELECTION_SET"] = "SelectionSet";
      Kind2["FIELD"] = "Field";
      Kind2["ARGUMENT"] = "Argument";
      Kind2["FRAGMENT_SPREAD"] = "FragmentSpread";
      Kind2["INLINE_FRAGMENT"] = "InlineFragment";
      Kind2["FRAGMENT_DEFINITION"] = "FragmentDefinition";
      Kind2["VARIABLE"] = "Variable";
      Kind2["INT"] = "IntValue";
      Kind2["FLOAT"] = "FloatValue";
      Kind2["STRING"] = "StringValue";
      Kind2["BOOLEAN"] = "BooleanValue";
      Kind2["NULL"] = "NullValue";
      Kind2["ENUM"] = "EnumValue";
      Kind2["LIST"] = "ListValue";
      Kind2["OBJECT"] = "ObjectValue";
      Kind2["OBJECT_FIELD"] = "ObjectField";
      Kind2["DIRECTIVE"] = "Directive";
      Kind2["NAMED_TYPE"] = "NamedType";
      Kind2["LIST_TYPE"] = "ListType";
      Kind2["NON_NULL_TYPE"] = "NonNullType";
      Kind2["SCHEMA_DEFINITION"] = "SchemaDefinition";
      Kind2["OPERATION_TYPE_DEFINITION"] = "OperationTypeDefinition";
      Kind2["SCALAR_TYPE_DEFINITION"] = "ScalarTypeDefinition";
      Kind2["OBJECT_TYPE_DEFINITION"] = "ObjectTypeDefinition";
      Kind2["FIELD_DEFINITION"] = "FieldDefinition";
      Kind2["INPUT_VALUE_DEFINITION"] = "InputValueDefinition";
      Kind2["INTERFACE_TYPE_DEFINITION"] = "InterfaceTypeDefinition";
      Kind2["UNION_TYPE_DEFINITION"] = "UnionTypeDefinition";
      Kind2["ENUM_TYPE_DEFINITION"] = "EnumTypeDefinition";
      Kind2["ENUM_VALUE_DEFINITION"] = "EnumValueDefinition";
      Kind2["INPUT_OBJECT_TYPE_DEFINITION"] = "InputObjectTypeDefinition";
      Kind2["DIRECTIVE_DEFINITION"] = "DirectiveDefinition";
      Kind2["SCHEMA_EXTENSION"] = "SchemaExtension";
      Kind2["SCALAR_TYPE_EXTENSION"] = "ScalarTypeExtension";
      Kind2["OBJECT_TYPE_EXTENSION"] = "ObjectTypeExtension";
      Kind2["INTERFACE_TYPE_EXTENSION"] = "InterfaceTypeExtension";
      Kind2["UNION_TYPE_EXTENSION"] = "UnionTypeExtension";
      Kind2["ENUM_TYPE_EXTENSION"] = "EnumTypeExtension";
      Kind2["INPUT_OBJECT_TYPE_EXTENSION"] = "InputObjectTypeExtension";
    })(Kind || (exports2.Kind = Kind = {}));
  }
});

// node_modules/graphql/language/characterClasses.js
var require_characterClasses = __commonJS({
  "node_modules/graphql/language/characterClasses.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isDigit = isDigit;
    exports2.isLetter = isLetter;
    exports2.isNameContinue = isNameContinue;
    exports2.isNameStart = isNameStart;
    exports2.isWhiteSpace = isWhiteSpace;
    function isWhiteSpace(code) {
      return code === 9 || code === 32;
    }
    function isDigit(code) {
      return code >= 48 && code <= 57;
    }
    function isLetter(code) {
      return code >= 97 && code <= 122 || // A-Z
      code >= 65 && code <= 90;
    }
    function isNameStart(code) {
      return isLetter(code) || code === 95;
    }
    function isNameContinue(code) {
      return isLetter(code) || isDigit(code) || code === 95;
    }
  }
});

// node_modules/graphql/language/blockString.js
var require_blockString = __commonJS({
  "node_modules/graphql/language/blockString.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.dedentBlockStringLines = dedentBlockStringLines;
    exports2.isPrintableAsBlockString = isPrintableAsBlockString;
    exports2.printBlockString = printBlockString;
    var _characterClasses = require_characterClasses();
    function dedentBlockStringLines(lines) {
      var _firstNonEmptyLine2;
      let commonIndent = Number.MAX_SAFE_INTEGER;
      let firstNonEmptyLine = null;
      let lastNonEmptyLine = -1;
      for (let i = 0; i < lines.length; ++i) {
        var _firstNonEmptyLine;
        const line = lines[i];
        const indent = leadingWhitespace(line);
        if (indent === line.length) {
          continue;
        }
        firstNonEmptyLine = (_firstNonEmptyLine = firstNonEmptyLine) !== null && _firstNonEmptyLine !== void 0 ? _firstNonEmptyLine : i;
        lastNonEmptyLine = i;
        if (i !== 0 && indent < commonIndent) {
          commonIndent = indent;
        }
      }
      return lines.map((line, i) => i === 0 ? line : line.slice(commonIndent)).slice(
        (_firstNonEmptyLine2 = firstNonEmptyLine) !== null && _firstNonEmptyLine2 !== void 0 ? _firstNonEmptyLine2 : 0,
        lastNonEmptyLine + 1
      );
    }
    function leadingWhitespace(str) {
      let i = 0;
      while (i < str.length && (0, _characterClasses.isWhiteSpace)(str.charCodeAt(i))) {
        ++i;
      }
      return i;
    }
    function isPrintableAsBlockString(value) {
      if (value === "") {
        return true;
      }
      let isEmptyLine = true;
      let hasIndent = false;
      let hasCommonIndent = true;
      let seenNonEmptyLine = false;
      for (let i = 0; i < value.length; ++i) {
        switch (value.codePointAt(i)) {
          case 0:
          case 1:
          case 2:
          case 3:
          case 4:
          case 5:
          case 6:
          case 7:
          case 8:
          case 11:
          case 12:
          case 14:
          case 15:
            return false;
          case 13:
            return false;
          case 10:
            if (isEmptyLine && !seenNonEmptyLine) {
              return false;
            }
            seenNonEmptyLine = true;
            isEmptyLine = true;
            hasIndent = false;
            break;
          case 9:
          case 32:
            hasIndent || (hasIndent = isEmptyLine);
            break;
          default:
            hasCommonIndent && (hasCommonIndent = hasIndent);
            isEmptyLine = false;
        }
      }
      if (isEmptyLine) {
        return false;
      }
      if (hasCommonIndent && seenNonEmptyLine) {
        return false;
      }
      return true;
    }
    function printBlockString(value, options) {
      const escapedValue = value.replace(/"""/g, '\\"""');
      const lines = escapedValue.split(/\r\n|[\n\r]/g);
      const isSingleLine = lines.length === 1;
      const forceLeadingNewLine = lines.length > 1 && lines.slice(1).every(
        (line) => line.length === 0 || (0, _characterClasses.isWhiteSpace)(line.charCodeAt(0))
      );
      const hasTrailingTripleQuotes = escapedValue.endsWith('\\"""');
      const hasTrailingQuote = value.endsWith('"') && !hasTrailingTripleQuotes;
      const hasTrailingSlash = value.endsWith("\\");
      const forceTrailingNewline = hasTrailingQuote || hasTrailingSlash;
      const printAsMultipleLines = !(options !== null && options !== void 0 && options.minimize) && // add leading and trailing new lines only if it improves readability
      (!isSingleLine || value.length > 70 || forceTrailingNewline || forceLeadingNewLine || hasTrailingTripleQuotes);
      let result = "";
      const skipLeadingNewLine = isSingleLine && (0, _characterClasses.isWhiteSpace)(value.charCodeAt(0));
      if (printAsMultipleLines && !skipLeadingNewLine || forceLeadingNewLine) {
        result += "\n";
      }
      result += escapedValue;
      if (printAsMultipleLines || forceTrailingNewline) {
        result += "\n";
      }
      return '"""' + result + '"""';
    }
  }
});

// node_modules/graphql/language/tokenKind.js
var require_tokenKind = __commonJS({
  "node_modules/graphql/language/tokenKind.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.TokenKind = void 0;
    var TokenKind;
    exports2.TokenKind = TokenKind;
    (function(TokenKind2) {
      TokenKind2["SOF"] = "<SOF>";
      TokenKind2["EOF"] = "<EOF>";
      TokenKind2["BANG"] = "!";
      TokenKind2["DOLLAR"] = "$";
      TokenKind2["AMP"] = "&";
      TokenKind2["PAREN_L"] = "(";
      TokenKind2["PAREN_R"] = ")";
      TokenKind2["SPREAD"] = "...";
      TokenKind2["COLON"] = ":";
      TokenKind2["EQUALS"] = "=";
      TokenKind2["AT"] = "@";
      TokenKind2["BRACKET_L"] = "[";
      TokenKind2["BRACKET_R"] = "]";
      TokenKind2["BRACE_L"] = "{";
      TokenKind2["PIPE"] = "|";
      TokenKind2["BRACE_R"] = "}";
      TokenKind2["NAME"] = "Name";
      TokenKind2["INT"] = "Int";
      TokenKind2["FLOAT"] = "Float";
      TokenKind2["STRING"] = "String";
      TokenKind2["BLOCK_STRING"] = "BlockString";
      TokenKind2["COMMENT"] = "Comment";
    })(TokenKind || (exports2.TokenKind = TokenKind = {}));
  }
});

// node_modules/graphql/language/lexer.js
var require_lexer = __commonJS({
  "node_modules/graphql/language/lexer.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.Lexer = void 0;
    exports2.isPunctuatorTokenKind = isPunctuatorTokenKind;
    var _syntaxError = require_syntaxError();
    var _ast = require_ast();
    var _blockString = require_blockString();
    var _characterClasses = require_characterClasses();
    var _tokenKind = require_tokenKind();
    var Lexer = class {
      /**
       * The previously focused non-ignored token.
       */
      /**
       * The currently focused non-ignored token.
       */
      /**
       * The (1-indexed) line containing the current token.
       */
      /**
       * The character offset at which the current line begins.
       */
      constructor(source) {
        const startOfFileToken = new _ast.Token(
          _tokenKind.TokenKind.SOF,
          0,
          0,
          0,
          0
        );
        this.source = source;
        this.lastToken = startOfFileToken;
        this.token = startOfFileToken;
        this.line = 1;
        this.lineStart = 0;
      }
      get [Symbol.toStringTag]() {
        return "Lexer";
      }
      /**
       * Advances the token stream to the next non-ignored token.
       */
      advance() {
        this.lastToken = this.token;
        const token = this.token = this.lookahead();
        return token;
      }
      /**
       * Looks ahead and returns the next non-ignored token, but does not change
       * the state of Lexer.
       */
      lookahead() {
        let token = this.token;
        if (token.kind !== _tokenKind.TokenKind.EOF) {
          do {
            if (token.next) {
              token = token.next;
            } else {
              const nextToken = readNextToken(this, token.end);
              token.next = nextToken;
              nextToken.prev = token;
              token = nextToken;
            }
          } while (token.kind === _tokenKind.TokenKind.COMMENT);
        }
        return token;
      }
    };
    exports2.Lexer = Lexer;
    function isPunctuatorTokenKind(kind) {
      return kind === _tokenKind.TokenKind.BANG || kind === _tokenKind.TokenKind.DOLLAR || kind === _tokenKind.TokenKind.AMP || kind === _tokenKind.TokenKind.PAREN_L || kind === _tokenKind.TokenKind.PAREN_R || kind === _tokenKind.TokenKind.SPREAD || kind === _tokenKind.TokenKind.COLON || kind === _tokenKind.TokenKind.EQUALS || kind === _tokenKind.TokenKind.AT || kind === _tokenKind.TokenKind.BRACKET_L || kind === _tokenKind.TokenKind.BRACKET_R || kind === _tokenKind.TokenKind.BRACE_L || kind === _tokenKind.TokenKind.PIPE || kind === _tokenKind.TokenKind.BRACE_R;
    }
    function isUnicodeScalarValue(code) {
      return code >= 0 && code <= 55295 || code >= 57344 && code <= 1114111;
    }
    function isSupplementaryCodePoint(body, location) {
      return isLeadingSurrogate(body.charCodeAt(location)) && isTrailingSurrogate(body.charCodeAt(location + 1));
    }
    function isLeadingSurrogate(code) {
      return code >= 55296 && code <= 56319;
    }
    function isTrailingSurrogate(code) {
      return code >= 56320 && code <= 57343;
    }
    function printCodePointAt(lexer, location) {
      const code = lexer.source.body.codePointAt(location);
      if (code === void 0) {
        return _tokenKind.TokenKind.EOF;
      } else if (code >= 32 && code <= 126) {
        const char = String.fromCodePoint(code);
        return char === '"' ? `'"'` : `"${char}"`;
      }
      return "U+" + code.toString(16).toUpperCase().padStart(4, "0");
    }
    function createToken(lexer, kind, start, end, value) {
      const line = lexer.line;
      const col = 1 + start - lexer.lineStart;
      return new _ast.Token(kind, start, end, line, col, value);
    }
    function readNextToken(lexer, start) {
      const body = lexer.source.body;
      const bodyLength = body.length;
      let position = start;
      while (position < bodyLength) {
        const code = body.charCodeAt(position);
        switch (code) {
          case 65279:
          case 9:
          case 32:
          case 44:
            ++position;
            continue;
          case 10:
            ++position;
            ++lexer.line;
            lexer.lineStart = position;
            continue;
          case 13:
            if (body.charCodeAt(position + 1) === 10) {
              position += 2;
            } else {
              ++position;
            }
            ++lexer.line;
            lexer.lineStart = position;
            continue;
          case 35:
            return readComment(lexer, position);
          case 33:
            return createToken(
              lexer,
              _tokenKind.TokenKind.BANG,
              position,
              position + 1
            );
          case 36:
            return createToken(
              lexer,
              _tokenKind.TokenKind.DOLLAR,
              position,
              position + 1
            );
          case 38:
            return createToken(
              lexer,
              _tokenKind.TokenKind.AMP,
              position,
              position + 1
            );
          case 40:
            return createToken(
              lexer,
              _tokenKind.TokenKind.PAREN_L,
              position,
              position + 1
            );
          case 41:
            return createToken(
              lexer,
              _tokenKind.TokenKind.PAREN_R,
              position,
              position + 1
            );
          case 46:
            if (body.charCodeAt(position + 1) === 46 && body.charCodeAt(position + 2) === 46) {
              return createToken(
                lexer,
                _tokenKind.TokenKind.SPREAD,
                position,
                position + 3
              );
            }
            break;
          case 58:
            return createToken(
              lexer,
              _tokenKind.TokenKind.COLON,
              position,
              position + 1
            );
          case 61:
            return createToken(
              lexer,
              _tokenKind.TokenKind.EQUALS,
              position,
              position + 1
            );
          case 64:
            return createToken(
              lexer,
              _tokenKind.TokenKind.AT,
              position,
              position + 1
            );
          case 91:
            return createToken(
              lexer,
              _tokenKind.TokenKind.BRACKET_L,
              position,
              position + 1
            );
          case 93:
            return createToken(
              lexer,
              _tokenKind.TokenKind.BRACKET_R,
              position,
              position + 1
            );
          case 123:
            return createToken(
              lexer,
              _tokenKind.TokenKind.BRACE_L,
              position,
              position + 1
            );
          case 124:
            return createToken(
              lexer,
              _tokenKind.TokenKind.PIPE,
              position,
              position + 1
            );
          case 125:
            return createToken(
              lexer,
              _tokenKind.TokenKind.BRACE_R,
              position,
              position + 1
            );
          case 34:
            if (body.charCodeAt(position + 1) === 34 && body.charCodeAt(position + 2) === 34) {
              return readBlockString(lexer, position);
            }
            return readString(lexer, position);
        }
        if ((0, _characterClasses.isDigit)(code) || code === 45) {
          return readNumber(lexer, position, code);
        }
        if ((0, _characterClasses.isNameStart)(code)) {
          return readName(lexer, position);
        }
        throw (0, _syntaxError.syntaxError)(
          lexer.source,
          position,
          code === 39 ? `Unexpected single quote character ('), did you mean to use a double quote (")?` : isUnicodeScalarValue(code) || isSupplementaryCodePoint(body, position) ? `Unexpected character: ${printCodePointAt(lexer, position)}.` : `Invalid character: ${printCodePointAt(lexer, position)}.`
        );
      }
      return createToken(lexer, _tokenKind.TokenKind.EOF, bodyLength, bodyLength);
    }
    function readComment(lexer, start) {
      const body = lexer.source.body;
      const bodyLength = body.length;
      let position = start + 1;
      while (position < bodyLength) {
        const code = body.charCodeAt(position);
        if (code === 10 || code === 13) {
          break;
        }
        if (isUnicodeScalarValue(code)) {
          ++position;
        } else if (isSupplementaryCodePoint(body, position)) {
          position += 2;
        } else {
          break;
        }
      }
      return createToken(
        lexer,
        _tokenKind.TokenKind.COMMENT,
        start,
        position,
        body.slice(start + 1, position)
      );
    }
    function readNumber(lexer, start, firstCode) {
      const body = lexer.source.body;
      let position = start;
      let code = firstCode;
      let isFloat = false;
      if (code === 45) {
        code = body.charCodeAt(++position);
      }
      if (code === 48) {
        code = body.charCodeAt(++position);
        if ((0, _characterClasses.isDigit)(code)) {
          throw (0, _syntaxError.syntaxError)(
            lexer.source,
            position,
            `Invalid number, unexpected digit after 0: ${printCodePointAt(
              lexer,
              position
            )}.`
          );
        }
      } else {
        position = readDigits(lexer, position, code);
        code = body.charCodeAt(position);
      }
      if (code === 46) {
        isFloat = true;
        code = body.charCodeAt(++position);
        position = readDigits(lexer, position, code);
        code = body.charCodeAt(position);
      }
      if (code === 69 || code === 101) {
        isFloat = true;
        code = body.charCodeAt(++position);
        if (code === 43 || code === 45) {
          code = body.charCodeAt(++position);
        }
        position = readDigits(lexer, position, code);
        code = body.charCodeAt(position);
      }
      if (code === 46 || (0, _characterClasses.isNameStart)(code)) {
        throw (0, _syntaxError.syntaxError)(
          lexer.source,
          position,
          `Invalid number, expected digit but got: ${printCodePointAt(
            lexer,
            position
          )}.`
        );
      }
      return createToken(
        lexer,
        isFloat ? _tokenKind.TokenKind.FLOAT : _tokenKind.TokenKind.INT,
        start,
        position,
        body.slice(start, position)
      );
    }
    function readDigits(lexer, start, firstCode) {
      if (!(0, _characterClasses.isDigit)(firstCode)) {
        throw (0, _syntaxError.syntaxError)(
          lexer.source,
          start,
          `Invalid number, expected digit but got: ${printCodePointAt(
            lexer,
            start
          )}.`
        );
      }
      const body = lexer.source.body;
      let position = start + 1;
      while ((0, _characterClasses.isDigit)(body.charCodeAt(position))) {
        ++position;
      }
      return position;
    }
    function readString(lexer, start) {
      const body = lexer.source.body;
      const bodyLength = body.length;
      let position = start + 1;
      let chunkStart = position;
      let value = "";
      while (position < bodyLength) {
        const code = body.charCodeAt(position);
        if (code === 34) {
          value += body.slice(chunkStart, position);
          return createToken(
            lexer,
            _tokenKind.TokenKind.STRING,
            start,
            position + 1,
            value
          );
        }
        if (code === 92) {
          value += body.slice(chunkStart, position);
          const escape = body.charCodeAt(position + 1) === 117 ? body.charCodeAt(position + 2) === 123 ? readEscapedUnicodeVariableWidth(lexer, position) : readEscapedUnicodeFixedWidth(lexer, position) : readEscapedCharacter(lexer, position);
          value += escape.value;
          position += escape.size;
          chunkStart = position;
          continue;
        }
        if (code === 10 || code === 13) {
          break;
        }
        if (isUnicodeScalarValue(code)) {
          ++position;
        } else if (isSupplementaryCodePoint(body, position)) {
          position += 2;
        } else {
          throw (0, _syntaxError.syntaxError)(
            lexer.source,
            position,
            `Invalid character within String: ${printCodePointAt(
              lexer,
              position
            )}.`
          );
        }
      }
      throw (0, _syntaxError.syntaxError)(
        lexer.source,
        position,
        "Unterminated string."
      );
    }
    function readEscapedUnicodeVariableWidth(lexer, position) {
      const body = lexer.source.body;
      let point = 0;
      let size = 3;
      while (size < 12) {
        const code = body.charCodeAt(position + size++);
        if (code === 125) {
          if (size < 5 || !isUnicodeScalarValue(point)) {
            break;
          }
          return {
            value: String.fromCodePoint(point),
            size
          };
        }
        point = point << 4 | readHexDigit(code);
        if (point < 0) {
          break;
        }
      }
      throw (0, _syntaxError.syntaxError)(
        lexer.source,
        position,
        `Invalid Unicode escape sequence: "${body.slice(
          position,
          position + size
        )}".`
      );
    }
    function readEscapedUnicodeFixedWidth(lexer, position) {
      const body = lexer.source.body;
      const code = read16BitHexCode(body, position + 2);
      if (isUnicodeScalarValue(code)) {
        return {
          value: String.fromCodePoint(code),
          size: 6
        };
      }
      if (isLeadingSurrogate(code)) {
        if (body.charCodeAt(position + 6) === 92 && body.charCodeAt(position + 7) === 117) {
          const trailingCode = read16BitHexCode(body, position + 8);
          if (isTrailingSurrogate(trailingCode)) {
            return {
              value: String.fromCodePoint(code, trailingCode),
              size: 12
            };
          }
        }
      }
      throw (0, _syntaxError.syntaxError)(
        lexer.source,
        position,
        `Invalid Unicode escape sequence: "${body.slice(position, position + 6)}".`
      );
    }
    function read16BitHexCode(body, position) {
      return readHexDigit(body.charCodeAt(position)) << 12 | readHexDigit(body.charCodeAt(position + 1)) << 8 | readHexDigit(body.charCodeAt(position + 2)) << 4 | readHexDigit(body.charCodeAt(position + 3));
    }
    function readHexDigit(code) {
      return code >= 48 && code <= 57 ? code - 48 : code >= 65 && code <= 70 ? code - 55 : code >= 97 && code <= 102 ? code - 87 : -1;
    }
    function readEscapedCharacter(lexer, position) {
      const body = lexer.source.body;
      const code = body.charCodeAt(position + 1);
      switch (code) {
        case 34:
          return {
            value: '"',
            size: 2
          };
        case 92:
          return {
            value: "\\",
            size: 2
          };
        case 47:
          return {
            value: "/",
            size: 2
          };
        case 98:
          return {
            value: "\b",
            size: 2
          };
        case 102:
          return {
            value: "\f",
            size: 2
          };
        case 110:
          return {
            value: "\n",
            size: 2
          };
        case 114:
          return {
            value: "\r",
            size: 2
          };
        case 116:
          return {
            value: "	",
            size: 2
          };
      }
      throw (0, _syntaxError.syntaxError)(
        lexer.source,
        position,
        `Invalid character escape sequence: "${body.slice(
          position,
          position + 2
        )}".`
      );
    }
    function readBlockString(lexer, start) {
      const body = lexer.source.body;
      const bodyLength = body.length;
      let lineStart = lexer.lineStart;
      let position = start + 3;
      let chunkStart = position;
      let currentLine = "";
      const blockLines = [];
      while (position < bodyLength) {
        const code = body.charCodeAt(position);
        if (code === 34 && body.charCodeAt(position + 1) === 34 && body.charCodeAt(position + 2) === 34) {
          currentLine += body.slice(chunkStart, position);
          blockLines.push(currentLine);
          const token = createToken(
            lexer,
            _tokenKind.TokenKind.BLOCK_STRING,
            start,
            position + 3,
            // Return a string of the lines joined with U+000A.
            (0, _blockString.dedentBlockStringLines)(blockLines).join("\n")
          );
          lexer.line += blockLines.length - 1;
          lexer.lineStart = lineStart;
          return token;
        }
        if (code === 92 && body.charCodeAt(position + 1) === 34 && body.charCodeAt(position + 2) === 34 && body.charCodeAt(position + 3) === 34) {
          currentLine += body.slice(chunkStart, position);
          chunkStart = position + 1;
          position += 4;
          continue;
        }
        if (code === 10 || code === 13) {
          currentLine += body.slice(chunkStart, position);
          blockLines.push(currentLine);
          if (code === 13 && body.charCodeAt(position + 1) === 10) {
            position += 2;
          } else {
            ++position;
          }
          currentLine = "";
          chunkStart = position;
          lineStart = position;
          continue;
        }
        if (isUnicodeScalarValue(code)) {
          ++position;
        } else if (isSupplementaryCodePoint(body, position)) {
          position += 2;
        } else {
          throw (0, _syntaxError.syntaxError)(
            lexer.source,
            position,
            `Invalid character within String: ${printCodePointAt(
              lexer,
              position
            )}.`
          );
        }
      }
      throw (0, _syntaxError.syntaxError)(
        lexer.source,
        position,
        "Unterminated string."
      );
    }
    function readName(lexer, start) {
      const body = lexer.source.body;
      const bodyLength = body.length;
      let position = start + 1;
      while (position < bodyLength) {
        const code = body.charCodeAt(position);
        if ((0, _characterClasses.isNameContinue)(code)) {
          ++position;
        } else {
          break;
        }
      }
      return createToken(
        lexer,
        _tokenKind.TokenKind.NAME,
        start,
        position,
        body.slice(start, position)
      );
    }
  }
});

// node_modules/graphql/jsutils/inspect.js
var require_inspect = __commonJS({
  "node_modules/graphql/jsutils/inspect.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.inspect = inspect;
    var MAX_ARRAY_LENGTH = 10;
    var MAX_RECURSIVE_DEPTH = 2;
    function inspect(value) {
      return formatValue(value, []);
    }
    function formatValue(value, seenValues) {
      switch (typeof value) {
        case "string":
          return JSON.stringify(value);
        case "function":
          return value.name ? `[function ${value.name}]` : "[function]";
        case "object":
          return formatObjectValue(value, seenValues);
        default:
          return String(value);
      }
    }
    function formatObjectValue(value, previouslySeenValues) {
      if (value === null) {
        return "null";
      }
      if (previouslySeenValues.includes(value)) {
        return "[Circular]";
      }
      const seenValues = [...previouslySeenValues, value];
      if (isJSONable(value)) {
        const jsonValue = value.toJSON();
        if (jsonValue !== value) {
          return typeof jsonValue === "string" ? jsonValue : formatValue(jsonValue, seenValues);
        }
      } else if (Array.isArray(value)) {
        return formatArray(value, seenValues);
      }
      return formatObject(value, seenValues);
    }
    function isJSONable(value) {
      return typeof value.toJSON === "function";
    }
    function formatObject(object, seenValues) {
      const entries = Object.entries(object);
      if (entries.length === 0) {
        return "{}";
      }
      if (seenValues.length > MAX_RECURSIVE_DEPTH) {
        return "[" + getObjectTag(object) + "]";
      }
      const properties = entries.map(
        ([key, value]) => key + ": " + formatValue(value, seenValues)
      );
      return "{ " + properties.join(", ") + " }";
    }
    function formatArray(array, seenValues) {
      if (array.length === 0) {
        return "[]";
      }
      if (seenValues.length > MAX_RECURSIVE_DEPTH) {
        return "[Array]";
      }
      const len = Math.min(MAX_ARRAY_LENGTH, array.length);
      const remaining = array.length - len;
      const items = [];
      for (let i = 0; i < len; ++i) {
        items.push(formatValue(array[i], seenValues));
      }
      if (remaining === 1) {
        items.push("... 1 more item");
      } else if (remaining > 1) {
        items.push(`... ${remaining} more items`);
      }
      return "[" + items.join(", ") + "]";
    }
    function getObjectTag(object) {
      const tag = Object.prototype.toString.call(object).replace(/^\[object /, "").replace(/]$/, "");
      if (tag === "Object" && typeof object.constructor === "function") {
        const name = object.constructor.name;
        if (typeof name === "string" && name !== "") {
          return name;
        }
      }
      return tag;
    }
  }
});

// node_modules/graphql/jsutils/instanceOf.js
var require_instanceOf = __commonJS({
  "node_modules/graphql/jsutils/instanceOf.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.instanceOf = void 0;
    var _inspect = require_inspect();
    var instanceOf = (
      /* c8 ignore next 6 */
      // FIXME: https://github.com/graphql/graphql-js/issues/2317
      globalThis.process && globalThis.process.env.NODE_ENV === "production" ? function instanceOf2(value, constructor) {
        return value instanceof constructor;
      } : function instanceOf2(value, constructor) {
        if (value instanceof constructor) {
          return true;
        }
        if (typeof value === "object" && value !== null) {
          var _value$constructor;
          const className = constructor.prototype[Symbol.toStringTag];
          const valueClassName = (
            // We still need to support constructor's name to detect conflicts with older versions of this library.
            Symbol.toStringTag in value ? value[Symbol.toStringTag] : (_value$constructor = value.constructor) === null || _value$constructor === void 0 ? void 0 : _value$constructor.name
          );
          if (className === valueClassName) {
            const stringifiedValue = (0, _inspect.inspect)(value);
            throw new Error(`Cannot use ${className} "${stringifiedValue}" from another module or realm.

Ensure that there is only one instance of "graphql" in the node_modules
directory. If different versions of "graphql" are the dependencies of other
relied on modules, use "resolutions" to ensure only one version is installed.

https://yarnpkg.com/en/docs/selective-version-resolutions

Duplicate "graphql" modules cannot be used at the same time since different
versions may have different capabilities and behavior. The data from one
version used in the function from another could produce confusing and
spurious results.`);
          }
        }
        return false;
      }
    );
    exports2.instanceOf = instanceOf;
  }
});

// node_modules/graphql/language/source.js
var require_source = __commonJS({
  "node_modules/graphql/language/source.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.Source = void 0;
    exports2.isSource = isSource;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _instanceOf = require_instanceOf();
    var Source = class {
      constructor(body, name = "GraphQL request", locationOffset = {
        line: 1,
        column: 1
      }) {
        typeof body === "string" || (0, _devAssert.devAssert)(
          false,
          `Body must be a string. Received: ${(0, _inspect.inspect)(body)}.`
        );
        this.body = body;
        this.name = name;
        this.locationOffset = locationOffset;
        this.locationOffset.line > 0 || (0, _devAssert.devAssert)(
          false,
          "line in locationOffset is 1-indexed and must be positive."
        );
        this.locationOffset.column > 0 || (0, _devAssert.devAssert)(
          false,
          "column in locationOffset is 1-indexed and must be positive."
        );
      }
      get [Symbol.toStringTag]() {
        return "Source";
      }
    };
    exports2.Source = Source;
    function isSource(source) {
      return (0, _instanceOf.instanceOf)(source, Source);
    }
  }
});

// node_modules/graphql/language/parser.js
var require_parser = __commonJS({
  "node_modules/graphql/language/parser.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.Parser = void 0;
    exports2.parse = parse2;
    exports2.parseConstValue = parseConstValue;
    exports2.parseType = parseType;
    exports2.parseValue = parseValue;
    var _syntaxError = require_syntaxError();
    var _ast = require_ast();
    var _directiveLocation = require_directiveLocation();
    var _kinds = require_kinds();
    var _lexer = require_lexer();
    var _source = require_source();
    var _tokenKind = require_tokenKind();
    function parse2(source, options) {
      const parser = new Parser(source, options);
      return parser.parseDocument();
    }
    function parseValue(source, options) {
      const parser = new Parser(source, options);
      parser.expectToken(_tokenKind.TokenKind.SOF);
      const value = parser.parseValueLiteral(false);
      parser.expectToken(_tokenKind.TokenKind.EOF);
      return value;
    }
    function parseConstValue(source, options) {
      const parser = new Parser(source, options);
      parser.expectToken(_tokenKind.TokenKind.SOF);
      const value = parser.parseConstValueLiteral();
      parser.expectToken(_tokenKind.TokenKind.EOF);
      return value;
    }
    function parseType(source, options) {
      const parser = new Parser(source, options);
      parser.expectToken(_tokenKind.TokenKind.SOF);
      const type = parser.parseTypeReference();
      parser.expectToken(_tokenKind.TokenKind.EOF);
      return type;
    }
    var Parser = class {
      constructor(source, options = {}) {
        const sourceObj = (0, _source.isSource)(source) ? source : new _source.Source(source);
        this._lexer = new _lexer.Lexer(sourceObj);
        this._options = options;
        this._tokenCounter = 0;
      }
      /**
       * Converts a name lex token into a name parse node.
       */
      parseName() {
        const token = this.expectToken(_tokenKind.TokenKind.NAME);
        return this.node(token, {
          kind: _kinds.Kind.NAME,
          value: token.value
        });
      }
      // Implements the parsing rules in the Document section.
      /**
       * Document : Definition+
       */
      parseDocument() {
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.DOCUMENT,
          definitions: this.many(
            _tokenKind.TokenKind.SOF,
            this.parseDefinition,
            _tokenKind.TokenKind.EOF
          )
        });
      }
      /**
       * Definition :
       *   - ExecutableDefinition
       *   - TypeSystemDefinition
       *   - TypeSystemExtension
       *
       * ExecutableDefinition :
       *   - OperationDefinition
       *   - FragmentDefinition
       *
       * TypeSystemDefinition :
       *   - SchemaDefinition
       *   - TypeDefinition
       *   - DirectiveDefinition
       *
       * TypeDefinition :
       *   - ScalarTypeDefinition
       *   - ObjectTypeDefinition
       *   - InterfaceTypeDefinition
       *   - UnionTypeDefinition
       *   - EnumTypeDefinition
       *   - InputObjectTypeDefinition
       */
      parseDefinition() {
        if (this.peek(_tokenKind.TokenKind.BRACE_L)) {
          return this.parseOperationDefinition();
        }
        const hasDescription = this.peekDescription();
        const keywordToken = hasDescription ? this._lexer.lookahead() : this._lexer.token;
        if (keywordToken.kind === _tokenKind.TokenKind.NAME) {
          switch (keywordToken.value) {
            case "schema":
              return this.parseSchemaDefinition();
            case "scalar":
              return this.parseScalarTypeDefinition();
            case "type":
              return this.parseObjectTypeDefinition();
            case "interface":
              return this.parseInterfaceTypeDefinition();
            case "union":
              return this.parseUnionTypeDefinition();
            case "enum":
              return this.parseEnumTypeDefinition();
            case "input":
              return this.parseInputObjectTypeDefinition();
            case "directive":
              return this.parseDirectiveDefinition();
          }
          if (hasDescription) {
            throw (0, _syntaxError.syntaxError)(
              this._lexer.source,
              this._lexer.token.start,
              "Unexpected description, descriptions are supported only on type definitions."
            );
          }
          switch (keywordToken.value) {
            case "query":
            case "mutation":
            case "subscription":
              return this.parseOperationDefinition();
            case "fragment":
              return this.parseFragmentDefinition();
            case "extend":
              return this.parseTypeSystemExtension();
          }
        }
        throw this.unexpected(keywordToken);
      }
      // Implements the parsing rules in the Operations section.
      /**
       * OperationDefinition :
       *  - SelectionSet
       *  - OperationType Name? VariableDefinitions? Directives? SelectionSet
       */
      parseOperationDefinition() {
        const start = this._lexer.token;
        if (this.peek(_tokenKind.TokenKind.BRACE_L)) {
          return this.node(start, {
            kind: _kinds.Kind.OPERATION_DEFINITION,
            operation: _ast.OperationTypeNode.QUERY,
            name: void 0,
            variableDefinitions: [],
            directives: [],
            selectionSet: this.parseSelectionSet()
          });
        }
        const operation = this.parseOperationType();
        let name;
        if (this.peek(_tokenKind.TokenKind.NAME)) {
          name = this.parseName();
        }
        return this.node(start, {
          kind: _kinds.Kind.OPERATION_DEFINITION,
          operation,
          name,
          variableDefinitions: this.parseVariableDefinitions(),
          directives: this.parseDirectives(false),
          selectionSet: this.parseSelectionSet()
        });
      }
      /**
       * OperationType : one of query mutation subscription
       */
      parseOperationType() {
        const operationToken = this.expectToken(_tokenKind.TokenKind.NAME);
        switch (operationToken.value) {
          case "query":
            return _ast.OperationTypeNode.QUERY;
          case "mutation":
            return _ast.OperationTypeNode.MUTATION;
          case "subscription":
            return _ast.OperationTypeNode.SUBSCRIPTION;
        }
        throw this.unexpected(operationToken);
      }
      /**
       * VariableDefinitions : ( VariableDefinition+ )
       */
      parseVariableDefinitions() {
        return this.optionalMany(
          _tokenKind.TokenKind.PAREN_L,
          this.parseVariableDefinition,
          _tokenKind.TokenKind.PAREN_R
        );
      }
      /**
       * VariableDefinition : Variable : Type DefaultValue? Directives[Const]?
       */
      parseVariableDefinition() {
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.VARIABLE_DEFINITION,
          variable: this.parseVariable(),
          type: (this.expectToken(_tokenKind.TokenKind.COLON), this.parseTypeReference()),
          defaultValue: this.expectOptionalToken(_tokenKind.TokenKind.EQUALS) ? this.parseConstValueLiteral() : void 0,
          directives: this.parseConstDirectives()
        });
      }
      /**
       * Variable : $ Name
       */
      parseVariable() {
        const start = this._lexer.token;
        this.expectToken(_tokenKind.TokenKind.DOLLAR);
        return this.node(start, {
          kind: _kinds.Kind.VARIABLE,
          name: this.parseName()
        });
      }
      /**
       * ```
       * SelectionSet : { Selection+ }
       * ```
       */
      parseSelectionSet() {
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.SELECTION_SET,
          selections: this.many(
            _tokenKind.TokenKind.BRACE_L,
            this.parseSelection,
            _tokenKind.TokenKind.BRACE_R
          )
        });
      }
      /**
       * Selection :
       *   - Field
       *   - FragmentSpread
       *   - InlineFragment
       */
      parseSelection() {
        return this.peek(_tokenKind.TokenKind.SPREAD) ? this.parseFragment() : this.parseField();
      }
      /**
       * Field : Alias? Name Arguments? Directives? SelectionSet?
       *
       * Alias : Name :
       */
      parseField() {
        const start = this._lexer.token;
        const nameOrAlias = this.parseName();
        let alias;
        let name;
        if (this.expectOptionalToken(_tokenKind.TokenKind.COLON)) {
          alias = nameOrAlias;
          name = this.parseName();
        } else {
          name = nameOrAlias;
        }
        return this.node(start, {
          kind: _kinds.Kind.FIELD,
          alias,
          name,
          arguments: this.parseArguments(false),
          directives: this.parseDirectives(false),
          selectionSet: this.peek(_tokenKind.TokenKind.BRACE_L) ? this.parseSelectionSet() : void 0
        });
      }
      /**
       * Arguments[Const] : ( Argument[?Const]+ )
       */
      parseArguments(isConst) {
        const item = isConst ? this.parseConstArgument : this.parseArgument;
        return this.optionalMany(
          _tokenKind.TokenKind.PAREN_L,
          item,
          _tokenKind.TokenKind.PAREN_R
        );
      }
      /**
       * Argument[Const] : Name : Value[?Const]
       */
      parseArgument(isConst = false) {
        const start = this._lexer.token;
        const name = this.parseName();
        this.expectToken(_tokenKind.TokenKind.COLON);
        return this.node(start, {
          kind: _kinds.Kind.ARGUMENT,
          name,
          value: this.parseValueLiteral(isConst)
        });
      }
      parseConstArgument() {
        return this.parseArgument(true);
      }
      // Implements the parsing rules in the Fragments section.
      /**
       * Corresponds to both FragmentSpread and InlineFragment in the spec.
       *
       * FragmentSpread : ... FragmentName Directives?
       *
       * InlineFragment : ... TypeCondition? Directives? SelectionSet
       */
      parseFragment() {
        const start = this._lexer.token;
        this.expectToken(_tokenKind.TokenKind.SPREAD);
        const hasTypeCondition = this.expectOptionalKeyword("on");
        if (!hasTypeCondition && this.peek(_tokenKind.TokenKind.NAME)) {
          return this.node(start, {
            kind: _kinds.Kind.FRAGMENT_SPREAD,
            name: this.parseFragmentName(),
            directives: this.parseDirectives(false)
          });
        }
        return this.node(start, {
          kind: _kinds.Kind.INLINE_FRAGMENT,
          typeCondition: hasTypeCondition ? this.parseNamedType() : void 0,
          directives: this.parseDirectives(false),
          selectionSet: this.parseSelectionSet()
        });
      }
      /**
       * FragmentDefinition :
       *   - fragment FragmentName on TypeCondition Directives? SelectionSet
       *
       * TypeCondition : NamedType
       */
      parseFragmentDefinition() {
        const start = this._lexer.token;
        this.expectKeyword("fragment");
        if (this._options.allowLegacyFragmentVariables === true) {
          return this.node(start, {
            kind: _kinds.Kind.FRAGMENT_DEFINITION,
            name: this.parseFragmentName(),
            variableDefinitions: this.parseVariableDefinitions(),
            typeCondition: (this.expectKeyword("on"), this.parseNamedType()),
            directives: this.parseDirectives(false),
            selectionSet: this.parseSelectionSet()
          });
        }
        return this.node(start, {
          kind: _kinds.Kind.FRAGMENT_DEFINITION,
          name: this.parseFragmentName(),
          typeCondition: (this.expectKeyword("on"), this.parseNamedType()),
          directives: this.parseDirectives(false),
          selectionSet: this.parseSelectionSet()
        });
      }
      /**
       * FragmentName : Name but not `on`
       */
      parseFragmentName() {
        if (this._lexer.token.value === "on") {
          throw this.unexpected();
        }
        return this.parseName();
      }
      // Implements the parsing rules in the Values section.
      /**
       * Value[Const] :
       *   - [~Const] Variable
       *   - IntValue
       *   - FloatValue
       *   - StringValue
       *   - BooleanValue
       *   - NullValue
       *   - EnumValue
       *   - ListValue[?Const]
       *   - ObjectValue[?Const]
       *
       * BooleanValue : one of `true` `false`
       *
       * NullValue : `null`
       *
       * EnumValue : Name but not `true`, `false` or `null`
       */
      parseValueLiteral(isConst) {
        const token = this._lexer.token;
        switch (token.kind) {
          case _tokenKind.TokenKind.BRACKET_L:
            return this.parseList(isConst);
          case _tokenKind.TokenKind.BRACE_L:
            return this.parseObject(isConst);
          case _tokenKind.TokenKind.INT:
            this.advanceLexer();
            return this.node(token, {
              kind: _kinds.Kind.INT,
              value: token.value
            });
          case _tokenKind.TokenKind.FLOAT:
            this.advanceLexer();
            return this.node(token, {
              kind: _kinds.Kind.FLOAT,
              value: token.value
            });
          case _tokenKind.TokenKind.STRING:
          case _tokenKind.TokenKind.BLOCK_STRING:
            return this.parseStringLiteral();
          case _tokenKind.TokenKind.NAME:
            this.advanceLexer();
            switch (token.value) {
              case "true":
                return this.node(token, {
                  kind: _kinds.Kind.BOOLEAN,
                  value: true
                });
              case "false":
                return this.node(token, {
                  kind: _kinds.Kind.BOOLEAN,
                  value: false
                });
              case "null":
                return this.node(token, {
                  kind: _kinds.Kind.NULL
                });
              default:
                return this.node(token, {
                  kind: _kinds.Kind.ENUM,
                  value: token.value
                });
            }
          case _tokenKind.TokenKind.DOLLAR:
            if (isConst) {
              this.expectToken(_tokenKind.TokenKind.DOLLAR);
              if (this._lexer.token.kind === _tokenKind.TokenKind.NAME) {
                const varName = this._lexer.token.value;
                throw (0, _syntaxError.syntaxError)(
                  this._lexer.source,
                  token.start,
                  `Unexpected variable "$${varName}" in constant value.`
                );
              } else {
                throw this.unexpected(token);
              }
            }
            return this.parseVariable();
          default:
            throw this.unexpected();
        }
      }
      parseConstValueLiteral() {
        return this.parseValueLiteral(true);
      }
      parseStringLiteral() {
        const token = this._lexer.token;
        this.advanceLexer();
        return this.node(token, {
          kind: _kinds.Kind.STRING,
          value: token.value,
          block: token.kind === _tokenKind.TokenKind.BLOCK_STRING
        });
      }
      /**
       * ListValue[Const] :
       *   - [ ]
       *   - [ Value[?Const]+ ]
       */
      parseList(isConst) {
        const item = () => this.parseValueLiteral(isConst);
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.LIST,
          values: this.any(
            _tokenKind.TokenKind.BRACKET_L,
            item,
            _tokenKind.TokenKind.BRACKET_R
          )
        });
      }
      /**
       * ```
       * ObjectValue[Const] :
       *   - { }
       *   - { ObjectField[?Const]+ }
       * ```
       */
      parseObject(isConst) {
        const item = () => this.parseObjectField(isConst);
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.OBJECT,
          fields: this.any(
            _tokenKind.TokenKind.BRACE_L,
            item,
            _tokenKind.TokenKind.BRACE_R
          )
        });
      }
      /**
       * ObjectField[Const] : Name : Value[?Const]
       */
      parseObjectField(isConst) {
        const start = this._lexer.token;
        const name = this.parseName();
        this.expectToken(_tokenKind.TokenKind.COLON);
        return this.node(start, {
          kind: _kinds.Kind.OBJECT_FIELD,
          name,
          value: this.parseValueLiteral(isConst)
        });
      }
      // Implements the parsing rules in the Directives section.
      /**
       * Directives[Const] : Directive[?Const]+
       */
      parseDirectives(isConst) {
        const directives = [];
        while (this.peek(_tokenKind.TokenKind.AT)) {
          directives.push(this.parseDirective(isConst));
        }
        return directives;
      }
      parseConstDirectives() {
        return this.parseDirectives(true);
      }
      /**
       * ```
       * Directive[Const] : @ Name Arguments[?Const]?
       * ```
       */
      parseDirective(isConst) {
        const start = this._lexer.token;
        this.expectToken(_tokenKind.TokenKind.AT);
        return this.node(start, {
          kind: _kinds.Kind.DIRECTIVE,
          name: this.parseName(),
          arguments: this.parseArguments(isConst)
        });
      }
      // Implements the parsing rules in the Types section.
      /**
       * Type :
       *   - NamedType
       *   - ListType
       *   - NonNullType
       */
      parseTypeReference() {
        const start = this._lexer.token;
        let type;
        if (this.expectOptionalToken(_tokenKind.TokenKind.BRACKET_L)) {
          const innerType = this.parseTypeReference();
          this.expectToken(_tokenKind.TokenKind.BRACKET_R);
          type = this.node(start, {
            kind: _kinds.Kind.LIST_TYPE,
            type: innerType
          });
        } else {
          type = this.parseNamedType();
        }
        if (this.expectOptionalToken(_tokenKind.TokenKind.BANG)) {
          return this.node(start, {
            kind: _kinds.Kind.NON_NULL_TYPE,
            type
          });
        }
        return type;
      }
      /**
       * NamedType : Name
       */
      parseNamedType() {
        return this.node(this._lexer.token, {
          kind: _kinds.Kind.NAMED_TYPE,
          name: this.parseName()
        });
      }
      // Implements the parsing rules in the Type Definition section.
      peekDescription() {
        return this.peek(_tokenKind.TokenKind.STRING) || this.peek(_tokenKind.TokenKind.BLOCK_STRING);
      }
      /**
       * Description : StringValue
       */
      parseDescription() {
        if (this.peekDescription()) {
          return this.parseStringLiteral();
        }
      }
      /**
       * ```
       * SchemaDefinition : Description? schema Directives[Const]? { OperationTypeDefinition+ }
       * ```
       */
      parseSchemaDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("schema");
        const directives = this.parseConstDirectives();
        const operationTypes = this.many(
          _tokenKind.TokenKind.BRACE_L,
          this.parseOperationTypeDefinition,
          _tokenKind.TokenKind.BRACE_R
        );
        return this.node(start, {
          kind: _kinds.Kind.SCHEMA_DEFINITION,
          description,
          directives,
          operationTypes
        });
      }
      /**
       * OperationTypeDefinition : OperationType : NamedType
       */
      parseOperationTypeDefinition() {
        const start = this._lexer.token;
        const operation = this.parseOperationType();
        this.expectToken(_tokenKind.TokenKind.COLON);
        const type = this.parseNamedType();
        return this.node(start, {
          kind: _kinds.Kind.OPERATION_TYPE_DEFINITION,
          operation,
          type
        });
      }
      /**
       * ScalarTypeDefinition : Description? scalar Name Directives[Const]?
       */
      parseScalarTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("scalar");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        return this.node(start, {
          kind: _kinds.Kind.SCALAR_TYPE_DEFINITION,
          description,
          name,
          directives
        });
      }
      /**
       * ObjectTypeDefinition :
       *   Description?
       *   type Name ImplementsInterfaces? Directives[Const]? FieldsDefinition?
       */
      parseObjectTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("type");
        const name = this.parseName();
        const interfaces = this.parseImplementsInterfaces();
        const directives = this.parseConstDirectives();
        const fields = this.parseFieldsDefinition();
        return this.node(start, {
          kind: _kinds.Kind.OBJECT_TYPE_DEFINITION,
          description,
          name,
          interfaces,
          directives,
          fields
        });
      }
      /**
       * ImplementsInterfaces :
       *   - implements `&`? NamedType
       *   - ImplementsInterfaces & NamedType
       */
      parseImplementsInterfaces() {
        return this.expectOptionalKeyword("implements") ? this.delimitedMany(_tokenKind.TokenKind.AMP, this.parseNamedType) : [];
      }
      /**
       * ```
       * FieldsDefinition : { FieldDefinition+ }
       * ```
       */
      parseFieldsDefinition() {
        return this.optionalMany(
          _tokenKind.TokenKind.BRACE_L,
          this.parseFieldDefinition,
          _tokenKind.TokenKind.BRACE_R
        );
      }
      /**
       * FieldDefinition :
       *   - Description? Name ArgumentsDefinition? : Type Directives[Const]?
       */
      parseFieldDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        const name = this.parseName();
        const args = this.parseArgumentDefs();
        this.expectToken(_tokenKind.TokenKind.COLON);
        const type = this.parseTypeReference();
        const directives = this.parseConstDirectives();
        return this.node(start, {
          kind: _kinds.Kind.FIELD_DEFINITION,
          description,
          name,
          arguments: args,
          type,
          directives
        });
      }
      /**
       * ArgumentsDefinition : ( InputValueDefinition+ )
       */
      parseArgumentDefs() {
        return this.optionalMany(
          _tokenKind.TokenKind.PAREN_L,
          this.parseInputValueDef,
          _tokenKind.TokenKind.PAREN_R
        );
      }
      /**
       * InputValueDefinition :
       *   - Description? Name : Type DefaultValue? Directives[Const]?
       */
      parseInputValueDef() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        const name = this.parseName();
        this.expectToken(_tokenKind.TokenKind.COLON);
        const type = this.parseTypeReference();
        let defaultValue;
        if (this.expectOptionalToken(_tokenKind.TokenKind.EQUALS)) {
          defaultValue = this.parseConstValueLiteral();
        }
        const directives = this.parseConstDirectives();
        return this.node(start, {
          kind: _kinds.Kind.INPUT_VALUE_DEFINITION,
          description,
          name,
          type,
          defaultValue,
          directives
        });
      }
      /**
       * InterfaceTypeDefinition :
       *   - Description? interface Name Directives[Const]? FieldsDefinition?
       */
      parseInterfaceTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("interface");
        const name = this.parseName();
        const interfaces = this.parseImplementsInterfaces();
        const directives = this.parseConstDirectives();
        const fields = this.parseFieldsDefinition();
        return this.node(start, {
          kind: _kinds.Kind.INTERFACE_TYPE_DEFINITION,
          description,
          name,
          interfaces,
          directives,
          fields
        });
      }
      /**
       * UnionTypeDefinition :
       *   - Description? union Name Directives[Const]? UnionMemberTypes?
       */
      parseUnionTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("union");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const types = this.parseUnionMemberTypes();
        return this.node(start, {
          kind: _kinds.Kind.UNION_TYPE_DEFINITION,
          description,
          name,
          directives,
          types
        });
      }
      /**
       * UnionMemberTypes :
       *   - = `|`? NamedType
       *   - UnionMemberTypes | NamedType
       */
      parseUnionMemberTypes() {
        return this.expectOptionalToken(_tokenKind.TokenKind.EQUALS) ? this.delimitedMany(_tokenKind.TokenKind.PIPE, this.parseNamedType) : [];
      }
      /**
       * EnumTypeDefinition :
       *   - Description? enum Name Directives[Const]? EnumValuesDefinition?
       */
      parseEnumTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("enum");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const values = this.parseEnumValuesDefinition();
        return this.node(start, {
          kind: _kinds.Kind.ENUM_TYPE_DEFINITION,
          description,
          name,
          directives,
          values
        });
      }
      /**
       * ```
       * EnumValuesDefinition : { EnumValueDefinition+ }
       * ```
       */
      parseEnumValuesDefinition() {
        return this.optionalMany(
          _tokenKind.TokenKind.BRACE_L,
          this.parseEnumValueDefinition,
          _tokenKind.TokenKind.BRACE_R
        );
      }
      /**
       * EnumValueDefinition : Description? EnumValue Directives[Const]?
       */
      parseEnumValueDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        const name = this.parseEnumValueName();
        const directives = this.parseConstDirectives();
        return this.node(start, {
          kind: _kinds.Kind.ENUM_VALUE_DEFINITION,
          description,
          name,
          directives
        });
      }
      /**
       * EnumValue : Name but not `true`, `false` or `null`
       */
      parseEnumValueName() {
        if (this._lexer.token.value === "true" || this._lexer.token.value === "false" || this._lexer.token.value === "null") {
          throw (0, _syntaxError.syntaxError)(
            this._lexer.source,
            this._lexer.token.start,
            `${getTokenDesc(
              this._lexer.token
            )} is reserved and cannot be used for an enum value.`
          );
        }
        return this.parseName();
      }
      /**
       * InputObjectTypeDefinition :
       *   - Description? input Name Directives[Const]? InputFieldsDefinition?
       */
      parseInputObjectTypeDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("input");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const fields = this.parseInputFieldsDefinition();
        return this.node(start, {
          kind: _kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION,
          description,
          name,
          directives,
          fields
        });
      }
      /**
       * ```
       * InputFieldsDefinition : { InputValueDefinition+ }
       * ```
       */
      parseInputFieldsDefinition() {
        return this.optionalMany(
          _tokenKind.TokenKind.BRACE_L,
          this.parseInputValueDef,
          _tokenKind.TokenKind.BRACE_R
        );
      }
      /**
       * TypeSystemExtension :
       *   - SchemaExtension
       *   - TypeExtension
       *
       * TypeExtension :
       *   - ScalarTypeExtension
       *   - ObjectTypeExtension
       *   - InterfaceTypeExtension
       *   - UnionTypeExtension
       *   - EnumTypeExtension
       *   - InputObjectTypeDefinition
       */
      parseTypeSystemExtension() {
        const keywordToken = this._lexer.lookahead();
        if (keywordToken.kind === _tokenKind.TokenKind.NAME) {
          switch (keywordToken.value) {
            case "schema":
              return this.parseSchemaExtension();
            case "scalar":
              return this.parseScalarTypeExtension();
            case "type":
              return this.parseObjectTypeExtension();
            case "interface":
              return this.parseInterfaceTypeExtension();
            case "union":
              return this.parseUnionTypeExtension();
            case "enum":
              return this.parseEnumTypeExtension();
            case "input":
              return this.parseInputObjectTypeExtension();
          }
        }
        throw this.unexpected(keywordToken);
      }
      /**
       * ```
       * SchemaExtension :
       *  - extend schema Directives[Const]? { OperationTypeDefinition+ }
       *  - extend schema Directives[Const]
       * ```
       */
      parseSchemaExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("schema");
        const directives = this.parseConstDirectives();
        const operationTypes = this.optionalMany(
          _tokenKind.TokenKind.BRACE_L,
          this.parseOperationTypeDefinition,
          _tokenKind.TokenKind.BRACE_R
        );
        if (directives.length === 0 && operationTypes.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.SCHEMA_EXTENSION,
          directives,
          operationTypes
        });
      }
      /**
       * ScalarTypeExtension :
       *   - extend scalar Name Directives[Const]
       */
      parseScalarTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("scalar");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        if (directives.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.SCALAR_TYPE_EXTENSION,
          name,
          directives
        });
      }
      /**
       * ObjectTypeExtension :
       *  - extend type Name ImplementsInterfaces? Directives[Const]? FieldsDefinition
       *  - extend type Name ImplementsInterfaces? Directives[Const]
       *  - extend type Name ImplementsInterfaces
       */
      parseObjectTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("type");
        const name = this.parseName();
        const interfaces = this.parseImplementsInterfaces();
        const directives = this.parseConstDirectives();
        const fields = this.parseFieldsDefinition();
        if (interfaces.length === 0 && directives.length === 0 && fields.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.OBJECT_TYPE_EXTENSION,
          name,
          interfaces,
          directives,
          fields
        });
      }
      /**
       * InterfaceTypeExtension :
       *  - extend interface Name ImplementsInterfaces? Directives[Const]? FieldsDefinition
       *  - extend interface Name ImplementsInterfaces? Directives[Const]
       *  - extend interface Name ImplementsInterfaces
       */
      parseInterfaceTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("interface");
        const name = this.parseName();
        const interfaces = this.parseImplementsInterfaces();
        const directives = this.parseConstDirectives();
        const fields = this.parseFieldsDefinition();
        if (interfaces.length === 0 && directives.length === 0 && fields.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.INTERFACE_TYPE_EXTENSION,
          name,
          interfaces,
          directives,
          fields
        });
      }
      /**
       * UnionTypeExtension :
       *   - extend union Name Directives[Const]? UnionMemberTypes
       *   - extend union Name Directives[Const]
       */
      parseUnionTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("union");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const types = this.parseUnionMemberTypes();
        if (directives.length === 0 && types.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.UNION_TYPE_EXTENSION,
          name,
          directives,
          types
        });
      }
      /**
       * EnumTypeExtension :
       *   - extend enum Name Directives[Const]? EnumValuesDefinition
       *   - extend enum Name Directives[Const]
       */
      parseEnumTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("enum");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const values = this.parseEnumValuesDefinition();
        if (directives.length === 0 && values.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.ENUM_TYPE_EXTENSION,
          name,
          directives,
          values
        });
      }
      /**
       * InputObjectTypeExtension :
       *   - extend input Name Directives[Const]? InputFieldsDefinition
       *   - extend input Name Directives[Const]
       */
      parseInputObjectTypeExtension() {
        const start = this._lexer.token;
        this.expectKeyword("extend");
        this.expectKeyword("input");
        const name = this.parseName();
        const directives = this.parseConstDirectives();
        const fields = this.parseInputFieldsDefinition();
        if (directives.length === 0 && fields.length === 0) {
          throw this.unexpected();
        }
        return this.node(start, {
          kind: _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION,
          name,
          directives,
          fields
        });
      }
      /**
       * ```
       * DirectiveDefinition :
       *   - Description? directive @ Name ArgumentsDefinition? `repeatable`? on DirectiveLocations
       * ```
       */
      parseDirectiveDefinition() {
        const start = this._lexer.token;
        const description = this.parseDescription();
        this.expectKeyword("directive");
        this.expectToken(_tokenKind.TokenKind.AT);
        const name = this.parseName();
        const args = this.parseArgumentDefs();
        const repeatable = this.expectOptionalKeyword("repeatable");
        this.expectKeyword("on");
        const locations = this.parseDirectiveLocations();
        return this.node(start, {
          kind: _kinds.Kind.DIRECTIVE_DEFINITION,
          description,
          name,
          arguments: args,
          repeatable,
          locations
        });
      }
      /**
       * DirectiveLocations :
       *   - `|`? DirectiveLocation
       *   - DirectiveLocations | DirectiveLocation
       */
      parseDirectiveLocations() {
        return this.delimitedMany(
          _tokenKind.TokenKind.PIPE,
          this.parseDirectiveLocation
        );
      }
      /*
       * DirectiveLocation :
       *   - ExecutableDirectiveLocation
       *   - TypeSystemDirectiveLocation
       *
       * ExecutableDirectiveLocation : one of
       *   `QUERY`
       *   `MUTATION`
       *   `SUBSCRIPTION`
       *   `FIELD`
       *   `FRAGMENT_DEFINITION`
       *   `FRAGMENT_SPREAD`
       *   `INLINE_FRAGMENT`
       *
       * TypeSystemDirectiveLocation : one of
       *   `SCHEMA`
       *   `SCALAR`
       *   `OBJECT`
       *   `FIELD_DEFINITION`
       *   `ARGUMENT_DEFINITION`
       *   `INTERFACE`
       *   `UNION`
       *   `ENUM`
       *   `ENUM_VALUE`
       *   `INPUT_OBJECT`
       *   `INPUT_FIELD_DEFINITION`
       */
      parseDirectiveLocation() {
        const start = this._lexer.token;
        const name = this.parseName();
        if (Object.prototype.hasOwnProperty.call(
          _directiveLocation.DirectiveLocation,
          name.value
        )) {
          return name;
        }
        throw this.unexpected(start);
      }
      // Core parsing utility functions
      /**
       * Returns a node that, if configured to do so, sets a "loc" field as a
       * location object, used to identify the place in the source that created a
       * given parsed object.
       */
      node(startToken, node) {
        if (this._options.noLocation !== true) {
          node.loc = new _ast.Location(
            startToken,
            this._lexer.lastToken,
            this._lexer.source
          );
        }
        return node;
      }
      /**
       * Determines if the next token is of a given kind
       */
      peek(kind) {
        return this._lexer.token.kind === kind;
      }
      /**
       * If the next token is of the given kind, return that token after advancing the lexer.
       * Otherwise, do not change the parser state and throw an error.
       */
      expectToken(kind) {
        const token = this._lexer.token;
        if (token.kind === kind) {
          this.advanceLexer();
          return token;
        }
        throw (0, _syntaxError.syntaxError)(
          this._lexer.source,
          token.start,
          `Expected ${getTokenKindDesc(kind)}, found ${getTokenDesc(token)}.`
        );
      }
      /**
       * If the next token is of the given kind, return "true" after advancing the lexer.
       * Otherwise, do not change the parser state and return "false".
       */
      expectOptionalToken(kind) {
        const token = this._lexer.token;
        if (token.kind === kind) {
          this.advanceLexer();
          return true;
        }
        return false;
      }
      /**
       * If the next token is a given keyword, advance the lexer.
       * Otherwise, do not change the parser state and throw an error.
       */
      expectKeyword(value) {
        const token = this._lexer.token;
        if (token.kind === _tokenKind.TokenKind.NAME && token.value === value) {
          this.advanceLexer();
        } else {
          throw (0, _syntaxError.syntaxError)(
            this._lexer.source,
            token.start,
            `Expected "${value}", found ${getTokenDesc(token)}.`
          );
        }
      }
      /**
       * If the next token is a given keyword, return "true" after advancing the lexer.
       * Otherwise, do not change the parser state and return "false".
       */
      expectOptionalKeyword(value) {
        const token = this._lexer.token;
        if (token.kind === _tokenKind.TokenKind.NAME && token.value === value) {
          this.advanceLexer();
          return true;
        }
        return false;
      }
      /**
       * Helper function for creating an error when an unexpected lexed token is encountered.
       */
      unexpected(atToken) {
        const token = atToken !== null && atToken !== void 0 ? atToken : this._lexer.token;
        return (0, _syntaxError.syntaxError)(
          this._lexer.source,
          token.start,
          `Unexpected ${getTokenDesc(token)}.`
        );
      }
      /**
       * Returns a possibly empty list of parse nodes, determined by the parseFn.
       * This list begins with a lex token of openKind and ends with a lex token of closeKind.
       * Advances the parser to the next lex token after the closing token.
       */
      any(openKind, parseFn, closeKind) {
        this.expectToken(openKind);
        const nodes = [];
        while (!this.expectOptionalToken(closeKind)) {
          nodes.push(parseFn.call(this));
        }
        return nodes;
      }
      /**
       * Returns a list of parse nodes, determined by the parseFn.
       * It can be empty only if open token is missing otherwise it will always return non-empty list
       * that begins with a lex token of openKind and ends with a lex token of closeKind.
       * Advances the parser to the next lex token after the closing token.
       */
      optionalMany(openKind, parseFn, closeKind) {
        if (this.expectOptionalToken(openKind)) {
          const nodes = [];
          do {
            nodes.push(parseFn.call(this));
          } while (!this.expectOptionalToken(closeKind));
          return nodes;
        }
        return [];
      }
      /**
       * Returns a non-empty list of parse nodes, determined by the parseFn.
       * This list begins with a lex token of openKind and ends with a lex token of closeKind.
       * Advances the parser to the next lex token after the closing token.
       */
      many(openKind, parseFn, closeKind) {
        this.expectToken(openKind);
        const nodes = [];
        do {
          nodes.push(parseFn.call(this));
        } while (!this.expectOptionalToken(closeKind));
        return nodes;
      }
      /**
       * Returns a non-empty list of parse nodes, determined by the parseFn.
       * This list may begin with a lex token of delimiterKind followed by items separated by lex tokens of tokenKind.
       * Advances the parser to the next lex token after last item in the list.
       */
      delimitedMany(delimiterKind, parseFn) {
        this.expectOptionalToken(delimiterKind);
        const nodes = [];
        do {
          nodes.push(parseFn.call(this));
        } while (this.expectOptionalToken(delimiterKind));
        return nodes;
      }
      advanceLexer() {
        const { maxTokens } = this._options;
        const token = this._lexer.advance();
        if (maxTokens !== void 0 && token.kind !== _tokenKind.TokenKind.EOF) {
          ++this._tokenCounter;
          if (this._tokenCounter > maxTokens) {
            throw (0, _syntaxError.syntaxError)(
              this._lexer.source,
              token.start,
              `Document contains more that ${maxTokens} tokens. Parsing aborted.`
            );
          }
        }
      }
    };
    exports2.Parser = Parser;
    function getTokenDesc(token) {
      const value = token.value;
      return getTokenKindDesc(token.kind) + (value != null ? ` "${value}"` : "");
    }
    function getTokenKindDesc(kind) {
      return (0, _lexer.isPunctuatorTokenKind)(kind) ? `"${kind}"` : kind;
    }
  }
});

// node_modules/graphql/jsutils/didYouMean.js
var require_didYouMean = __commonJS({
  "node_modules/graphql/jsutils/didYouMean.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.didYouMean = didYouMean;
    var MAX_SUGGESTIONS = 5;
    function didYouMean(firstArg, secondArg) {
      const [subMessage, suggestionsArg] = secondArg ? [firstArg, secondArg] : [void 0, firstArg];
      let message = " Did you mean ";
      if (subMessage) {
        message += subMessage + " ";
      }
      const suggestions = suggestionsArg.map((x) => `"${x}"`);
      switch (suggestions.length) {
        case 0:
          return "";
        case 1:
          return message + suggestions[0] + "?";
        case 2:
          return message + suggestions[0] + " or " + suggestions[1] + "?";
      }
      const selected = suggestions.slice(0, MAX_SUGGESTIONS);
      const lastItem = selected.pop();
      return message + selected.join(", ") + ", or " + lastItem + "?";
    }
  }
});

// node_modules/graphql/jsutils/identityFunc.js
var require_identityFunc = __commonJS({
  "node_modules/graphql/jsutils/identityFunc.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.identityFunc = identityFunc;
    function identityFunc(x) {
      return x;
    }
  }
});

// node_modules/graphql/jsutils/keyMap.js
var require_keyMap = __commonJS({
  "node_modules/graphql/jsutils/keyMap.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.keyMap = keyMap;
    function keyMap(list, keyFn) {
      const result = /* @__PURE__ */ Object.create(null);
      for (const item of list) {
        result[keyFn(item)] = item;
      }
      return result;
    }
  }
});

// node_modules/graphql/jsutils/keyValMap.js
var require_keyValMap = __commonJS({
  "node_modules/graphql/jsutils/keyValMap.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.keyValMap = keyValMap;
    function keyValMap(list, keyFn, valFn) {
      const result = /* @__PURE__ */ Object.create(null);
      for (const item of list) {
        result[keyFn(item)] = valFn(item);
      }
      return result;
    }
  }
});

// node_modules/graphql/jsutils/mapValue.js
var require_mapValue = __commonJS({
  "node_modules/graphql/jsutils/mapValue.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.mapValue = mapValue;
    function mapValue(map, fn) {
      const result = /* @__PURE__ */ Object.create(null);
      for (const key of Object.keys(map)) {
        result[key] = fn(map[key], key);
      }
      return result;
    }
  }
});

// node_modules/graphql/jsutils/naturalCompare.js
var require_naturalCompare = __commonJS({
  "node_modules/graphql/jsutils/naturalCompare.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.naturalCompare = naturalCompare;
    function naturalCompare(aStr, bStr) {
      let aIndex = 0;
      let bIndex = 0;
      while (aIndex < aStr.length && bIndex < bStr.length) {
        let aChar = aStr.charCodeAt(aIndex);
        let bChar = bStr.charCodeAt(bIndex);
        if (isDigit(aChar) && isDigit(bChar)) {
          let aNum = 0;
          do {
            ++aIndex;
            aNum = aNum * 10 + aChar - DIGIT_0;
            aChar = aStr.charCodeAt(aIndex);
          } while (isDigit(aChar) && aNum > 0);
          let bNum = 0;
          do {
            ++bIndex;
            bNum = bNum * 10 + bChar - DIGIT_0;
            bChar = bStr.charCodeAt(bIndex);
          } while (isDigit(bChar) && bNum > 0);
          if (aNum < bNum) {
            return -1;
          }
          if (aNum > bNum) {
            return 1;
          }
        } else {
          if (aChar < bChar) {
            return -1;
          }
          if (aChar > bChar) {
            return 1;
          }
          ++aIndex;
          ++bIndex;
        }
      }
      return aStr.length - bStr.length;
    }
    var DIGIT_0 = 48;
    var DIGIT_9 = 57;
    function isDigit(code) {
      return !isNaN(code) && DIGIT_0 <= code && code <= DIGIT_9;
    }
  }
});

// node_modules/graphql/jsutils/suggestionList.js
var require_suggestionList = __commonJS({
  "node_modules/graphql/jsutils/suggestionList.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.suggestionList = suggestionList;
    var _naturalCompare = require_naturalCompare();
    function suggestionList(input, options) {
      const optionsByDistance = /* @__PURE__ */ Object.create(null);
      const lexicalDistance = new LexicalDistance(input);
      const threshold = Math.floor(input.length * 0.4) + 1;
      for (const option of options) {
        const distance = lexicalDistance.measure(option, threshold);
        if (distance !== void 0) {
          optionsByDistance[option] = distance;
        }
      }
      return Object.keys(optionsByDistance).sort((a, b) => {
        const distanceDiff = optionsByDistance[a] - optionsByDistance[b];
        return distanceDiff !== 0 ? distanceDiff : (0, _naturalCompare.naturalCompare)(a, b);
      });
    }
    var LexicalDistance = class {
      constructor(input) {
        this._input = input;
        this._inputLowerCase = input.toLowerCase();
        this._inputArray = stringToArray(this._inputLowerCase);
        this._rows = [
          new Array(input.length + 1).fill(0),
          new Array(input.length + 1).fill(0),
          new Array(input.length + 1).fill(0)
        ];
      }
      measure(option, threshold) {
        if (this._input === option) {
          return 0;
        }
        const optionLowerCase = option.toLowerCase();
        if (this._inputLowerCase === optionLowerCase) {
          return 1;
        }
        let a = stringToArray(optionLowerCase);
        let b = this._inputArray;
        if (a.length < b.length) {
          const tmp = a;
          a = b;
          b = tmp;
        }
        const aLength = a.length;
        const bLength = b.length;
        if (aLength - bLength > threshold) {
          return void 0;
        }
        const rows = this._rows;
        for (let j = 0; j <= bLength; j++) {
          rows[0][j] = j;
        }
        for (let i = 1; i <= aLength; i++) {
          const upRow = rows[(i - 1) % 3];
          const currentRow = rows[i % 3];
          let smallestCell = currentRow[0] = i;
          for (let j = 1; j <= bLength; j++) {
            const cost = a[i - 1] === b[j - 1] ? 0 : 1;
            let currentCell = Math.min(
              upRow[j] + 1,
              // delete
              currentRow[j - 1] + 1,
              // insert
              upRow[j - 1] + cost
              // substitute
            );
            if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) {
              const doubleDiagonalCell = rows[(i - 2) % 3][j - 2];
              currentCell = Math.min(currentCell, doubleDiagonalCell + 1);
            }
            if (currentCell < smallestCell) {
              smallestCell = currentCell;
            }
            currentRow[j] = currentCell;
          }
          if (smallestCell > threshold) {
            return void 0;
          }
        }
        const distance = rows[aLength % 3][bLength];
        return distance <= threshold ? distance : void 0;
      }
    };
    function stringToArray(str) {
      const strLength = str.length;
      const array = new Array(strLength);
      for (let i = 0; i < strLength; ++i) {
        array[i] = str.charCodeAt(i);
      }
      return array;
    }
  }
});

// node_modules/graphql/jsutils/toObjMap.js
var require_toObjMap = __commonJS({
  "node_modules/graphql/jsutils/toObjMap.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.toObjMap = toObjMap;
    function toObjMap(obj) {
      if (obj == null) {
        return /* @__PURE__ */ Object.create(null);
      }
      if (Object.getPrototypeOf(obj) === null) {
        return obj;
      }
      const map = /* @__PURE__ */ Object.create(null);
      for (const [key, value] of Object.entries(obj)) {
        map[key] = value;
      }
      return map;
    }
  }
});

// node_modules/graphql/language/printString.js
var require_printString = __commonJS({
  "node_modules/graphql/language/printString.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.printString = printString;
    function printString(str) {
      return `"${str.replace(escapedRegExp, escapedReplacer)}"`;
    }
    var escapedRegExp = /[\x00-\x1f\x22\x5c\x7f-\x9f]/g;
    function escapedReplacer(str) {
      return escapeSequences[str.charCodeAt(0)];
    }
    var escapeSequences = [
      "\\u0000",
      "\\u0001",
      "\\u0002",
      "\\u0003",
      "\\u0004",
      "\\u0005",
      "\\u0006",
      "\\u0007",
      "\\b",
      "\\t",
      "\\n",
      "\\u000B",
      "\\f",
      "\\r",
      "\\u000E",
      "\\u000F",
      "\\u0010",
      "\\u0011",
      "\\u0012",
      "\\u0013",
      "\\u0014",
      "\\u0015",
      "\\u0016",
      "\\u0017",
      "\\u0018",
      "\\u0019",
      "\\u001A",
      "\\u001B",
      "\\u001C",
      "\\u001D",
      "\\u001E",
      "\\u001F",
      "",
      "",
      '\\"',
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      // 2F
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      // 3F
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      // 4F
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "\\\\",
      "",
      "",
      "",
      // 5F
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      // 6F
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "",
      "\\u007F",
      "\\u0080",
      "\\u0081",
      "\\u0082",
      "\\u0083",
      "\\u0084",
      "\\u0085",
      "\\u0086",
      "\\u0087",
      "\\u0088",
      "\\u0089",
      "\\u008A",
      "\\u008B",
      "\\u008C",
      "\\u008D",
      "\\u008E",
      "\\u008F",
      "\\u0090",
      "\\u0091",
      "\\u0092",
      "\\u0093",
      "\\u0094",
      "\\u0095",
      "\\u0096",
      "\\u0097",
      "\\u0098",
      "\\u0099",
      "\\u009A",
      "\\u009B",
      "\\u009C",
      "\\u009D",
      "\\u009E",
      "\\u009F"
    ];
  }
});

// node_modules/graphql/language/visitor.js
var require_visitor = __commonJS({
  "node_modules/graphql/language/visitor.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.BREAK = void 0;
    exports2.getEnterLeaveForKind = getEnterLeaveForKind;
    exports2.getVisitFn = getVisitFn;
    exports2.visit = visit;
    exports2.visitInParallel = visitInParallel;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _ast = require_ast();
    var _kinds = require_kinds();
    var BREAK = Object.freeze({});
    exports2.BREAK = BREAK;
    function visit(root, visitor, visitorKeys = _ast.QueryDocumentKeys) {
      const enterLeaveMap = /* @__PURE__ */ new Map();
      for (const kind of Object.values(_kinds.Kind)) {
        enterLeaveMap.set(kind, getEnterLeaveForKind(visitor, kind));
      }
      let stack = void 0;
      let inArray = Array.isArray(root);
      let keys = [root];
      let index = -1;
      let edits = [];
      let node = root;
      let key = void 0;
      let parent = void 0;
      const path4 = [];
      const ancestors = [];
      do {
        index++;
        const isLeaving = index === keys.length;
        const isEdited = isLeaving && edits.length !== 0;
        if (isLeaving) {
          key = ancestors.length === 0 ? void 0 : path4[path4.length - 1];
          node = parent;
          parent = ancestors.pop();
          if (isEdited) {
            if (inArray) {
              node = node.slice();
              let editOffset = 0;
              for (const [editKey, editValue] of edits) {
                const arrayKey = editKey - editOffset;
                if (editValue === null) {
                  node.splice(arrayKey, 1);
                  editOffset++;
                } else {
                  node[arrayKey] = editValue;
                }
              }
            } else {
              node = Object.defineProperties(
                {},
                Object.getOwnPropertyDescriptors(node)
              );
              for (const [editKey, editValue] of edits) {
                node[editKey] = editValue;
              }
            }
          }
          index = stack.index;
          keys = stack.keys;
          edits = stack.edits;
          inArray = stack.inArray;
          stack = stack.prev;
        } else if (parent) {
          key = inArray ? index : keys[index];
          node = parent[key];
          if (node === null || node === void 0) {
            continue;
          }
          path4.push(key);
        }
        let result;
        if (!Array.isArray(node)) {
          var _enterLeaveMap$get, _enterLeaveMap$get2;
          (0, _ast.isNode)(node) || (0, _devAssert.devAssert)(
            false,
            `Invalid AST Node: ${(0, _inspect.inspect)(node)}.`
          );
          const visitFn = isLeaving ? (_enterLeaveMap$get = enterLeaveMap.get(node.kind)) === null || _enterLeaveMap$get === void 0 ? void 0 : _enterLeaveMap$get.leave : (_enterLeaveMap$get2 = enterLeaveMap.get(node.kind)) === null || _enterLeaveMap$get2 === void 0 ? void 0 : _enterLeaveMap$get2.enter;
          result = visitFn === null || visitFn === void 0 ? void 0 : visitFn.call(visitor, node, key, parent, path4, ancestors);
          if (result === BREAK) {
            break;
          }
          if (result === false) {
            if (!isLeaving) {
              path4.pop();
              continue;
            }
          } else if (result !== void 0) {
            edits.push([key, result]);
            if (!isLeaving) {
              if ((0, _ast.isNode)(result)) {
                node = result;
              } else {
                path4.pop();
                continue;
              }
            }
          }
        }
        if (result === void 0 && isEdited) {
          edits.push([key, node]);
        }
        if (isLeaving) {
          path4.pop();
        } else {
          var _node$kind;
          stack = {
            inArray,
            index,
            keys,
            edits,
            prev: stack
          };
          inArray = Array.isArray(node);
          keys = inArray ? node : (_node$kind = visitorKeys[node.kind]) !== null && _node$kind !== void 0 ? _node$kind : [];
          index = -1;
          edits = [];
          if (parent) {
            ancestors.push(parent);
          }
          parent = node;
        }
      } while (stack !== void 0);
      if (edits.length !== 0) {
        return edits[edits.length - 1][1];
      }
      return root;
    }
    function visitInParallel(visitors) {
      const skipping = new Array(visitors.length).fill(null);
      const mergedVisitor = /* @__PURE__ */ Object.create(null);
      for (const kind of Object.values(_kinds.Kind)) {
        let hasVisitor = false;
        const enterList = new Array(visitors.length).fill(void 0);
        const leaveList = new Array(visitors.length).fill(void 0);
        for (let i = 0; i < visitors.length; ++i) {
          const { enter, leave } = getEnterLeaveForKind(visitors[i], kind);
          hasVisitor || (hasVisitor = enter != null || leave != null);
          enterList[i] = enter;
          leaveList[i] = leave;
        }
        if (!hasVisitor) {
          continue;
        }
        const mergedEnterLeave = {
          enter(...args) {
            const node = args[0];
            for (let i = 0; i < visitors.length; i++) {
              if (skipping[i] === null) {
                var _enterList$i;
                const result = (_enterList$i = enterList[i]) === null || _enterList$i === void 0 ? void 0 : _enterList$i.apply(visitors[i], args);
                if (result === false) {
                  skipping[i] = node;
                } else if (result === BREAK) {
                  skipping[i] = BREAK;
                } else if (result !== void 0) {
                  return result;
                }
              }
            }
          },
          leave(...args) {
            const node = args[0];
            for (let i = 0; i < visitors.length; i++) {
              if (skipping[i] === null) {
                var _leaveList$i;
                const result = (_leaveList$i = leaveList[i]) === null || _leaveList$i === void 0 ? void 0 : _leaveList$i.apply(visitors[i], args);
                if (result === BREAK) {
                  skipping[i] = BREAK;
                } else if (result !== void 0 && result !== false) {
                  return result;
                }
              } else if (skipping[i] === node) {
                skipping[i] = null;
              }
            }
          }
        };
        mergedVisitor[kind] = mergedEnterLeave;
      }
      return mergedVisitor;
    }
    function getEnterLeaveForKind(visitor, kind) {
      const kindVisitor = visitor[kind];
      if (typeof kindVisitor === "object") {
        return kindVisitor;
      } else if (typeof kindVisitor === "function") {
        return {
          enter: kindVisitor,
          leave: void 0
        };
      }
      return {
        enter: visitor.enter,
        leave: visitor.leave
      };
    }
    function getVisitFn(visitor, kind, isLeaving) {
      const { enter, leave } = getEnterLeaveForKind(visitor, kind);
      return isLeaving ? leave : enter;
    }
  }
});

// node_modules/graphql/language/printer.js
var require_printer = __commonJS({
  "node_modules/graphql/language/printer.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.print = print;
    var _blockString = require_blockString();
    var _printString = require_printString();
    var _visitor = require_visitor();
    function print(ast) {
      return (0, _visitor.visit)(ast, printDocASTReducer);
    }
    var MAX_LINE_LENGTH = 80;
    var printDocASTReducer = {
      Name: {
        leave: (node) => node.value
      },
      Variable: {
        leave: (node) => "$" + node.name
      },
      // Document
      Document: {
        leave: (node) => join(node.definitions, "\n\n")
      },
      OperationDefinition: {
        leave(node) {
          const varDefs = wrap("(", join(node.variableDefinitions, ", "), ")");
          const prefix = join(
            [
              node.operation,
              join([node.name, varDefs]),
              join(node.directives, " ")
            ],
            " "
          );
          return (prefix === "query" ? "" : prefix + " ") + node.selectionSet;
        }
      },
      VariableDefinition: {
        leave: ({ variable, type, defaultValue, directives }) => variable + ": " + type + wrap(" = ", defaultValue) + wrap(" ", join(directives, " "))
      },
      SelectionSet: {
        leave: ({ selections }) => block(selections)
      },
      Field: {
        leave({ alias, name, arguments: args, directives, selectionSet }) {
          const prefix = wrap("", alias, ": ") + name;
          let argsLine = prefix + wrap("(", join(args, ", "), ")");
          if (argsLine.length > MAX_LINE_LENGTH) {
            argsLine = prefix + wrap("(\n", indent(join(args, "\n")), "\n)");
          }
          return join([argsLine, join(directives, " "), selectionSet], " ");
        }
      },
      Argument: {
        leave: ({ name, value }) => name + ": " + value
      },
      // Fragments
      FragmentSpread: {
        leave: ({ name, directives }) => "..." + name + wrap(" ", join(directives, " "))
      },
      InlineFragment: {
        leave: ({ typeCondition, directives, selectionSet }) => join(
          [
            "...",
            wrap("on ", typeCondition),
            join(directives, " "),
            selectionSet
          ],
          " "
        )
      },
      FragmentDefinition: {
        leave: ({ name, typeCondition, variableDefinitions, directives, selectionSet }) => (
          // or removed in the future.
          `fragment ${name}${wrap("(", join(variableDefinitions, ", "), ")")} on ${typeCondition} ${wrap("", join(directives, " "), " ")}` + selectionSet
        )
      },
      // Value
      IntValue: {
        leave: ({ value }) => value
      },
      FloatValue: {
        leave: ({ value }) => value
      },
      StringValue: {
        leave: ({ value, block: isBlockString }) => isBlockString ? (0, _blockString.printBlockString)(value) : (0, _printString.printString)(value)
      },
      BooleanValue: {
        leave: ({ value }) => value ? "true" : "false"
      },
      NullValue: {
        leave: () => "null"
      },
      EnumValue: {
        leave: ({ value }) => value
      },
      ListValue: {
        leave: ({ values }) => "[" + join(values, ", ") + "]"
      },
      ObjectValue: {
        leave: ({ fields }) => "{" + join(fields, ", ") + "}"
      },
      ObjectField: {
        leave: ({ name, value }) => name + ": " + value
      },
      // Directive
      Directive: {
        leave: ({ name, arguments: args }) => "@" + name + wrap("(", join(args, ", "), ")")
      },
      // Type
      NamedType: {
        leave: ({ name }) => name
      },
      ListType: {
        leave: ({ type }) => "[" + type + "]"
      },
      NonNullType: {
        leave: ({ type }) => type + "!"
      },
      // Type System Definitions
      SchemaDefinition: {
        leave: ({ description, directives, operationTypes }) => wrap("", description, "\n") + join(["schema", join(directives, " "), block(operationTypes)], " ")
      },
      OperationTypeDefinition: {
        leave: ({ operation, type }) => operation + ": " + type
      },
      ScalarTypeDefinition: {
        leave: ({ description, name, directives }) => wrap("", description, "\n") + join(["scalar", name, join(directives, " ")], " ")
      },
      ObjectTypeDefinition: {
        leave: ({ description, name, interfaces, directives, fields }) => wrap("", description, "\n") + join(
          [
            "type",
            name,
            wrap("implements ", join(interfaces, " & ")),
            join(directives, " "),
            block(fields)
          ],
          " "
        )
      },
      FieldDefinition: {
        leave: ({ description, name, arguments: args, type, directives }) => wrap("", description, "\n") + name + (hasMultilineItems(args) ? wrap("(\n", indent(join(args, "\n")), "\n)") : wrap("(", join(args, ", "), ")")) + ": " + type + wrap(" ", join(directives, " "))
      },
      InputValueDefinition: {
        leave: ({ description, name, type, defaultValue, directives }) => wrap("", description, "\n") + join(
          [name + ": " + type, wrap("= ", defaultValue), join(directives, " ")],
          " "
        )
      },
      InterfaceTypeDefinition: {
        leave: ({ description, name, interfaces, directives, fields }) => wrap("", description, "\n") + join(
          [
            "interface",
            name,
            wrap("implements ", join(interfaces, " & ")),
            join(directives, " "),
            block(fields)
          ],
          " "
        )
      },
      UnionTypeDefinition: {
        leave: ({ description, name, directives, types }) => wrap("", description, "\n") + join(
          ["union", name, join(directives, " "), wrap("= ", join(types, " | "))],
          " "
        )
      },
      EnumTypeDefinition: {
        leave: ({ description, name, directives, values }) => wrap("", description, "\n") + join(["enum", name, join(directives, " "), block(values)], " ")
      },
      EnumValueDefinition: {
        leave: ({ description, name, directives }) => wrap("", description, "\n") + join([name, join(directives, " ")], " ")
      },
      InputObjectTypeDefinition: {
        leave: ({ description, name, directives, fields }) => wrap("", description, "\n") + join(["input", name, join(directives, " "), block(fields)], " ")
      },
      DirectiveDefinition: {
        leave: ({ description, name, arguments: args, repeatable, locations }) => wrap("", description, "\n") + "directive @" + name + (hasMultilineItems(args) ? wrap("(\n", indent(join(args, "\n")), "\n)") : wrap("(", join(args, ", "), ")")) + (repeatable ? " repeatable" : "") + " on " + join(locations, " | ")
      },
      SchemaExtension: {
        leave: ({ directives, operationTypes }) => join(
          ["extend schema", join(directives, " "), block(operationTypes)],
          " "
        )
      },
      ScalarTypeExtension: {
        leave: ({ name, directives }) => join(["extend scalar", name, join(directives, " ")], " ")
      },
      ObjectTypeExtension: {
        leave: ({ name, interfaces, directives, fields }) => join(
          [
            "extend type",
            name,
            wrap("implements ", join(interfaces, " & ")),
            join(directives, " "),
            block(fields)
          ],
          " "
        )
      },
      InterfaceTypeExtension: {
        leave: ({ name, interfaces, directives, fields }) => join(
          [
            "extend interface",
            name,
            wrap("implements ", join(interfaces, " & ")),
            join(directives, " "),
            block(fields)
          ],
          " "
        )
      },
      UnionTypeExtension: {
        leave: ({ name, directives, types }) => join(
          [
            "extend union",
            name,
            join(directives, " "),
            wrap("= ", join(types, " | "))
          ],
          " "
        )
      },
      EnumTypeExtension: {
        leave: ({ name, directives, values }) => join(["extend enum", name, join(directives, " "), block(values)], " ")
      },
      InputObjectTypeExtension: {
        leave: ({ name, directives, fields }) => join(["extend input", name, join(directives, " "), block(fields)], " ")
      }
    };
    function join(maybeArray, separator = "") {
      var _maybeArray$filter$jo;
      return (_maybeArray$filter$jo = maybeArray === null || maybeArray === void 0 ? void 0 : maybeArray.filter((x) => x).join(separator)) !== null && _maybeArray$filter$jo !== void 0 ? _maybeArray$filter$jo : "";
    }
    function block(array) {
      return wrap("{\n", indent(join(array, "\n")), "\n}");
    }
    function wrap(start, maybeString, end = "") {
      return maybeString != null && maybeString !== "" ? start + maybeString + end : "";
    }
    function indent(str) {
      return wrap("  ", str.replace(/\n/g, "\n  "));
    }
    function hasMultilineItems(maybeArray) {
      var _maybeArray$some;
      return (_maybeArray$some = maybeArray === null || maybeArray === void 0 ? void 0 : maybeArray.some((str) => str.includes("\n"))) !== null && _maybeArray$some !== void 0 ? _maybeArray$some : false;
    }
  }
});

// node_modules/graphql/utilities/valueFromASTUntyped.js
var require_valueFromASTUntyped = __commonJS({
  "node_modules/graphql/utilities/valueFromASTUntyped.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.valueFromASTUntyped = valueFromASTUntyped;
    var _keyValMap = require_keyValMap();
    var _kinds = require_kinds();
    function valueFromASTUntyped(valueNode, variables) {
      switch (valueNode.kind) {
        case _kinds.Kind.NULL:
          return null;
        case _kinds.Kind.INT:
          return parseInt(valueNode.value, 10);
        case _kinds.Kind.FLOAT:
          return parseFloat(valueNode.value);
        case _kinds.Kind.STRING:
        case _kinds.Kind.ENUM:
        case _kinds.Kind.BOOLEAN:
          return valueNode.value;
        case _kinds.Kind.LIST:
          return valueNode.values.map(
            (node) => valueFromASTUntyped(node, variables)
          );
        case _kinds.Kind.OBJECT:
          return (0, _keyValMap.keyValMap)(
            valueNode.fields,
            (field) => field.name.value,
            (field) => valueFromASTUntyped(field.value, variables)
          );
        case _kinds.Kind.VARIABLE:
          return variables === null || variables === void 0 ? void 0 : variables[valueNode.name.value];
      }
    }
  }
});

// node_modules/graphql/type/assertName.js
var require_assertName = __commonJS({
  "node_modules/graphql/type/assertName.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.assertEnumValueName = assertEnumValueName;
    exports2.assertName = assertName;
    var _devAssert = require_devAssert();
    var _GraphQLError = require_GraphQLError();
    var _characterClasses = require_characterClasses();
    function assertName(name) {
      name != null || (0, _devAssert.devAssert)(false, "Must provide name.");
      typeof name === "string" || (0, _devAssert.devAssert)(false, "Expected name to be a string.");
      if (name.length === 0) {
        throw new _GraphQLError.GraphQLError(
          "Expected name to be a non-empty string."
        );
      }
      for (let i = 1; i < name.length; ++i) {
        if (!(0, _characterClasses.isNameContinue)(name.charCodeAt(i))) {
          throw new _GraphQLError.GraphQLError(
            `Names must only contain [_a-zA-Z0-9] but "${name}" does not.`
          );
        }
      }
      if (!(0, _characterClasses.isNameStart)(name.charCodeAt(0))) {
        throw new _GraphQLError.GraphQLError(
          `Names must start with [_a-zA-Z] but "${name}" does not.`
        );
      }
      return name;
    }
    function assertEnumValueName(name) {
      if (name === "true" || name === "false" || name === "null") {
        throw new _GraphQLError.GraphQLError(
          `Enum values cannot be named: ${name}`
        );
      }
      return assertName(name);
    }
  }
});

// node_modules/graphql/type/definition.js
var require_definition = __commonJS({
  "node_modules/graphql/type/definition.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.GraphQLUnionType = exports2.GraphQLScalarType = exports2.GraphQLObjectType = exports2.GraphQLNonNull = exports2.GraphQLList = exports2.GraphQLInterfaceType = exports2.GraphQLInputObjectType = exports2.GraphQLEnumType = void 0;
    exports2.argsToArgsConfig = argsToArgsConfig;
    exports2.assertAbstractType = assertAbstractType;
    exports2.assertCompositeType = assertCompositeType;
    exports2.assertEnumType = assertEnumType;
    exports2.assertInputObjectType = assertInputObjectType;
    exports2.assertInputType = assertInputType;
    exports2.assertInterfaceType = assertInterfaceType;
    exports2.assertLeafType = assertLeafType;
    exports2.assertListType = assertListType;
    exports2.assertNamedType = assertNamedType;
    exports2.assertNonNullType = assertNonNullType;
    exports2.assertNullableType = assertNullableType;
    exports2.assertObjectType = assertObjectType;
    exports2.assertOutputType = assertOutputType;
    exports2.assertScalarType = assertScalarType;
    exports2.assertType = assertType;
    exports2.assertUnionType = assertUnionType;
    exports2.assertWrappingType = assertWrappingType;
    exports2.defineArguments = defineArguments;
    exports2.getNamedType = getNamedType;
    exports2.getNullableType = getNullableType;
    exports2.isAbstractType = isAbstractType;
    exports2.isCompositeType = isCompositeType;
    exports2.isEnumType = isEnumType;
    exports2.isInputObjectType = isInputObjectType;
    exports2.isInputType = isInputType;
    exports2.isInterfaceType = isInterfaceType;
    exports2.isLeafType = isLeafType;
    exports2.isListType = isListType;
    exports2.isNamedType = isNamedType;
    exports2.isNonNullType = isNonNullType;
    exports2.isNullableType = isNullableType;
    exports2.isObjectType = isObjectType;
    exports2.isOutputType = isOutputType;
    exports2.isRequiredArgument = isRequiredArgument;
    exports2.isRequiredInputField = isRequiredInputField;
    exports2.isScalarType = isScalarType;
    exports2.isType = isType;
    exports2.isUnionType = isUnionType;
    exports2.isWrappingType = isWrappingType;
    exports2.resolveObjMapThunk = resolveObjMapThunk;
    exports2.resolveReadonlyArrayThunk = resolveReadonlyArrayThunk;
    var _devAssert = require_devAssert();
    var _didYouMean = require_didYouMean();
    var _identityFunc = require_identityFunc();
    var _inspect = require_inspect();
    var _instanceOf = require_instanceOf();
    var _isObjectLike = require_isObjectLike();
    var _keyMap = require_keyMap();
    var _keyValMap = require_keyValMap();
    var _mapValue = require_mapValue();
    var _suggestionList = require_suggestionList();
    var _toObjMap = require_toObjMap();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _valueFromASTUntyped = require_valueFromASTUntyped();
    var _assertName = require_assertName();
    function isType(type) {
      return isScalarType(type) || isObjectType(type) || isInterfaceType(type) || isUnionType(type) || isEnumType(type) || isInputObjectType(type) || isListType(type) || isNonNullType(type);
    }
    function assertType(type) {
      if (!isType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL type.`
        );
      }
      return type;
    }
    function isScalarType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLScalarType);
    }
    function assertScalarType(type) {
      if (!isScalarType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Scalar type.`
        );
      }
      return type;
    }
    function isObjectType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLObjectType);
    }
    function assertObjectType(type) {
      if (!isObjectType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Object type.`
        );
      }
      return type;
    }
    function isInterfaceType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLInterfaceType);
    }
    function assertInterfaceType(type) {
      if (!isInterfaceType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Interface type.`
        );
      }
      return type;
    }
    function isUnionType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLUnionType);
    }
    function assertUnionType(type) {
      if (!isUnionType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Union type.`
        );
      }
      return type;
    }
    function isEnumType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLEnumType);
    }
    function assertEnumType(type) {
      if (!isEnumType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Enum type.`
        );
      }
      return type;
    }
    function isInputObjectType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLInputObjectType);
    }
    function assertInputObjectType(type) {
      if (!isInputObjectType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(
            type
          )} to be a GraphQL Input Object type.`
        );
      }
      return type;
    }
    function isListType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLList);
    }
    function assertListType(type) {
      if (!isListType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL List type.`
        );
      }
      return type;
    }
    function isNonNullType(type) {
      return (0, _instanceOf.instanceOf)(type, GraphQLNonNull);
    }
    function assertNonNullType(type) {
      if (!isNonNullType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL Non-Null type.`
        );
      }
      return type;
    }
    function isInputType(type) {
      return isScalarType(type) || isEnumType(type) || isInputObjectType(type) || isWrappingType(type) && isInputType(type.ofType);
    }
    function assertInputType(type) {
      if (!isInputType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL input type.`
        );
      }
      return type;
    }
    function isOutputType(type) {
      return isScalarType(type) || isObjectType(type) || isInterfaceType(type) || isUnionType(type) || isEnumType(type) || isWrappingType(type) && isOutputType(type.ofType);
    }
    function assertOutputType(type) {
      if (!isOutputType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL output type.`
        );
      }
      return type;
    }
    function isLeafType(type) {
      return isScalarType(type) || isEnumType(type);
    }
    function assertLeafType(type) {
      if (!isLeafType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL leaf type.`
        );
      }
      return type;
    }
    function isCompositeType(type) {
      return isObjectType(type) || isInterfaceType(type) || isUnionType(type);
    }
    function assertCompositeType(type) {
      if (!isCompositeType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL composite type.`
        );
      }
      return type;
    }
    function isAbstractType(type) {
      return isInterfaceType(type) || isUnionType(type);
    }
    function assertAbstractType(type) {
      if (!isAbstractType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL abstract type.`
        );
      }
      return type;
    }
    var GraphQLList = class {
      constructor(ofType) {
        isType(ofType) || (0, _devAssert.devAssert)(
          false,
          `Expected ${(0, _inspect.inspect)(ofType)} to be a GraphQL type.`
        );
        this.ofType = ofType;
      }
      get [Symbol.toStringTag]() {
        return "GraphQLList";
      }
      toString() {
        return "[" + String(this.ofType) + "]";
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLList = GraphQLList;
    var GraphQLNonNull = class {
      constructor(ofType) {
        isNullableType(ofType) || (0, _devAssert.devAssert)(
          false,
          `Expected ${(0, _inspect.inspect)(
            ofType
          )} to be a GraphQL nullable type.`
        );
        this.ofType = ofType;
      }
      get [Symbol.toStringTag]() {
        return "GraphQLNonNull";
      }
      toString() {
        return String(this.ofType) + "!";
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLNonNull = GraphQLNonNull;
    function isWrappingType(type) {
      return isListType(type) || isNonNullType(type);
    }
    function assertWrappingType(type) {
      if (!isWrappingType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL wrapping type.`
        );
      }
      return type;
    }
    function isNullableType(type) {
      return isType(type) && !isNonNullType(type);
    }
    function assertNullableType(type) {
      if (!isNullableType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL nullable type.`
        );
      }
      return type;
    }
    function getNullableType(type) {
      if (type) {
        return isNonNullType(type) ? type.ofType : type;
      }
    }
    function isNamedType(type) {
      return isScalarType(type) || isObjectType(type) || isInterfaceType(type) || isUnionType(type) || isEnumType(type) || isInputObjectType(type);
    }
    function assertNamedType(type) {
      if (!isNamedType(type)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(type)} to be a GraphQL named type.`
        );
      }
      return type;
    }
    function getNamedType(type) {
      if (type) {
        let unwrappedType = type;
        while (isWrappingType(unwrappedType)) {
          unwrappedType = unwrappedType.ofType;
        }
        return unwrappedType;
      }
    }
    function resolveReadonlyArrayThunk(thunk) {
      return typeof thunk === "function" ? thunk() : thunk;
    }
    function resolveObjMapThunk(thunk) {
      return typeof thunk === "function" ? thunk() : thunk;
    }
    var GraphQLScalarType = class {
      constructor(config) {
        var _config$parseValue, _config$serialize, _config$parseLiteral, _config$extensionASTN;
        const parseValue = (_config$parseValue = config.parseValue) !== null && _config$parseValue !== void 0 ? _config$parseValue : _identityFunc.identityFunc;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.specifiedByURL = config.specifiedByURL;
        this.serialize = (_config$serialize = config.serialize) !== null && _config$serialize !== void 0 ? _config$serialize : _identityFunc.identityFunc;
        this.parseValue = parseValue;
        this.parseLiteral = (_config$parseLiteral = config.parseLiteral) !== null && _config$parseLiteral !== void 0 ? _config$parseLiteral : (node, variables) => parseValue(
          (0, _valueFromASTUntyped.valueFromASTUntyped)(node, variables)
        );
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN = config.extensionASTNodes) !== null && _config$extensionASTN !== void 0 ? _config$extensionASTN : [];
        config.specifiedByURL == null || typeof config.specifiedByURL === "string" || (0, _devAssert.devAssert)(
          false,
          `${this.name} must provide "specifiedByURL" as a string, but got: ${(0, _inspect.inspect)(config.specifiedByURL)}.`
        );
        config.serialize == null || typeof config.serialize === "function" || (0, _devAssert.devAssert)(
          false,
          `${this.name} must provide "serialize" function. If this custom Scalar is also used as an input type, ensure "parseValue" and "parseLiteral" functions are also provided.`
        );
        if (config.parseLiteral) {
          typeof config.parseValue === "function" && typeof config.parseLiteral === "function" || (0, _devAssert.devAssert)(
            false,
            `${this.name} must provide both "parseValue" and "parseLiteral" functions.`
          );
        }
      }
      get [Symbol.toStringTag]() {
        return "GraphQLScalarType";
      }
      toConfig() {
        return {
          name: this.name,
          description: this.description,
          specifiedByURL: this.specifiedByURL,
          serialize: this.serialize,
          parseValue: this.parseValue,
          parseLiteral: this.parseLiteral,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLScalarType = GraphQLScalarType;
    var GraphQLObjectType = class {
      constructor(config) {
        var _config$extensionASTN2;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.isTypeOf = config.isTypeOf;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN2 = config.extensionASTNodes) !== null && _config$extensionASTN2 !== void 0 ? _config$extensionASTN2 : [];
        this._fields = () => defineFieldMap(config);
        this._interfaces = () => defineInterfaces(config);
        config.isTypeOf == null || typeof config.isTypeOf === "function" || (0, _devAssert.devAssert)(
          false,
          `${this.name} must provide "isTypeOf" as a function, but got: ${(0, _inspect.inspect)(config.isTypeOf)}.`
        );
      }
      get [Symbol.toStringTag]() {
        return "GraphQLObjectType";
      }
      getFields() {
        if (typeof this._fields === "function") {
          this._fields = this._fields();
        }
        return this._fields;
      }
      getInterfaces() {
        if (typeof this._interfaces === "function") {
          this._interfaces = this._interfaces();
        }
        return this._interfaces;
      }
      toConfig() {
        return {
          name: this.name,
          description: this.description,
          interfaces: this.getInterfaces(),
          fields: fieldsToFieldsConfig(this.getFields()),
          isTypeOf: this.isTypeOf,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLObjectType = GraphQLObjectType;
    function defineInterfaces(config) {
      var _config$interfaces;
      const interfaces = resolveReadonlyArrayThunk(
        (_config$interfaces = config.interfaces) !== null && _config$interfaces !== void 0 ? _config$interfaces : []
      );
      Array.isArray(interfaces) || (0, _devAssert.devAssert)(
        false,
        `${config.name} interfaces must be an Array or a function which returns an Array.`
      );
      return interfaces;
    }
    function defineFieldMap(config) {
      const fieldMap = resolveObjMapThunk(config.fields);
      isPlainObj(fieldMap) || (0, _devAssert.devAssert)(
        false,
        `${config.name} fields must be an object with field names as keys or a function which returns such an object.`
      );
      return (0, _mapValue.mapValue)(fieldMap, (fieldConfig, fieldName) => {
        var _fieldConfig$args;
        isPlainObj(fieldConfig) || (0, _devAssert.devAssert)(
          false,
          `${config.name}.${fieldName} field config must be an object.`
        );
        fieldConfig.resolve == null || typeof fieldConfig.resolve === "function" || (0, _devAssert.devAssert)(
          false,
          `${config.name}.${fieldName} field resolver must be a function if provided, but got: ${(0, _inspect.inspect)(fieldConfig.resolve)}.`
        );
        const argsConfig = (_fieldConfig$args = fieldConfig.args) !== null && _fieldConfig$args !== void 0 ? _fieldConfig$args : {};
        isPlainObj(argsConfig) || (0, _devAssert.devAssert)(
          false,
          `${config.name}.${fieldName} args must be an object with argument names as keys.`
        );
        return {
          name: (0, _assertName.assertName)(fieldName),
          description: fieldConfig.description,
          type: fieldConfig.type,
          args: defineArguments(argsConfig),
          resolve: fieldConfig.resolve,
          subscribe: fieldConfig.subscribe,
          deprecationReason: fieldConfig.deprecationReason,
          extensions: (0, _toObjMap.toObjMap)(fieldConfig.extensions),
          astNode: fieldConfig.astNode
        };
      });
    }
    function defineArguments(config) {
      return Object.entries(config).map(([argName, argConfig]) => ({
        name: (0, _assertName.assertName)(argName),
        description: argConfig.description,
        type: argConfig.type,
        defaultValue: argConfig.defaultValue,
        deprecationReason: argConfig.deprecationReason,
        extensions: (0, _toObjMap.toObjMap)(argConfig.extensions),
        astNode: argConfig.astNode
      }));
    }
    function isPlainObj(obj) {
      return (0, _isObjectLike.isObjectLike)(obj) && !Array.isArray(obj);
    }
    function fieldsToFieldsConfig(fields) {
      return (0, _mapValue.mapValue)(fields, (field) => ({
        description: field.description,
        type: field.type,
        args: argsToArgsConfig(field.args),
        resolve: field.resolve,
        subscribe: field.subscribe,
        deprecationReason: field.deprecationReason,
        extensions: field.extensions,
        astNode: field.astNode
      }));
    }
    function argsToArgsConfig(args) {
      return (0, _keyValMap.keyValMap)(
        args,
        (arg) => arg.name,
        (arg) => ({
          description: arg.description,
          type: arg.type,
          defaultValue: arg.defaultValue,
          deprecationReason: arg.deprecationReason,
          extensions: arg.extensions,
          astNode: arg.astNode
        })
      );
    }
    function isRequiredArgument(arg) {
      return isNonNullType(arg.type) && arg.defaultValue === void 0;
    }
    var GraphQLInterfaceType = class {
      constructor(config) {
        var _config$extensionASTN3;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.resolveType = config.resolveType;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN3 = config.extensionASTNodes) !== null && _config$extensionASTN3 !== void 0 ? _config$extensionASTN3 : [];
        this._fields = defineFieldMap.bind(void 0, config);
        this._interfaces = defineInterfaces.bind(void 0, config);
        config.resolveType == null || typeof config.resolveType === "function" || (0, _devAssert.devAssert)(
          false,
          `${this.name} must provide "resolveType" as a function, but got: ${(0, _inspect.inspect)(config.resolveType)}.`
        );
      }
      get [Symbol.toStringTag]() {
        return "GraphQLInterfaceType";
      }
      getFields() {
        if (typeof this._fields === "function") {
          this._fields = this._fields();
        }
        return this._fields;
      }
      getInterfaces() {
        if (typeof this._interfaces === "function") {
          this._interfaces = this._interfaces();
        }
        return this._interfaces;
      }
      toConfig() {
        return {
          name: this.name,
          description: this.description,
          interfaces: this.getInterfaces(),
          fields: fieldsToFieldsConfig(this.getFields()),
          resolveType: this.resolveType,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLInterfaceType = GraphQLInterfaceType;
    var GraphQLUnionType = class {
      constructor(config) {
        var _config$extensionASTN4;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.resolveType = config.resolveType;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN4 = config.extensionASTNodes) !== null && _config$extensionASTN4 !== void 0 ? _config$extensionASTN4 : [];
        this._types = defineTypes.bind(void 0, config);
        config.resolveType == null || typeof config.resolveType === "function" || (0, _devAssert.devAssert)(
          false,
          `${this.name} must provide "resolveType" as a function, but got: ${(0, _inspect.inspect)(config.resolveType)}.`
        );
      }
      get [Symbol.toStringTag]() {
        return "GraphQLUnionType";
      }
      getTypes() {
        if (typeof this._types === "function") {
          this._types = this._types();
        }
        return this._types;
      }
      toConfig() {
        return {
          name: this.name,
          description: this.description,
          types: this.getTypes(),
          resolveType: this.resolveType,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLUnionType = GraphQLUnionType;
    function defineTypes(config) {
      const types = resolveReadonlyArrayThunk(config.types);
      Array.isArray(types) || (0, _devAssert.devAssert)(
        false,
        `Must provide Array of types or a function which returns such an array for Union ${config.name}.`
      );
      return types;
    }
    var GraphQLEnumType = class {
      /* <T> */
      constructor(config) {
        var _config$extensionASTN5;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN5 = config.extensionASTNodes) !== null && _config$extensionASTN5 !== void 0 ? _config$extensionASTN5 : [];
        this._values = defineEnumValues(this.name, config.values);
        this._valueLookup = new Map(
          this._values.map((enumValue) => [enumValue.value, enumValue])
        );
        this._nameLookup = (0, _keyMap.keyMap)(this._values, (value) => value.name);
      }
      get [Symbol.toStringTag]() {
        return "GraphQLEnumType";
      }
      getValues() {
        return this._values;
      }
      getValue(name) {
        return this._nameLookup[name];
      }
      serialize(outputValue) {
        const enumValue = this._valueLookup.get(outputValue);
        if (enumValue === void 0) {
          throw new _GraphQLError.GraphQLError(
            `Enum "${this.name}" cannot represent value: ${(0, _inspect.inspect)(
              outputValue
            )}`
          );
        }
        return enumValue.name;
      }
      parseValue(inputValue) {
        if (typeof inputValue !== "string") {
          const valueStr = (0, _inspect.inspect)(inputValue);
          throw new _GraphQLError.GraphQLError(
            `Enum "${this.name}" cannot represent non-string value: ${valueStr}.` + didYouMeanEnumValue(this, valueStr)
          );
        }
        const enumValue = this.getValue(inputValue);
        if (enumValue == null) {
          throw new _GraphQLError.GraphQLError(
            `Value "${inputValue}" does not exist in "${this.name}" enum.` + didYouMeanEnumValue(this, inputValue)
          );
        }
        return enumValue.value;
      }
      parseLiteral(valueNode, _variables) {
        if (valueNode.kind !== _kinds.Kind.ENUM) {
          const valueStr = (0, _printer.print)(valueNode);
          throw new _GraphQLError.GraphQLError(
            `Enum "${this.name}" cannot represent non-enum value: ${valueStr}.` + didYouMeanEnumValue(this, valueStr),
            {
              nodes: valueNode
            }
          );
        }
        const enumValue = this.getValue(valueNode.value);
        if (enumValue == null) {
          const valueStr = (0, _printer.print)(valueNode);
          throw new _GraphQLError.GraphQLError(
            `Value "${valueStr}" does not exist in "${this.name}" enum.` + didYouMeanEnumValue(this, valueStr),
            {
              nodes: valueNode
            }
          );
        }
        return enumValue.value;
      }
      toConfig() {
        const values = (0, _keyValMap.keyValMap)(
          this.getValues(),
          (value) => value.name,
          (value) => ({
            description: value.description,
            value: value.value,
            deprecationReason: value.deprecationReason,
            extensions: value.extensions,
            astNode: value.astNode
          })
        );
        return {
          name: this.name,
          description: this.description,
          values,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLEnumType = GraphQLEnumType;
    function didYouMeanEnumValue(enumType, unknownValueStr) {
      const allNames = enumType.getValues().map((value) => value.name);
      const suggestedValues = (0, _suggestionList.suggestionList)(
        unknownValueStr,
        allNames
      );
      return (0, _didYouMean.didYouMean)("the enum value", suggestedValues);
    }
    function defineEnumValues(typeName, valueMap) {
      isPlainObj(valueMap) || (0, _devAssert.devAssert)(
        false,
        `${typeName} values must be an object with value names as keys.`
      );
      return Object.entries(valueMap).map(([valueName, valueConfig]) => {
        isPlainObj(valueConfig) || (0, _devAssert.devAssert)(
          false,
          `${typeName}.${valueName} must refer to an object with a "value" key representing an internal value but got: ${(0, _inspect.inspect)(
            valueConfig
          )}.`
        );
        return {
          name: (0, _assertName.assertEnumValueName)(valueName),
          description: valueConfig.description,
          value: valueConfig.value !== void 0 ? valueConfig.value : valueName,
          deprecationReason: valueConfig.deprecationReason,
          extensions: (0, _toObjMap.toObjMap)(valueConfig.extensions),
          astNode: valueConfig.astNode
        };
      });
    }
    var GraphQLInputObjectType = class {
      constructor(config) {
        var _config$extensionASTN6;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN6 = config.extensionASTNodes) !== null && _config$extensionASTN6 !== void 0 ? _config$extensionASTN6 : [];
        this._fields = defineInputFieldMap.bind(void 0, config);
      }
      get [Symbol.toStringTag]() {
        return "GraphQLInputObjectType";
      }
      getFields() {
        if (typeof this._fields === "function") {
          this._fields = this._fields();
        }
        return this._fields;
      }
      toConfig() {
        const fields = (0, _mapValue.mapValue)(this.getFields(), (field) => ({
          description: field.description,
          type: field.type,
          defaultValue: field.defaultValue,
          deprecationReason: field.deprecationReason,
          extensions: field.extensions,
          astNode: field.astNode
        }));
        return {
          name: this.name,
          description: this.description,
          fields,
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes
        };
      }
      toString() {
        return this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLInputObjectType = GraphQLInputObjectType;
    function defineInputFieldMap(config) {
      const fieldMap = resolveObjMapThunk(config.fields);
      isPlainObj(fieldMap) || (0, _devAssert.devAssert)(
        false,
        `${config.name} fields must be an object with field names as keys or a function which returns such an object.`
      );
      return (0, _mapValue.mapValue)(fieldMap, (fieldConfig, fieldName) => {
        !("resolve" in fieldConfig) || (0, _devAssert.devAssert)(
          false,
          `${config.name}.${fieldName} field has a resolve property, but Input Types cannot define resolvers.`
        );
        return {
          name: (0, _assertName.assertName)(fieldName),
          description: fieldConfig.description,
          type: fieldConfig.type,
          defaultValue: fieldConfig.defaultValue,
          deprecationReason: fieldConfig.deprecationReason,
          extensions: (0, _toObjMap.toObjMap)(fieldConfig.extensions),
          astNode: fieldConfig.astNode
        };
      });
    }
    function isRequiredInputField(field) {
      return isNonNullType(field.type) && field.defaultValue === void 0;
    }
  }
});

// node_modules/graphql/utilities/typeComparators.js
var require_typeComparators = __commonJS({
  "node_modules/graphql/utilities/typeComparators.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.doTypesOverlap = doTypesOverlap;
    exports2.isEqualType = isEqualType;
    exports2.isTypeSubTypeOf = isTypeSubTypeOf;
    var _definition = require_definition();
    function isEqualType(typeA, typeB) {
      if (typeA === typeB) {
        return true;
      }
      if ((0, _definition.isNonNullType)(typeA) && (0, _definition.isNonNullType)(typeB)) {
        return isEqualType(typeA.ofType, typeB.ofType);
      }
      if ((0, _definition.isListType)(typeA) && (0, _definition.isListType)(typeB)) {
        return isEqualType(typeA.ofType, typeB.ofType);
      }
      return false;
    }
    function isTypeSubTypeOf(schema, maybeSubType, superType) {
      if (maybeSubType === superType) {
        return true;
      }
      if ((0, _definition.isNonNullType)(superType)) {
        if ((0, _definition.isNonNullType)(maybeSubType)) {
          return isTypeSubTypeOf(schema, maybeSubType.ofType, superType.ofType);
        }
        return false;
      }
      if ((0, _definition.isNonNullType)(maybeSubType)) {
        return isTypeSubTypeOf(schema, maybeSubType.ofType, superType);
      }
      if ((0, _definition.isListType)(superType)) {
        if ((0, _definition.isListType)(maybeSubType)) {
          return isTypeSubTypeOf(schema, maybeSubType.ofType, superType.ofType);
        }
        return false;
      }
      if ((0, _definition.isListType)(maybeSubType)) {
        return false;
      }
      return (0, _definition.isAbstractType)(superType) && ((0, _definition.isInterfaceType)(maybeSubType) || (0, _definition.isObjectType)(maybeSubType)) && schema.isSubType(superType, maybeSubType);
    }
    function doTypesOverlap(schema, typeA, typeB) {
      if (typeA === typeB) {
        return true;
      }
      if ((0, _definition.isAbstractType)(typeA)) {
        if ((0, _definition.isAbstractType)(typeB)) {
          return schema.getPossibleTypes(typeA).some((type) => schema.isSubType(typeB, type));
        }
        return schema.isSubType(typeA, typeB);
      }
      if ((0, _definition.isAbstractType)(typeB)) {
        return schema.isSubType(typeB, typeA);
      }
      return false;
    }
  }
});

// node_modules/graphql/type/scalars.js
var require_scalars = __commonJS({
  "node_modules/graphql/type/scalars.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.GraphQLString = exports2.GraphQLInt = exports2.GraphQLID = exports2.GraphQLFloat = exports2.GraphQLBoolean = exports2.GRAPHQL_MIN_INT = exports2.GRAPHQL_MAX_INT = void 0;
    exports2.isSpecifiedScalarType = isSpecifiedScalarType;
    exports2.specifiedScalarTypes = void 0;
    var _inspect = require_inspect();
    var _isObjectLike = require_isObjectLike();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _definition = require_definition();
    var GRAPHQL_MAX_INT = 2147483647;
    exports2.GRAPHQL_MAX_INT = GRAPHQL_MAX_INT;
    var GRAPHQL_MIN_INT = -2147483648;
    exports2.GRAPHQL_MIN_INT = GRAPHQL_MIN_INT;
    var GraphQLInt = new _definition.GraphQLScalarType({
      name: "Int",
      description: "The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.",
      serialize(outputValue) {
        const coercedValue = serializeObject(outputValue);
        if (typeof coercedValue === "boolean") {
          return coercedValue ? 1 : 0;
        }
        let num = coercedValue;
        if (typeof coercedValue === "string" && coercedValue !== "") {
          num = Number(coercedValue);
        }
        if (typeof num !== "number" || !Number.isInteger(num)) {
          throw new _GraphQLError.GraphQLError(
            `Int cannot represent non-integer value: ${(0, _inspect.inspect)(
              coercedValue
            )}`
          );
        }
        if (num > GRAPHQL_MAX_INT || num < GRAPHQL_MIN_INT) {
          throw new _GraphQLError.GraphQLError(
            "Int cannot represent non 32-bit signed integer value: " + (0, _inspect.inspect)(coercedValue)
          );
        }
        return num;
      },
      parseValue(inputValue) {
        if (typeof inputValue !== "number" || !Number.isInteger(inputValue)) {
          throw new _GraphQLError.GraphQLError(
            `Int cannot represent non-integer value: ${(0, _inspect.inspect)(
              inputValue
            )}`
          );
        }
        if (inputValue > GRAPHQL_MAX_INT || inputValue < GRAPHQL_MIN_INT) {
          throw new _GraphQLError.GraphQLError(
            `Int cannot represent non 32-bit signed integer value: ${inputValue}`
          );
        }
        return inputValue;
      },
      parseLiteral(valueNode) {
        if (valueNode.kind !== _kinds.Kind.INT) {
          throw new _GraphQLError.GraphQLError(
            `Int cannot represent non-integer value: ${(0, _printer.print)(
              valueNode
            )}`,
            {
              nodes: valueNode
            }
          );
        }
        const num = parseInt(valueNode.value, 10);
        if (num > GRAPHQL_MAX_INT || num < GRAPHQL_MIN_INT) {
          throw new _GraphQLError.GraphQLError(
            `Int cannot represent non 32-bit signed integer value: ${valueNode.value}`,
            {
              nodes: valueNode
            }
          );
        }
        return num;
      }
    });
    exports2.GraphQLInt = GraphQLInt;
    var GraphQLFloat = new _definition.GraphQLScalarType({
      name: "Float",
      description: "The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).",
      serialize(outputValue) {
        const coercedValue = serializeObject(outputValue);
        if (typeof coercedValue === "boolean") {
          return coercedValue ? 1 : 0;
        }
        let num = coercedValue;
        if (typeof coercedValue === "string" && coercedValue !== "") {
          num = Number(coercedValue);
        }
        if (typeof num !== "number" || !Number.isFinite(num)) {
          throw new _GraphQLError.GraphQLError(
            `Float cannot represent non numeric value: ${(0, _inspect.inspect)(
              coercedValue
            )}`
          );
        }
        return num;
      },
      parseValue(inputValue) {
        if (typeof inputValue !== "number" || !Number.isFinite(inputValue)) {
          throw new _GraphQLError.GraphQLError(
            `Float cannot represent non numeric value: ${(0, _inspect.inspect)(
              inputValue
            )}`
          );
        }
        return inputValue;
      },
      parseLiteral(valueNode) {
        if (valueNode.kind !== _kinds.Kind.FLOAT && valueNode.kind !== _kinds.Kind.INT) {
          throw new _GraphQLError.GraphQLError(
            `Float cannot represent non numeric value: ${(0, _printer.print)(
              valueNode
            )}`,
            valueNode
          );
        }
        return parseFloat(valueNode.value);
      }
    });
    exports2.GraphQLFloat = GraphQLFloat;
    var GraphQLString = new _definition.GraphQLScalarType({
      name: "String",
      description: "The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.",
      serialize(outputValue) {
        const coercedValue = serializeObject(outputValue);
        if (typeof coercedValue === "string") {
          return coercedValue;
        }
        if (typeof coercedValue === "boolean") {
          return coercedValue ? "true" : "false";
        }
        if (typeof coercedValue === "number" && Number.isFinite(coercedValue)) {
          return coercedValue.toString();
        }
        throw new _GraphQLError.GraphQLError(
          `String cannot represent value: ${(0, _inspect.inspect)(outputValue)}`
        );
      },
      parseValue(inputValue) {
        if (typeof inputValue !== "string") {
          throw new _GraphQLError.GraphQLError(
            `String cannot represent a non string value: ${(0, _inspect.inspect)(
              inputValue
            )}`
          );
        }
        return inputValue;
      },
      parseLiteral(valueNode) {
        if (valueNode.kind !== _kinds.Kind.STRING) {
          throw new _GraphQLError.GraphQLError(
            `String cannot represent a non string value: ${(0, _printer.print)(
              valueNode
            )}`,
            {
              nodes: valueNode
            }
          );
        }
        return valueNode.value;
      }
    });
    exports2.GraphQLString = GraphQLString;
    var GraphQLBoolean = new _definition.GraphQLScalarType({
      name: "Boolean",
      description: "The `Boolean` scalar type represents `true` or `false`.",
      serialize(outputValue) {
        const coercedValue = serializeObject(outputValue);
        if (typeof coercedValue === "boolean") {
          return coercedValue;
        }
        if (Number.isFinite(coercedValue)) {
          return coercedValue !== 0;
        }
        throw new _GraphQLError.GraphQLError(
          `Boolean cannot represent a non boolean value: ${(0, _inspect.inspect)(
            coercedValue
          )}`
        );
      },
      parseValue(inputValue) {
        if (typeof inputValue !== "boolean") {
          throw new _GraphQLError.GraphQLError(
            `Boolean cannot represent a non boolean value: ${(0, _inspect.inspect)(
              inputValue
            )}`
          );
        }
        return inputValue;
      },
      parseLiteral(valueNode) {
        if (valueNode.kind !== _kinds.Kind.BOOLEAN) {
          throw new _GraphQLError.GraphQLError(
            `Boolean cannot represent a non boolean value: ${(0, _printer.print)(
              valueNode
            )}`,
            {
              nodes: valueNode
            }
          );
        }
        return valueNode.value;
      }
    });
    exports2.GraphQLBoolean = GraphQLBoolean;
    var GraphQLID = new _definition.GraphQLScalarType({
      name: "ID",
      description: 'The `ID` scalar type represents a unique identifier, often used to refetch an object or as key for a cache. The ID type appears in a JSON response as a String; however, it is not intended to be human-readable. When expected as an input type, any string (such as `"4"`) or integer (such as `4`) input value will be accepted as an ID.',
      serialize(outputValue) {
        const coercedValue = serializeObject(outputValue);
        if (typeof coercedValue === "string") {
          return coercedValue;
        }
        if (Number.isInteger(coercedValue)) {
          return String(coercedValue);
        }
        throw new _GraphQLError.GraphQLError(
          `ID cannot represent value: ${(0, _inspect.inspect)(outputValue)}`
        );
      },
      parseValue(inputValue) {
        if (typeof inputValue === "string") {
          return inputValue;
        }
        if (typeof inputValue === "number" && Number.isInteger(inputValue)) {
          return inputValue.toString();
        }
        throw new _GraphQLError.GraphQLError(
          `ID cannot represent value: ${(0, _inspect.inspect)(inputValue)}`
        );
      },
      parseLiteral(valueNode) {
        if (valueNode.kind !== _kinds.Kind.STRING && valueNode.kind !== _kinds.Kind.INT) {
          throw new _GraphQLError.GraphQLError(
            "ID cannot represent a non-string and non-integer value: " + (0, _printer.print)(valueNode),
            {
              nodes: valueNode
            }
          );
        }
        return valueNode.value;
      }
    });
    exports2.GraphQLID = GraphQLID;
    var specifiedScalarTypes = Object.freeze([
      GraphQLString,
      GraphQLInt,
      GraphQLFloat,
      GraphQLBoolean,
      GraphQLID
    ]);
    exports2.specifiedScalarTypes = specifiedScalarTypes;
    function isSpecifiedScalarType(type) {
      return specifiedScalarTypes.some(({ name }) => type.name === name);
    }
    function serializeObject(outputValue) {
      if ((0, _isObjectLike.isObjectLike)(outputValue)) {
        if (typeof outputValue.valueOf === "function") {
          const valueOfResult = outputValue.valueOf();
          if (!(0, _isObjectLike.isObjectLike)(valueOfResult)) {
            return valueOfResult;
          }
        }
        if (typeof outputValue.toJSON === "function") {
          return outputValue.toJSON();
        }
      }
      return outputValue;
    }
  }
});

// node_modules/graphql/type/directives.js
var require_directives = __commonJS({
  "node_modules/graphql/type/directives.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.GraphQLSpecifiedByDirective = exports2.GraphQLSkipDirective = exports2.GraphQLIncludeDirective = exports2.GraphQLDirective = exports2.GraphQLDeprecatedDirective = exports2.DEFAULT_DEPRECATION_REASON = void 0;
    exports2.assertDirective = assertDirective;
    exports2.isDirective = isDirective;
    exports2.isSpecifiedDirective = isSpecifiedDirective;
    exports2.specifiedDirectives = void 0;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _instanceOf = require_instanceOf();
    var _isObjectLike = require_isObjectLike();
    var _toObjMap = require_toObjMap();
    var _directiveLocation = require_directiveLocation();
    var _assertName = require_assertName();
    var _definition = require_definition();
    var _scalars = require_scalars();
    function isDirective(directive) {
      return (0, _instanceOf.instanceOf)(directive, GraphQLDirective);
    }
    function assertDirective(directive) {
      if (!isDirective(directive)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(directive)} to be a GraphQL directive.`
        );
      }
      return directive;
    }
    var GraphQLDirective = class {
      constructor(config) {
        var _config$isRepeatable, _config$args;
        this.name = (0, _assertName.assertName)(config.name);
        this.description = config.description;
        this.locations = config.locations;
        this.isRepeatable = (_config$isRepeatable = config.isRepeatable) !== null && _config$isRepeatable !== void 0 ? _config$isRepeatable : false;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        Array.isArray(config.locations) || (0, _devAssert.devAssert)(
          false,
          `@${config.name} locations must be an Array.`
        );
        const args = (_config$args = config.args) !== null && _config$args !== void 0 ? _config$args : {};
        (0, _isObjectLike.isObjectLike)(args) && !Array.isArray(args) || (0, _devAssert.devAssert)(
          false,
          `@${config.name} args must be an object with argument names as keys.`
        );
        this.args = (0, _definition.defineArguments)(args);
      }
      get [Symbol.toStringTag]() {
        return "GraphQLDirective";
      }
      toConfig() {
        return {
          name: this.name,
          description: this.description,
          locations: this.locations,
          args: (0, _definition.argsToArgsConfig)(this.args),
          isRepeatable: this.isRepeatable,
          extensions: this.extensions,
          astNode: this.astNode
        };
      }
      toString() {
        return "@" + this.name;
      }
      toJSON() {
        return this.toString();
      }
    };
    exports2.GraphQLDirective = GraphQLDirective;
    var GraphQLIncludeDirective = new GraphQLDirective({
      name: "include",
      description: "Directs the executor to include this field or fragment only when the `if` argument is true.",
      locations: [
        _directiveLocation.DirectiveLocation.FIELD,
        _directiveLocation.DirectiveLocation.FRAGMENT_SPREAD,
        _directiveLocation.DirectiveLocation.INLINE_FRAGMENT
      ],
      args: {
        if: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          description: "Included when true."
        }
      }
    });
    exports2.GraphQLIncludeDirective = GraphQLIncludeDirective;
    var GraphQLSkipDirective = new GraphQLDirective({
      name: "skip",
      description: "Directs the executor to skip this field or fragment when the `if` argument is true.",
      locations: [
        _directiveLocation.DirectiveLocation.FIELD,
        _directiveLocation.DirectiveLocation.FRAGMENT_SPREAD,
        _directiveLocation.DirectiveLocation.INLINE_FRAGMENT
      ],
      args: {
        if: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          description: "Skipped when true."
        }
      }
    });
    exports2.GraphQLSkipDirective = GraphQLSkipDirective;
    var DEFAULT_DEPRECATION_REASON = "No longer supported";
    exports2.DEFAULT_DEPRECATION_REASON = DEFAULT_DEPRECATION_REASON;
    var GraphQLDeprecatedDirective = new GraphQLDirective({
      name: "deprecated",
      description: "Marks an element of a GraphQL schema as no longer supported.",
      locations: [
        _directiveLocation.DirectiveLocation.FIELD_DEFINITION,
        _directiveLocation.DirectiveLocation.ARGUMENT_DEFINITION,
        _directiveLocation.DirectiveLocation.INPUT_FIELD_DEFINITION,
        _directiveLocation.DirectiveLocation.ENUM_VALUE
      ],
      args: {
        reason: {
          type: _scalars.GraphQLString,
          description: "Explains why this element was deprecated, usually also including a suggestion for how to access supported similar data. Formatted using the Markdown syntax, as specified by [CommonMark](https://commonmark.org/).",
          defaultValue: DEFAULT_DEPRECATION_REASON
        }
      }
    });
    exports2.GraphQLDeprecatedDirective = GraphQLDeprecatedDirective;
    var GraphQLSpecifiedByDirective = new GraphQLDirective({
      name: "specifiedBy",
      description: "Exposes a URL that specifies the behavior of this scalar.",
      locations: [_directiveLocation.DirectiveLocation.SCALAR],
      args: {
        url: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          description: "The URL that specifies the behavior of this scalar."
        }
      }
    });
    exports2.GraphQLSpecifiedByDirective = GraphQLSpecifiedByDirective;
    var specifiedDirectives = Object.freeze([
      GraphQLIncludeDirective,
      GraphQLSkipDirective,
      GraphQLDeprecatedDirective,
      GraphQLSpecifiedByDirective
    ]);
    exports2.specifiedDirectives = specifiedDirectives;
    function isSpecifiedDirective(directive) {
      return specifiedDirectives.some(({ name }) => name === directive.name);
    }
  }
});

// node_modules/graphql/jsutils/isIterableObject.js
var require_isIterableObject = __commonJS({
  "node_modules/graphql/jsutils/isIterableObject.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isIterableObject = isIterableObject;
    function isIterableObject(maybeIterable) {
      return typeof maybeIterable === "object" && typeof (maybeIterable === null || maybeIterable === void 0 ? void 0 : maybeIterable[Symbol.iterator]) === "function";
    }
  }
});

// node_modules/graphql/utilities/astFromValue.js
var require_astFromValue = __commonJS({
  "node_modules/graphql/utilities/astFromValue.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.astFromValue = astFromValue;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _isIterableObject = require_isIterableObject();
    var _isObjectLike = require_isObjectLike();
    var _kinds = require_kinds();
    var _definition = require_definition();
    var _scalars = require_scalars();
    function astFromValue(value, type) {
      if ((0, _definition.isNonNullType)(type)) {
        const astValue = astFromValue(value, type.ofType);
        if ((astValue === null || astValue === void 0 ? void 0 : astValue.kind) === _kinds.Kind.NULL) {
          return null;
        }
        return astValue;
      }
      if (value === null) {
        return {
          kind: _kinds.Kind.NULL
        };
      }
      if (value === void 0) {
        return null;
      }
      if ((0, _definition.isListType)(type)) {
        const itemType = type.ofType;
        if ((0, _isIterableObject.isIterableObject)(value)) {
          const valuesNodes = [];
          for (const item of value) {
            const itemNode = astFromValue(item, itemType);
            if (itemNode != null) {
              valuesNodes.push(itemNode);
            }
          }
          return {
            kind: _kinds.Kind.LIST,
            values: valuesNodes
          };
        }
        return astFromValue(value, itemType);
      }
      if ((0, _definition.isInputObjectType)(type)) {
        if (!(0, _isObjectLike.isObjectLike)(value)) {
          return null;
        }
        const fieldNodes = [];
        for (const field of Object.values(type.getFields())) {
          const fieldValue = astFromValue(value[field.name], field.type);
          if (fieldValue) {
            fieldNodes.push({
              kind: _kinds.Kind.OBJECT_FIELD,
              name: {
                kind: _kinds.Kind.NAME,
                value: field.name
              },
              value: fieldValue
            });
          }
        }
        return {
          kind: _kinds.Kind.OBJECT,
          fields: fieldNodes
        };
      }
      if ((0, _definition.isLeafType)(type)) {
        const serialized = type.serialize(value);
        if (serialized == null) {
          return null;
        }
        if (typeof serialized === "boolean") {
          return {
            kind: _kinds.Kind.BOOLEAN,
            value: serialized
          };
        }
        if (typeof serialized === "number" && Number.isFinite(serialized)) {
          const stringNum = String(serialized);
          return integerStringRegExp.test(stringNum) ? {
            kind: _kinds.Kind.INT,
            value: stringNum
          } : {
            kind: _kinds.Kind.FLOAT,
            value: stringNum
          };
        }
        if (typeof serialized === "string") {
          if ((0, _definition.isEnumType)(type)) {
            return {
              kind: _kinds.Kind.ENUM,
              value: serialized
            };
          }
          if (type === _scalars.GraphQLID && integerStringRegExp.test(serialized)) {
            return {
              kind: _kinds.Kind.INT,
              value: serialized
            };
          }
          return {
            kind: _kinds.Kind.STRING,
            value: serialized
          };
        }
        throw new TypeError(
          `Cannot convert value to AST: ${(0, _inspect.inspect)(serialized)}.`
        );
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected input type: " + (0, _inspect.inspect)(type)
      );
    }
    var integerStringRegExp = /^-?(?:0|[1-9][0-9]*)$/;
  }
});

// node_modules/graphql/type/introspection.js
var require_introspection = __commonJS({
  "node_modules/graphql/type/introspection.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.introspectionTypes = exports2.__TypeKind = exports2.__Type = exports2.__Schema = exports2.__InputValue = exports2.__Field = exports2.__EnumValue = exports2.__DirectiveLocation = exports2.__Directive = exports2.TypeNameMetaFieldDef = exports2.TypeMetaFieldDef = exports2.TypeKind = exports2.SchemaMetaFieldDef = void 0;
    exports2.isIntrospectionType = isIntrospectionType;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _directiveLocation = require_directiveLocation();
    var _printer = require_printer();
    var _astFromValue = require_astFromValue();
    var _definition = require_definition();
    var _scalars = require_scalars();
    var __Schema = new _definition.GraphQLObjectType({
      name: "__Schema",
      description: "A GraphQL Schema defines the capabilities of a GraphQL server. It exposes all available types and directives on the server, as well as the entry points for query, mutation, and subscription operations.",
      fields: () => ({
        description: {
          type: _scalars.GraphQLString,
          resolve: (schema) => schema.description
        },
        types: {
          description: "A list of all types supported by this server.",
          type: new _definition.GraphQLNonNull(
            new _definition.GraphQLList(new _definition.GraphQLNonNull(__Type))
          ),
          resolve(schema) {
            return Object.values(schema.getTypeMap());
          }
        },
        queryType: {
          description: "The type that query operations will be rooted at.",
          type: new _definition.GraphQLNonNull(__Type),
          resolve: (schema) => schema.getQueryType()
        },
        mutationType: {
          description: "If this server supports mutation, the type that mutation operations will be rooted at.",
          type: __Type,
          resolve: (schema) => schema.getMutationType()
        },
        subscriptionType: {
          description: "If this server support subscription, the type that subscription operations will be rooted at.",
          type: __Type,
          resolve: (schema) => schema.getSubscriptionType()
        },
        directives: {
          description: "A list of all directives supported by this server.",
          type: new _definition.GraphQLNonNull(
            new _definition.GraphQLList(
              new _definition.GraphQLNonNull(__Directive)
            )
          ),
          resolve: (schema) => schema.getDirectives()
        }
      })
    });
    exports2.__Schema = __Schema;
    var __Directive = new _definition.GraphQLObjectType({
      name: "__Directive",
      description: "A Directive provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.\n\nIn some cases, you need to provide options to alter GraphQL's execution behavior in ways field arguments will not suffice, such as conditionally including or skipping a field. Directives provide this by describing additional information to the executor.",
      fields: () => ({
        name: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          resolve: (directive) => directive.name
        },
        description: {
          type: _scalars.GraphQLString,
          resolve: (directive) => directive.description
        },
        isRepeatable: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          resolve: (directive) => directive.isRepeatable
        },
        locations: {
          type: new _definition.GraphQLNonNull(
            new _definition.GraphQLList(
              new _definition.GraphQLNonNull(__DirectiveLocation)
            )
          ),
          resolve: (directive) => directive.locations
        },
        args: {
          type: new _definition.GraphQLNonNull(
            new _definition.GraphQLList(
              new _definition.GraphQLNonNull(__InputValue)
            )
          ),
          args: {
            includeDeprecated: {
              type: _scalars.GraphQLBoolean,
              defaultValue: false
            }
          },
          resolve(field, { includeDeprecated }) {
            return includeDeprecated ? field.args : field.args.filter((arg) => arg.deprecationReason == null);
          }
        }
      })
    });
    exports2.__Directive = __Directive;
    var __DirectiveLocation = new _definition.GraphQLEnumType({
      name: "__DirectiveLocation",
      description: "A Directive can be adjacent to many parts of the GraphQL language, a __DirectiveLocation describes one such possible adjacencies.",
      values: {
        QUERY: {
          value: _directiveLocation.DirectiveLocation.QUERY,
          description: "Location adjacent to a query operation."
        },
        MUTATION: {
          value: _directiveLocation.DirectiveLocation.MUTATION,
          description: "Location adjacent to a mutation operation."
        },
        SUBSCRIPTION: {
          value: _directiveLocation.DirectiveLocation.SUBSCRIPTION,
          description: "Location adjacent to a subscription operation."
        },
        FIELD: {
          value: _directiveLocation.DirectiveLocation.FIELD,
          description: "Location adjacent to a field."
        },
        FRAGMENT_DEFINITION: {
          value: _directiveLocation.DirectiveLocation.FRAGMENT_DEFINITION,
          description: "Location adjacent to a fragment definition."
        },
        FRAGMENT_SPREAD: {
          value: _directiveLocation.DirectiveLocation.FRAGMENT_SPREAD,
          description: "Location adjacent to a fragment spread."
        },
        INLINE_FRAGMENT: {
          value: _directiveLocation.DirectiveLocation.INLINE_FRAGMENT,
          description: "Location adjacent to an inline fragment."
        },
        VARIABLE_DEFINITION: {
          value: _directiveLocation.DirectiveLocation.VARIABLE_DEFINITION,
          description: "Location adjacent to a variable definition."
        },
        SCHEMA: {
          value: _directiveLocation.DirectiveLocation.SCHEMA,
          description: "Location adjacent to a schema definition."
        },
        SCALAR: {
          value: _directiveLocation.DirectiveLocation.SCALAR,
          description: "Location adjacent to a scalar definition."
        },
        OBJECT: {
          value: _directiveLocation.DirectiveLocation.OBJECT,
          description: "Location adjacent to an object type definition."
        },
        FIELD_DEFINITION: {
          value: _directiveLocation.DirectiveLocation.FIELD_DEFINITION,
          description: "Location adjacent to a field definition."
        },
        ARGUMENT_DEFINITION: {
          value: _directiveLocation.DirectiveLocation.ARGUMENT_DEFINITION,
          description: "Location adjacent to an argument definition."
        },
        INTERFACE: {
          value: _directiveLocation.DirectiveLocation.INTERFACE,
          description: "Location adjacent to an interface definition."
        },
        UNION: {
          value: _directiveLocation.DirectiveLocation.UNION,
          description: "Location adjacent to a union definition."
        },
        ENUM: {
          value: _directiveLocation.DirectiveLocation.ENUM,
          description: "Location adjacent to an enum definition."
        },
        ENUM_VALUE: {
          value: _directiveLocation.DirectiveLocation.ENUM_VALUE,
          description: "Location adjacent to an enum value definition."
        },
        INPUT_OBJECT: {
          value: _directiveLocation.DirectiveLocation.INPUT_OBJECT,
          description: "Location adjacent to an input object type definition."
        },
        INPUT_FIELD_DEFINITION: {
          value: _directiveLocation.DirectiveLocation.INPUT_FIELD_DEFINITION,
          description: "Location adjacent to an input object field definition."
        }
      }
    });
    exports2.__DirectiveLocation = __DirectiveLocation;
    var __Type = new _definition.GraphQLObjectType({
      name: "__Type",
      description: "The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.\n\nDepending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name, description and optional `specifiedByURL`, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.",
      fields: () => ({
        kind: {
          type: new _definition.GraphQLNonNull(__TypeKind),
          resolve(type) {
            if ((0, _definition.isScalarType)(type)) {
              return TypeKind.SCALAR;
            }
            if ((0, _definition.isObjectType)(type)) {
              return TypeKind.OBJECT;
            }
            if ((0, _definition.isInterfaceType)(type)) {
              return TypeKind.INTERFACE;
            }
            if ((0, _definition.isUnionType)(type)) {
              return TypeKind.UNION;
            }
            if ((0, _definition.isEnumType)(type)) {
              return TypeKind.ENUM;
            }
            if ((0, _definition.isInputObjectType)(type)) {
              return TypeKind.INPUT_OBJECT;
            }
            if ((0, _definition.isListType)(type)) {
              return TypeKind.LIST;
            }
            if ((0, _definition.isNonNullType)(type)) {
              return TypeKind.NON_NULL;
            }
            (0, _invariant.invariant)(
              false,
              `Unexpected type: "${(0, _inspect.inspect)(type)}".`
            );
          }
        },
        name: {
          type: _scalars.GraphQLString,
          resolve: (type) => "name" in type ? type.name : void 0
        },
        description: {
          type: _scalars.GraphQLString,
          resolve: (type) => (
            /* c8 ignore next */
            "description" in type ? type.description : void 0
          )
        },
        specifiedByURL: {
          type: _scalars.GraphQLString,
          resolve: (obj) => "specifiedByURL" in obj ? obj.specifiedByURL : void 0
        },
        fields: {
          type: new _definition.GraphQLList(
            new _definition.GraphQLNonNull(__Field)
          ),
          args: {
            includeDeprecated: {
              type: _scalars.GraphQLBoolean,
              defaultValue: false
            }
          },
          resolve(type, { includeDeprecated }) {
            if ((0, _definition.isObjectType)(type) || (0, _definition.isInterfaceType)(type)) {
              const fields = Object.values(type.getFields());
              return includeDeprecated ? fields : fields.filter((field) => field.deprecationReason == null);
            }
          }
        },
        interfaces: {
          type: new _definition.GraphQLList(new _definition.GraphQLNonNull(__Type)),
          resolve(type) {
            if ((0, _definition.isObjectType)(type) || (0, _definition.isInterfaceType)(type)) {
              return type.getInterfaces();
            }
          }
        },
        possibleTypes: {
          type: new _definition.GraphQLList(new _definition.GraphQLNonNull(__Type)),
          resolve(type, _args, _context, { schema }) {
            if ((0, _definition.isAbstractType)(type)) {
              return schema.getPossibleTypes(type);
            }
          }
        },
        enumValues: {
          type: new _definition.GraphQLList(
            new _definition.GraphQLNonNull(__EnumValue)
          ),
          args: {
            includeDeprecated: {
              type: _scalars.GraphQLBoolean,
              defaultValue: false
            }
          },
          resolve(type, { includeDeprecated }) {
            if ((0, _definition.isEnumType)(type)) {
              const values = type.getValues();
              return includeDeprecated ? values : values.filter((field) => field.deprecationReason == null);
            }
          }
        },
        inputFields: {
          type: new _definition.GraphQLList(
            new _definition.GraphQLNonNull(__InputValue)
          ),
          args: {
            includeDeprecated: {
              type: _scalars.GraphQLBoolean,
              defaultValue: false
            }
          },
          resolve(type, { includeDeprecated }) {
            if ((0, _definition.isInputObjectType)(type)) {
              const values = Object.values(type.getFields());
              return includeDeprecated ? values : values.filter((field) => field.deprecationReason == null);
            }
          }
        },
        ofType: {
          type: __Type,
          resolve: (type) => "ofType" in type ? type.ofType : void 0
        }
      })
    });
    exports2.__Type = __Type;
    var __Field = new _definition.GraphQLObjectType({
      name: "__Field",
      description: "Object and Interface types are described by a list of Fields, each of which has a name, potentially a list of arguments, and a return type.",
      fields: () => ({
        name: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          resolve: (field) => field.name
        },
        description: {
          type: _scalars.GraphQLString,
          resolve: (field) => field.description
        },
        args: {
          type: new _definition.GraphQLNonNull(
            new _definition.GraphQLList(
              new _definition.GraphQLNonNull(__InputValue)
            )
          ),
          args: {
            includeDeprecated: {
              type: _scalars.GraphQLBoolean,
              defaultValue: false
            }
          },
          resolve(field, { includeDeprecated }) {
            return includeDeprecated ? field.args : field.args.filter((arg) => arg.deprecationReason == null);
          }
        },
        type: {
          type: new _definition.GraphQLNonNull(__Type),
          resolve: (field) => field.type
        },
        isDeprecated: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          resolve: (field) => field.deprecationReason != null
        },
        deprecationReason: {
          type: _scalars.GraphQLString,
          resolve: (field) => field.deprecationReason
        }
      })
    });
    exports2.__Field = __Field;
    var __InputValue = new _definition.GraphQLObjectType({
      name: "__InputValue",
      description: "Arguments provided to Fields or Directives and the input fields of an InputObject are represented as Input Values which describe their type and optionally a default value.",
      fields: () => ({
        name: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          resolve: (inputValue) => inputValue.name
        },
        description: {
          type: _scalars.GraphQLString,
          resolve: (inputValue) => inputValue.description
        },
        type: {
          type: new _definition.GraphQLNonNull(__Type),
          resolve: (inputValue) => inputValue.type
        },
        defaultValue: {
          type: _scalars.GraphQLString,
          description: "A GraphQL-formatted string representing the default value for this input value.",
          resolve(inputValue) {
            const { type, defaultValue } = inputValue;
            const valueAST = (0, _astFromValue.astFromValue)(defaultValue, type);
            return valueAST ? (0, _printer.print)(valueAST) : null;
          }
        },
        isDeprecated: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          resolve: (field) => field.deprecationReason != null
        },
        deprecationReason: {
          type: _scalars.GraphQLString,
          resolve: (obj) => obj.deprecationReason
        }
      })
    });
    exports2.__InputValue = __InputValue;
    var __EnumValue = new _definition.GraphQLObjectType({
      name: "__EnumValue",
      description: "One possible value for a given Enum. Enum values are unique values, not a placeholder for a string or numeric value. However an Enum value is returned in a JSON response as a string.",
      fields: () => ({
        name: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          resolve: (enumValue) => enumValue.name
        },
        description: {
          type: _scalars.GraphQLString,
          resolve: (enumValue) => enumValue.description
        },
        isDeprecated: {
          type: new _definition.GraphQLNonNull(_scalars.GraphQLBoolean),
          resolve: (enumValue) => enumValue.deprecationReason != null
        },
        deprecationReason: {
          type: _scalars.GraphQLString,
          resolve: (enumValue) => enumValue.deprecationReason
        }
      })
    });
    exports2.__EnumValue = __EnumValue;
    var TypeKind;
    exports2.TypeKind = TypeKind;
    (function(TypeKind2) {
      TypeKind2["SCALAR"] = "SCALAR";
      TypeKind2["OBJECT"] = "OBJECT";
      TypeKind2["INTERFACE"] = "INTERFACE";
      TypeKind2["UNION"] = "UNION";
      TypeKind2["ENUM"] = "ENUM";
      TypeKind2["INPUT_OBJECT"] = "INPUT_OBJECT";
      TypeKind2["LIST"] = "LIST";
      TypeKind2["NON_NULL"] = "NON_NULL";
    })(TypeKind || (exports2.TypeKind = TypeKind = {}));
    var __TypeKind = new _definition.GraphQLEnumType({
      name: "__TypeKind",
      description: "An enum describing what kind of type a given `__Type` is.",
      values: {
        SCALAR: {
          value: TypeKind.SCALAR,
          description: "Indicates this type is a scalar."
        },
        OBJECT: {
          value: TypeKind.OBJECT,
          description: "Indicates this type is an object. `fields` and `interfaces` are valid fields."
        },
        INTERFACE: {
          value: TypeKind.INTERFACE,
          description: "Indicates this type is an interface. `fields`, `interfaces`, and `possibleTypes` are valid fields."
        },
        UNION: {
          value: TypeKind.UNION,
          description: "Indicates this type is a union. `possibleTypes` is a valid field."
        },
        ENUM: {
          value: TypeKind.ENUM,
          description: "Indicates this type is an enum. `enumValues` is a valid field."
        },
        INPUT_OBJECT: {
          value: TypeKind.INPUT_OBJECT,
          description: "Indicates this type is an input object. `inputFields` is a valid field."
        },
        LIST: {
          value: TypeKind.LIST,
          description: "Indicates this type is a list. `ofType` is a valid field."
        },
        NON_NULL: {
          value: TypeKind.NON_NULL,
          description: "Indicates this type is a non-null. `ofType` is a valid field."
        }
      }
    });
    exports2.__TypeKind = __TypeKind;
    var SchemaMetaFieldDef = {
      name: "__schema",
      type: new _definition.GraphQLNonNull(__Schema),
      description: "Access the current type schema of this server.",
      args: [],
      resolve: (_source, _args, _context, { schema }) => schema,
      deprecationReason: void 0,
      extensions: /* @__PURE__ */ Object.create(null),
      astNode: void 0
    };
    exports2.SchemaMetaFieldDef = SchemaMetaFieldDef;
    var TypeMetaFieldDef = {
      name: "__type",
      type: __Type,
      description: "Request the type information of a single type.",
      args: [
        {
          name: "name",
          description: void 0,
          type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
          defaultValue: void 0,
          deprecationReason: void 0,
          extensions: /* @__PURE__ */ Object.create(null),
          astNode: void 0
        }
      ],
      resolve: (_source, { name }, _context, { schema }) => schema.getType(name),
      deprecationReason: void 0,
      extensions: /* @__PURE__ */ Object.create(null),
      astNode: void 0
    };
    exports2.TypeMetaFieldDef = TypeMetaFieldDef;
    var TypeNameMetaFieldDef = {
      name: "__typename",
      type: new _definition.GraphQLNonNull(_scalars.GraphQLString),
      description: "The name of the current Object type at runtime.",
      args: [],
      resolve: (_source, _args, _context, { parentType }) => parentType.name,
      deprecationReason: void 0,
      extensions: /* @__PURE__ */ Object.create(null),
      astNode: void 0
    };
    exports2.TypeNameMetaFieldDef = TypeNameMetaFieldDef;
    var introspectionTypes = Object.freeze([
      __Schema,
      __Directive,
      __DirectiveLocation,
      __Type,
      __Field,
      __InputValue,
      __EnumValue,
      __TypeKind
    ]);
    exports2.introspectionTypes = introspectionTypes;
    function isIntrospectionType(type) {
      return introspectionTypes.some(({ name }) => type.name === name);
    }
  }
});

// node_modules/graphql/type/schema.js
var require_schema = __commonJS({
  "node_modules/graphql/type/schema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.GraphQLSchema = void 0;
    exports2.assertSchema = assertSchema;
    exports2.isSchema = isSchema;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _instanceOf = require_instanceOf();
    var _isObjectLike = require_isObjectLike();
    var _toObjMap = require_toObjMap();
    var _ast = require_ast();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    function isSchema(schema) {
      return (0, _instanceOf.instanceOf)(schema, GraphQLSchema);
    }
    function assertSchema(schema) {
      if (!isSchema(schema)) {
        throw new Error(
          `Expected ${(0, _inspect.inspect)(schema)} to be a GraphQL schema.`
        );
      }
      return schema;
    }
    var GraphQLSchema = class {
      // Used as a cache for validateSchema().
      constructor(config) {
        var _config$extensionASTN, _config$directives;
        this.__validationErrors = config.assumeValid === true ? [] : void 0;
        (0, _isObjectLike.isObjectLike)(config) || (0, _devAssert.devAssert)(false, "Must provide configuration object.");
        !config.types || Array.isArray(config.types) || (0, _devAssert.devAssert)(
          false,
          `"types" must be Array if provided but got: ${(0, _inspect.inspect)(
            config.types
          )}.`
        );
        !config.directives || Array.isArray(config.directives) || (0, _devAssert.devAssert)(
          false,
          `"directives" must be Array if provided but got: ${(0, _inspect.inspect)(config.directives)}.`
        );
        this.description = config.description;
        this.extensions = (0, _toObjMap.toObjMap)(config.extensions);
        this.astNode = config.astNode;
        this.extensionASTNodes = (_config$extensionASTN = config.extensionASTNodes) !== null && _config$extensionASTN !== void 0 ? _config$extensionASTN : [];
        this._queryType = config.query;
        this._mutationType = config.mutation;
        this._subscriptionType = config.subscription;
        this._directives = (_config$directives = config.directives) !== null && _config$directives !== void 0 ? _config$directives : _directives.specifiedDirectives;
        const allReferencedTypes = new Set(config.types);
        if (config.types != null) {
          for (const type of config.types) {
            allReferencedTypes.delete(type);
            collectReferencedTypes(type, allReferencedTypes);
          }
        }
        if (this._queryType != null) {
          collectReferencedTypes(this._queryType, allReferencedTypes);
        }
        if (this._mutationType != null) {
          collectReferencedTypes(this._mutationType, allReferencedTypes);
        }
        if (this._subscriptionType != null) {
          collectReferencedTypes(this._subscriptionType, allReferencedTypes);
        }
        for (const directive of this._directives) {
          if ((0, _directives.isDirective)(directive)) {
            for (const arg of directive.args) {
              collectReferencedTypes(arg.type, allReferencedTypes);
            }
          }
        }
        collectReferencedTypes(_introspection.__Schema, allReferencedTypes);
        this._typeMap = /* @__PURE__ */ Object.create(null);
        this._subTypeMap = /* @__PURE__ */ Object.create(null);
        this._implementationsMap = /* @__PURE__ */ Object.create(null);
        for (const namedType of allReferencedTypes) {
          if (namedType == null) {
            continue;
          }
          const typeName = namedType.name;
          typeName || (0, _devAssert.devAssert)(
            false,
            "One of the provided types for building the Schema is missing a name."
          );
          if (this._typeMap[typeName] !== void 0) {
            throw new Error(
              `Schema must contain uniquely named types but contains multiple types named "${typeName}".`
            );
          }
          this._typeMap[typeName] = namedType;
          if ((0, _definition.isInterfaceType)(namedType)) {
            for (const iface of namedType.getInterfaces()) {
              if ((0, _definition.isInterfaceType)(iface)) {
                let implementations = this._implementationsMap[iface.name];
                if (implementations === void 0) {
                  implementations = this._implementationsMap[iface.name] = {
                    objects: [],
                    interfaces: []
                  };
                }
                implementations.interfaces.push(namedType);
              }
            }
          } else if ((0, _definition.isObjectType)(namedType)) {
            for (const iface of namedType.getInterfaces()) {
              if ((0, _definition.isInterfaceType)(iface)) {
                let implementations = this._implementationsMap[iface.name];
                if (implementations === void 0) {
                  implementations = this._implementationsMap[iface.name] = {
                    objects: [],
                    interfaces: []
                  };
                }
                implementations.objects.push(namedType);
              }
            }
          }
        }
      }
      get [Symbol.toStringTag]() {
        return "GraphQLSchema";
      }
      getQueryType() {
        return this._queryType;
      }
      getMutationType() {
        return this._mutationType;
      }
      getSubscriptionType() {
        return this._subscriptionType;
      }
      getRootType(operation) {
        switch (operation) {
          case _ast.OperationTypeNode.QUERY:
            return this.getQueryType();
          case _ast.OperationTypeNode.MUTATION:
            return this.getMutationType();
          case _ast.OperationTypeNode.SUBSCRIPTION:
            return this.getSubscriptionType();
        }
      }
      getTypeMap() {
        return this._typeMap;
      }
      getType(name) {
        return this.getTypeMap()[name];
      }
      getPossibleTypes(abstractType) {
        return (0, _definition.isUnionType)(abstractType) ? abstractType.getTypes() : this.getImplementations(abstractType).objects;
      }
      getImplementations(interfaceType) {
        const implementations = this._implementationsMap[interfaceType.name];
        return implementations !== null && implementations !== void 0 ? implementations : {
          objects: [],
          interfaces: []
        };
      }
      isSubType(abstractType, maybeSubType) {
        let map = this._subTypeMap[abstractType.name];
        if (map === void 0) {
          map = /* @__PURE__ */ Object.create(null);
          if ((0, _definition.isUnionType)(abstractType)) {
            for (const type of abstractType.getTypes()) {
              map[type.name] = true;
            }
          } else {
            const implementations = this.getImplementations(abstractType);
            for (const type of implementations.objects) {
              map[type.name] = true;
            }
            for (const type of implementations.interfaces) {
              map[type.name] = true;
            }
          }
          this._subTypeMap[abstractType.name] = map;
        }
        return map[maybeSubType.name] !== void 0;
      }
      getDirectives() {
        return this._directives;
      }
      getDirective(name) {
        return this.getDirectives().find((directive) => directive.name === name);
      }
      toConfig() {
        return {
          description: this.description,
          query: this.getQueryType(),
          mutation: this.getMutationType(),
          subscription: this.getSubscriptionType(),
          types: Object.values(this.getTypeMap()),
          directives: this.getDirectives(),
          extensions: this.extensions,
          astNode: this.astNode,
          extensionASTNodes: this.extensionASTNodes,
          assumeValid: this.__validationErrors !== void 0
        };
      }
    };
    exports2.GraphQLSchema = GraphQLSchema;
    function collectReferencedTypes(type, typeSet) {
      const namedType = (0, _definition.getNamedType)(type);
      if (!typeSet.has(namedType)) {
        typeSet.add(namedType);
        if ((0, _definition.isUnionType)(namedType)) {
          for (const memberType of namedType.getTypes()) {
            collectReferencedTypes(memberType, typeSet);
          }
        } else if ((0, _definition.isObjectType)(namedType) || (0, _definition.isInterfaceType)(namedType)) {
          for (const interfaceType of namedType.getInterfaces()) {
            collectReferencedTypes(interfaceType, typeSet);
          }
          for (const field of Object.values(namedType.getFields())) {
            collectReferencedTypes(field.type, typeSet);
            for (const arg of field.args) {
              collectReferencedTypes(arg.type, typeSet);
            }
          }
        } else if ((0, _definition.isInputObjectType)(namedType)) {
          for (const field of Object.values(namedType.getFields())) {
            collectReferencedTypes(field.type, typeSet);
          }
        }
      }
      return typeSet;
    }
  }
});

// node_modules/graphql/type/validate.js
var require_validate = __commonJS({
  "node_modules/graphql/type/validate.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.assertValidSchema = assertValidSchema;
    exports2.validateSchema = validateSchema;
    var _inspect = require_inspect();
    var _GraphQLError = require_GraphQLError();
    var _ast = require_ast();
    var _typeComparators = require_typeComparators();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    var _schema = require_schema();
    function validateSchema(schema) {
      (0, _schema.assertSchema)(schema);
      if (schema.__validationErrors) {
        return schema.__validationErrors;
      }
      const context = new SchemaValidationContext(schema);
      validateRootTypes(context);
      validateDirectives(context);
      validateTypes(context);
      const errors = context.getErrors();
      schema.__validationErrors = errors;
      return errors;
    }
    function assertValidSchema(schema) {
      const errors = validateSchema(schema);
      if (errors.length !== 0) {
        throw new Error(errors.map((error) => error.message).join("\n\n"));
      }
    }
    var SchemaValidationContext = class {
      constructor(schema) {
        this._errors = [];
        this.schema = schema;
      }
      reportError(message, nodes) {
        const _nodes = Array.isArray(nodes) ? nodes.filter(Boolean) : nodes;
        this._errors.push(
          new _GraphQLError.GraphQLError(message, {
            nodes: _nodes
          })
        );
      }
      getErrors() {
        return this._errors;
      }
    };
    function validateRootTypes(context) {
      const schema = context.schema;
      const queryType = schema.getQueryType();
      if (!queryType) {
        context.reportError("Query root type must be provided.", schema.astNode);
      } else if (!(0, _definition.isObjectType)(queryType)) {
        var _getOperationTypeNode;
        context.reportError(
          `Query root type must be Object type, it cannot be ${(0, _inspect.inspect)(queryType)}.`,
          (_getOperationTypeNode = getOperationTypeNode(
            schema,
            _ast.OperationTypeNode.QUERY
          )) !== null && _getOperationTypeNode !== void 0 ? _getOperationTypeNode : queryType.astNode
        );
      }
      const mutationType = schema.getMutationType();
      if (mutationType && !(0, _definition.isObjectType)(mutationType)) {
        var _getOperationTypeNode2;
        context.reportError(
          `Mutation root type must be Object type if provided, it cannot be ${(0, _inspect.inspect)(mutationType)}.`,
          (_getOperationTypeNode2 = getOperationTypeNode(
            schema,
            _ast.OperationTypeNode.MUTATION
          )) !== null && _getOperationTypeNode2 !== void 0 ? _getOperationTypeNode2 : mutationType.astNode
        );
      }
      const subscriptionType = schema.getSubscriptionType();
      if (subscriptionType && !(0, _definition.isObjectType)(subscriptionType)) {
        var _getOperationTypeNode3;
        context.reportError(
          `Subscription root type must be Object type if provided, it cannot be ${(0, _inspect.inspect)(subscriptionType)}.`,
          (_getOperationTypeNode3 = getOperationTypeNode(
            schema,
            _ast.OperationTypeNode.SUBSCRIPTION
          )) !== null && _getOperationTypeNode3 !== void 0 ? _getOperationTypeNode3 : subscriptionType.astNode
        );
      }
    }
    function getOperationTypeNode(schema, operation) {
      var _flatMap$find;
      return (_flatMap$find = [schema.astNode, ...schema.extensionASTNodes].flatMap(
        // FIXME: https://github.com/graphql/graphql-js/issues/2203
        (schemaNode) => {
          var _schemaNode$operation;
          return (
            /* c8 ignore next */
            (_schemaNode$operation = schemaNode === null || schemaNode === void 0 ? void 0 : schemaNode.operationTypes) !== null && _schemaNode$operation !== void 0 ? _schemaNode$operation : []
          );
        }
      ).find((operationNode) => operationNode.operation === operation)) === null || _flatMap$find === void 0 ? void 0 : _flatMap$find.type;
    }
    function validateDirectives(context) {
      for (const directive of context.schema.getDirectives()) {
        if (!(0, _directives.isDirective)(directive)) {
          context.reportError(
            `Expected directive but got: ${(0, _inspect.inspect)(directive)}.`,
            directive === null || directive === void 0 ? void 0 : directive.astNode
          );
          continue;
        }
        validateName(context, directive);
        for (const arg of directive.args) {
          validateName(context, arg);
          if (!(0, _definition.isInputType)(arg.type)) {
            context.reportError(
              `The type of @${directive.name}(${arg.name}:) must be Input Type but got: ${(0, _inspect.inspect)(arg.type)}.`,
              arg.astNode
            );
          }
          if ((0, _definition.isRequiredArgument)(arg) && arg.deprecationReason != null) {
            var _arg$astNode;
            context.reportError(
              `Required argument @${directive.name}(${arg.name}:) cannot be deprecated.`,
              [
                getDeprecatedDirectiveNode(arg.astNode),
                (_arg$astNode = arg.astNode) === null || _arg$astNode === void 0 ? void 0 : _arg$astNode.type
              ]
            );
          }
        }
      }
    }
    function validateName(context, node) {
      if (node.name.startsWith("__")) {
        context.reportError(
          `Name "${node.name}" must not begin with "__", which is reserved by GraphQL introspection.`,
          node.astNode
        );
      }
    }
    function validateTypes(context) {
      const validateInputObjectCircularRefs = createInputObjectCircularRefsValidator(context);
      const typeMap = context.schema.getTypeMap();
      for (const type of Object.values(typeMap)) {
        if (!(0, _definition.isNamedType)(type)) {
          context.reportError(
            `Expected GraphQL named type but got: ${(0, _inspect.inspect)(type)}.`,
            type.astNode
          );
          continue;
        }
        if (!(0, _introspection.isIntrospectionType)(type)) {
          validateName(context, type);
        }
        if ((0, _definition.isObjectType)(type)) {
          validateFields(context, type);
          validateInterfaces(context, type);
        } else if ((0, _definition.isInterfaceType)(type)) {
          validateFields(context, type);
          validateInterfaces(context, type);
        } else if ((0, _definition.isUnionType)(type)) {
          validateUnionMembers(context, type);
        } else if ((0, _definition.isEnumType)(type)) {
          validateEnumValues(context, type);
        } else if ((0, _definition.isInputObjectType)(type)) {
          validateInputFields(context, type);
          validateInputObjectCircularRefs(type);
        }
      }
    }
    function validateFields(context, type) {
      const fields = Object.values(type.getFields());
      if (fields.length === 0) {
        context.reportError(`Type ${type.name} must define one or more fields.`, [
          type.astNode,
          ...type.extensionASTNodes
        ]);
      }
      for (const field of fields) {
        validateName(context, field);
        if (!(0, _definition.isOutputType)(field.type)) {
          var _field$astNode;
          context.reportError(
            `The type of ${type.name}.${field.name} must be Output Type but got: ${(0, _inspect.inspect)(field.type)}.`,
            (_field$astNode = field.astNode) === null || _field$astNode === void 0 ? void 0 : _field$astNode.type
          );
        }
        for (const arg of field.args) {
          const argName = arg.name;
          validateName(context, arg);
          if (!(0, _definition.isInputType)(arg.type)) {
            var _arg$astNode2;
            context.reportError(
              `The type of ${type.name}.${field.name}(${argName}:) must be Input Type but got: ${(0, _inspect.inspect)(arg.type)}.`,
              (_arg$astNode2 = arg.astNode) === null || _arg$astNode2 === void 0 ? void 0 : _arg$astNode2.type
            );
          }
          if ((0, _definition.isRequiredArgument)(arg) && arg.deprecationReason != null) {
            var _arg$astNode3;
            context.reportError(
              `Required argument ${type.name}.${field.name}(${argName}:) cannot be deprecated.`,
              [
                getDeprecatedDirectiveNode(arg.astNode),
                (_arg$astNode3 = arg.astNode) === null || _arg$astNode3 === void 0 ? void 0 : _arg$astNode3.type
              ]
            );
          }
        }
      }
    }
    function validateInterfaces(context, type) {
      const ifaceTypeNames = /* @__PURE__ */ Object.create(null);
      for (const iface of type.getInterfaces()) {
        if (!(0, _definition.isInterfaceType)(iface)) {
          context.reportError(
            `Type ${(0, _inspect.inspect)(
              type
            )} must only implement Interface types, it cannot implement ${(0, _inspect.inspect)(iface)}.`,
            getAllImplementsInterfaceNodes(type, iface)
          );
          continue;
        }
        if (type === iface) {
          context.reportError(
            `Type ${type.name} cannot implement itself because it would create a circular reference.`,
            getAllImplementsInterfaceNodes(type, iface)
          );
          continue;
        }
        if (ifaceTypeNames[iface.name]) {
          context.reportError(
            `Type ${type.name} can only implement ${iface.name} once.`,
            getAllImplementsInterfaceNodes(type, iface)
          );
          continue;
        }
        ifaceTypeNames[iface.name] = true;
        validateTypeImplementsAncestors(context, type, iface);
        validateTypeImplementsInterface(context, type, iface);
      }
    }
    function validateTypeImplementsInterface(context, type, iface) {
      const typeFieldMap = type.getFields();
      for (const ifaceField of Object.values(iface.getFields())) {
        const fieldName = ifaceField.name;
        const typeField = typeFieldMap[fieldName];
        if (!typeField) {
          context.reportError(
            `Interface field ${iface.name}.${fieldName} expected but ${type.name} does not provide it.`,
            [ifaceField.astNode, type.astNode, ...type.extensionASTNodes]
          );
          continue;
        }
        if (!(0, _typeComparators.isTypeSubTypeOf)(
          context.schema,
          typeField.type,
          ifaceField.type
        )) {
          var _ifaceField$astNode, _typeField$astNode;
          context.reportError(
            `Interface field ${iface.name}.${fieldName} expects type ${(0, _inspect.inspect)(ifaceField.type)} but ${type.name}.${fieldName} is type ${(0, _inspect.inspect)(typeField.type)}.`,
            [
              (_ifaceField$astNode = ifaceField.astNode) === null || _ifaceField$astNode === void 0 ? void 0 : _ifaceField$astNode.type,
              (_typeField$astNode = typeField.astNode) === null || _typeField$astNode === void 0 ? void 0 : _typeField$astNode.type
            ]
          );
        }
        for (const ifaceArg of ifaceField.args) {
          const argName = ifaceArg.name;
          const typeArg = typeField.args.find((arg) => arg.name === argName);
          if (!typeArg) {
            context.reportError(
              `Interface field argument ${iface.name}.${fieldName}(${argName}:) expected but ${type.name}.${fieldName} does not provide it.`,
              [ifaceArg.astNode, typeField.astNode]
            );
            continue;
          }
          if (!(0, _typeComparators.isEqualType)(ifaceArg.type, typeArg.type)) {
            var _ifaceArg$astNode, _typeArg$astNode;
            context.reportError(
              `Interface field argument ${iface.name}.${fieldName}(${argName}:) expects type ${(0, _inspect.inspect)(ifaceArg.type)} but ${type.name}.${fieldName}(${argName}:) is type ${(0, _inspect.inspect)(typeArg.type)}.`,
              [
                (_ifaceArg$astNode = ifaceArg.astNode) === null || _ifaceArg$astNode === void 0 ? void 0 : _ifaceArg$astNode.type,
                (_typeArg$astNode = typeArg.astNode) === null || _typeArg$astNode === void 0 ? void 0 : _typeArg$astNode.type
              ]
            );
          }
        }
        for (const typeArg of typeField.args) {
          const argName = typeArg.name;
          const ifaceArg = ifaceField.args.find((arg) => arg.name === argName);
          if (!ifaceArg && (0, _definition.isRequiredArgument)(typeArg)) {
            context.reportError(
              `Object field ${type.name}.${fieldName} includes required argument ${argName} that is missing from the Interface field ${iface.name}.${fieldName}.`,
              [typeArg.astNode, ifaceField.astNode]
            );
          }
        }
      }
    }
    function validateTypeImplementsAncestors(context, type, iface) {
      const ifaceInterfaces = type.getInterfaces();
      for (const transitive of iface.getInterfaces()) {
        if (!ifaceInterfaces.includes(transitive)) {
          context.reportError(
            transitive === type ? `Type ${type.name} cannot implement ${iface.name} because it would create a circular reference.` : `Type ${type.name} must implement ${transitive.name} because it is implemented by ${iface.name}.`,
            [
              ...getAllImplementsInterfaceNodes(iface, transitive),
              ...getAllImplementsInterfaceNodes(type, iface)
            ]
          );
        }
      }
    }
    function validateUnionMembers(context, union) {
      const memberTypes = union.getTypes();
      if (memberTypes.length === 0) {
        context.reportError(
          `Union type ${union.name} must define one or more member types.`,
          [union.astNode, ...union.extensionASTNodes]
        );
      }
      const includedTypeNames = /* @__PURE__ */ Object.create(null);
      for (const memberType of memberTypes) {
        if (includedTypeNames[memberType.name]) {
          context.reportError(
            `Union type ${union.name} can only include type ${memberType.name} once.`,
            getUnionMemberTypeNodes(union, memberType.name)
          );
          continue;
        }
        includedTypeNames[memberType.name] = true;
        if (!(0, _definition.isObjectType)(memberType)) {
          context.reportError(
            `Union type ${union.name} can only include Object types, it cannot include ${(0, _inspect.inspect)(memberType)}.`,
            getUnionMemberTypeNodes(union, String(memberType))
          );
        }
      }
    }
    function validateEnumValues(context, enumType) {
      const enumValues = enumType.getValues();
      if (enumValues.length === 0) {
        context.reportError(
          `Enum type ${enumType.name} must define one or more values.`,
          [enumType.astNode, ...enumType.extensionASTNodes]
        );
      }
      for (const enumValue of enumValues) {
        validateName(context, enumValue);
      }
    }
    function validateInputFields(context, inputObj) {
      const fields = Object.values(inputObj.getFields());
      if (fields.length === 0) {
        context.reportError(
          `Input Object type ${inputObj.name} must define one or more fields.`,
          [inputObj.astNode, ...inputObj.extensionASTNodes]
        );
      }
      for (const field of fields) {
        validateName(context, field);
        if (!(0, _definition.isInputType)(field.type)) {
          var _field$astNode2;
          context.reportError(
            `The type of ${inputObj.name}.${field.name} must be Input Type but got: ${(0, _inspect.inspect)(field.type)}.`,
            (_field$astNode2 = field.astNode) === null || _field$astNode2 === void 0 ? void 0 : _field$astNode2.type
          );
        }
        if ((0, _definition.isRequiredInputField)(field) && field.deprecationReason != null) {
          var _field$astNode3;
          context.reportError(
            `Required input field ${inputObj.name}.${field.name} cannot be deprecated.`,
            [
              getDeprecatedDirectiveNode(field.astNode),
              (_field$astNode3 = field.astNode) === null || _field$astNode3 === void 0 ? void 0 : _field$astNode3.type
            ]
          );
        }
      }
    }
    function createInputObjectCircularRefsValidator(context) {
      const visitedTypes = /* @__PURE__ */ Object.create(null);
      const fieldPath = [];
      const fieldPathIndexByTypeName = /* @__PURE__ */ Object.create(null);
      return detectCycleRecursive;
      function detectCycleRecursive(inputObj) {
        if (visitedTypes[inputObj.name]) {
          return;
        }
        visitedTypes[inputObj.name] = true;
        fieldPathIndexByTypeName[inputObj.name] = fieldPath.length;
        const fields = Object.values(inputObj.getFields());
        for (const field of fields) {
          if ((0, _definition.isNonNullType)(field.type) && (0, _definition.isInputObjectType)(field.type.ofType)) {
            const fieldType = field.type.ofType;
            const cycleIndex = fieldPathIndexByTypeName[fieldType.name];
            fieldPath.push(field);
            if (cycleIndex === void 0) {
              detectCycleRecursive(fieldType);
            } else {
              const cyclePath = fieldPath.slice(cycleIndex);
              const pathStr = cyclePath.map((fieldObj) => fieldObj.name).join(".");
              context.reportError(
                `Cannot reference Input Object "${fieldType.name}" within itself through a series of non-null fields: "${pathStr}".`,
                cyclePath.map((fieldObj) => fieldObj.astNode)
              );
            }
            fieldPath.pop();
          }
        }
        fieldPathIndexByTypeName[inputObj.name] = void 0;
      }
    }
    function getAllImplementsInterfaceNodes(type, iface) {
      const { astNode, extensionASTNodes } = type;
      const nodes = astNode != null ? [astNode, ...extensionASTNodes] : extensionASTNodes;
      return nodes.flatMap((typeNode) => {
        var _typeNode$interfaces;
        return (
          /* c8 ignore next */
          (_typeNode$interfaces = typeNode.interfaces) !== null && _typeNode$interfaces !== void 0 ? _typeNode$interfaces : []
        );
      }).filter((ifaceNode) => ifaceNode.name.value === iface.name);
    }
    function getUnionMemberTypeNodes(union, typeName) {
      const { astNode, extensionASTNodes } = union;
      const nodes = astNode != null ? [astNode, ...extensionASTNodes] : extensionASTNodes;
      return nodes.flatMap((unionNode) => {
        var _unionNode$types;
        return (
          /* c8 ignore next */
          (_unionNode$types = unionNode.types) !== null && _unionNode$types !== void 0 ? _unionNode$types : []
        );
      }).filter((typeNode) => typeNode.name.value === typeName);
    }
    function getDeprecatedDirectiveNode(definitionNode) {
      var _definitionNode$direc;
      return definitionNode === null || definitionNode === void 0 ? void 0 : (_definitionNode$direc = definitionNode.directives) === null || _definitionNode$direc === void 0 ? void 0 : _definitionNode$direc.find(
        (node) => node.name.value === _directives.GraphQLDeprecatedDirective.name
      );
    }
  }
});

// node_modules/graphql/utilities/typeFromAST.js
var require_typeFromAST = __commonJS({
  "node_modules/graphql/utilities/typeFromAST.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.typeFromAST = typeFromAST;
    var _kinds = require_kinds();
    var _definition = require_definition();
    function typeFromAST(schema, typeNode) {
      switch (typeNode.kind) {
        case _kinds.Kind.LIST_TYPE: {
          const innerType = typeFromAST(schema, typeNode.type);
          return innerType && new _definition.GraphQLList(innerType);
        }
        case _kinds.Kind.NON_NULL_TYPE: {
          const innerType = typeFromAST(schema, typeNode.type);
          return innerType && new _definition.GraphQLNonNull(innerType);
        }
        case _kinds.Kind.NAMED_TYPE:
          return schema.getType(typeNode.name.value);
      }
    }
  }
});

// node_modules/graphql/utilities/TypeInfo.js
var require_TypeInfo = __commonJS({
  "node_modules/graphql/utilities/TypeInfo.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.TypeInfo = void 0;
    exports2.visitWithTypeInfo = visitWithTypeInfo;
    var _ast = require_ast();
    var _kinds = require_kinds();
    var _visitor = require_visitor();
    var _definition = require_definition();
    var _introspection = require_introspection();
    var _typeFromAST = require_typeFromAST();
    var TypeInfo = class {
      constructor(schema, initialType, getFieldDefFn) {
        this._schema = schema;
        this._typeStack = [];
        this._parentTypeStack = [];
        this._inputTypeStack = [];
        this._fieldDefStack = [];
        this._defaultValueStack = [];
        this._directive = null;
        this._argument = null;
        this._enumValue = null;
        this._getFieldDef = getFieldDefFn !== null && getFieldDefFn !== void 0 ? getFieldDefFn : getFieldDef;
        if (initialType) {
          if ((0, _definition.isInputType)(initialType)) {
            this._inputTypeStack.push(initialType);
          }
          if ((0, _definition.isCompositeType)(initialType)) {
            this._parentTypeStack.push(initialType);
          }
          if ((0, _definition.isOutputType)(initialType)) {
            this._typeStack.push(initialType);
          }
        }
      }
      get [Symbol.toStringTag]() {
        return "TypeInfo";
      }
      getType() {
        if (this._typeStack.length > 0) {
          return this._typeStack[this._typeStack.length - 1];
        }
      }
      getParentType() {
        if (this._parentTypeStack.length > 0) {
          return this._parentTypeStack[this._parentTypeStack.length - 1];
        }
      }
      getInputType() {
        if (this._inputTypeStack.length > 0) {
          return this._inputTypeStack[this._inputTypeStack.length - 1];
        }
      }
      getParentInputType() {
        if (this._inputTypeStack.length > 1) {
          return this._inputTypeStack[this._inputTypeStack.length - 2];
        }
      }
      getFieldDef() {
        if (this._fieldDefStack.length > 0) {
          return this._fieldDefStack[this._fieldDefStack.length - 1];
        }
      }
      getDefaultValue() {
        if (this._defaultValueStack.length > 0) {
          return this._defaultValueStack[this._defaultValueStack.length - 1];
        }
      }
      getDirective() {
        return this._directive;
      }
      getArgument() {
        return this._argument;
      }
      getEnumValue() {
        return this._enumValue;
      }
      enter(node) {
        const schema = this._schema;
        switch (node.kind) {
          case _kinds.Kind.SELECTION_SET: {
            const namedType = (0, _definition.getNamedType)(this.getType());
            this._parentTypeStack.push(
              (0, _definition.isCompositeType)(namedType) ? namedType : void 0
            );
            break;
          }
          case _kinds.Kind.FIELD: {
            const parentType = this.getParentType();
            let fieldDef;
            let fieldType;
            if (parentType) {
              fieldDef = this._getFieldDef(schema, parentType, node);
              if (fieldDef) {
                fieldType = fieldDef.type;
              }
            }
            this._fieldDefStack.push(fieldDef);
            this._typeStack.push(
              (0, _definition.isOutputType)(fieldType) ? fieldType : void 0
            );
            break;
          }
          case _kinds.Kind.DIRECTIVE:
            this._directive = schema.getDirective(node.name.value);
            break;
          case _kinds.Kind.OPERATION_DEFINITION: {
            const rootType = schema.getRootType(node.operation);
            this._typeStack.push(
              (0, _definition.isObjectType)(rootType) ? rootType : void 0
            );
            break;
          }
          case _kinds.Kind.INLINE_FRAGMENT:
          case _kinds.Kind.FRAGMENT_DEFINITION: {
            const typeConditionAST = node.typeCondition;
            const outputType = typeConditionAST ? (0, _typeFromAST.typeFromAST)(schema, typeConditionAST) : (0, _definition.getNamedType)(this.getType());
            this._typeStack.push(
              (0, _definition.isOutputType)(outputType) ? outputType : void 0
            );
            break;
          }
          case _kinds.Kind.VARIABLE_DEFINITION: {
            const inputType = (0, _typeFromAST.typeFromAST)(schema, node.type);
            this._inputTypeStack.push(
              (0, _definition.isInputType)(inputType) ? inputType : void 0
            );
            break;
          }
          case _kinds.Kind.ARGUMENT: {
            var _this$getDirective;
            let argDef;
            let argType;
            const fieldOrDirective = (_this$getDirective = this.getDirective()) !== null && _this$getDirective !== void 0 ? _this$getDirective : this.getFieldDef();
            if (fieldOrDirective) {
              argDef = fieldOrDirective.args.find(
                (arg) => arg.name === node.name.value
              );
              if (argDef) {
                argType = argDef.type;
              }
            }
            this._argument = argDef;
            this._defaultValueStack.push(argDef ? argDef.defaultValue : void 0);
            this._inputTypeStack.push(
              (0, _definition.isInputType)(argType) ? argType : void 0
            );
            break;
          }
          case _kinds.Kind.LIST: {
            const listType = (0, _definition.getNullableType)(this.getInputType());
            const itemType = (0, _definition.isListType)(listType) ? listType.ofType : listType;
            this._defaultValueStack.push(void 0);
            this._inputTypeStack.push(
              (0, _definition.isInputType)(itemType) ? itemType : void 0
            );
            break;
          }
          case _kinds.Kind.OBJECT_FIELD: {
            const objectType = (0, _definition.getNamedType)(this.getInputType());
            let inputFieldType;
            let inputField;
            if ((0, _definition.isInputObjectType)(objectType)) {
              inputField = objectType.getFields()[node.name.value];
              if (inputField) {
                inputFieldType = inputField.type;
              }
            }
            this._defaultValueStack.push(
              inputField ? inputField.defaultValue : void 0
            );
            this._inputTypeStack.push(
              (0, _definition.isInputType)(inputFieldType) ? inputFieldType : void 0
            );
            break;
          }
          case _kinds.Kind.ENUM: {
            const enumType = (0, _definition.getNamedType)(this.getInputType());
            let enumValue;
            if ((0, _definition.isEnumType)(enumType)) {
              enumValue = enumType.getValue(node.value);
            }
            this._enumValue = enumValue;
            break;
          }
          default:
        }
      }
      leave(node) {
        switch (node.kind) {
          case _kinds.Kind.SELECTION_SET:
            this._parentTypeStack.pop();
            break;
          case _kinds.Kind.FIELD:
            this._fieldDefStack.pop();
            this._typeStack.pop();
            break;
          case _kinds.Kind.DIRECTIVE:
            this._directive = null;
            break;
          case _kinds.Kind.OPERATION_DEFINITION:
          case _kinds.Kind.INLINE_FRAGMENT:
          case _kinds.Kind.FRAGMENT_DEFINITION:
            this._typeStack.pop();
            break;
          case _kinds.Kind.VARIABLE_DEFINITION:
            this._inputTypeStack.pop();
            break;
          case _kinds.Kind.ARGUMENT:
            this._argument = null;
            this._defaultValueStack.pop();
            this._inputTypeStack.pop();
            break;
          case _kinds.Kind.LIST:
          case _kinds.Kind.OBJECT_FIELD:
            this._defaultValueStack.pop();
            this._inputTypeStack.pop();
            break;
          case _kinds.Kind.ENUM:
            this._enumValue = null;
            break;
          default:
        }
      }
    };
    exports2.TypeInfo = TypeInfo;
    function getFieldDef(schema, parentType, fieldNode) {
      const name = fieldNode.name.value;
      if (name === _introspection.SchemaMetaFieldDef.name && schema.getQueryType() === parentType) {
        return _introspection.SchemaMetaFieldDef;
      }
      if (name === _introspection.TypeMetaFieldDef.name && schema.getQueryType() === parentType) {
        return _introspection.TypeMetaFieldDef;
      }
      if (name === _introspection.TypeNameMetaFieldDef.name && (0, _definition.isCompositeType)(parentType)) {
        return _introspection.TypeNameMetaFieldDef;
      }
      if ((0, _definition.isObjectType)(parentType) || (0, _definition.isInterfaceType)(parentType)) {
        return parentType.getFields()[name];
      }
    }
    function visitWithTypeInfo(typeInfo, visitor) {
      return {
        enter(...args) {
          const node = args[0];
          typeInfo.enter(node);
          const fn = (0, _visitor.getEnterLeaveForKind)(visitor, node.kind).enter;
          if (fn) {
            const result = fn.apply(visitor, args);
            if (result !== void 0) {
              typeInfo.leave(node);
              if ((0, _ast.isNode)(result)) {
                typeInfo.enter(result);
              }
            }
            return result;
          }
        },
        leave(...args) {
          const node = args[0];
          const fn = (0, _visitor.getEnterLeaveForKind)(visitor, node.kind).leave;
          let result;
          if (fn) {
            result = fn.apply(visitor, args);
          }
          typeInfo.leave(node);
          return result;
        }
      };
    }
  }
});

// node_modules/graphql/language/predicates.js
var require_predicates = __commonJS({
  "node_modules/graphql/language/predicates.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isConstValueNode = isConstValueNode;
    exports2.isDefinitionNode = isDefinitionNode;
    exports2.isExecutableDefinitionNode = isExecutableDefinitionNode;
    exports2.isSelectionNode = isSelectionNode;
    exports2.isTypeDefinitionNode = isTypeDefinitionNode;
    exports2.isTypeExtensionNode = isTypeExtensionNode;
    exports2.isTypeNode = isTypeNode;
    exports2.isTypeSystemDefinitionNode = isTypeSystemDefinitionNode;
    exports2.isTypeSystemExtensionNode = isTypeSystemExtensionNode;
    exports2.isValueNode = isValueNode;
    var _kinds = require_kinds();
    function isDefinitionNode(node) {
      return isExecutableDefinitionNode(node) || isTypeSystemDefinitionNode(node) || isTypeSystemExtensionNode(node);
    }
    function isExecutableDefinitionNode(node) {
      return node.kind === _kinds.Kind.OPERATION_DEFINITION || node.kind === _kinds.Kind.FRAGMENT_DEFINITION;
    }
    function isSelectionNode(node) {
      return node.kind === _kinds.Kind.FIELD || node.kind === _kinds.Kind.FRAGMENT_SPREAD || node.kind === _kinds.Kind.INLINE_FRAGMENT;
    }
    function isValueNode(node) {
      return node.kind === _kinds.Kind.VARIABLE || node.kind === _kinds.Kind.INT || node.kind === _kinds.Kind.FLOAT || node.kind === _kinds.Kind.STRING || node.kind === _kinds.Kind.BOOLEAN || node.kind === _kinds.Kind.NULL || node.kind === _kinds.Kind.ENUM || node.kind === _kinds.Kind.LIST || node.kind === _kinds.Kind.OBJECT;
    }
    function isConstValueNode(node) {
      return isValueNode(node) && (node.kind === _kinds.Kind.LIST ? node.values.some(isConstValueNode) : node.kind === _kinds.Kind.OBJECT ? node.fields.some((field) => isConstValueNode(field.value)) : node.kind !== _kinds.Kind.VARIABLE);
    }
    function isTypeNode(node) {
      return node.kind === _kinds.Kind.NAMED_TYPE || node.kind === _kinds.Kind.LIST_TYPE || node.kind === _kinds.Kind.NON_NULL_TYPE;
    }
    function isTypeSystemDefinitionNode(node) {
      return node.kind === _kinds.Kind.SCHEMA_DEFINITION || isTypeDefinitionNode(node) || node.kind === _kinds.Kind.DIRECTIVE_DEFINITION;
    }
    function isTypeDefinitionNode(node) {
      return node.kind === _kinds.Kind.SCALAR_TYPE_DEFINITION || node.kind === _kinds.Kind.OBJECT_TYPE_DEFINITION || node.kind === _kinds.Kind.INTERFACE_TYPE_DEFINITION || node.kind === _kinds.Kind.UNION_TYPE_DEFINITION || node.kind === _kinds.Kind.ENUM_TYPE_DEFINITION || node.kind === _kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION;
    }
    function isTypeSystemExtensionNode(node) {
      return node.kind === _kinds.Kind.SCHEMA_EXTENSION || isTypeExtensionNode(node);
    }
    function isTypeExtensionNode(node) {
      return node.kind === _kinds.Kind.SCALAR_TYPE_EXTENSION || node.kind === _kinds.Kind.OBJECT_TYPE_EXTENSION || node.kind === _kinds.Kind.INTERFACE_TYPE_EXTENSION || node.kind === _kinds.Kind.UNION_TYPE_EXTENSION || node.kind === _kinds.Kind.ENUM_TYPE_EXTENSION || node.kind === _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION;
    }
  }
});

// node_modules/graphql/validation/rules/ExecutableDefinitionsRule.js
var require_ExecutableDefinitionsRule = __commonJS({
  "node_modules/graphql/validation/rules/ExecutableDefinitionsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.ExecutableDefinitionsRule = ExecutableDefinitionsRule;
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _predicates = require_predicates();
    function ExecutableDefinitionsRule(context) {
      return {
        Document(node) {
          for (const definition of node.definitions) {
            if (!(0, _predicates.isExecutableDefinitionNode)(definition)) {
              const defName = definition.kind === _kinds.Kind.SCHEMA_DEFINITION || definition.kind === _kinds.Kind.SCHEMA_EXTENSION ? "schema" : '"' + definition.name.value + '"';
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `The ${defName} definition is not executable.`,
                  {
                    nodes: definition
                  }
                )
              );
            }
          }
          return false;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/FieldsOnCorrectTypeRule.js
var require_FieldsOnCorrectTypeRule = __commonJS({
  "node_modules/graphql/validation/rules/FieldsOnCorrectTypeRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.FieldsOnCorrectTypeRule = FieldsOnCorrectTypeRule;
    var _didYouMean = require_didYouMean();
    var _naturalCompare = require_naturalCompare();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function FieldsOnCorrectTypeRule(context) {
      return {
        Field(node) {
          const type = context.getParentType();
          if (type) {
            const fieldDef = context.getFieldDef();
            if (!fieldDef) {
              const schema = context.getSchema();
              const fieldName = node.name.value;
              let suggestion = (0, _didYouMean.didYouMean)(
                "to use an inline fragment on",
                getSuggestedTypeNames(schema, type, fieldName)
              );
              if (suggestion === "") {
                suggestion = (0, _didYouMean.didYouMean)(
                  getSuggestedFieldNames(type, fieldName)
                );
              }
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Cannot query field "${fieldName}" on type "${type.name}".` + suggestion,
                  {
                    nodes: node
                  }
                )
              );
            }
          }
        }
      };
    }
    function getSuggestedTypeNames(schema, type, fieldName) {
      if (!(0, _definition.isAbstractType)(type)) {
        return [];
      }
      const suggestedTypes = /* @__PURE__ */ new Set();
      const usageCount = /* @__PURE__ */ Object.create(null);
      for (const possibleType of schema.getPossibleTypes(type)) {
        if (!possibleType.getFields()[fieldName]) {
          continue;
        }
        suggestedTypes.add(possibleType);
        usageCount[possibleType.name] = 1;
        for (const possibleInterface of possibleType.getInterfaces()) {
          var _usageCount$possibleI;
          if (!possibleInterface.getFields()[fieldName]) {
            continue;
          }
          suggestedTypes.add(possibleInterface);
          usageCount[possibleInterface.name] = ((_usageCount$possibleI = usageCount[possibleInterface.name]) !== null && _usageCount$possibleI !== void 0 ? _usageCount$possibleI : 0) + 1;
        }
      }
      return [...suggestedTypes].sort((typeA, typeB) => {
        const usageCountDiff = usageCount[typeB.name] - usageCount[typeA.name];
        if (usageCountDiff !== 0) {
          return usageCountDiff;
        }
        if ((0, _definition.isInterfaceType)(typeA) && schema.isSubType(typeA, typeB)) {
          return -1;
        }
        if ((0, _definition.isInterfaceType)(typeB) && schema.isSubType(typeB, typeA)) {
          return 1;
        }
        return (0, _naturalCompare.naturalCompare)(typeA.name, typeB.name);
      }).map((x) => x.name);
    }
    function getSuggestedFieldNames(type, fieldName) {
      if ((0, _definition.isObjectType)(type) || (0, _definition.isInterfaceType)(type)) {
        const possibleFieldNames = Object.keys(type.getFields());
        return (0, _suggestionList.suggestionList)(fieldName, possibleFieldNames);
      }
      return [];
    }
  }
});

// node_modules/graphql/validation/rules/FragmentsOnCompositeTypesRule.js
var require_FragmentsOnCompositeTypesRule = __commonJS({
  "node_modules/graphql/validation/rules/FragmentsOnCompositeTypesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.FragmentsOnCompositeTypesRule = FragmentsOnCompositeTypesRule;
    var _GraphQLError = require_GraphQLError();
    var _printer = require_printer();
    var _definition = require_definition();
    var _typeFromAST = require_typeFromAST();
    function FragmentsOnCompositeTypesRule(context) {
      return {
        InlineFragment(node) {
          const typeCondition = node.typeCondition;
          if (typeCondition) {
            const type = (0, _typeFromAST.typeFromAST)(
              context.getSchema(),
              typeCondition
            );
            if (type && !(0, _definition.isCompositeType)(type)) {
              const typeStr = (0, _printer.print)(typeCondition);
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Fragment cannot condition on non composite type "${typeStr}".`,
                  {
                    nodes: typeCondition
                  }
                )
              );
            }
          }
        },
        FragmentDefinition(node) {
          const type = (0, _typeFromAST.typeFromAST)(
            context.getSchema(),
            node.typeCondition
          );
          if (type && !(0, _definition.isCompositeType)(type)) {
            const typeStr = (0, _printer.print)(node.typeCondition);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Fragment "${node.name.value}" cannot condition on non composite type "${typeStr}".`,
                {
                  nodes: node.typeCondition
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/KnownArgumentNamesRule.js
var require_KnownArgumentNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/KnownArgumentNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.KnownArgumentNamesOnDirectivesRule = KnownArgumentNamesOnDirectivesRule;
    exports2.KnownArgumentNamesRule = KnownArgumentNamesRule;
    var _didYouMean = require_didYouMean();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _directives = require_directives();
    function KnownArgumentNamesRule(context) {
      return {
        // eslint-disable-next-line new-cap
        ...KnownArgumentNamesOnDirectivesRule(context),
        Argument(argNode) {
          const argDef = context.getArgument();
          const fieldDef = context.getFieldDef();
          const parentType = context.getParentType();
          if (!argDef && fieldDef && parentType) {
            const argName = argNode.name.value;
            const knownArgsNames = fieldDef.args.map((arg) => arg.name);
            const suggestions = (0, _suggestionList.suggestionList)(
              argName,
              knownArgsNames
            );
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Unknown argument "${argName}" on field "${parentType.name}.${fieldDef.name}".` + (0, _didYouMean.didYouMean)(suggestions),
                {
                  nodes: argNode
                }
              )
            );
          }
        }
      };
    }
    function KnownArgumentNamesOnDirectivesRule(context) {
      const directiveArgs = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      const definedDirectives = schema ? schema.getDirectives() : _directives.specifiedDirectives;
      for (const directive of definedDirectives) {
        directiveArgs[directive.name] = directive.args.map((arg) => arg.name);
      }
      const astDefinitions = context.getDocument().definitions;
      for (const def of astDefinitions) {
        if (def.kind === _kinds.Kind.DIRECTIVE_DEFINITION) {
          var _def$arguments;
          const argsNodes = (_def$arguments = def.arguments) !== null && _def$arguments !== void 0 ? _def$arguments : [];
          directiveArgs[def.name.value] = argsNodes.map((arg) => arg.name.value);
        }
      }
      return {
        Directive(directiveNode) {
          const directiveName = directiveNode.name.value;
          const knownArgs = directiveArgs[directiveName];
          if (directiveNode.arguments && knownArgs) {
            for (const argNode of directiveNode.arguments) {
              const argName = argNode.name.value;
              if (!knownArgs.includes(argName)) {
                const suggestions = (0, _suggestionList.suggestionList)(
                  argName,
                  knownArgs
                );
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    `Unknown argument "${argName}" on directive "@${directiveName}".` + (0, _didYouMean.didYouMean)(suggestions),
                    {
                      nodes: argNode
                    }
                  )
                );
              }
            }
          }
          return false;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/KnownDirectivesRule.js
var require_KnownDirectivesRule = __commonJS({
  "node_modules/graphql/validation/rules/KnownDirectivesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.KnownDirectivesRule = KnownDirectivesRule;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _GraphQLError = require_GraphQLError();
    var _ast = require_ast();
    var _directiveLocation = require_directiveLocation();
    var _kinds = require_kinds();
    var _directives = require_directives();
    function KnownDirectivesRule(context) {
      const locationsMap = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      const definedDirectives = schema ? schema.getDirectives() : _directives.specifiedDirectives;
      for (const directive of definedDirectives) {
        locationsMap[directive.name] = directive.locations;
      }
      const astDefinitions = context.getDocument().definitions;
      for (const def of astDefinitions) {
        if (def.kind === _kinds.Kind.DIRECTIVE_DEFINITION) {
          locationsMap[def.name.value] = def.locations.map((name) => name.value);
        }
      }
      return {
        Directive(node, _key, _parent, _path, ancestors) {
          const name = node.name.value;
          const locations = locationsMap[name];
          if (!locations) {
            context.reportError(
              new _GraphQLError.GraphQLError(`Unknown directive "@${name}".`, {
                nodes: node
              })
            );
            return;
          }
          const candidateLocation = getDirectiveLocationForASTPath(ancestors);
          if (candidateLocation && !locations.includes(candidateLocation)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Directive "@${name}" may not be used on ${candidateLocation}.`,
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
    function getDirectiveLocationForASTPath(ancestors) {
      const appliedTo = ancestors[ancestors.length - 1];
      "kind" in appliedTo || (0, _invariant.invariant)(false);
      switch (appliedTo.kind) {
        case _kinds.Kind.OPERATION_DEFINITION:
          return getDirectiveLocationForOperation(appliedTo.operation);
        case _kinds.Kind.FIELD:
          return _directiveLocation.DirectiveLocation.FIELD;
        case _kinds.Kind.FRAGMENT_SPREAD:
          return _directiveLocation.DirectiveLocation.FRAGMENT_SPREAD;
        case _kinds.Kind.INLINE_FRAGMENT:
          return _directiveLocation.DirectiveLocation.INLINE_FRAGMENT;
        case _kinds.Kind.FRAGMENT_DEFINITION:
          return _directiveLocation.DirectiveLocation.FRAGMENT_DEFINITION;
        case _kinds.Kind.VARIABLE_DEFINITION:
          return _directiveLocation.DirectiveLocation.VARIABLE_DEFINITION;
        case _kinds.Kind.SCHEMA_DEFINITION:
        case _kinds.Kind.SCHEMA_EXTENSION:
          return _directiveLocation.DirectiveLocation.SCHEMA;
        case _kinds.Kind.SCALAR_TYPE_DEFINITION:
        case _kinds.Kind.SCALAR_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.SCALAR;
        case _kinds.Kind.OBJECT_TYPE_DEFINITION:
        case _kinds.Kind.OBJECT_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.OBJECT;
        case _kinds.Kind.FIELD_DEFINITION:
          return _directiveLocation.DirectiveLocation.FIELD_DEFINITION;
        case _kinds.Kind.INTERFACE_TYPE_DEFINITION:
        case _kinds.Kind.INTERFACE_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.INTERFACE;
        case _kinds.Kind.UNION_TYPE_DEFINITION:
        case _kinds.Kind.UNION_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.UNION;
        case _kinds.Kind.ENUM_TYPE_DEFINITION:
        case _kinds.Kind.ENUM_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.ENUM;
        case _kinds.Kind.ENUM_VALUE_DEFINITION:
          return _directiveLocation.DirectiveLocation.ENUM_VALUE;
        case _kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION:
        case _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION:
          return _directiveLocation.DirectiveLocation.INPUT_OBJECT;
        case _kinds.Kind.INPUT_VALUE_DEFINITION: {
          const parentNode = ancestors[ancestors.length - 3];
          "kind" in parentNode || (0, _invariant.invariant)(false);
          return parentNode.kind === _kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION ? _directiveLocation.DirectiveLocation.INPUT_FIELD_DEFINITION : _directiveLocation.DirectiveLocation.ARGUMENT_DEFINITION;
        }
        default:
          (0, _invariant.invariant)(
            false,
            "Unexpected kind: " + (0, _inspect.inspect)(appliedTo.kind)
          );
      }
    }
    function getDirectiveLocationForOperation(operation) {
      switch (operation) {
        case _ast.OperationTypeNode.QUERY:
          return _directiveLocation.DirectiveLocation.QUERY;
        case _ast.OperationTypeNode.MUTATION:
          return _directiveLocation.DirectiveLocation.MUTATION;
        case _ast.OperationTypeNode.SUBSCRIPTION:
          return _directiveLocation.DirectiveLocation.SUBSCRIPTION;
      }
    }
  }
});

// node_modules/graphql/validation/rules/KnownFragmentNamesRule.js
var require_KnownFragmentNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/KnownFragmentNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.KnownFragmentNamesRule = KnownFragmentNamesRule;
    var _GraphQLError = require_GraphQLError();
    function KnownFragmentNamesRule(context) {
      return {
        FragmentSpread(node) {
          const fragmentName = node.name.value;
          const fragment = context.getFragment(fragmentName);
          if (!fragment) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Unknown fragment "${fragmentName}".`,
                {
                  nodes: node.name
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/KnownTypeNamesRule.js
var require_KnownTypeNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/KnownTypeNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.KnownTypeNamesRule = KnownTypeNamesRule;
    var _didYouMean = require_didYouMean();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _predicates = require_predicates();
    var _introspection = require_introspection();
    var _scalars = require_scalars();
    function KnownTypeNamesRule(context) {
      const schema = context.getSchema();
      const existingTypesMap = schema ? schema.getTypeMap() : /* @__PURE__ */ Object.create(null);
      const definedTypes = /* @__PURE__ */ Object.create(null);
      for (const def of context.getDocument().definitions) {
        if ((0, _predicates.isTypeDefinitionNode)(def)) {
          definedTypes[def.name.value] = true;
        }
      }
      const typeNames = [
        ...Object.keys(existingTypesMap),
        ...Object.keys(definedTypes)
      ];
      return {
        NamedType(node, _1, parent, _2, ancestors) {
          const typeName = node.name.value;
          if (!existingTypesMap[typeName] && !definedTypes[typeName]) {
            var _ancestors$;
            const definitionNode = (_ancestors$ = ancestors[2]) !== null && _ancestors$ !== void 0 ? _ancestors$ : parent;
            const isSDL = definitionNode != null && isSDLNode(definitionNode);
            if (isSDL && standardTypeNames.includes(typeName)) {
              return;
            }
            const suggestedTypes = (0, _suggestionList.suggestionList)(
              typeName,
              isSDL ? standardTypeNames.concat(typeNames) : typeNames
            );
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Unknown type "${typeName}".` + (0, _didYouMean.didYouMean)(suggestedTypes),
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
    var standardTypeNames = [
      ..._scalars.specifiedScalarTypes,
      ..._introspection.introspectionTypes
    ].map((type) => type.name);
    function isSDLNode(value) {
      return "kind" in value && ((0, _predicates.isTypeSystemDefinitionNode)(value) || (0, _predicates.isTypeSystemExtensionNode)(value));
    }
  }
});

// node_modules/graphql/validation/rules/LoneAnonymousOperationRule.js
var require_LoneAnonymousOperationRule = __commonJS({
  "node_modules/graphql/validation/rules/LoneAnonymousOperationRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.LoneAnonymousOperationRule = LoneAnonymousOperationRule;
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    function LoneAnonymousOperationRule(context) {
      let operationCount = 0;
      return {
        Document(node) {
          operationCount = node.definitions.filter(
            (definition) => definition.kind === _kinds.Kind.OPERATION_DEFINITION
          ).length;
        },
        OperationDefinition(node) {
          if (!node.name && operationCount > 1) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                "This anonymous operation must be the only defined operation.",
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/LoneSchemaDefinitionRule.js
var require_LoneSchemaDefinitionRule = __commonJS({
  "node_modules/graphql/validation/rules/LoneSchemaDefinitionRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.LoneSchemaDefinitionRule = LoneSchemaDefinitionRule;
    var _GraphQLError = require_GraphQLError();
    function LoneSchemaDefinitionRule(context) {
      var _ref, _ref2, _oldSchema$astNode;
      const oldSchema = context.getSchema();
      const alreadyDefined = (_ref = (_ref2 = (_oldSchema$astNode = oldSchema === null || oldSchema === void 0 ? void 0 : oldSchema.astNode) !== null && _oldSchema$astNode !== void 0 ? _oldSchema$astNode : oldSchema === null || oldSchema === void 0 ? void 0 : oldSchema.getQueryType()) !== null && _ref2 !== void 0 ? _ref2 : oldSchema === null || oldSchema === void 0 ? void 0 : oldSchema.getMutationType()) !== null && _ref !== void 0 ? _ref : oldSchema === null || oldSchema === void 0 ? void 0 : oldSchema.getSubscriptionType();
      let schemaDefinitionsCount = 0;
      return {
        SchemaDefinition(node) {
          if (alreadyDefined) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                "Cannot define a new schema within a schema extension.",
                {
                  nodes: node
                }
              )
            );
            return;
          }
          if (schemaDefinitionsCount > 0) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                "Must provide only one schema definition.",
                {
                  nodes: node
                }
              )
            );
          }
          ++schemaDefinitionsCount;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/NoFragmentCyclesRule.js
var require_NoFragmentCyclesRule = __commonJS({
  "node_modules/graphql/validation/rules/NoFragmentCyclesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoFragmentCyclesRule = NoFragmentCyclesRule;
    var _GraphQLError = require_GraphQLError();
    function NoFragmentCyclesRule(context) {
      const visitedFrags = /* @__PURE__ */ Object.create(null);
      const spreadPath = [];
      const spreadPathIndexByName = /* @__PURE__ */ Object.create(null);
      return {
        OperationDefinition: () => false,
        FragmentDefinition(node) {
          detectCycleRecursive(node);
          return false;
        }
      };
      function detectCycleRecursive(fragment) {
        if (visitedFrags[fragment.name.value]) {
          return;
        }
        const fragmentName = fragment.name.value;
        visitedFrags[fragmentName] = true;
        const spreadNodes = context.getFragmentSpreads(fragment.selectionSet);
        if (spreadNodes.length === 0) {
          return;
        }
        spreadPathIndexByName[fragmentName] = spreadPath.length;
        for (const spreadNode of spreadNodes) {
          const spreadName = spreadNode.name.value;
          const cycleIndex = spreadPathIndexByName[spreadName];
          spreadPath.push(spreadNode);
          if (cycleIndex === void 0) {
            const spreadFragment = context.getFragment(spreadName);
            if (spreadFragment) {
              detectCycleRecursive(spreadFragment);
            }
          } else {
            const cyclePath = spreadPath.slice(cycleIndex);
            const viaPath = cyclePath.slice(0, -1).map((s) => '"' + s.name.value + '"').join(", ");
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Cannot spread fragment "${spreadName}" within itself` + (viaPath !== "" ? ` via ${viaPath}.` : "."),
                {
                  nodes: cyclePath
                }
              )
            );
          }
          spreadPath.pop();
        }
        spreadPathIndexByName[fragmentName] = void 0;
      }
    }
  }
});

// node_modules/graphql/validation/rules/NoUndefinedVariablesRule.js
var require_NoUndefinedVariablesRule = __commonJS({
  "node_modules/graphql/validation/rules/NoUndefinedVariablesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoUndefinedVariablesRule = NoUndefinedVariablesRule;
    var _GraphQLError = require_GraphQLError();
    function NoUndefinedVariablesRule(context) {
      let variableNameDefined = /* @__PURE__ */ Object.create(null);
      return {
        OperationDefinition: {
          enter() {
            variableNameDefined = /* @__PURE__ */ Object.create(null);
          },
          leave(operation) {
            const usages = context.getRecursiveVariableUsages(operation);
            for (const { node } of usages) {
              const varName = node.name.value;
              if (variableNameDefined[varName] !== true) {
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    operation.name ? `Variable "$${varName}" is not defined by operation "${operation.name.value}".` : `Variable "$${varName}" is not defined.`,
                    {
                      nodes: [node, operation]
                    }
                  )
                );
              }
            }
          }
        },
        VariableDefinition(node) {
          variableNameDefined[node.variable.name.value] = true;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/NoUnusedFragmentsRule.js
var require_NoUnusedFragmentsRule = __commonJS({
  "node_modules/graphql/validation/rules/NoUnusedFragmentsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoUnusedFragmentsRule = NoUnusedFragmentsRule;
    var _GraphQLError = require_GraphQLError();
    function NoUnusedFragmentsRule(context) {
      const operationDefs = [];
      const fragmentDefs = [];
      return {
        OperationDefinition(node) {
          operationDefs.push(node);
          return false;
        },
        FragmentDefinition(node) {
          fragmentDefs.push(node);
          return false;
        },
        Document: {
          leave() {
            const fragmentNameUsed = /* @__PURE__ */ Object.create(null);
            for (const operation of operationDefs) {
              for (const fragment of context.getRecursivelyReferencedFragments(
                operation
              )) {
                fragmentNameUsed[fragment.name.value] = true;
              }
            }
            for (const fragmentDef of fragmentDefs) {
              const fragName = fragmentDef.name.value;
              if (fragmentNameUsed[fragName] !== true) {
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    `Fragment "${fragName}" is never used.`,
                    {
                      nodes: fragmentDef
                    }
                  )
                );
              }
            }
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/NoUnusedVariablesRule.js
var require_NoUnusedVariablesRule = __commonJS({
  "node_modules/graphql/validation/rules/NoUnusedVariablesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoUnusedVariablesRule = NoUnusedVariablesRule;
    var _GraphQLError = require_GraphQLError();
    function NoUnusedVariablesRule(context) {
      let variableDefs = [];
      return {
        OperationDefinition: {
          enter() {
            variableDefs = [];
          },
          leave(operation) {
            const variableNameUsed = /* @__PURE__ */ Object.create(null);
            const usages = context.getRecursiveVariableUsages(operation);
            for (const { node } of usages) {
              variableNameUsed[node.name.value] = true;
            }
            for (const variableDef of variableDefs) {
              const variableName = variableDef.variable.name.value;
              if (variableNameUsed[variableName] !== true) {
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    operation.name ? `Variable "$${variableName}" is never used in operation "${operation.name.value}".` : `Variable "$${variableName}" is never used.`,
                    {
                      nodes: variableDef
                    }
                  )
                );
              }
            }
          }
        },
        VariableDefinition(def) {
          variableDefs.push(def);
        }
      };
    }
  }
});

// node_modules/graphql/utilities/sortValueNode.js
var require_sortValueNode = __commonJS({
  "node_modules/graphql/utilities/sortValueNode.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.sortValueNode = sortValueNode;
    var _naturalCompare = require_naturalCompare();
    var _kinds = require_kinds();
    function sortValueNode(valueNode) {
      switch (valueNode.kind) {
        case _kinds.Kind.OBJECT:
          return { ...valueNode, fields: sortFields(valueNode.fields) };
        case _kinds.Kind.LIST:
          return { ...valueNode, values: valueNode.values.map(sortValueNode) };
        case _kinds.Kind.INT:
        case _kinds.Kind.FLOAT:
        case _kinds.Kind.STRING:
        case _kinds.Kind.BOOLEAN:
        case _kinds.Kind.NULL:
        case _kinds.Kind.ENUM:
        case _kinds.Kind.VARIABLE:
          return valueNode;
      }
    }
    function sortFields(fields) {
      return fields.map((fieldNode) => ({
        ...fieldNode,
        value: sortValueNode(fieldNode.value)
      })).sort(
        (fieldA, fieldB) => (0, _naturalCompare.naturalCompare)(fieldA.name.value, fieldB.name.value)
      );
    }
  }
});

// node_modules/graphql/validation/rules/OverlappingFieldsCanBeMergedRule.js
var require_OverlappingFieldsCanBeMergedRule = __commonJS({
  "node_modules/graphql/validation/rules/OverlappingFieldsCanBeMergedRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.OverlappingFieldsCanBeMergedRule = OverlappingFieldsCanBeMergedRule;
    var _inspect = require_inspect();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _definition = require_definition();
    var _sortValueNode = require_sortValueNode();
    var _typeFromAST = require_typeFromAST();
    function reasonMessage(reason) {
      if (Array.isArray(reason)) {
        return reason.map(
          ([responseName, subReason]) => `subfields "${responseName}" conflict because ` + reasonMessage(subReason)
        ).join(" and ");
      }
      return reason;
    }
    function OverlappingFieldsCanBeMergedRule(context) {
      const comparedFragmentPairs = new PairSet();
      const cachedFieldsAndFragmentNames = /* @__PURE__ */ new Map();
      return {
        SelectionSet(selectionSet) {
          const conflicts = findConflictsWithinSelectionSet(
            context,
            cachedFieldsAndFragmentNames,
            comparedFragmentPairs,
            context.getParentType(),
            selectionSet
          );
          for (const [[responseName, reason], fields1, fields2] of conflicts) {
            const reasonMsg = reasonMessage(reason);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Fields "${responseName}" conflict because ${reasonMsg}. Use different aliases on the fields to fetch both if this was intentional.`,
                {
                  nodes: fields1.concat(fields2)
                }
              )
            );
          }
        }
      };
    }
    function findConflictsWithinSelectionSet(context, cachedFieldsAndFragmentNames, comparedFragmentPairs, parentType, selectionSet) {
      const conflicts = [];
      const [fieldMap, fragmentNames] = getFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        parentType,
        selectionSet
      );
      collectConflictsWithin(
        context,
        conflicts,
        cachedFieldsAndFragmentNames,
        comparedFragmentPairs,
        fieldMap
      );
      if (fragmentNames.length !== 0) {
        for (let i = 0; i < fragmentNames.length; i++) {
          collectConflictsBetweenFieldsAndFragment(
            context,
            conflicts,
            cachedFieldsAndFragmentNames,
            comparedFragmentPairs,
            false,
            fieldMap,
            fragmentNames[i]
          );
          for (let j = i + 1; j < fragmentNames.length; j++) {
            collectConflictsBetweenFragments(
              context,
              conflicts,
              cachedFieldsAndFragmentNames,
              comparedFragmentPairs,
              false,
              fragmentNames[i],
              fragmentNames[j]
            );
          }
        }
      }
      return conflicts;
    }
    function collectConflictsBetweenFieldsAndFragment(context, conflicts, cachedFieldsAndFragmentNames, comparedFragmentPairs, areMutuallyExclusive, fieldMap, fragmentName) {
      const fragment = context.getFragment(fragmentName);
      if (!fragment) {
        return;
      }
      const [fieldMap2, referencedFragmentNames] = getReferencedFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        fragment
      );
      if (fieldMap === fieldMap2) {
        return;
      }
      collectConflictsBetween(
        context,
        conflicts,
        cachedFieldsAndFragmentNames,
        comparedFragmentPairs,
        areMutuallyExclusive,
        fieldMap,
        fieldMap2
      );
      for (const referencedFragmentName of referencedFragmentNames) {
        if (comparedFragmentPairs.has(
          referencedFragmentName,
          fragmentName,
          areMutuallyExclusive
        )) {
          continue;
        }
        comparedFragmentPairs.add(
          referencedFragmentName,
          fragmentName,
          areMutuallyExclusive
        );
        collectConflictsBetweenFieldsAndFragment(
          context,
          conflicts,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          fieldMap,
          referencedFragmentName
        );
      }
    }
    function collectConflictsBetweenFragments(context, conflicts, cachedFieldsAndFragmentNames, comparedFragmentPairs, areMutuallyExclusive, fragmentName1, fragmentName2) {
      if (fragmentName1 === fragmentName2) {
        return;
      }
      if (comparedFragmentPairs.has(
        fragmentName1,
        fragmentName2,
        areMutuallyExclusive
      )) {
        return;
      }
      comparedFragmentPairs.add(fragmentName1, fragmentName2, areMutuallyExclusive);
      const fragment1 = context.getFragment(fragmentName1);
      const fragment2 = context.getFragment(fragmentName2);
      if (!fragment1 || !fragment2) {
        return;
      }
      const [fieldMap1, referencedFragmentNames1] = getReferencedFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        fragment1
      );
      const [fieldMap2, referencedFragmentNames2] = getReferencedFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        fragment2
      );
      collectConflictsBetween(
        context,
        conflicts,
        cachedFieldsAndFragmentNames,
        comparedFragmentPairs,
        areMutuallyExclusive,
        fieldMap1,
        fieldMap2
      );
      for (const referencedFragmentName2 of referencedFragmentNames2) {
        collectConflictsBetweenFragments(
          context,
          conflicts,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          fragmentName1,
          referencedFragmentName2
        );
      }
      for (const referencedFragmentName1 of referencedFragmentNames1) {
        collectConflictsBetweenFragments(
          context,
          conflicts,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          referencedFragmentName1,
          fragmentName2
        );
      }
    }
    function findConflictsBetweenSubSelectionSets(context, cachedFieldsAndFragmentNames, comparedFragmentPairs, areMutuallyExclusive, parentType1, selectionSet1, parentType2, selectionSet2) {
      const conflicts = [];
      const [fieldMap1, fragmentNames1] = getFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        parentType1,
        selectionSet1
      );
      const [fieldMap2, fragmentNames2] = getFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        parentType2,
        selectionSet2
      );
      collectConflictsBetween(
        context,
        conflicts,
        cachedFieldsAndFragmentNames,
        comparedFragmentPairs,
        areMutuallyExclusive,
        fieldMap1,
        fieldMap2
      );
      for (const fragmentName2 of fragmentNames2) {
        collectConflictsBetweenFieldsAndFragment(
          context,
          conflicts,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          fieldMap1,
          fragmentName2
        );
      }
      for (const fragmentName1 of fragmentNames1) {
        collectConflictsBetweenFieldsAndFragment(
          context,
          conflicts,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          fieldMap2,
          fragmentName1
        );
      }
      for (const fragmentName1 of fragmentNames1) {
        for (const fragmentName2 of fragmentNames2) {
          collectConflictsBetweenFragments(
            context,
            conflicts,
            cachedFieldsAndFragmentNames,
            comparedFragmentPairs,
            areMutuallyExclusive,
            fragmentName1,
            fragmentName2
          );
        }
      }
      return conflicts;
    }
    function collectConflictsWithin(context, conflicts, cachedFieldsAndFragmentNames, comparedFragmentPairs, fieldMap) {
      for (const [responseName, fields] of Object.entries(fieldMap)) {
        if (fields.length > 1) {
          for (let i = 0; i < fields.length; i++) {
            for (let j = i + 1; j < fields.length; j++) {
              const conflict = findConflict(
                context,
                cachedFieldsAndFragmentNames,
                comparedFragmentPairs,
                false,
                // within one collection is never mutually exclusive
                responseName,
                fields[i],
                fields[j]
              );
              if (conflict) {
                conflicts.push(conflict);
              }
            }
          }
        }
      }
    }
    function collectConflictsBetween(context, conflicts, cachedFieldsAndFragmentNames, comparedFragmentPairs, parentFieldsAreMutuallyExclusive, fieldMap1, fieldMap2) {
      for (const [responseName, fields1] of Object.entries(fieldMap1)) {
        const fields2 = fieldMap2[responseName];
        if (fields2) {
          for (const field1 of fields1) {
            for (const field2 of fields2) {
              const conflict = findConflict(
                context,
                cachedFieldsAndFragmentNames,
                comparedFragmentPairs,
                parentFieldsAreMutuallyExclusive,
                responseName,
                field1,
                field2
              );
              if (conflict) {
                conflicts.push(conflict);
              }
            }
          }
        }
      }
    }
    function findConflict(context, cachedFieldsAndFragmentNames, comparedFragmentPairs, parentFieldsAreMutuallyExclusive, responseName, field1, field2) {
      const [parentType1, node1, def1] = field1;
      const [parentType2, node2, def2] = field2;
      const areMutuallyExclusive = parentFieldsAreMutuallyExclusive || parentType1 !== parentType2 && (0, _definition.isObjectType)(parentType1) && (0, _definition.isObjectType)(parentType2);
      if (!areMutuallyExclusive) {
        const name1 = node1.name.value;
        const name2 = node2.name.value;
        if (name1 !== name2) {
          return [
            [responseName, `"${name1}" and "${name2}" are different fields`],
            [node1],
            [node2]
          ];
        }
        if (!sameArguments(node1, node2)) {
          return [
            [responseName, "they have differing arguments"],
            [node1],
            [node2]
          ];
        }
      }
      const type1 = def1 === null || def1 === void 0 ? void 0 : def1.type;
      const type2 = def2 === null || def2 === void 0 ? void 0 : def2.type;
      if (type1 && type2 && doTypesConflict(type1, type2)) {
        return [
          [
            responseName,
            `they return conflicting types "${(0, _inspect.inspect)(
              type1
            )}" and "${(0, _inspect.inspect)(type2)}"`
          ],
          [node1],
          [node2]
        ];
      }
      const selectionSet1 = node1.selectionSet;
      const selectionSet2 = node2.selectionSet;
      if (selectionSet1 && selectionSet2) {
        const conflicts = findConflictsBetweenSubSelectionSets(
          context,
          cachedFieldsAndFragmentNames,
          comparedFragmentPairs,
          areMutuallyExclusive,
          (0, _definition.getNamedType)(type1),
          selectionSet1,
          (0, _definition.getNamedType)(type2),
          selectionSet2
        );
        return subfieldConflicts(conflicts, responseName, node1, node2);
      }
    }
    function sameArguments(node1, node2) {
      const args1 = node1.arguments;
      const args2 = node2.arguments;
      if (args1 === void 0 || args1.length === 0) {
        return args2 === void 0 || args2.length === 0;
      }
      if (args2 === void 0 || args2.length === 0) {
        return false;
      }
      if (args1.length !== args2.length) {
        return false;
      }
      const values2 = new Map(args2.map(({ name, value }) => [name.value, value]));
      return args1.every((arg1) => {
        const value1 = arg1.value;
        const value2 = values2.get(arg1.name.value);
        if (value2 === void 0) {
          return false;
        }
        return stringifyValue(value1) === stringifyValue(value2);
      });
    }
    function stringifyValue(value) {
      return (0, _printer.print)((0, _sortValueNode.sortValueNode)(value));
    }
    function doTypesConflict(type1, type2) {
      if ((0, _definition.isListType)(type1)) {
        return (0, _definition.isListType)(type2) ? doTypesConflict(type1.ofType, type2.ofType) : true;
      }
      if ((0, _definition.isListType)(type2)) {
        return true;
      }
      if ((0, _definition.isNonNullType)(type1)) {
        return (0, _definition.isNonNullType)(type2) ? doTypesConflict(type1.ofType, type2.ofType) : true;
      }
      if ((0, _definition.isNonNullType)(type2)) {
        return true;
      }
      if ((0, _definition.isLeafType)(type1) || (0, _definition.isLeafType)(type2)) {
        return type1 !== type2;
      }
      return false;
    }
    function getFieldsAndFragmentNames(context, cachedFieldsAndFragmentNames, parentType, selectionSet) {
      const cached = cachedFieldsAndFragmentNames.get(selectionSet);
      if (cached) {
        return cached;
      }
      const nodeAndDefs = /* @__PURE__ */ Object.create(null);
      const fragmentNames = /* @__PURE__ */ Object.create(null);
      _collectFieldsAndFragmentNames(
        context,
        parentType,
        selectionSet,
        nodeAndDefs,
        fragmentNames
      );
      const result = [nodeAndDefs, Object.keys(fragmentNames)];
      cachedFieldsAndFragmentNames.set(selectionSet, result);
      return result;
    }
    function getReferencedFieldsAndFragmentNames(context, cachedFieldsAndFragmentNames, fragment) {
      const cached = cachedFieldsAndFragmentNames.get(fragment.selectionSet);
      if (cached) {
        return cached;
      }
      const fragmentType = (0, _typeFromAST.typeFromAST)(
        context.getSchema(),
        fragment.typeCondition
      );
      return getFieldsAndFragmentNames(
        context,
        cachedFieldsAndFragmentNames,
        fragmentType,
        fragment.selectionSet
      );
    }
    function _collectFieldsAndFragmentNames(context, parentType, selectionSet, nodeAndDefs, fragmentNames) {
      for (const selection of selectionSet.selections) {
        switch (selection.kind) {
          case _kinds.Kind.FIELD: {
            const fieldName = selection.name.value;
            let fieldDef;
            if ((0, _definition.isObjectType)(parentType) || (0, _definition.isInterfaceType)(parentType)) {
              fieldDef = parentType.getFields()[fieldName];
            }
            const responseName = selection.alias ? selection.alias.value : fieldName;
            if (!nodeAndDefs[responseName]) {
              nodeAndDefs[responseName] = [];
            }
            nodeAndDefs[responseName].push([parentType, selection, fieldDef]);
            break;
          }
          case _kinds.Kind.FRAGMENT_SPREAD:
            fragmentNames[selection.name.value] = true;
            break;
          case _kinds.Kind.INLINE_FRAGMENT: {
            const typeCondition = selection.typeCondition;
            const inlineFragmentType = typeCondition ? (0, _typeFromAST.typeFromAST)(context.getSchema(), typeCondition) : parentType;
            _collectFieldsAndFragmentNames(
              context,
              inlineFragmentType,
              selection.selectionSet,
              nodeAndDefs,
              fragmentNames
            );
            break;
          }
        }
      }
    }
    function subfieldConflicts(conflicts, responseName, node1, node2) {
      if (conflicts.length > 0) {
        return [
          [responseName, conflicts.map(([reason]) => reason)],
          [node1, ...conflicts.map(([, fields1]) => fields1).flat()],
          [node2, ...conflicts.map(([, , fields2]) => fields2).flat()]
        ];
      }
    }
    var PairSet = class {
      constructor() {
        this._data = /* @__PURE__ */ new Map();
      }
      has(a, b, areMutuallyExclusive) {
        var _this$_data$get;
        const [key1, key2] = a < b ? [a, b] : [b, a];
        const result = (_this$_data$get = this._data.get(key1)) === null || _this$_data$get === void 0 ? void 0 : _this$_data$get.get(key2);
        if (result === void 0) {
          return false;
        }
        return areMutuallyExclusive ? true : areMutuallyExclusive === result;
      }
      add(a, b, areMutuallyExclusive) {
        const [key1, key2] = a < b ? [a, b] : [b, a];
        const map = this._data.get(key1);
        if (map === void 0) {
          this._data.set(key1, /* @__PURE__ */ new Map([[key2, areMutuallyExclusive]]));
        } else {
          map.set(key2, areMutuallyExclusive);
        }
      }
    };
  }
});

// node_modules/graphql/validation/rules/PossibleFragmentSpreadsRule.js
var require_PossibleFragmentSpreadsRule = __commonJS({
  "node_modules/graphql/validation/rules/PossibleFragmentSpreadsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.PossibleFragmentSpreadsRule = PossibleFragmentSpreadsRule;
    var _inspect = require_inspect();
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    var _typeComparators = require_typeComparators();
    var _typeFromAST = require_typeFromAST();
    function PossibleFragmentSpreadsRule(context) {
      return {
        InlineFragment(node) {
          const fragType = context.getType();
          const parentType = context.getParentType();
          if ((0, _definition.isCompositeType)(fragType) && (0, _definition.isCompositeType)(parentType) && !(0, _typeComparators.doTypesOverlap)(
            context.getSchema(),
            fragType,
            parentType
          )) {
            const parentTypeStr = (0, _inspect.inspect)(parentType);
            const fragTypeStr = (0, _inspect.inspect)(fragType);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Fragment cannot be spread here as objects of type "${parentTypeStr}" can never be of type "${fragTypeStr}".`,
                {
                  nodes: node
                }
              )
            );
          }
        },
        FragmentSpread(node) {
          const fragName = node.name.value;
          const fragType = getFragmentType(context, fragName);
          const parentType = context.getParentType();
          if (fragType && parentType && !(0, _typeComparators.doTypesOverlap)(
            context.getSchema(),
            fragType,
            parentType
          )) {
            const parentTypeStr = (0, _inspect.inspect)(parentType);
            const fragTypeStr = (0, _inspect.inspect)(fragType);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Fragment "${fragName}" cannot be spread here as objects of type "${parentTypeStr}" can never be of type "${fragTypeStr}".`,
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
    function getFragmentType(context, name) {
      const frag = context.getFragment(name);
      if (frag) {
        const type = (0, _typeFromAST.typeFromAST)(
          context.getSchema(),
          frag.typeCondition
        );
        if ((0, _definition.isCompositeType)(type)) {
          return type;
        }
      }
    }
  }
});

// node_modules/graphql/validation/rules/PossibleTypeExtensionsRule.js
var require_PossibleTypeExtensionsRule = __commonJS({
  "node_modules/graphql/validation/rules/PossibleTypeExtensionsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.PossibleTypeExtensionsRule = PossibleTypeExtensionsRule;
    var _didYouMean = require_didYouMean();
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _predicates = require_predicates();
    var _definition = require_definition();
    function PossibleTypeExtensionsRule(context) {
      const schema = context.getSchema();
      const definedTypes = /* @__PURE__ */ Object.create(null);
      for (const def of context.getDocument().definitions) {
        if ((0, _predicates.isTypeDefinitionNode)(def)) {
          definedTypes[def.name.value] = def;
        }
      }
      return {
        ScalarTypeExtension: checkExtension,
        ObjectTypeExtension: checkExtension,
        InterfaceTypeExtension: checkExtension,
        UnionTypeExtension: checkExtension,
        EnumTypeExtension: checkExtension,
        InputObjectTypeExtension: checkExtension
      };
      function checkExtension(node) {
        const typeName = node.name.value;
        const defNode = definedTypes[typeName];
        const existingType = schema === null || schema === void 0 ? void 0 : schema.getType(typeName);
        let expectedKind;
        if (defNode) {
          expectedKind = defKindToExtKind[defNode.kind];
        } else if (existingType) {
          expectedKind = typeToExtKind(existingType);
        }
        if (expectedKind) {
          if (expectedKind !== node.kind) {
            const kindStr = extensionKindToTypeName(node.kind);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Cannot extend non-${kindStr} type "${typeName}".`,
                {
                  nodes: defNode ? [defNode, node] : node
                }
              )
            );
          }
        } else {
          const allTypeNames = Object.keys({
            ...definedTypes,
            ...schema === null || schema === void 0 ? void 0 : schema.getTypeMap()
          });
          const suggestedTypes = (0, _suggestionList.suggestionList)(
            typeName,
            allTypeNames
          );
          context.reportError(
            new _GraphQLError.GraphQLError(
              `Cannot extend type "${typeName}" because it is not defined.` + (0, _didYouMean.didYouMean)(suggestedTypes),
              {
                nodes: node.name
              }
            )
          );
        }
      }
    }
    var defKindToExtKind = {
      [_kinds.Kind.SCALAR_TYPE_DEFINITION]: _kinds.Kind.SCALAR_TYPE_EXTENSION,
      [_kinds.Kind.OBJECT_TYPE_DEFINITION]: _kinds.Kind.OBJECT_TYPE_EXTENSION,
      [_kinds.Kind.INTERFACE_TYPE_DEFINITION]: _kinds.Kind.INTERFACE_TYPE_EXTENSION,
      [_kinds.Kind.UNION_TYPE_DEFINITION]: _kinds.Kind.UNION_TYPE_EXTENSION,
      [_kinds.Kind.ENUM_TYPE_DEFINITION]: _kinds.Kind.ENUM_TYPE_EXTENSION,
      [_kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION]: _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION
    };
    function typeToExtKind(type) {
      if ((0, _definition.isScalarType)(type)) {
        return _kinds.Kind.SCALAR_TYPE_EXTENSION;
      }
      if ((0, _definition.isObjectType)(type)) {
        return _kinds.Kind.OBJECT_TYPE_EXTENSION;
      }
      if ((0, _definition.isInterfaceType)(type)) {
        return _kinds.Kind.INTERFACE_TYPE_EXTENSION;
      }
      if ((0, _definition.isUnionType)(type)) {
        return _kinds.Kind.UNION_TYPE_EXTENSION;
      }
      if ((0, _definition.isEnumType)(type)) {
        return _kinds.Kind.ENUM_TYPE_EXTENSION;
      }
      if ((0, _definition.isInputObjectType)(type)) {
        return _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION;
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected type: " + (0, _inspect.inspect)(type)
      );
    }
    function extensionKindToTypeName(kind) {
      switch (kind) {
        case _kinds.Kind.SCALAR_TYPE_EXTENSION:
          return "scalar";
        case _kinds.Kind.OBJECT_TYPE_EXTENSION:
          return "object";
        case _kinds.Kind.INTERFACE_TYPE_EXTENSION:
          return "interface";
        case _kinds.Kind.UNION_TYPE_EXTENSION:
          return "union";
        case _kinds.Kind.ENUM_TYPE_EXTENSION:
          return "enum";
        case _kinds.Kind.INPUT_OBJECT_TYPE_EXTENSION:
          return "input object";
        default:
          (0, _invariant.invariant)(
            false,
            "Unexpected kind: " + (0, _inspect.inspect)(kind)
          );
      }
    }
  }
});

// node_modules/graphql/validation/rules/ProvidedRequiredArgumentsRule.js
var require_ProvidedRequiredArgumentsRule = __commonJS({
  "node_modules/graphql/validation/rules/ProvidedRequiredArgumentsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.ProvidedRequiredArgumentsOnDirectivesRule = ProvidedRequiredArgumentsOnDirectivesRule;
    exports2.ProvidedRequiredArgumentsRule = ProvidedRequiredArgumentsRule;
    var _inspect = require_inspect();
    var _keyMap = require_keyMap();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _definition = require_definition();
    var _directives = require_directives();
    function ProvidedRequiredArgumentsRule(context) {
      return {
        // eslint-disable-next-line new-cap
        ...ProvidedRequiredArgumentsOnDirectivesRule(context),
        Field: {
          // Validate on leave to allow for deeper errors to appear first.
          leave(fieldNode) {
            var _fieldNode$arguments;
            const fieldDef = context.getFieldDef();
            if (!fieldDef) {
              return false;
            }
            const providedArgs = new Set(
              // FIXME: https://github.com/graphql/graphql-js/issues/2203
              /* c8 ignore next */
              (_fieldNode$arguments = fieldNode.arguments) === null || _fieldNode$arguments === void 0 ? void 0 : _fieldNode$arguments.map((arg) => arg.name.value)
            );
            for (const argDef of fieldDef.args) {
              if (!providedArgs.has(argDef.name) && (0, _definition.isRequiredArgument)(argDef)) {
                const argTypeStr = (0, _inspect.inspect)(argDef.type);
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    `Field "${fieldDef.name}" argument "${argDef.name}" of type "${argTypeStr}" is required, but it was not provided.`,
                    {
                      nodes: fieldNode
                    }
                  )
                );
              }
            }
          }
        }
      };
    }
    function ProvidedRequiredArgumentsOnDirectivesRule(context) {
      var _schema$getDirectives;
      const requiredArgsMap = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      const definedDirectives = (_schema$getDirectives = schema === null || schema === void 0 ? void 0 : schema.getDirectives()) !== null && _schema$getDirectives !== void 0 ? _schema$getDirectives : _directives.specifiedDirectives;
      for (const directive of definedDirectives) {
        requiredArgsMap[directive.name] = (0, _keyMap.keyMap)(
          directive.args.filter(_definition.isRequiredArgument),
          (arg) => arg.name
        );
      }
      const astDefinitions = context.getDocument().definitions;
      for (const def of astDefinitions) {
        if (def.kind === _kinds.Kind.DIRECTIVE_DEFINITION) {
          var _def$arguments;
          const argNodes = (_def$arguments = def.arguments) !== null && _def$arguments !== void 0 ? _def$arguments : [];
          requiredArgsMap[def.name.value] = (0, _keyMap.keyMap)(
            argNodes.filter(isRequiredArgumentNode),
            (arg) => arg.name.value
          );
        }
      }
      return {
        Directive: {
          // Validate on leave to allow for deeper errors to appear first.
          leave(directiveNode) {
            const directiveName = directiveNode.name.value;
            const requiredArgs = requiredArgsMap[directiveName];
            if (requiredArgs) {
              var _directiveNode$argume;
              const argNodes = (_directiveNode$argume = directiveNode.arguments) !== null && _directiveNode$argume !== void 0 ? _directiveNode$argume : [];
              const argNodeMap = new Set(argNodes.map((arg) => arg.name.value));
              for (const [argName, argDef] of Object.entries(requiredArgs)) {
                if (!argNodeMap.has(argName)) {
                  const argType = (0, _definition.isType)(argDef.type) ? (0, _inspect.inspect)(argDef.type) : (0, _printer.print)(argDef.type);
                  context.reportError(
                    new _GraphQLError.GraphQLError(
                      `Directive "@${directiveName}" argument "${argName}" of type "${argType}" is required, but it was not provided.`,
                      {
                        nodes: directiveNode
                      }
                    )
                  );
                }
              }
            }
          }
        }
      };
    }
    function isRequiredArgumentNode(arg) {
      return arg.type.kind === _kinds.Kind.NON_NULL_TYPE && arg.defaultValue == null;
    }
  }
});

// node_modules/graphql/validation/rules/ScalarLeafsRule.js
var require_ScalarLeafsRule = __commonJS({
  "node_modules/graphql/validation/rules/ScalarLeafsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.ScalarLeafsRule = ScalarLeafsRule;
    var _inspect = require_inspect();
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function ScalarLeafsRule(context) {
      return {
        Field(node) {
          const type = context.getType();
          const selectionSet = node.selectionSet;
          if (type) {
            if ((0, _definition.isLeafType)((0, _definition.getNamedType)(type))) {
              if (selectionSet) {
                const fieldName = node.name.value;
                const typeStr = (0, _inspect.inspect)(type);
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    `Field "${fieldName}" must not have a selection since type "${typeStr}" has no subfields.`,
                    {
                      nodes: selectionSet
                    }
                  )
                );
              }
            } else if (!selectionSet) {
              const fieldName = node.name.value;
              const typeStr = (0, _inspect.inspect)(type);
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Field "${fieldName}" of type "${typeStr}" must have a selection of subfields. Did you mean "${fieldName} { ... }"?`,
                  {
                    nodes: node
                  }
                )
              );
            }
          }
        }
      };
    }
  }
});

// node_modules/graphql/jsutils/printPathArray.js
var require_printPathArray = __commonJS({
  "node_modules/graphql/jsutils/printPathArray.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.printPathArray = printPathArray;
    function printPathArray(path4) {
      return path4.map(
        (key) => typeof key === "number" ? "[" + key.toString() + "]" : "." + key
      ).join("");
    }
  }
});

// node_modules/graphql/jsutils/Path.js
var require_Path = __commonJS({
  "node_modules/graphql/jsutils/Path.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.addPath = addPath;
    exports2.pathToArray = pathToArray;
    function addPath(prev, key, typename) {
      return {
        prev,
        key,
        typename
      };
    }
    function pathToArray(path4) {
      const flattened = [];
      let curr = path4;
      while (curr) {
        flattened.push(curr.key);
        curr = curr.prev;
      }
      return flattened.reverse();
    }
  }
});

// node_modules/graphql/utilities/coerceInputValue.js
var require_coerceInputValue = __commonJS({
  "node_modules/graphql/utilities/coerceInputValue.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.coerceInputValue = coerceInputValue;
    var _didYouMean = require_didYouMean();
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _isIterableObject = require_isIterableObject();
    var _isObjectLike = require_isObjectLike();
    var _Path = require_Path();
    var _printPathArray = require_printPathArray();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function coerceInputValue(inputValue, type, onError = defaultOnError) {
      return coerceInputValueImpl(inputValue, type, onError, void 0);
    }
    function defaultOnError(path4, invalidValue, error) {
      let errorPrefix = "Invalid value " + (0, _inspect.inspect)(invalidValue);
      if (path4.length > 0) {
        errorPrefix += ` at "value${(0, _printPathArray.printPathArray)(path4)}"`;
      }
      error.message = errorPrefix + ": " + error.message;
      throw error;
    }
    function coerceInputValueImpl(inputValue, type, onError, path4) {
      if ((0, _definition.isNonNullType)(type)) {
        if (inputValue != null) {
          return coerceInputValueImpl(inputValue, type.ofType, onError, path4);
        }
        onError(
          (0, _Path.pathToArray)(path4),
          inputValue,
          new _GraphQLError.GraphQLError(
            `Expected non-nullable type "${(0, _inspect.inspect)(
              type
            )}" not to be null.`
          )
        );
        return;
      }
      if (inputValue == null) {
        return null;
      }
      if ((0, _definition.isListType)(type)) {
        const itemType = type.ofType;
        if ((0, _isIterableObject.isIterableObject)(inputValue)) {
          return Array.from(inputValue, (itemValue, index) => {
            const itemPath = (0, _Path.addPath)(path4, index, void 0);
            return coerceInputValueImpl(itemValue, itemType, onError, itemPath);
          });
        }
        return [coerceInputValueImpl(inputValue, itemType, onError, path4)];
      }
      if ((0, _definition.isInputObjectType)(type)) {
        if (!(0, _isObjectLike.isObjectLike)(inputValue)) {
          onError(
            (0, _Path.pathToArray)(path4),
            inputValue,
            new _GraphQLError.GraphQLError(
              `Expected type "${type.name}" to be an object.`
            )
          );
          return;
        }
        const coercedValue = {};
        const fieldDefs = type.getFields();
        for (const field of Object.values(fieldDefs)) {
          const fieldValue = inputValue[field.name];
          if (fieldValue === void 0) {
            if (field.defaultValue !== void 0) {
              coercedValue[field.name] = field.defaultValue;
            } else if ((0, _definition.isNonNullType)(field.type)) {
              const typeStr = (0, _inspect.inspect)(field.type);
              onError(
                (0, _Path.pathToArray)(path4),
                inputValue,
                new _GraphQLError.GraphQLError(
                  `Field "${field.name}" of required type "${typeStr}" was not provided.`
                )
              );
            }
            continue;
          }
          coercedValue[field.name] = coerceInputValueImpl(
            fieldValue,
            field.type,
            onError,
            (0, _Path.addPath)(path4, field.name, type.name)
          );
        }
        for (const fieldName of Object.keys(inputValue)) {
          if (!fieldDefs[fieldName]) {
            const suggestions = (0, _suggestionList.suggestionList)(
              fieldName,
              Object.keys(type.getFields())
            );
            onError(
              (0, _Path.pathToArray)(path4),
              inputValue,
              new _GraphQLError.GraphQLError(
                `Field "${fieldName}" is not defined by type "${type.name}".` + (0, _didYouMean.didYouMean)(suggestions)
              )
            );
          }
        }
        return coercedValue;
      }
      if ((0, _definition.isLeafType)(type)) {
        let parseResult;
        try {
          parseResult = type.parseValue(inputValue);
        } catch (error) {
          if (error instanceof _GraphQLError.GraphQLError) {
            onError((0, _Path.pathToArray)(path4), inputValue, error);
          } else {
            onError(
              (0, _Path.pathToArray)(path4),
              inputValue,
              new _GraphQLError.GraphQLError(
                `Expected type "${type.name}". ` + error.message,
                {
                  originalError: error
                }
              )
            );
          }
          return;
        }
        if (parseResult === void 0) {
          onError(
            (0, _Path.pathToArray)(path4),
            inputValue,
            new _GraphQLError.GraphQLError(`Expected type "${type.name}".`)
          );
        }
        return parseResult;
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected input type: " + (0, _inspect.inspect)(type)
      );
    }
  }
});

// node_modules/graphql/utilities/valueFromAST.js
var require_valueFromAST = __commonJS({
  "node_modules/graphql/utilities/valueFromAST.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.valueFromAST = valueFromAST;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _keyMap = require_keyMap();
    var _kinds = require_kinds();
    var _definition = require_definition();
    function valueFromAST(valueNode, type, variables) {
      if (!valueNode) {
        return;
      }
      if (valueNode.kind === _kinds.Kind.VARIABLE) {
        const variableName = valueNode.name.value;
        if (variables == null || variables[variableName] === void 0) {
          return;
        }
        const variableValue = variables[variableName];
        if (variableValue === null && (0, _definition.isNonNullType)(type)) {
          return;
        }
        return variableValue;
      }
      if ((0, _definition.isNonNullType)(type)) {
        if (valueNode.kind === _kinds.Kind.NULL) {
          return;
        }
        return valueFromAST(valueNode, type.ofType, variables);
      }
      if (valueNode.kind === _kinds.Kind.NULL) {
        return null;
      }
      if ((0, _definition.isListType)(type)) {
        const itemType = type.ofType;
        if (valueNode.kind === _kinds.Kind.LIST) {
          const coercedValues = [];
          for (const itemNode of valueNode.values) {
            if (isMissingVariable(itemNode, variables)) {
              if ((0, _definition.isNonNullType)(itemType)) {
                return;
              }
              coercedValues.push(null);
            } else {
              const itemValue = valueFromAST(itemNode, itemType, variables);
              if (itemValue === void 0) {
                return;
              }
              coercedValues.push(itemValue);
            }
          }
          return coercedValues;
        }
        const coercedValue = valueFromAST(valueNode, itemType, variables);
        if (coercedValue === void 0) {
          return;
        }
        return [coercedValue];
      }
      if ((0, _definition.isInputObjectType)(type)) {
        if (valueNode.kind !== _kinds.Kind.OBJECT) {
          return;
        }
        const coercedObj = /* @__PURE__ */ Object.create(null);
        const fieldNodes = (0, _keyMap.keyMap)(
          valueNode.fields,
          (field) => field.name.value
        );
        for (const field of Object.values(type.getFields())) {
          const fieldNode = fieldNodes[field.name];
          if (!fieldNode || isMissingVariable(fieldNode.value, variables)) {
            if (field.defaultValue !== void 0) {
              coercedObj[field.name] = field.defaultValue;
            } else if ((0, _definition.isNonNullType)(field.type)) {
              return;
            }
            continue;
          }
          const fieldValue = valueFromAST(fieldNode.value, field.type, variables);
          if (fieldValue === void 0) {
            return;
          }
          coercedObj[field.name] = fieldValue;
        }
        return coercedObj;
      }
      if ((0, _definition.isLeafType)(type)) {
        let result;
        try {
          result = type.parseLiteral(valueNode, variables);
        } catch (_error) {
          return;
        }
        if (result === void 0) {
          return;
        }
        return result;
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected input type: " + (0, _inspect.inspect)(type)
      );
    }
    function isMissingVariable(valueNode, variables) {
      return valueNode.kind === _kinds.Kind.VARIABLE && (variables == null || variables[valueNode.name.value] === void 0);
    }
  }
});

// node_modules/graphql/execution/values.js
var require_values = __commonJS({
  "node_modules/graphql/execution/values.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.getArgumentValues = getArgumentValues;
    exports2.getDirectiveValues = getDirectiveValues;
    exports2.getVariableValues = getVariableValues;
    var _inspect = require_inspect();
    var _keyMap = require_keyMap();
    var _printPathArray = require_printPathArray();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _definition = require_definition();
    var _coerceInputValue = require_coerceInputValue();
    var _typeFromAST = require_typeFromAST();
    var _valueFromAST = require_valueFromAST();
    function getVariableValues(schema, varDefNodes, inputs, options) {
      const errors = [];
      const maxErrors = options === null || options === void 0 ? void 0 : options.maxErrors;
      try {
        const coerced = coerceVariableValues(
          schema,
          varDefNodes,
          inputs,
          (error) => {
            if (maxErrors != null && errors.length >= maxErrors) {
              throw new _GraphQLError.GraphQLError(
                "Too many errors processing variables, error limit reached. Execution aborted."
              );
            }
            errors.push(error);
          }
        );
        if (errors.length === 0) {
          return {
            coerced
          };
        }
      } catch (error) {
        errors.push(error);
      }
      return {
        errors
      };
    }
    function coerceVariableValues(schema, varDefNodes, inputs, onError) {
      const coercedValues = {};
      for (const varDefNode of varDefNodes) {
        const varName = varDefNode.variable.name.value;
        const varType = (0, _typeFromAST.typeFromAST)(schema, varDefNode.type);
        if (!(0, _definition.isInputType)(varType)) {
          const varTypeStr = (0, _printer.print)(varDefNode.type);
          onError(
            new _GraphQLError.GraphQLError(
              `Variable "$${varName}" expected value of type "${varTypeStr}" which cannot be used as an input type.`,
              {
                nodes: varDefNode.type
              }
            )
          );
          continue;
        }
        if (!hasOwnProperty(inputs, varName)) {
          if (varDefNode.defaultValue) {
            coercedValues[varName] = (0, _valueFromAST.valueFromAST)(
              varDefNode.defaultValue,
              varType
            );
          } else if ((0, _definition.isNonNullType)(varType)) {
            const varTypeStr = (0, _inspect.inspect)(varType);
            onError(
              new _GraphQLError.GraphQLError(
                `Variable "$${varName}" of required type "${varTypeStr}" was not provided.`,
                {
                  nodes: varDefNode
                }
              )
            );
          }
          continue;
        }
        const value = inputs[varName];
        if (value === null && (0, _definition.isNonNullType)(varType)) {
          const varTypeStr = (0, _inspect.inspect)(varType);
          onError(
            new _GraphQLError.GraphQLError(
              `Variable "$${varName}" of non-null type "${varTypeStr}" must not be null.`,
              {
                nodes: varDefNode
              }
            )
          );
          continue;
        }
        coercedValues[varName] = (0, _coerceInputValue.coerceInputValue)(
          value,
          varType,
          (path4, invalidValue, error) => {
            let prefix = `Variable "$${varName}" got invalid value ` + (0, _inspect.inspect)(invalidValue);
            if (path4.length > 0) {
              prefix += ` at "${varName}${(0, _printPathArray.printPathArray)(
                path4
              )}"`;
            }
            onError(
              new _GraphQLError.GraphQLError(prefix + "; " + error.message, {
                nodes: varDefNode,
                originalError: error
              })
            );
          }
        );
      }
      return coercedValues;
    }
    function getArgumentValues(def, node, variableValues) {
      var _node$arguments;
      const coercedValues = {};
      const argumentNodes = (_node$arguments = node.arguments) !== null && _node$arguments !== void 0 ? _node$arguments : [];
      const argNodeMap = (0, _keyMap.keyMap)(
        argumentNodes,
        (arg) => arg.name.value
      );
      for (const argDef of def.args) {
        const name = argDef.name;
        const argType = argDef.type;
        const argumentNode = argNodeMap[name];
        if (!argumentNode) {
          if (argDef.defaultValue !== void 0) {
            coercedValues[name] = argDef.defaultValue;
          } else if ((0, _definition.isNonNullType)(argType)) {
            throw new _GraphQLError.GraphQLError(
              `Argument "${name}" of required type "${(0, _inspect.inspect)(
                argType
              )}" was not provided.`,
              {
                nodes: node
              }
            );
          }
          continue;
        }
        const valueNode = argumentNode.value;
        let isNull = valueNode.kind === _kinds.Kind.NULL;
        if (valueNode.kind === _kinds.Kind.VARIABLE) {
          const variableName = valueNode.name.value;
          if (variableValues == null || !hasOwnProperty(variableValues, variableName)) {
            if (argDef.defaultValue !== void 0) {
              coercedValues[name] = argDef.defaultValue;
            } else if ((0, _definition.isNonNullType)(argType)) {
              throw new _GraphQLError.GraphQLError(
                `Argument "${name}" of required type "${(0, _inspect.inspect)(
                  argType
                )}" was provided the variable "$${variableName}" which was not provided a runtime value.`,
                {
                  nodes: valueNode
                }
              );
            }
            continue;
          }
          isNull = variableValues[variableName] == null;
        }
        if (isNull && (0, _definition.isNonNullType)(argType)) {
          throw new _GraphQLError.GraphQLError(
            `Argument "${name}" of non-null type "${(0, _inspect.inspect)(
              argType
            )}" must not be null.`,
            {
              nodes: valueNode
            }
          );
        }
        const coercedValue = (0, _valueFromAST.valueFromAST)(
          valueNode,
          argType,
          variableValues
        );
        if (coercedValue === void 0) {
          throw new _GraphQLError.GraphQLError(
            `Argument "${name}" has invalid value ${(0, _printer.print)(
              valueNode
            )}.`,
            {
              nodes: valueNode
            }
          );
        }
        coercedValues[name] = coercedValue;
      }
      return coercedValues;
    }
    function getDirectiveValues(directiveDef, node, variableValues) {
      var _node$directives;
      const directiveNode = (_node$directives = node.directives) === null || _node$directives === void 0 ? void 0 : _node$directives.find(
        (directive) => directive.name.value === directiveDef.name
      );
      if (directiveNode) {
        return getArgumentValues(directiveDef, directiveNode, variableValues);
      }
    }
    function hasOwnProperty(obj, prop) {
      return Object.prototype.hasOwnProperty.call(obj, prop);
    }
  }
});

// node_modules/graphql/execution/collectFields.js
var require_collectFields = __commonJS({
  "node_modules/graphql/execution/collectFields.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.collectFields = collectFields;
    exports2.collectSubfields = collectSubfields;
    var _kinds = require_kinds();
    var _definition = require_definition();
    var _directives = require_directives();
    var _typeFromAST = require_typeFromAST();
    var _values = require_values();
    function collectFields(schema, fragments, variableValues, runtimeType, selectionSet) {
      const fields = /* @__PURE__ */ new Map();
      collectFieldsImpl(
        schema,
        fragments,
        variableValues,
        runtimeType,
        selectionSet,
        fields,
        /* @__PURE__ */ new Set()
      );
      return fields;
    }
    function collectSubfields(schema, fragments, variableValues, returnType, fieldNodes) {
      const subFieldNodes = /* @__PURE__ */ new Map();
      const visitedFragmentNames = /* @__PURE__ */ new Set();
      for (const node of fieldNodes) {
        if (node.selectionSet) {
          collectFieldsImpl(
            schema,
            fragments,
            variableValues,
            returnType,
            node.selectionSet,
            subFieldNodes,
            visitedFragmentNames
          );
        }
      }
      return subFieldNodes;
    }
    function collectFieldsImpl(schema, fragments, variableValues, runtimeType, selectionSet, fields, visitedFragmentNames) {
      for (const selection of selectionSet.selections) {
        switch (selection.kind) {
          case _kinds.Kind.FIELD: {
            if (!shouldIncludeNode(variableValues, selection)) {
              continue;
            }
            const name = getFieldEntryKey(selection);
            const fieldList = fields.get(name);
            if (fieldList !== void 0) {
              fieldList.push(selection);
            } else {
              fields.set(name, [selection]);
            }
            break;
          }
          case _kinds.Kind.INLINE_FRAGMENT: {
            if (!shouldIncludeNode(variableValues, selection) || !doesFragmentConditionMatch(schema, selection, runtimeType)) {
              continue;
            }
            collectFieldsImpl(
              schema,
              fragments,
              variableValues,
              runtimeType,
              selection.selectionSet,
              fields,
              visitedFragmentNames
            );
            break;
          }
          case _kinds.Kind.FRAGMENT_SPREAD: {
            const fragName = selection.name.value;
            if (visitedFragmentNames.has(fragName) || !shouldIncludeNode(variableValues, selection)) {
              continue;
            }
            visitedFragmentNames.add(fragName);
            const fragment = fragments[fragName];
            if (!fragment || !doesFragmentConditionMatch(schema, fragment, runtimeType)) {
              continue;
            }
            collectFieldsImpl(
              schema,
              fragments,
              variableValues,
              runtimeType,
              fragment.selectionSet,
              fields,
              visitedFragmentNames
            );
            break;
          }
        }
      }
    }
    function shouldIncludeNode(variableValues, node) {
      const skip = (0, _values.getDirectiveValues)(
        _directives.GraphQLSkipDirective,
        node,
        variableValues
      );
      if ((skip === null || skip === void 0 ? void 0 : skip.if) === true) {
        return false;
      }
      const include = (0, _values.getDirectiveValues)(
        _directives.GraphQLIncludeDirective,
        node,
        variableValues
      );
      if ((include === null || include === void 0 ? void 0 : include.if) === false) {
        return false;
      }
      return true;
    }
    function doesFragmentConditionMatch(schema, fragment, type) {
      const typeConditionNode = fragment.typeCondition;
      if (!typeConditionNode) {
        return true;
      }
      const conditionalType = (0, _typeFromAST.typeFromAST)(
        schema,
        typeConditionNode
      );
      if (conditionalType === type) {
        return true;
      }
      if ((0, _definition.isAbstractType)(conditionalType)) {
        return schema.isSubType(conditionalType, type);
      }
      return false;
    }
    function getFieldEntryKey(node) {
      return node.alias ? node.alias.value : node.name.value;
    }
  }
});

// node_modules/graphql/validation/rules/SingleFieldSubscriptionsRule.js
var require_SingleFieldSubscriptionsRule = __commonJS({
  "node_modules/graphql/validation/rules/SingleFieldSubscriptionsRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.SingleFieldSubscriptionsRule = SingleFieldSubscriptionsRule;
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _collectFields = require_collectFields();
    function SingleFieldSubscriptionsRule(context) {
      return {
        OperationDefinition(node) {
          if (node.operation === "subscription") {
            const schema = context.getSchema();
            const subscriptionType = schema.getSubscriptionType();
            if (subscriptionType) {
              const operationName = node.name ? node.name.value : null;
              const variableValues = /* @__PURE__ */ Object.create(null);
              const document = context.getDocument();
              const fragments = /* @__PURE__ */ Object.create(null);
              for (const definition of document.definitions) {
                if (definition.kind === _kinds.Kind.FRAGMENT_DEFINITION) {
                  fragments[definition.name.value] = definition;
                }
              }
              const fields = (0, _collectFields.collectFields)(
                schema,
                fragments,
                variableValues,
                subscriptionType,
                node.selectionSet
              );
              if (fields.size > 1) {
                const fieldSelectionLists = [...fields.values()];
                const extraFieldSelectionLists = fieldSelectionLists.slice(1);
                const extraFieldSelections = extraFieldSelectionLists.flat();
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    operationName != null ? `Subscription "${operationName}" must select only one top level field.` : "Anonymous Subscription must select only one top level field.",
                    {
                      nodes: extraFieldSelections
                    }
                  )
                );
              }
              for (const fieldNodes of fields.values()) {
                const field = fieldNodes[0];
                const fieldName = field.name.value;
                if (fieldName.startsWith("__")) {
                  context.reportError(
                    new _GraphQLError.GraphQLError(
                      operationName != null ? `Subscription "${operationName}" must not select an introspection top level field.` : "Anonymous Subscription must not select an introspection top level field.",
                      {
                        nodes: fieldNodes
                      }
                    )
                  );
                }
              }
            }
          }
        }
      };
    }
  }
});

// node_modules/graphql/jsutils/groupBy.js
var require_groupBy = __commonJS({
  "node_modules/graphql/jsutils/groupBy.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.groupBy = groupBy;
    function groupBy(list, keyFn) {
      const result = /* @__PURE__ */ new Map();
      for (const item of list) {
        const key = keyFn(item);
        const group = result.get(key);
        if (group === void 0) {
          result.set(key, [item]);
        } else {
          group.push(item);
        }
      }
      return result;
    }
  }
});

// node_modules/graphql/validation/rules/UniqueArgumentDefinitionNamesRule.js
var require_UniqueArgumentDefinitionNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueArgumentDefinitionNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueArgumentDefinitionNamesRule = UniqueArgumentDefinitionNamesRule;
    var _groupBy = require_groupBy();
    var _GraphQLError = require_GraphQLError();
    function UniqueArgumentDefinitionNamesRule(context) {
      return {
        DirectiveDefinition(directiveNode) {
          var _directiveNode$argume;
          const argumentNodes = (_directiveNode$argume = directiveNode.arguments) !== null && _directiveNode$argume !== void 0 ? _directiveNode$argume : [];
          return checkArgUniqueness(`@${directiveNode.name.value}`, argumentNodes);
        },
        InterfaceTypeDefinition: checkArgUniquenessPerField,
        InterfaceTypeExtension: checkArgUniquenessPerField,
        ObjectTypeDefinition: checkArgUniquenessPerField,
        ObjectTypeExtension: checkArgUniquenessPerField
      };
      function checkArgUniquenessPerField(typeNode) {
        var _typeNode$fields;
        const typeName = typeNode.name.value;
        const fieldNodes = (_typeNode$fields = typeNode.fields) !== null && _typeNode$fields !== void 0 ? _typeNode$fields : [];
        for (const fieldDef of fieldNodes) {
          var _fieldDef$arguments;
          const fieldName = fieldDef.name.value;
          const argumentNodes = (_fieldDef$arguments = fieldDef.arguments) !== null && _fieldDef$arguments !== void 0 ? _fieldDef$arguments : [];
          checkArgUniqueness(`${typeName}.${fieldName}`, argumentNodes);
        }
        return false;
      }
      function checkArgUniqueness(parentName, argumentNodes) {
        const seenArgs = (0, _groupBy.groupBy)(
          argumentNodes,
          (arg) => arg.name.value
        );
        for (const [argName, argNodes] of seenArgs) {
          if (argNodes.length > 1) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Argument "${parentName}(${argName}:)" can only be defined once.`,
                {
                  nodes: argNodes.map((node) => node.name)
                }
              )
            );
          }
        }
        return false;
      }
    }
  }
});

// node_modules/graphql/validation/rules/UniqueArgumentNamesRule.js
var require_UniqueArgumentNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueArgumentNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueArgumentNamesRule = UniqueArgumentNamesRule;
    var _groupBy = require_groupBy();
    var _GraphQLError = require_GraphQLError();
    function UniqueArgumentNamesRule(context) {
      return {
        Field: checkArgUniqueness,
        Directive: checkArgUniqueness
      };
      function checkArgUniqueness(parentNode) {
        var _parentNode$arguments;
        const argumentNodes = (_parentNode$arguments = parentNode.arguments) !== null && _parentNode$arguments !== void 0 ? _parentNode$arguments : [];
        const seenArgs = (0, _groupBy.groupBy)(
          argumentNodes,
          (arg) => arg.name.value
        );
        for (const [argName, argNodes] of seenArgs) {
          if (argNodes.length > 1) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `There can be only one argument named "${argName}".`,
                {
                  nodes: argNodes.map((node) => node.name)
                }
              )
            );
          }
        }
      }
    }
  }
});

// node_modules/graphql/validation/rules/UniqueDirectiveNamesRule.js
var require_UniqueDirectiveNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueDirectiveNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueDirectiveNamesRule = UniqueDirectiveNamesRule;
    var _GraphQLError = require_GraphQLError();
    function UniqueDirectiveNamesRule(context) {
      const knownDirectiveNames = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      return {
        DirectiveDefinition(node) {
          const directiveName = node.name.value;
          if (schema !== null && schema !== void 0 && schema.getDirective(directiveName)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Directive "@${directiveName}" already exists in the schema. It cannot be redefined.`,
                {
                  nodes: node.name
                }
              )
            );
            return;
          }
          if (knownDirectiveNames[directiveName]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `There can be only one directive named "@${directiveName}".`,
                {
                  nodes: [knownDirectiveNames[directiveName], node.name]
                }
              )
            );
          } else {
            knownDirectiveNames[directiveName] = node.name;
          }
          return false;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/UniqueDirectivesPerLocationRule.js
var require_UniqueDirectivesPerLocationRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueDirectivesPerLocationRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueDirectivesPerLocationRule = UniqueDirectivesPerLocationRule;
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _predicates = require_predicates();
    var _directives = require_directives();
    function UniqueDirectivesPerLocationRule(context) {
      const uniqueDirectiveMap = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      const definedDirectives = schema ? schema.getDirectives() : _directives.specifiedDirectives;
      for (const directive of definedDirectives) {
        uniqueDirectiveMap[directive.name] = !directive.isRepeatable;
      }
      const astDefinitions = context.getDocument().definitions;
      for (const def of astDefinitions) {
        if (def.kind === _kinds.Kind.DIRECTIVE_DEFINITION) {
          uniqueDirectiveMap[def.name.value] = !def.repeatable;
        }
      }
      const schemaDirectives = /* @__PURE__ */ Object.create(null);
      const typeDirectivesMap = /* @__PURE__ */ Object.create(null);
      return {
        // Many different AST nodes may contain directives. Rather than listing
        // them all, just listen for entering any node, and check to see if it
        // defines any directives.
        enter(node) {
          if (!("directives" in node) || !node.directives) {
            return;
          }
          let seenDirectives;
          if (node.kind === _kinds.Kind.SCHEMA_DEFINITION || node.kind === _kinds.Kind.SCHEMA_EXTENSION) {
            seenDirectives = schemaDirectives;
          } else if ((0, _predicates.isTypeDefinitionNode)(node) || (0, _predicates.isTypeExtensionNode)(node)) {
            const typeName = node.name.value;
            seenDirectives = typeDirectivesMap[typeName];
            if (seenDirectives === void 0) {
              typeDirectivesMap[typeName] = seenDirectives = /* @__PURE__ */ Object.create(null);
            }
          } else {
            seenDirectives = /* @__PURE__ */ Object.create(null);
          }
          for (const directive of node.directives) {
            const directiveName = directive.name.value;
            if (uniqueDirectiveMap[directiveName]) {
              if (seenDirectives[directiveName]) {
                context.reportError(
                  new _GraphQLError.GraphQLError(
                    `The directive "@${directiveName}" can only be used once at this location.`,
                    {
                      nodes: [seenDirectives[directiveName], directive]
                    }
                  )
                );
              } else {
                seenDirectives[directiveName] = directive;
              }
            }
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/UniqueEnumValueNamesRule.js
var require_UniqueEnumValueNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueEnumValueNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueEnumValueNamesRule = UniqueEnumValueNamesRule;
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function UniqueEnumValueNamesRule(context) {
      const schema = context.getSchema();
      const existingTypeMap = schema ? schema.getTypeMap() : /* @__PURE__ */ Object.create(null);
      const knownValueNames = /* @__PURE__ */ Object.create(null);
      return {
        EnumTypeDefinition: checkValueUniqueness,
        EnumTypeExtension: checkValueUniqueness
      };
      function checkValueUniqueness(node) {
        var _node$values;
        const typeName = node.name.value;
        if (!knownValueNames[typeName]) {
          knownValueNames[typeName] = /* @__PURE__ */ Object.create(null);
        }
        const valueNodes = (_node$values = node.values) !== null && _node$values !== void 0 ? _node$values : [];
        const valueNames = knownValueNames[typeName];
        for (const valueDef of valueNodes) {
          const valueName = valueDef.name.value;
          const existingType = existingTypeMap[typeName];
          if ((0, _definition.isEnumType)(existingType) && existingType.getValue(valueName)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Enum value "${typeName}.${valueName}" already exists in the schema. It cannot also be defined in this type extension.`,
                {
                  nodes: valueDef.name
                }
              )
            );
          } else if (valueNames[valueName]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Enum value "${typeName}.${valueName}" can only be defined once.`,
                {
                  nodes: [valueNames[valueName], valueDef.name]
                }
              )
            );
          } else {
            valueNames[valueName] = valueDef.name;
          }
        }
        return false;
      }
    }
  }
});

// node_modules/graphql/validation/rules/UniqueFieldDefinitionNamesRule.js
var require_UniqueFieldDefinitionNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueFieldDefinitionNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueFieldDefinitionNamesRule = UniqueFieldDefinitionNamesRule;
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function UniqueFieldDefinitionNamesRule(context) {
      const schema = context.getSchema();
      const existingTypeMap = schema ? schema.getTypeMap() : /* @__PURE__ */ Object.create(null);
      const knownFieldNames = /* @__PURE__ */ Object.create(null);
      return {
        InputObjectTypeDefinition: checkFieldUniqueness,
        InputObjectTypeExtension: checkFieldUniqueness,
        InterfaceTypeDefinition: checkFieldUniqueness,
        InterfaceTypeExtension: checkFieldUniqueness,
        ObjectTypeDefinition: checkFieldUniqueness,
        ObjectTypeExtension: checkFieldUniqueness
      };
      function checkFieldUniqueness(node) {
        var _node$fields;
        const typeName = node.name.value;
        if (!knownFieldNames[typeName]) {
          knownFieldNames[typeName] = /* @__PURE__ */ Object.create(null);
        }
        const fieldNodes = (_node$fields = node.fields) !== null && _node$fields !== void 0 ? _node$fields : [];
        const fieldNames = knownFieldNames[typeName];
        for (const fieldDef of fieldNodes) {
          const fieldName = fieldDef.name.value;
          if (hasField(existingTypeMap[typeName], fieldName)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Field "${typeName}.${fieldName}" already exists in the schema. It cannot also be defined in this type extension.`,
                {
                  nodes: fieldDef.name
                }
              )
            );
          } else if (fieldNames[fieldName]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Field "${typeName}.${fieldName}" can only be defined once.`,
                {
                  nodes: [fieldNames[fieldName], fieldDef.name]
                }
              )
            );
          } else {
            fieldNames[fieldName] = fieldDef.name;
          }
        }
        return false;
      }
    }
    function hasField(type, fieldName) {
      if ((0, _definition.isObjectType)(type) || (0, _definition.isInterfaceType)(type) || (0, _definition.isInputObjectType)(type)) {
        return type.getFields()[fieldName] != null;
      }
      return false;
    }
  }
});

// node_modules/graphql/validation/rules/UniqueFragmentNamesRule.js
var require_UniqueFragmentNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueFragmentNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueFragmentNamesRule = UniqueFragmentNamesRule;
    var _GraphQLError = require_GraphQLError();
    function UniqueFragmentNamesRule(context) {
      const knownFragmentNames = /* @__PURE__ */ Object.create(null);
      return {
        OperationDefinition: () => false,
        FragmentDefinition(node) {
          const fragmentName = node.name.value;
          if (knownFragmentNames[fragmentName]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `There can be only one fragment named "${fragmentName}".`,
                {
                  nodes: [knownFragmentNames[fragmentName], node.name]
                }
              )
            );
          } else {
            knownFragmentNames[fragmentName] = node.name;
          }
          return false;
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/UniqueInputFieldNamesRule.js
var require_UniqueInputFieldNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueInputFieldNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueInputFieldNamesRule = UniqueInputFieldNamesRule;
    var _invariant = require_invariant();
    var _GraphQLError = require_GraphQLError();
    function UniqueInputFieldNamesRule(context) {
      const knownNameStack = [];
      let knownNames = /* @__PURE__ */ Object.create(null);
      return {
        ObjectValue: {
          enter() {
            knownNameStack.push(knownNames);
            knownNames = /* @__PURE__ */ Object.create(null);
          },
          leave() {
            const prevKnownNames = knownNameStack.pop();
            prevKnownNames || (0, _invariant.invariant)(false);
            knownNames = prevKnownNames;
          }
        },
        ObjectField(node) {
          const fieldName = node.name.value;
          if (knownNames[fieldName]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `There can be only one input field named "${fieldName}".`,
                {
                  nodes: [knownNames[fieldName], node.name]
                }
              )
            );
          } else {
            knownNames[fieldName] = node.name;
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/UniqueOperationNamesRule.js
var require_UniqueOperationNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueOperationNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueOperationNamesRule = UniqueOperationNamesRule;
    var _GraphQLError = require_GraphQLError();
    function UniqueOperationNamesRule(context) {
      const knownOperationNames = /* @__PURE__ */ Object.create(null);
      return {
        OperationDefinition(node) {
          const operationName = node.name;
          if (operationName) {
            if (knownOperationNames[operationName.value]) {
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `There can be only one operation named "${operationName.value}".`,
                  {
                    nodes: [
                      knownOperationNames[operationName.value],
                      operationName
                    ]
                  }
                )
              );
            } else {
              knownOperationNames[operationName.value] = operationName;
            }
          }
          return false;
        },
        FragmentDefinition: () => false
      };
    }
  }
});

// node_modules/graphql/validation/rules/UniqueOperationTypesRule.js
var require_UniqueOperationTypesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueOperationTypesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueOperationTypesRule = UniqueOperationTypesRule;
    var _GraphQLError = require_GraphQLError();
    function UniqueOperationTypesRule(context) {
      const schema = context.getSchema();
      const definedOperationTypes = /* @__PURE__ */ Object.create(null);
      const existingOperationTypes = schema ? {
        query: schema.getQueryType(),
        mutation: schema.getMutationType(),
        subscription: schema.getSubscriptionType()
      } : {};
      return {
        SchemaDefinition: checkOperationTypes,
        SchemaExtension: checkOperationTypes
      };
      function checkOperationTypes(node) {
        var _node$operationTypes;
        const operationTypesNodes = (_node$operationTypes = node.operationTypes) !== null && _node$operationTypes !== void 0 ? _node$operationTypes : [];
        for (const operationType of operationTypesNodes) {
          const operation = operationType.operation;
          const alreadyDefinedOperationType = definedOperationTypes[operation];
          if (existingOperationTypes[operation]) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Type for ${operation} already defined in the schema. It cannot be redefined.`,
                {
                  nodes: operationType
                }
              )
            );
          } else if (alreadyDefinedOperationType) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `There can be only one ${operation} type in schema.`,
                {
                  nodes: [alreadyDefinedOperationType, operationType]
                }
              )
            );
          } else {
            definedOperationTypes[operation] = operationType;
          }
        }
        return false;
      }
    }
  }
});

// node_modules/graphql/validation/rules/UniqueTypeNamesRule.js
var require_UniqueTypeNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueTypeNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueTypeNamesRule = UniqueTypeNamesRule;
    var _GraphQLError = require_GraphQLError();
    function UniqueTypeNamesRule(context) {
      const knownTypeNames = /* @__PURE__ */ Object.create(null);
      const schema = context.getSchema();
      return {
        ScalarTypeDefinition: checkTypeName,
        ObjectTypeDefinition: checkTypeName,
        InterfaceTypeDefinition: checkTypeName,
        UnionTypeDefinition: checkTypeName,
        EnumTypeDefinition: checkTypeName,
        InputObjectTypeDefinition: checkTypeName
      };
      function checkTypeName(node) {
        const typeName = node.name.value;
        if (schema !== null && schema !== void 0 && schema.getType(typeName)) {
          context.reportError(
            new _GraphQLError.GraphQLError(
              `Type "${typeName}" already exists in the schema. It cannot also be defined in this type definition.`,
              {
                nodes: node.name
              }
            )
          );
          return;
        }
        if (knownTypeNames[typeName]) {
          context.reportError(
            new _GraphQLError.GraphQLError(
              `There can be only one type named "${typeName}".`,
              {
                nodes: [knownTypeNames[typeName], node.name]
              }
            )
          );
        } else {
          knownTypeNames[typeName] = node.name;
        }
        return false;
      }
    }
  }
});

// node_modules/graphql/validation/rules/UniqueVariableNamesRule.js
var require_UniqueVariableNamesRule = __commonJS({
  "node_modules/graphql/validation/rules/UniqueVariableNamesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.UniqueVariableNamesRule = UniqueVariableNamesRule;
    var _groupBy = require_groupBy();
    var _GraphQLError = require_GraphQLError();
    function UniqueVariableNamesRule(context) {
      return {
        OperationDefinition(operationNode) {
          var _operationNode$variab;
          const variableDefinitions = (_operationNode$variab = operationNode.variableDefinitions) !== null && _operationNode$variab !== void 0 ? _operationNode$variab : [];
          const seenVariableDefinitions = (0, _groupBy.groupBy)(
            variableDefinitions,
            (node) => node.variable.name.value
          );
          for (const [variableName, variableNodes] of seenVariableDefinitions) {
            if (variableNodes.length > 1) {
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `There can be only one variable named "$${variableName}".`,
                  {
                    nodes: variableNodes.map((node) => node.variable.name)
                  }
                )
              );
            }
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/ValuesOfCorrectTypeRule.js
var require_ValuesOfCorrectTypeRule = __commonJS({
  "node_modules/graphql/validation/rules/ValuesOfCorrectTypeRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.ValuesOfCorrectTypeRule = ValuesOfCorrectTypeRule;
    var _didYouMean = require_didYouMean();
    var _inspect = require_inspect();
    var _keyMap = require_keyMap();
    var _suggestionList = require_suggestionList();
    var _GraphQLError = require_GraphQLError();
    var _printer = require_printer();
    var _definition = require_definition();
    function ValuesOfCorrectTypeRule(context) {
      return {
        ListValue(node) {
          const type = (0, _definition.getNullableType)(
            context.getParentInputType()
          );
          if (!(0, _definition.isListType)(type)) {
            isValidValueNode(context, node);
            return false;
          }
        },
        ObjectValue(node) {
          const type = (0, _definition.getNamedType)(context.getInputType());
          if (!(0, _definition.isInputObjectType)(type)) {
            isValidValueNode(context, node);
            return false;
          }
          const fieldNodeMap = (0, _keyMap.keyMap)(
            node.fields,
            (field) => field.name.value
          );
          for (const fieldDef of Object.values(type.getFields())) {
            const fieldNode = fieldNodeMap[fieldDef.name];
            if (!fieldNode && (0, _definition.isRequiredInputField)(fieldDef)) {
              const typeStr = (0, _inspect.inspect)(fieldDef.type);
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Field "${type.name}.${fieldDef.name}" of required type "${typeStr}" was not provided.`,
                  {
                    nodes: node
                  }
                )
              );
            }
          }
        },
        ObjectField(node) {
          const parentType = (0, _definition.getNamedType)(
            context.getParentInputType()
          );
          const fieldType = context.getInputType();
          if (!fieldType && (0, _definition.isInputObjectType)(parentType)) {
            const suggestions = (0, _suggestionList.suggestionList)(
              node.name.value,
              Object.keys(parentType.getFields())
            );
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Field "${node.name.value}" is not defined by type "${parentType.name}".` + (0, _didYouMean.didYouMean)(suggestions),
                {
                  nodes: node
                }
              )
            );
          }
        },
        NullValue(node) {
          const type = context.getInputType();
          if ((0, _definition.isNonNullType)(type)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Expected value of type "${(0, _inspect.inspect)(
                  type
                )}", found ${(0, _printer.print)(node)}.`,
                {
                  nodes: node
                }
              )
            );
          }
        },
        EnumValue: (node) => isValidValueNode(context, node),
        IntValue: (node) => isValidValueNode(context, node),
        FloatValue: (node) => isValidValueNode(context, node),
        StringValue: (node) => isValidValueNode(context, node),
        BooleanValue: (node) => isValidValueNode(context, node)
      };
    }
    function isValidValueNode(context, node) {
      const locationType = context.getInputType();
      if (!locationType) {
        return;
      }
      const type = (0, _definition.getNamedType)(locationType);
      if (!(0, _definition.isLeafType)(type)) {
        const typeStr = (0, _inspect.inspect)(locationType);
        context.reportError(
          new _GraphQLError.GraphQLError(
            `Expected value of type "${typeStr}", found ${(0, _printer.print)(
              node
            )}.`,
            {
              nodes: node
            }
          )
        );
        return;
      }
      try {
        const parseResult = type.parseLiteral(
          node,
          void 0
          /* variables */
        );
        if (parseResult === void 0) {
          const typeStr = (0, _inspect.inspect)(locationType);
          context.reportError(
            new _GraphQLError.GraphQLError(
              `Expected value of type "${typeStr}", found ${(0, _printer.print)(
                node
              )}.`,
              {
                nodes: node
              }
            )
          );
        }
      } catch (error) {
        const typeStr = (0, _inspect.inspect)(locationType);
        if (error instanceof _GraphQLError.GraphQLError) {
          context.reportError(error);
        } else {
          context.reportError(
            new _GraphQLError.GraphQLError(
              `Expected value of type "${typeStr}", found ${(0, _printer.print)(
                node
              )}; ` + error.message,
              {
                nodes: node,
                originalError: error
              }
            )
          );
        }
      }
    }
  }
});

// node_modules/graphql/validation/rules/VariablesAreInputTypesRule.js
var require_VariablesAreInputTypesRule = __commonJS({
  "node_modules/graphql/validation/rules/VariablesAreInputTypesRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.VariablesAreInputTypesRule = VariablesAreInputTypesRule;
    var _GraphQLError = require_GraphQLError();
    var _printer = require_printer();
    var _definition = require_definition();
    var _typeFromAST = require_typeFromAST();
    function VariablesAreInputTypesRule(context) {
      return {
        VariableDefinition(node) {
          const type = (0, _typeFromAST.typeFromAST)(
            context.getSchema(),
            node.type
          );
          if (type !== void 0 && !(0, _definition.isInputType)(type)) {
            const variableName = node.variable.name.value;
            const typeName = (0, _printer.print)(node.type);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `Variable "$${variableName}" cannot be non-input type "${typeName}".`,
                {
                  nodes: node.type
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/VariablesInAllowedPositionRule.js
var require_VariablesInAllowedPositionRule = __commonJS({
  "node_modules/graphql/validation/rules/VariablesInAllowedPositionRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.VariablesInAllowedPositionRule = VariablesInAllowedPositionRule;
    var _inspect = require_inspect();
    var _GraphQLError = require_GraphQLError();
    var _kinds = require_kinds();
    var _definition = require_definition();
    var _typeComparators = require_typeComparators();
    var _typeFromAST = require_typeFromAST();
    function VariablesInAllowedPositionRule(context) {
      let varDefMap = /* @__PURE__ */ Object.create(null);
      return {
        OperationDefinition: {
          enter() {
            varDefMap = /* @__PURE__ */ Object.create(null);
          },
          leave(operation) {
            const usages = context.getRecursiveVariableUsages(operation);
            for (const { node, type, defaultValue } of usages) {
              const varName = node.name.value;
              const varDef = varDefMap[varName];
              if (varDef && type) {
                const schema = context.getSchema();
                const varType = (0, _typeFromAST.typeFromAST)(schema, varDef.type);
                if (varType && !allowedVariableUsage(
                  schema,
                  varType,
                  varDef.defaultValue,
                  type,
                  defaultValue
                )) {
                  const varTypeStr = (0, _inspect.inspect)(varType);
                  const typeStr = (0, _inspect.inspect)(type);
                  context.reportError(
                    new _GraphQLError.GraphQLError(
                      `Variable "$${varName}" of type "${varTypeStr}" used in position expecting type "${typeStr}".`,
                      {
                        nodes: [varDef, node]
                      }
                    )
                  );
                }
              }
            }
          }
        },
        VariableDefinition(node) {
          varDefMap[node.variable.name.value] = node;
        }
      };
    }
    function allowedVariableUsage(schema, varType, varDefaultValue, locationType, locationDefaultValue) {
      if ((0, _definition.isNonNullType)(locationType) && !(0, _definition.isNonNullType)(varType)) {
        const hasNonNullVariableDefaultValue = varDefaultValue != null && varDefaultValue.kind !== _kinds.Kind.NULL;
        const hasLocationDefaultValue = locationDefaultValue !== void 0;
        if (!hasNonNullVariableDefaultValue && !hasLocationDefaultValue) {
          return false;
        }
        const nullableLocationType = locationType.ofType;
        return (0, _typeComparators.isTypeSubTypeOf)(
          schema,
          varType,
          nullableLocationType
        );
      }
      return (0, _typeComparators.isTypeSubTypeOf)(schema, varType, locationType);
    }
  }
});

// node_modules/graphql/validation/specifiedRules.js
var require_specifiedRules = __commonJS({
  "node_modules/graphql/validation/specifiedRules.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.specifiedSDLRules = exports2.specifiedRules = void 0;
    var _ExecutableDefinitionsRule = require_ExecutableDefinitionsRule();
    var _FieldsOnCorrectTypeRule = require_FieldsOnCorrectTypeRule();
    var _FragmentsOnCompositeTypesRule = require_FragmentsOnCompositeTypesRule();
    var _KnownArgumentNamesRule = require_KnownArgumentNamesRule();
    var _KnownDirectivesRule = require_KnownDirectivesRule();
    var _KnownFragmentNamesRule = require_KnownFragmentNamesRule();
    var _KnownTypeNamesRule = require_KnownTypeNamesRule();
    var _LoneAnonymousOperationRule = require_LoneAnonymousOperationRule();
    var _LoneSchemaDefinitionRule = require_LoneSchemaDefinitionRule();
    var _NoFragmentCyclesRule = require_NoFragmentCyclesRule();
    var _NoUndefinedVariablesRule = require_NoUndefinedVariablesRule();
    var _NoUnusedFragmentsRule = require_NoUnusedFragmentsRule();
    var _NoUnusedVariablesRule = require_NoUnusedVariablesRule();
    var _OverlappingFieldsCanBeMergedRule = require_OverlappingFieldsCanBeMergedRule();
    var _PossibleFragmentSpreadsRule = require_PossibleFragmentSpreadsRule();
    var _PossibleTypeExtensionsRule = require_PossibleTypeExtensionsRule();
    var _ProvidedRequiredArgumentsRule = require_ProvidedRequiredArgumentsRule();
    var _ScalarLeafsRule = require_ScalarLeafsRule();
    var _SingleFieldSubscriptionsRule = require_SingleFieldSubscriptionsRule();
    var _UniqueArgumentDefinitionNamesRule = require_UniqueArgumentDefinitionNamesRule();
    var _UniqueArgumentNamesRule = require_UniqueArgumentNamesRule();
    var _UniqueDirectiveNamesRule = require_UniqueDirectiveNamesRule();
    var _UniqueDirectivesPerLocationRule = require_UniqueDirectivesPerLocationRule();
    var _UniqueEnumValueNamesRule = require_UniqueEnumValueNamesRule();
    var _UniqueFieldDefinitionNamesRule = require_UniqueFieldDefinitionNamesRule();
    var _UniqueFragmentNamesRule = require_UniqueFragmentNamesRule();
    var _UniqueInputFieldNamesRule = require_UniqueInputFieldNamesRule();
    var _UniqueOperationNamesRule = require_UniqueOperationNamesRule();
    var _UniqueOperationTypesRule = require_UniqueOperationTypesRule();
    var _UniqueTypeNamesRule = require_UniqueTypeNamesRule();
    var _UniqueVariableNamesRule = require_UniqueVariableNamesRule();
    var _ValuesOfCorrectTypeRule = require_ValuesOfCorrectTypeRule();
    var _VariablesAreInputTypesRule = require_VariablesAreInputTypesRule();
    var _VariablesInAllowedPositionRule = require_VariablesInAllowedPositionRule();
    var specifiedRules = Object.freeze([
      _ExecutableDefinitionsRule.ExecutableDefinitionsRule,
      _UniqueOperationNamesRule.UniqueOperationNamesRule,
      _LoneAnonymousOperationRule.LoneAnonymousOperationRule,
      _SingleFieldSubscriptionsRule.SingleFieldSubscriptionsRule,
      _KnownTypeNamesRule.KnownTypeNamesRule,
      _FragmentsOnCompositeTypesRule.FragmentsOnCompositeTypesRule,
      _VariablesAreInputTypesRule.VariablesAreInputTypesRule,
      _ScalarLeafsRule.ScalarLeafsRule,
      _FieldsOnCorrectTypeRule.FieldsOnCorrectTypeRule,
      _UniqueFragmentNamesRule.UniqueFragmentNamesRule,
      _KnownFragmentNamesRule.KnownFragmentNamesRule,
      _NoUnusedFragmentsRule.NoUnusedFragmentsRule,
      _PossibleFragmentSpreadsRule.PossibleFragmentSpreadsRule,
      _NoFragmentCyclesRule.NoFragmentCyclesRule,
      _UniqueVariableNamesRule.UniqueVariableNamesRule,
      _NoUndefinedVariablesRule.NoUndefinedVariablesRule,
      _NoUnusedVariablesRule.NoUnusedVariablesRule,
      _KnownDirectivesRule.KnownDirectivesRule,
      _UniqueDirectivesPerLocationRule.UniqueDirectivesPerLocationRule,
      _KnownArgumentNamesRule.KnownArgumentNamesRule,
      _UniqueArgumentNamesRule.UniqueArgumentNamesRule,
      _ValuesOfCorrectTypeRule.ValuesOfCorrectTypeRule,
      _ProvidedRequiredArgumentsRule.ProvidedRequiredArgumentsRule,
      _VariablesInAllowedPositionRule.VariablesInAllowedPositionRule,
      _OverlappingFieldsCanBeMergedRule.OverlappingFieldsCanBeMergedRule,
      _UniqueInputFieldNamesRule.UniqueInputFieldNamesRule
    ]);
    exports2.specifiedRules = specifiedRules;
    var specifiedSDLRules = Object.freeze([
      _LoneSchemaDefinitionRule.LoneSchemaDefinitionRule,
      _UniqueOperationTypesRule.UniqueOperationTypesRule,
      _UniqueTypeNamesRule.UniqueTypeNamesRule,
      _UniqueEnumValueNamesRule.UniqueEnumValueNamesRule,
      _UniqueFieldDefinitionNamesRule.UniqueFieldDefinitionNamesRule,
      _UniqueArgumentDefinitionNamesRule.UniqueArgumentDefinitionNamesRule,
      _UniqueDirectiveNamesRule.UniqueDirectiveNamesRule,
      _KnownTypeNamesRule.KnownTypeNamesRule,
      _KnownDirectivesRule.KnownDirectivesRule,
      _UniqueDirectivesPerLocationRule.UniqueDirectivesPerLocationRule,
      _PossibleTypeExtensionsRule.PossibleTypeExtensionsRule,
      _KnownArgumentNamesRule.KnownArgumentNamesOnDirectivesRule,
      _UniqueArgumentNamesRule.UniqueArgumentNamesRule,
      _UniqueInputFieldNamesRule.UniqueInputFieldNamesRule,
      _ProvidedRequiredArgumentsRule.ProvidedRequiredArgumentsOnDirectivesRule
    ]);
    exports2.specifiedSDLRules = specifiedSDLRules;
  }
});

// node_modules/graphql/validation/ValidationContext.js
var require_ValidationContext = __commonJS({
  "node_modules/graphql/validation/ValidationContext.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.ValidationContext = exports2.SDLValidationContext = exports2.ASTValidationContext = void 0;
    var _kinds = require_kinds();
    var _visitor = require_visitor();
    var _TypeInfo = require_TypeInfo();
    var ASTValidationContext = class {
      constructor(ast, onError) {
        this._ast = ast;
        this._fragments = void 0;
        this._fragmentSpreads = /* @__PURE__ */ new Map();
        this._recursivelyReferencedFragments = /* @__PURE__ */ new Map();
        this._onError = onError;
      }
      get [Symbol.toStringTag]() {
        return "ASTValidationContext";
      }
      reportError(error) {
        this._onError(error);
      }
      getDocument() {
        return this._ast;
      }
      getFragment(name) {
        let fragments;
        if (this._fragments) {
          fragments = this._fragments;
        } else {
          fragments = /* @__PURE__ */ Object.create(null);
          for (const defNode of this.getDocument().definitions) {
            if (defNode.kind === _kinds.Kind.FRAGMENT_DEFINITION) {
              fragments[defNode.name.value] = defNode;
            }
          }
          this._fragments = fragments;
        }
        return fragments[name];
      }
      getFragmentSpreads(node) {
        let spreads = this._fragmentSpreads.get(node);
        if (!spreads) {
          spreads = [];
          const setsToVisit = [node];
          let set;
          while (set = setsToVisit.pop()) {
            for (const selection of set.selections) {
              if (selection.kind === _kinds.Kind.FRAGMENT_SPREAD) {
                spreads.push(selection);
              } else if (selection.selectionSet) {
                setsToVisit.push(selection.selectionSet);
              }
            }
          }
          this._fragmentSpreads.set(node, spreads);
        }
        return spreads;
      }
      getRecursivelyReferencedFragments(operation) {
        let fragments = this._recursivelyReferencedFragments.get(operation);
        if (!fragments) {
          fragments = [];
          const collectedNames = /* @__PURE__ */ Object.create(null);
          const nodesToVisit = [operation.selectionSet];
          let node;
          while (node = nodesToVisit.pop()) {
            for (const spread of this.getFragmentSpreads(node)) {
              const fragName = spread.name.value;
              if (collectedNames[fragName] !== true) {
                collectedNames[fragName] = true;
                const fragment = this.getFragment(fragName);
                if (fragment) {
                  fragments.push(fragment);
                  nodesToVisit.push(fragment.selectionSet);
                }
              }
            }
          }
          this._recursivelyReferencedFragments.set(operation, fragments);
        }
        return fragments;
      }
    };
    exports2.ASTValidationContext = ASTValidationContext;
    var SDLValidationContext = class extends ASTValidationContext {
      constructor(ast, schema, onError) {
        super(ast, onError);
        this._schema = schema;
      }
      get [Symbol.toStringTag]() {
        return "SDLValidationContext";
      }
      getSchema() {
        return this._schema;
      }
    };
    exports2.SDLValidationContext = SDLValidationContext;
    var ValidationContext = class extends ASTValidationContext {
      constructor(schema, ast, typeInfo, onError) {
        super(ast, onError);
        this._schema = schema;
        this._typeInfo = typeInfo;
        this._variableUsages = /* @__PURE__ */ new Map();
        this._recursiveVariableUsages = /* @__PURE__ */ new Map();
      }
      get [Symbol.toStringTag]() {
        return "ValidationContext";
      }
      getSchema() {
        return this._schema;
      }
      getVariableUsages(node) {
        let usages = this._variableUsages.get(node);
        if (!usages) {
          const newUsages = [];
          const typeInfo = new _TypeInfo.TypeInfo(this._schema);
          (0, _visitor.visit)(
            node,
            (0, _TypeInfo.visitWithTypeInfo)(typeInfo, {
              VariableDefinition: () => false,
              Variable(variable) {
                newUsages.push({
                  node: variable,
                  type: typeInfo.getInputType(),
                  defaultValue: typeInfo.getDefaultValue()
                });
              }
            })
          );
          usages = newUsages;
          this._variableUsages.set(node, usages);
        }
        return usages;
      }
      getRecursiveVariableUsages(operation) {
        let usages = this._recursiveVariableUsages.get(operation);
        if (!usages) {
          usages = this.getVariableUsages(operation);
          for (const frag of this.getRecursivelyReferencedFragments(operation)) {
            usages = usages.concat(this.getVariableUsages(frag));
          }
          this._recursiveVariableUsages.set(operation, usages);
        }
        return usages;
      }
      getType() {
        return this._typeInfo.getType();
      }
      getParentType() {
        return this._typeInfo.getParentType();
      }
      getInputType() {
        return this._typeInfo.getInputType();
      }
      getParentInputType() {
        return this._typeInfo.getParentInputType();
      }
      getFieldDef() {
        return this._typeInfo.getFieldDef();
      }
      getDirective() {
        return this._typeInfo.getDirective();
      }
      getArgument() {
        return this._typeInfo.getArgument();
      }
      getEnumValue() {
        return this._typeInfo.getEnumValue();
      }
    };
    exports2.ValidationContext = ValidationContext;
  }
});

// node_modules/graphql/validation/validate.js
var require_validate2 = __commonJS({
  "node_modules/graphql/validation/validate.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.assertValidSDL = assertValidSDL;
    exports2.assertValidSDLExtension = assertValidSDLExtension;
    exports2.validate = validate;
    exports2.validateSDL = validateSDL;
    var _devAssert = require_devAssert();
    var _GraphQLError = require_GraphQLError();
    var _visitor = require_visitor();
    var _validate = require_validate();
    var _TypeInfo = require_TypeInfo();
    var _specifiedRules = require_specifiedRules();
    var _ValidationContext = require_ValidationContext();
    function validate(schema, documentAST, rules = _specifiedRules.specifiedRules, options, typeInfo = new _TypeInfo.TypeInfo(schema)) {
      var _options$maxErrors;
      const maxErrors = (_options$maxErrors = options === null || options === void 0 ? void 0 : options.maxErrors) !== null && _options$maxErrors !== void 0 ? _options$maxErrors : 100;
      documentAST || (0, _devAssert.devAssert)(false, "Must provide document.");
      (0, _validate.assertValidSchema)(schema);
      const abortObj = Object.freeze({});
      const errors = [];
      const context = new _ValidationContext.ValidationContext(
        schema,
        documentAST,
        typeInfo,
        (error) => {
          if (errors.length >= maxErrors) {
            errors.push(
              new _GraphQLError.GraphQLError(
                "Too many validation errors, error limit reached. Validation aborted."
              )
            );
            throw abortObj;
          }
          errors.push(error);
        }
      );
      const visitor = (0, _visitor.visitInParallel)(
        rules.map((rule) => rule(context))
      );
      try {
        (0, _visitor.visit)(
          documentAST,
          (0, _TypeInfo.visitWithTypeInfo)(typeInfo, visitor)
        );
      } catch (e) {
        if (e !== abortObj) {
          throw e;
        }
      }
      return errors;
    }
    function validateSDL(documentAST, schemaToExtend, rules = _specifiedRules.specifiedSDLRules) {
      const errors = [];
      const context = new _ValidationContext.SDLValidationContext(
        documentAST,
        schemaToExtend,
        (error) => {
          errors.push(error);
        }
      );
      const visitors = rules.map((rule) => rule(context));
      (0, _visitor.visit)(documentAST, (0, _visitor.visitInParallel)(visitors));
      return errors;
    }
    function assertValidSDL(documentAST) {
      const errors = validateSDL(documentAST);
      if (errors.length !== 0) {
        throw new Error(errors.map((error) => error.message).join("\n\n"));
      }
    }
    function assertValidSDLExtension(documentAST, schema) {
      const errors = validateSDL(documentAST, schema);
      if (errors.length !== 0) {
        throw new Error(errors.map((error) => error.message).join("\n\n"));
      }
    }
  }
});

// node_modules/graphql/jsutils/memoize3.js
var require_memoize3 = __commonJS({
  "node_modules/graphql/jsutils/memoize3.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.memoize3 = memoize3;
    function memoize3(fn) {
      let cache0;
      return function memoized(a1, a2, a3) {
        if (cache0 === void 0) {
          cache0 = /* @__PURE__ */ new WeakMap();
        }
        let cache1 = cache0.get(a1);
        if (cache1 === void 0) {
          cache1 = /* @__PURE__ */ new WeakMap();
          cache0.set(a1, cache1);
        }
        let cache2 = cache1.get(a2);
        if (cache2 === void 0) {
          cache2 = /* @__PURE__ */ new WeakMap();
          cache1.set(a2, cache2);
        }
        let fnResult = cache2.get(a3);
        if (fnResult === void 0) {
          fnResult = fn(a1, a2, a3);
          cache2.set(a3, fnResult);
        }
        return fnResult;
      };
    }
  }
});

// node_modules/graphql/jsutils/promiseForObject.js
var require_promiseForObject = __commonJS({
  "node_modules/graphql/jsutils/promiseForObject.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.promiseForObject = promiseForObject;
    function promiseForObject(object) {
      return Promise.all(Object.values(object)).then((resolvedValues) => {
        const resolvedObject = /* @__PURE__ */ Object.create(null);
        for (const [i, key] of Object.keys(object).entries()) {
          resolvedObject[key] = resolvedValues[i];
        }
        return resolvedObject;
      });
    }
  }
});

// node_modules/graphql/jsutils/promiseReduce.js
var require_promiseReduce = __commonJS({
  "node_modules/graphql/jsutils/promiseReduce.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.promiseReduce = promiseReduce;
    var _isPromise = require_isPromise();
    function promiseReduce(values, callbackFn, initialValue) {
      let accumulator = initialValue;
      for (const value of values) {
        accumulator = (0, _isPromise.isPromise)(accumulator) ? accumulator.then((resolved) => callbackFn(resolved, value)) : callbackFn(accumulator, value);
      }
      return accumulator;
    }
  }
});

// node_modules/graphql/jsutils/toError.js
var require_toError = __commonJS({
  "node_modules/graphql/jsutils/toError.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.toError = toError;
    var _inspect = require_inspect();
    function toError(thrownValue) {
      return thrownValue instanceof Error ? thrownValue : new NonErrorThrown(thrownValue);
    }
    var NonErrorThrown = class extends Error {
      constructor(thrownValue) {
        super("Unexpected error value: " + (0, _inspect.inspect)(thrownValue));
        this.name = "NonErrorThrown";
        this.thrownValue = thrownValue;
      }
    };
  }
});

// node_modules/graphql/error/locatedError.js
var require_locatedError = __commonJS({
  "node_modules/graphql/error/locatedError.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.locatedError = locatedError;
    var _toError = require_toError();
    var _GraphQLError = require_GraphQLError();
    function locatedError(rawOriginalError, nodes, path4) {
      var _nodes;
      const originalError = (0, _toError.toError)(rawOriginalError);
      if (isLocatedGraphQLError(originalError)) {
        return originalError;
      }
      return new _GraphQLError.GraphQLError(originalError.message, {
        nodes: (_nodes = originalError.nodes) !== null && _nodes !== void 0 ? _nodes : nodes,
        source: originalError.source,
        positions: originalError.positions,
        path: path4,
        originalError
      });
    }
    function isLocatedGraphQLError(error) {
      return Array.isArray(error.path);
    }
  }
});

// node_modules/graphql/execution/execute.js
var require_execute = __commonJS({
  "node_modules/graphql/execution/execute.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.assertValidExecutionArguments = assertValidExecutionArguments;
    exports2.buildExecutionContext = buildExecutionContext;
    exports2.buildResolveInfo = buildResolveInfo;
    exports2.defaultTypeResolver = exports2.defaultFieldResolver = void 0;
    exports2.execute = execute;
    exports2.executeSync = executeSync;
    exports2.getFieldDef = getFieldDef;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _isIterableObject = require_isIterableObject();
    var _isObjectLike = require_isObjectLike();
    var _isPromise = require_isPromise();
    var _memoize = require_memoize3();
    var _Path = require_Path();
    var _promiseForObject = require_promiseForObject();
    var _promiseReduce = require_promiseReduce();
    var _GraphQLError = require_GraphQLError();
    var _locatedError = require_locatedError();
    var _ast = require_ast();
    var _kinds = require_kinds();
    var _definition = require_definition();
    var _introspection = require_introspection();
    var _validate = require_validate();
    var _collectFields = require_collectFields();
    var _values = require_values();
    var collectSubfields = (0, _memoize.memoize3)(
      (exeContext, returnType, fieldNodes) => (0, _collectFields.collectSubfields)(
        exeContext.schema,
        exeContext.fragments,
        exeContext.variableValues,
        returnType,
        fieldNodes
      )
    );
    function execute(args) {
      arguments.length < 2 || (0, _devAssert.devAssert)(
        false,
        "graphql@16 dropped long-deprecated support for positional arguments, please pass an object instead."
      );
      const { schema, document, variableValues, rootValue } = args;
      assertValidExecutionArguments(schema, document, variableValues);
      const exeContext = buildExecutionContext(args);
      if (!("schema" in exeContext)) {
        return {
          errors: exeContext
        };
      }
      try {
        const { operation } = exeContext;
        const result = executeOperation(exeContext, operation, rootValue);
        if ((0, _isPromise.isPromise)(result)) {
          return result.then(
            (data) => buildResponse(data, exeContext.errors),
            (error) => {
              exeContext.errors.push(error);
              return buildResponse(null, exeContext.errors);
            }
          );
        }
        return buildResponse(result, exeContext.errors);
      } catch (error) {
        exeContext.errors.push(error);
        return buildResponse(null, exeContext.errors);
      }
    }
    function executeSync(args) {
      const result = execute(args);
      if ((0, _isPromise.isPromise)(result)) {
        throw new Error("GraphQL execution failed to complete synchronously.");
      }
      return result;
    }
    function buildResponse(data, errors) {
      return errors.length === 0 ? {
        data
      } : {
        errors,
        data
      };
    }
    function assertValidExecutionArguments(schema, document, rawVariableValues) {
      document || (0, _devAssert.devAssert)(false, "Must provide document.");
      (0, _validate.assertValidSchema)(schema);
      rawVariableValues == null || (0, _isObjectLike.isObjectLike)(rawVariableValues) || (0, _devAssert.devAssert)(
        false,
        "Variables must be provided as an Object where each property is a variable value. Perhaps look to see if an unparsed JSON string was provided."
      );
    }
    function buildExecutionContext(args) {
      var _definition$name, _operation$variableDe;
      const {
        schema,
        document,
        rootValue,
        contextValue,
        variableValues: rawVariableValues,
        operationName,
        fieldResolver,
        typeResolver,
        subscribeFieldResolver
      } = args;
      let operation;
      const fragments = /* @__PURE__ */ Object.create(null);
      for (const definition of document.definitions) {
        switch (definition.kind) {
          case _kinds.Kind.OPERATION_DEFINITION:
            if (operationName == null) {
              if (operation !== void 0) {
                return [
                  new _GraphQLError.GraphQLError(
                    "Must provide operation name if query contains multiple operations."
                  )
                ];
              }
              operation = definition;
            } else if (((_definition$name = definition.name) === null || _definition$name === void 0 ? void 0 : _definition$name.value) === operationName) {
              operation = definition;
            }
            break;
          case _kinds.Kind.FRAGMENT_DEFINITION:
            fragments[definition.name.value] = definition;
            break;
          default:
        }
      }
      if (!operation) {
        if (operationName != null) {
          return [
            new _GraphQLError.GraphQLError(
              `Unknown operation named "${operationName}".`
            )
          ];
        }
        return [new _GraphQLError.GraphQLError("Must provide an operation.")];
      }
      const variableDefinitions = (_operation$variableDe = operation.variableDefinitions) !== null && _operation$variableDe !== void 0 ? _operation$variableDe : [];
      const coercedVariableValues = (0, _values.getVariableValues)(
        schema,
        variableDefinitions,
        rawVariableValues !== null && rawVariableValues !== void 0 ? rawVariableValues : {},
        {
          maxErrors: 50
        }
      );
      if (coercedVariableValues.errors) {
        return coercedVariableValues.errors;
      }
      return {
        schema,
        fragments,
        rootValue,
        contextValue,
        operation,
        variableValues: coercedVariableValues.coerced,
        fieldResolver: fieldResolver !== null && fieldResolver !== void 0 ? fieldResolver : defaultFieldResolver,
        typeResolver: typeResolver !== null && typeResolver !== void 0 ? typeResolver : defaultTypeResolver,
        subscribeFieldResolver: subscribeFieldResolver !== null && subscribeFieldResolver !== void 0 ? subscribeFieldResolver : defaultFieldResolver,
        errors: []
      };
    }
    function executeOperation(exeContext, operation, rootValue) {
      const rootType = exeContext.schema.getRootType(operation.operation);
      if (rootType == null) {
        throw new _GraphQLError.GraphQLError(
          `Schema is not configured to execute ${operation.operation} operation.`,
          {
            nodes: operation
          }
        );
      }
      const rootFields = (0, _collectFields.collectFields)(
        exeContext.schema,
        exeContext.fragments,
        exeContext.variableValues,
        rootType,
        operation.selectionSet
      );
      const path4 = void 0;
      switch (operation.operation) {
        case _ast.OperationTypeNode.QUERY:
          return executeFields(exeContext, rootType, rootValue, path4, rootFields);
        case _ast.OperationTypeNode.MUTATION:
          return executeFieldsSerially(
            exeContext,
            rootType,
            rootValue,
            path4,
            rootFields
          );
        case _ast.OperationTypeNode.SUBSCRIPTION:
          return executeFields(exeContext, rootType, rootValue, path4, rootFields);
      }
    }
    function executeFieldsSerially(exeContext, parentType, sourceValue, path4, fields) {
      return (0, _promiseReduce.promiseReduce)(
        fields.entries(),
        (results, [responseName, fieldNodes]) => {
          const fieldPath = (0, _Path.addPath)(path4, responseName, parentType.name);
          const result = executeField(
            exeContext,
            parentType,
            sourceValue,
            fieldNodes,
            fieldPath
          );
          if (result === void 0) {
            return results;
          }
          if ((0, _isPromise.isPromise)(result)) {
            return result.then((resolvedResult) => {
              results[responseName] = resolvedResult;
              return results;
            });
          }
          results[responseName] = result;
          return results;
        },
        /* @__PURE__ */ Object.create(null)
      );
    }
    function executeFields(exeContext, parentType, sourceValue, path4, fields) {
      const results = /* @__PURE__ */ Object.create(null);
      let containsPromise = false;
      try {
        for (const [responseName, fieldNodes] of fields.entries()) {
          const fieldPath = (0, _Path.addPath)(path4, responseName, parentType.name);
          const result = executeField(
            exeContext,
            parentType,
            sourceValue,
            fieldNodes,
            fieldPath
          );
          if (result !== void 0) {
            results[responseName] = result;
            if ((0, _isPromise.isPromise)(result)) {
              containsPromise = true;
            }
          }
        }
      } catch (error) {
        if (containsPromise) {
          return (0, _promiseForObject.promiseForObject)(results).finally(() => {
            throw error;
          });
        }
        throw error;
      }
      if (!containsPromise) {
        return results;
      }
      return (0, _promiseForObject.promiseForObject)(results);
    }
    function executeField(exeContext, parentType, source, fieldNodes, path4) {
      var _fieldDef$resolve;
      const fieldDef = getFieldDef(exeContext.schema, parentType, fieldNodes[0]);
      if (!fieldDef) {
        return;
      }
      const returnType = fieldDef.type;
      const resolveFn = (_fieldDef$resolve = fieldDef.resolve) !== null && _fieldDef$resolve !== void 0 ? _fieldDef$resolve : exeContext.fieldResolver;
      const info = buildResolveInfo(
        exeContext,
        fieldDef,
        fieldNodes,
        parentType,
        path4
      );
      try {
        const args = (0, _values.getArgumentValues)(
          fieldDef,
          fieldNodes[0],
          exeContext.variableValues
        );
        const contextValue = exeContext.contextValue;
        const result = resolveFn(source, args, contextValue, info);
        let completed;
        if ((0, _isPromise.isPromise)(result)) {
          completed = result.then(
            (resolved) => completeValue(exeContext, returnType, fieldNodes, info, path4, resolved)
          );
        } else {
          completed = completeValue(
            exeContext,
            returnType,
            fieldNodes,
            info,
            path4,
            result
          );
        }
        if ((0, _isPromise.isPromise)(completed)) {
          return completed.then(void 0, (rawError) => {
            const error = (0, _locatedError.locatedError)(
              rawError,
              fieldNodes,
              (0, _Path.pathToArray)(path4)
            );
            return handleFieldError(error, returnType, exeContext);
          });
        }
        return completed;
      } catch (rawError) {
        const error = (0, _locatedError.locatedError)(
          rawError,
          fieldNodes,
          (0, _Path.pathToArray)(path4)
        );
        return handleFieldError(error, returnType, exeContext);
      }
    }
    function buildResolveInfo(exeContext, fieldDef, fieldNodes, parentType, path4) {
      return {
        fieldName: fieldDef.name,
        fieldNodes,
        returnType: fieldDef.type,
        parentType,
        path: path4,
        schema: exeContext.schema,
        fragments: exeContext.fragments,
        rootValue: exeContext.rootValue,
        operation: exeContext.operation,
        variableValues: exeContext.variableValues
      };
    }
    function handleFieldError(error, returnType, exeContext) {
      if ((0, _definition.isNonNullType)(returnType)) {
        throw error;
      }
      exeContext.errors.push(error);
      return null;
    }
    function completeValue(exeContext, returnType, fieldNodes, info, path4, result) {
      if (result instanceof Error) {
        throw result;
      }
      if ((0, _definition.isNonNullType)(returnType)) {
        const completed = completeValue(
          exeContext,
          returnType.ofType,
          fieldNodes,
          info,
          path4,
          result
        );
        if (completed === null) {
          throw new Error(
            `Cannot return null for non-nullable field ${info.parentType.name}.${info.fieldName}.`
          );
        }
        return completed;
      }
      if (result == null) {
        return null;
      }
      if ((0, _definition.isListType)(returnType)) {
        return completeListValue(
          exeContext,
          returnType,
          fieldNodes,
          info,
          path4,
          result
        );
      }
      if ((0, _definition.isLeafType)(returnType)) {
        return completeLeafValue(returnType, result);
      }
      if ((0, _definition.isAbstractType)(returnType)) {
        return completeAbstractValue(
          exeContext,
          returnType,
          fieldNodes,
          info,
          path4,
          result
        );
      }
      if ((0, _definition.isObjectType)(returnType)) {
        return completeObjectValue(
          exeContext,
          returnType,
          fieldNodes,
          info,
          path4,
          result
        );
      }
      (0, _invariant.invariant)(
        false,
        "Cannot complete value of unexpected output type: " + (0, _inspect.inspect)(returnType)
      );
    }
    function completeListValue(exeContext, returnType, fieldNodes, info, path4, result) {
      if (!(0, _isIterableObject.isIterableObject)(result)) {
        throw new _GraphQLError.GraphQLError(
          `Expected Iterable, but did not find one for field "${info.parentType.name}.${info.fieldName}".`
        );
      }
      const itemType = returnType.ofType;
      let containsPromise = false;
      const completedResults = Array.from(result, (item, index) => {
        const itemPath = (0, _Path.addPath)(path4, index, void 0);
        try {
          let completedItem;
          if ((0, _isPromise.isPromise)(item)) {
            completedItem = item.then(
              (resolved) => completeValue(
                exeContext,
                itemType,
                fieldNodes,
                info,
                itemPath,
                resolved
              )
            );
          } else {
            completedItem = completeValue(
              exeContext,
              itemType,
              fieldNodes,
              info,
              itemPath,
              item
            );
          }
          if ((0, _isPromise.isPromise)(completedItem)) {
            containsPromise = true;
            return completedItem.then(void 0, (rawError) => {
              const error = (0, _locatedError.locatedError)(
                rawError,
                fieldNodes,
                (0, _Path.pathToArray)(itemPath)
              );
              return handleFieldError(error, itemType, exeContext);
            });
          }
          return completedItem;
        } catch (rawError) {
          const error = (0, _locatedError.locatedError)(
            rawError,
            fieldNodes,
            (0, _Path.pathToArray)(itemPath)
          );
          return handleFieldError(error, itemType, exeContext);
        }
      });
      return containsPromise ? Promise.all(completedResults) : completedResults;
    }
    function completeLeafValue(returnType, result) {
      const serializedResult = returnType.serialize(result);
      if (serializedResult == null) {
        throw new Error(
          `Expected \`${(0, _inspect.inspect)(returnType)}.serialize(${(0, _inspect.inspect)(result)})\` to return non-nullable value, returned: ${(0, _inspect.inspect)(
            serializedResult
          )}`
        );
      }
      return serializedResult;
    }
    function completeAbstractValue(exeContext, returnType, fieldNodes, info, path4, result) {
      var _returnType$resolveTy;
      const resolveTypeFn = (_returnType$resolveTy = returnType.resolveType) !== null && _returnType$resolveTy !== void 0 ? _returnType$resolveTy : exeContext.typeResolver;
      const contextValue = exeContext.contextValue;
      const runtimeType = resolveTypeFn(result, contextValue, info, returnType);
      if ((0, _isPromise.isPromise)(runtimeType)) {
        return runtimeType.then(
          (resolvedRuntimeType) => completeObjectValue(
            exeContext,
            ensureValidRuntimeType(
              resolvedRuntimeType,
              exeContext,
              returnType,
              fieldNodes,
              info,
              result
            ),
            fieldNodes,
            info,
            path4,
            result
          )
        );
      }
      return completeObjectValue(
        exeContext,
        ensureValidRuntimeType(
          runtimeType,
          exeContext,
          returnType,
          fieldNodes,
          info,
          result
        ),
        fieldNodes,
        info,
        path4,
        result
      );
    }
    function ensureValidRuntimeType(runtimeTypeName, exeContext, returnType, fieldNodes, info, result) {
      if (runtimeTypeName == null) {
        throw new _GraphQLError.GraphQLError(
          `Abstract type "${returnType.name}" must resolve to an Object type at runtime for field "${info.parentType.name}.${info.fieldName}". Either the "${returnType.name}" type should provide a "resolveType" function or each possible type should provide an "isTypeOf" function.`,
          fieldNodes
        );
      }
      if ((0, _definition.isObjectType)(runtimeTypeName)) {
        throw new _GraphQLError.GraphQLError(
          "Support for returning GraphQLObjectType from resolveType was removed in graphql-js@16.0.0 please return type name instead."
        );
      }
      if (typeof runtimeTypeName !== "string") {
        throw new _GraphQLError.GraphQLError(
          `Abstract type "${returnType.name}" must resolve to an Object type at runtime for field "${info.parentType.name}.${info.fieldName}" with value ${(0, _inspect.inspect)(result)}, received "${(0, _inspect.inspect)(runtimeTypeName)}".`
        );
      }
      const runtimeType = exeContext.schema.getType(runtimeTypeName);
      if (runtimeType == null) {
        throw new _GraphQLError.GraphQLError(
          `Abstract type "${returnType.name}" was resolved to a type "${runtimeTypeName}" that does not exist inside the schema.`,
          {
            nodes: fieldNodes
          }
        );
      }
      if (!(0, _definition.isObjectType)(runtimeType)) {
        throw new _GraphQLError.GraphQLError(
          `Abstract type "${returnType.name}" was resolved to a non-object type "${runtimeTypeName}".`,
          {
            nodes: fieldNodes
          }
        );
      }
      if (!exeContext.schema.isSubType(returnType, runtimeType)) {
        throw new _GraphQLError.GraphQLError(
          `Runtime Object type "${runtimeType.name}" is not a possible type for "${returnType.name}".`,
          {
            nodes: fieldNodes
          }
        );
      }
      return runtimeType;
    }
    function completeObjectValue(exeContext, returnType, fieldNodes, info, path4, result) {
      const subFieldNodes = collectSubfields(exeContext, returnType, fieldNodes);
      if (returnType.isTypeOf) {
        const isTypeOf = returnType.isTypeOf(result, exeContext.contextValue, info);
        if ((0, _isPromise.isPromise)(isTypeOf)) {
          return isTypeOf.then((resolvedIsTypeOf) => {
            if (!resolvedIsTypeOf) {
              throw invalidReturnTypeError(returnType, result, fieldNodes);
            }
            return executeFields(
              exeContext,
              returnType,
              result,
              path4,
              subFieldNodes
            );
          });
        }
        if (!isTypeOf) {
          throw invalidReturnTypeError(returnType, result, fieldNodes);
        }
      }
      return executeFields(exeContext, returnType, result, path4, subFieldNodes);
    }
    function invalidReturnTypeError(returnType, result, fieldNodes) {
      return new _GraphQLError.GraphQLError(
        `Expected value of type "${returnType.name}" but got: ${(0, _inspect.inspect)(result)}.`,
        {
          nodes: fieldNodes
        }
      );
    }
    var defaultTypeResolver = function(value, contextValue, info, abstractType) {
      if ((0, _isObjectLike.isObjectLike)(value) && typeof value.__typename === "string") {
        return value.__typename;
      }
      const possibleTypes = info.schema.getPossibleTypes(abstractType);
      const promisedIsTypeOfResults = [];
      for (let i = 0; i < possibleTypes.length; i++) {
        const type = possibleTypes[i];
        if (type.isTypeOf) {
          const isTypeOfResult = type.isTypeOf(value, contextValue, info);
          if ((0, _isPromise.isPromise)(isTypeOfResult)) {
            promisedIsTypeOfResults[i] = isTypeOfResult;
          } else if (isTypeOfResult) {
            return type.name;
          }
        }
      }
      if (promisedIsTypeOfResults.length) {
        return Promise.all(promisedIsTypeOfResults).then((isTypeOfResults) => {
          for (let i = 0; i < isTypeOfResults.length; i++) {
            if (isTypeOfResults[i]) {
              return possibleTypes[i].name;
            }
          }
        });
      }
    };
    exports2.defaultTypeResolver = defaultTypeResolver;
    var defaultFieldResolver = function(source, args, contextValue, info) {
      if ((0, _isObjectLike.isObjectLike)(source) || typeof source === "function") {
        const property = source[info.fieldName];
        if (typeof property === "function") {
          return source[info.fieldName](args, contextValue, info);
        }
        return property;
      }
    };
    exports2.defaultFieldResolver = defaultFieldResolver;
    function getFieldDef(schema, parentType, fieldNode) {
      const fieldName = fieldNode.name.value;
      if (fieldName === _introspection.SchemaMetaFieldDef.name && schema.getQueryType() === parentType) {
        return _introspection.SchemaMetaFieldDef;
      } else if (fieldName === _introspection.TypeMetaFieldDef.name && schema.getQueryType() === parentType) {
        return _introspection.TypeMetaFieldDef;
      } else if (fieldName === _introspection.TypeNameMetaFieldDef.name) {
        return _introspection.TypeNameMetaFieldDef;
      }
      return parentType.getFields()[fieldName];
    }
  }
});

// node_modules/graphql/graphql.js
var require_graphql = __commonJS({
  "node_modules/graphql/graphql.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.graphql = graphql;
    exports2.graphqlSync = graphqlSync;
    var _devAssert = require_devAssert();
    var _isPromise = require_isPromise();
    var _parser = require_parser();
    var _validate = require_validate();
    var _validate2 = require_validate2();
    var _execute = require_execute();
    function graphql(args) {
      return new Promise((resolve) => resolve(graphqlImpl(args)));
    }
    function graphqlSync(args) {
      const result = graphqlImpl(args);
      if ((0, _isPromise.isPromise)(result)) {
        throw new Error("GraphQL execution failed to complete synchronously.");
      }
      return result;
    }
    function graphqlImpl(args) {
      arguments.length < 2 || (0, _devAssert.devAssert)(
        false,
        "graphql@16 dropped long-deprecated support for positional arguments, please pass an object instead."
      );
      const {
        schema,
        source,
        rootValue,
        contextValue,
        variableValues,
        operationName,
        fieldResolver,
        typeResolver
      } = args;
      const schemaValidationErrors = (0, _validate.validateSchema)(schema);
      if (schemaValidationErrors.length > 0) {
        return {
          errors: schemaValidationErrors
        };
      }
      let document;
      try {
        document = (0, _parser.parse)(source);
      } catch (syntaxError) {
        return {
          errors: [syntaxError]
        };
      }
      const validationErrors = (0, _validate2.validate)(schema, document);
      if (validationErrors.length > 0) {
        return {
          errors: validationErrors
        };
      }
      return (0, _execute.execute)({
        schema,
        document,
        rootValue,
        contextValue,
        variableValues,
        operationName,
        fieldResolver,
        typeResolver
      });
    }
  }
});

// node_modules/graphql/type/index.js
var require_type = __commonJS({
  "node_modules/graphql/type/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "DEFAULT_DEPRECATION_REASON", {
      enumerable: true,
      get: function() {
        return _directives.DEFAULT_DEPRECATION_REASON;
      }
    });
    Object.defineProperty(exports2, "GRAPHQL_MAX_INT", {
      enumerable: true,
      get: function() {
        return _scalars.GRAPHQL_MAX_INT;
      }
    });
    Object.defineProperty(exports2, "GRAPHQL_MIN_INT", {
      enumerable: true,
      get: function() {
        return _scalars.GRAPHQL_MIN_INT;
      }
    });
    Object.defineProperty(exports2, "GraphQLBoolean", {
      enumerable: true,
      get: function() {
        return _scalars.GraphQLBoolean;
      }
    });
    Object.defineProperty(exports2, "GraphQLDeprecatedDirective", {
      enumerable: true,
      get: function() {
        return _directives.GraphQLDeprecatedDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLDirective", {
      enumerable: true,
      get: function() {
        return _directives.GraphQLDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLEnumType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLEnumType;
      }
    });
    Object.defineProperty(exports2, "GraphQLFloat", {
      enumerable: true,
      get: function() {
        return _scalars.GraphQLFloat;
      }
    });
    Object.defineProperty(exports2, "GraphQLID", {
      enumerable: true,
      get: function() {
        return _scalars.GraphQLID;
      }
    });
    Object.defineProperty(exports2, "GraphQLIncludeDirective", {
      enumerable: true,
      get: function() {
        return _directives.GraphQLIncludeDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLInputObjectType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLInputObjectType;
      }
    });
    Object.defineProperty(exports2, "GraphQLInt", {
      enumerable: true,
      get: function() {
        return _scalars.GraphQLInt;
      }
    });
    Object.defineProperty(exports2, "GraphQLInterfaceType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLInterfaceType;
      }
    });
    Object.defineProperty(exports2, "GraphQLList", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLList;
      }
    });
    Object.defineProperty(exports2, "GraphQLNonNull", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLNonNull;
      }
    });
    Object.defineProperty(exports2, "GraphQLObjectType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLObjectType;
      }
    });
    Object.defineProperty(exports2, "GraphQLScalarType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLScalarType;
      }
    });
    Object.defineProperty(exports2, "GraphQLSchema", {
      enumerable: true,
      get: function() {
        return _schema.GraphQLSchema;
      }
    });
    Object.defineProperty(exports2, "GraphQLSkipDirective", {
      enumerable: true,
      get: function() {
        return _directives.GraphQLSkipDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLSpecifiedByDirective", {
      enumerable: true,
      get: function() {
        return _directives.GraphQLSpecifiedByDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLString", {
      enumerable: true,
      get: function() {
        return _scalars.GraphQLString;
      }
    });
    Object.defineProperty(exports2, "GraphQLUnionType", {
      enumerable: true,
      get: function() {
        return _definition.GraphQLUnionType;
      }
    });
    Object.defineProperty(exports2, "SchemaMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _introspection.SchemaMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "TypeKind", {
      enumerable: true,
      get: function() {
        return _introspection.TypeKind;
      }
    });
    Object.defineProperty(exports2, "TypeMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _introspection.TypeMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "TypeNameMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _introspection.TypeNameMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "__Directive", {
      enumerable: true,
      get: function() {
        return _introspection.__Directive;
      }
    });
    Object.defineProperty(exports2, "__DirectiveLocation", {
      enumerable: true,
      get: function() {
        return _introspection.__DirectiveLocation;
      }
    });
    Object.defineProperty(exports2, "__EnumValue", {
      enumerable: true,
      get: function() {
        return _introspection.__EnumValue;
      }
    });
    Object.defineProperty(exports2, "__Field", {
      enumerable: true,
      get: function() {
        return _introspection.__Field;
      }
    });
    Object.defineProperty(exports2, "__InputValue", {
      enumerable: true,
      get: function() {
        return _introspection.__InputValue;
      }
    });
    Object.defineProperty(exports2, "__Schema", {
      enumerable: true,
      get: function() {
        return _introspection.__Schema;
      }
    });
    Object.defineProperty(exports2, "__Type", {
      enumerable: true,
      get: function() {
        return _introspection.__Type;
      }
    });
    Object.defineProperty(exports2, "__TypeKind", {
      enumerable: true,
      get: function() {
        return _introspection.__TypeKind;
      }
    });
    Object.defineProperty(exports2, "assertAbstractType", {
      enumerable: true,
      get: function() {
        return _definition.assertAbstractType;
      }
    });
    Object.defineProperty(exports2, "assertCompositeType", {
      enumerable: true,
      get: function() {
        return _definition.assertCompositeType;
      }
    });
    Object.defineProperty(exports2, "assertDirective", {
      enumerable: true,
      get: function() {
        return _directives.assertDirective;
      }
    });
    Object.defineProperty(exports2, "assertEnumType", {
      enumerable: true,
      get: function() {
        return _definition.assertEnumType;
      }
    });
    Object.defineProperty(exports2, "assertEnumValueName", {
      enumerable: true,
      get: function() {
        return _assertName.assertEnumValueName;
      }
    });
    Object.defineProperty(exports2, "assertInputObjectType", {
      enumerable: true,
      get: function() {
        return _definition.assertInputObjectType;
      }
    });
    Object.defineProperty(exports2, "assertInputType", {
      enumerable: true,
      get: function() {
        return _definition.assertInputType;
      }
    });
    Object.defineProperty(exports2, "assertInterfaceType", {
      enumerable: true,
      get: function() {
        return _definition.assertInterfaceType;
      }
    });
    Object.defineProperty(exports2, "assertLeafType", {
      enumerable: true,
      get: function() {
        return _definition.assertLeafType;
      }
    });
    Object.defineProperty(exports2, "assertListType", {
      enumerable: true,
      get: function() {
        return _definition.assertListType;
      }
    });
    Object.defineProperty(exports2, "assertName", {
      enumerable: true,
      get: function() {
        return _assertName.assertName;
      }
    });
    Object.defineProperty(exports2, "assertNamedType", {
      enumerable: true,
      get: function() {
        return _definition.assertNamedType;
      }
    });
    Object.defineProperty(exports2, "assertNonNullType", {
      enumerable: true,
      get: function() {
        return _definition.assertNonNullType;
      }
    });
    Object.defineProperty(exports2, "assertNullableType", {
      enumerable: true,
      get: function() {
        return _definition.assertNullableType;
      }
    });
    Object.defineProperty(exports2, "assertObjectType", {
      enumerable: true,
      get: function() {
        return _definition.assertObjectType;
      }
    });
    Object.defineProperty(exports2, "assertOutputType", {
      enumerable: true,
      get: function() {
        return _definition.assertOutputType;
      }
    });
    Object.defineProperty(exports2, "assertScalarType", {
      enumerable: true,
      get: function() {
        return _definition.assertScalarType;
      }
    });
    Object.defineProperty(exports2, "assertSchema", {
      enumerable: true,
      get: function() {
        return _schema.assertSchema;
      }
    });
    Object.defineProperty(exports2, "assertType", {
      enumerable: true,
      get: function() {
        return _definition.assertType;
      }
    });
    Object.defineProperty(exports2, "assertUnionType", {
      enumerable: true,
      get: function() {
        return _definition.assertUnionType;
      }
    });
    Object.defineProperty(exports2, "assertValidSchema", {
      enumerable: true,
      get: function() {
        return _validate.assertValidSchema;
      }
    });
    Object.defineProperty(exports2, "assertWrappingType", {
      enumerable: true,
      get: function() {
        return _definition.assertWrappingType;
      }
    });
    Object.defineProperty(exports2, "getNamedType", {
      enumerable: true,
      get: function() {
        return _definition.getNamedType;
      }
    });
    Object.defineProperty(exports2, "getNullableType", {
      enumerable: true,
      get: function() {
        return _definition.getNullableType;
      }
    });
    Object.defineProperty(exports2, "introspectionTypes", {
      enumerable: true,
      get: function() {
        return _introspection.introspectionTypes;
      }
    });
    Object.defineProperty(exports2, "isAbstractType", {
      enumerable: true,
      get: function() {
        return _definition.isAbstractType;
      }
    });
    Object.defineProperty(exports2, "isCompositeType", {
      enumerable: true,
      get: function() {
        return _definition.isCompositeType;
      }
    });
    Object.defineProperty(exports2, "isDirective", {
      enumerable: true,
      get: function() {
        return _directives.isDirective;
      }
    });
    Object.defineProperty(exports2, "isEnumType", {
      enumerable: true,
      get: function() {
        return _definition.isEnumType;
      }
    });
    Object.defineProperty(exports2, "isInputObjectType", {
      enumerable: true,
      get: function() {
        return _definition.isInputObjectType;
      }
    });
    Object.defineProperty(exports2, "isInputType", {
      enumerable: true,
      get: function() {
        return _definition.isInputType;
      }
    });
    Object.defineProperty(exports2, "isInterfaceType", {
      enumerable: true,
      get: function() {
        return _definition.isInterfaceType;
      }
    });
    Object.defineProperty(exports2, "isIntrospectionType", {
      enumerable: true,
      get: function() {
        return _introspection.isIntrospectionType;
      }
    });
    Object.defineProperty(exports2, "isLeafType", {
      enumerable: true,
      get: function() {
        return _definition.isLeafType;
      }
    });
    Object.defineProperty(exports2, "isListType", {
      enumerable: true,
      get: function() {
        return _definition.isListType;
      }
    });
    Object.defineProperty(exports2, "isNamedType", {
      enumerable: true,
      get: function() {
        return _definition.isNamedType;
      }
    });
    Object.defineProperty(exports2, "isNonNullType", {
      enumerable: true,
      get: function() {
        return _definition.isNonNullType;
      }
    });
    Object.defineProperty(exports2, "isNullableType", {
      enumerable: true,
      get: function() {
        return _definition.isNullableType;
      }
    });
    Object.defineProperty(exports2, "isObjectType", {
      enumerable: true,
      get: function() {
        return _definition.isObjectType;
      }
    });
    Object.defineProperty(exports2, "isOutputType", {
      enumerable: true,
      get: function() {
        return _definition.isOutputType;
      }
    });
    Object.defineProperty(exports2, "isRequiredArgument", {
      enumerable: true,
      get: function() {
        return _definition.isRequiredArgument;
      }
    });
    Object.defineProperty(exports2, "isRequiredInputField", {
      enumerable: true,
      get: function() {
        return _definition.isRequiredInputField;
      }
    });
    Object.defineProperty(exports2, "isScalarType", {
      enumerable: true,
      get: function() {
        return _definition.isScalarType;
      }
    });
    Object.defineProperty(exports2, "isSchema", {
      enumerable: true,
      get: function() {
        return _schema.isSchema;
      }
    });
    Object.defineProperty(exports2, "isSpecifiedDirective", {
      enumerable: true,
      get: function() {
        return _directives.isSpecifiedDirective;
      }
    });
    Object.defineProperty(exports2, "isSpecifiedScalarType", {
      enumerable: true,
      get: function() {
        return _scalars.isSpecifiedScalarType;
      }
    });
    Object.defineProperty(exports2, "isType", {
      enumerable: true,
      get: function() {
        return _definition.isType;
      }
    });
    Object.defineProperty(exports2, "isUnionType", {
      enumerable: true,
      get: function() {
        return _definition.isUnionType;
      }
    });
    Object.defineProperty(exports2, "isWrappingType", {
      enumerable: true,
      get: function() {
        return _definition.isWrappingType;
      }
    });
    Object.defineProperty(exports2, "resolveObjMapThunk", {
      enumerable: true,
      get: function() {
        return _definition.resolveObjMapThunk;
      }
    });
    Object.defineProperty(exports2, "resolveReadonlyArrayThunk", {
      enumerable: true,
      get: function() {
        return _definition.resolveReadonlyArrayThunk;
      }
    });
    Object.defineProperty(exports2, "specifiedDirectives", {
      enumerable: true,
      get: function() {
        return _directives.specifiedDirectives;
      }
    });
    Object.defineProperty(exports2, "specifiedScalarTypes", {
      enumerable: true,
      get: function() {
        return _scalars.specifiedScalarTypes;
      }
    });
    Object.defineProperty(exports2, "validateSchema", {
      enumerable: true,
      get: function() {
        return _validate.validateSchema;
      }
    });
    var _schema = require_schema();
    var _definition = require_definition();
    var _directives = require_directives();
    var _scalars = require_scalars();
    var _introspection = require_introspection();
    var _validate = require_validate();
    var _assertName = require_assertName();
  }
});

// node_modules/graphql/language/index.js
var require_language = __commonJS({
  "node_modules/graphql/language/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "BREAK", {
      enumerable: true,
      get: function() {
        return _visitor.BREAK;
      }
    });
    Object.defineProperty(exports2, "DirectiveLocation", {
      enumerable: true,
      get: function() {
        return _directiveLocation.DirectiveLocation;
      }
    });
    Object.defineProperty(exports2, "Kind", {
      enumerable: true,
      get: function() {
        return _kinds.Kind;
      }
    });
    Object.defineProperty(exports2, "Lexer", {
      enumerable: true,
      get: function() {
        return _lexer.Lexer;
      }
    });
    Object.defineProperty(exports2, "Location", {
      enumerable: true,
      get: function() {
        return _ast.Location;
      }
    });
    Object.defineProperty(exports2, "OperationTypeNode", {
      enumerable: true,
      get: function() {
        return _ast.OperationTypeNode;
      }
    });
    Object.defineProperty(exports2, "Source", {
      enumerable: true,
      get: function() {
        return _source.Source;
      }
    });
    Object.defineProperty(exports2, "Token", {
      enumerable: true,
      get: function() {
        return _ast.Token;
      }
    });
    Object.defineProperty(exports2, "TokenKind", {
      enumerable: true,
      get: function() {
        return _tokenKind.TokenKind;
      }
    });
    Object.defineProperty(exports2, "getEnterLeaveForKind", {
      enumerable: true,
      get: function() {
        return _visitor.getEnterLeaveForKind;
      }
    });
    Object.defineProperty(exports2, "getLocation", {
      enumerable: true,
      get: function() {
        return _location.getLocation;
      }
    });
    Object.defineProperty(exports2, "getVisitFn", {
      enumerable: true,
      get: function() {
        return _visitor.getVisitFn;
      }
    });
    Object.defineProperty(exports2, "isConstValueNode", {
      enumerable: true,
      get: function() {
        return _predicates.isConstValueNode;
      }
    });
    Object.defineProperty(exports2, "isDefinitionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isExecutableDefinitionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isExecutableDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isSelectionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isSelectionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeDefinitionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isTypeDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeExtensionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isTypeExtensionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeNode", {
      enumerable: true,
      get: function() {
        return _predicates.isTypeNode;
      }
    });
    Object.defineProperty(exports2, "isTypeSystemDefinitionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isTypeSystemDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeSystemExtensionNode", {
      enumerable: true,
      get: function() {
        return _predicates.isTypeSystemExtensionNode;
      }
    });
    Object.defineProperty(exports2, "isValueNode", {
      enumerable: true,
      get: function() {
        return _predicates.isValueNode;
      }
    });
    Object.defineProperty(exports2, "parse", {
      enumerable: true,
      get: function() {
        return _parser.parse;
      }
    });
    Object.defineProperty(exports2, "parseConstValue", {
      enumerable: true,
      get: function() {
        return _parser.parseConstValue;
      }
    });
    Object.defineProperty(exports2, "parseType", {
      enumerable: true,
      get: function() {
        return _parser.parseType;
      }
    });
    Object.defineProperty(exports2, "parseValue", {
      enumerable: true,
      get: function() {
        return _parser.parseValue;
      }
    });
    Object.defineProperty(exports2, "print", {
      enumerable: true,
      get: function() {
        return _printer.print;
      }
    });
    Object.defineProperty(exports2, "printLocation", {
      enumerable: true,
      get: function() {
        return _printLocation.printLocation;
      }
    });
    Object.defineProperty(exports2, "printSourceLocation", {
      enumerable: true,
      get: function() {
        return _printLocation.printSourceLocation;
      }
    });
    Object.defineProperty(exports2, "visit", {
      enumerable: true,
      get: function() {
        return _visitor.visit;
      }
    });
    Object.defineProperty(exports2, "visitInParallel", {
      enumerable: true,
      get: function() {
        return _visitor.visitInParallel;
      }
    });
    var _source = require_source();
    var _location = require_location();
    var _printLocation = require_printLocation();
    var _kinds = require_kinds();
    var _tokenKind = require_tokenKind();
    var _lexer = require_lexer();
    var _parser = require_parser();
    var _printer = require_printer();
    var _visitor = require_visitor();
    var _ast = require_ast();
    var _predicates = require_predicates();
    var _directiveLocation = require_directiveLocation();
  }
});

// node_modules/graphql/jsutils/isAsyncIterable.js
var require_isAsyncIterable = __commonJS({
  "node_modules/graphql/jsutils/isAsyncIterable.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.isAsyncIterable = isAsyncIterable;
    function isAsyncIterable(maybeAsyncIterable) {
      return typeof (maybeAsyncIterable === null || maybeAsyncIterable === void 0 ? void 0 : maybeAsyncIterable[Symbol.asyncIterator]) === "function";
    }
  }
});

// node_modules/graphql/execution/mapAsyncIterator.js
var require_mapAsyncIterator = __commonJS({
  "node_modules/graphql/execution/mapAsyncIterator.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.mapAsyncIterator = mapAsyncIterator;
    function mapAsyncIterator(iterable, callback) {
      const iterator = iterable[Symbol.asyncIterator]();
      async function mapResult(result) {
        if (result.done) {
          return result;
        }
        try {
          return {
            value: await callback(result.value),
            done: false
          };
        } catch (error) {
          if (typeof iterator.return === "function") {
            try {
              await iterator.return();
            } catch (_e) {
            }
          }
          throw error;
        }
      }
      return {
        async next() {
          return mapResult(await iterator.next());
        },
        async return() {
          return typeof iterator.return === "function" ? mapResult(await iterator.return()) : {
            value: void 0,
            done: true
          };
        },
        async throw(error) {
          if (typeof iterator.throw === "function") {
            return mapResult(await iterator.throw(error));
          }
          throw error;
        },
        [Symbol.asyncIterator]() {
          return this;
        }
      };
    }
  }
});

// node_modules/graphql/execution/subscribe.js
var require_subscribe = __commonJS({
  "node_modules/graphql/execution/subscribe.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.createSourceEventStream = createSourceEventStream;
    exports2.subscribe = subscribe;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _isAsyncIterable = require_isAsyncIterable();
    var _Path = require_Path();
    var _GraphQLError = require_GraphQLError();
    var _locatedError = require_locatedError();
    var _collectFields = require_collectFields();
    var _execute = require_execute();
    var _mapAsyncIterator = require_mapAsyncIterator();
    var _values = require_values();
    async function subscribe(args) {
      arguments.length < 2 || (0, _devAssert.devAssert)(
        false,
        "graphql@16 dropped long-deprecated support for positional arguments, please pass an object instead."
      );
      const resultOrStream = await createSourceEventStream(args);
      if (!(0, _isAsyncIterable.isAsyncIterable)(resultOrStream)) {
        return resultOrStream;
      }
      const mapSourceToResponse = (payload) => (0, _execute.execute)({ ...args, rootValue: payload });
      return (0, _mapAsyncIterator.mapAsyncIterator)(
        resultOrStream,
        mapSourceToResponse
      );
    }
    function toNormalizedArgs(args) {
      const firstArg = args[0];
      if (firstArg && "document" in firstArg) {
        return firstArg;
      }
      return {
        schema: firstArg,
        // FIXME: when underlying TS bug fixed, see https://github.com/microsoft/TypeScript/issues/31613
        document: args[1],
        rootValue: args[2],
        contextValue: args[3],
        variableValues: args[4],
        operationName: args[5],
        subscribeFieldResolver: args[6]
      };
    }
    async function createSourceEventStream(...rawArgs) {
      const args = toNormalizedArgs(rawArgs);
      const { schema, document, variableValues } = args;
      (0, _execute.assertValidExecutionArguments)(schema, document, variableValues);
      const exeContext = (0, _execute.buildExecutionContext)(args);
      if (!("schema" in exeContext)) {
        return {
          errors: exeContext
        };
      }
      try {
        const eventStream = await executeSubscription(exeContext);
        if (!(0, _isAsyncIterable.isAsyncIterable)(eventStream)) {
          throw new Error(
            `Subscription field must return Async Iterable. Received: ${(0, _inspect.inspect)(eventStream)}.`
          );
        }
        return eventStream;
      } catch (error) {
        if (error instanceof _GraphQLError.GraphQLError) {
          return {
            errors: [error]
          };
        }
        throw error;
      }
    }
    async function executeSubscription(exeContext) {
      const { schema, fragments, operation, variableValues, rootValue } = exeContext;
      const rootType = schema.getSubscriptionType();
      if (rootType == null) {
        throw new _GraphQLError.GraphQLError(
          "Schema is not configured to execute subscription operation.",
          {
            nodes: operation
          }
        );
      }
      const rootFields = (0, _collectFields.collectFields)(
        schema,
        fragments,
        variableValues,
        rootType,
        operation.selectionSet
      );
      const [responseName, fieldNodes] = [...rootFields.entries()][0];
      const fieldDef = (0, _execute.getFieldDef)(schema, rootType, fieldNodes[0]);
      if (!fieldDef) {
        const fieldName = fieldNodes[0].name.value;
        throw new _GraphQLError.GraphQLError(
          `The subscription field "${fieldName}" is not defined.`,
          {
            nodes: fieldNodes
          }
        );
      }
      const path4 = (0, _Path.addPath)(void 0, responseName, rootType.name);
      const info = (0, _execute.buildResolveInfo)(
        exeContext,
        fieldDef,
        fieldNodes,
        rootType,
        path4
      );
      try {
        var _fieldDef$subscribe;
        const args = (0, _values.getArgumentValues)(
          fieldDef,
          fieldNodes[0],
          variableValues
        );
        const contextValue = exeContext.contextValue;
        const resolveFn = (_fieldDef$subscribe = fieldDef.subscribe) !== null && _fieldDef$subscribe !== void 0 ? _fieldDef$subscribe : exeContext.subscribeFieldResolver;
        const eventStream = await resolveFn(rootValue, args, contextValue, info);
        if (eventStream instanceof Error) {
          throw eventStream;
        }
        return eventStream;
      } catch (error) {
        throw (0, _locatedError.locatedError)(
          error,
          fieldNodes,
          (0, _Path.pathToArray)(path4)
        );
      }
    }
  }
});

// node_modules/graphql/execution/index.js
var require_execution = __commonJS({
  "node_modules/graphql/execution/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "createSourceEventStream", {
      enumerable: true,
      get: function() {
        return _subscribe.createSourceEventStream;
      }
    });
    Object.defineProperty(exports2, "defaultFieldResolver", {
      enumerable: true,
      get: function() {
        return _execute.defaultFieldResolver;
      }
    });
    Object.defineProperty(exports2, "defaultTypeResolver", {
      enumerable: true,
      get: function() {
        return _execute.defaultTypeResolver;
      }
    });
    Object.defineProperty(exports2, "execute", {
      enumerable: true,
      get: function() {
        return _execute.execute;
      }
    });
    Object.defineProperty(exports2, "executeSync", {
      enumerable: true,
      get: function() {
        return _execute.executeSync;
      }
    });
    Object.defineProperty(exports2, "getArgumentValues", {
      enumerable: true,
      get: function() {
        return _values.getArgumentValues;
      }
    });
    Object.defineProperty(exports2, "getDirectiveValues", {
      enumerable: true,
      get: function() {
        return _values.getDirectiveValues;
      }
    });
    Object.defineProperty(exports2, "getVariableValues", {
      enumerable: true,
      get: function() {
        return _values.getVariableValues;
      }
    });
    Object.defineProperty(exports2, "responsePathAsArray", {
      enumerable: true,
      get: function() {
        return _Path.pathToArray;
      }
    });
    Object.defineProperty(exports2, "subscribe", {
      enumerable: true,
      get: function() {
        return _subscribe.subscribe;
      }
    });
    var _Path = require_Path();
    var _execute = require_execute();
    var _subscribe = require_subscribe();
    var _values = require_values();
  }
});

// node_modules/graphql/validation/rules/custom/NoDeprecatedCustomRule.js
var require_NoDeprecatedCustomRule = __commonJS({
  "node_modules/graphql/validation/rules/custom/NoDeprecatedCustomRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoDeprecatedCustomRule = NoDeprecatedCustomRule;
    var _invariant = require_invariant();
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    function NoDeprecatedCustomRule(context) {
      return {
        Field(node) {
          const fieldDef = context.getFieldDef();
          const deprecationReason = fieldDef === null || fieldDef === void 0 ? void 0 : fieldDef.deprecationReason;
          if (fieldDef && deprecationReason != null) {
            const parentType = context.getParentType();
            parentType != null || (0, _invariant.invariant)(false);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `The field ${parentType.name}.${fieldDef.name} is deprecated. ${deprecationReason}`,
                {
                  nodes: node
                }
              )
            );
          }
        },
        Argument(node) {
          const argDef = context.getArgument();
          const deprecationReason = argDef === null || argDef === void 0 ? void 0 : argDef.deprecationReason;
          if (argDef && deprecationReason != null) {
            const directiveDef = context.getDirective();
            if (directiveDef != null) {
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Directive "@${directiveDef.name}" argument "${argDef.name}" is deprecated. ${deprecationReason}`,
                  {
                    nodes: node
                  }
                )
              );
            } else {
              const parentType = context.getParentType();
              const fieldDef = context.getFieldDef();
              parentType != null && fieldDef != null || (0, _invariant.invariant)(false);
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `Field "${parentType.name}.${fieldDef.name}" argument "${argDef.name}" is deprecated. ${deprecationReason}`,
                  {
                    nodes: node
                  }
                )
              );
            }
          }
        },
        ObjectField(node) {
          const inputObjectDef = (0, _definition.getNamedType)(
            context.getParentInputType()
          );
          if ((0, _definition.isInputObjectType)(inputObjectDef)) {
            const inputFieldDef = inputObjectDef.getFields()[node.name.value];
            const deprecationReason = inputFieldDef === null || inputFieldDef === void 0 ? void 0 : inputFieldDef.deprecationReason;
            if (deprecationReason != null) {
              context.reportError(
                new _GraphQLError.GraphQLError(
                  `The input field ${inputObjectDef.name}.${inputFieldDef.name} is deprecated. ${deprecationReason}`,
                  {
                    nodes: node
                  }
                )
              );
            }
          }
        },
        EnumValue(node) {
          const enumValueDef = context.getEnumValue();
          const deprecationReason = enumValueDef === null || enumValueDef === void 0 ? void 0 : enumValueDef.deprecationReason;
          if (enumValueDef && deprecationReason != null) {
            const enumTypeDef = (0, _definition.getNamedType)(
              context.getInputType()
            );
            enumTypeDef != null || (0, _invariant.invariant)(false);
            context.reportError(
              new _GraphQLError.GraphQLError(
                `The enum value "${enumTypeDef.name}.${enumValueDef.name}" is deprecated. ${deprecationReason}`,
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/rules/custom/NoSchemaIntrospectionCustomRule.js
var require_NoSchemaIntrospectionCustomRule = __commonJS({
  "node_modules/graphql/validation/rules/custom/NoSchemaIntrospectionCustomRule.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.NoSchemaIntrospectionCustomRule = NoSchemaIntrospectionCustomRule;
    var _GraphQLError = require_GraphQLError();
    var _definition = require_definition();
    var _introspection = require_introspection();
    function NoSchemaIntrospectionCustomRule(context) {
      return {
        Field(node) {
          const type = (0, _definition.getNamedType)(context.getType());
          if (type && (0, _introspection.isIntrospectionType)(type)) {
            context.reportError(
              new _GraphQLError.GraphQLError(
                `GraphQL introspection has been disabled, but the requested query contained the field "${node.name.value}".`,
                {
                  nodes: node
                }
              )
            );
          }
        }
      };
    }
  }
});

// node_modules/graphql/validation/index.js
var require_validation = __commonJS({
  "node_modules/graphql/validation/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "ExecutableDefinitionsRule", {
      enumerable: true,
      get: function() {
        return _ExecutableDefinitionsRule.ExecutableDefinitionsRule;
      }
    });
    Object.defineProperty(exports2, "FieldsOnCorrectTypeRule", {
      enumerable: true,
      get: function() {
        return _FieldsOnCorrectTypeRule.FieldsOnCorrectTypeRule;
      }
    });
    Object.defineProperty(exports2, "FragmentsOnCompositeTypesRule", {
      enumerable: true,
      get: function() {
        return _FragmentsOnCompositeTypesRule.FragmentsOnCompositeTypesRule;
      }
    });
    Object.defineProperty(exports2, "KnownArgumentNamesRule", {
      enumerable: true,
      get: function() {
        return _KnownArgumentNamesRule.KnownArgumentNamesRule;
      }
    });
    Object.defineProperty(exports2, "KnownDirectivesRule", {
      enumerable: true,
      get: function() {
        return _KnownDirectivesRule.KnownDirectivesRule;
      }
    });
    Object.defineProperty(exports2, "KnownFragmentNamesRule", {
      enumerable: true,
      get: function() {
        return _KnownFragmentNamesRule.KnownFragmentNamesRule;
      }
    });
    Object.defineProperty(exports2, "KnownTypeNamesRule", {
      enumerable: true,
      get: function() {
        return _KnownTypeNamesRule.KnownTypeNamesRule;
      }
    });
    Object.defineProperty(exports2, "LoneAnonymousOperationRule", {
      enumerable: true,
      get: function() {
        return _LoneAnonymousOperationRule.LoneAnonymousOperationRule;
      }
    });
    Object.defineProperty(exports2, "LoneSchemaDefinitionRule", {
      enumerable: true,
      get: function() {
        return _LoneSchemaDefinitionRule.LoneSchemaDefinitionRule;
      }
    });
    Object.defineProperty(exports2, "NoDeprecatedCustomRule", {
      enumerable: true,
      get: function() {
        return _NoDeprecatedCustomRule.NoDeprecatedCustomRule;
      }
    });
    Object.defineProperty(exports2, "NoFragmentCyclesRule", {
      enumerable: true,
      get: function() {
        return _NoFragmentCyclesRule.NoFragmentCyclesRule;
      }
    });
    Object.defineProperty(exports2, "NoSchemaIntrospectionCustomRule", {
      enumerable: true,
      get: function() {
        return _NoSchemaIntrospectionCustomRule.NoSchemaIntrospectionCustomRule;
      }
    });
    Object.defineProperty(exports2, "NoUndefinedVariablesRule", {
      enumerable: true,
      get: function() {
        return _NoUndefinedVariablesRule.NoUndefinedVariablesRule;
      }
    });
    Object.defineProperty(exports2, "NoUnusedFragmentsRule", {
      enumerable: true,
      get: function() {
        return _NoUnusedFragmentsRule.NoUnusedFragmentsRule;
      }
    });
    Object.defineProperty(exports2, "NoUnusedVariablesRule", {
      enumerable: true,
      get: function() {
        return _NoUnusedVariablesRule.NoUnusedVariablesRule;
      }
    });
    Object.defineProperty(exports2, "OverlappingFieldsCanBeMergedRule", {
      enumerable: true,
      get: function() {
        return _OverlappingFieldsCanBeMergedRule.OverlappingFieldsCanBeMergedRule;
      }
    });
    Object.defineProperty(exports2, "PossibleFragmentSpreadsRule", {
      enumerable: true,
      get: function() {
        return _PossibleFragmentSpreadsRule.PossibleFragmentSpreadsRule;
      }
    });
    Object.defineProperty(exports2, "PossibleTypeExtensionsRule", {
      enumerable: true,
      get: function() {
        return _PossibleTypeExtensionsRule.PossibleTypeExtensionsRule;
      }
    });
    Object.defineProperty(exports2, "ProvidedRequiredArgumentsRule", {
      enumerable: true,
      get: function() {
        return _ProvidedRequiredArgumentsRule.ProvidedRequiredArgumentsRule;
      }
    });
    Object.defineProperty(exports2, "ScalarLeafsRule", {
      enumerable: true,
      get: function() {
        return _ScalarLeafsRule.ScalarLeafsRule;
      }
    });
    Object.defineProperty(exports2, "SingleFieldSubscriptionsRule", {
      enumerable: true,
      get: function() {
        return _SingleFieldSubscriptionsRule.SingleFieldSubscriptionsRule;
      }
    });
    Object.defineProperty(exports2, "UniqueArgumentDefinitionNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueArgumentDefinitionNamesRule.UniqueArgumentDefinitionNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueArgumentNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueArgumentNamesRule.UniqueArgumentNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueDirectiveNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueDirectiveNamesRule.UniqueDirectiveNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueDirectivesPerLocationRule", {
      enumerable: true,
      get: function() {
        return _UniqueDirectivesPerLocationRule.UniqueDirectivesPerLocationRule;
      }
    });
    Object.defineProperty(exports2, "UniqueEnumValueNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueEnumValueNamesRule.UniqueEnumValueNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueFieldDefinitionNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueFieldDefinitionNamesRule.UniqueFieldDefinitionNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueFragmentNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueFragmentNamesRule.UniqueFragmentNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueInputFieldNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueInputFieldNamesRule.UniqueInputFieldNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueOperationNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueOperationNamesRule.UniqueOperationNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueOperationTypesRule", {
      enumerable: true,
      get: function() {
        return _UniqueOperationTypesRule.UniqueOperationTypesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueTypeNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueTypeNamesRule.UniqueTypeNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueVariableNamesRule", {
      enumerable: true,
      get: function() {
        return _UniqueVariableNamesRule.UniqueVariableNamesRule;
      }
    });
    Object.defineProperty(exports2, "ValidationContext", {
      enumerable: true,
      get: function() {
        return _ValidationContext.ValidationContext;
      }
    });
    Object.defineProperty(exports2, "ValuesOfCorrectTypeRule", {
      enumerable: true,
      get: function() {
        return _ValuesOfCorrectTypeRule.ValuesOfCorrectTypeRule;
      }
    });
    Object.defineProperty(exports2, "VariablesAreInputTypesRule", {
      enumerable: true,
      get: function() {
        return _VariablesAreInputTypesRule.VariablesAreInputTypesRule;
      }
    });
    Object.defineProperty(exports2, "VariablesInAllowedPositionRule", {
      enumerable: true,
      get: function() {
        return _VariablesInAllowedPositionRule.VariablesInAllowedPositionRule;
      }
    });
    Object.defineProperty(exports2, "specifiedRules", {
      enumerable: true,
      get: function() {
        return _specifiedRules.specifiedRules;
      }
    });
    Object.defineProperty(exports2, "validate", {
      enumerable: true,
      get: function() {
        return _validate.validate;
      }
    });
    var _validate = require_validate2();
    var _ValidationContext = require_ValidationContext();
    var _specifiedRules = require_specifiedRules();
    var _ExecutableDefinitionsRule = require_ExecutableDefinitionsRule();
    var _FieldsOnCorrectTypeRule = require_FieldsOnCorrectTypeRule();
    var _FragmentsOnCompositeTypesRule = require_FragmentsOnCompositeTypesRule();
    var _KnownArgumentNamesRule = require_KnownArgumentNamesRule();
    var _KnownDirectivesRule = require_KnownDirectivesRule();
    var _KnownFragmentNamesRule = require_KnownFragmentNamesRule();
    var _KnownTypeNamesRule = require_KnownTypeNamesRule();
    var _LoneAnonymousOperationRule = require_LoneAnonymousOperationRule();
    var _NoFragmentCyclesRule = require_NoFragmentCyclesRule();
    var _NoUndefinedVariablesRule = require_NoUndefinedVariablesRule();
    var _NoUnusedFragmentsRule = require_NoUnusedFragmentsRule();
    var _NoUnusedVariablesRule = require_NoUnusedVariablesRule();
    var _OverlappingFieldsCanBeMergedRule = require_OverlappingFieldsCanBeMergedRule();
    var _PossibleFragmentSpreadsRule = require_PossibleFragmentSpreadsRule();
    var _ProvidedRequiredArgumentsRule = require_ProvidedRequiredArgumentsRule();
    var _ScalarLeafsRule = require_ScalarLeafsRule();
    var _SingleFieldSubscriptionsRule = require_SingleFieldSubscriptionsRule();
    var _UniqueArgumentNamesRule = require_UniqueArgumentNamesRule();
    var _UniqueDirectivesPerLocationRule = require_UniqueDirectivesPerLocationRule();
    var _UniqueFragmentNamesRule = require_UniqueFragmentNamesRule();
    var _UniqueInputFieldNamesRule = require_UniqueInputFieldNamesRule();
    var _UniqueOperationNamesRule = require_UniqueOperationNamesRule();
    var _UniqueVariableNamesRule = require_UniqueVariableNamesRule();
    var _ValuesOfCorrectTypeRule = require_ValuesOfCorrectTypeRule();
    var _VariablesAreInputTypesRule = require_VariablesAreInputTypesRule();
    var _VariablesInAllowedPositionRule = require_VariablesInAllowedPositionRule();
    var _LoneSchemaDefinitionRule = require_LoneSchemaDefinitionRule();
    var _UniqueOperationTypesRule = require_UniqueOperationTypesRule();
    var _UniqueTypeNamesRule = require_UniqueTypeNamesRule();
    var _UniqueEnumValueNamesRule = require_UniqueEnumValueNamesRule();
    var _UniqueFieldDefinitionNamesRule = require_UniqueFieldDefinitionNamesRule();
    var _UniqueArgumentDefinitionNamesRule = require_UniqueArgumentDefinitionNamesRule();
    var _UniqueDirectiveNamesRule = require_UniqueDirectiveNamesRule();
    var _PossibleTypeExtensionsRule = require_PossibleTypeExtensionsRule();
    var _NoDeprecatedCustomRule = require_NoDeprecatedCustomRule();
    var _NoSchemaIntrospectionCustomRule = require_NoSchemaIntrospectionCustomRule();
  }
});

// node_modules/graphql/error/index.js
var require_error = __commonJS({
  "node_modules/graphql/error/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "GraphQLError", {
      enumerable: true,
      get: function() {
        return _GraphQLError.GraphQLError;
      }
    });
    Object.defineProperty(exports2, "formatError", {
      enumerable: true,
      get: function() {
        return _GraphQLError.formatError;
      }
    });
    Object.defineProperty(exports2, "locatedError", {
      enumerable: true,
      get: function() {
        return _locatedError.locatedError;
      }
    });
    Object.defineProperty(exports2, "printError", {
      enumerable: true,
      get: function() {
        return _GraphQLError.printError;
      }
    });
    Object.defineProperty(exports2, "syntaxError", {
      enumerable: true,
      get: function() {
        return _syntaxError.syntaxError;
      }
    });
    var _GraphQLError = require_GraphQLError();
    var _syntaxError = require_syntaxError();
    var _locatedError = require_locatedError();
  }
});

// node_modules/graphql/utilities/getIntrospectionQuery.js
var require_getIntrospectionQuery = __commonJS({
  "node_modules/graphql/utilities/getIntrospectionQuery.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.getIntrospectionQuery = getIntrospectionQuery;
    function getIntrospectionQuery(options) {
      const optionsWithDefault = {
        descriptions: true,
        specifiedByUrl: false,
        directiveIsRepeatable: false,
        schemaDescription: false,
        inputValueDeprecation: false,
        ...options
      };
      const descriptions = optionsWithDefault.descriptions ? "description" : "";
      const specifiedByUrl = optionsWithDefault.specifiedByUrl ? "specifiedByURL" : "";
      const directiveIsRepeatable = optionsWithDefault.directiveIsRepeatable ? "isRepeatable" : "";
      const schemaDescription = optionsWithDefault.schemaDescription ? descriptions : "";
      function inputDeprecation(str) {
        return optionsWithDefault.inputValueDeprecation ? str : "";
      }
      return `
    query IntrospectionQuery {
      __schema {
        ${schemaDescription}
        queryType { name }
        mutationType { name }
        subscriptionType { name }
        types {
          ...FullType
        }
        directives {
          name
          ${descriptions}
          ${directiveIsRepeatable}
          locations
          args${inputDeprecation("(includeDeprecated: true)")} {
            ...InputValue
          }
        }
      }
    }

    fragment FullType on __Type {
      kind
      name
      ${descriptions}
      ${specifiedByUrl}
      fields(includeDeprecated: true) {
        name
        ${descriptions}
        args${inputDeprecation("(includeDeprecated: true)")} {
          ...InputValue
        }
        type {
          ...TypeRef
        }
        isDeprecated
        deprecationReason
      }
      inputFields${inputDeprecation("(includeDeprecated: true)")} {
        ...InputValue
      }
      interfaces {
        ...TypeRef
      }
      enumValues(includeDeprecated: true) {
        name
        ${descriptions}
        isDeprecated
        deprecationReason
      }
      possibleTypes {
        ...TypeRef
      }
    }

    fragment InputValue on __InputValue {
      name
      ${descriptions}
      type { ...TypeRef }
      defaultValue
      ${inputDeprecation("isDeprecated")}
      ${inputDeprecation("deprecationReason")}
    }

    fragment TypeRef on __Type {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                  ofType {
                    kind
                    name
                    ofType {
                      kind
                      name
                      ofType {
                        kind
                        name
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  `;
    }
  }
});

// node_modules/graphql/utilities/getOperationAST.js
var require_getOperationAST = __commonJS({
  "node_modules/graphql/utilities/getOperationAST.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.getOperationAST = getOperationAST;
    var _kinds = require_kinds();
    function getOperationAST(documentAST, operationName) {
      let operation = null;
      for (const definition of documentAST.definitions) {
        if (definition.kind === _kinds.Kind.OPERATION_DEFINITION) {
          var _definition$name;
          if (operationName == null) {
            if (operation) {
              return null;
            }
            operation = definition;
          } else if (((_definition$name = definition.name) === null || _definition$name === void 0 ? void 0 : _definition$name.value) === operationName) {
            return definition;
          }
        }
      }
      return operation;
    }
  }
});

// node_modules/graphql/utilities/getOperationRootType.js
var require_getOperationRootType = __commonJS({
  "node_modules/graphql/utilities/getOperationRootType.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.getOperationRootType = getOperationRootType;
    var _GraphQLError = require_GraphQLError();
    function getOperationRootType(schema, operation) {
      if (operation.operation === "query") {
        const queryType = schema.getQueryType();
        if (!queryType) {
          throw new _GraphQLError.GraphQLError(
            "Schema does not define the required query root type.",
            {
              nodes: operation
            }
          );
        }
        return queryType;
      }
      if (operation.operation === "mutation") {
        const mutationType = schema.getMutationType();
        if (!mutationType) {
          throw new _GraphQLError.GraphQLError(
            "Schema is not configured for mutations.",
            {
              nodes: operation
            }
          );
        }
        return mutationType;
      }
      if (operation.operation === "subscription") {
        const subscriptionType = schema.getSubscriptionType();
        if (!subscriptionType) {
          throw new _GraphQLError.GraphQLError(
            "Schema is not configured for subscriptions.",
            {
              nodes: operation
            }
          );
        }
        return subscriptionType;
      }
      throw new _GraphQLError.GraphQLError(
        "Can only have query, mutation and subscription operations.",
        {
          nodes: operation
        }
      );
    }
  }
});

// node_modules/graphql/utilities/introspectionFromSchema.js
var require_introspectionFromSchema = __commonJS({
  "node_modules/graphql/utilities/introspectionFromSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.introspectionFromSchema = introspectionFromSchema;
    var _invariant = require_invariant();
    var _parser = require_parser();
    var _execute = require_execute();
    var _getIntrospectionQuery = require_getIntrospectionQuery();
    function introspectionFromSchema(schema, options) {
      const optionsWithDefaults = {
        specifiedByUrl: true,
        directiveIsRepeatable: true,
        schemaDescription: true,
        inputValueDeprecation: true,
        ...options
      };
      const document = (0, _parser.parse)(
        (0, _getIntrospectionQuery.getIntrospectionQuery)(optionsWithDefaults)
      );
      const result = (0, _execute.executeSync)({
        schema,
        document
      });
      !result.errors && result.data || (0, _invariant.invariant)(false);
      return result.data;
    }
  }
});

// node_modules/graphql/utilities/buildClientSchema.js
var require_buildClientSchema = __commonJS({
  "node_modules/graphql/utilities/buildClientSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.buildClientSchema = buildClientSchema;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _isObjectLike = require_isObjectLike();
    var _keyValMap = require_keyValMap();
    var _parser = require_parser();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    var _scalars = require_scalars();
    var _schema = require_schema();
    var _valueFromAST = require_valueFromAST();
    function buildClientSchema(introspection, options) {
      (0, _isObjectLike.isObjectLike)(introspection) && (0, _isObjectLike.isObjectLike)(introspection.__schema) || (0, _devAssert.devAssert)(
        false,
        `Invalid or incomplete introspection result. Ensure that you are passing "data" property of introspection response and no "errors" was returned alongside: ${(0, _inspect.inspect)(introspection)}.`
      );
      const schemaIntrospection = introspection.__schema;
      const typeMap = (0, _keyValMap.keyValMap)(
        schemaIntrospection.types,
        (typeIntrospection) => typeIntrospection.name,
        (typeIntrospection) => buildType(typeIntrospection)
      );
      for (const stdType of [
        ..._scalars.specifiedScalarTypes,
        ..._introspection.introspectionTypes
      ]) {
        if (typeMap[stdType.name]) {
          typeMap[stdType.name] = stdType;
        }
      }
      const queryType = schemaIntrospection.queryType ? getObjectType(schemaIntrospection.queryType) : null;
      const mutationType = schemaIntrospection.mutationType ? getObjectType(schemaIntrospection.mutationType) : null;
      const subscriptionType = schemaIntrospection.subscriptionType ? getObjectType(schemaIntrospection.subscriptionType) : null;
      const directives = schemaIntrospection.directives ? schemaIntrospection.directives.map(buildDirective) : [];
      return new _schema.GraphQLSchema({
        description: schemaIntrospection.description,
        query: queryType,
        mutation: mutationType,
        subscription: subscriptionType,
        types: Object.values(typeMap),
        directives,
        assumeValid: options === null || options === void 0 ? void 0 : options.assumeValid
      });
      function getType(typeRef) {
        if (typeRef.kind === _introspection.TypeKind.LIST) {
          const itemRef = typeRef.ofType;
          if (!itemRef) {
            throw new Error("Decorated type deeper than introspection query.");
          }
          return new _definition.GraphQLList(getType(itemRef));
        }
        if (typeRef.kind === _introspection.TypeKind.NON_NULL) {
          const nullableRef = typeRef.ofType;
          if (!nullableRef) {
            throw new Error("Decorated type deeper than introspection query.");
          }
          const nullableType = getType(nullableRef);
          return new _definition.GraphQLNonNull(
            (0, _definition.assertNullableType)(nullableType)
          );
        }
        return getNamedType(typeRef);
      }
      function getNamedType(typeRef) {
        const typeName = typeRef.name;
        if (!typeName) {
          throw new Error(
            `Unknown type reference: ${(0, _inspect.inspect)(typeRef)}.`
          );
        }
        const type = typeMap[typeName];
        if (!type) {
          throw new Error(
            `Invalid or incomplete schema, unknown type: ${typeName}. Ensure that a full introspection query is used in order to build a client schema.`
          );
        }
        return type;
      }
      function getObjectType(typeRef) {
        return (0, _definition.assertObjectType)(getNamedType(typeRef));
      }
      function getInterfaceType(typeRef) {
        return (0, _definition.assertInterfaceType)(getNamedType(typeRef));
      }
      function buildType(type) {
        if (type != null && type.name != null && type.kind != null) {
          switch (type.kind) {
            case _introspection.TypeKind.SCALAR:
              return buildScalarDef(type);
            case _introspection.TypeKind.OBJECT:
              return buildObjectDef(type);
            case _introspection.TypeKind.INTERFACE:
              return buildInterfaceDef(type);
            case _introspection.TypeKind.UNION:
              return buildUnionDef(type);
            case _introspection.TypeKind.ENUM:
              return buildEnumDef(type);
            case _introspection.TypeKind.INPUT_OBJECT:
              return buildInputObjectDef(type);
          }
        }
        const typeStr = (0, _inspect.inspect)(type);
        throw new Error(
          `Invalid or incomplete introspection result. Ensure that a full introspection query is used in order to build a client schema: ${typeStr}.`
        );
      }
      function buildScalarDef(scalarIntrospection) {
        return new _definition.GraphQLScalarType({
          name: scalarIntrospection.name,
          description: scalarIntrospection.description,
          specifiedByURL: scalarIntrospection.specifiedByURL
        });
      }
      function buildImplementationsList(implementingIntrospection) {
        if (implementingIntrospection.interfaces === null && implementingIntrospection.kind === _introspection.TypeKind.INTERFACE) {
          return [];
        }
        if (!implementingIntrospection.interfaces) {
          const implementingIntrospectionStr = (0, _inspect.inspect)(
            implementingIntrospection
          );
          throw new Error(
            `Introspection result missing interfaces: ${implementingIntrospectionStr}.`
          );
        }
        return implementingIntrospection.interfaces.map(getInterfaceType);
      }
      function buildObjectDef(objectIntrospection) {
        return new _definition.GraphQLObjectType({
          name: objectIntrospection.name,
          description: objectIntrospection.description,
          interfaces: () => buildImplementationsList(objectIntrospection),
          fields: () => buildFieldDefMap(objectIntrospection)
        });
      }
      function buildInterfaceDef(interfaceIntrospection) {
        return new _definition.GraphQLInterfaceType({
          name: interfaceIntrospection.name,
          description: interfaceIntrospection.description,
          interfaces: () => buildImplementationsList(interfaceIntrospection),
          fields: () => buildFieldDefMap(interfaceIntrospection)
        });
      }
      function buildUnionDef(unionIntrospection) {
        if (!unionIntrospection.possibleTypes) {
          const unionIntrospectionStr = (0, _inspect.inspect)(unionIntrospection);
          throw new Error(
            `Introspection result missing possibleTypes: ${unionIntrospectionStr}.`
          );
        }
        return new _definition.GraphQLUnionType({
          name: unionIntrospection.name,
          description: unionIntrospection.description,
          types: () => unionIntrospection.possibleTypes.map(getObjectType)
        });
      }
      function buildEnumDef(enumIntrospection) {
        if (!enumIntrospection.enumValues) {
          const enumIntrospectionStr = (0, _inspect.inspect)(enumIntrospection);
          throw new Error(
            `Introspection result missing enumValues: ${enumIntrospectionStr}.`
          );
        }
        return new _definition.GraphQLEnumType({
          name: enumIntrospection.name,
          description: enumIntrospection.description,
          values: (0, _keyValMap.keyValMap)(
            enumIntrospection.enumValues,
            (valueIntrospection) => valueIntrospection.name,
            (valueIntrospection) => ({
              description: valueIntrospection.description,
              deprecationReason: valueIntrospection.deprecationReason
            })
          )
        });
      }
      function buildInputObjectDef(inputObjectIntrospection) {
        if (!inputObjectIntrospection.inputFields) {
          const inputObjectIntrospectionStr = (0, _inspect.inspect)(
            inputObjectIntrospection
          );
          throw new Error(
            `Introspection result missing inputFields: ${inputObjectIntrospectionStr}.`
          );
        }
        return new _definition.GraphQLInputObjectType({
          name: inputObjectIntrospection.name,
          description: inputObjectIntrospection.description,
          fields: () => buildInputValueDefMap(inputObjectIntrospection.inputFields)
        });
      }
      function buildFieldDefMap(typeIntrospection) {
        if (!typeIntrospection.fields) {
          throw new Error(
            `Introspection result missing fields: ${(0, _inspect.inspect)(
              typeIntrospection
            )}.`
          );
        }
        return (0, _keyValMap.keyValMap)(
          typeIntrospection.fields,
          (fieldIntrospection) => fieldIntrospection.name,
          buildField
        );
      }
      function buildField(fieldIntrospection) {
        const type = getType(fieldIntrospection.type);
        if (!(0, _definition.isOutputType)(type)) {
          const typeStr = (0, _inspect.inspect)(type);
          throw new Error(
            `Introspection must provide output type for fields, but received: ${typeStr}.`
          );
        }
        if (!fieldIntrospection.args) {
          const fieldIntrospectionStr = (0, _inspect.inspect)(fieldIntrospection);
          throw new Error(
            `Introspection result missing field args: ${fieldIntrospectionStr}.`
          );
        }
        return {
          description: fieldIntrospection.description,
          deprecationReason: fieldIntrospection.deprecationReason,
          type,
          args: buildInputValueDefMap(fieldIntrospection.args)
        };
      }
      function buildInputValueDefMap(inputValueIntrospections) {
        return (0, _keyValMap.keyValMap)(
          inputValueIntrospections,
          (inputValue) => inputValue.name,
          buildInputValue
        );
      }
      function buildInputValue(inputValueIntrospection) {
        const type = getType(inputValueIntrospection.type);
        if (!(0, _definition.isInputType)(type)) {
          const typeStr = (0, _inspect.inspect)(type);
          throw new Error(
            `Introspection must provide input type for arguments, but received: ${typeStr}.`
          );
        }
        const defaultValue = inputValueIntrospection.defaultValue != null ? (0, _valueFromAST.valueFromAST)(
          (0, _parser.parseValue)(inputValueIntrospection.defaultValue),
          type
        ) : void 0;
        return {
          description: inputValueIntrospection.description,
          type,
          defaultValue,
          deprecationReason: inputValueIntrospection.deprecationReason
        };
      }
      function buildDirective(directiveIntrospection) {
        if (!directiveIntrospection.args) {
          const directiveIntrospectionStr = (0, _inspect.inspect)(
            directiveIntrospection
          );
          throw new Error(
            `Introspection result missing directive args: ${directiveIntrospectionStr}.`
          );
        }
        if (!directiveIntrospection.locations) {
          const directiveIntrospectionStr = (0, _inspect.inspect)(
            directiveIntrospection
          );
          throw new Error(
            `Introspection result missing directive locations: ${directiveIntrospectionStr}.`
          );
        }
        return new _directives.GraphQLDirective({
          name: directiveIntrospection.name,
          description: directiveIntrospection.description,
          isRepeatable: directiveIntrospection.isRepeatable,
          locations: directiveIntrospection.locations.slice(),
          args: buildInputValueDefMap(directiveIntrospection.args)
        });
      }
    }
  }
});

// node_modules/graphql/utilities/extendSchema.js
var require_extendSchema = __commonJS({
  "node_modules/graphql/utilities/extendSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.extendSchema = extendSchema;
    exports2.extendSchemaImpl = extendSchemaImpl;
    var _devAssert = require_devAssert();
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _keyMap = require_keyMap();
    var _mapValue = require_mapValue();
    var _kinds = require_kinds();
    var _predicates = require_predicates();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    var _scalars = require_scalars();
    var _schema = require_schema();
    var _validate = require_validate2();
    var _values = require_values();
    var _valueFromAST = require_valueFromAST();
    function extendSchema(schema, documentAST, options) {
      (0, _schema.assertSchema)(schema);
      documentAST != null && documentAST.kind === _kinds.Kind.DOCUMENT || (0, _devAssert.devAssert)(false, "Must provide valid Document AST.");
      if ((options === null || options === void 0 ? void 0 : options.assumeValid) !== true && (options === null || options === void 0 ? void 0 : options.assumeValidSDL) !== true) {
        (0, _validate.assertValidSDLExtension)(documentAST, schema);
      }
      const schemaConfig = schema.toConfig();
      const extendedConfig = extendSchemaImpl(schemaConfig, documentAST, options);
      return schemaConfig === extendedConfig ? schema : new _schema.GraphQLSchema(extendedConfig);
    }
    function extendSchemaImpl(schemaConfig, documentAST, options) {
      var _schemaDef, _schemaDef$descriptio, _schemaDef2, _options$assumeValid;
      const typeDefs = [];
      const typeExtensionsMap = /* @__PURE__ */ Object.create(null);
      const directiveDefs = [];
      let schemaDef;
      const schemaExtensions = [];
      for (const def of documentAST.definitions) {
        if (def.kind === _kinds.Kind.SCHEMA_DEFINITION) {
          schemaDef = def;
        } else if (def.kind === _kinds.Kind.SCHEMA_EXTENSION) {
          schemaExtensions.push(def);
        } else if ((0, _predicates.isTypeDefinitionNode)(def)) {
          typeDefs.push(def);
        } else if ((0, _predicates.isTypeExtensionNode)(def)) {
          const extendedTypeName = def.name.value;
          const existingTypeExtensions = typeExtensionsMap[extendedTypeName];
          typeExtensionsMap[extendedTypeName] = existingTypeExtensions ? existingTypeExtensions.concat([def]) : [def];
        } else if (def.kind === _kinds.Kind.DIRECTIVE_DEFINITION) {
          directiveDefs.push(def);
        }
      }
      if (Object.keys(typeExtensionsMap).length === 0 && typeDefs.length === 0 && directiveDefs.length === 0 && schemaExtensions.length === 0 && schemaDef == null) {
        return schemaConfig;
      }
      const typeMap = /* @__PURE__ */ Object.create(null);
      for (const existingType of schemaConfig.types) {
        typeMap[existingType.name] = extendNamedType(existingType);
      }
      for (const typeNode of typeDefs) {
        var _stdTypeMap$name;
        const name = typeNode.name.value;
        typeMap[name] = (_stdTypeMap$name = stdTypeMap[name]) !== null && _stdTypeMap$name !== void 0 ? _stdTypeMap$name : buildType(typeNode);
      }
      const operationTypes = {
        // Get the extended root operation types.
        query: schemaConfig.query && replaceNamedType(schemaConfig.query),
        mutation: schemaConfig.mutation && replaceNamedType(schemaConfig.mutation),
        subscription: schemaConfig.subscription && replaceNamedType(schemaConfig.subscription),
        // Then, incorporate schema definition and all schema extensions.
        ...schemaDef && getOperationTypes([schemaDef]),
        ...getOperationTypes(schemaExtensions)
      };
      return {
        description: (_schemaDef = schemaDef) === null || _schemaDef === void 0 ? void 0 : (_schemaDef$descriptio = _schemaDef.description) === null || _schemaDef$descriptio === void 0 ? void 0 : _schemaDef$descriptio.value,
        ...operationTypes,
        types: Object.values(typeMap),
        directives: [
          ...schemaConfig.directives.map(replaceDirective),
          ...directiveDefs.map(buildDirective)
        ],
        extensions: /* @__PURE__ */ Object.create(null),
        astNode: (_schemaDef2 = schemaDef) !== null && _schemaDef2 !== void 0 ? _schemaDef2 : schemaConfig.astNode,
        extensionASTNodes: schemaConfig.extensionASTNodes.concat(schemaExtensions),
        assumeValid: (_options$assumeValid = options === null || options === void 0 ? void 0 : options.assumeValid) !== null && _options$assumeValid !== void 0 ? _options$assumeValid : false
      };
      function replaceType(type) {
        if ((0, _definition.isListType)(type)) {
          return new _definition.GraphQLList(replaceType(type.ofType));
        }
        if ((0, _definition.isNonNullType)(type)) {
          return new _definition.GraphQLNonNull(replaceType(type.ofType));
        }
        return replaceNamedType(type);
      }
      function replaceNamedType(type) {
        return typeMap[type.name];
      }
      function replaceDirective(directive) {
        const config = directive.toConfig();
        return new _directives.GraphQLDirective({
          ...config,
          args: (0, _mapValue.mapValue)(config.args, extendArg)
        });
      }
      function extendNamedType(type) {
        if ((0, _introspection.isIntrospectionType)(type) || (0, _scalars.isSpecifiedScalarType)(type)) {
          return type;
        }
        if ((0, _definition.isScalarType)(type)) {
          return extendScalarType(type);
        }
        if ((0, _definition.isObjectType)(type)) {
          return extendObjectType(type);
        }
        if ((0, _definition.isInterfaceType)(type)) {
          return extendInterfaceType(type);
        }
        if ((0, _definition.isUnionType)(type)) {
          return extendUnionType(type);
        }
        if ((0, _definition.isEnumType)(type)) {
          return extendEnumType(type);
        }
        if ((0, _definition.isInputObjectType)(type)) {
          return extendInputObjectType(type);
        }
        (0, _invariant.invariant)(
          false,
          "Unexpected type: " + (0, _inspect.inspect)(type)
        );
      }
      function extendInputObjectType(type) {
        var _typeExtensionsMap$co;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$co = typeExtensionsMap[config.name]) !== null && _typeExtensionsMap$co !== void 0 ? _typeExtensionsMap$co : [];
        return new _definition.GraphQLInputObjectType({
          ...config,
          fields: () => ({
            ...(0, _mapValue.mapValue)(config.fields, (field) => ({
              ...field,
              type: replaceType(field.type)
            })),
            ...buildInputFieldMap(extensions)
          }),
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendEnumType(type) {
        var _typeExtensionsMap$ty;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$ty = typeExtensionsMap[type.name]) !== null && _typeExtensionsMap$ty !== void 0 ? _typeExtensionsMap$ty : [];
        return new _definition.GraphQLEnumType({
          ...config,
          values: { ...config.values, ...buildEnumValueMap(extensions) },
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendScalarType(type) {
        var _typeExtensionsMap$co2;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$co2 = typeExtensionsMap[config.name]) !== null && _typeExtensionsMap$co2 !== void 0 ? _typeExtensionsMap$co2 : [];
        let specifiedByURL = config.specifiedByURL;
        for (const extensionNode of extensions) {
          var _getSpecifiedByURL;
          specifiedByURL = (_getSpecifiedByURL = getSpecifiedByURL(extensionNode)) !== null && _getSpecifiedByURL !== void 0 ? _getSpecifiedByURL : specifiedByURL;
        }
        return new _definition.GraphQLScalarType({
          ...config,
          specifiedByURL,
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendObjectType(type) {
        var _typeExtensionsMap$co3;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$co3 = typeExtensionsMap[config.name]) !== null && _typeExtensionsMap$co3 !== void 0 ? _typeExtensionsMap$co3 : [];
        return new _definition.GraphQLObjectType({
          ...config,
          interfaces: () => [
            ...type.getInterfaces().map(replaceNamedType),
            ...buildInterfaces(extensions)
          ],
          fields: () => ({
            ...(0, _mapValue.mapValue)(config.fields, extendField),
            ...buildFieldMap(extensions)
          }),
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendInterfaceType(type) {
        var _typeExtensionsMap$co4;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$co4 = typeExtensionsMap[config.name]) !== null && _typeExtensionsMap$co4 !== void 0 ? _typeExtensionsMap$co4 : [];
        return new _definition.GraphQLInterfaceType({
          ...config,
          interfaces: () => [
            ...type.getInterfaces().map(replaceNamedType),
            ...buildInterfaces(extensions)
          ],
          fields: () => ({
            ...(0, _mapValue.mapValue)(config.fields, extendField),
            ...buildFieldMap(extensions)
          }),
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendUnionType(type) {
        var _typeExtensionsMap$co5;
        const config = type.toConfig();
        const extensions = (_typeExtensionsMap$co5 = typeExtensionsMap[config.name]) !== null && _typeExtensionsMap$co5 !== void 0 ? _typeExtensionsMap$co5 : [];
        return new _definition.GraphQLUnionType({
          ...config,
          types: () => [
            ...type.getTypes().map(replaceNamedType),
            ...buildUnionTypes(extensions)
          ],
          extensionASTNodes: config.extensionASTNodes.concat(extensions)
        });
      }
      function extendField(field) {
        return {
          ...field,
          type: replaceType(field.type),
          args: field.args && (0, _mapValue.mapValue)(field.args, extendArg)
        };
      }
      function extendArg(arg) {
        return { ...arg, type: replaceType(arg.type) };
      }
      function getOperationTypes(nodes) {
        const opTypes = {};
        for (const node of nodes) {
          var _node$operationTypes;
          const operationTypesNodes = (
            /* c8 ignore next */
            (_node$operationTypes = node.operationTypes) !== null && _node$operationTypes !== void 0 ? _node$operationTypes : []
          );
          for (const operationType of operationTypesNodes) {
            opTypes[operationType.operation] = getNamedType(operationType.type);
          }
        }
        return opTypes;
      }
      function getNamedType(node) {
        var _stdTypeMap$name2;
        const name = node.name.value;
        const type = (_stdTypeMap$name2 = stdTypeMap[name]) !== null && _stdTypeMap$name2 !== void 0 ? _stdTypeMap$name2 : typeMap[name];
        if (type === void 0) {
          throw new Error(`Unknown type: "${name}".`);
        }
        return type;
      }
      function getWrappedType(node) {
        if (node.kind === _kinds.Kind.LIST_TYPE) {
          return new _definition.GraphQLList(getWrappedType(node.type));
        }
        if (node.kind === _kinds.Kind.NON_NULL_TYPE) {
          return new _definition.GraphQLNonNull(getWrappedType(node.type));
        }
        return getNamedType(node);
      }
      function buildDirective(node) {
        var _node$description;
        return new _directives.GraphQLDirective({
          name: node.name.value,
          description: (_node$description = node.description) === null || _node$description === void 0 ? void 0 : _node$description.value,
          // @ts-expect-error
          locations: node.locations.map(({ value }) => value),
          isRepeatable: node.repeatable,
          args: buildArgumentMap(node.arguments),
          astNode: node
        });
      }
      function buildFieldMap(nodes) {
        const fieldConfigMap = /* @__PURE__ */ Object.create(null);
        for (const node of nodes) {
          var _node$fields;
          const nodeFields = (
            /* c8 ignore next */
            (_node$fields = node.fields) !== null && _node$fields !== void 0 ? _node$fields : []
          );
          for (const field of nodeFields) {
            var _field$description;
            fieldConfigMap[field.name.value] = {
              // Note: While this could make assertions to get the correctly typed
              // value, that would throw immediately while type system validation
              // with validateSchema() will produce more actionable results.
              type: getWrappedType(field.type),
              description: (_field$description = field.description) === null || _field$description === void 0 ? void 0 : _field$description.value,
              args: buildArgumentMap(field.arguments),
              deprecationReason: getDeprecationReason(field),
              astNode: field
            };
          }
        }
        return fieldConfigMap;
      }
      function buildArgumentMap(args) {
        const argsNodes = (
          /* c8 ignore next */
          args !== null && args !== void 0 ? args : []
        );
        const argConfigMap = /* @__PURE__ */ Object.create(null);
        for (const arg of argsNodes) {
          var _arg$description;
          const type = getWrappedType(arg.type);
          argConfigMap[arg.name.value] = {
            type,
            description: (_arg$description = arg.description) === null || _arg$description === void 0 ? void 0 : _arg$description.value,
            defaultValue: (0, _valueFromAST.valueFromAST)(arg.defaultValue, type),
            deprecationReason: getDeprecationReason(arg),
            astNode: arg
          };
        }
        return argConfigMap;
      }
      function buildInputFieldMap(nodes) {
        const inputFieldMap = /* @__PURE__ */ Object.create(null);
        for (const node of nodes) {
          var _node$fields2;
          const fieldsNodes = (
            /* c8 ignore next */
            (_node$fields2 = node.fields) !== null && _node$fields2 !== void 0 ? _node$fields2 : []
          );
          for (const field of fieldsNodes) {
            var _field$description2;
            const type = getWrappedType(field.type);
            inputFieldMap[field.name.value] = {
              type,
              description: (_field$description2 = field.description) === null || _field$description2 === void 0 ? void 0 : _field$description2.value,
              defaultValue: (0, _valueFromAST.valueFromAST)(
                field.defaultValue,
                type
              ),
              deprecationReason: getDeprecationReason(field),
              astNode: field
            };
          }
        }
        return inputFieldMap;
      }
      function buildEnumValueMap(nodes) {
        const enumValueMap = /* @__PURE__ */ Object.create(null);
        for (const node of nodes) {
          var _node$values;
          const valuesNodes = (
            /* c8 ignore next */
            (_node$values = node.values) !== null && _node$values !== void 0 ? _node$values : []
          );
          for (const value of valuesNodes) {
            var _value$description;
            enumValueMap[value.name.value] = {
              description: (_value$description = value.description) === null || _value$description === void 0 ? void 0 : _value$description.value,
              deprecationReason: getDeprecationReason(value),
              astNode: value
            };
          }
        }
        return enumValueMap;
      }
      function buildInterfaces(nodes) {
        return nodes.flatMap(
          // FIXME: https://github.com/graphql/graphql-js/issues/2203
          (node) => {
            var _node$interfaces$map, _node$interfaces;
            return (
              /* c8 ignore next */
              (_node$interfaces$map = (_node$interfaces = node.interfaces) === null || _node$interfaces === void 0 ? void 0 : _node$interfaces.map(getNamedType)) !== null && _node$interfaces$map !== void 0 ? _node$interfaces$map : []
            );
          }
        );
      }
      function buildUnionTypes(nodes) {
        return nodes.flatMap(
          // FIXME: https://github.com/graphql/graphql-js/issues/2203
          (node) => {
            var _node$types$map, _node$types;
            return (
              /* c8 ignore next */
              (_node$types$map = (_node$types = node.types) === null || _node$types === void 0 ? void 0 : _node$types.map(getNamedType)) !== null && _node$types$map !== void 0 ? _node$types$map : []
            );
          }
        );
      }
      function buildType(astNode) {
        var _typeExtensionsMap$na;
        const name = astNode.name.value;
        const extensionASTNodes = (_typeExtensionsMap$na = typeExtensionsMap[name]) !== null && _typeExtensionsMap$na !== void 0 ? _typeExtensionsMap$na : [];
        switch (astNode.kind) {
          case _kinds.Kind.OBJECT_TYPE_DEFINITION: {
            var _astNode$description;
            const allNodes = [astNode, ...extensionASTNodes];
            return new _definition.GraphQLObjectType({
              name,
              description: (_astNode$description = astNode.description) === null || _astNode$description === void 0 ? void 0 : _astNode$description.value,
              interfaces: () => buildInterfaces(allNodes),
              fields: () => buildFieldMap(allNodes),
              astNode,
              extensionASTNodes
            });
          }
          case _kinds.Kind.INTERFACE_TYPE_DEFINITION: {
            var _astNode$description2;
            const allNodes = [astNode, ...extensionASTNodes];
            return new _definition.GraphQLInterfaceType({
              name,
              description: (_astNode$description2 = astNode.description) === null || _astNode$description2 === void 0 ? void 0 : _astNode$description2.value,
              interfaces: () => buildInterfaces(allNodes),
              fields: () => buildFieldMap(allNodes),
              astNode,
              extensionASTNodes
            });
          }
          case _kinds.Kind.ENUM_TYPE_DEFINITION: {
            var _astNode$description3;
            const allNodes = [astNode, ...extensionASTNodes];
            return new _definition.GraphQLEnumType({
              name,
              description: (_astNode$description3 = astNode.description) === null || _astNode$description3 === void 0 ? void 0 : _astNode$description3.value,
              values: buildEnumValueMap(allNodes),
              astNode,
              extensionASTNodes
            });
          }
          case _kinds.Kind.UNION_TYPE_DEFINITION: {
            var _astNode$description4;
            const allNodes = [astNode, ...extensionASTNodes];
            return new _definition.GraphQLUnionType({
              name,
              description: (_astNode$description4 = astNode.description) === null || _astNode$description4 === void 0 ? void 0 : _astNode$description4.value,
              types: () => buildUnionTypes(allNodes),
              astNode,
              extensionASTNodes
            });
          }
          case _kinds.Kind.SCALAR_TYPE_DEFINITION: {
            var _astNode$description5;
            return new _definition.GraphQLScalarType({
              name,
              description: (_astNode$description5 = astNode.description) === null || _astNode$description5 === void 0 ? void 0 : _astNode$description5.value,
              specifiedByURL: getSpecifiedByURL(astNode),
              astNode,
              extensionASTNodes
            });
          }
          case _kinds.Kind.INPUT_OBJECT_TYPE_DEFINITION: {
            var _astNode$description6;
            const allNodes = [astNode, ...extensionASTNodes];
            return new _definition.GraphQLInputObjectType({
              name,
              description: (_astNode$description6 = astNode.description) === null || _astNode$description6 === void 0 ? void 0 : _astNode$description6.value,
              fields: () => buildInputFieldMap(allNodes),
              astNode,
              extensionASTNodes
            });
          }
        }
      }
    }
    var stdTypeMap = (0, _keyMap.keyMap)(
      [..._scalars.specifiedScalarTypes, ..._introspection.introspectionTypes],
      (type) => type.name
    );
    function getDeprecationReason(node) {
      const deprecated = (0, _values.getDirectiveValues)(
        _directives.GraphQLDeprecatedDirective,
        node
      );
      return deprecated === null || deprecated === void 0 ? void 0 : deprecated.reason;
    }
    function getSpecifiedByURL(node) {
      const specifiedBy = (0, _values.getDirectiveValues)(
        _directives.GraphQLSpecifiedByDirective,
        node
      );
      return specifiedBy === null || specifiedBy === void 0 ? void 0 : specifiedBy.url;
    }
  }
});

// node_modules/graphql/utilities/buildASTSchema.js
var require_buildASTSchema = __commonJS({
  "node_modules/graphql/utilities/buildASTSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.buildASTSchema = buildASTSchema;
    exports2.buildSchema = buildSchema;
    var _devAssert = require_devAssert();
    var _kinds = require_kinds();
    var _parser = require_parser();
    var _directives = require_directives();
    var _schema = require_schema();
    var _validate = require_validate2();
    var _extendSchema = require_extendSchema();
    function buildASTSchema(documentAST, options) {
      documentAST != null && documentAST.kind === _kinds.Kind.DOCUMENT || (0, _devAssert.devAssert)(false, "Must provide valid Document AST.");
      if ((options === null || options === void 0 ? void 0 : options.assumeValid) !== true && (options === null || options === void 0 ? void 0 : options.assumeValidSDL) !== true) {
        (0, _validate.assertValidSDL)(documentAST);
      }
      const emptySchemaConfig = {
        description: void 0,
        types: [],
        directives: [],
        extensions: /* @__PURE__ */ Object.create(null),
        extensionASTNodes: [],
        assumeValid: false
      };
      const config = (0, _extendSchema.extendSchemaImpl)(
        emptySchemaConfig,
        documentAST,
        options
      );
      if (config.astNode == null) {
        for (const type of config.types) {
          switch (type.name) {
            case "Query":
              config.query = type;
              break;
            case "Mutation":
              config.mutation = type;
              break;
            case "Subscription":
              config.subscription = type;
              break;
          }
        }
      }
      const directives = [
        ...config.directives,
        // If specified directives were not explicitly declared, add them.
        ..._directives.specifiedDirectives.filter(
          (stdDirective) => config.directives.every(
            (directive) => directive.name !== stdDirective.name
          )
        )
      ];
      return new _schema.GraphQLSchema({ ...config, directives });
    }
    function buildSchema(source, options) {
      const document = (0, _parser.parse)(source, {
        noLocation: options === null || options === void 0 ? void 0 : options.noLocation,
        allowLegacyFragmentVariables: options === null || options === void 0 ? void 0 : options.allowLegacyFragmentVariables
      });
      return buildASTSchema(document, {
        assumeValidSDL: options === null || options === void 0 ? void 0 : options.assumeValidSDL,
        assumeValid: options === null || options === void 0 ? void 0 : options.assumeValid
      });
    }
  }
});

// node_modules/graphql/utilities/lexicographicSortSchema.js
var require_lexicographicSortSchema = __commonJS({
  "node_modules/graphql/utilities/lexicographicSortSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.lexicographicSortSchema = lexicographicSortSchema;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _keyValMap = require_keyValMap();
    var _naturalCompare = require_naturalCompare();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    var _schema = require_schema();
    function lexicographicSortSchema(schema) {
      const schemaConfig = schema.toConfig();
      const typeMap = (0, _keyValMap.keyValMap)(
        sortByName(schemaConfig.types),
        (type) => type.name,
        sortNamedType
      );
      return new _schema.GraphQLSchema({
        ...schemaConfig,
        types: Object.values(typeMap),
        directives: sortByName(schemaConfig.directives).map(sortDirective),
        query: replaceMaybeType(schemaConfig.query),
        mutation: replaceMaybeType(schemaConfig.mutation),
        subscription: replaceMaybeType(schemaConfig.subscription)
      });
      function replaceType(type) {
        if ((0, _definition.isListType)(type)) {
          return new _definition.GraphQLList(replaceType(type.ofType));
        } else if ((0, _definition.isNonNullType)(type)) {
          return new _definition.GraphQLNonNull(replaceType(type.ofType));
        }
        return replaceNamedType(type);
      }
      function replaceNamedType(type) {
        return typeMap[type.name];
      }
      function replaceMaybeType(maybeType) {
        return maybeType && replaceNamedType(maybeType);
      }
      function sortDirective(directive) {
        const config = directive.toConfig();
        return new _directives.GraphQLDirective({
          ...config,
          locations: sortBy(config.locations, (x) => x),
          args: sortArgs(config.args)
        });
      }
      function sortArgs(args) {
        return sortObjMap(args, (arg) => ({ ...arg, type: replaceType(arg.type) }));
      }
      function sortFields(fieldsMap) {
        return sortObjMap(fieldsMap, (field) => ({
          ...field,
          type: replaceType(field.type),
          args: field.args && sortArgs(field.args)
        }));
      }
      function sortInputFields(fieldsMap) {
        return sortObjMap(fieldsMap, (field) => ({
          ...field,
          type: replaceType(field.type)
        }));
      }
      function sortTypes(array) {
        return sortByName(array).map(replaceNamedType);
      }
      function sortNamedType(type) {
        if ((0, _definition.isScalarType)(type) || (0, _introspection.isIntrospectionType)(type)) {
          return type;
        }
        if ((0, _definition.isObjectType)(type)) {
          const config = type.toConfig();
          return new _definition.GraphQLObjectType({
            ...config,
            interfaces: () => sortTypes(config.interfaces),
            fields: () => sortFields(config.fields)
          });
        }
        if ((0, _definition.isInterfaceType)(type)) {
          const config = type.toConfig();
          return new _definition.GraphQLInterfaceType({
            ...config,
            interfaces: () => sortTypes(config.interfaces),
            fields: () => sortFields(config.fields)
          });
        }
        if ((0, _definition.isUnionType)(type)) {
          const config = type.toConfig();
          return new _definition.GraphQLUnionType({
            ...config,
            types: () => sortTypes(config.types)
          });
        }
        if ((0, _definition.isEnumType)(type)) {
          const config = type.toConfig();
          return new _definition.GraphQLEnumType({
            ...config,
            values: sortObjMap(config.values, (value) => value)
          });
        }
        if ((0, _definition.isInputObjectType)(type)) {
          const config = type.toConfig();
          return new _definition.GraphQLInputObjectType({
            ...config,
            fields: () => sortInputFields(config.fields)
          });
        }
        (0, _invariant.invariant)(
          false,
          "Unexpected type: " + (0, _inspect.inspect)(type)
        );
      }
    }
    function sortObjMap(map, sortValueFn) {
      const sortedMap = /* @__PURE__ */ Object.create(null);
      for (const key of Object.keys(map).sort(_naturalCompare.naturalCompare)) {
        sortedMap[key] = sortValueFn(map[key]);
      }
      return sortedMap;
    }
    function sortByName(array) {
      return sortBy(array, (obj) => obj.name);
    }
    function sortBy(array, mapToKey) {
      return array.slice().sort((obj1, obj2) => {
        const key1 = mapToKey(obj1);
        const key2 = mapToKey(obj2);
        return (0, _naturalCompare.naturalCompare)(key1, key2);
      });
    }
  }
});

// node_modules/graphql/utilities/printSchema.js
var require_printSchema = __commonJS({
  "node_modules/graphql/utilities/printSchema.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.printIntrospectionSchema = printIntrospectionSchema;
    exports2.printSchema = printSchema;
    exports2.printType = printType;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _blockString = require_blockString();
    var _kinds = require_kinds();
    var _printer = require_printer();
    var _definition = require_definition();
    var _directives = require_directives();
    var _introspection = require_introspection();
    var _scalars = require_scalars();
    var _astFromValue = require_astFromValue();
    function printSchema(schema) {
      return printFilteredSchema(
        schema,
        (n) => !(0, _directives.isSpecifiedDirective)(n),
        isDefinedType
      );
    }
    function printIntrospectionSchema(schema) {
      return printFilteredSchema(
        schema,
        _directives.isSpecifiedDirective,
        _introspection.isIntrospectionType
      );
    }
    function isDefinedType(type) {
      return !(0, _scalars.isSpecifiedScalarType)(type) && !(0, _introspection.isIntrospectionType)(type);
    }
    function printFilteredSchema(schema, directiveFilter, typeFilter) {
      const directives = schema.getDirectives().filter(directiveFilter);
      const types = Object.values(schema.getTypeMap()).filter(typeFilter);
      return [
        printSchemaDefinition(schema),
        ...directives.map((directive) => printDirective(directive)),
        ...types.map((type) => printType(type))
      ].filter(Boolean).join("\n\n");
    }
    function printSchemaDefinition(schema) {
      if (schema.description == null && isSchemaOfCommonNames(schema)) {
        return;
      }
      const operationTypes = [];
      const queryType = schema.getQueryType();
      if (queryType) {
        operationTypes.push(`  query: ${queryType.name}`);
      }
      const mutationType = schema.getMutationType();
      if (mutationType) {
        operationTypes.push(`  mutation: ${mutationType.name}`);
      }
      const subscriptionType = schema.getSubscriptionType();
      if (subscriptionType) {
        operationTypes.push(`  subscription: ${subscriptionType.name}`);
      }
      return printDescription(schema) + `schema {
${operationTypes.join("\n")}
}`;
    }
    function isSchemaOfCommonNames(schema) {
      const queryType = schema.getQueryType();
      if (queryType && queryType.name !== "Query") {
        return false;
      }
      const mutationType = schema.getMutationType();
      if (mutationType && mutationType.name !== "Mutation") {
        return false;
      }
      const subscriptionType = schema.getSubscriptionType();
      if (subscriptionType && subscriptionType.name !== "Subscription") {
        return false;
      }
      return true;
    }
    function printType(type) {
      if ((0, _definition.isScalarType)(type)) {
        return printScalar(type);
      }
      if ((0, _definition.isObjectType)(type)) {
        return printObject(type);
      }
      if ((0, _definition.isInterfaceType)(type)) {
        return printInterface(type);
      }
      if ((0, _definition.isUnionType)(type)) {
        return printUnion(type);
      }
      if ((0, _definition.isEnumType)(type)) {
        return printEnum(type);
      }
      if ((0, _definition.isInputObjectType)(type)) {
        return printInputObject(type);
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected type: " + (0, _inspect.inspect)(type)
      );
    }
    function printScalar(type) {
      return printDescription(type) + `scalar ${type.name}` + printSpecifiedByURL(type);
    }
    function printImplementedInterfaces(type) {
      const interfaces = type.getInterfaces();
      return interfaces.length ? " implements " + interfaces.map((i) => i.name).join(" & ") : "";
    }
    function printObject(type) {
      return printDescription(type) + `type ${type.name}` + printImplementedInterfaces(type) + printFields(type);
    }
    function printInterface(type) {
      return printDescription(type) + `interface ${type.name}` + printImplementedInterfaces(type) + printFields(type);
    }
    function printUnion(type) {
      const types = type.getTypes();
      const possibleTypes = types.length ? " = " + types.join(" | ") : "";
      return printDescription(type) + "union " + type.name + possibleTypes;
    }
    function printEnum(type) {
      const values = type.getValues().map(
        (value, i) => printDescription(value, "  ", !i) + "  " + value.name + printDeprecated(value.deprecationReason)
      );
      return printDescription(type) + `enum ${type.name}` + printBlock(values);
    }
    function printInputObject(type) {
      const fields = Object.values(type.getFields()).map(
        (f, i) => printDescription(f, "  ", !i) + "  " + printInputValue(f)
      );
      return printDescription(type) + `input ${type.name}` + printBlock(fields);
    }
    function printFields(type) {
      const fields = Object.values(type.getFields()).map(
        (f, i) => printDescription(f, "  ", !i) + "  " + f.name + printArgs(f.args, "  ") + ": " + String(f.type) + printDeprecated(f.deprecationReason)
      );
      return printBlock(fields);
    }
    function printBlock(items) {
      return items.length !== 0 ? " {\n" + items.join("\n") + "\n}" : "";
    }
    function printArgs(args, indentation = "") {
      if (args.length === 0) {
        return "";
      }
      if (args.every((arg) => !arg.description)) {
        return "(" + args.map(printInputValue).join(", ") + ")";
      }
      return "(\n" + args.map(
        (arg, i) => printDescription(arg, "  " + indentation, !i) + "  " + indentation + printInputValue(arg)
      ).join("\n") + "\n" + indentation + ")";
    }
    function printInputValue(arg) {
      const defaultAST = (0, _astFromValue.astFromValue)(
        arg.defaultValue,
        arg.type
      );
      let argDecl = arg.name + ": " + String(arg.type);
      if (defaultAST) {
        argDecl += ` = ${(0, _printer.print)(defaultAST)}`;
      }
      return argDecl + printDeprecated(arg.deprecationReason);
    }
    function printDirective(directive) {
      return printDescription(directive) + "directive @" + directive.name + printArgs(directive.args) + (directive.isRepeatable ? " repeatable" : "") + " on " + directive.locations.join(" | ");
    }
    function printDeprecated(reason) {
      if (reason == null) {
        return "";
      }
      if (reason !== _directives.DEFAULT_DEPRECATION_REASON) {
        const astValue = (0, _printer.print)({
          kind: _kinds.Kind.STRING,
          value: reason
        });
        return ` @deprecated(reason: ${astValue})`;
      }
      return " @deprecated";
    }
    function printSpecifiedByURL(scalar) {
      if (scalar.specifiedByURL == null) {
        return "";
      }
      const astValue = (0, _printer.print)({
        kind: _kinds.Kind.STRING,
        value: scalar.specifiedByURL
      });
      return ` @specifiedBy(url: ${astValue})`;
    }
    function printDescription(def, indentation = "", firstInBlock = true) {
      const { description } = def;
      if (description == null) {
        return "";
      }
      const blockString = (0, _printer.print)({
        kind: _kinds.Kind.STRING,
        value: description,
        block: (0, _blockString.isPrintableAsBlockString)(description)
      });
      const prefix = indentation && !firstInBlock ? "\n" + indentation : indentation;
      return prefix + blockString.replace(/\n/g, "\n" + indentation) + "\n";
    }
  }
});

// node_modules/graphql/utilities/concatAST.js
var require_concatAST = __commonJS({
  "node_modules/graphql/utilities/concatAST.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.concatAST = concatAST;
    var _kinds = require_kinds();
    function concatAST(documents) {
      const definitions = [];
      for (const doc of documents) {
        definitions.push(...doc.definitions);
      }
      return {
        kind: _kinds.Kind.DOCUMENT,
        definitions
      };
    }
  }
});

// node_modules/graphql/utilities/separateOperations.js
var require_separateOperations = __commonJS({
  "node_modules/graphql/utilities/separateOperations.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.separateOperations = separateOperations;
    var _kinds = require_kinds();
    var _visitor = require_visitor();
    function separateOperations(documentAST) {
      const operations = [];
      const depGraph = /* @__PURE__ */ Object.create(null);
      for (const definitionNode of documentAST.definitions) {
        switch (definitionNode.kind) {
          case _kinds.Kind.OPERATION_DEFINITION:
            operations.push(definitionNode);
            break;
          case _kinds.Kind.FRAGMENT_DEFINITION:
            depGraph[definitionNode.name.value] = collectDependencies(
              definitionNode.selectionSet
            );
            break;
          default:
        }
      }
      const separatedDocumentASTs = /* @__PURE__ */ Object.create(null);
      for (const operation of operations) {
        const dependencies = /* @__PURE__ */ new Set();
        for (const fragmentName of collectDependencies(operation.selectionSet)) {
          collectTransitiveDependencies(dependencies, depGraph, fragmentName);
        }
        const operationName = operation.name ? operation.name.value : "";
        separatedDocumentASTs[operationName] = {
          kind: _kinds.Kind.DOCUMENT,
          definitions: documentAST.definitions.filter(
            (node) => node === operation || node.kind === _kinds.Kind.FRAGMENT_DEFINITION && dependencies.has(node.name.value)
          )
        };
      }
      return separatedDocumentASTs;
    }
    function collectTransitiveDependencies(collected, depGraph, fromName) {
      if (!collected.has(fromName)) {
        collected.add(fromName);
        const immediateDeps = depGraph[fromName];
        if (immediateDeps !== void 0) {
          for (const toName of immediateDeps) {
            collectTransitiveDependencies(collected, depGraph, toName);
          }
        }
      }
    }
    function collectDependencies(selectionSet) {
      const dependencies = [];
      (0, _visitor.visit)(selectionSet, {
        FragmentSpread(node) {
          dependencies.push(node.name.value);
        }
      });
      return dependencies;
    }
  }
});

// node_modules/graphql/utilities/stripIgnoredCharacters.js
var require_stripIgnoredCharacters = __commonJS({
  "node_modules/graphql/utilities/stripIgnoredCharacters.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.stripIgnoredCharacters = stripIgnoredCharacters;
    var _blockString = require_blockString();
    var _lexer = require_lexer();
    var _source = require_source();
    var _tokenKind = require_tokenKind();
    function stripIgnoredCharacters(source) {
      const sourceObj = (0, _source.isSource)(source) ? source : new _source.Source(source);
      const body = sourceObj.body;
      const lexer = new _lexer.Lexer(sourceObj);
      let strippedBody = "";
      let wasLastAddedTokenNonPunctuator = false;
      while (lexer.advance().kind !== _tokenKind.TokenKind.EOF) {
        const currentToken = lexer.token;
        const tokenKind = currentToken.kind;
        const isNonPunctuator = !(0, _lexer.isPunctuatorTokenKind)(
          currentToken.kind
        );
        if (wasLastAddedTokenNonPunctuator) {
          if (isNonPunctuator || currentToken.kind === _tokenKind.TokenKind.SPREAD) {
            strippedBody += " ";
          }
        }
        const tokenBody = body.slice(currentToken.start, currentToken.end);
        if (tokenKind === _tokenKind.TokenKind.BLOCK_STRING) {
          strippedBody += (0, _blockString.printBlockString)(currentToken.value, {
            minimize: true
          });
        } else {
          strippedBody += tokenBody;
        }
        wasLastAddedTokenNonPunctuator = isNonPunctuator;
      }
      return strippedBody;
    }
  }
});

// node_modules/graphql/utilities/assertValidName.js
var require_assertValidName = __commonJS({
  "node_modules/graphql/utilities/assertValidName.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.assertValidName = assertValidName;
    exports2.isValidNameError = isValidNameError;
    var _devAssert = require_devAssert();
    var _GraphQLError = require_GraphQLError();
    var _assertName = require_assertName();
    function assertValidName(name) {
      const error = isValidNameError(name);
      if (error) {
        throw error;
      }
      return name;
    }
    function isValidNameError(name) {
      typeof name === "string" || (0, _devAssert.devAssert)(false, "Expected name to be a string.");
      if (name.startsWith("__")) {
        return new _GraphQLError.GraphQLError(
          `Name "${name}" must not begin with "__", which is reserved by GraphQL introspection.`
        );
      }
      try {
        (0, _assertName.assertName)(name);
      } catch (error) {
        return error;
      }
    }
  }
});

// node_modules/graphql/utilities/findBreakingChanges.js
var require_findBreakingChanges = __commonJS({
  "node_modules/graphql/utilities/findBreakingChanges.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    exports2.DangerousChangeType = exports2.BreakingChangeType = void 0;
    exports2.findBreakingChanges = findBreakingChanges;
    exports2.findDangerousChanges = findDangerousChanges;
    var _inspect = require_inspect();
    var _invariant = require_invariant();
    var _keyMap = require_keyMap();
    var _printer = require_printer();
    var _definition = require_definition();
    var _scalars = require_scalars();
    var _astFromValue = require_astFromValue();
    var _sortValueNode = require_sortValueNode();
    var BreakingChangeType;
    exports2.BreakingChangeType = BreakingChangeType;
    (function(BreakingChangeType2) {
      BreakingChangeType2["TYPE_REMOVED"] = "TYPE_REMOVED";
      BreakingChangeType2["TYPE_CHANGED_KIND"] = "TYPE_CHANGED_KIND";
      BreakingChangeType2["TYPE_REMOVED_FROM_UNION"] = "TYPE_REMOVED_FROM_UNION";
      BreakingChangeType2["VALUE_REMOVED_FROM_ENUM"] = "VALUE_REMOVED_FROM_ENUM";
      BreakingChangeType2["REQUIRED_INPUT_FIELD_ADDED"] = "REQUIRED_INPUT_FIELD_ADDED";
      BreakingChangeType2["IMPLEMENTED_INTERFACE_REMOVED"] = "IMPLEMENTED_INTERFACE_REMOVED";
      BreakingChangeType2["FIELD_REMOVED"] = "FIELD_REMOVED";
      BreakingChangeType2["FIELD_CHANGED_KIND"] = "FIELD_CHANGED_KIND";
      BreakingChangeType2["REQUIRED_ARG_ADDED"] = "REQUIRED_ARG_ADDED";
      BreakingChangeType2["ARG_REMOVED"] = "ARG_REMOVED";
      BreakingChangeType2["ARG_CHANGED_KIND"] = "ARG_CHANGED_KIND";
      BreakingChangeType2["DIRECTIVE_REMOVED"] = "DIRECTIVE_REMOVED";
      BreakingChangeType2["DIRECTIVE_ARG_REMOVED"] = "DIRECTIVE_ARG_REMOVED";
      BreakingChangeType2["REQUIRED_DIRECTIVE_ARG_ADDED"] = "REQUIRED_DIRECTIVE_ARG_ADDED";
      BreakingChangeType2["DIRECTIVE_REPEATABLE_REMOVED"] = "DIRECTIVE_REPEATABLE_REMOVED";
      BreakingChangeType2["DIRECTIVE_LOCATION_REMOVED"] = "DIRECTIVE_LOCATION_REMOVED";
    })(
      BreakingChangeType || (exports2.BreakingChangeType = BreakingChangeType = {})
    );
    var DangerousChangeType;
    exports2.DangerousChangeType = DangerousChangeType;
    (function(DangerousChangeType2) {
      DangerousChangeType2["VALUE_ADDED_TO_ENUM"] = "VALUE_ADDED_TO_ENUM";
      DangerousChangeType2["TYPE_ADDED_TO_UNION"] = "TYPE_ADDED_TO_UNION";
      DangerousChangeType2["OPTIONAL_INPUT_FIELD_ADDED"] = "OPTIONAL_INPUT_FIELD_ADDED";
      DangerousChangeType2["OPTIONAL_ARG_ADDED"] = "OPTIONAL_ARG_ADDED";
      DangerousChangeType2["IMPLEMENTED_INTERFACE_ADDED"] = "IMPLEMENTED_INTERFACE_ADDED";
      DangerousChangeType2["ARG_DEFAULT_VALUE_CHANGE"] = "ARG_DEFAULT_VALUE_CHANGE";
    })(
      DangerousChangeType || (exports2.DangerousChangeType = DangerousChangeType = {})
    );
    function findBreakingChanges(oldSchema, newSchema) {
      return findSchemaChanges(oldSchema, newSchema).filter(
        (change) => change.type in BreakingChangeType
      );
    }
    function findDangerousChanges(oldSchema, newSchema) {
      return findSchemaChanges(oldSchema, newSchema).filter(
        (change) => change.type in DangerousChangeType
      );
    }
    function findSchemaChanges(oldSchema, newSchema) {
      return [
        ...findTypeChanges(oldSchema, newSchema),
        ...findDirectiveChanges(oldSchema, newSchema)
      ];
    }
    function findDirectiveChanges(oldSchema, newSchema) {
      const schemaChanges = [];
      const directivesDiff = diff(
        oldSchema.getDirectives(),
        newSchema.getDirectives()
      );
      for (const oldDirective of directivesDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.DIRECTIVE_REMOVED,
          description: `${oldDirective.name} was removed.`
        });
      }
      for (const [oldDirective, newDirective] of directivesDiff.persisted) {
        const argsDiff = diff(oldDirective.args, newDirective.args);
        for (const newArg of argsDiff.added) {
          if ((0, _definition.isRequiredArgument)(newArg)) {
            schemaChanges.push({
              type: BreakingChangeType.REQUIRED_DIRECTIVE_ARG_ADDED,
              description: `A required arg ${newArg.name} on directive ${oldDirective.name} was added.`
            });
          }
        }
        for (const oldArg of argsDiff.removed) {
          schemaChanges.push({
            type: BreakingChangeType.DIRECTIVE_ARG_REMOVED,
            description: `${oldArg.name} was removed from ${oldDirective.name}.`
          });
        }
        if (oldDirective.isRepeatable && !newDirective.isRepeatable) {
          schemaChanges.push({
            type: BreakingChangeType.DIRECTIVE_REPEATABLE_REMOVED,
            description: `Repeatable flag was removed from ${oldDirective.name}.`
          });
        }
        for (const location of oldDirective.locations) {
          if (!newDirective.locations.includes(location)) {
            schemaChanges.push({
              type: BreakingChangeType.DIRECTIVE_LOCATION_REMOVED,
              description: `${location} was removed from ${oldDirective.name}.`
            });
          }
        }
      }
      return schemaChanges;
    }
    function findTypeChanges(oldSchema, newSchema) {
      const schemaChanges = [];
      const typesDiff = diff(
        Object.values(oldSchema.getTypeMap()),
        Object.values(newSchema.getTypeMap())
      );
      for (const oldType of typesDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.TYPE_REMOVED,
          description: (0, _scalars.isSpecifiedScalarType)(oldType) ? `Standard scalar ${oldType.name} was removed because it is not referenced anymore.` : `${oldType.name} was removed.`
        });
      }
      for (const [oldType, newType] of typesDiff.persisted) {
        if ((0, _definition.isEnumType)(oldType) && (0, _definition.isEnumType)(newType)) {
          schemaChanges.push(...findEnumTypeChanges(oldType, newType));
        } else if ((0, _definition.isUnionType)(oldType) && (0, _definition.isUnionType)(newType)) {
          schemaChanges.push(...findUnionTypeChanges(oldType, newType));
        } else if ((0, _definition.isInputObjectType)(oldType) && (0, _definition.isInputObjectType)(newType)) {
          schemaChanges.push(...findInputObjectTypeChanges(oldType, newType));
        } else if ((0, _definition.isObjectType)(oldType) && (0, _definition.isObjectType)(newType)) {
          schemaChanges.push(
            ...findFieldChanges(oldType, newType),
            ...findImplementedInterfacesChanges(oldType, newType)
          );
        } else if ((0, _definition.isInterfaceType)(oldType) && (0, _definition.isInterfaceType)(newType)) {
          schemaChanges.push(
            ...findFieldChanges(oldType, newType),
            ...findImplementedInterfacesChanges(oldType, newType)
          );
        } else if (oldType.constructor !== newType.constructor) {
          schemaChanges.push({
            type: BreakingChangeType.TYPE_CHANGED_KIND,
            description: `${oldType.name} changed from ${typeKindName(oldType)} to ${typeKindName(newType)}.`
          });
        }
      }
      return schemaChanges;
    }
    function findInputObjectTypeChanges(oldType, newType) {
      const schemaChanges = [];
      const fieldsDiff = diff(
        Object.values(oldType.getFields()),
        Object.values(newType.getFields())
      );
      for (const newField of fieldsDiff.added) {
        if ((0, _definition.isRequiredInputField)(newField)) {
          schemaChanges.push({
            type: BreakingChangeType.REQUIRED_INPUT_FIELD_ADDED,
            description: `A required field ${newField.name} on input type ${oldType.name} was added.`
          });
        } else {
          schemaChanges.push({
            type: DangerousChangeType.OPTIONAL_INPUT_FIELD_ADDED,
            description: `An optional field ${newField.name} on input type ${oldType.name} was added.`
          });
        }
      }
      for (const oldField of fieldsDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.FIELD_REMOVED,
          description: `${oldType.name}.${oldField.name} was removed.`
        });
      }
      for (const [oldField, newField] of fieldsDiff.persisted) {
        const isSafe = isChangeSafeForInputObjectFieldOrFieldArg(
          oldField.type,
          newField.type
        );
        if (!isSafe) {
          schemaChanges.push({
            type: BreakingChangeType.FIELD_CHANGED_KIND,
            description: `${oldType.name}.${oldField.name} changed type from ${String(oldField.type)} to ${String(newField.type)}.`
          });
        }
      }
      return schemaChanges;
    }
    function findUnionTypeChanges(oldType, newType) {
      const schemaChanges = [];
      const possibleTypesDiff = diff(oldType.getTypes(), newType.getTypes());
      for (const newPossibleType of possibleTypesDiff.added) {
        schemaChanges.push({
          type: DangerousChangeType.TYPE_ADDED_TO_UNION,
          description: `${newPossibleType.name} was added to union type ${oldType.name}.`
        });
      }
      for (const oldPossibleType of possibleTypesDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.TYPE_REMOVED_FROM_UNION,
          description: `${oldPossibleType.name} was removed from union type ${oldType.name}.`
        });
      }
      return schemaChanges;
    }
    function findEnumTypeChanges(oldType, newType) {
      const schemaChanges = [];
      const valuesDiff = diff(oldType.getValues(), newType.getValues());
      for (const newValue of valuesDiff.added) {
        schemaChanges.push({
          type: DangerousChangeType.VALUE_ADDED_TO_ENUM,
          description: `${newValue.name} was added to enum type ${oldType.name}.`
        });
      }
      for (const oldValue of valuesDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.VALUE_REMOVED_FROM_ENUM,
          description: `${oldValue.name} was removed from enum type ${oldType.name}.`
        });
      }
      return schemaChanges;
    }
    function findImplementedInterfacesChanges(oldType, newType) {
      const schemaChanges = [];
      const interfacesDiff = diff(oldType.getInterfaces(), newType.getInterfaces());
      for (const newInterface of interfacesDiff.added) {
        schemaChanges.push({
          type: DangerousChangeType.IMPLEMENTED_INTERFACE_ADDED,
          description: `${newInterface.name} added to interfaces implemented by ${oldType.name}.`
        });
      }
      for (const oldInterface of interfacesDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.IMPLEMENTED_INTERFACE_REMOVED,
          description: `${oldType.name} no longer implements interface ${oldInterface.name}.`
        });
      }
      return schemaChanges;
    }
    function findFieldChanges(oldType, newType) {
      const schemaChanges = [];
      const fieldsDiff = diff(
        Object.values(oldType.getFields()),
        Object.values(newType.getFields())
      );
      for (const oldField of fieldsDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.FIELD_REMOVED,
          description: `${oldType.name}.${oldField.name} was removed.`
        });
      }
      for (const [oldField, newField] of fieldsDiff.persisted) {
        schemaChanges.push(...findArgChanges(oldType, oldField, newField));
        const isSafe = isChangeSafeForObjectOrInterfaceField(
          oldField.type,
          newField.type
        );
        if (!isSafe) {
          schemaChanges.push({
            type: BreakingChangeType.FIELD_CHANGED_KIND,
            description: `${oldType.name}.${oldField.name} changed type from ${String(oldField.type)} to ${String(newField.type)}.`
          });
        }
      }
      return schemaChanges;
    }
    function findArgChanges(oldType, oldField, newField) {
      const schemaChanges = [];
      const argsDiff = diff(oldField.args, newField.args);
      for (const oldArg of argsDiff.removed) {
        schemaChanges.push({
          type: BreakingChangeType.ARG_REMOVED,
          description: `${oldType.name}.${oldField.name} arg ${oldArg.name} was removed.`
        });
      }
      for (const [oldArg, newArg] of argsDiff.persisted) {
        const isSafe = isChangeSafeForInputObjectFieldOrFieldArg(
          oldArg.type,
          newArg.type
        );
        if (!isSafe) {
          schemaChanges.push({
            type: BreakingChangeType.ARG_CHANGED_KIND,
            description: `${oldType.name}.${oldField.name} arg ${oldArg.name} has changed type from ${String(oldArg.type)} to ${String(newArg.type)}.`
          });
        } else if (oldArg.defaultValue !== void 0) {
          if (newArg.defaultValue === void 0) {
            schemaChanges.push({
              type: DangerousChangeType.ARG_DEFAULT_VALUE_CHANGE,
              description: `${oldType.name}.${oldField.name} arg ${oldArg.name} defaultValue was removed.`
            });
          } else {
            const oldValueStr = stringifyValue(oldArg.defaultValue, oldArg.type);
            const newValueStr = stringifyValue(newArg.defaultValue, newArg.type);
            if (oldValueStr !== newValueStr) {
              schemaChanges.push({
                type: DangerousChangeType.ARG_DEFAULT_VALUE_CHANGE,
                description: `${oldType.name}.${oldField.name} arg ${oldArg.name} has changed defaultValue from ${oldValueStr} to ${newValueStr}.`
              });
            }
          }
        }
      }
      for (const newArg of argsDiff.added) {
        if ((0, _definition.isRequiredArgument)(newArg)) {
          schemaChanges.push({
            type: BreakingChangeType.REQUIRED_ARG_ADDED,
            description: `A required arg ${newArg.name} on ${oldType.name}.${oldField.name} was added.`
          });
        } else {
          schemaChanges.push({
            type: DangerousChangeType.OPTIONAL_ARG_ADDED,
            description: `An optional arg ${newArg.name} on ${oldType.name}.${oldField.name} was added.`
          });
        }
      }
      return schemaChanges;
    }
    function isChangeSafeForObjectOrInterfaceField(oldType, newType) {
      if ((0, _definition.isListType)(oldType)) {
        return (
          // if they're both lists, make sure the underlying types are compatible
          (0, _definition.isListType)(newType) && isChangeSafeForObjectOrInterfaceField(
            oldType.ofType,
            newType.ofType
          ) || // moving from nullable to non-null of the same underlying type is safe
          (0, _definition.isNonNullType)(newType) && isChangeSafeForObjectOrInterfaceField(oldType, newType.ofType)
        );
      }
      if ((0, _definition.isNonNullType)(oldType)) {
        return (0, _definition.isNonNullType)(newType) && isChangeSafeForObjectOrInterfaceField(oldType.ofType, newType.ofType);
      }
      return (
        // if they're both named types, see if their names are equivalent
        (0, _definition.isNamedType)(newType) && oldType.name === newType.name || // moving from nullable to non-null of the same underlying type is safe
        (0, _definition.isNonNullType)(newType) && isChangeSafeForObjectOrInterfaceField(oldType, newType.ofType)
      );
    }
    function isChangeSafeForInputObjectFieldOrFieldArg(oldType, newType) {
      if ((0, _definition.isListType)(oldType)) {
        return (0, _definition.isListType)(newType) && isChangeSafeForInputObjectFieldOrFieldArg(oldType.ofType, newType.ofType);
      }
      if ((0, _definition.isNonNullType)(oldType)) {
        return (
          // if they're both non-null, make sure the underlying types are
          // compatible
          (0, _definition.isNonNullType)(newType) && isChangeSafeForInputObjectFieldOrFieldArg(
            oldType.ofType,
            newType.ofType
          ) || // moving from non-null to nullable of the same underlying type is safe
          !(0, _definition.isNonNullType)(newType) && isChangeSafeForInputObjectFieldOrFieldArg(oldType.ofType, newType)
        );
      }
      return (0, _definition.isNamedType)(newType) && oldType.name === newType.name;
    }
    function typeKindName(type) {
      if ((0, _definition.isScalarType)(type)) {
        return "a Scalar type";
      }
      if ((0, _definition.isObjectType)(type)) {
        return "an Object type";
      }
      if ((0, _definition.isInterfaceType)(type)) {
        return "an Interface type";
      }
      if ((0, _definition.isUnionType)(type)) {
        return "a Union type";
      }
      if ((0, _definition.isEnumType)(type)) {
        return "an Enum type";
      }
      if ((0, _definition.isInputObjectType)(type)) {
        return "an Input type";
      }
      (0, _invariant.invariant)(
        false,
        "Unexpected type: " + (0, _inspect.inspect)(type)
      );
    }
    function stringifyValue(value, type) {
      const ast = (0, _astFromValue.astFromValue)(value, type);
      ast != null || (0, _invariant.invariant)(false);
      return (0, _printer.print)((0, _sortValueNode.sortValueNode)(ast));
    }
    function diff(oldArray, newArray) {
      const added = [];
      const removed = [];
      const persisted = [];
      const oldMap = (0, _keyMap.keyMap)(oldArray, ({ name }) => name);
      const newMap = (0, _keyMap.keyMap)(newArray, ({ name }) => name);
      for (const oldItem of oldArray) {
        const newItem = newMap[oldItem.name];
        if (newItem === void 0) {
          removed.push(oldItem);
        } else {
          persisted.push([oldItem, newItem]);
        }
      }
      for (const newItem of newArray) {
        if (oldMap[newItem.name] === void 0) {
          added.push(newItem);
        }
      }
      return {
        added,
        persisted,
        removed
      };
    }
  }
});

// node_modules/graphql/utilities/index.js
var require_utilities = __commonJS({
  "node_modules/graphql/utilities/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "BreakingChangeType", {
      enumerable: true,
      get: function() {
        return _findBreakingChanges.BreakingChangeType;
      }
    });
    Object.defineProperty(exports2, "DangerousChangeType", {
      enumerable: true,
      get: function() {
        return _findBreakingChanges.DangerousChangeType;
      }
    });
    Object.defineProperty(exports2, "TypeInfo", {
      enumerable: true,
      get: function() {
        return _TypeInfo.TypeInfo;
      }
    });
    Object.defineProperty(exports2, "assertValidName", {
      enumerable: true,
      get: function() {
        return _assertValidName.assertValidName;
      }
    });
    Object.defineProperty(exports2, "astFromValue", {
      enumerable: true,
      get: function() {
        return _astFromValue.astFromValue;
      }
    });
    Object.defineProperty(exports2, "buildASTSchema", {
      enumerable: true,
      get: function() {
        return _buildASTSchema.buildASTSchema;
      }
    });
    Object.defineProperty(exports2, "buildClientSchema", {
      enumerable: true,
      get: function() {
        return _buildClientSchema.buildClientSchema;
      }
    });
    Object.defineProperty(exports2, "buildSchema", {
      enumerable: true,
      get: function() {
        return _buildASTSchema.buildSchema;
      }
    });
    Object.defineProperty(exports2, "coerceInputValue", {
      enumerable: true,
      get: function() {
        return _coerceInputValue.coerceInputValue;
      }
    });
    Object.defineProperty(exports2, "concatAST", {
      enumerable: true,
      get: function() {
        return _concatAST.concatAST;
      }
    });
    Object.defineProperty(exports2, "doTypesOverlap", {
      enumerable: true,
      get: function() {
        return _typeComparators.doTypesOverlap;
      }
    });
    Object.defineProperty(exports2, "extendSchema", {
      enumerable: true,
      get: function() {
        return _extendSchema.extendSchema;
      }
    });
    Object.defineProperty(exports2, "findBreakingChanges", {
      enumerable: true,
      get: function() {
        return _findBreakingChanges.findBreakingChanges;
      }
    });
    Object.defineProperty(exports2, "findDangerousChanges", {
      enumerable: true,
      get: function() {
        return _findBreakingChanges.findDangerousChanges;
      }
    });
    Object.defineProperty(exports2, "getIntrospectionQuery", {
      enumerable: true,
      get: function() {
        return _getIntrospectionQuery.getIntrospectionQuery;
      }
    });
    Object.defineProperty(exports2, "getOperationAST", {
      enumerable: true,
      get: function() {
        return _getOperationAST.getOperationAST;
      }
    });
    Object.defineProperty(exports2, "getOperationRootType", {
      enumerable: true,
      get: function() {
        return _getOperationRootType.getOperationRootType;
      }
    });
    Object.defineProperty(exports2, "introspectionFromSchema", {
      enumerable: true,
      get: function() {
        return _introspectionFromSchema.introspectionFromSchema;
      }
    });
    Object.defineProperty(exports2, "isEqualType", {
      enumerable: true,
      get: function() {
        return _typeComparators.isEqualType;
      }
    });
    Object.defineProperty(exports2, "isTypeSubTypeOf", {
      enumerable: true,
      get: function() {
        return _typeComparators.isTypeSubTypeOf;
      }
    });
    Object.defineProperty(exports2, "isValidNameError", {
      enumerable: true,
      get: function() {
        return _assertValidName.isValidNameError;
      }
    });
    Object.defineProperty(exports2, "lexicographicSortSchema", {
      enumerable: true,
      get: function() {
        return _lexicographicSortSchema.lexicographicSortSchema;
      }
    });
    Object.defineProperty(exports2, "printIntrospectionSchema", {
      enumerable: true,
      get: function() {
        return _printSchema.printIntrospectionSchema;
      }
    });
    Object.defineProperty(exports2, "printSchema", {
      enumerable: true,
      get: function() {
        return _printSchema.printSchema;
      }
    });
    Object.defineProperty(exports2, "printType", {
      enumerable: true,
      get: function() {
        return _printSchema.printType;
      }
    });
    Object.defineProperty(exports2, "separateOperations", {
      enumerable: true,
      get: function() {
        return _separateOperations.separateOperations;
      }
    });
    Object.defineProperty(exports2, "stripIgnoredCharacters", {
      enumerable: true,
      get: function() {
        return _stripIgnoredCharacters.stripIgnoredCharacters;
      }
    });
    Object.defineProperty(exports2, "typeFromAST", {
      enumerable: true,
      get: function() {
        return _typeFromAST.typeFromAST;
      }
    });
    Object.defineProperty(exports2, "valueFromAST", {
      enumerable: true,
      get: function() {
        return _valueFromAST.valueFromAST;
      }
    });
    Object.defineProperty(exports2, "valueFromASTUntyped", {
      enumerable: true,
      get: function() {
        return _valueFromASTUntyped.valueFromASTUntyped;
      }
    });
    Object.defineProperty(exports2, "visitWithTypeInfo", {
      enumerable: true,
      get: function() {
        return _TypeInfo.visitWithTypeInfo;
      }
    });
    var _getIntrospectionQuery = require_getIntrospectionQuery();
    var _getOperationAST = require_getOperationAST();
    var _getOperationRootType = require_getOperationRootType();
    var _introspectionFromSchema = require_introspectionFromSchema();
    var _buildClientSchema = require_buildClientSchema();
    var _buildASTSchema = require_buildASTSchema();
    var _extendSchema = require_extendSchema();
    var _lexicographicSortSchema = require_lexicographicSortSchema();
    var _printSchema = require_printSchema();
    var _typeFromAST = require_typeFromAST();
    var _valueFromAST = require_valueFromAST();
    var _valueFromASTUntyped = require_valueFromASTUntyped();
    var _astFromValue = require_astFromValue();
    var _TypeInfo = require_TypeInfo();
    var _coerceInputValue = require_coerceInputValue();
    var _concatAST = require_concatAST();
    var _separateOperations = require_separateOperations();
    var _stripIgnoredCharacters = require_stripIgnoredCharacters();
    var _typeComparators = require_typeComparators();
    var _assertValidName = require_assertValidName();
    var _findBreakingChanges = require_findBreakingChanges();
  }
});

// node_modules/graphql/index.js
var require_graphql2 = __commonJS({
  "node_modules/graphql/index.js"(exports2) {
    "use strict";
    Object.defineProperty(exports2, "__esModule", {
      value: true
    });
    Object.defineProperty(exports2, "BREAK", {
      enumerable: true,
      get: function() {
        return _index2.BREAK;
      }
    });
    Object.defineProperty(exports2, "BreakingChangeType", {
      enumerable: true,
      get: function() {
        return _index6.BreakingChangeType;
      }
    });
    Object.defineProperty(exports2, "DEFAULT_DEPRECATION_REASON", {
      enumerable: true,
      get: function() {
        return _index.DEFAULT_DEPRECATION_REASON;
      }
    });
    Object.defineProperty(exports2, "DangerousChangeType", {
      enumerable: true,
      get: function() {
        return _index6.DangerousChangeType;
      }
    });
    Object.defineProperty(exports2, "DirectiveLocation", {
      enumerable: true,
      get: function() {
        return _index2.DirectiveLocation;
      }
    });
    Object.defineProperty(exports2, "ExecutableDefinitionsRule", {
      enumerable: true,
      get: function() {
        return _index4.ExecutableDefinitionsRule;
      }
    });
    Object.defineProperty(exports2, "FieldsOnCorrectTypeRule", {
      enumerable: true,
      get: function() {
        return _index4.FieldsOnCorrectTypeRule;
      }
    });
    Object.defineProperty(exports2, "FragmentsOnCompositeTypesRule", {
      enumerable: true,
      get: function() {
        return _index4.FragmentsOnCompositeTypesRule;
      }
    });
    Object.defineProperty(exports2, "GRAPHQL_MAX_INT", {
      enumerable: true,
      get: function() {
        return _index.GRAPHQL_MAX_INT;
      }
    });
    Object.defineProperty(exports2, "GRAPHQL_MIN_INT", {
      enumerable: true,
      get: function() {
        return _index.GRAPHQL_MIN_INT;
      }
    });
    Object.defineProperty(exports2, "GraphQLBoolean", {
      enumerable: true,
      get: function() {
        return _index.GraphQLBoolean;
      }
    });
    Object.defineProperty(exports2, "GraphQLDeprecatedDirective", {
      enumerable: true,
      get: function() {
        return _index.GraphQLDeprecatedDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLDirective", {
      enumerable: true,
      get: function() {
        return _index.GraphQLDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLEnumType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLEnumType;
      }
    });
    Object.defineProperty(exports2, "GraphQLError", {
      enumerable: true,
      get: function() {
        return _index5.GraphQLError;
      }
    });
    Object.defineProperty(exports2, "GraphQLFloat", {
      enumerable: true,
      get: function() {
        return _index.GraphQLFloat;
      }
    });
    Object.defineProperty(exports2, "GraphQLID", {
      enumerable: true,
      get: function() {
        return _index.GraphQLID;
      }
    });
    Object.defineProperty(exports2, "GraphQLIncludeDirective", {
      enumerable: true,
      get: function() {
        return _index.GraphQLIncludeDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLInputObjectType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLInputObjectType;
      }
    });
    Object.defineProperty(exports2, "GraphQLInt", {
      enumerable: true,
      get: function() {
        return _index.GraphQLInt;
      }
    });
    Object.defineProperty(exports2, "GraphQLInterfaceType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLInterfaceType;
      }
    });
    Object.defineProperty(exports2, "GraphQLList", {
      enumerable: true,
      get: function() {
        return _index.GraphQLList;
      }
    });
    Object.defineProperty(exports2, "GraphQLNonNull", {
      enumerable: true,
      get: function() {
        return _index.GraphQLNonNull;
      }
    });
    Object.defineProperty(exports2, "GraphQLObjectType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLObjectType;
      }
    });
    Object.defineProperty(exports2, "GraphQLScalarType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLScalarType;
      }
    });
    Object.defineProperty(exports2, "GraphQLSchema", {
      enumerable: true,
      get: function() {
        return _index.GraphQLSchema;
      }
    });
    Object.defineProperty(exports2, "GraphQLSkipDirective", {
      enumerable: true,
      get: function() {
        return _index.GraphQLSkipDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLSpecifiedByDirective", {
      enumerable: true,
      get: function() {
        return _index.GraphQLSpecifiedByDirective;
      }
    });
    Object.defineProperty(exports2, "GraphQLString", {
      enumerable: true,
      get: function() {
        return _index.GraphQLString;
      }
    });
    Object.defineProperty(exports2, "GraphQLUnionType", {
      enumerable: true,
      get: function() {
        return _index.GraphQLUnionType;
      }
    });
    Object.defineProperty(exports2, "Kind", {
      enumerable: true,
      get: function() {
        return _index2.Kind;
      }
    });
    Object.defineProperty(exports2, "KnownArgumentNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.KnownArgumentNamesRule;
      }
    });
    Object.defineProperty(exports2, "KnownDirectivesRule", {
      enumerable: true,
      get: function() {
        return _index4.KnownDirectivesRule;
      }
    });
    Object.defineProperty(exports2, "KnownFragmentNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.KnownFragmentNamesRule;
      }
    });
    Object.defineProperty(exports2, "KnownTypeNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.KnownTypeNamesRule;
      }
    });
    Object.defineProperty(exports2, "Lexer", {
      enumerable: true,
      get: function() {
        return _index2.Lexer;
      }
    });
    Object.defineProperty(exports2, "Location", {
      enumerable: true,
      get: function() {
        return _index2.Location;
      }
    });
    Object.defineProperty(exports2, "LoneAnonymousOperationRule", {
      enumerable: true,
      get: function() {
        return _index4.LoneAnonymousOperationRule;
      }
    });
    Object.defineProperty(exports2, "LoneSchemaDefinitionRule", {
      enumerable: true,
      get: function() {
        return _index4.LoneSchemaDefinitionRule;
      }
    });
    Object.defineProperty(exports2, "NoDeprecatedCustomRule", {
      enumerable: true,
      get: function() {
        return _index4.NoDeprecatedCustomRule;
      }
    });
    Object.defineProperty(exports2, "NoFragmentCyclesRule", {
      enumerable: true,
      get: function() {
        return _index4.NoFragmentCyclesRule;
      }
    });
    Object.defineProperty(exports2, "NoSchemaIntrospectionCustomRule", {
      enumerable: true,
      get: function() {
        return _index4.NoSchemaIntrospectionCustomRule;
      }
    });
    Object.defineProperty(exports2, "NoUndefinedVariablesRule", {
      enumerable: true,
      get: function() {
        return _index4.NoUndefinedVariablesRule;
      }
    });
    Object.defineProperty(exports2, "NoUnusedFragmentsRule", {
      enumerable: true,
      get: function() {
        return _index4.NoUnusedFragmentsRule;
      }
    });
    Object.defineProperty(exports2, "NoUnusedVariablesRule", {
      enumerable: true,
      get: function() {
        return _index4.NoUnusedVariablesRule;
      }
    });
    Object.defineProperty(exports2, "OperationTypeNode", {
      enumerable: true,
      get: function() {
        return _index2.OperationTypeNode;
      }
    });
    Object.defineProperty(exports2, "OverlappingFieldsCanBeMergedRule", {
      enumerable: true,
      get: function() {
        return _index4.OverlappingFieldsCanBeMergedRule;
      }
    });
    Object.defineProperty(exports2, "PossibleFragmentSpreadsRule", {
      enumerable: true,
      get: function() {
        return _index4.PossibleFragmentSpreadsRule;
      }
    });
    Object.defineProperty(exports2, "PossibleTypeExtensionsRule", {
      enumerable: true,
      get: function() {
        return _index4.PossibleTypeExtensionsRule;
      }
    });
    Object.defineProperty(exports2, "ProvidedRequiredArgumentsRule", {
      enumerable: true,
      get: function() {
        return _index4.ProvidedRequiredArgumentsRule;
      }
    });
    Object.defineProperty(exports2, "ScalarLeafsRule", {
      enumerable: true,
      get: function() {
        return _index4.ScalarLeafsRule;
      }
    });
    Object.defineProperty(exports2, "SchemaMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _index.SchemaMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "SingleFieldSubscriptionsRule", {
      enumerable: true,
      get: function() {
        return _index4.SingleFieldSubscriptionsRule;
      }
    });
    Object.defineProperty(exports2, "Source", {
      enumerable: true,
      get: function() {
        return _index2.Source;
      }
    });
    Object.defineProperty(exports2, "Token", {
      enumerable: true,
      get: function() {
        return _index2.Token;
      }
    });
    Object.defineProperty(exports2, "TokenKind", {
      enumerable: true,
      get: function() {
        return _index2.TokenKind;
      }
    });
    Object.defineProperty(exports2, "TypeInfo", {
      enumerable: true,
      get: function() {
        return _index6.TypeInfo;
      }
    });
    Object.defineProperty(exports2, "TypeKind", {
      enumerable: true,
      get: function() {
        return _index.TypeKind;
      }
    });
    Object.defineProperty(exports2, "TypeMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _index.TypeMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "TypeNameMetaFieldDef", {
      enumerable: true,
      get: function() {
        return _index.TypeNameMetaFieldDef;
      }
    });
    Object.defineProperty(exports2, "UniqueArgumentDefinitionNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueArgumentDefinitionNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueArgumentNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueArgumentNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueDirectiveNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueDirectiveNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueDirectivesPerLocationRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueDirectivesPerLocationRule;
      }
    });
    Object.defineProperty(exports2, "UniqueEnumValueNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueEnumValueNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueFieldDefinitionNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueFieldDefinitionNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueFragmentNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueFragmentNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueInputFieldNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueInputFieldNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueOperationNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueOperationNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueOperationTypesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueOperationTypesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueTypeNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueTypeNamesRule;
      }
    });
    Object.defineProperty(exports2, "UniqueVariableNamesRule", {
      enumerable: true,
      get: function() {
        return _index4.UniqueVariableNamesRule;
      }
    });
    Object.defineProperty(exports2, "ValidationContext", {
      enumerable: true,
      get: function() {
        return _index4.ValidationContext;
      }
    });
    Object.defineProperty(exports2, "ValuesOfCorrectTypeRule", {
      enumerable: true,
      get: function() {
        return _index4.ValuesOfCorrectTypeRule;
      }
    });
    Object.defineProperty(exports2, "VariablesAreInputTypesRule", {
      enumerable: true,
      get: function() {
        return _index4.VariablesAreInputTypesRule;
      }
    });
    Object.defineProperty(exports2, "VariablesInAllowedPositionRule", {
      enumerable: true,
      get: function() {
        return _index4.VariablesInAllowedPositionRule;
      }
    });
    Object.defineProperty(exports2, "__Directive", {
      enumerable: true,
      get: function() {
        return _index.__Directive;
      }
    });
    Object.defineProperty(exports2, "__DirectiveLocation", {
      enumerable: true,
      get: function() {
        return _index.__DirectiveLocation;
      }
    });
    Object.defineProperty(exports2, "__EnumValue", {
      enumerable: true,
      get: function() {
        return _index.__EnumValue;
      }
    });
    Object.defineProperty(exports2, "__Field", {
      enumerable: true,
      get: function() {
        return _index.__Field;
      }
    });
    Object.defineProperty(exports2, "__InputValue", {
      enumerable: true,
      get: function() {
        return _index.__InputValue;
      }
    });
    Object.defineProperty(exports2, "__Schema", {
      enumerable: true,
      get: function() {
        return _index.__Schema;
      }
    });
    Object.defineProperty(exports2, "__Type", {
      enumerable: true,
      get: function() {
        return _index.__Type;
      }
    });
    Object.defineProperty(exports2, "__TypeKind", {
      enumerable: true,
      get: function() {
        return _index.__TypeKind;
      }
    });
    Object.defineProperty(exports2, "assertAbstractType", {
      enumerable: true,
      get: function() {
        return _index.assertAbstractType;
      }
    });
    Object.defineProperty(exports2, "assertCompositeType", {
      enumerable: true,
      get: function() {
        return _index.assertCompositeType;
      }
    });
    Object.defineProperty(exports2, "assertDirective", {
      enumerable: true,
      get: function() {
        return _index.assertDirective;
      }
    });
    Object.defineProperty(exports2, "assertEnumType", {
      enumerable: true,
      get: function() {
        return _index.assertEnumType;
      }
    });
    Object.defineProperty(exports2, "assertEnumValueName", {
      enumerable: true,
      get: function() {
        return _index.assertEnumValueName;
      }
    });
    Object.defineProperty(exports2, "assertInputObjectType", {
      enumerable: true,
      get: function() {
        return _index.assertInputObjectType;
      }
    });
    Object.defineProperty(exports2, "assertInputType", {
      enumerable: true,
      get: function() {
        return _index.assertInputType;
      }
    });
    Object.defineProperty(exports2, "assertInterfaceType", {
      enumerable: true,
      get: function() {
        return _index.assertInterfaceType;
      }
    });
    Object.defineProperty(exports2, "assertLeafType", {
      enumerable: true,
      get: function() {
        return _index.assertLeafType;
      }
    });
    Object.defineProperty(exports2, "assertListType", {
      enumerable: true,
      get: function() {
        return _index.assertListType;
      }
    });
    Object.defineProperty(exports2, "assertName", {
      enumerable: true,
      get: function() {
        return _index.assertName;
      }
    });
    Object.defineProperty(exports2, "assertNamedType", {
      enumerable: true,
      get: function() {
        return _index.assertNamedType;
      }
    });
    Object.defineProperty(exports2, "assertNonNullType", {
      enumerable: true,
      get: function() {
        return _index.assertNonNullType;
      }
    });
    Object.defineProperty(exports2, "assertNullableType", {
      enumerable: true,
      get: function() {
        return _index.assertNullableType;
      }
    });
    Object.defineProperty(exports2, "assertObjectType", {
      enumerable: true,
      get: function() {
        return _index.assertObjectType;
      }
    });
    Object.defineProperty(exports2, "assertOutputType", {
      enumerable: true,
      get: function() {
        return _index.assertOutputType;
      }
    });
    Object.defineProperty(exports2, "assertScalarType", {
      enumerable: true,
      get: function() {
        return _index.assertScalarType;
      }
    });
    Object.defineProperty(exports2, "assertSchema", {
      enumerable: true,
      get: function() {
        return _index.assertSchema;
      }
    });
    Object.defineProperty(exports2, "assertType", {
      enumerable: true,
      get: function() {
        return _index.assertType;
      }
    });
    Object.defineProperty(exports2, "assertUnionType", {
      enumerable: true,
      get: function() {
        return _index.assertUnionType;
      }
    });
    Object.defineProperty(exports2, "assertValidName", {
      enumerable: true,
      get: function() {
        return _index6.assertValidName;
      }
    });
    Object.defineProperty(exports2, "assertValidSchema", {
      enumerable: true,
      get: function() {
        return _index.assertValidSchema;
      }
    });
    Object.defineProperty(exports2, "assertWrappingType", {
      enumerable: true,
      get: function() {
        return _index.assertWrappingType;
      }
    });
    Object.defineProperty(exports2, "astFromValue", {
      enumerable: true,
      get: function() {
        return _index6.astFromValue;
      }
    });
    Object.defineProperty(exports2, "buildASTSchema", {
      enumerable: true,
      get: function() {
        return _index6.buildASTSchema;
      }
    });
    Object.defineProperty(exports2, "buildClientSchema", {
      enumerable: true,
      get: function() {
        return _index6.buildClientSchema;
      }
    });
    Object.defineProperty(exports2, "buildSchema", {
      enumerable: true,
      get: function() {
        return _index6.buildSchema;
      }
    });
    Object.defineProperty(exports2, "coerceInputValue", {
      enumerable: true,
      get: function() {
        return _index6.coerceInputValue;
      }
    });
    Object.defineProperty(exports2, "concatAST", {
      enumerable: true,
      get: function() {
        return _index6.concatAST;
      }
    });
    Object.defineProperty(exports2, "createSourceEventStream", {
      enumerable: true,
      get: function() {
        return _index3.createSourceEventStream;
      }
    });
    Object.defineProperty(exports2, "defaultFieldResolver", {
      enumerable: true,
      get: function() {
        return _index3.defaultFieldResolver;
      }
    });
    Object.defineProperty(exports2, "defaultTypeResolver", {
      enumerable: true,
      get: function() {
        return _index3.defaultTypeResolver;
      }
    });
    Object.defineProperty(exports2, "doTypesOverlap", {
      enumerable: true,
      get: function() {
        return _index6.doTypesOverlap;
      }
    });
    Object.defineProperty(exports2, "execute", {
      enumerable: true,
      get: function() {
        return _index3.execute;
      }
    });
    Object.defineProperty(exports2, "executeSync", {
      enumerable: true,
      get: function() {
        return _index3.executeSync;
      }
    });
    Object.defineProperty(exports2, "extendSchema", {
      enumerable: true,
      get: function() {
        return _index6.extendSchema;
      }
    });
    Object.defineProperty(exports2, "findBreakingChanges", {
      enumerable: true,
      get: function() {
        return _index6.findBreakingChanges;
      }
    });
    Object.defineProperty(exports2, "findDangerousChanges", {
      enumerable: true,
      get: function() {
        return _index6.findDangerousChanges;
      }
    });
    Object.defineProperty(exports2, "formatError", {
      enumerable: true,
      get: function() {
        return _index5.formatError;
      }
    });
    Object.defineProperty(exports2, "getArgumentValues", {
      enumerable: true,
      get: function() {
        return _index3.getArgumentValues;
      }
    });
    Object.defineProperty(exports2, "getDirectiveValues", {
      enumerable: true,
      get: function() {
        return _index3.getDirectiveValues;
      }
    });
    Object.defineProperty(exports2, "getEnterLeaveForKind", {
      enumerable: true,
      get: function() {
        return _index2.getEnterLeaveForKind;
      }
    });
    Object.defineProperty(exports2, "getIntrospectionQuery", {
      enumerable: true,
      get: function() {
        return _index6.getIntrospectionQuery;
      }
    });
    Object.defineProperty(exports2, "getLocation", {
      enumerable: true,
      get: function() {
        return _index2.getLocation;
      }
    });
    Object.defineProperty(exports2, "getNamedType", {
      enumerable: true,
      get: function() {
        return _index.getNamedType;
      }
    });
    Object.defineProperty(exports2, "getNullableType", {
      enumerable: true,
      get: function() {
        return _index.getNullableType;
      }
    });
    Object.defineProperty(exports2, "getOperationAST", {
      enumerable: true,
      get: function() {
        return _index6.getOperationAST;
      }
    });
    Object.defineProperty(exports2, "getOperationRootType", {
      enumerable: true,
      get: function() {
        return _index6.getOperationRootType;
      }
    });
    Object.defineProperty(exports2, "getVariableValues", {
      enumerable: true,
      get: function() {
        return _index3.getVariableValues;
      }
    });
    Object.defineProperty(exports2, "getVisitFn", {
      enumerable: true,
      get: function() {
        return _index2.getVisitFn;
      }
    });
    Object.defineProperty(exports2, "graphql", {
      enumerable: true,
      get: function() {
        return _graphql.graphql;
      }
    });
    Object.defineProperty(exports2, "graphqlSync", {
      enumerable: true,
      get: function() {
        return _graphql.graphqlSync;
      }
    });
    Object.defineProperty(exports2, "introspectionFromSchema", {
      enumerable: true,
      get: function() {
        return _index6.introspectionFromSchema;
      }
    });
    Object.defineProperty(exports2, "introspectionTypes", {
      enumerable: true,
      get: function() {
        return _index.introspectionTypes;
      }
    });
    Object.defineProperty(exports2, "isAbstractType", {
      enumerable: true,
      get: function() {
        return _index.isAbstractType;
      }
    });
    Object.defineProperty(exports2, "isCompositeType", {
      enumerable: true,
      get: function() {
        return _index.isCompositeType;
      }
    });
    Object.defineProperty(exports2, "isConstValueNode", {
      enumerable: true,
      get: function() {
        return _index2.isConstValueNode;
      }
    });
    Object.defineProperty(exports2, "isDefinitionNode", {
      enumerable: true,
      get: function() {
        return _index2.isDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isDirective", {
      enumerable: true,
      get: function() {
        return _index.isDirective;
      }
    });
    Object.defineProperty(exports2, "isEnumType", {
      enumerable: true,
      get: function() {
        return _index.isEnumType;
      }
    });
    Object.defineProperty(exports2, "isEqualType", {
      enumerable: true,
      get: function() {
        return _index6.isEqualType;
      }
    });
    Object.defineProperty(exports2, "isExecutableDefinitionNode", {
      enumerable: true,
      get: function() {
        return _index2.isExecutableDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isInputObjectType", {
      enumerable: true,
      get: function() {
        return _index.isInputObjectType;
      }
    });
    Object.defineProperty(exports2, "isInputType", {
      enumerable: true,
      get: function() {
        return _index.isInputType;
      }
    });
    Object.defineProperty(exports2, "isInterfaceType", {
      enumerable: true,
      get: function() {
        return _index.isInterfaceType;
      }
    });
    Object.defineProperty(exports2, "isIntrospectionType", {
      enumerable: true,
      get: function() {
        return _index.isIntrospectionType;
      }
    });
    Object.defineProperty(exports2, "isLeafType", {
      enumerable: true,
      get: function() {
        return _index.isLeafType;
      }
    });
    Object.defineProperty(exports2, "isListType", {
      enumerable: true,
      get: function() {
        return _index.isListType;
      }
    });
    Object.defineProperty(exports2, "isNamedType", {
      enumerable: true,
      get: function() {
        return _index.isNamedType;
      }
    });
    Object.defineProperty(exports2, "isNonNullType", {
      enumerable: true,
      get: function() {
        return _index.isNonNullType;
      }
    });
    Object.defineProperty(exports2, "isNullableType", {
      enumerable: true,
      get: function() {
        return _index.isNullableType;
      }
    });
    Object.defineProperty(exports2, "isObjectType", {
      enumerable: true,
      get: function() {
        return _index.isObjectType;
      }
    });
    Object.defineProperty(exports2, "isOutputType", {
      enumerable: true,
      get: function() {
        return _index.isOutputType;
      }
    });
    Object.defineProperty(exports2, "isRequiredArgument", {
      enumerable: true,
      get: function() {
        return _index.isRequiredArgument;
      }
    });
    Object.defineProperty(exports2, "isRequiredInputField", {
      enumerable: true,
      get: function() {
        return _index.isRequiredInputField;
      }
    });
    Object.defineProperty(exports2, "isScalarType", {
      enumerable: true,
      get: function() {
        return _index.isScalarType;
      }
    });
    Object.defineProperty(exports2, "isSchema", {
      enumerable: true,
      get: function() {
        return _index.isSchema;
      }
    });
    Object.defineProperty(exports2, "isSelectionNode", {
      enumerable: true,
      get: function() {
        return _index2.isSelectionNode;
      }
    });
    Object.defineProperty(exports2, "isSpecifiedDirective", {
      enumerable: true,
      get: function() {
        return _index.isSpecifiedDirective;
      }
    });
    Object.defineProperty(exports2, "isSpecifiedScalarType", {
      enumerable: true,
      get: function() {
        return _index.isSpecifiedScalarType;
      }
    });
    Object.defineProperty(exports2, "isType", {
      enumerable: true,
      get: function() {
        return _index.isType;
      }
    });
    Object.defineProperty(exports2, "isTypeDefinitionNode", {
      enumerable: true,
      get: function() {
        return _index2.isTypeDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeExtensionNode", {
      enumerable: true,
      get: function() {
        return _index2.isTypeExtensionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeNode", {
      enumerable: true,
      get: function() {
        return _index2.isTypeNode;
      }
    });
    Object.defineProperty(exports2, "isTypeSubTypeOf", {
      enumerable: true,
      get: function() {
        return _index6.isTypeSubTypeOf;
      }
    });
    Object.defineProperty(exports2, "isTypeSystemDefinitionNode", {
      enumerable: true,
      get: function() {
        return _index2.isTypeSystemDefinitionNode;
      }
    });
    Object.defineProperty(exports2, "isTypeSystemExtensionNode", {
      enumerable: true,
      get: function() {
        return _index2.isTypeSystemExtensionNode;
      }
    });
    Object.defineProperty(exports2, "isUnionType", {
      enumerable: true,
      get: function() {
        return _index.isUnionType;
      }
    });
    Object.defineProperty(exports2, "isValidNameError", {
      enumerable: true,
      get: function() {
        return _index6.isValidNameError;
      }
    });
    Object.defineProperty(exports2, "isValueNode", {
      enumerable: true,
      get: function() {
        return _index2.isValueNode;
      }
    });
    Object.defineProperty(exports2, "isWrappingType", {
      enumerable: true,
      get: function() {
        return _index.isWrappingType;
      }
    });
    Object.defineProperty(exports2, "lexicographicSortSchema", {
      enumerable: true,
      get: function() {
        return _index6.lexicographicSortSchema;
      }
    });
    Object.defineProperty(exports2, "locatedError", {
      enumerable: true,
      get: function() {
        return _index5.locatedError;
      }
    });
    Object.defineProperty(exports2, "parse", {
      enumerable: true,
      get: function() {
        return _index2.parse;
      }
    });
    Object.defineProperty(exports2, "parseConstValue", {
      enumerable: true,
      get: function() {
        return _index2.parseConstValue;
      }
    });
    Object.defineProperty(exports2, "parseType", {
      enumerable: true,
      get: function() {
        return _index2.parseType;
      }
    });
    Object.defineProperty(exports2, "parseValue", {
      enumerable: true,
      get: function() {
        return _index2.parseValue;
      }
    });
    Object.defineProperty(exports2, "print", {
      enumerable: true,
      get: function() {
        return _index2.print;
      }
    });
    Object.defineProperty(exports2, "printError", {
      enumerable: true,
      get: function() {
        return _index5.printError;
      }
    });
    Object.defineProperty(exports2, "printIntrospectionSchema", {
      enumerable: true,
      get: function() {
        return _index6.printIntrospectionSchema;
      }
    });
    Object.defineProperty(exports2, "printLocation", {
      enumerable: true,
      get: function() {
        return _index2.printLocation;
      }
    });
    Object.defineProperty(exports2, "printSchema", {
      enumerable: true,
      get: function() {
        return _index6.printSchema;
      }
    });
    Object.defineProperty(exports2, "printSourceLocation", {
      enumerable: true,
      get: function() {
        return _index2.printSourceLocation;
      }
    });
    Object.defineProperty(exports2, "printType", {
      enumerable: true,
      get: function() {
        return _index6.printType;
      }
    });
    Object.defineProperty(exports2, "resolveObjMapThunk", {
      enumerable: true,
      get: function() {
        return _index.resolveObjMapThunk;
      }
    });
    Object.defineProperty(exports2, "resolveReadonlyArrayThunk", {
      enumerable: true,
      get: function() {
        return _index.resolveReadonlyArrayThunk;
      }
    });
    Object.defineProperty(exports2, "responsePathAsArray", {
      enumerable: true,
      get: function() {
        return _index3.responsePathAsArray;
      }
    });
    Object.defineProperty(exports2, "separateOperations", {
      enumerable: true,
      get: function() {
        return _index6.separateOperations;
      }
    });
    Object.defineProperty(exports2, "specifiedDirectives", {
      enumerable: true,
      get: function() {
        return _index.specifiedDirectives;
      }
    });
    Object.defineProperty(exports2, "specifiedRules", {
      enumerable: true,
      get: function() {
        return _index4.specifiedRules;
      }
    });
    Object.defineProperty(exports2, "specifiedScalarTypes", {
      enumerable: true,
      get: function() {
        return _index.specifiedScalarTypes;
      }
    });
    Object.defineProperty(exports2, "stripIgnoredCharacters", {
      enumerable: true,
      get: function() {
        return _index6.stripIgnoredCharacters;
      }
    });
    Object.defineProperty(exports2, "subscribe", {
      enumerable: true,
      get: function() {
        return _index3.subscribe;
      }
    });
    Object.defineProperty(exports2, "syntaxError", {
      enumerable: true,
      get: function() {
        return _index5.syntaxError;
      }
    });
    Object.defineProperty(exports2, "typeFromAST", {
      enumerable: true,
      get: function() {
        return _index6.typeFromAST;
      }
    });
    Object.defineProperty(exports2, "validate", {
      enumerable: true,
      get: function() {
        return _index4.validate;
      }
    });
    Object.defineProperty(exports2, "validateSchema", {
      enumerable: true,
      get: function() {
        return _index.validateSchema;
      }
    });
    Object.defineProperty(exports2, "valueFromAST", {
      enumerable: true,
      get: function() {
        return _index6.valueFromAST;
      }
    });
    Object.defineProperty(exports2, "valueFromASTUntyped", {
      enumerable: true,
      get: function() {
        return _index6.valueFromASTUntyped;
      }
    });
    Object.defineProperty(exports2, "version", {
      enumerable: true,
      get: function() {
        return _version.version;
      }
    });
    Object.defineProperty(exports2, "versionInfo", {
      enumerable: true,
      get: function() {
        return _version.versionInfo;
      }
    });
    Object.defineProperty(exports2, "visit", {
      enumerable: true,
      get: function() {
        return _index2.visit;
      }
    });
    Object.defineProperty(exports2, "visitInParallel", {
      enumerable: true,
      get: function() {
        return _index2.visitInParallel;
      }
    });
    Object.defineProperty(exports2, "visitWithTypeInfo", {
      enumerable: true,
      get: function() {
        return _index6.visitWithTypeInfo;
      }
    });
    var _version = require_version();
    var _graphql = require_graphql();
    var _index = require_type();
    var _index2 = require_language();
    var _index3 = require_execution();
    var _index4 = require_validation();
    var _index5 = require_error();
    var _index6 = require_utilities();
  }
});

// index.ts
var import_http = require("http");
var import_stream2 = require("stream");
var import_kv = __toESM(require_src2());
var import_storage_memory = __toESM(require_src3());
var import_graphql = __toESM(require_graphql2());
var LogLevel = /* @__PURE__ */ ((LogLevel2) => {
  LogLevel2["Debug"] = "debug";
  LogLevel2["Error"] = "error";
  LogLevel2["Info"] = "info";
  LogLevel2["Warn"] = "warn";
  return LogLevel2;
})(LogLevel || {});
var PORT = 0;
var HOST = "127.0.0.1";
var DUMMY_HOST = "https://grafbase-cli";
var MIME_PROPERTY_SEPARATOR = ";";
var CONSOLE_LOG = Symbol();
var FETCH = Symbol();
var LOG_ENTRIES = Symbol();
var FETCH_REQUESTS = Symbol();
globalThis[CONSOLE_LOG] = console.log;
globalThis[FETCH] = globalThis.fetch;
globalThis[LOG_ENTRIES] = [];
globalThis[FETCH_REQUESTS] = [];
var udf = async (_parent, _args, context, _info) => {
  await context.kv.put("test", "1");
  console.log(await context.kv.get("test"));
  await fetch("https://example.com").then((response) => response.text());
  return { hello: "world" };
};
var server = (0, import_http.createServer)((request, response) => {
  router(
    new Request(`${DUMMY_HOST}${request.url}`, {
      method: request.method,
      // the cast here is likely required because of node fetch still being experimental
      headers: request.headers,
      body: import_stream2.Readable.toWeb(request),
      // @ts-expect-error https://github.com/node-fetch/node-fetch/issues/1769
      duplex: "half" /* Half */
    })
  ).then((udfResponse) => {
    udfResponse.headers.forEach((value, key) => response.setHeader(key, value));
    response.statusMessage = udfResponse.statusText;
    response.statusCode = udfResponse.status;
    import_stream2.Readable.fromWeb(udfResponse.body).on("data" /* Data */, (chunk) => response.write(chunk)).on("end" /* End */, () => response.end());
  });
});
server.listen(PORT, HOST, () => {
  const port = server.address().port;
  globalThis[CONSOLE_LOG](port);
});
var arrayBufferToBase64 = (buffer) => {
  let binaryString = "";
  for (const byte of new Uint8Array(buffer)) {
    binaryString += String.fromCharCode(byte);
  }
  return btoa(binaryString);
};
for (const level of Object.values(LogLevel)) {
  globalThis.console[level] = (...message) => {
    globalThis[LOG_ENTRIES].push({
      loggedAt: Date.now(),
      level,
      message: Array.from(message).map((message2) => JSON.stringify(message2)).join(" ")
    });
  };
}
globalThis.console.log = globalThis.console.info;
globalThis.fetch = async (input, init) => {
  const request = new Request(input, init);
  const startTime = Date.now();
  const response = await globalThis[FETCH](request);
  const endTime = Date.now();
  const contentType = response.headers.get("content-type" /* ContentType */)?.split(MIME_PROPERTY_SEPARATOR)[0].trim();
  let body = null;
  switch (contentType) {
    case "application/json" /* ApplicationJson */:
    case "text/plain" /* TextPlain */:
    case "text/html" /* TextHtml */:
      body = await response.clone().text();
      break;
  }
  const fetchRequest = {
    loggedAt: Date.now(),
    url: request.url,
    method: request.method,
    statusCode: response.status,
    duration: endTime - startTime,
    contentType,
    body
  };
  globalThis[FETCH_REQUESTS].push(fetchRequest);
  return response;
};
var router = async (request) => {
  const url2 = new URL(request.url);
  switch (url2.pathname) {
    case "/health" /* Health */:
      switch (request.method) {
        case "GET" /* Get */:
          return new Response(JSON.stringify({ ready: true }), {
            headers: { ["content-type" /* ContentType */]: "application/json" /* ApplicationJson */ }
          });
        default:
          return new Response(`method not allowed for ${"/health" /* Health */}`, { status: 405 /* MethodNotAllowed */ });
      }
    case "/invoke" /* Invoke */:
      switch (request.method) {
        case "POST" /* Post */:
          return invoke(request);
        default:
          return new Response(`method not allowed for ${"/invoke" /* Invoke */}`, { status: 405 /* MethodNotAllowed */ });
      }
    default:
      return new Response(`${url2.pathname} not found`, { status: 404 /* NotFound */ });
  }
};
var invoke = async (request) => {
  globalThis[LOG_ENTRIES] = [];
  globalThis[FETCH_REQUESTS] = [];
  const { parent, args, context, info } = await request.json();
  let returnValue = null;
  try {
    if (context) {
      context.kv = new import_kv.KVNamespace(new import_storage_memory.MemoryStorage());
    }
    returnValue = udf(parent, args, context, info);
    if (returnValue instanceof Promise) {
      returnValue = await returnValue;
    }
    if (returnValue instanceof Response) {
      const contentType = returnValue.headers.get("content-type" /* ContentType */)?.split(MIME_PROPERTY_SEPARATOR)[0].trim();
      switch (contentType) {
        case "application/json" /* ApplicationJson */:
          returnValue = await returnValue.json();
          break;
        case "text/plain" /* TextPlain */:
        case "text/html" /* TextHtml */:
          returnValue = await returnValue.text();
          break;
        default:
          returnValue = arrayBufferToBase64(await returnValue.arrayBuffer());
          break;
      }
    }
    returnValue = {
      Success: returnValue
    };
  } catch (error) {
    if (error == null) {
      returnValue = {
        Error: "nullish value thrown"
      };
    } else {
      if (error instanceof import_graphql.GraphQLError) {
        returnValue = {
          GraphQLError: {
            message: error.message,
            extensions: error.extensions
          }
        };
      } else {
        returnValue = {
          Error: error.toString()
        };
      }
    }
  }
  const jsonResponse = {
    value: returnValue,
    fetchRequests: globalThis[FETCH_REQUESTS],
    logEntries: globalThis[LOG_ENTRIES]
  };
  return new Response(JSON.stringify(jsonResponse), {
    headers: { ["content-type" /* ContentType */]: "application/json" /* ApplicationJson */ }
  });
};
/*! Bundled license information:

@miniflare/shared/dist/src/index.js:
  (*! Path sanitisation regexps adapted from node-sanitize-filename:
   * https://github.com/parshap/node-sanitize-filename/blob/209c39b914c8eb48ee27bcbde64b2c7822fdf3de/index.js#L4-L37
   *
   * Licensed under the ISC license:
   *
   * Copyright Parsha Pourkhomami <parshap@gmail.com>
   *
   * Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted, provided that the
   * above copyright notice and this permission notice appear in all copies.
   *
   * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
   * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
   * DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
   * ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
   *)
*/
