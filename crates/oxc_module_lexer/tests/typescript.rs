mod esm;
use esm::{ExportSpecifier, ImportSpecifier, ModuleLexer};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn parse(source: &str) -> ModuleLexer {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_module(true).with_typescript_definition(true);
    let ret = Parser::new(&allocator, source, source_type).parse();
    assert!(ret.errors.is_empty(), "{source} should not produce errors.\n{:?}", ret.errors);
    let module_lexer = oxc_module_lexer::ModuleLexer::new().build(&ret.program);
    ModuleLexer {
        imports: module_lexer
            .imports
            .into_iter()
            .map(|i| ImportSpecifier {
                n: i.n.map(|n| n.to_string()),
                s: i.s,
                e: i.e,
                ss: i.ss,
                se: i.se,
                d: i.d,
                a: i.a,
                t: i.t,
            })
            .collect(),
        exports: module_lexer
            .exports
            .into_iter()
            .map(|e| ExportSpecifier {
                n: e.n.to_string(),
                ln: e.ln.map(|ln| ln.to_string()),
                s: e.s,
                e: e.e,
                ls: e.ls,
                le: e.le,
                t: e.t,
            })
            .collect(),
        has_module_syntax: module_lexer.has_module_syntax,
        facade: module_lexer.facade,
    }
}

#[test]
fn import_type_named() {
    let source = "import type { foo } from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt.t);
}

#[test]
fn import_type_namespace() {
    let source = "import type * as foo from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt.t);
}

#[test]
fn import_type_default() {
    let source = "import type foo from 'foo'";
    let impt = &parse(source).imports[0];
    assert!(impt.t);
}

#[test]
fn dynamic_import_value() {
    let source = "import('foo')";
    let impt = &parse(source).imports[0];
    assert!(!impt.t);
}

#[test]
fn dynamic_import_type() {
    let source = "const foo: import('foo')";
    let impt = &parse(source).imports[0];
    assert!(impt.t);
    assert_eq!(impt.n.as_ref().unwrap(), "foo");
}

#[test]
fn export_type_named() {
    let source = "export type { foo } from 'foo'";
    let expt = &parse(source).exports[0];
    assert!(expt.t);
}

#[test]
fn export_type_namespace() {
    let source = "export type * as foo from 'foo'";
    let expt = &parse(source).exports[0];
    assert!(expt.t);
}
