export * from "@oxc-project/types";
export interface Comment {
  type: 'Line' | 'Block'
  value: string
  start: number
  end: number
}

export interface ModuleLexer {
  imports: Array<ModuleLexerImportSpecifier>
  exports: Array<ModuleLexerExportSpecifier>
  /**
   * ESM syntax detection
   *
   * The use of ESM syntax: import / export statements and `import.meta`
   */
  hasModuleSyntax: boolean
  /** Facade modules that only use import / export syntax */
  facade: boolean
}

/**
 * # Panics
 *
 * * Tokio crashes
 */
export declare function moduleLexerAsync(sourceText: string, options?: ParserOptions | undefined | null): Promise<ModuleLexer>

export interface ModuleLexerExportSpecifier {
  /** Exported name */
  n: string
  /** Local name, or undefined. */
  ln?: string
  /** Start of exported name */
  s: number
  /** End of exported name */
  e: number
  /** Start of local name */
  ls?: number
  /** End of local name */
  le?: number
}

export interface ModuleLexerImportSpecifier {
  /**
   * Module name
   *
   * To handle escape sequences in specifier strings, the .n field of imported specifiers will be provided where possible.
   *
   * For dynamic import expressions, this field will be empty if not a valid JS string.
   */
  n?: string
  /** Start of module specifier */
  s: number
  /** End of module specifier */
  e: number
  /** Start of import statement */
  ss: number
  /** End of import statement */
  se: number
  /**
   * Import Type
   * * If this import keyword is a dynamic import, this is the start value.
   * * If this import keyword is a static import, this is -1.
   * * If this import keyword is an import.meta expression, this is -2.
   * * If this import is an `export *`, this is -3.
   */
  d: number
  /**
   * If this import has an import assertion, this is the start value
   * Otherwise this is `-1`.
   */
  a: number
}

/**
 * Outputs the list of exports and locations of import specifiers,
 * including dynamic import and import meta handling.
 *
 * # Panics
 *
 * * File extension is invalid
 */
export declare function moduleLexerSync(sourceText: string, options?: ParserOptions | undefined | null): ModuleLexer

/**
 * # Panics
 *
 * * Tokio crashes
 */
export declare function parseAsync(sourceText: string, options?: ParserOptions | undefined | null): Promise<ParseResult>

export interface ParseResult {
  program: import("@oxc-project/types").Program
  comments: Array<Comment>
  errors: Array<string>
}

/**
 * Babel Parser Options
 *
 * <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
 */
export interface ParserOptions {
  sourceType?: 'script' | 'module' | 'unambiguous' | undefined
  sourceFilename?: string
  /**
   * Emit `ParenthesizedExpression` in AST.
   *
   * If this option is true, parenthesized expressions are represented by
   * (non-standard) `ParenthesizedExpression` nodes that have a single `expression` property
   * containing the expression inside parentheses.
   *
   * Default: true
   */
  preserveParens?: boolean
}

/**
 * # Panics
 *
 * * File extension is invalid
 * * Serde JSON serialization
 */
export declare function parseSync(sourceText: string, options?: ParserOptions | undefined | null): ParseResult

/**
 * Parse without returning anything.
 * This is for benchmark purposes such as measuring napi communication overhead.
 *
 * # Panics
 *
 * * File extension is invalid
 * * Serde JSON serialization
 */
export declare function parseWithoutReturn(sourceText: string, options?: ParserOptions | undefined | null): void

