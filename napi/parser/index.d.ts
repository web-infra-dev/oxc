/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

/**
 * Babel Parser Options
 *
 * <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
 */
export interface ParserOptions {
  sourceType?: 'script' | 'module' | 'unambiguous' | undefined
  sourceFilename?: string
}
export interface ParseResult {
  program: string
  comments: Array<Comment>
  errors: Array<string>
}
export interface Comment {
  type: string
  value: 'Line' | 'Block'
  start: number
  end: number
}
/**
 * Parse without returning anything.
 * This is for benchmark purposes such as measuring napi communication overhead.
 *
 * # Panics
 *
 * * File extension is invalid
 * * Serde JSON serialization
 */
export function parseWithoutReturn(sourceText: string, options?: ParserOptions | undefined | null): void
/**
 * # Panics
 *
 * * File extension is invalid
 * * Serde JSON serialization
 */
export function parseSync(sourceText: string, options?: ParserOptions | undefined | null): ParseResult
/**
 * Returns a binary AST in flexbuffers format.
 * This is a POC API. Error handling is not done yet.
 * # Panics
 *
 * * File extension is invalid
 * * FlexbufferSerializer serialization error
 */
export function parseSyncBuffer(sourceText: string, options?: ParserOptions | undefined | null): Buffer
/**
 * # Panics
 *
 * * Tokio crashes
 */
export function parseAsync(sourceText: string, options?: ParserOptions | undefined | null): Promise<ParseResult>
