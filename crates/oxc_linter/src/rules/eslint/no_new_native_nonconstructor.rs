use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-new-native-nonconstructor): `{0}` cannot be called as a constructor.")]
#[diagnostic(severity(warning))]
struct NoNewNativeNonconstructorDiagnostic(CompactStr, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNewNativeNonconstructor;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow new operators with global non-constructor functions (Symbol, BigInt)
    ///
    /// ### Why is this bad?
    ///
    /// Both new Symbol and new BigInt throw a type error because they are functions and not classes.
    /// It is easy to make this mistake by assuming the uppercase letters indicate classes.
    ///
    /// ### Example
    /// ```javascript
    /// // throws a TypeError
    /// let foo = new Symbol("foo");
    ///
    /// // throws a TypeError
    /// let result = new BigInt(9007199254740991);
    /// ```
    NoNewNativeNonconstructor,
    correctness,
);

impl Rule for NoNewNativeNonconstructor {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(expr) = node.kind() else { return };
        let Expression::Identifier(ident) = &expr.callee else { return };
        if matches!(ident.name.as_str(), "Symbol" | "BigInt")
            && ctx.semantic().is_reference_to_global_variable(ident)
        {
            let start = expr.span.start;
            let end = start + 3;
            ctx.diagnostic(NoNewNativeNonconstructorDiagnostic(
                ident.name.to_compact_str(),
                Span::new(start, end),
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var foo = Symbol('foo');",
        "function bar(Symbol) { var baz = new Symbol('baz');}",
        "function Symbol() {} new Symbol();",
        "new foo(Symbol);",
        "new foo(bar, Symbol);",
        "var foo = BigInt(9007199254740991);",
        "function bar(BigInt) { var baz = new BigInt(9007199254740991);}",
        "function BigInt() {} new BigInt();",
        "new foo(BigInt);",
        "new foo(bar, BigInt);",
    ];

    let fail = vec![
        "var foo = new Symbol('foo');",
        "function bar() { return function Symbol() {}; } var baz = new Symbol('baz');",
        "var foo = new BigInt(9007199254740991);",
        "function bar() { return function BigInt() {}; } var baz = new BigInt(9007199254740991);",
    ];

    Tester::new(NoNewNativeNonconstructor::NAME, pass, fail).test_and_snapshot();
}
