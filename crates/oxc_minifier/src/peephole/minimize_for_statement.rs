use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ctx::Ctx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// `mangleFor`: <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_ast/js_parser.go#L9801>
    pub fn minimize_for_statement(&mut self, for_stmt: &mut ForStatement<'a>, ctx: Ctx<'a, '_>) {
        // Get the first statement in the loop
        let mut first = &for_stmt.body;
        if let Statement::BlockStatement(block_stmt) = first {
            if let Some(b) = block_stmt.body.get(0) {
                first = b;
            } else {
                return;
            }
        };

        let Statement::IfStatement(if_stmt) = first else {
            return;
        };
        // "for (;;) if (x) break;" => "for (; !x;) ;"
        // "for (; a;) if (x) break;" => "for (; a && !x;) ;"
        // "for (;;) if (x) break; else y();" => "for (; !x;) y();"
        // "for (; a;) if (x) break; else y();" => "for (; a && !x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) = if_stmt.consequent.get_one_child() {
            if break_stmt.label.is_some() {
                return;
            }

            let for_stmt_body = ctx.ast.move_statement(&mut for_stmt.body);

            let expr = match ctx.ast.move_expression(&mut if_stmt.test) {
                Expression::UnaryExpression(unary_expr) if unary_expr.operator.is_not() => {
                    unary_expr.unbox().argument
                }
                e => Self::minimize_not(e.span(), e, ctx),
            };
            if let Some(test) = &mut for_stmt.test {
                let e = ctx.ast.move_expression(test);
                *test = ctx.ast.expression_logical(test.span(), e, LogicalOperator::And, expr);
            } else {
                for_stmt.test = Some(expr);
            }
            for_stmt.body = Self::drop_first_statement(body, if_stmt.alternate.take(), ctx);
            // if_stmt.alternate.take().unwrap_or_else(|| ctx.ast.statement_empty(if_stmt.span));
            self.mark_current_function_as_changed();
            return;
        }
        // "for (;;) if (x) y(); else break;" => "for (; x;) y();"
        // "for (; a;) if (x) y(); else break;" => "for (; a && x;) y();"
        if let Some(Statement::BreakStatement(break_stmt)) =
            if_stmt.alternate.as_ref().and_then(|stmt| stmt.get_one_child())
        {
            if break_stmt.label.is_some() {
                return;
            }
            let expr = ctx.ast.move_expression(&mut if_stmt.test);
            if let Some(test) = &mut for_stmt.test {
                let e = ctx.ast.move_expression(test);
                *test = ctx.ast.expression_logical(test.span(), e, LogicalOperator::And, expr);
            } else {
                for_stmt.test = Some(expr);
            }
            for_stmt.body = ctx.ast.move_statement(&mut if_stmt.consequent);
            self.mark_current_function_as_changed();
        }
    }

    fn drop_first_statement(
        body: Statement<'a>,
        replace: Option<Statement<'a>>,
        ctx: Ctx<'a, '_>,
    ) -> Statement<'a> {
        let span = body.span();
        match body {
            Statement::BlockStatement(mut block_stmt) if !block_stmt.body.is_empty() => {
                if let Some(replace) = replace {
                    block_stmt.body[0] = replace;
                } else if block_stmt.body.len() == 2
                    && !Self::statement_cares_about_scope(&block_stmt.body[1])
                {
                    return ctx.ast.move_statement(&mut block_stmt.body[1]);
                } else {
                    block_stmt.body.remove(0);
                }
                Statement::BlockStatement(block_stmt)
            }
            _ => replace.unwrap_or_else(|| ctx.ast.statement_empty(span)),
        }
    }
}
