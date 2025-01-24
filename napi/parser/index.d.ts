/* auto-generated by NAPI-RS */
/* eslint-disable */

export * from '@oxc-project/types';
export declare class MagicString {
  /** Get source text from utf8 offset. */
  getSourceText(start: number, end: number): string
  /** Get 0-based line and column number from utf8 offset. */
  getLineColumnNumber(offset: number): LineColumn
  /** Get UTF16 byte offset from UTF8 byte offset. */
  getUtf16ByteOffset(offset: number): number
  length(): number
  toString(): string
  hasChanged(): boolean
  append(input: string): this
  appendLeft(index: number, input: string): this
  appendRight(index: number, input: string): this
  indent(): this
  prepend(input: string): this
  prependLeft(index: number, input: string): this
  prependRight(index: number, input: string): this
  relocate(start: number, end: number, to: number): this
  remove(start: number, end: number): this
  generateMap(options?: Partial<GenerateDecodedMapOptions>): {
    toString: () => string;
    toUrl: () => string;
    toMap: () => {
      file?: string
      mappings: string
      names: Array<string>
      sourceRoot?: string
      sources: Array<string>
      sourcesContent?: Array<string>
      version: number
      x_google_ignoreList?: Array<number>
    }
  }
}

export declare class ParseResult {
  get program(): import("@oxc-project/types").Program
  get module(): EcmaScriptModule
  get comments(): Array<Comment>
  get errors(): Array<OxcError>
  get magicString(): MagicString
}

export interface Comment {
  type: 'Line' | 'Block'
  value: string
  start: number
  end: number
}

export interface DynamicImport {
  start: number
  end: number
  moduleRequest: Span
}

export interface EcmaScriptModule {
  /**
   * Has ESM syntax.
   *
   * i.e. `import` and `export` statements, and `import.meta`.
   *
   * Dynamic imports `import('foo')` are ignored since they can be used in non-ESM files.
   */
  hasModuleSyntax: boolean
  /** Import statements. */
  staticImports: Array<StaticImport>
  /** Export statements. */
  staticExports: Array<StaticExport>
  /** Dynamic import expressions. */
  dynamicImports: Array<DynamicImport>
  /** Span positions` of `import.meta` */
  importMetas: Array<Span>
}

export interface ErrorLabel {
  message?: string
  start: number
  end: number
}

export interface ExportExportName {
  kind: ExportExportNameKind
  name?: string
  start?: number
  end?: number
}

export declare const enum ExportExportNameKind {
  /** `export { name } */
  Name = 'Name',
  /** `export default expression` */
  Default = 'Default',
  /** `export * from "mod" */
  None = 'None'
}

export interface ExportImportName {
  kind: ExportImportNameKind
  name?: string
  start?: number
  end?: number
}

export declare const enum ExportImportNameKind {
  /** `export { name } */
  Name = 'Name',
  /** `export * as ns from "mod"` */
  All = 'All',
  /** `export * from "mod"` */
  AllButDefault = 'AllButDefault',
  /** Does not have a specifier. */
  None = 'None'
}

export interface ExportLocalName {
  kind: ExportLocalNameKind
  name?: string
  start?: number
  end?: number
}

export declare const enum ExportLocalNameKind {
  /** `export { name } */
  Name = 'Name',
  /** `export default expression` */
  Default = 'Default',
  /**
   * If the exported value is not locally accessible from within the module.
   * `export default function () {}`
   */
  None = 'None'
}

export interface GenerateDecodedMapOptions {
  /** The filename of the file containing the original source. */
  source?: string
  /** Whether to include the original content in the map's `sourcesContent` array. */
  includeContent: boolean
  /** Whether the mapping should be high-resolution. */
  hires: boolean | 'boundary'
}

export interface ImportName {
  kind: ImportNameKind
  name?: string
  start?: number
  end?: number
}

export declare const enum ImportNameKind {
  /** `import { x } from "mod"` */
  Name = 'Name',
  /** `import * as ns from "mod"` */
  NamespaceObject = 'NamespaceObject',
  /** `import defaultExport from "mod"` */
  Default = 'Default'
}

export interface LineColumn {
  line: number
  column: number
}

export interface OverwriteOptions {
  contentOnly: boolean
}

export interface OxcError {
  severity: Severity
  message: string
  labels: Array<ErrorLabel>
  helpMessage?: string
}

/**
 * Parse asynchronously.
 *
 * Note: This function can be slower than `parseSync` due to the overhead of spawning a thread.
 */
export declare function parseAsync(filename: string, sourceText: string, options?: ParserOptions | undefined | null): Promise<ParseResult>

export interface ParserOptions {
  sourceType?: 'script' | 'module' | 'unambiguous' | undefined
  /** Treat the source text as `js`, `jsx`, `ts`, or `tsx`. */
  lang?: 'js' | 'jsx' | 'ts' | 'tsx'
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

/** Parse synchronously. */
export declare function parseSync(filename: string, sourceText: string, options?: ParserOptions | undefined | null): ParseResult

/**
 * Parse without returning anything.
 *
 * This is for benchmark purposes such as measuring napi communication overhead.
 */
export declare function parseWithoutReturn(filename: string, sourceText: string, options?: ParserOptions | undefined | null): void

export declare const enum Severity {
  Error = 'Error',
  Warning = 'Warning',
  Advice = 'Advice'
}

export interface SourceMap {
  file?: string
  mappings: string
  names: Array<string>
  sourceRoot?: string
  sources: Array<string>
  sourcesContent?: Array<string>
  version: number
  x_google_ignoreList?: Array<number>
}

export interface SourceMapOptions {
  includeContent?: boolean
  source?: string
  hires?: boolean
}

export interface Span {
  start: number
  end: number
}

export interface StaticExport {
  start: number
  end: number
  entries: Array<StaticExportEntry>
}

export interface StaticExportEntry {
  start: number
  end: number
  moduleRequest?: ValueSpan
  /** The name under which the desired binding is exported by the module`. */
  importName: ExportImportName
  /** The name used to export this binding by this module. */
  exportName: ExportExportName
  /** The name that is used to locally access the exported value from within the importing module. */
  localName: ExportLocalName
}

export interface StaticImport {
  /** Start of import statement. */
  start: number
  /** End of import statement. */
  end: number
  /**
   * Import source.
   *
   * ```js
   * import { foo } from "mod";
   * //                   ^^^
   * ```
   */
  moduleRequest: ValueSpan
  /**
   * Import specifiers.
   *
   * Empty for `import "mod"`.
   */
  entries: Array<StaticImportEntry>
}

export interface StaticImportEntry {
  /**
   * The name under which the desired binding is exported by the module.
   *
   * ```js
   * import { foo } from "mod";
   * //       ^^^
   * import { foo as bar } from "mod";
   * //       ^^^
   * ```
   */
  importName: ImportName
  /**
   * The name that is used to locally access the imported value from within the importing module.
   * ```js
   * import { foo } from "mod";
   * //       ^^^
   * import { foo as bar } from "mod";
   * //              ^^^
   * ```
   */
  localName: ValueSpan
  /**
   * Whether this binding is for a TypeScript type-only import.
   *
   * `true` for the following imports:
   * ```ts
   * import type { foo } from "mod";
   * import { type foo } from "mod";
   * ```
   */
  isType: boolean
}

export interface ValueSpan {
  value: string
  start: number
  end: number
}
