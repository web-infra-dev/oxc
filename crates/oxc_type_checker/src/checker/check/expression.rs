//! `checkExpression` and its subfunctions.

#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;
use oxc_syntax::types::TypeId;

use crate::ast::PseudoBigInt;

use super::{Check, CheckContext, Checker};

impl<'a> Check<'a> for Expression<'a> {
    /// ```typescript
    /// function checkExpression(node: Expression | QualifiedName, checkMode?: CheckMode, forceTuple?: boolean): Type {
    ///     tracing?.push(tracing.Phase.Check, "checkExpression", { kind: node.kind, pos: node.pos, end: node.end, path: (node as TracingNode).tracingPath });
    ///     const saveCurrentNode = currentNode;
    ///     currentNode = node;
    ///     instantiationCount = 0;
    ///     const uninstantiatedType = checkExpressionWorker(node, checkMode, forceTuple);
    ///     const type = instantiateTypeWithSingleGenericCallSignature(node, uninstantiatedType, checkMode);
    ///     if (isConstEnumObjectType(type)) {
    ///         checkConstEnumAccess(node, type);
    ///     }
    ///     currentNode = saveCurrentNode;
    ///     tracing?.pop();
    ///     return type;
    /// }
    /// ```
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        let uninstantiated_type = checker.check_expression_worker(self, ctx);
        // let ty = checker.instantiate_type_with_single_generic_call_signature(
        //     self,
        //     uninstantiated_type,
        //     &ctx,
        // );
        let ty = uninstantiated_type;

        if checker.is_const_enum_object_type(ty) {
            checker.check_const_enum_access(self, ty, ctx);
        }

        ty
    }
}

impl<'a> Checker<'a> {
    /*
            switch (kind) {
            case SyntaxKind.Identifier:
                return checkIdentifier(node as Identifier, checkMode);
            case SyntaxKind.PrivateIdentifier:
                return checkPrivateIdentifierExpression(node as PrivateIdentifier);
            case SyntaxKind.ThisKeyword:
                return checkThisExpression(node);
            case SyntaxKind.SuperKeyword:
                return checkSuperExpression(node);
            case SyntaxKind.NullKeyword:
                return nullWideningType;
            case SyntaxKind.NoSubstitutionTemplateLiteral:
            case SyntaxKind.StringLiteral:
                return hasSkipDirectInferenceFlag(node) ?
                    blockedStringType :
                    getFreshTypeOfLiteralType(getStringLiteralType((node as StringLiteralLike).text));
            case SyntaxKind.NumericLiteral:
                checkGrammarNumericLiteral(node as NumericLiteral);
                return getFreshTypeOfLiteralType(getNumberLiteralType(+(node as NumericLiteral).text));
            case SyntaxKind.BigIntLiteral:
                checkGrammarBigIntLiteral(node as BigIntLiteral);
                return getFreshTypeOfLiteralType(getBigIntLiteralType({
                    negative: false,
                    base10Value: parsePseudoBigInt((node as BigIntLiteral).text),
                }));
            case SyntaxKind.TrueKeyword:
                return trueType;
            case SyntaxKind.FalseKeyword:
                return falseType;
            case SyntaxKind.TemplateExpression:
                return checkTemplateExpression(node as TemplateExpression);
            case SyntaxKind.RegularExpressionLiteral:
                return checkRegularExpressionLiteral(node as RegularExpressionLiteral);
            case SyntaxKind.ArrayLiteralExpression:
                return checkArrayLiteral(node as ArrayLiteralExpression, checkMode, forceTuple);
            case SyntaxKind.ObjectLiteralExpression:
                return checkObjectLiteral(node as ObjectLiteralExpression, checkMode);
            case SyntaxKind.PropertyAccessExpression:
                return checkPropertyAccessExpression(node as PropertyAccessExpression, checkMode);
            case SyntaxKind.QualifiedName:
                return checkQualifiedName(node as QualifiedName, checkMode);
            case SyntaxKind.ElementAccessExpression:
                return checkIndexedAccess(node as ElementAccessExpression, checkMode);
            case SyntaxKind.CallExpression:
                if ((node as CallExpression).expression.kind === SyntaxKind.ImportKeyword) {
                    return checkImportCallExpression(node as ImportCall);
                }
                // falls through
            case SyntaxKind.NewExpression:
                return checkCallExpression(node as CallExpression, checkMode);
            case SyntaxKind.TaggedTemplateExpression:
                return checkTaggedTemplateExpression(node as TaggedTemplateExpression);
            case SyntaxKind.ParenthesizedExpression:
                return checkParenthesizedExpression(node as ParenthesizedExpression, checkMode);
            case SyntaxKind.ClassExpression:
                return checkClassExpression(node as ClassExpression);
            case SyntaxKind.FunctionExpression:
            case SyntaxKind.ArrowFunction:
                return checkFunctionExpressionOrObjectLiteralMethod(node as FunctionExpression | ArrowFunction, checkMode);
            case SyntaxKind.TypeOfExpression:
                return checkTypeOfExpression(node as TypeOfExpression);
            case SyntaxKind.TypeAssertionExpression:
            case SyntaxKind.AsExpression:
                return checkAssertion(node as AssertionExpression, checkMode);
            case SyntaxKind.NonNullExpression:
                return checkNonNullAssertion(node as NonNullExpression);
            case SyntaxKind.ExpressionWithTypeArguments:
                return checkExpressionWithTypeArguments(node as ExpressionWithTypeArguments);
            case SyntaxKind.SatisfiesExpression:
                return checkSatisfiesExpression(node as SatisfiesExpression);
            case SyntaxKind.MetaProperty:
                return checkMetaProperty(node as MetaProperty);
            case SyntaxKind.DeleteExpression:
                return checkDeleteExpression(node as DeleteExpression);
            case SyntaxKind.VoidExpression:
                return checkVoidExpression(node as VoidExpression);
            case SyntaxKind.AwaitExpression:
                return checkAwaitExpression(node as AwaitExpression);
            case SyntaxKind.PrefixUnaryExpression:
                return checkPrefixUnaryExpression(node as PrefixUnaryExpression);
            case SyntaxKind.PostfixUnaryExpression:
                return checkPostfixUnaryExpression(node as PostfixUnaryExpression);
            case SyntaxKind.BinaryExpression:
                return checkBinaryExpression(node as BinaryExpression, checkMode);
            case SyntaxKind.ConditionalExpression:
                return checkConditionalExpression(node as ConditionalExpression, checkMode);
            case SyntaxKind.SpreadElement:
                return checkSpreadExpression(node as SpreadElement, checkMode);
            case SyntaxKind.OmittedExpression:
                return undefinedWideningType;
            case SyntaxKind.YieldExpression:
                return checkYieldExpression(node as YieldExpression);
            case SyntaxKind.SyntheticExpression:
                return checkSyntheticExpression(node as SyntheticExpression);
            case SyntaxKind.JsxExpression:
                return checkJsxExpression(node as JsxExpression, checkMode);
            case SyntaxKind.JsxElement:
                return checkJsxElement(node as JsxElement, checkMode);
            case SyntaxKind.JsxSelfClosingElement:
                return checkJsxSelfClosingElement(node as JsxSelfClosingElement, checkMode);
            case SyntaxKind.JsxFragment:
                return checkJsxFragment(node as JsxFragment);
            case SyntaxKind.JsxAttributes:
                return checkJsxAttributes(node as JsxAttributes, checkMode);
            case SyntaxKind.JsxOpeningElement:
                Debug.fail("Shouldn't ever directly check a JsxOpeningElement");
        }
    */
    fn check_expression_worker(&mut self, expr: &Expression<'a>, ctx: &CheckContext) -> TypeId {
        // NOTE: TypeScript checks Expressions and MemberExpressions with the
        // same function. We need to split it up.
        // NOTE: in order as it appears in TS' checkExpressionWorker
        match expr {
            Expression::Identifier(_) => todo!("checkIdentifier"),
            // todo: checkPrivateIdentifier
            Expression::ThisExpression(_) => todo!("checkThisExpression"),
            Expression::Super(_) => todo!("checkSuperExpression"),
            Expression::NullLiteral(null) => null.check(self, ctx),
            Expression::StringLiteral(lit) => lit.check(self, ctx),
            Expression::NumericLiteral(lit) => lit.check(self, ctx),
            Expression::BigIntLiteral(lit) => lit.check(self, ctx),

            Expression::BooleanLiteral(lit) => lit.check(self, ctx),
            Expression::TemplateLiteral(lit) => lit.check(self, ctx),
            Expression::RegExpLiteral(lit) => lit.check(self, ctx),
            Expression::ArrayExpression(lit) => lit.check(self, ctx),
            Expression::ObjectExpression(lit) => lit.check(self, ctx),

            Expression::StaticMemberExpression(expr) => expr.check(self, ctx), // SyntaxKind.PropertyAccessExpression
            Expression::ChainExpression(expr) => expr.check(self, ctx), // NOTE: I _think_ tsc coniders these as PropertyAccessExpressions
            // TODO: SyntaxKind.QualifiedName
            Expression::ComputedMemberExpression(expr) => expr.check(self, ctx), // SyntaxKind.ElementAccessExpression
            Expression::CallExpression(expr) => expr.check(self, ctx),
            // NOTE: TS treats `import()` as a CallExpression
            Expression::ImportExpression(expr) => expr.check(self, ctx),
            Expression::NewExpression(expr) => expr.check(self, ctx),
            Expression::TaggedTemplateExpression(expr) => expr.check(self, ctx),
            Expression::ParenthesizedExpression(expr) => expr.check(self, ctx),
            Expression::ClassExpression(expr) => self.check_class_expression(expr.as_ref(), ctx),
            Expression::FunctionExpression(expr) => {
                self.check_function_expression(expr.as_ref(), ctx)
            }
            Expression::ArrowFunctionExpression(expr) => expr.check(self, ctx),
            // TODO: SyntaxKind.TypeOfExpression. I forget what node we use for
            // that
            Expression::TSTypeAssertion(expr) => expr.check(self, ctx),
            Expression::TSAsExpression(expr) => expr.check(self, ctx),
            Expression::TSNonNullExpression(expr) => expr.check(self, ctx),
            // TODO: SyntaxKind.ExpressionWithTypeArguments
            Expression::TSSatisfiesExpression(expr) => expr.check(self, ctx),
            Expression::MetaProperty(expr) => expr.check(self, ctx),
            // TODO: SyntaxKind.DeleteExpression
            // TODO: SyntaxKind.VoidExpression
            Expression::AwaitExpression(expr) => expr.check(self, ctx),
            Expression::UnaryExpression(expr) => expr.check(self, ctx), // SyntaxKind.PrefixUnaryExpression and SyntaxKind.PostfixUnaryExpression

            Expression::BinaryExpression(expr) => expr.check(self, ctx),
            Expression::LogicalExpression(expr) => expr.check(self, ctx), // NOTE: tsc consideres logical exprs to be binary exprs
            Expression::ConditionalExpression(expr) => expr.check(self, ctx),
            // TODO: SyntaxKind.SpreadElement
            // TODO: SyntaxKind.OmittedExpression (???)
            Expression::YieldExpression(expr) => expr.check(self, ctx),
            // TODO: SyntaxKind.SyntheticExpression (???)
            // TODO: SyntaxKind.JSXExpression
            Expression::JSXElement(expr) => expr.check(self, ctx),
            // NOTE: SyntaxKind.JSXSelfClosingElement is part of JSXElement
            Expression::JSXFragment(expr) => expr.check(self, ctx),
            // NOTE: JSXAttributes is not an Expression.
            // NOTE: no partial parsing, so no SyntaxKind.JsxOpeningElement

            // expr => todo!("checkExpressionWorker({expr:?})"),

            // NOTE: I did not see these in checkExpressionWorker, but these are
            // expressions in oxc
            Expression::UpdateExpression(_) => todo!("check_expression_worker(UpdateExpression)"),
            Expression::AssignmentExpression(_) => {
                todo!("check_expression_worker(AssignmentExpression)")
            }
            Expression::SequenceExpression(_) => {
                todo!("check_expression_worker(SequenceExpression)")
            }
            Expression::PrivateInExpression(_) => {
                todo!("check_expression_worker(PrivateInExpression)")
            }
            Expression::PrivateFieldExpression(_) => {
                todo!("check_expression_worker(PrivateFieldExpression)")
            }
            Expression::TSInstantiationExpression(_) => {
                todo!("check_expression_worker(TSInstantiationExpression)")
            }
        }
    }
    /// ```typescript
    /// function checkConstEnumAccess(node: Expression | QualifiedName, type: Type) {
    ///     // enum object type for const enums are only permitted in:
    ///     // - 'left' in property access
    ///     // - 'object' in indexed access
    ///     // - target in rhs of import statement
    ///     const ok = (node.parent.kind === SyntaxKind.PropertyAccessExpression && (node.parent as PropertyAccessExpression).expression === node) ||
    ///         (node.parent.kind === SyntaxKind.ElementAccessExpression && (node.parent as ElementAccessExpression).expression === node) ||
    ///         ((node.kind === SyntaxKind.Identifier || node.kind === SyntaxKind.QualifiedName) && isInRightSideOfImportOrExportAssignment(node as Identifier) ||
    ///             (node.parent.kind === SyntaxKind.TypeQuery && (node.parent as TypeQueryNode).exprName === node)) ||
    ///         (node.parent.kind === SyntaxKind.ExportSpecifier); // We allow reexporting const enums

    ///     if (!ok) {
    ///         error(node, Diagnostics.const_enums_can_only_be_used_in_property_or_index_access_expressions_or_the_right_hand_side_of_an_import_declaration_or_export_assignment_or_type_query);
    ///     }

    ///     // --verbatimModuleSyntax only gets checked here when the enum usage does not
    ///     // resolve to an import, because imports of ambient const enums get checked
    ///     // separately in `checkAliasSymbol`.
    ///     if (
    ///         compilerOptions.isolatedModules
    ///         || compilerOptions.verbatimModuleSyntax
    ///             && ok
    ///             && !resolveName(
    ///                 node,
    ///                 getFirstIdentifier(node as EntityNameOrEntityNameExpression),
    ///                 SymbolFlags.Alias,
    ///                 /*nameNotFoundMessage*/ undefined,
    ///                 /*isUse*/ false,
    ///                 /*excludeGlobals*/ true,
    ///             )
    ///     ) {
    ///         Debug.assert(!!(type.symbol.flags & SymbolFlags.ConstEnum));
    ///         const constEnumDeclaration = type.symbol.valueDeclaration as EnumDeclaration;
    ///         const redirect = host.getRedirectReferenceForResolutionFromSourceOfProject(getSourceFileOfNode(constEnumDeclaration).resolvedPath);
    ///         if (constEnumDeclaration.flags & NodeFlags.Ambient && !isValidTypeOnlyAliasUseSite(node) && (!redirect || !shouldPreserveConstEnums(redirect.commandLine.options))) {
    ///             error(node, Diagnostics.Cannot_access_ambient_const_enums_when_0_is_enabled, isolatedModulesLikeFlagName);
    ///         }
    ///     }
    /// }
    /// ```
    fn check_const_enum_access(
        &mut self,
        expr: &Expression<'a>,
        type_id: TypeId,
        ctx: &CheckContext,
    ) {
        // TODO
    }
}

impl<'a> Check<'a> for NullLiteral {
    #[inline]
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        checker.intrinsics.null_widening
    }
}

impl<'a> Check<'a> for StringLiteral<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // return hasSkipDirectInferenceFlag(node) ?
        //     blockedStringType :
        //     getFreshTypeOfLiteralType(getStringLiteralType((node as
        //     StringLiteralLike).text));
        // TODO: no-substitution template literals
        checker.get_fresh_type_of_literal_type(checker.get_string_literal_type(&self.value))
    }
}

impl<'a> Check<'a> for NumericLiteral<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // checkGrammarNumericLiteral(node as NumericLiteral);
        // return getFreshTypeOfLiteralType(getNumberLiteralType(+(node as NumericLiteral).text));

        checker.get_fresh_type_of_literal_type(checker.get_number_literal_type(self.value))
    }
}

impl<'a> Check<'a> for BigIntLiteral<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // checkGrammarBigIntLiteral(node as BigIntLiteral);
        // return getFreshTypeOfLiteralType(getBigIntLiteralType({
        //     negative: false,
        //     base10Value: parsePseudoBigInt((node as BigIntLiteral).text),
        // }));
        // TODO: avoid clone
        let big_int = PseudoBigInt { raw: self.raw.clone(), base: self.base };
        checker.get_fresh_type_of_literal_type(checker.get_big_int_literal_type(&big_int))
    }
}

impl<'a> Check<'a> for BooleanLiteral {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // case SyntaxKind.TrueKeyword:
        //     return trueType;
        // case SyntaxKind.FalseKeyword:
        //     return falseType;
        if self.value {
            checker.intrinsics.true_type
        } else {
            checker.intrinsics.false_type
        }
    }
}

impl<'a> Check<'a> for TemplateLiteral<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // zero-sub template exprs are checked identically to strings
        if self.is_no_substitution_template() {
            // return hasSkipDirectInferenceFlag(node) ?
            //     blockedStringType :
            //     getFreshTypeOfLiteralType(getStringLiteralType((node as
            //     StringLiteralLike).text));
            // TODO: no-substitution template literals
            return checker.get_fresh_type_of_literal_type(
                checker.get_string_literal_type(&self.quasi().unwrap()),
            );
        }
        todo!("checkTemplateExpression");
        // function checkTemplateExpression(node: TemplateExpression): Type {
        //     const texts = [node.head.text];
        //     const types = [];
        //     for (const span of node.templateSpans) {
        //         const type = checkExpression(span.expression);
        //         if (maybeTypeOfKindConsideringBaseConstraint(type, TypeFlags.ESSymbolLike)) {
        //             error(span.expression, Diagnostics.Implicit_conversion_of_a_symbol_to_a_string_will_fail_at_runtime_Consider_wrapping_this_expression_in_String);
        //         }
        //         texts.push(span.literal.text);
        //         types.push(isTypeAssignableTo(type, templateConstraintType) ? type : stringType);
        //     }
        //     const evaluated = node.parent.kind !== SyntaxKind.TaggedTemplateExpression && evaluate(node).value;
        //     if (evaluated) {
        //         return getFreshTypeOfLiteralType(getStringLiteralType(evaluated));
        //     }
        //     if (isConstContext(node) || isTemplateLiteralContext(node) || someType(getContextualType(node, /*contextFlags*/ undefined) || unknownType, isTemplateLiteralContextualType)) {
        //         return getTemplateLiteralType(texts, types);
        //     }
        //     return stringType;
        // }
    }
}

impl<'a> Check<'a> for RegExpLiteral<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkRegularExpressionLiteral")
    }
}

impl<'a> Check<'a> for ArrayExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkArrayLiteral")
    }
}

impl<'a> Check<'a> for ObjectExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkObjectLiteral")
    }
}

impl<'a> Check<'a> for StaticMemberExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkPropertyAccessExpression")
    }
}

impl<'a> Check<'a> for ChainExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkPropertyAccessExpression")
    }
}

// TODO: QualifiedName

impl<'a> Check<'a> for ComputedMemberExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkIndexedAccess")
    }
}

impl<'a> Check<'a> for CallExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkCallExpression")
    }
}

impl<'a> Check<'a> for ImportExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkImportCallExpression")
    }
}

impl<'a> Check<'a> for NewExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // NOTE: NewExpression gets cast to CallExpression. We can't do that.
        todo!("checkCallExpression")
    }
}

impl<'a> Check<'a> for TaggedTemplateExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // NOTE: NewExpression gets cast to CallExpression. We can't do that.
        todo!("checkTaggedTemplateExpression")
    }
}

impl<'a> Check<'a> for ParenthesizedExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        // NOTE: NewExpression gets cast to CallExpression. We can't do that.
        todo!("checkParenthesizedExpression")
    }
}

impl<'a> Checker<'a> {
    fn check_class_expression(&mut self, expr: &Class<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkClassExpression")
    }

    fn check_function_expression(&mut self, expr: &Function<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkFunctionExpressionOrObjectLiteralMethod")
    }
}

impl<'a> Check<'a> for ArrowFunctionExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkFunctionExpressionOrObjectLiteralMethod")
    }
}

// TODO: TypeOfExpression
impl<'a> Check<'a> for TSTypeAssertion<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkAssertion")
    }
}

impl<'a> Check<'a> for TSAsExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkAssertion")
    }
}

impl<'a> Check<'a> for TSNonNullExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkNonNullAssertion")
    }
}

// TODO: ExpressionWithTypeArguments

impl<'a> Check<'a> for TSSatisfiesExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkSatisfiesExpression")
    }
}

impl<'a> Check<'a> for MetaProperty<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkMetaProperty")
    }
}

impl<'a> Check<'a> for AwaitExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkAwaitExpression")
    }
}

// TODO: DeleteExpression
// TODO: VoidExpression

impl<'a> Check<'a> for UnaryExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkPrefixUnaryExpression, checkPostfixUnaryExpression")
    }
}

impl<'a> Check<'a> for BinaryExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkBinaryExpression")
    }
}

impl<'a> Check<'a> for LogicalExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkBinaryExpression")
    }
}

impl<'a> Check<'a> for ConditionalExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkConditionalExpression")
    }
}

impl<'a> Check<'a> for SpreadElement<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkSpreadExpression")
    }
}

// TODO: OmittedExpression

impl<'a> Check<'a> for YieldExpression<'a> {
    fn check(&self, checker: &mut Checker<'a>, ctx: &CheckContext) -> TypeId {
        todo!("checkYieldExpression")
    }
}

// NOTE: jsx check impls are in jsx.rs
