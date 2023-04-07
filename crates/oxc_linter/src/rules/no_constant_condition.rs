use oxc_ast::{ast::Expression, AstKind, Span};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{ast_util::IsConstant, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-constant-condition): Unexpected constant condition")]
#[diagnostic(severity(warning), help("Constant expression as a test condition is not allowed"))]
struct NoConstantConditionDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoConstantCondition {
    _check_loops: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow constant expressions in conditions
    ///
    /// ### Why is this bad?
    ///
    /// A constant expression (for example, a literal) as a test condition might be a typo or development trigger for a specific behavior.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// if (false) {
    ///    doSomethingUnfinished();
    /// }
    /// ```
    NoConstantCondition,
    nursery
);

impl Rule for NoConstantCondition {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            _check_loops: obj
                .and_then(|v| v.get("checkLoops"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
            AstKind::IfStatement(if_stmt) => {
                if if_stmt.test.is_constant(true, ctx) {
                    diagnose_constant_expr(&if_stmt.test, ctx);
                }
            }
            AstKind::ConditionalExpression(cond_expr) => {
                if cond_expr.test.is_constant(true, ctx) {
                    diagnose_constant_expr(&cond_expr.test, ctx);
                }
            }
            _ => {}
        }
    }
}

fn diagnose_constant_expr(expr: &Expression, ctx: &LintContext) {
    let diagnose = |span: Span| {
        ctx.diagnostic(NoConstantConditionDiagnostic(span));
    };

    match expr {
        Expression::ArrowFunctionExpression(expr) => diagnose(expr.span),
        Expression::FunctionExpression(expr) => diagnose(expr.span),
        Expression::ClassExpression(expr) => diagnose(expr.span),
        Expression::ObjectExpression(expr) => diagnose(expr.span),
        Expression::TemplateLiteral(expr) => diagnose(expr.span),
        Expression::ArrayExpression(expr) => diagnose(expr.span),
        Expression::UnaryExpression(expr) => diagnose(expr.span),
        Expression::BinaryExpression(expr) => diagnose(expr.span),
        Expression::LogicalExpression(expr) => diagnose(expr.span),
        Expression::NewExpression(expr) => diagnose(expr.span),
        Expression::AssignmentExpression(expr) => diagnose(expr.span),
        Expression::SequenceExpression(expr) => diagnose(expr.span),
        Expression::CallExpression(expr) => diagnose(expr.span),
        Expression::ParenthesizedExpression(expr) => diagnose(expr.span),
        Expression::Identifier(expr) => diagnose(expr.span),
        Expression::BooleanLiteral(expr) => diagnose(expr.span),
        Expression::NullLiteral(expr) => diagnose(expr.span),
        Expression::NumberLiteral(expr) => diagnose(expr.span),
        Expression::BigintLiteral(expr) => diagnose(expr.span),
        Expression::RegExpLiteral(expr) => diagnose(expr.span),
        Expression::StringLiteral(expr) => diagnose(expr.span),
        _ => {}
    }
}

#[test]
fn test_no_constant() {
    use crate::tester::Tester;

    let pass = vec![
        ("if(a);", None),
        ("if(a == 0);", None),
        ("if(a = f());", None),
        ("if(a += 1);", None),
        ("if(a |= 1);", None),
        ("if(a |= true);", None),
        ("if(a |= false);", None),
        ("if(a &= 1);", None),
        ("if(a &= true);", None),
        ("if(a &= false);", None),
        ("if(a >>= 1);", None),
        ("if(a >>= true);", None),
        ("if(a >>= false);", None),
        ("if(a >>>= 1);", None),
        ("if(a ??= 1);", None),
        ("if(a ??= true);", None),
        ("if(a ??= false);", None),
        ("if(a ||= b);", None),
        ("if(a ||= false);", None),
        ("if(a ||= 0);", None),
        ("if(a ||= void 0);", None),
        ("if(+(a ||= 1));", None),
        ("if(f(a ||= true));", None),
        ("if((a ||= 1) + 2);", None),
        ("if(1 + (a ||= true));", None),
        ("if(a ||= '' || false);", None),
        ("if(a ||= void 0 || null);", None),
        ("if((a ||= false) || b);", None),
        ("if(a || (b ||= false));", None),
        ("if((a ||= true) && b);", None),
        ("if(a && (b ||= true));", None),
        ("if(a &&= b);", None),
        ("if(a &&= true);", None),
        ("if(a &&= 1);", None),
        ("if(a &&= 'foo');", None),
        ("if((a &&= '') + false);", None),
        ("if('' + (a &&= null));", None),
        ("if(a &&= 1 && 2);", None),
        ("if((a &&= true) && b);", None),
        ("if(a && (b &&= true));", None),
        ("if((a &&= false) || b);", None),
        ("if(a || (b &&= false));", None),
        ("if(a ||= b ||= false);", None),
        ("if(a &&= b &&= true);", None),
        ("if(a ||= b &&= false);", None),
        ("if(a ||= b &&= true);", None),
        ("if(a &&= b ||= false);", None),
        ("if(a &&= b ||= true);", None),
        ("if(1, a);", None),
        ("if ('every' in []);", None),
        ("if (`\\\n${a}`) {}", None),
        ("if (`${a}`);", None),
        ("if (`${foo()}`);", None),
        ("if (`${a === 'b' && b==='a'}`);", None),
        ("if (`foo${a}` === 'fooa');", None),
        ("if (tag`a`);", None),
        ("if (tag`${a}`);", None),
        ("if (+(a || true));", None),
        ("if (-(a || true));", None),
        ("if (~(a || 1));", None),
        ("if (+(a && 0) === +(b && 0));", None),
        ("if(typeof x === 'undefined'){}", None),
        ("if(`${typeof x}` === 'undefined'){}", None),
        ("if(a === 'str' && typeof b){}", None),
        ("typeof a == typeof b", None),
        ("typeof 'a' === 'string'|| typeof b === 'string'", None),
        ("`${typeof 'a'}` === 'string'|| `${typeof b}` === 'string'", None),
        ("if (void a || a);", None),
        ("if (a || void a);", None),
        ("if(xyz === 'str1' && abc==='str2'){}", None),
        ("if(xyz === 'str1' || abc==='str2'){}", None),
        ("if(xyz === 'str1' || abc==='str2' && pqr === 5){}", None),
        ("if(typeof abc === 'string' && abc==='str2'){}", None),
        ("if(false || abc==='str'){}", None),
        ("if(true && abc==='str'){}", None),
        ("if(typeof 'str' && abc==='str'){}", None),
        ("if(abc==='str' || false || def ==='str'){}", None),
        ("if(true && abc==='str' || def ==='str'){}", None),
        ("if(true && typeof abc==='string'){}", None),
        ("if('str1' && a){}", None),
        ("if(a && 'str'){}", None),
        ("if ((foo || true) === 'baz') {}", None),
        ("if ((foo || 'bar') === 'baz') {}", None),
        ("if ((foo || 'bar') !== 'baz') {}", None),
        ("if ((foo || 'bar') == 'baz') {}", None),
        ("if ((foo || 'bar') != 'baz') {}", None),
        ("if ((foo || 233) > 666) {}", None),
        ("if ((foo || 233) < 666) {}", None),
        ("if ((foo || 233) >= 666) {}", None),
        ("if ((foo || 233) <= 666) {}", None),
        ("if ((key || 'k') in obj) {}", None),
        ("if ((foo || {}) instanceof obj) {}", None),
        ("if ((foo || 'bar' || 'bar') === 'bar');", None),
        ("if ((foo || 1n) === 'baz') {}", None),
        ("if (a && 0n || b);", None),
        ("if(1n && a){};", None),
        ("if ('' + [y] === '' + [ty]) {}", None),
        ("if ('a' === '' + [ty]) {}", None),
        ("if ('' + [y, m, d] === 'a') {}", None),
        ("if ('' + [y, 'm'] === '' + [ty, 'tm']) {}", None),
        ("if ('' + [y, 'm'] === '' + ['ty']) {}", None),
        ("if ([,] in\n\n($2))\n ;\nelse\n ;", None),
        ("if ([...x]+'' === 'y'){}", None),
        ("if([a]==[b]) {}", None),
        ("if (+[...a]) {}", None),
        ("if (+[...[...a]]) {}", None),
        ("if (`${[...a]}`) {}", None),
        ("if (`${[a]}`) {}", None),
        ("if (+[a]) {}", None),
        ("if (0 - [a]) {}", None),
        ("if (1 * [a]) {}", None),
        ("if (Boolean(a)) {}", None),
        ("if (Boolean(...args)) {}", None),
        ("if (foo.Boolean(1)) {}", None),
        // TODO
        // ("const undefined = 'lol'; if (undefined) {}", None),
        // ("function foo(Boolean) { if (Boolean(1)) {} }", None),
        // ("const Boolean = () => {}; if (Boolean(1)) {}", None),
        // "if (Boolean()) {}",
        // "if (undefined) {}",
        ("q > 0 ? 1 : 2;", None),
        ("`${a}` === a ? 1 : 2", None),
        ("`foo${a}` === a ? 1 : 2", None),
        ("tag`a` === a ? 1 : 2", None),
        ("tag`${a}` === a ? 1 : 2", None),
    ];

    let fail = vec![
        ("if(-2);", None),
        ("if(-2);", None),
        ("if(true);", None),
        ("if(1);", None),
        ("if({});", None),
        ("if(0 < 1);", None),
        ("if(0 || 1);", None),
        ("if(a, 1);", None),
        ("if(`foo`);", None),
        ("if(``);", None),
        ("if(`\\\n`);", None),
        ("if(`${'bar'}`);", None),
        ("if(`${'bar' + `foo`}`);", None),
        ("if(`foo${false || true}`);", None),
        ("if(`foo${0 || 1}`);", None),
        ("if(`foo${bar}`);", None),
        ("if(`${bar}foo`);", None),
        ("if(!(true || a));", None),
        ("if(!(a && void b && c));", None),
        ("if(0 || !(a && null));", None),
        ("if(1 + !(a || true));", None),
        ("if(!(null && a) > 1);", None),
        ("if(+(!(a && 0)));", None),
        ("if(!typeof a === 'string');", None),
        ("if(-('foo' || a));", None),
        ("if(+(void a && b) === ~(1 || c));", None),
        ("if(a ||= true);", None),
        ("if(a ||= 5);", None),
        ("if(a ||= 'foo' || b);", None),
        ("if(a ||= b || /regex/);", None),
        ("if(a ||= b ||= true);", None),
        ("if(a ||= b ||= c || 1);", None),
        ("if(!(a ||= true));", None),
        ("if(!(a ||= 'foo') === true);", None),
        ("if(!(a ||= 'foo') === false);", None),
        ("if(a || (b ||= true));", None),
        ("if((a ||= 1) || b);", None),
        ("if((a ||= true) && true);", None),
        ("if(true && (a ||= true));", None),
        ("if(a &&= false);", None),
        ("if(a &&= null);", None),
        ("if(a &&= void b);", None),
        ("if(a &&= 0 && b);", None),
        ("if(a &&= b && '');", None),
        ("if(a &&= b &&= false);", None),
        ("if(a &&= b &&= c && false);", None),
        ("if(!(a &&= false));", None),
        ("if(!(a &&= 0) + 1);", None),
        ("if(a && (b &&= false));", None),
        ("if((a &&= null) && b);", None),
        ("if(false || (a &&= false));", None),
        ("if((a &&= false) || false);", None),
        ("if(typeof x){}", None),
        ("if(typeof 'abc' === 'string'){}", None),
        ("if(a = typeof b){}", None),
        ("if(a, typeof b){}", None),
        ("if(typeof 'a' == 'string' || typeof 'b' == 'string'){}", None),
        ("if(1 || void x);", None),
        ("if(void x);", None),
        ("if(y = void x);", None),
        ("if(x, void x);", None),
        ("if(void x === void y);", None),
        ("if(void x && a);", None),
        ("if(a && void x);", None),
        ("if(false && abc==='str'){}", None),
        ("if(true || abc==='str'){}", None),
        ("if(1 || abc==='str'){}", None),
        ("if(abc==='str' || true){}", None),
        ("if(abc==='str' || true || def ==='str'){}", None),
        ("if(false || true){}", None),
        ("if(typeof abc==='str' || true){}", None),
        ("if('str' || a){}", None),
        ("if('str' || abc==='str'){}", None),
        ("if('str1' || 'str2'){}", None),
        ("if('str1' && 'str2'){}", None),
        ("if(abc==='str' || 'str'){}", None),
        ("if(a || 'str'){}", None),
        ("if([a]) {}", None),
        ("if([]) {}", None),
        ("if(''+['a']) {}", None),
        ("if(''+[]) {}", None),
        ("if(+1) {}", None),
        ("if ([,] + ''){}", None),
        ("if(/foo/ui);", None),
        ("if(0n);", None),
        ("if(0b0n);", None),
        ("if(0o0n);", None),
        ("if(0x0n);", None),
        ("if(0b1n);", None),
        ("if(0o1n);", None),
        ("if(0x1n);", None),
        ("if(0x1n || foo);", None),
        // Classes and instances are always truthy
        ("if(class {}) {}", None),
        ("if(new Foo()) {}", None),
        // Boxed primitives are always truthy
        ("if(new Boolean(foo)) {}", None),
        ("if(new String(foo)) {}", None),
        ("if(new Number(foo)) {}", None),
        // Spreading a constant array
        ("if(`${[...['a']]}`) {}", None),
        // undefined is always falsy (except in old browsers that let you
        // re-assign, but that's an obscure enough edge case to not worry about)
        ("if (undefined) {}", None),
        // Coercion to boolean via Boolean function
        ("if (Boolean(1)) {}", None),
        ("if (Boolean()) {}", None),
        ("if (Boolean([a])) {}", None),
        ("if (Boolean(1)) { function Boolean() {}}", None),
        ("true ? 1 : 2;", None),
        ("1 ? 1 : 2;", None),
        ("q = 0 ? 1 : 2;", None),
        ("(q = 0) ? 1 : 2;", None),
        ("`` ? 1 : 2;", None),
        ("`foo` ? 1 : 2;", None),
        ("`foo${bar}` ? 1 : 2;", None),
    ];

    Tester::new(NoConstantCondition::NAME, pass, fail).test_and_snapshot();
}
