//! Typescript Experimental Decorator

use oxc_allocator::{Address, GetAddress, Vec as ArenaVec};
use oxc_ast::{ast::*, Visit};
use oxc_semantic::SymbolFlags;
use oxc_span::SPAN;
use oxc_syntax::operator::AssignmentOperator;
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::{utils::ast_builder::create_prototype_member, Helper, TransformCtx};

pub struct LegacyDecorators<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> LegacyDecorators<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a> Traverse<'a> for LegacyDecorators<'a, '_> {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if let Statement::ClassDeclaration(class) = stmt {
            self.transform_class(class, ctx);
        }
    }
}

impl<'a> LegacyDecorators<'a, '_> {
    fn transform_class(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let (
            class_or_constructor_parameter_is_decorated,
            child_is_decorated,
            has_private_in_expression_in_decorator,
        ) = self.check_class_decorators(class);

        // if class_or_constructor_parameter_is_decorated {
        //     self.transform_class_declaration_with_class_decorators(class, ctx);
        // }

        if child_is_decorated {
            self.transform_class_declaration_without_class_decorators(class, ctx);
        }
    }

    /// Transforms a decorated class declaration and appends the resulting statements. If
    /// the class requires an alias to avoid issues with double-binding, the alias is returned.
    fn transform_class_declaration_with_class_decorators(
        &self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // When we emit an ES6 class that has a class decorator, we must tailor the
        // emit to certain specific cases.
        //
        // In the simplest case, we emit the class declaration as a let declaration, and
        // evaluate decorators after the close of the class body:
        //
        //  [Example 1]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  class C {                       | }
        //  }                               | C = __decorate([dec], C);
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export class C {                | }
        //  }                               | C = __decorate([dec], C);
        //                                  | export { C };
        //  ---------------------------------------------------------------------
        //
        // If a class declaration contains a reference to itself *inside* of the class body,
        // this introduces two bindings to the class: One outside of the class body, and one
        // inside of the class body. If we apply decorators as in [Example 1] above, there
        // is the possibility that the decorator `dec` will return a new value for the
        // constructor, which would result in the binding inside of the class no longer
        // pointing to the same reference as the binding outside of the class.
        //
        // As a result, we must instead rewrite all references to the class *inside* of the
        // class body to instead point to a local temporary alias for the class:
        //
        //  [Example 2]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = C_1 = class C {
        //  class C {                       |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = __decorate([dec], C);
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export class C {                |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = __decorate([dec], C);
        //                                  | export { C };
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //
        // If a class declaration is the default export of a module, we instead emit
        // the export after the decorated declaration:
        //
        //  [Example 3]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let default_1 = class {
        //  export default class {          | }
        //  }                               | default_1 = __decorate([dec], default_1);
        //                                  | export default default_1;
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export default class C {        | }
        //  }                               | C = __decorate([dec], C);
        //                                  | export default C;
        //  ---------------------------------------------------------------------
        //
        // If the class declaration is the default export and a reference to itself
        // inside of the class body, we must emit both an alias for the class *and*
        // move the export after the declaration:
        //
        //  [Example 4]
        //  ---------------------------------------------------------------------
        //  TypeScript                      | Javascript
        //  ---------------------------------------------------------------------
        //  @dec                            | let C = class C {
        //  export default class C {        |   static x() { return C_1.y; }
        //    static x() { return C.y; }    | }
        //    static y = 1;                 | C.y = 1;
        //  }                               | C = C_1 = __decorate([dec], C);
        //                                  | export default C;
        //                                  | var C_1;
        //  ---------------------------------------------------------------------
        //
    }

    /// Transforms a non-decorated class declaration.
    fn transform_class_declaration_without_class_decorators(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `export default class {}`
        if class.id.is_none() {
            ctx.generate_uid_in_current_scope("default", SymbolFlags::Class);
        }

        // If the class declaration
        let statements = self.transform_decorators_of_class_elements(class, ctx);
        // Insert statements after class
        let stmt_address = match ctx.parent() {
            parent @ (Ancestor::ExportDefaultDeclarationDeclaration(_)
            | Ancestor::ExportNamedDeclarationDeclaration(_)) => parent.address(),
            // `Class` is always stored in a `Box`, so has a stable memory location
            _ => Address::from_ptr(class),
        };
        self.ctx.statement_injector.insert_many_after(&stmt_address, statements);
    }

    fn transform_decorators_of_class_elements(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        // FIXME: In TypeScript, it will transform non-static class elements and static class elements
        // separately so that static decorators will be inserted last. Check if there is any reason to do this.
        let mut decoration_stmts = ctx.ast.vec_with_capacity(class.body.body.len());
        let class_binding = BoundIdentifier::from_binding_ident(class.id.as_ref().unwrap());

        for element in &mut class.body.body {
            let (is_static, key, descriptor, decorations) =
                match element {
                    ClassElement::MethodDefinition(method) => {
                        let params = &mut method.value.params;
                        let param_decoration_count =
                            params.items.iter().fold(0, |acc, param| acc + param.decorators.len());
                        let decoration_count = method.decorators.len() + param_decoration_count;

                        if decoration_count == 0 {
                            continue;
                        }

                        let mut decorations = ctx.ast.vec_with_capacity(decoration_count);
                        decorations.extend(
                            method.decorators.drain(..).map(|decorator| {
                                ArrayExpressionElement::from(decorator.expression)
                            }),
                        );

                        if param_decoration_count > 0 {
                            self.transform_decorators_of_parameters(
                                &mut decorations,
                                &mut method.value.params,
                                ctx,
                            );
                        }

                        // We emit `null` here to indicate to `__decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
                        // We have this extra argument here so that we can inject an explicit property descriptor at a later date.
                        let descriptor = ctx.ast.expression_null_literal(SPAN);

                        (method.r#static, &mut method.key, descriptor, decorations)
                    }
                    ClassElement::PropertyDefinition(prop) if !prop.decorators.is_empty() => {
                        let decorations =
                            ctx.ast.vec_from_iter(prop.decorators.drain(..).map(|decorator| {
                                ArrayExpressionElement::from(decorator.expression)
                            }));

                        // We emit `void 0` here to indicate to `__decorate` that it can invoke `Object.defineProperty` directly, but that it
                        // should not invoke `Object.getOwnPropertyDescriptor`.
                        let descriptor = ctx.ast.expression_null_literal(SPAN);

                        (prop.r#static, &mut prop.key, descriptor, decorations)
                    }
                    _ => {
                        continue;
                    }
                };

            let decorations = ctx.ast.expression_array(SPAN, decorations, None);
            // `Class` or `Class.prototype`
            let prefix = Self::get_class_member_prefix(&class_binding, is_static, ctx);
            let key = self.get_expression_for_property_name(key, ctx);
            decoration_stmts.push(self.create_decorator(decorations, prefix, key, descriptor, ctx));
        }

        decoration_stmts
    }

    fn transform_decorators_of_parameters(
        &self,
        decorations: &mut ArenaVec<'a, ArrayExpressionElement<'a>>,
        params: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for (index, param) in &mut params.items.iter_mut().enumerate() {
            if param.decorators.is_empty() {
                continue;
            }
            decorations.extend(param.decorators.drain(..).map(|decorator| {
                let arguments = ctx.ast.vec_from_array([
                    Argument::from(ctx.ast.expression_numeric_literal(
                        SPAN,
                        index as f64,
                        None,
                        NumberBase::Decimal,
                    )),
                    Argument::from(decorator.expression),
                ]);
                ArrayExpressionElement::from(self.ctx.helper_call_expr(
                    Helper::DecoratorParam,
                    decorator.span,
                    arguments,
                    ctx,
                ))
            }));
        }
    }

    fn check_class_decorators(&self, class: &Class<'a>) -> (bool, bool, bool) {
        let mut class_or_constructor_parameter_is_decorated = !class.decorators.is_empty();
        let mut child_is_decorated = false;
        let mut has_private_in_expression_in_decorator = false;

        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) if method.kind.is_constructor() => {
                    class_or_constructor_parameter_is_decorated |=
                        Self::class_method_parameter_is_decorated(&method.value);

                    if class_or_constructor_parameter_is_decorated
                        && !has_private_in_expression_in_decorator
                    {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression_in_method_decorator(method);
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    child_is_decorated |= !method.decorators.is_empty()
                        || Self::class_method_parameter_is_decorated(&method.value);

                    if child_is_decorated && !has_private_in_expression_in_decorator {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression_in_method_decorator(method);
                    }
                }
                ClassElement::PropertyDefinition(prop) => {
                    child_is_decorated |= !prop.decorators.is_empty();

                    if child_is_decorated && !has_private_in_expression_in_decorator {
                        has_private_in_expression_in_decorator =
                            PrivateInExpressionDetector::has_private_in_expression(
                                &prop.decorators,
                            );
                    }
                }
                _ => {}
            }
        }

        (
            class_or_constructor_parameter_is_decorated,
            child_is_decorated,
            has_private_in_expression_in_decorator,
        )
    }

    fn class_method_parameter_is_decorated(func: &Function<'a>) -> bool {
        func.params.items.iter().any(|param| !param.decorators.is_empty())
    }

    /// * is_static is `true`: `Class`
    /// * is_static is `false`: `Class.prototype`
    fn get_class_member_prefix(
        class_binding: &BoundIdentifier<'a>,
        is_static: bool,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let ident = class_binding.create_read_expression(ctx);
        if is_static {
            ident
        } else {
            create_prototype_member(ident, ctx)
        }
    }

    fn get_expression_for_property_name(
        &mut self,
        key: &mut PropertyKey<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        match key {
            PropertyKey::StaticIdentifier(ident) => {
                ctx.ast.expression_string_literal(SPAN, ident.name, None)
            }
            // Legacy decorators do not support private properties/methods
            PropertyKey::PrivateIdentifier(_) => ctx.ast.expression_string_literal(SPAN, "", None),
            // Copiable literals
            PropertyKey::NumericLiteral(literal) => {
                Expression::NumericLiteral(ctx.ast.alloc(literal.clone()))
            }
            PropertyKey::StringLiteral(literal) => {
                Expression::StringLiteral(ctx.ast.alloc(literal.clone()))
            }
            PropertyKey::TemplateLiteral(literal) if literal.expressions.is_empty() => {
                ctx.ast.expression_template_literal(
                    SPAN,
                    ctx.ast.vec_from_iter(literal.quasis.iter().copied()),
                    ctx.ast.vec(),
                )
            }
            PropertyKey::NullLiteral(_) => ctx.ast.expression_null_literal(SPAN),
            _ => {
                let expr = ctx.ast.move_expression(key.to_expression_mut());
                let binding = self.ctx.var_declarations.create_uid_var_based_on_node(&expr, ctx);

                let left = binding.create_read_write_target(ctx);

                // FIXME: This is a little different from the typescript version, we need to check if this is correct
                //
                // ```js
                // Input:
                // class Test {
                //  static [a()] = 0;
                // }
                //
                // TypeScript Output:
                // let _a;
                // class Test {
                //   static { _a = a(); }
                //   static { this[_a] = 0; }
                // }
                //
                // Our Output:
                // let _a;
                // class Test {
                //   static [_a = a()] = 0;
                // ```

                // binding = expr
                let key_expr =
                    ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, expr);
                *key = PropertyKey::from(key_expr);
                binding.create_read_expression(ctx)
            }
        }
    }

    /// `_decorator([...decorators], Class, "key", descriptor)`
    fn create_decorator(
        &self,
        decoration_elements_array: Expression<'a>,
        prefix: Expression<'a>,
        key: Expression<'a>,
        descriptor: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let arguments = ctx.ast.vec_from_array([
            Argument::from(decoration_elements_array),
            Argument::from(prefix),
            Argument::from(key),
            Argument::from(descriptor),
        ]);
        let helper = self.ctx.helper_call_expr(Helper::Decorator, SPAN, arguments, ctx);
        ctx.ast.statement_expression(SPAN, helper)
    }
}

/// Visitor to detect if a private-in expression is present in a decorator
#[derive(Default)]
struct PrivateInExpressionDetector {
    has_private_in_expression: bool,
}

impl Visit<'_> for PrivateInExpressionDetector {
    fn visit_private_in_expression(&mut self, _it: &PrivateInExpression<'_>) {
        self.has_private_in_expression = true;
    }

    fn visit_decorators(&mut self, decorators: &ArenaVec<'_, Decorator<'_>>) {
        for decorator in decorators {
            self.visit_expression(&decorator.expression);
            // Early exit if a private-in expression is found
            if self.has_private_in_expression {
                break;
            }
        }
    }
}

impl PrivateInExpressionDetector {
    fn has_private_in_expression(decorators: &ArenaVec<'_, Decorator<'_>>) -> bool {
        let mut detector = Self::default();
        detector.visit_decorators(decorators);
        detector.has_private_in_expression
    }

    fn has_private_in_expression_in_method_decorator(method: &MethodDefinition<'_>) -> bool {
        let mut detector = Self::default();
        detector.visit_decorators(&method.decorators);
        if detector.has_private_in_expression {
            return true;
        }
        method.value.params.items.iter().any(|param| {
            detector.visit_decorators(&param.decorators);
            detector.has_private_in_expression
        })
    }
}
