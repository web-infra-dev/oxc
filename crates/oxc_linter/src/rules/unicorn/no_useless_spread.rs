use oxc_ast::{
    ast::{
        match_expression, Argument, ArrayExpression, ArrayExpressionElement, CallExpression,
        Expression, SpreadElement,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::{
        get_new_expr_ident_name, is_method_call, is_new_expression, outermost_paren_parent,
    },
    context::LintContext,
    fixer::{Fix, RuleFixer},
    rule::Rule,
    AstNode,
};

fn spread_in_list(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-unicorn(no-useless-spread): Using a spread operator here creates a new {x1} unnecessarily."))
        .with_help("Consider removing the spread operator.")
        .with_label(span)
}

fn spread_in_arguments(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(no-useless-spread): Using a spread operator here creates a new array unnecessarily.").with_help("This function accepts a rest parameter, it's unnecessary to create a new array and then spread it. Instead, supply the arguments directly.\nFor example, replace `foo(...[1, 2, 3])` with `foo(1, 2, 3)`.").with_label(span)
}

fn iterable_to_array(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("eslint-plugin-unicorn(no-useless-spread): `{x1}` accepts an iterable, so it's unnecessary to convert the iterable to an array."))
        .with_help("Consider removing the spread operator.")
        .with_label(span)
}

fn iterable_to_array_in_for_of(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(no-useless-spread): Using a spread operator here creates a new array unnecessarily.")
        .with_help("`for…of` can iterate over iterable, it's unnecessary to convert to an array.")
        .with_label(span)
}

fn iterable_to_array_in_yield_star(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(no-useless-spread): Using a spread operator here creates a new array unnecessarily.")
        .with_help("`yield*` can delegate to another iterable, so it's unnecessary to convert the iterable to an array.")
        .with_label(span)
}

fn clone_array(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint-plugin-unicorn(no-useless-spread): Using a spread operator here creates a new array unnecessarily.")
        .with_help(format!("`{x1}` returns a new array. Spreading it into an array expression to create a new array is redundant."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessSpread;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows using spread syntax in following, unnecessary cases:
    ///
    ///   - Spread an array literal as elements of an array literal
    ///   - Spread an array literal as arguments of a call or a `new` call
    ///   - Spread an object literal as properties of an object literal
    ///   - Use spread syntax to clone an array created inline
    ///
    /// ### Why is this bad?
    ///
    /// - The following builtins accept an iterable, so it's unnecessary to convert the iterable to an array:
    ///
    ///   - `Map` constructor
    ///   - `WeakMap` constructor
    ///   - `Set` constructor
    ///   - `WeakSet` constructor
    ///   - `TypedArray` constructor
    ///   - `Array.from(…)`
    ///   - `TypedArray.from(…)`
    ///   - `Promise.{all,allSettled,any,race}(…)`
    ///   - `Object.fromEntries(…)`
    ///
    /// - `for…of` loop can iterate over any iterable object not just array, so it's unnecessary to convert the iterable to an array.
    ///
    /// - `yield*` can delegate to another iterable, so it's unnecessary to convert the iterable to an array.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// const array = [firstElement, ...[secondElement], thirdElement];
    /// const object = {firstProperty, ...{secondProperty}, thirdProperty};
    /// foo(firstArgument, ...[secondArgument], thirdArgument);
    /// const object = new Foo(firstArgument, ...[secondArgument], thirdArgument);
    /// const set = new Set([...iterable]);
    /// const results = await Promise.all([...iterable]);
    /// for (const foo of [...set]);
    /// function * foo() {
    /// 	yield * [...anotherGenerator()];
    /// }
    /// function foo(bar) {
    /// 	return [
    /// 		...bar.map(x => x * 2),
    /// 	];
    /// }
    ///
    /// // Pass
    ///
    /// const array = [firstElement, secondElement, thirdElement];
    /// const object = {firstProperty, secondProperty, thirdProperty};
    /// foo(firstArgument, secondArgument, thirdArgument);
    /// const object = new Foo(firstArgument, secondArgument, thirdArgument);
    /// const array = [...foo, bar];
    /// const object = {...foo, bar};
    /// foo(foo, ...bar);
    /// const object = new Foo(...foo, bar);
    /// const set = new Set(iterable);
    /// const results = await Promise.all(iterable);
    /// for (const foo of set);
    /// function * foo() {
    /// 	yield * anotherGenerator();
    /// }
    /// function foo(bar) {
    /// 	return bar.map(x => x * 2);
    /// }
    ///
    /// ```
    NoUselessSpread,
    correctness
);

impl Rule for NoUselessSpread {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        check_useless_spread_in_list(node, ctx);

        if let AstKind::ArrayExpression(array_expr) = node.kind() {
            check_useless_iterable_to_array(node, array_expr, ctx);
            check_useless_array_clone(array_expr, ctx);
        }
    }
}

fn check_useless_spread_in_list<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) {
    if !matches!(node.kind(), AstKind::ArrayExpression(_) | AstKind::ObjectExpression(_)) {
        return;
    }
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return;
    };

    // we're in ...[]
    let AstKind::SpreadElement(spread_elem) = parent.kind() else {
        return;
    };
    let Some(parent_parent) = outermost_paren_parent(parent, ctx) else {
        return;
    };

    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);

    match node.kind() {
        AstKind::ObjectExpression(_) => {
            // { ...{ } }
            if matches!(parent_parent.kind(), AstKind::ObjectExpression(_)) {
                ctx.diagnostic(spread_in_list(span, "object"));
            }
        }
        AstKind::ArrayExpression(array_expr) => match parent_parent.kind() {
            // ...[ ...[] ]
            AstKind::ArrayExpressionElement(_) => {
                let diagnostic = spread_in_list(span, "array");
                if let Some(outer_array) = ctx.nodes().parent_kind(parent_parent.id()) {
                    diagnose_array_in_array_spread(ctx, diagnostic, &outer_array, array_expr);
                } else {
                    ctx.diagnostic(diagnostic);
                }
            }
            // foo(...[ ])
            AstKind::Argument(_) => {
                ctx.diagnostic_with_fix(spread_in_arguments(span), |fixer| {
                    fix_by_removing_spread(fixer, array_expr, spread_elem)
                });
            }
            _ => {}
        },
        _ => {
            unreachable!()
        }
    }
}

/// `...[ ...[] ]`. May contain multiple spread elements.
fn diagnose_array_in_array_spread<'a>(
    ctx: &LintContext<'a>,
    diagnostic: OxcDiagnostic,
    outer_array: &AstKind<'a>,
    inner_array: &ArrayExpression<'a>,
) {
    let AstKind::ArrayExpression(outer_array) = outer_array else {
        ctx.diagnostic(diagnostic);
        return;
    };
    match outer_array.elements.len() {
        0 => unreachable!(),
        1 => {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fix_replace(fixer, &outer_array.span, inner_array)
            });
        }
        _ => {
            // If all elements are array spreads, we can merge them all together
            let mut spreads: Vec<&'a ArrayExpression> = vec![];
            for el in &outer_array.elements {
                let ArrayExpressionElement::SpreadElement(spread) = el else {
                    ctx.diagnostic(diagnostic);
                    return;
                };
                let Expression::ArrayExpression(arr) = &spread.argument else {
                    ctx.diagnostic(diagnostic);
                    return;
                };
                spreads.push(arr.as_ref());
            }

            // [ ...[a, b, c], ...[d, e, f] ] -> [a, b, c, d, e, f]
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                let mut codegen = fixer.codegen();
                codegen.print(b'[');
                let elements =
                    spreads.iter().flat_map(|arr| arr.elements.iter()).collect::<Vec<_>>();
                let n = elements.len();
                for (i, el) in elements.into_iter().enumerate() {
                    codegen.print_expression(el.to_expression());
                    if i < n - 1 {
                        codegen.print(b',');
                        codegen.print_hard_space();
                    }
                }
                codegen.print(b']');
                fixer.replace(outer_array.span, codegen)
            });
        }
    }
}

fn check_useless_iterable_to_array<'a>(
    node: &AstNode<'a>,
    array_expr: &ArrayExpression<'a>,
    ctx: &LintContext<'a>,
) {
    let Some(parent) = outermost_paren_parent(node, ctx) else {
        return;
    };

    if !is_single_array_spread(array_expr) {
        return;
    }

    let ArrayExpressionElement::SpreadElement(spread_elem) = &array_expr.elements[0] else {
        return;
    };

    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);

    let parent = if let AstKind::Argument(_) = parent.kind() {
        let Some(parent) = outermost_paren_parent(parent, ctx) else {
            return;
        };
        parent
    } else {
        parent
    };

    match parent.kind() {
        AstKind::ForOfStatement(for_of_stmt) => {
            if for_of_stmt.right.without_parenthesized().span() == array_expr.span {
                ctx.diagnostic(iterable_to_array_in_for_of(span));
            }
        }
        AstKind::YieldExpression(yield_expr) => {
            if yield_expr.delegate
                && yield_expr.argument.as_ref().is_some_and(|arg| arg.span() == array_expr.span)
            {
                ctx.diagnostic(iterable_to_array_in_yield_star(span));
            }
        }

        AstKind::NewExpression(new_expr) => {
            if !((is_new_expression(
                new_expr,
                &["Map", "WeakMap", "Set", "WeakSet"],
                Some(1),
                Some(1),
            ) || is_new_expression(
                new_expr,
                &[
                    "Int8Array",
                    "Uint8Array",
                    "Uint8ClampedArray",
                    "Int16Array",
                    "Uint16Array",
                    "Int32Array",
                    "Uint32Array",
                    "Float32Array",
                    "Float64Array",
                    "BigInt64Array",
                    "BigUint64Array",
                ],
                Some(1),
                None,
            )) && innermost_paren_arg_span(&new_expr.arguments[0]) == array_expr.span)
            {
                return;
            }
            ctx.diagnostic_with_fix(
                iterable_to_array(span, get_new_expr_ident_name(new_expr).unwrap_or("unknown")),
                |fixer| fix_by_removing_spread(fixer, &new_expr.arguments[0], spread_elem),
            );
        }
        AstKind::CallExpression(call_expr) => {
            if !((is_method_call(
                call_expr,
                Some(&["Promise"]),
                Some(&["all", "allSettled", "any", "race"]),
                Some(1),
                Some(1),
            ) || is_method_call(
                call_expr,
                Some(&[
                    "Array",
                    "Int8Array",
                    "Uint8Array",
                    "Uint8ClampedArray",
                    "Int16Array",
                    "Uint16Array",
                    "Int32Array",
                    "Uint32Array",
                    "Float32Array",
                    "Float64Array",
                    "BigInt64Array",
                    "BigUint64Array",
                ]),
                Some(&["from"]),
                Some(1),
                Some(1),
            ) || is_method_call(
                call_expr,
                Some(&["Object"]),
                Some(&["fromEntries"]),
                Some(1),
                Some(1),
            )) && innermost_paren_arg_span(&call_expr.arguments[0]) == array_expr.span)
            {
                return;
            }

            ctx.diagnostic_with_fix(
                iterable_to_array(
                    span,
                    &get_method_name(call_expr).unwrap_or_else(|| "unknown".into()),
                ),
                |fixer| fix_by_removing_spread(fixer, array_expr, spread_elem),
            );
        }
        _ => {}
    }
}

fn check_useless_array_clone<'a>(array_expr: &ArrayExpression<'a>, ctx: &LintContext<'a>) {
    if !is_single_array_spread(array_expr) {
        return;
    }

    let ArrayExpressionElement::SpreadElement(spread_elem) = &array_expr.elements[0] else {
        return;
    };

    let span = Span::new(spread_elem.span.start, spread_elem.span.start + 3);

    if let Expression::CallExpression(call_expr) = &spread_elem.argument {
        if !(is_method_call(
            call_expr,
            None,
            Some(&[
                "concat",
                "copyWithin",
                "filter",
                "flat",
                "flatMap",
                "map",
                "slice",
                "splice",
                "toReversed",
                "toSorted",
                "toSpliced",
                "with",
            ]),
            None,
            None,
        ) || is_method_call(call_expr, None, Some(&["split"]), None, None)
            || is_method_call(call_expr, Some(&["Object"]), Some(&["keys", "values"]), None, None)
            || is_method_call(call_expr, Some(&["Array"]), Some(&["from", "of"]), None, None))
        {
            return;
        }

        let method = call_expr.callee.get_member_expr().map_or_else(
            || "unknown".into(),
            |method| {
                let object_name = if let Expression::Identifier(ident) = &method.object() {
                    ident.name.as_str()
                } else {
                    "unknown"
                };

                format!("{}.{}", object_name, method.static_property_name().unwrap())
            },
        );

        ctx.diagnostic_with_fix(clone_array(span, &method), |fixer| {
            fix_by_removing_spread(fixer, array_expr, spread_elem)
        });
    }

    if let Expression::AwaitExpression(await_expr) = &spread_elem.argument {
        if let Expression::CallExpression(call_expr) = &await_expr.argument {
            if !is_method_call(
                call_expr,
                Some(&["Promise"]),
                Some(&["all", "allSettled"]),
                Some(1),
                Some(1),
            ) {
                return;
            }
            let method_name =
                call_expr.callee.get_member_expr().unwrap().static_property_name().unwrap();

            ctx.diagnostic_with_fix(
                clone_array(span, &format!("Promise.{method_name}")),
                |fixer| fix_by_removing_spread(fixer, array_expr, spread_elem),
            );
        }
    }
}

fn fix_replace<'a, T: GetSpan, U: GetSpan>(
    fixer: RuleFixer<'_, 'a>,
    target: &T,
    replacement: &U,
) -> Fix<'a> {
    let replacement = fixer.source_range(replacement.span());
    fixer.replace(target.span(), replacement)
}

/// Creates a fix that replaces `[...spread]` with `spread`
fn fix_by_removing_spread<'a, S: GetSpan>(
    fixer: RuleFixer<'_, 'a>,
    iterable: &S,
    spread: &SpreadElement<'a>,
) -> Fix<'a> {
    fixer.replace(iterable.span(), fixer.source_range(spread.argument.span()))
}

/// Checks if `node` is `[...(expr)]`
fn is_single_array_spread(node: &ArrayExpression) -> bool {
    node.elements.len() == 1 && matches!(node.elements[0], ArrayExpressionElement::SpreadElement(_))
}

fn innermost_paren_arg_span(arg: &Argument) -> Span {
    match arg {
        match_expression!(Argument) => arg.to_expression().without_parenthesized().span(),
        Argument::SpreadElement(spread_elem) => spread_elem.argument.span(),
    }
}

fn get_method_name(call_expr: &CallExpression) -> Option<String> {
    let callee = call_expr.callee.get_member_expr()?;

    let object_name = if let Expression::Identifier(ident) = &callee.object() {
        ident.name.as_str()
    } else {
        "unknown"
    };

    Some(format!("{}.{}", object_name, callee.static_property_name().unwrap()))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const array = [[]]",
        r"const array = [{}]",
        r"const object = ({...[]})",
        r"foo([])",
        r"foo({})",
        r"new Foo([])",
        r"new Foo({})",
        r"const array = [...a]",
        r"const object = {...a}",
        r"const [first, ...rest] = []",
        r"const {foo, ...rest} = {}",
        r"function a(foo, ...rest) {}",
        r"new NotMatchedConstructor([...iterable])",
        r"new foo.Map([...iterable])",
        r"new Map([...iterable], extraArgument)",
        r"new Map()",
        r"new Map(...iterable)",
        r"new Map([,...iterable])",
        r"new Map([...iterable, extraElement])",
        r"new Map({...iterable})",
        r"new Uint8Array(...iterable)",
        r"new Uint8Array(before, [...iterable])",
        r"Promise.notMatchedMethod([...iterable])",
        r"NotPromise.all([...iterable])",
        r"foo.Promise.all([...iterable])",
        r"Promise[all]([...iterable])",
        r"Promise.all()",
        r"Promise.all([...iterable], extraArgument)",
        r"Promise.all(...iterable)",
        r"Promise.all([,...iterable])",
        r"Promise.all([...iterable, extraElement])",
        r"Promise.all({...iterable})",
        r"Object.notFromEntries([...iterable])",
        r"NotObject.fromEntries([...iterable])",
        r"Object[fromEntries]([...iterable])",
        r"Object.fromEntries()",
        r"Object.fromEntries([...iterable], extraArgument)",
        r"Object.fromEntries(...iterable)",
        r"Object.fromEntries({...iterable})",
        r"Uint8Array.notFrom([...iterable])",
        r"NotTypedArray.from([...iterable])",
        r"Uint8Array.from([...iterable], extraArgument)",
        r"Uint8Array.from(...iterable)",
        r"Uint8Array.from({...iterable})",
        r"for (const [...iterable] of foo);",
        r"for (const foo of bar) [...iterable];",
        r"for (const foo of [,...iterable]);",
        r"for (const foo of [...iterable, extraElement]);",
        r"for (const foo of {...iterable});",
        r"for (const foo in [...iterable]);",
        r"function * fn() {yield [...iterable];}",
        r"function * fn() {yield* [...iterable, extraElement];}",
        r"function * fn() {yield* {...iterable};}",
        r"[...not.array]",
        r"[...not.array()]",
        r"[...array.unknown()]",
        r"[...Object.notReturningArray(foo)]",
        r"[...NotObject.keys(foo)]",
        r"[...Int8Array.from(foo)]",
        r"[...Int8Array.of()]",
        r"[...new Int8Array(3)]",
        r"[...Promise.all(foo)]",
        r"[...Promise.allSettled(foo)]",
        r"[...await Promise.all(foo, extraArgument)]",
        r"[...new Array(3)]",
    ];

    let fail = vec![
        r"const array = [...[a]]",
        r"const object = {...{a}}",
        r"foo(...[a])",
        r"new Foo(...[a])",
        r"const array = [...[a,]]",
        r"const object = {...{a,}}",
        r"foo(...[a,])",
        r"new Foo(...[a,])",
        r"const array = [...[a,],]",
        r"const object = {...{a,},}",
        r"foo(...[a,],)",
        r"new Foo(...[a,],)",
        r"const array = [...(( [a] ))]",
        r"const object = {...(( {a} ))}",
        r"foo(...(( [a] )))",
        r"new Foo(...(( [a] )))",
        r"const array = [...[]]",
        r"const object = {...{}}",
        r"foo(...[])",
        r"new Foo(...[])",
        r"const array = [...[,]]",
        r"foo(...[,])",
        r"new Foo(...[,])",
        r"const array = [...[,,]]",
        r"foo(...[,,])",
        r"new Foo(...[,,])",
        r"const array = [...[a, , b,]]",
        r"foo(...[a, , b,])",
        r"new Foo(...[a, , b,])",
        r"const array = [...[a, , b,],]",
        r"foo(...[a, , b,],)",
        r"new Foo(...[a, , b,],)",
        r"foo(...[,, ,(( a )), ,,(0, b), ,,])",
        r"const array = [a, ...[a, b]]",
        r"const object = {a, ...{a, b}}",
        r"foo(a, ...[a, b])",
        r"new Foo(a, ...[a, b])",
        r"const array = [...[a, b], b,]",
        r"const object = {...{a, b}, b,}",
        r"foo(...[a, b], b,)",
        r"new Foo(...[a, b], b,)",
        r"const array = [a, ...[a, b], b,]",
        r"const object = {a, ...{a, b}, b,}",
        r"foo(a, ...[a, b], b,)",
        r"new Foo(a, ...[a, b], b,)",
        r"({a:1, ...{a: 2}})",
        r"({...{a:1}, ...{a: 2}})",
        r"({[a]:1, ...{[a]: 2}})",
        r"Promise.all(...[...iterable])",
        r"new Map(...[...iterable])",
        r"const map = new Map([...iterable])",
        r"const weakMap = new WeakMap([...iterable])",
        r"const set = new Set([...iterable])",
        r"const weakSet = new WeakSet([...iterable])",
        r"const typed = new BigUint64Array([...iterable], byteOffset, length)",
        r"const typed = new BigUint64Array([...iterable], ...args)",
        r"const promise = Promise.all([...iterable])",
        r"const promise = Promise.allSettled([...iterable])",
        r"const promise = Promise.any([...iterable])",
        r"const promise = Promise.race([...iterable])",
        r"const array = Array.from([...iterable])",
        r"const typed = BigUint64Array.from([...iterable])",
        r"const object = Object.fromEntries([...iterable])",
        r"for (const foo of [...iterable]);",
        r"async () => {for await (const foo of [...iterable]);}",
        r"const map = new Map([...iterable,])",
        r"for (const foo of [...iterable]);",
        r"const map = new Map([...iterable,],)",
        r"const map = new Map([...(( iterable ))])",
        r"for (const foo of [...(( iterable ))]);",
        r"const map = new Map((( [...(( iterable ))] )))",
        r"for (const foo of (( [...(( iterable ))] )));",
        r"for (const foo of [...iterable]);",
        r"[...foo.concat(bar)]",
        r"[...foo.copyWithin(-2)]",
        r"[...foo.filter(bar)]",
        r"[...foo.flat()]",
        r"[...foo.flatMap(bar)]",
        r"[...foo.map(bar)]",
        r"[...foo.slice(1)]",
        r"[...foo.splice(1)]",
        r"[...foo.toReversed()]",
        r"[...foo.toSorted()]",
        r"[...foo.toSpliced(0, 1)]",
        r"[...foo.with(0, bar)]",
        r#"[...foo.split("|")]"#,
        r"[...Object.keys(foo)]",
        r"[...Object.values(foo)]",
        r"[...Array.from(foo)]",
        r"[...Array.of()]",
        r"[...await Promise.all(foo)]",
        r"[...await Promise.allSettled(foo)]",
        r"for (const foo of[...iterable]);",
        r"for (const foo of[...iterable2]);",
        // https://github.com/getsentry/sentry/blob/9e4359030e7ec088aa3f47582f1afbad539a6377/static/app/views/performance/database/useAvailableDurationAggregates.tsx#L15-L17
        r"
        if (organization.features?.includes('performance-database-view-percentiles')) {
            availableAggregates.push(...['p50', 'p75', 'p95', 'p99']);
        }
        ",
    ];

    let fix = vec![
        ("[...[1,2,3]]", "[1,2,3]"),
        ("[...[1,2,3], ...[4,5,6]]", "[1, 2, 3, 4, 5, 6]"),
        ("[...[1,2,3], ...x]", "[...[1,2,3], ...x]"),
        ("[...[...[1,2,3]]]", "[...[1,2,3]]"),
        ("[...foo.map(x => !!x)]", "foo.map(x => !!x)"),
        (r"[...await Promise.all(foo)]", r"await Promise.all(foo)"),
        (r"const promise = Promise.any([...iterable])", r"const promise = Promise.any(iterable)"),
        (r"[...Array.from(iterable)]", r"Array.from(iterable)"),
        (r"new Map([...iterable])", r"new Map(iterable)"),
        (r"new Map([ ...((iterable)) ])", r"new Map(((iterable)))"),
        // (r"new Map(...[...iterable])", r"new Map(iterable)"),
    ];
    Tester::new(NoUselessSpread::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
