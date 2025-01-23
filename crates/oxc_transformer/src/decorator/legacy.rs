//! Typescript Experimental Decorator

use std::mem::swap;

use oxc_allocator::{Address, Box as ArenaBox, GetAddress, Vec as ArenaVec};
use oxc_ast::{ast::*, Visit, VisitMut};
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
        if matches!(stmt, Statement::ClassDeclaration(_)) {
            self.transform_class(stmt, ctx);
        }
    }
}

impl<'a> LegacyDecorators<'a, '_> {
    fn transform_class(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        let Statement::ClassDeclaration(class) = stmt else { unreachable!() };

        let (
            class_or_constructor_parameter_is_decorated,
            child_is_decorated,
            has_private_in_expression_in_decorator,
        ) = self.check_class_decorators(class);

        if class_or_constructor_parameter_is_decorated {
            let Statement::ClassDeclaration(class) = ctx.ast.move_statement(stmt) else {
                unreachable!()
            };
            *stmt = self.transform_class_declaration_with_class_decorators(class, ctx);
        } else if child_is_decorated {
            self.transform_class_declaration_without_class_decorators(class, ctx);
        }
    }

    /// Transforms a decorated class declaration and appends the resulting statements. If
    /// the class requires an alias to avoid issues with double-binding, the alias is returned.
    fn transform_class_declaration_with_class_decorators(
        &mut self,
        mut class: ArenaBox<'a, Class<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
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

        let span = class.span;
        // TODO: In TypeScript, the class binding will keep it as-is, but our implementation will take it,
        // this way we can avoid syncing semantic data, need to check if there is any runtime impact.
        let class_binding =
            class.id.take().map(|ident| BoundIdentifier::from_binding_ident(&ident));
        let class_alias_binding = class_binding.as_ref().and_then(|id| {
            ClassReferenceChanger::new(id.clone(), ctx, self.ctx)
                .get_class_alias_if_needed(&mut class.body)
        });
        let class_binding = class_binding
            .unwrap_or_else(|| ctx.generate_uid_in_current_scope("default", SymbolFlags::Class));

        let constructor_decoration = self.generate_constructor_decoration_expression(
            &mut class,
            &class_binding,
            class_alias_binding.as_ref(),
            ctx,
        );
        let mut stmts =
            self.transform_decorators_of_class_elements(&mut class, &class_binding, ctx);
        stmts.push(constructor_decoration);

        // `class C {}` -> `let C = class {}`
        class.r#type = ClassType::ClassExpression;
        let initializer = Self::get_class_initializer(
            Expression::ClassExpression(class),
            class_alias_binding.as_ref(),
            ctx,
        );
        let declarator = ctx.ast.variable_declarator(
            SPAN,
            VariableDeclarationKind::Let,
            class_binding.create_binding_pattern(ctx),
            Some(initializer),
            false,
        );
        let var_declaration = ctx.ast.declaration_variable(
            span,
            VariableDeclarationKind::Let,
            ctx.ast.vec1(declarator),
            false,
        );
        let statement = Statement::from(var_declaration);

        self.ctx.statement_injector.insert_many_after(&statement, stmts);

        statement
    }

    /// Transforms a non-decorated class declaration.
    fn transform_class_declaration_without_class_decorators(
        &mut self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        // `export default class {}`
        // class.id.map(|ident| BoundIdentifier::from_binding_ident(&ident)).get_or_insert(|| {});
        let class_binding = if let Some(ident) = &class.id {
            BoundIdentifier::from_binding_ident(ident)
        } else {
            let class_binding = ctx.generate_uid_in_current_scope("default", SymbolFlags::Class);
            class.id.replace(class_binding.create_binding_identifier(ctx));
            class_binding
        };

        // If the class declaration
        let statements = self.transform_decorators_of_class_elements(class, &class_binding, ctx);
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
        class_binding: &BoundIdentifier<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        // FIXME: In TypeScript, it will transform non-static class elements and static class elements
        // separately so that static decorators will be inserted last. Check if there is any reason to do this.
        let mut decoration_stmts = ctx.ast.vec_with_capacity(class.body.body.len());

        for element in &mut class.body.body {
            let (is_static, key, descriptor, decorations) = match element {
                ClassElement::MethodDefinition(method) => {
                    let Some(decorations) = self.get_all_decorators_of_class_method(method, ctx)
                    else {
                        continue;
                    };

                    // We emit `null` here to indicate to `__decorate` that it can invoke `Object.getOwnPropertyDescriptor` directly.
                    // We have this extra argument here so that we can inject an explicit property descriptor at a later date.
                    let descriptor = ctx.ast.expression_null_literal(SPAN);

                    (method.r#static, &mut method.key, descriptor, decorations)
                }
                ClassElement::PropertyDefinition(prop) if !prop.decorators.is_empty() => {
                    let decorations = Self::convert_decorators_to_array_expression(
                        prop.decorators.drain(..),
                        ctx,
                    );

                    // We emit `void 0` here to indicate to `__decorate` that it can invoke `Object.defineProperty` directly, but that it
                    // should not invoke `Object.getOwnPropertyDescriptor`.
                    let descriptor = ctx.ast.expression_null_literal(SPAN);

                    (prop.r#static, &mut prop.key, descriptor, decorations)
                }
                _ => {
                    continue;
                }
            };

            // `Class` or `Class.prototype`
            let prefix = Self::get_class_member_prefix(class_binding, is_static, ctx);
            let key = self.get_expression_for_property_name(key, ctx);
            decoration_stmts.push(self.create_decorator(decorations, prefix, key, descriptor, ctx));
        }

        decoration_stmts
    }

    fn get_all_decorators_of_class_method(
        &self,
        method: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Option<Expression<'a>> {
        let params = &mut method.value.params;
        let param_decoration_count =
            params.items.iter().fold(0, |acc, param| acc + param.decorators.len());
        let method_decoration_count = method.decorators.len() + param_decoration_count;

        if method_decoration_count == 0 {
            return None;
        }

        let mut decorations = ctx.ast.vec_with_capacity(method_decoration_count);
        decorations.extend(
            method
                .decorators
                .drain(..)
                .map(|decorator| ArrayExpressionElement::from(decorator.expression)),
        );

        if param_decoration_count > 0 {
            self.transform_decorators_of_parameters(
                &mut decorations,
                &mut method.value.params,
                ctx,
            );
        }

        Some(ctx.ast.expression_array(SPAN, decorations, None))
    }

    #[allow(clippy::cast_precision_loss)]
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
                // (index, decorator)
                let arguments = ctx.ast.vec_from_array([
                    Argument::from(ctx.ast.expression_numeric_literal(
                        SPAN,
                        index as f64,
                        None,
                        NumberBase::Decimal,
                    )),
                    Argument::from(decorator.expression),
                ]);
                // __decoratorParam(index, decorator)
                ArrayExpressionElement::from(self.ctx.helper_call_expr(
                    Helper::DecoratorParam,
                    decorator.span,
                    arguments,
                    ctx,
                ))
            }));
        }
    }

    fn convert_decorators_to_array_expression(
        decorators_iter: impl Iterator<Item = Decorator<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        let decorations = ctx.ast.vec_from_iter(
            decorators_iter.map(|decorator| ArrayExpressionElement::from(decorator.expression)),
        );
        ctx.ast.expression_array(SPAN, decorations, None)
    }

    fn generate_constructor_decoration_expression(
        &self,
        class: &mut Class<'a>,
        class_binding: &BoundIdentifier<'a>,
        class_alias_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let constructor = class.body.body.iter_mut().find_map(|element| match element {
            ClassElement::MethodDefinition(method) if method.kind.is_constructor() => Some(method),
            _ => None,
        });

        let decorations = if let Some(constructor) = constructor {
            // Constructor cannot have decorators, swap decorators of class and constructor to use
            // `get_all_decorators_of_class_method` to get all decorators of the class and constructor params
            swap(&mut class.decorators, &mut constructor.decorators);
            //  constructor.decorators
            self.get_all_decorators_of_class_method(constructor, ctx)
                .expect("At least one decorator")
        } else {
            Self::convert_decorators_to_array_expression(class.decorators.drain(..), ctx)
        };

        let arguments = ctx.ast.vec_from_array([
            Argument::from(decorations),
            Argument::from(class_binding.create_read_expression(ctx)),
        ]);

        let left = class_binding.create_write_target(ctx);
        let right = Self::get_class_initializer(
            self.ctx.helper_call_expr(Helper::Decorator, SPAN, arguments, ctx),
            class_alias_binding,
            ctx,
        );
        let assignment =
            ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, right);
        ctx.ast.statement_expression(SPAN, assignment)
    }

    /// * class_alias_binding is `Some`: `Class = _Class = expr`
    /// * class_alias_binding is `None`: `Class = expr`
    fn get_class_initializer(
        expr: Expression<'a>,
        class_alias_binding: Option<&BoundIdentifier<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Some(class_alias_binding) = class_alias_binding {
            let left = class_alias_binding.create_write_target(ctx);
            ctx.ast.expression_assignment(SPAN, AssignmentOperator::Assign, left, expr)
        } else {
            expr
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
                    ctx.ast.vec_from_iter(literal.quasis.iter().cloned()),
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
        decorations: Expression<'a>,
        prefix: Expression<'a>,
        key: Expression<'a>,
        descriptor: Expression<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> Statement<'a> {
        let arguments = ctx.ast.vec_from_array([
            Argument::from(decorations),
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

/// Visitor to change references to the class to a local alias
/// <https://github.com/microsoft/TypeScript/blob/8da951cbb629b648753454872df4e1754982aef1/src/compiler/transformers/legacyDecorators.ts#L770-L783>
struct ClassReferenceChanger<'a, 'ctx> {
    class_binding: BoundIdentifier<'a>,
    // `Some` if there are references to the class inside the class body
    class_alias_binding: Option<BoundIdentifier<'a>>,
    ctx: &'ctx mut TraverseCtx<'a>,
    transformer_ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ClassReferenceChanger<'a, 'ctx> {
    fn new(
        class_binding: BoundIdentifier<'a>,
        ctx: &'ctx mut TraverseCtx<'a>,
        transformer_ctx: &'ctx TransformCtx<'a>,
    ) -> Self {
        Self { class_binding, class_alias_binding: None, ctx, transformer_ctx }
    }

    fn get_class_alias_if_needed(
        mut self,
        class: &mut ClassBody<'a>,
    ) -> Option<BoundIdentifier<'a>> {
        self.visit_class_body(class);
        self.class_alias_binding
    }
}

impl<'a> VisitMut<'a> for ClassReferenceChanger<'a, '_> {
    #[inline]
    fn visit_identifier_reference(&mut self, ident: &mut IdentifierReference<'a>) {
        if self.is_class_reference(ident) {
            *ident = self.get_alias_ident_reference();
        }
    }
}

impl<'a> ClassReferenceChanger<'a, '_> {
    // Check if the identifier reference is a reference to the class
    fn is_class_reference(&self, ident: &IdentifierReference<'a>) -> bool {
        self.ctx
            .symbols()
            .get_reference(ident.reference_id())
            .symbol_id()
            .is_some_and(|symbol_id| self.class_binding.symbol_id == symbol_id)
    }

    fn get_alias_ident_reference(&mut self) -> IdentifierReference<'a> {
        if self.class_alias_binding.is_none() {
            self.class_alias_binding.replace(
                self.transformer_ctx
                    .var_declarations
                    .create_uid_var(&self.class_binding.name, self.ctx),
            );
        }

        self.class_alias_binding.as_ref().unwrap().create_read_reference(self.ctx)
    }
}
