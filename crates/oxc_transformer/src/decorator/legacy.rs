//! Typescript Experimental Decorator

use std::mem;

use oxc_allocator::{Address, CloneIn, GetAddress, Vec as ArenaVec};
use oxc_ast::{ast::*, NONE};
use oxc_semantic::ReferenceFlags;
use oxc_span::SPAN;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};
use oxc_traverse::{Ancestor, BoundIdentifier, Traverse, TraverseCtx};

use crate::{Helper, TransformCtx};

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
    fn transform_class(&self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        let (class_or_constructor_parameter_is_decorated, child_is_decorated) =
            self.check_class_decorators(class);

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
        &self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
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
        &self,
        class: &mut Class<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) -> ArenaVec<'a, Statement<'a>> {
        let mut decoration_statements = ctx.ast.vec_with_capacity(class.body.body.len());
        let mut static_decoration_statements = Vec::with_capacity(class.body.body.len());

        for element in &mut class.body.body {
            let (is_static, elements) =
                match element {
                    ClassElement::MethodDefinition(method) => {
                        let decoration_count = method.decorators.len();
                        let params = &mut method.value.params;
                        let param_decoration_count =
                            params.items.iter().fold(0, |acc, param| acc + param.decorators.len());
                        let decoration_count = decoration_count + param_decoration_count;

                        if decoration_count == 0 {
                            continue;
                        }

                        let mut elements = ctx.ast.vec_with_capacity(decoration_count);
                        elements.extend(
                            method.decorators.drain(..).map(|decorator| {
                                ArrayExpressionElement::from(decorator.expression)
                            }),
                        );

                        if param_decoration_count > 0 {
                            self.transform_decorators_of_parameters(
                                &mut elements,
                                &mut method.value.params,
                                ctx,
                            );
                        }

                        (method.r#static, elements)
                    }
                    ClassElement::PropertyDefinition(prop) if !prop.decorators.is_empty() => {
                        let elements =
                            ctx.ast.vec_from_iter(prop.decorators.drain(..).map(|decorator| {
                                ArrayExpressionElement::from(decorator.expression)
                            }));
                        (prop.r#static, elements)
                    }
                    _ => {
                        continue;
                    }
                };
            let array = ctx.ast.expression_array(SPAN, elements, None);
            let arguments = ctx.ast.vec1(Argument::from(array));
            let helper = self.ctx.helper_call_expr(Helper::Decorator, SPAN, arguments, ctx);
            let decoration_statement = ctx.ast.statement_expression(SPAN, helper);

            if is_static {
                static_decoration_statements.push(decoration_statement);
            } else {
                decoration_statements.push(decoration_statement);
            }
        }

        decoration_statements.extend(static_decoration_statements);
        decoration_statements
    }

    fn transform_decorators_of_parameters(
        &self,
        elements: &mut ArenaVec<'a, ArrayExpressionElement<'a>>,
        params: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        for (index, param) in &mut params.items.iter_mut().enumerate() {
            if param.decorators.is_empty() {
                continue;
            }
            elements.extend(param.decorators.drain(..).map(|decorator| {
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

    fn check_class_decorators(&self, class: &Class<'a>) -> (bool, bool) {
        let mut class_or_constructor_parameter_is_decorated = !class.decorators.is_empty();
        let mut child_is_decorated = false;

        for element in &class.body.body {
            match element {
                ClassElement::MethodDefinition(method) if method.kind.is_constructor() => {
                    class_or_constructor_parameter_is_decorated |=
                        Self::class_method_parameter_is_decorated(&method.value);
                }
                ClassElement::MethodDefinition(method) => {
                    child_is_decorated |= !method.decorators.is_empty()
                        || Self::class_method_parameter_is_decorated(&method.value);
                }
                ClassElement::PropertyDefinition(prop) => {
                    child_is_decorated |= !prop.decorators.is_empty();
                }
                _ => {}
            }
            if class_or_constructor_parameter_is_decorated {
                break;
            }
        }

        (class_or_constructor_parameter_is_decorated, child_is_decorated)
    }

    fn class_method_parameter_is_decorated(func: &Function<'a>) -> bool {
        func.params.items.iter().any(|param| !param.decorators.is_empty())
    }
}
