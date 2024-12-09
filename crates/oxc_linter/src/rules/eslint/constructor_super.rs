use oxc_ast::ast::{
    AssignmentOperator, ClassBody, ClassElement, Expression, LogicalOperator, MethodDefinition,
    MethodDefinitionKind, Statement,
};
use oxc_ast::AstKind;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(PartialEq)]
enum ErrorReason {
    NotFound,
    MissingCallOnBranch,
    ReturnWithoutCall,
    MissingDefaultBranchOnSwitch,
    MultipleCalls,
}

struct ErrorReport {
    reason: ErrorReason,
    spans: Vec<Span>,
}

fn has_missing_all_super_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected to call 'super()'").with_label(span)
}

fn has_missing_some_super_call_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of 'super()' in some code paths.").with_labels(
        spans
            .into_iter()
            .map(|span| span.label("This path is lacking of a 'super()' call"))
            .collect::<Vec<LabeledSpan>>(),
    )
}

fn has_missing_default_switch_branch_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of 'super()' default.").with_labels(
        spans
            .into_iter()
            .map(|span| {
                span.label("This path is lacking of a 'super()' call inside 'case default:'")
            })
            .collect::<Vec<LabeledSpan>>(),
    )
}

fn has_unexpected_return_statement(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Lacked a call of 'super()' in some code paths.").with_labels(
        spans
            .into_iter()
            .map(|span| span.label("This path fast returns without calling 'super()'"))
            .collect::<Vec<LabeledSpan>>(),
    )
}

fn has_unexpected_super_call_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected 'super()' because 'super' is not a constructor").with_labels(
        spans
            .into_iter()
            .map(|span| span.label("Remove this 'super()' call expression"))
            .collect::<Vec<LabeledSpan>>(),
    )
}

fn has_multiple_super_call_diagnostic(spans: Vec<Span>) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected duplicate 'super()'.").with_labels(
        spans
            .into_iter()
            .map(|span| span.label("Remove one of this 'super()' call expression"))
            .collect::<Vec<LabeledSpan>>(),
    )
}

#[derive(Debug, Default, Clone)]
pub struct ConstructorSuper;

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
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ConstructorSuper,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details

    pending  // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for ConstructorSuper {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::Class(class) = node.kind() else { return };

        let Some(constructor) = get_constructor_method(&class.body) else {
            return;
        };

        if let Some(super_class) = &class.super_class {
            let super_class = &super_class.without_parentheses();
            let has_super_constructor = is_possible_constructor(super_class);

            if has_super_constructor {
                if let Err(error) = validate_method_super_call_expression(constructor) {
                    match error.reason {
                        ErrorReason::MissingDefaultBranchOnSwitch => {
                            ctx.diagnostic(has_missing_default_switch_branch_diagnostic(
                                error.spans,
                            ));
                        }
                        ErrorReason::ReturnWithoutCall => {
                            ctx.diagnostic(has_unexpected_return_statement(error.spans));
                        }
                        ErrorReason::MissingCallOnBranch => {
                            ctx.diagnostic(has_missing_some_super_call_diagnostic(error.spans));
                        }
                        ErrorReason::NotFound => {
                            ctx.diagnostic(has_missing_all_super_call_diagnostic(
                                constructor.key.span(),
                            ));
                        }
                        ErrorReason::MultipleCalls => {
                            ctx.diagnostic(has_multiple_super_call_diagnostic(error.spans));
                        }
                    };
                }
            } else if !has_return_statement(constructor) {
                if let Ok(result) = validate_method_super_call_expression(constructor) {
                    ctx.diagnostic(has_unexpected_super_call_diagnostic(result));
                } else {
                    ctx.diagnostic(has_missing_all_super_call_diagnostic(constructor.key.span()));
                }
            }
        } else if let Ok(result) = validate_method_super_call_expression(constructor) {
            ctx.diagnostic(has_unexpected_super_call_diagnostic(result));
        }
    }
}

fn is_possible_constructor(expression: &Expression<'_>) -> bool {
    if matches!(
        expression,
        Expression::ClassExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ThisExpression(_)
            | Expression::CallExpression(_)
            | Expression::NewExpression(_)
            | Expression::ChainExpression(_)
            | Expression::YieldExpression(_)
            | Expression::TaggedTemplateExpression(_)
            | Expression::MetaProperty(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
    ) {
        return true;
    }

    if matches!(expression, Expression::Identifier(identifier) if identifier.name != "undefined") {
        return true;
    }

    if let Expression::AssignmentExpression(assignment) = expression {
        if matches!(
            assignment.operator,
            AssignmentOperator::Assign | AssignmentOperator::LogicalAnd
        ) {
            return is_possible_constructor(&assignment.right);
        }

        if matches!(
            assignment.operator,
            AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
        ) {
            // ToDo check if left or right side
            return true;
        }

        return false;
    }

    if let Expression::LogicalExpression(logical) = expression {
        if logical.operator == LogicalOperator::And {
            return is_possible_constructor(&logical.right);
        }

        return is_possible_constructor(&logical.left) || is_possible_constructor(&logical.right);
    }

    if let Expression::ConditionalExpression(conditional) = expression {
        return is_possible_constructor(&conditional.alternate)
            || is_possible_constructor(&conditional.consequent);
    }

    if let Expression::SequenceExpression(sequence) = expression {
        return sequence.expressions.last().is_some_and(is_possible_constructor);
    }

    false
}

fn executes_always_super_expression<'a>(
    statement: &'a Statement<'a>,
) -> Result<Vec<Span>, ErrorReport> {
    if let Statement::ExpressionStatement(expression) = statement {
        if expression.expression.is_super_call_expression() {
            return Ok(vec![expression.expression.span()]);
        }

        if let Expression::ConditionalExpression(conditional) = &expression.expression {
            if conditional.consequent.is_super_call_expression()
                && conditional.alternate.is_super_call_expression()
            {
                return Ok(vec![conditional.consequent.span(), conditional.alternate.span()]);
            }
        }

        if let Expression::LogicalExpression(logical) = &expression.expression {
            if logical.left.is_super_call_expression() && logical.right.is_super_call_expression() {
                return Err(ErrorReport {
                    reason: ErrorReason::MultipleCalls,
                    spans: vec![logical.left.span(), logical.right.span()]
                })
            }
        }

        return Err(ErrorReport {
            reason: ErrorReason::NotFound,
            spans: vec![expression.expression.span()],
        });
    }

    if let Statement::IfStatement(if_statement) = &statement {
        if matches!(&if_statement.consequent, Statement::ReturnStatement(return_statement) if return_statement.argument.is_none())
        {
            return Err(ErrorReport {
                reason: ErrorReason::ReturnWithoutCall,
                spans: vec![if_statement.consequent.span()],
            });
        }

        if let Ok(mut consequent) = executes_always_super_expression(&if_statement.consequent) {
            let Some(alternative_call) = if_statement.alternate.as_ref() else {
                return Err(ErrorReport {
                    reason: ErrorReason::MissingCallOnBranch,
                    spans: vec![if_statement.span],
                });
            };

            if let Ok(alternative) = executes_always_super_expression(alternative_call) {
                consequent.extend(alternative);
                return Ok(consequent);
            }

            return Err(ErrorReport {
                reason: ErrorReason::MissingCallOnBranch,
                spans: vec![if_statement.span],
            });
        }

        return Err(ErrorReport { reason: ErrorReason::NotFound, spans: vec![if_statement.span] });
    }

    if let Statement::BlockStatement(block) = &statement {
        return has_body_possible_super_call_expression(&block.body);
    }

    if let Statement::SwitchStatement(switch) = &statement {
        let has_default = switch.cases.iter().any(oxc_ast::ast::SwitchCase::is_default_case);

        if !has_default {
            return Err(ErrorReport {
                reason: ErrorReason::MissingDefaultBranchOnSwitch,
                spans: vec![switch.span],
            });
        }

        let calls_grouped: Vec<Option<Vec<Span>>> = switch
            .cases
            .iter()
            .map(|case| {
                // ToDo: check for fast return and break statement (no double call)
                let all = case
                    .consequent
                    .iter()
                    .map(|statement| executes_always_super_expression(statement));

                // we found a super call, filter them
                if all.clone().any(|case| case.is_ok()) {
                    return Some(all.flatten().flatten().collect::<Vec<Span>>());
                }

                None
            })
            .collect();

        if !calls_grouped.iter().all(std::option::Option::is_some) {
            return Err(ErrorReport {
                reason: ErrorReason::MissingCallOnBranch,
                spans: vec![switch.span],
            });
        }

        let calls = calls_grouped.into_iter().flatten().flatten().collect::<Vec<Span>>();

        if !calls.is_empty() {
            return Ok(calls);
        }

        return Err(ErrorReport { reason: ErrorReason::NotFound, spans: vec![statement.span()] });
    }

    if let Statement::TryStatement(try_block) = &statement {
        if try_block.finalizer.is_none() {
            return Err(ErrorReport { reason: ErrorReason::NotFound, spans: vec![try_block.span] });
        }

        return has_body_possible_super_call_expression(
            &try_block.finalizer.as_ref().unwrap().body,
        );
    }

    Err(ErrorReport { reason: ErrorReason::NotFound, spans: vec![statement.span()] })
}

fn validate_method_super_call_expression<'a>(
    method: &'a MethodDefinition<'a>,
) -> Result<Vec<Span>, ErrorReport> {
    let Some(func_body) = &method.value.body else {
        return Ok(vec![]);
    };

    has_body_possible_super_call_expression(&func_body.statements)
}

fn has_body_possible_super_call_expression<'a>(
    body: &'a oxc_allocator::Vec<Statement<'a>>,
) -> Result<Vec<Span>, ErrorReport> {
    let mut found_calls: Vec<Vec<Span>> = vec![];

    for statement in body {
        if matches!(statement, Statement::ReturnStatement(_)) {
            return Err(ErrorReport {
                reason: ErrorReason::ReturnWithoutCall,
                spans: vec![statement.span()],
            });
        }

        let result = executes_always_super_expression(statement);

        if let Ok(result) = executes_always_super_expression(statement) {
            found_calls.push(result);
        } else if result.as_ref().err().unwrap().reason != ErrorReason::NotFound {
            return result;
        };
    }

    if found_calls.len() > 1 {
        return Err(ErrorReport {
            reason: ErrorReason::MultipleCalls,
            spans: found_calls.into_iter().flatten().collect::<Vec<Span>>(),
        });
    } else if found_calls.is_empty() {
        return Err(ErrorReport { reason: ErrorReason::NotFound, spans: vec![] });
    }

    Ok(found_calls.into_iter().flatten().collect::<Vec<Span>>())
}

fn has_return_statement<'a>(method: &'a MethodDefinition<'a>) -> bool {
    let Some(func_body) = &method.value.body else {
        return false;
    };

    for statement in &func_body.statements {
        if is_blocking_execution(statement) {
            return true;
        }
    }

    false
}

fn is_blocking_execution<'a>(statement: &'a Statement<'a>) -> bool {
    if matches!(statement, Statement::ReturnStatement(_) | Statement::ThrowStatement(_)) {
        return true;
    }

    false
}

fn get_constructor_method<'a>(class: &'a ClassBody<'a>) -> Option<&'a MethodDefinition<'a>> {
    if class.body.is_empty() {
        return None;
    }

    let constructor = class.body.iter().find(|part| matches!(part, ClassElement::MethodDefinition(method) if method.kind == MethodDefinitionKind::Constructor));

    constructor?;

    // we already checked it, only for the compiler
    let ClassElement::MethodDefinition(method) = constructor.unwrap() else {
        return None;
    };

    Some(method)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "class A { }",
"class A { constructor() { } }",
"class A extends null { }",
"class A extends B { }",
"class A extends B { constructor() { super(); } }",
"class A extends B { constructor() { if (true) { super(); } else { super(); } } }",
"class A extends (class B {}) { constructor() { super(); } }",
"class A extends (B = C) { constructor() { super(); } }",
"class A extends (B &&= C) { constructor() { super(); } }",
"class A extends (B ||= C) { constructor() { super(); } }",
"class A extends (B ??= C) { constructor() { super(); } }",
"class A extends (B ||= 5) { constructor() { super(); } }",
"class A extends (B ??= 5) { constructor() { super(); } }",
"class A extends (B || C) { constructor() { super(); } }",
"class A extends (5 && B) { constructor() { super(); } }",
"class A extends (false && B) { constructor() { super(); } }",
"class A extends (B || 5) { constructor() { super(); } }",
"class A extends (B ?? 5) { constructor() { super(); } }",
"class A extends (a ? B : C) { constructor() { super(); } }",
"class A extends (B, C) { constructor() { super(); } }",
"class A { constructor() { class B extends C { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C { constructor() { } } } }",
"class A extends B { constructor() { a ? super() : super(); } }",
"class A extends B { constructor() { if (a) super(); else super(); } }",
"class A extends B { constructor() { switch (a) { case 0: super(); break; default: super(); } } }",
"class A extends B { constructor() { try {} finally { super(); } } }",
"class A extends B { constructor() { if (a) throw Error(); super(); } }",
"class A extends B { constructor() { if (true) return a; super(); } }",
"class A extends null { constructor() { return a; } }",
"class A { constructor() { return a; } }",
"class A extends B { constructor(a) { super(); for (const b of a) { this.a(); } } }",
"class A extends B { constructor(a) { super(); for (b in a) ( foo(b) ); } }",
"class Foo extends Object { constructor(method) { super(); this.method = method || function() {}; } }",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0; i++);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (; i < 0; i++);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;; i++) {
			            if (foo) break;
			        }
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0; i < 0;);
			    }
			}
			",
"class A extends Object {
			    constructor() {
			        super();
			        for (let i = 0;;) {
			            if (foo) break;
			        }
			    }
			}
			",
"
			            class A extends B {
			                constructor(props) {
			                    super(props);
			
			                    try {
			                        let arr = [];
			                        for (let a of arr) {
			                        }
			                    } catch (err) {
			                    }
			                }
			            }
			        ",
"class A extends obj?.prop { constructor() { super(); } }",
"
			            class A extends Base {
			                constructor(list) {
			                    for (const a of list) {
			                        if (a.foo) {
			                            super(a);
			                            return;
			                        }
			                    }
			                    super();
			                }
			            }
			        "
    ];

    let fail = vec![
        "class A extends null { constructor() { super(); } }",
"class A extends null { constructor() { } }",
"class A extends 100 { constructor() { super(); } }",
"class A extends 'test' { constructor() { super(); } }",
"class A extends (B = 5) { constructor() { super(); } }",
"class A extends (B && 5) { constructor() { super(); } }",
"class A extends (B &&= 5) { constructor() { super(); } }",
"class A extends (B += C) { constructor() { super(); } }",
"class A extends (B -= C) { constructor() { super(); } }",
"class A extends (B **= C) { constructor() { super(); } }",
"class A extends (B |= C) { constructor() { super(); } }",
"class A extends (B &= C) { constructor() { super(); } }",
"class A extends B { constructor() { } }",
"class A extends B { constructor() { for (var a of b) super.foo(); } }",
"class A extends B { constructor() { for (var i = 1; i < 10; i++) super.foo(); } }",
"class A extends B { constructor() { var c = class extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { var c = () => super(); } }",
"class A extends B { constructor() { class C extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { var C = class extends D { constructor() { super(); } } } }",
"class A extends B { constructor() { super(); class C extends D { constructor() { } } } }",
"class A extends B { constructor() { super(); var C = class extends D { constructor() { } } } }",
"class A extends B { constructor() { if (a) super(); } }",
"class A extends B { constructor() { if (a); else super(); } }",
"class A extends B { constructor() { a && super(); } }",
"class A extends B { constructor() { switch (a) { case 0: super(); } } }",
"class A extends B { constructor() { switch (a) { case 0: break; default: super(); } } }",
"class A extends B { constructor() { try { super(); } catch (err) {} } }",
"class A extends B { constructor() { try { a; } catch (err) { super(); } } }",
"class A extends B { constructor() { if (a) return; super(); } }",
"class A extends B { constructor() { super(); super(); } }",
"class A extends B { constructor() { super() || super(); } }",
"class A extends B { constructor() { if (a) super(); super(); } }",
// "class A extends B { constructor() { switch (a) { case 0: super(); default: super(); } } }",
"class A extends B { constructor(a) { while (a) super(); } }",
"class A extends B { constructor() { return; super(); } }",
"class Foo extends Bar {
			                constructor() {
			                    for (a in b) for (c in d);
			                }
			            }",
"class C extends D {
			
			                constructor() {
			                    do {
			                        something();
			                    } while (foo);
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    for (let i = 1;;i++) {
			                        if (bar) {
			                            break;
			                        }
			                    }
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    do {
			                        super();
			                    } while (foo);
			                }
			
			            }",
"class C extends D {
			
			                constructor() {
			                    while (foo) {
			                        if (bar) {
			                            super();
			                            break;
			                        }
			                    }
			                }
			
			            }"
    ];

    Tester::new(ConstructorSuper::NAME, ConstructorSuper::CATEGORY, pass, fail).test_and_snapshot();
}
