use oxc_ast::{
    ast::{Expression, JSXAttributeItem, JSXAttributeName, JSXChild, ObjectPropertyKind},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_danger_with_children_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Only set one of `children` or `props.dangerouslySetInnerHTML`")
			.with_help("`dangerouslySetInnerHTML` is not compatible with also passing children and React will throw a warning at runtime.")
			.with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDangerWithChildren;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    NoDangerWithChildren,
    correctness
);

impl Rule for NoDangerWithChildren {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXElement(jsx) => {
                let has_children = if !jsx.children.is_empty() && !is_line_break(&jsx.children[0]) {
                    true
                } else if has_jsx_prop(ctx, node, "children") {
                    true
                } else {
                    false
                };

                if !has_children {
                    return;
                }

                let has_danger_prop = has_jsx_prop(ctx, node, "dangerouslySetInnerHTML");

                if has_danger_prop {
                    ctx.diagnostic(no_danger_with_children_diagnostic(jsx.span));
                }
            }
            AstKind::CallExpression(call_expr) => {
                if call_expr.arguments.len() <= 1 {
                    return;
                }
                if let Expression::StaticMemberExpression(callee) = &call_expr.callee {
                    if callee.property.name != "createElement" {
                        return;
                    }

                    let props = call_expr
                        .arguments
                        .get(1)
                        .expect("Could not get props argument for createElement call")
                        .as_expression();

                    dbg!(call_expr);

                    // If there are three arguments, then it is a JSX element with children.
                    // If it's just two arguments, it only has children if the props object has a children property.
                    let has_children = if call_expr.arguments.len() == 2 {
                        match props {
                            Some(Expression::ObjectExpression(obj_expr)) => {
                                is_object_with_prop_name(&obj_expr.properties, "children")
                            }
                            Some(Expression::Identifier(ident)) => {
                                does_object_var_have_prop_name(ctx, node, &ident.name, "children")
                            }
                            _ => false,
                        }
                    } else {
                        true
                    };

                    let has_danger_prop = match props {
                        Some(Expression::ObjectExpression(obj_expr)) => is_object_with_prop_name(
                            &obj_expr.properties,
                            "dangerouslySetInnerHTML",
                        ),
                        Some(Expression::Identifier(ident)) => does_object_var_have_prop_name(
                            ctx,
                            node,
                            &ident.name,
                            "dangerouslySetInnerHTML",
                        ),
                        _ => false,
                    };

                    if has_danger_prop && has_children {
                        ctx.diagnostic(no_danger_with_children_diagnostic(call_expr.span));
                    }
                }
            }
            _ => (),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div>Children</div>",
        "<div {...props} />",
        r#"<div dangerouslySetInnerHTML={{ __html: "HTML" }} />"#,
        r#"<div children="Children" />"#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props} />
        "#,
        r#"
        const moreProps = { className: "eslint" };
        const props = { children: "Children", ...moreProps };
        <div {...props} />
        "#,
        r#"
        const otherProps = { children: "Children" };
        const { a, b, ...props } = otherProps;
        <div {...props} />
        "#,
        "<Hello>Children</Hello>",
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }} />"#,
        r#"
        <Hello dangerouslySetInnerHTML={{ __html: "HTML" }}>
        </Hello>
        "#,
        r#"React.createElement("div", { dangerouslySetInnerHTML: { __html: "HTML" } });"#,
        r#"React.createElement("div", {}, "Children");"#,
        r#"React.createElement("Hello", { dangerouslySetInnerHTML: { __html: "HTML" } });"#,
        r#"React.createElement("Hello", {}, "Children");"#,
        "<Hello {...undefined}>Children</Hello>",
        r#"React.createElement("Hello", undefined, "Children")"#,
        "
        const props = {...props, scratch: {mode: 'edit'}};
        const component = shallow(<TaskEditableTitle {...props} />);
        ",
    ];

    let fail = vec![
        r#"
        <div dangerouslySetInnerHTML={{ __html: "HTML" }}>
            Children
        </div>
        "#,
        r#"<div dangerouslySetInnerHTML={{ __html: "HTML" }} children="Children" />"#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props}>Children</div>
        "#,
        r#"
        const props = { children: "Children", dangerouslySetInnerHTML: { __html: "HTML" } };
        <div {...props} />
        "#,
        r#"
        <Hello dangerouslySetInnerHTML={{ __html: "HTML" }}>
            Children
        </Hello>
        "#,
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }} children="Children" />"#,
        r#"<Hello dangerouslySetInnerHTML={{ __html: "HTML" }}> </Hello>"#,
        r#"
        React.createElement(
            "div",
            { dangerouslySetInnerHTML: { __html: "HTML" } },
            "Children"
        );
        "#,
        r#"
        React.createElement(
            "div",
            {
                dangerouslySetInnerHTML: { __html: "HTML" },
                children: "Children",
            }
        );
        "#,
        r#"
        React.createElement(
            "Hello",
            { dangerouslySetInnerHTML: { __html: "HTML" } },
            "Children"
        );
        "#,
        r#"
        React.createElement(
            "Hello",
            {
                dangerouslySetInnerHTML: { __html: "HTML" },
                children: "Children",
            }
        );
        "#,
        r#"
        const props = { dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props, "Children");
        "#,
        r#"
        const props = { children: "Children", dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props);
        "#,
        r#"
        const moreProps = { children: "Children" };
        const otherProps = { ...moreProps };
        const props = { ...otherProps, dangerouslySetInnerHTML: { __html: "HTML" } };
        React.createElement("div", props);
        "#,
    ];

    Tester::new(NoDangerWithChildren::NAME, pass, fail).test_and_snapshot();
}

fn is_whitespace(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}

fn is_line_break(child: &JSXChild) -> bool {
    if let JSXChild::Text(text) = child {
        let is_multi_line = text.value.contains("\n");
        is_multi_line && is_whitespace(text.value.as_str())
    } else {
        false
    }
}

/// Given a JSX element, find the JSXAttributeItem with the given name.
/// If there are spread props, it will search within those as well.
fn has_jsx_prop<'a>(ctx: &LintContext<'a>, node: &AstNode<'a>, prop_name: &'static str) -> bool {
    if let AstKind::JSXElement(jsx) = node.kind() {
        jsx.opening_element.attributes.iter().any(|attr| match attr {
            JSXAttributeItem::Attribute(attr) => {
                if let JSXAttributeName::Identifier(ident) = &attr.name {
                    ident.name == prop_name
                } else {
                    false
                }
            }
            JSXAttributeItem::SpreadAttribute(attr) => {
                if let Some(ident) = attr.argument.get_identifier_reference() {
                    does_object_var_have_prop_name(ctx, node, &ident.name.as_str(), prop_name)
                } else {
                    false
                }
            }
        })
    } else {
        false
    }
}

fn does_object_var_have_prop_name<'a>(
    ctx: &LintContext<'a>,
    node: &AstNode<'a>,
    name: &str,
    prop_name: &str,
) -> bool {
    if let Some(symbol) = ctx
        .scopes()
        .find_binding(node.scope_id(), name)
        .map(|symbol_id| ctx.semantic().symbol_declaration(symbol_id))
    {
        if let AstKind::VariableDeclarator(var_decl) = symbol.kind() {
            if let Some(init) = &var_decl.init {
                if let Expression::ObjectExpression(obj_expr) = init {
                    is_object_with_prop_name(&obj_expr.properties, prop_name)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// Returns whether a given object has a property with the given name.
fn is_object_with_prop_name(
    obj_props: &oxc_allocator::Vec<'_, ObjectPropertyKind<'_>>,
    prop_name: &str,
) -> bool {
    obj_props.iter().any(|prop| match prop {
        ObjectPropertyKind::ObjectProperty(obj_prop) => {
            if let Some(key) = obj_prop.key.static_name() {
                key == prop_name
            } else {
                false
            }
        }
        _ => false,
    })
}
