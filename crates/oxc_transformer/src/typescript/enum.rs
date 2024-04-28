use std::{mem, rc::Rc};

use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_syntax::{
    operator::{AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator},
    NumberBase,
};
use rustc_hash::FxHashMap;

use crate::{context::Ctx, utils::is_valid_identifier};

pub struct TypeScriptEnum<'a> {
    ctx: Ctx<'a>,
    enums: FxHashMap<Atom<'a>, FxHashMap<Atom<'a>, ConstantValue>>,
}

impl<'a> TypeScriptEnum<'a> {
    pub fn new(ctx: &Ctx<'a>) -> Self {
        Self { ctx: Rc::clone(ctx), enums: FxHashMap::default() }
    }
    /// ```TypeScript
    /// enum Foo {
    ///   X
    /// }
    /// ```
    /// ```JavaScript
    /// var Foo = ((Foo) => {
    ///   const X = 0; Foo[Foo["X"] = X] = "X";
    ///   return Foo;
    /// })(Foo || {});
    /// ```
    pub fn transform_ts_enum(
        &mut self,
        decl: &Box<'a, TSEnumDeclaration<'a>>,
        is_export: bool,
    ) -> Option<Statement<'a>> {
        if decl.modifiers.contains(ModifierKind::Declare) {
            return None;
        }

        let span = decl.span;
        let ident = decl.id.clone();
        let kind = self.ctx.ast.binding_pattern_identifier(ident);
        let id = self.ctx.ast.binding_pattern(kind, None, false);

        // ((Foo) => {
        let mut params = self.ctx.ast.new_vec_single(self.ctx.ast.formal_parameter(
            SPAN,
            id,
            None,
            false,
            false,
            self.ctx.ast.new_vec(),
        ));

        let params = self.ctx.ast.formal_parameters(
            SPAN,
            FormalParameterKind::ArrowFormalParameters,
            params,
            None,
        );

        // Foo[Foo["X"] = 0] = "X";
        let enum_name = decl.id.name.clone();
        let is_already_declared = self.enums.contains_key(&enum_name);
        let statements = self.transform_ts_enum_members(&decl.members, &enum_name);
        let body = self.ctx.ast.function_body(decl.span, self.ctx.ast.new_vec(), statements);
        let r#type = FunctionType::FunctionExpression;
        let callee = self.ctx.ast.plain_function(r#type, SPAN, None, params, Some(body));
        let callee = Expression::FunctionExpression(callee);

        let arguments = if is_export && !is_already_declared {
            // }({});
            let object_expr = self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
            self.ctx.ast.new_vec_single(Argument::Expression(object_expr))
        } else {
            // }(Foo || {});
            let op = LogicalOperator::Or;
            let left = self
                .ctx
                .ast
                .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
            let right = self.ctx.ast.object_expression(SPAN, self.ctx.ast.new_vec(), None);
            let expression = self.ctx.ast.logical_expression(SPAN, left, op, right);
            self.ctx.ast.new_vec_single(Argument::Expression(expression))
        };

        let call_expression = self.ctx.ast.call_expression(SPAN, callee, arguments, false, None);

        if is_already_declared {
            let op = AssignmentOperator::Assign;
            let left = self.ctx.ast.simple_assignment_target_identifier(IdentifierReference::new(
                SPAN,
                enum_name.clone(),
            ));
            let expr = self.ctx.ast.assignment_expression(SPAN, op, left, call_expression);
            return Some(self.ctx.ast.expression_statement(SPAN, expr));
        }

        let kind =
            if is_export { VariableDeclarationKind::Let } else { VariableDeclarationKind::Var };
        let decls = {
            let mut decls = self.ctx.ast.new_vec();

            let binding_identifier = BindingIdentifier::new(SPAN, enum_name.clone());
            let binding_pattern_kind = self.ctx.ast.binding_pattern_identifier(binding_identifier);
            let binding = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
            let decl =
                self.ctx.ast.variable_declarator(SPAN, kind, binding, Some(call_expression), false);

            decls.push(decl);
            decls
        };
        let variable_declaration =
            self.ctx.ast.variable_declaration(span, kind, decls, Modifiers::empty());
        let variable_declaration = Declaration::VariableDeclaration(variable_declaration);

        if is_export {
            let declaration =
                self.ctx.ast.plain_export_named_declaration_declaration(SPAN, variable_declaration);
            Some(
                self.ctx
                    .ast
                    .module_declaration(ModuleDeclaration::ExportNamedDeclaration(declaration)),
            )
        } else {
            Some(Statement::Declaration(variable_declaration))
        }
    }

    pub fn transform_ts_enum_members(
        &mut self,
        members: &Vec<'a, TSEnumMember<'a>>,
        enum_name: &Atom<'a>,
    ) -> Vec<'a, Statement<'a>> {
        let mut statements = self.ctx.ast.new_vec();
        let mut prev_constant_value = Some(ConstantValue::I64(-1));

        let mut previous_enum_members =
            self.enums.entry(enum_name.clone()).or_insert(FxHashMap::default()).clone();

        let mut prev_member_name: Option<Atom<'a>> = None;

        for member in members.iter() {
            let (member_name, member_span) = match &member.id {
                TSEnumMemberName::Identifier(id) => (&id.name, id.span),
                TSEnumMemberName::StringLiteral(str) => (&str.value, str.span),
                TSEnumMemberName::ComputedPropertyName(..)
                | TSEnumMemberName::NumericLiteral(..) => unreachable!(),
            };

            let init = if let Some(initializer) = member.initializer.as_ref() {
                let constant_value =
                    self.computed_constant_value(initializer, &previous_enum_members);

                // prev_constant_value = constant_value
                let init = match constant_value {
                    None => {
                        prev_constant_value = None;
                        self.ctx.ast.copy(initializer)
                    }
                    Some(constant_value) => {
                        previous_enum_members.insert(member_name.clone(), constant_value.clone());
                        match constant_value {
                            ConstantValue::F64(v) => {
                                prev_constant_value = Some(ConstantValue::F64(v));
                                self.get_numeric_literal_expression_f64(v)
                            }
                            ConstantValue::I64(v) => {
                                prev_constant_value = Some(ConstantValue::I64(v));
                                self.get_numeric_literal_expression_i64(v)
                            }
                            ConstantValue::String(str) => {
                                prev_constant_value = None;
                                self.ctx.ast.literal_string_expression(StringLiteral {
                                    span: SPAN,
                                    value: self.ctx.ast.new_atom(&str),
                                })
                            }
                            ConstantValue::Numberic((_, value)) => {
                                prev_constant_value = Some(ConstantValue::F64(value));
                                self.get_numeric_literal_expression_f64(value)
                            }
                            ConstantValue::Identifier(ident) => {
                                prev_constant_value = None;
                                self.ctx.ast.identifier_reference_expression(
                                    IdentifierReference::new(SPAN, self.ctx.ast.new_atom(&ident)),
                                )
                            }
                        }
                    }
                };

                init
            } else if let Some(value) = prev_constant_value {
                match value {
                    ConstantValue::I64(value) => {
                        let value = value + 1;
                        let constant_value = ConstantValue::I64(value);
                        prev_constant_value = Some(constant_value.clone());
                        previous_enum_members.insert(member_name.clone(), constant_value);
                        self.get_numeric_literal_expression_i64(value)
                    }
                    ConstantValue::F64(value) => {
                        let value = value + 1.0;
                        let constant_value = ConstantValue::F64(value);
                        prev_constant_value = Some(constant_value.clone());
                        previous_enum_members.insert(member_name.clone(), constant_value);
                        self.get_numeric_literal_expression_f64(value)
                    }
                    _ => {
                        unreachable!()
                    }
                }
            } else if let Some(prev_member_name) = prev_member_name {
                let self_ref = {
                    let obj = self.ctx.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, enum_name.clone()),
                    );
                    let expr = self
                        .ctx
                        .ast
                        .literal_string_expression(StringLiteral::new(SPAN, prev_member_name));
                    self.ctx.ast.computed_member_expression(SPAN, obj, expr, false)
                };

                // 1 + Foo["x"]
                let one = self.ctx.ast.literal_number_expression(NumericLiteral {
                    span: SPAN,
                    value: 1.0,
                    raw: "1",
                    base: NumberBase::Decimal,
                });

                self.ctx.ast.binary_expression(SPAN, one, BinaryOperator::Addition, self_ref)
            } else {
                self.ctx.ast.literal_number_expression(NumericLiteral {
                    span: SPAN,
                    value: 0.0,
                    raw: "0",
                    base: NumberBase::Decimal,
                })
            };

            let is_str = init.is_string_literal();

            // Foo["x"] = init
            let member_expr = {
                let obj = self.ctx.ast.identifier_reference_expression(IdentifierReference::new(
                    SPAN,
                    enum_name.clone(),
                ));
                let expr = self
                    .ctx
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));

                self.ctx.ast.computed_member(SPAN, obj, expr, false)
            };
            let left = self.ctx.ast.simple_assignment_target_member_expression(member_expr);
            let mut expr =
                self.ctx.ast.assignment_expression(SPAN, AssignmentOperator::Assign, left, init);

            // Foo[Foo["x"] = init] = "x"
            if !is_str {
                let member_expr = {
                    let obj = self.ctx.ast.identifier_reference_expression(
                        IdentifierReference::new(SPAN, enum_name.clone()),
                    );
                    self.ctx.ast.computed_member(SPAN, obj, expr, false)
                };
                let left = self.ctx.ast.simple_assignment_target_member_expression(member_expr);
                let right = self
                    .ctx
                    .ast
                    .literal_string_expression(StringLiteral::new(SPAN, member_name.clone()));
                expr = self.ctx.ast.assignment_expression(
                    SPAN,
                    AssignmentOperator::Assign,
                    left,
                    right,
                );
            }

            prev_member_name = Some(member_name.clone());
            statements.push(self.ctx.ast.expression_statement(member.span, expr));
        }

        self.enums.insert(enum_name.clone(), previous_enum_members.clone());

        let enum_ref = self
            .ctx
            .ast
            .identifier_reference_expression(IdentifierReference::new(SPAN, enum_name.clone()));
        // return Foo;
        let return_stmt = self.ctx.ast.return_statement(SPAN, Some(enum_ref));
        statements.push(return_stmt);

        statements
    }

    fn get_numeric_literal_expression_f64(&self, value: f64) -> Expression<'a> {
        let is_negative = value < 0.0;
        let value = if is_negative { -value } else { value };

        let expr = self.ctx.ast.literal_number_expression(NumericLiteral {
            span: SPAN,
            value,
            raw: self.ctx.ast.new_str(&value.to_string()),
            base: NumberBase::Decimal,
        });

        if is_negative {
            self.ctx.ast.unary_expression(SPAN, UnaryOperator::UnaryNegation, expr)
        } else {
            expr
        }
    }

    fn get_numeric_literal_expression_i64(&self, value: i64) -> Expression<'a> {
        let is_negative = value < 0;
        let value = if is_negative { -value } else { value };
        let expr = self.ctx.ast.literal_number_expression(NumericLiteral {
            span: SPAN,
            value: value as f64,
            raw: self.ctx.ast.new_str(&value.to_string()),
            base: NumberBase::Decimal,
        });
        if is_negative {
            self.ctx.ast.unary_expression(SPAN, UnaryOperator::UnaryNegation, expr)
        } else {
            expr
        }
    }
}

// // Based on the TypeScript repository's `computeConstantValue` in `checker.ts`.
// function computeConstantValue(
//   path: NodePath,
//   prevMembers?: PreviousEnumMembers,
//   seen: Set<t.Identifier> = new Set(),
// ): number | string | undefined {
//   return evaluate(path);

//   function evaluate(path: NodePath): number | string | undefined {
//     const expr = path.node;
//     switch (expr.type) {
//       case "MemberExpression":z
//         return evaluateRef(path, prevMembers, seen);
//       case "StringLiteral":
//         return expr.value;
//       case "UnaryExpression":
//         return evalUnaryExpression(path as NodePath<t.UnaryExpression>);
//       case "BinaryExpression":
//         return evalBinaryExpression(path as NodePath<t.BinaryExpression>);
//       case "NumericLiteral":
//         return expr.value;
//       case "ParenthesizedExpression":
//         return evaluate(path.get("expression"));
//       case "Identifier":
//         return evaluateRef(path, prevMembers, seen);
//       case "TemplateLiteral": {
//         if (expr.quasis.length === 1) {
//           return expr.quasis[0].value.cooked;
//         }

//         const paths = (path as NodePath<t.TemplateLiteral>).get("expressions");
//         const quasis = expr.quasis;
//         let str = "";

//         for (let i = 0; i < quasis.length; i++) {
//           str += quasis[i].value.cooked;

//           if (i + 1 < quasis.length) {
//             const value = evaluateRef(paths[i], prevMembers, seen);
//             if (value === undefined) return undefined;
//             str += value;
//           }
//         }
//         return str;
//       }
//       default:
//         return undefined;
//     }
//   }

//   function evaluateRef(
//     path: NodePath,
//     prevMembers: PreviousEnumMembers,
//     seen: Set<t.Identifier>,
//   ): number | string | undefined {
//     if (path.isMemberExpression()) {
//       const expr = path.node;

//       const obj = expr.object;
//       const prop = expr.property;
//       if (
//         !t.isIdentifier(obj) ||
//         (expr.computed ? !t.isStringLiteral(prop) : !t.isIdentifier(prop))
//       ) {
//         return;
//       }
//       const bindingIdentifier = path.scope.getBindingIdentifier(obj.name);
//       const data = ENUMS.get(bindingIdentifier);
//       if (!data) return;
//       // @ts-expect-error checked above
//       return data.get(prop.computed ? prop.value : prop.name);
//     } else if (path.isIdentifier()) {
//       const name = path.node.name;

//       if (["Infinity", "NaN"].includes(name)) {
//         return Number(name);
//       }

//       let value = prevMembers?.get(name);
//       if (value !== undefined) {
//         return value;
//       }

//       if (seen.has(path.node)) return;
//       seen.add(path.node);

//       value = computeConstantValue(path.resolve(), prevMembers, seen);
//       prevMembers?.set(name, value);
//       return value;
//     }
//   }

//   function evalUnaryExpression(
//     path: NodePath<t.UnaryExpression>,
//   ): number | string | undefined {
//     const value = evaluate(path.get("argument"));
//     if (value === undefined) {
//       return undefined;
//     }

//     switch (path.node.operator) {
//       case "+":
//         return value;
//       case "-":
//         return -value;
//       case "~":
//         return ~value;
//       default:
//         return undefined;
//     }
//   }

//   function evalBinaryExpression(
//     path: NodePath<t.BinaryExpression>,
//   ): number | string | undefined {
//     const left = evaluate(path.get("left")) as any;
//     if (left === undefined) {
//       return undefined;
//     }
//     const right = evaluate(path.get("right")) as any;
//     if (right === undefined) {
//       return undefined;
//     }

//     switch (path.node.operator) {
//       case "|":
//         return left | right;
//       case "&":
//         return left & right;
//       case ">>":
//         return left >> right;
//       case ">>>":
//         return left >>> right;
//       case "<<":
//         return left << right;
//       case "^":
//         return left ^ right;
//       case "*":
//         return left * right;
//       case "/":
//         return left / right;
//       case "+":
//         return left + right;
//       case "-":
//         return left - right;
//       case "%":
//         return left % right;
//       case "**":
//         return left ** right;
//       default:
//         return undefined;
//     }
//   }
// }
//

#[derive(Debug, Clone)]
enum ConstantValue {
    Numberic((String, f64)),
    I64(i64),
    F64(f64),
    Identifier(String),
    String(String),
}

impl<'a> TypeScriptEnum<'a> {
    fn computed_constant_value(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
        // seen: &mut HashSet<Identifier>,
    ) -> Option<ConstantValue> {
        self.evaluate(&expr, prev_members)
    }

    fn evalaute_ref(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            Expression::MemberExpression(expr) => {
                let Expression::Identifier(ident) = expr.object() else { return None };
                let Some(members) = self.enums.get(&ident.name) else { return None };
                let property = expr.static_property_name()?;
                return members.get(property).cloned();
            }
            Expression::Identifier(ident) => {
                if ident.name == "Infinity" || ident.name == "NaN" {
                    return Some(ConstantValue::Identifier(ident.name.to_string()));
                }

                if let Some(value) = prev_members.get(&ident.name) {
                    return Some(value.clone());
                }

                None
            }
            _ => None,
        }
    }

    fn evaluate(
        &self,
        expr: &Expression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        match expr {
            Expression::MemberExpression(member_expr) => self.evalaute_ref(expr, &prev_members),
            Expression::Identifier(ident) => self.evalaute_ref(expr, &prev_members),
            Expression::BinaryExpression(expr) => self.eval_binary_expression(expr, &prev_members),
            Expression::UnaryExpression(expr) => self.eval_unary_expression(expr, &prev_members),
            Expression::NumericLiteral(lit) => {
                Some(ConstantValue::Numberic((lit.raw.to_string(), lit.value)))
            }
            Expression::StringLiteral(lit) => Some(ConstantValue::String(lit.value.to_string())),
            Expression::TemplateLiteral(lit) => {
                let mut value = String::new();
                for part in &lit.quasis {
                    value.push_str(&part.value.raw);
                }
                Some(ConstantValue::String(value))
            }
            _ => None,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn eval_binary_expression(
        &self,
        expr: &BinaryExpression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let left = self.evaluate(&expr.left, prev_members)?;
        let right = self.evaluate(&expr.right, prev_members)?;

        if matches!(expr.operator, BinaryOperator::Addition)
            && (matches!(left, ConstantValue::String(_))
                || matches!(right, ConstantValue::String(_)))
        {
            let left_string = match left {
                ConstantValue::Numberic(left) => left.0.to_string(),
                ConstantValue::String(left) => left,
                ConstantValue::I64(v) => v.to_string(),
                ConstantValue::F64(v) => v.to_string(),
                ConstantValue::Identifier(_) => unreachable!(),
            };

            let right_string = match right {
                ConstantValue::Numberic(right) => right.0.to_string(),
                ConstantValue::String(right) => right,
                ConstantValue::I64(v) => v.to_string(),
                ConstantValue::F64(v) => v.to_string(),
                ConstantValue::Identifier(_) => unreachable!(),
            };

            return Some(ConstantValue::String(format!("{left_string}{right_string}")));
        }

        let left = match left {
            ConstantValue::Numberic(left) => left.1,
            ConstantValue::String(_) => return None,
            ConstantValue::I64(v) => v as f64,
            ConstantValue::F64(v) => v,
            ConstantValue::Identifier(ident) => return None,
        };

        let right = match right {
            ConstantValue::Numberic(right) => right.1,
            ConstantValue::String(_) => return None,
            ConstantValue::I64(v) => v as f64,
            ConstantValue::F64(v) => v,
            ConstantValue::Identifier(ident) => return None,
        };

        match expr.operator {
            BinaryOperator::BitwiseOR => Some(ConstantValue::I64(left as i64 | right as i64)),
            BinaryOperator::BitwiseAnd => Some(ConstantValue::I64(left as i64 & right as i64)),
            BinaryOperator::ShiftRight => Some(ConstantValue::I64(left as i64 >> right as i64)),
            BinaryOperator::ShiftRightZeroFill => {
                Some(ConstantValue::I64(left as i64 >> right as i64))
            }
            BinaryOperator::ShiftLeft => Some(ConstantValue::I64((left as i64) << right as i64)),
            BinaryOperator::BitwiseXOR => Some(ConstantValue::I64(left as i64 ^ right as i64)),
            BinaryOperator::Multiplication => Some(ConstantValue::F64(left * right)),
            BinaryOperator::Division => Some(ConstantValue::F64(left / right)),
            BinaryOperator::Addition => Some(ConstantValue::F64(left + right)),
            BinaryOperator::Subtraction => Some(ConstantValue::F64(left - right)),
            BinaryOperator::Remainder => Some(ConstantValue::F64(left % right)),
            BinaryOperator::Exponential => Some(ConstantValue::F64(left.powf(right))),
            _ => None,
        }
    }

    fn eval_unary_expression(
        &self,
        expr: &UnaryExpression<'a>,
        prev_members: &FxHashMap<Atom<'a>, ConstantValue>,
    ) -> Option<ConstantValue> {
        let value = self.evaluate(&expr.argument, &prev_members)?;

        let value = match value.clone() {
            ConstantValue::Numberic((_raw, value)) => value,
            ConstantValue::I64(value) => value as f64,
            ConstantValue::F64(value) => value,
            ConstantValue::String(value) => unreachable!(),
            ConstantValue::Identifier(ident) => return Some(value),
        };

        match expr.operator {
            UnaryOperator::UnaryPlus => Some(ConstantValue::F64(value)),
            UnaryOperator::UnaryNegation => Some(ConstantValue::F64(-value)),
            UnaryOperator::LogicalNot => Some(ConstantValue::I64(!(value as i64))),
            UnaryOperator::BitwiseNot => Some(ConstantValue::I64(!(value as i64))),
            _ => None,
        }
    }
}
