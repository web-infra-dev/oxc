use oxc_ast::{
    ast::{ExportDefaultDeclarationKind, TSType},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("typescript-eslint(consistent-type-definitions):")]
#[diagnostic(severity(warning), help("Use an `{0}` instead of a `{1}`"))]
struct ConsistentTypeDefinitionsDiagnostic(
    &'static str,
    &'static str,
    #[label("Use an `{0}` instead of a `{1}`")] pub Span,
);

#[derive(Debug, Default, Clone)]
pub struct ConsistentTypeDefinitions {
    config: ConsistentTypeDefinitionsConfig,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum ConsistentTypeDefinitionsConfig {
    #[default]
    Interface,
    Type,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    ConsistentTypeDefinitions,
    style
);

impl Rule for ConsistentTypeDefinitions {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            ConsistentTypeDefinitionsConfig::default,
            |value| match value {
                "type" => ConsistentTypeDefinitionsConfig::Type,
                _ => ConsistentTypeDefinitionsConfig::Interface,
            },
        );
        Self { config }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::TSTypeAliasDeclaration(typ) => match &typ.type_annotation {
                TSType::TSTypeLiteral(lit)
                    if self.config == ConsistentTypeDefinitionsConfig::Interface =>
                {
                    ctx.diagnostic(ConsistentTypeDefinitionsDiagnostic("interface", "type", lit.span))
                }
                _ => {}
            },

            AstKind::ExportDefaultDeclaration(exp) => match &exp.declaration {
                ExportDefaultDeclarationKind::TSInterfaceDeclaration(decl) => ctx.diagnostic(
                    ConsistentTypeDefinitionsDiagnostic("type", "interface", decl.span),
                ),
                _ => {}
            },

            AstKind::TSInterfaceDeclaration(typ)
                if self.config == ConsistentTypeDefinitionsConfig::Type =>
            {
                ctx.diagnostic(ConsistentTypeDefinitionsDiagnostic("type", "interface", typ.span))
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = {};", Some(serde_json::json!(["interface"]))),
        ("interface A {}", Some(serde_json::json!(["interface"]))),
        (
            "
            interface A extends B {
              x: number;
            }
                  ",
            Some(serde_json::json!(["interface"])),
        ),
        ("type U = string;", Some(serde_json::json!(["interface"]))),
        ("type V = { x: number } | { y: string };", Some(serde_json::json!(["interface"]))),
        (
            "
            type Record<T, U> = {
              [K in T]: U;
            };
                  ",
            Some(serde_json::json!(["interface"])),
        ),
        ("type T = { x: number };", Some(serde_json::json!(["type"]))),
        ("type A = { x: number } & B & C;", Some(serde_json::json!(["type"]))),
        ("type A = { x: number } & B<T1> & C<T2>;", Some(serde_json::json!(["type"]))),
        (
            "
            export type W<T> = {
              x: T;
            };
                  ",
            Some(serde_json::json!(["type"])),
        ),
    ];

    let fail = vec![
        (
            "
        export type W<T> = {
          x: T;
        };
              ",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "
        export interface W<T> {
          x: T;
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
        namespace JSX {
          interface Array<T> {
            foo(x: (x: number) => T): T[];
          }
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
        global {
          interface Array<T> {
            foo(x: (x: number) => T): T[];
          }
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
        declare global {
          interface Array<T> {
            foo(x: (x: number) => T): T[];
          }
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
        declare global {
          namespace Foo {
            interface Bar {}
          }
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
			export default interface Test {
			  bar(): string;
			  foo(): number;
			}
			      ",
            Some(serde_json::json!(["type"])),
        ),
        (
            "
        export declare type Test = {
          foo: string;
          bar: string;
        };
              ",
            Some(serde_json::json!(["interface"])),
        ),
        (
            "
        export declare interface Test {
          foo: string;
          bar: string;
        }
              ",
            Some(serde_json::json!(["type"])),
        ),
    ];

    Tester::new(ConsistentTypeDefinitions::NAME, pass, fail).test_and_snapshot();
}
