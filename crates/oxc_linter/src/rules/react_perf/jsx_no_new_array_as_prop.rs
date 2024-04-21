use oxc_ast::{
    ast::{Expression, JSXAttributeValue, JSXElement, JSXExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_prop_value, is_constructor_matching_name},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react-perf(jsx-no-new-array-as-prop): JSX attribute values should not contain Arrays created in the same scope.")]
#[diagnostic(severity(warning), help(r"simplify props or memoize props in the parent component (https://react.dev/reference/react/memo#my-component-rerenders-when-a-prop-is-an-object-or-array)."))]
struct JsxNoNewArrayAsPropDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct JsxNoNewArrayAsProp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevent Arrays that are local to the current method from being used as values of JSX props
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <Item list={[]} />

    /// <Item list={new Array()} />
    /// <Item list={Array()} />
    /// <Item list={this.props.list || []} />
    /// <Item list={this.props.list ? this.props.list : []} />
    ///
    /// // Good
    /// <Item list={this.props.list} />
    /// ```
    JsxNoNewArrayAsProp,
    correctness
);

impl Rule for JsxNoNewArrayAsProp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_elem) = node.kind() {
            check_jsx_element(jsx_elem, ctx);
        }
    }
}

fn check_jsx_element<'a>(jsx_elem: &JSXElement<'a>, ctx: &LintContext<'a>) {
    for item in &jsx_elem.opening_element.attributes {
        match get_prop_value(item) {
            None => return,
            Some(JSXAttributeValue::ExpressionContainer(container)) => {
                if let JSXExpression::Expression(expr) = &container.expression {
                    if let Some(span) = check_expression(expr) {
                        ctx.diagnostic(JsxNoNewArrayAsPropDiagnostic(span));
                    }
                }
            }
            _ => {}
        };
    }
}

fn check_expression(expr: &Expression) -> Option<Span> {
    match expr.without_parenthesized() {
        Expression::ArrayExpression(expr) => Some(expr.span),
        Expression::CallExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Array") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::NewExpression(expr) => {
            if is_constructor_matching_name(&expr.callee, "Array") {
                Some(expr.span)
            } else {
                None
            }
        }
        Expression::LogicalExpression(expr) => {
            check_expression(&expr.left).or_else(|| check_expression(&expr.right))
        }
        Expression::ConditionalExpression(expr) => {
            check_expression(&expr.consequent).or_else(|| check_expression(&expr.alternate))
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![r"<Item list={this.props.list} />"];

    let fail = vec![
        r"<Item list={[]} />",
        r"<Item list={new Array()} />",
        r"<Item list={Array()} />",
        r"<Item list={this.props.list || []} />",
        r"<Item list={this.props.list ? this.props.list : []} />",
        r"<Item list={this.props.list || (this.props.arr ? this.props.arr : [])} />",
    ];

    Tester::new(JsxNoNewArrayAsProp::NAME, pass, fail)
        .with_react_perf_plugin(true)
        .test_and_snapshot();
}
